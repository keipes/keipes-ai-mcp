# Storage Implementation v3-Simple - Minimal Scope, Type-Erased

```
trait IntoKeyBytes<'a>
    into_key_bytes(self, arena) -> Cow<'a, [u8]>

impl IntoKeyBytes<'a> for &'a str
    into_key_bytes(self, _arena) -> Cow::Borrowed(self.as_bytes())

impl IntoKeyBytes<'a> for u64
    into_key_bytes(self, arena) ->
        start = arena.len()
        arena.extend_from_slice(&self.to_le_bytes())
        Cow::Borrowed(&arena[start..start+8])

mod storage
    Database
        backend: RedbBackend
        arena: BufferArena
        
        new(path) -> Result<Self>
        
        guarded<F, R>(operation) -> Result<R>
            mut guard = self.arena.acquire(&self.backend)
            match operation(guard) {
                Ok(result) => {
                    guard.commit()?
                    Ok(result)
                }
                Err(e) => {
                    Err(e)
                }
            }

    BufferArena
        pool: Vec<Vec<u8>>
        
        acquire(backend) -> WriteGuard
            key_arena = self.pool.pop().unwrap_or_default()
            WriteGuard::new(key_arena, self, backend)

    WriteGuard<'arena>
        key_arena: Option<Vec<u8>>
        arena: &'arena BufferArena
        transaction: WriteTransaction
        committed: bool
        
        new(key_arena, arena, backend) -> Self
            transaction = backend.begin_write_transaction()
            WriteGuard { 
                key_arena: Some(key_arena), 
                arena, 
                transaction, 
                committed: false 
            }
        
        put_raw(table_name, key_bytes, value_bytes) -> Result<()>
            table_handle = self.transaction.open_table(table_name)
            table_handle.put(key_bytes, value_bytes)
        
        delete_raw(table_name, key_bytes) -> Result<()>
            table_handle = self.transaction.open_table(table_name)
            table_handle.delete(key_bytes)
        
        put<K, V, S>(table_name, key, value, serializer) -> Result<()>
            key_bytes = key.into_key_bytes(self.key_arena.as_mut().unwrap())
            value_bytes = serializer.serialize(value, self.key_arena.as_mut().unwrap())
            self.put_raw(table_name, key_bytes.as_ref(), value_bytes.as_ref())
        
        delete<K>(table_name, key) -> Result<()>
            key_bytes = key.into_key_bytes(self.key_arena.as_mut().unwrap())
            self.delete_raw(table_name, key_bytes.as_ref())
        
        commit(&mut self) -> Result<()>
            self.transaction.commit()?
            self.committed = true
            Ok(())
        
        drop(self)
            if let Some(key_arena) = self.key_arena.take()
                key_arena.clear()
                self.arena.pool.push(key_arena)
            if !self.committed
                self.transaction.rollback()

    trait Serializer<T>
        type Output<'a>
        serialize(value, arena) -> Cow<[u8]>
        deserialize<'a>(bytes) -> Self::Output<'a>

    RkyvSerializer
        impl<T> Serializer<T> for RkyvSerializer
        where T: rkyv::Archive + rkyv::Serialize<AllocSerializer<256>>
            type Output<'a> = &'a T::Archived
            
            serialize(value, arena) -> Cow<[u8]>
                let start = arena.len()
                let bytes = rkyv::to_bytes(value).unwrap()
                arena.extend_from_slice(&bytes)
                Cow::Borrowed(&arena[start..])
            
            deserialize<'a>(bytes) -> &'a T::Archived
                rkyv::access_unchecked(bytes)

    FlatbuffersSerializer
        impl<T> Serializer<T> for FlatbuffersSerializer
        where T: flatbuffers::Follow<'static> + 'static
            type Output<'a> = T::Inner
            
            serialize(value, _arena) -> Cow<[u8]>
                builder = FlatBufferBuilder::new()
                root = T::create(&mut builder, value)
                builder.finish(root, None)
                Cow::Owned(builder.finished_data().to_vec())
            
            deserialize<'a>(bytes) -> T::Inner
                flatbuffers::root::<T>(bytes).unwrap()

    RawSerializer
        impl Serializer<&[u8]> for RawSerializer
            type Output<'a> = &'a [u8]
            
            serialize(value, _arena) -> Cow<[u8]>
                Cow::Borrowed(value)
            
            deserialize<'a>(bytes) -> &'a [u8]
                bytes

    Table<K, V, S>
        database: &Database
        name: &'static str
        serializer: S
        _phantom: PhantomData<(K, V)>
        
        put(key, value) -> Result<()>
            self.database.guarded(|guard| {
                guard.put(self.name, key, value, &self.serializer)
            })
        
        get<F, R>(key, accessor) -> Result<Option<R>>
            let txn = self.database.backend.begin_read_transaction()
            let table_handle = txn.open_table(self.name)
            
            let mut temp_arena = Vec::new()
            let key_bytes = key.into_key_bytes(&mut temp_arena)
            
            table_handle.get(key_bytes.as_ref(), |bytes| {
                let value = self.serializer.deserialize(bytes)
                accessor(value)
            })
        
        delete(key) -> Result<bool>
            self.database.guarded(|guard| {
                guard.delete(self.name, key)
            })
        
        scan<F, R>(scanner) -> Result<Vec<R>>
            let results = Vec::new()
            let txn = self.database.backend.begin_read_transaction()
            let table_handle = txn.open_table(self.name)
            
            table_handle.iter(|key_bytes, value_bytes| {
                let key = K::from_key_bytes(key_bytes)
                let value = self.serializer.deserialize(value_bytes)
                if let Some(result) = scanner(&key, &value) {
                    results.push(result)
                }
                true
            })?
            
            Ok(results)
        
        put_all<I>(items) -> Result<()>
            self.database.guarded(|guard| {
                for (key, value) in items {
                    guard.put(self.name, key, value, &self.serializer)?
                }
                Ok(())
            })
        
        batch_write<F, R>(operation) -> Result<R>
            self.database.guarded(|guard| {
                operation(guard, self.name, &self.serializer)
            })

    impl Database
        rkyv_table<K, V>(name) -> Table<K, V, RkyvSerializer>
            Table::new(self, name, RkyvSerializer)
        
        flatbuffers_table<K, V>(name) -> Table<K, V, FlatbuffersSerializer>
            Table::new(self, name, FlatbuffersSerializer)
        
        raw_table<K>(name) -> Table<K, &[u8], RawSerializer>
            Table::new(self, name, RawSerializer)
        
        table_with_serializer<K, V, S>(name, serializer) -> Table<K, V, S>
            Table::new(self, name, serializer)

usage_with_serializers()
    database = Database::new("app.redb")
    
    users = database.rkyv_table::<u64, User>("users")
    monsters = database.flatbuffers_table::<u64, Monster>("monsters")
    cache = database.raw_table::<&str>("cache")
    
    users.put(123, &user)?
    monsters.put(456, &monster)?
    cache.put("session:abc", b"session_data")?
    
    user_name = users.get(123, |archived| {
        archived.name.as_str().to_string()
    })?
    
    monster_hp = monsters.get(456, |monster| {
        monster.hp()
    })?
    
    session_data = cache.get("session:abc", |bytes| {
        bytes.to_vec()
    })?
    
    users.delete(123)?
    
    active_users = users.scan(|id, user| {
        if user.active.to_native() {
            Some(*id)
        } else {
            None
        }
    })?
    
    user_data = vec![(1, user1), (2, user2), (3, user3)]
    users.put_all(user_data)?
    
    database.guarded(|guard| {
        guard.put("users", user.id, &user, &rkyv_serializer)?
        guard.delete("users", old_user_id)?
        guard.put("monsters", monster.id, &monster, &flatbuffers_serializer)?
    })?

serialization_benefits()
    users = database.rkyv_table::<u64, User>("users")
    user_name = users.get(123, |archived| archived.name.as_str())
    
    monsters = database.flatbuffers_table::<u64, Monster>("monsters") 
    hp = monsters.get(456, |monster| monster.hp())
    
    cache = database.raw_table::<&str>("cache")
    data = cache.get("key", |bytes| bytes.to_vec())
```
