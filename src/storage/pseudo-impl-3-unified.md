# Storage Implementation v3-Unified - Single Typed API

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
        
        // Single unified API - always typed
        table<K, V>(name) -> Table<K, V>
        
        // Batch operations
        with_batch<F, R>(operation) -> Result<R>
            guard = self.arena.acquire()
            result = operation(BatchContext::new(guard))
            Ok(result)

    Table<K, V>
        database: &Database
        name: &'static str
        _phantom: PhantomData<(K, V)>
        
        put(key, value) -> Result<()>
            key_bytes = self.serialize_key(key)
            value_bytes = self.serialize_value(value)
            self.database.backend.put_bytes(self.name, &key_bytes, value_bytes)
        
        get<F, R>(key, accessor) -> Result<Option<R>>
            key_bytes = self.serialize_key(key)
            self.database.backend.get_bytes(self.name, &key_bytes, |bytes| {
                value = self.deserialize_value(bytes)
                accessor(value)
            })
        
        delete(key) -> Result<bool>
            key_bytes = self.serialize_key(key)
            self.database.backend.delete_bytes(self.name, &key_bytes)
        
        scan<F, R>(scanner) -> Result<Vec<R>>
            results = Vec::new()
            self.database.backend.scan_table(self.name, |key_bytes, value_bytes| {
                key = self.deserialize_key(key_bytes)
                value = self.deserialize_value(value_bytes)
                if let Some(result) = scanner(&key, value)
                    results.push(result)
                true
            })
            Ok(results)
        
        batch_write<F, R>(operation) -> Result<R>
            self.database.with_batch(|ctx| {
                let table_guard = ctx.table(self.name)
                operation(table_guard)
            })
        
        // Serialization methods - specialized by type
        serialize_key(key) -> Vec<u8>
            match K {
                u64 => key.to_le_bytes().to_vec(),
                &str => key.as_bytes().to_vec(),
                &[u8] => key.to_vec(),                    // Raw bytes!
                String => key.into_bytes(),
                _ => rkyv::to_bytes(key).unwrap().to_vec() // Generic fallback
            }
        
        serialize_value(value) -> Vec<u8>
            match V {
                &[u8] => value.to_vec(),                  // Raw bytes!
                Vec<u8> => value,                         // Already bytes
                _ => rkyv::to_bytes(value).unwrap().to_vec() // rkyv serialization
            }
        
        deserialize_key(bytes) -> K
            match K {
                u64 => u64::from_le_bytes(bytes.try_into().unwrap()),
                String => String::from_utf8(bytes.to_vec()).unwrap(),
                &[u8] => bytes,                           // Zero-copy!
                _ => rkyv::from_bytes(bytes).unwrap()     // Generic fallback
            }
        
        deserialize_value(bytes) -> V::Output
            match V {
                &[u8] => bytes,                           // Zero-copy raw bytes!
                _ => rkyv::access_unchecked::<V::Archived>(bytes) // Zero-copy rkyv
            }

    BatchContext<'guard>
        guard: &'guard mut BufferGuard<'guard>
        
        table<K, V>(name) -> TableGuard<'guard, K, V>
            TableGuard::new(self.guard, name)

    TableGuard<'guard, K, V>
        guard: &'guard mut BufferGuard<'guard>
        name: &'static str
        _phantom: PhantomData<(K, V)>
        
        put(key, value) -> Result<()>
            key_bytes = Self::serialize_key(key)
            value_bytes = Self::serialize_value(value)
            self.guard.put_raw(self.name, key_bytes, value_bytes)
        
        delete(key) -> Result<()>
            key_bytes = Self::serialize_key(key)
            self.guard.delete_raw(self.name, key_bytes)

usage_unified()
    database = Database::new("app.redb")
    
    // Typed tables - same API for everything
    users = database.table::<u64, User>("users")                    // Structured data
    posts = database.table::<String, Post>("posts")                 // String keys
    cache = database.table::<&str, &[u8]>("cache")                 // Raw byte values
    blobs = database.table::<&[u8], &[u8]>("blobs")               // Raw key+value
    
    // All use the same API
    users.put(123, &user)?
    posts.put("my-post", &post)?
    cache.put("session:abc", b"session_data")?                     // Raw bytes
    blobs.put(b"binary_key", b"binary_data")?                      // Both raw
    
    // Zero-copy access for raw bytes
    session_data = cache.get("session:abc", |bytes| bytes.to_vec())?  // bytes is &[u8]
    blob_data = blobs.get(b"binary_key", |bytes| bytes.to_vec())?     // Zero-copy
    
    // Structured access for rkyv types
    user_name = users.get(123, |archived| archived.name.as_str().to_string())?
    
    // Batch operations - same API
    database.with_batch(|ctx| {
        let mut user_table = ctx.table::<u64, User>("users")
        let mut cache_table = ctx.table::<&str, &[u8]>("cache")
        
        user_table.put(user.id, &user)?
        cache_table.put("key", b"value")?
    })?

tower_service_unified()
    TowerService
        database: Database
        users: Table<u64, User>              // Structured
        sessions: Table<&str, &[u8]>         // Raw values  
        files: Table<&[u8], &[u8]>          // Raw everything
        
        handle_request(&self, req) -> Response
            match req.action {
                "get_user" => {
                    user = self.users.get(req.user_id, |archived| {
                        UserResponse::from_archived(archived)
                    })?
                    Response::json(user)
                }
                
                "store_file" => {
                    self.files.put(&req.file_hash, &req.file_data)?  // Raw binary
                    Response::ok()
                }
                
                "cache_session" => {
                    self.sessions.put(&req.session_id, &req.session_data)?  // Raw bytes
                    Response::ok()
                }
                
                "bulk_update" => {
                    self.database.with_batch(|ctx| {
                        let mut users = ctx.table::<u64, User>("users")
                        let mut sessions = ctx.table::<&str, &[u8]>("sessions")
                        
                        for update in req.updates {
                            users.put(update.user.id, &update.user)?
                            sessions.put(&update.session_id, &update.session_data)?
                        }
                    })?
                    Response::ok()
                }
            }

key_benefits()
    // Single API surface
    - Only one way to create tables: database.table::<K, V>()
    - Only one way to access data: table.get(key, |value| ...)
    - Only one way to batch: database.with_batch(|ctx| ...)
    
    // Type system handles everything
    - Table<u64, User> → rkyv serialization automatically
    - Table<&str, &[u8]> → raw byte access automatically  
    - Table<&[u8], &[u8]> → fully raw access automatically
    
    // Zero learning curve
    - Same patterns for structured and raw data
    - Type annotations determine behavior
    - No separate APIs to learn
    
    // Maximum flexibility
    - Want rkyv? Use Table<K, StructType>
    - Want raw bytes? Use Table<K, &[u8]>  
    - Want mixed? Use different tables
    - Want custom serialization? Implement traits
```
