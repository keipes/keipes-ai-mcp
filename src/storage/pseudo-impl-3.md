# Storage Implementation v3 - Hierarchical Lifetime Management

```
trait IntoKeyBytes<'a>
    into_key_bytes(self, arena) -> Cow<'a, [u8]>

impl IntoKeyBytes<'a> for &'a str
    into_key_bytes(self, _arena) -> Cow::Borrowed(self.as_bytes())

impl IntoKeyBytes<'a> for &'a [u8]
    into_key_bytes(self, _arena) -> Cow::Borrowed(self)

impl IntoKeyBytes<'a> for u64
    into_key_bytes(self, arena) ->
        start = arena.len()
        arena.extend_from_slice(&self.to_le_bytes())
        Cow::Borrowed(&arena[start..start+8])

impl IntoKeyBytes<'a> for String
    into_key_bytes(self, _arena) -> Cow::Owned(self.into_bytes())

mod storage
    Database
        backend: RedbBackend
        arena: BufferArena
        
        new(path) -> Result<Self>
            backend = RedbBackend::new(path)
            arena = BufferArena::new()
            Ok(Database { backend, arena })
        
        table<K, V>(name) -> Table<K, V>
            Table::new(self, name)
        
        with_buffer<F, R>(table_name, operation) -> Result<R>
            guard = self.arena.acquire(table_name)
            result = operation(guard)
            self.arena.release(guard)
            result
        
        scoped_operations<F, R>(operation) -> Result<R>
            scope = ArenaScope::new(&mut self.arena)
            operation(scope)

    BufferArena
        pool: Vec<WriteBatch>
        max_size: usize
        
        new() -> Self
            BufferArena { pool: Vec::new(), max_size: 16 }
        
        acquire(table_name) -> BufferGuard
            batch = self.pool.pop().unwrap_or_else(|| WriteBatch::new(table_name))
            BufferGuard::new(batch, self)
        
        release(batch: WriteBatch)
            batch.reset()
            if self.pool.len() < self.max_size
                self.pool.push(batch)

    BufferGuard<'arena>
        batch: Option<WriteBatch>
        arena: &'arena BufferArena
        
        new(batch, arena) -> Self
            BufferGuard { batch: Some(batch), arena }
        
        put<K, V>(key, value) -> Result<()>
            key_bytes = key.into_key_bytes(&mut self.batch.key_arena)
            value_bytes = rkyv::to_bytes(value)
            self.batch.buffer.push((key_bytes, value_bytes))
        
        put_raw<K>(key, value_bytes) -> Result<()>
            key_bytes = key.into_key_bytes(&mut self.batch.key_arena)
            self.batch.buffer.push((key_bytes, value_bytes))
        
        commit(&mut self) -> Result<()>
            self.batch.commit_internal()
            self.batch.reset()
        
        drop(self)
            if let Some(batch) = self.batch.take()
                self.arena.release(batch)

    ArenaScope<'scope>
        arena: &'scope mut BufferArena
        active_guards: Vec<BufferGuard<'scope>>
        
        new(arena) -> Self
            ArenaScope { arena, active_guards: Vec::new() }
        
        table<K, V>(name) -> ScopedTable<'scope, K, V>
            ScopedTable::new(self.arena, name)
        
        buffer(table_name) -> BufferGuard<'scope>
            guard = self.arena.acquire(table_name)
            self.active_guards.push(guard)
            guard
        
        drop(self)
            for guard in self.active_guards
                self.arena.release(guard.take_batch())

    ScopedTable<'scope, K, V>
        arena: &'scope BufferArena
        name: &'static str
        _phantom: PhantomData<(K, V)>
        
        new(arena, name) -> Self
            ScopedTable { arena, name, _phantom: PhantomData }
        
        batch_write<F, R>(operation) -> Result<R>
            guard = self.arena.acquire(self.name)
            operation(guard)

    Table<K, V>
        database: &Database
        name: &'static str
        _phantom: PhantomData<(K, V)>
        
        new(database, name) -> Self
            Table { database, name, _phantom: PhantomData }
        
        put(key, value) -> Result<()>
            self.database.with_buffer(self.name, |mut guard| {
                guard.put(key, value)
            })
        
        put_batch<I>(items) -> Result<()>
            self.database.with_buffer(self.name, |mut guard| {
                for (key, value) in items
                    guard.put(key, value)
                guard.commit()
            })
        
        get<F, R>(key, accessor) -> Result<Option<R>>
            key_bytes = key.into_key_bytes(&mut temp_arena)
            self.database.backend.get(self.name, &key_bytes, accessor)
        
        scan<F, R>(scanner) -> Result<Vec<R>>
            results = Vec::new()
            self.database.backend.scan(self.name, |key_bytes, value_bytes| {
                key = K::from_bytes(key_bytes)
                archived = rkyv::access_unchecked(value_bytes)
                if let Some(result) = scanner(&key, archived)
                    results.push(result)
                true
            })
            Ok(results)

    WriteBatch
        table_name: String
        key_arena: Vec<u8>
        buffer: Vec<(Vec<u8>, Vec<u8>)>
        
        new(table_name) -> Self
            WriteBatch {
                table_name: table_name.to_string(),
                key_arena: Vec::new(),
                buffer: Vec::new(),
            }
        
        reset(&mut self)
            self.key_arena.clear()
            self.buffer.clear()
        
        commit_internal(&self, backend) -> Result<()>
            items = self.buffer.iter().map(|(k, v)| (Cow::Borrowed(k.as_slice()), v.clone()))
            backend.put_batch(&self.table_name, items)

mod backend
    RedbBackend
        db: Database
        
        new(path) -> Result<Self>
            db = Database::create(path)
            Ok(RedbBackend { db })
        
        get<F, R>(table_name, key, accessor) -> Result<Option<R>>
            txn = db.begin_read()
            tbl = txn.open_table(table_name)
            if let Some(val) = tbl.get(key)
                return Ok(Some(accessor(val.value())))
            Ok(None)
        
        put_batch<I>(table_name, items) -> Result<()>
            txn = db.begin_write()
            tbl = txn.open_table(table_name)
            for (key, value) in items
                tbl.insert(key, value)
            txn.commit()
        
        delete(table_name, key) -> Result<bool>
            txn = db.begin_write()
            tbl = txn.open_table(table_name)
            existed = tbl.remove(key).is_some()
            txn.commit()
            Ok(existed)
        
        scan<F>(table_name, callback) -> Result<()>
            txn = db.begin_read()
            tbl = txn.open_table(table_name)
            for (key, value) in tbl.iter()
                if !callback(key.value(), value.value())
                    break
            Ok(())

usage_patterns()
    // Service-level database ownership
    database = Database::new("app.redb")
    
    // Simple table operations
    users = database.table::<u64, User>("users")
    users.put(123, &user)
    user_data = users.get(123, |archived| archived.name.as_str())
    
    // Scoped batch operations with automatic cleanup
    database.scoped_operations(|mut scope| {
        users = scope.table::<u64, User>("users")
        posts = scope.table::<u64, Post>("posts")
        
        // Both tables share the same arena scope
        users.batch_write(|mut guard| {
            guard.put(user1.id, &user1)
            guard.put(user2.id, &user2)
            guard.commit()
        })
        
        posts.batch_write(|mut guard| {
            guard.put(post1.id, &post1)
            guard.put(post2.id, &post2)
            guard.commit()
        })
    })  // All buffers automatically returned to pool
    
    // Manual buffer management for fine control
    database.with_buffer("users", |mut guard| {
        guard.put(123u64, &user)
        guard.put("alice", &user)
        guard.commit()
    })
    
    // Tower service integration
    TowerService
        database: Database
        
        handle_request(&self, req) -> Response
            self.database.scoped_operations(|mut scope| {
                match req.path {
                    "/users" => {
                        users = scope.table::<u64, User>("users")
                        user = users.get(req.user_id, |archived| {
                            UserResponse {
                                name: archived.name.as_str().to_string(),
                                email: archived.email.as_str().to_string(),
                            }
                        })
                        Response::json(user)
                    }
                    "/bulk_update" => {
                        users = scope.table::<u64, User>("users")
                        users.batch_write(|mut guard| {
                            for update in req.updates
                                guard.put(update.id, &update.user)
                            guard.commit()
                        })
                        Response::ok()
                    }
                }
            })

lifetime_hierarchy()
    // Ownership: TowerService -> Database -> Arena -> Guards -> Buffers
    // Lifetimes: Database('static) -> Arena('db) -> Guards('arena) -> KeyBytes('guard)
    
    TowerService {
        database: Database,  // Owns database for service lifetime
    }
    
    Database {
        backend: RedbBackend,     // Owned
        arena: BufferArena,       // Owned, lifetime = database lifetime
    }
    
    BufferGuard<'arena> {
        batch: WriteBatch,        // Lifetime tied to arena
        arena: &'arena Arena,     // Borrows arena
    }
    
    WriteBatch {
        key_arena: Vec<u8>,       // Arena for key bytes
        buffer: Vec<(Vec<u8>, Vec<u8>)>,  // Owned data
    }
    
    // Guarantee: Buffers never outlive database
    // Benefit: Safe to pool and reuse without escape analysis
    // Performance: Zero allocation in steady state
```
