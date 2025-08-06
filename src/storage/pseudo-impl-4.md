# Storage Implementation v4 - Direct Transactions

```
trait IntoKeyBytes<'a>
    into_key_bytes(self, arena) -> Cow<'a, [u8]>

trait FromKeyBytes
    fn from_key_bytes(bytes: &[u8], arena: &mut Vec<u8>) -> Self

impl IntoKeyBytes<'a> for &'a str
    into_key_bytes(self, _arena) -> Cow::Borrowed(self.as_bytes())

impl FromKeyBytes for String
    fn from_key_bytes(bytes: &[u8], _arena: &mut Vec<u8>) -> Self
        String::from_utf8(bytes.to_vec()).unwrap()

impl IntoKeyBytes<'a> for &'a [u8]
    into_key_bytes(self, _arena) -> Cow::Borrowed(self)

impl FromKeyBytes for Vec<u8>
    fn from_key_bytes(bytes: &[u8], _arena: &mut Vec<u8>) -> Self
        bytes.to_vec()

impl IntoKeyBytes<'a> for String
    into_key_bytes(self, _arena) -> Cow::Owned(self.into_bytes())

impl IntoKeyBytes<'a> for u64
    into_key_bytes(self, arena) ->
        start = arena.len()
        arena.extend_from_slice(&self.to_le_bytes())
        Cow::Borrowed(&arena[start..start+8])

impl FromKeyBytes for u64
    fn from_key_bytes(bytes: &[u8], _arena: &mut Vec<u8>) -> Self
        u64::from_le_bytes(bytes.try_into().unwrap())

mod storage
    #[derive(Debug)]
    enum StorageError
        Backend(String)
        Serialization(String)
        Transaction(String)
        KeyDeserialization(String)

    trait Backend
        fn get<F, R>(&self, table: &str, key: &[u8], accessor: F) -> Result<Option<R>, StorageError>
            where F: FnOnce(&[u8]) -> R
        fn put(&self, table: &str, key: &[u8], value: Vec<u8>) -> Result<(), StorageError>
        fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>
        fn scan<F>(&self, table: &str, callback: F) -> Result<(), StorageError>
            where F: FnMut(&[u8], &[u8]) -> bool
        fn begin_write_transaction(&self) -> WriteTransaction
        fn begin_read_transaction(&self) -> ReadTransaction

    Database
        backend: Box<dyn Backend>
        
        new(path) -> Result<Self, StorageError>
            backend = Box::new(RedbBackend::new(path)?)
            Ok(Database { backend })
        
        write_transaction(&self) -> WriteTransaction
            self.backend.begin_write_transaction()
        
        read_transaction(&self) -> ReadTransaction
            self.backend.begin_read_transaction()

    WriteTransaction
        put_raw(table_name, key_bytes, value_bytes) -> Result<(), StorageError>
            table_handle = self.open_table(table_name)
            table_handle.put(key_bytes, value_bytes)
        
        delete_raw(table_name, key_bytes) -> Result<bool, StorageError>
            table_handle = self.open_table(table_name)
            let existed = table_handle.remove(key_bytes).is_some()
            Ok(existed)
        
        put<K, V, S>(table_name, key, value, serializer, arena) -> Result<(), StorageError>
            key_bytes = key.into_key_bytes(arena)
            value_bytes = serializer.serialize(value, arena)
            self.put_raw(table_name, key_bytes.as_ref(), value_bytes.as_ref())
        
        delete<K>(table_name, key, arena) -> Result<bool, StorageError>
            key_bytes = key.into_key_bytes(arena)
            self.delete_raw(table_name, key_bytes.as_ref())
        
        commit(self) -> Result<(), StorageError>
        rollback(self) -> Result<(), StorageError>

    ReadTransaction
        get_raw<F, R>(table_name, key_bytes, accessor) -> Result<Option<R>, StorageError>
            table_handle = self.open_table(table_name)
            table_handle.get(key_bytes, accessor)
        
        get<K, V, S, F, R>(table_name, key, serializer, arena, accessor) -> Result<Option<R>, StorageError>
            key_bytes = key.into_key_bytes(arena)
            self.get_raw(table_name, key_bytes.as_ref(), |bytes| {
                value = serializer.deserialize(bytes)
                accessor(value)
            })
        
        scan_raw<F>(table_name, callback) -> Result<(), StorageError>
            table_handle = self.open_table(table_name)
            table_handle.iter(callback)
        
        scan<K, V, S, F, R>(table_name, serializer, arena, scanner) -> Result<Vec<R>, StorageError>
            let mut results = Vec::new()
            table_handle = self.open_table(table_name)
            table_handle.iter(|key_bytes, value_bytes| {
                arena.clear()
                let key = K::from_key_bytes(key_bytes, arena)
                let value = serializer.deserialize(value_bytes)
                if let Some(result) = scanner(&key, &value) {
                    results.push(result)
                }
                true
            })?
            Ok(results)

    trait Serializer<T>
        type Output<'a>
        serialize(value, arena) -> Result<Cow<[u8]>, StorageError>
        deserialize<'a>(bytes) -> Result<Self::Output<'a>, StorageError>

    RkyvSerializer
        impl<T> Serializer<T> for RkyvSerializer
        where T: rkyv::Archive + rkyv::Serialize<AllocSerializer<256>>
            type Output<'a> = &'a T::Archived
            
            serialize(value, arena) -> Result<Cow<[u8]>, StorageError>
                let start = arena.len()
                let bytes = rkyv::to_bytes(value)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?
                arena.extend_from_slice(&bytes)
                Ok(Cow::Borrowed(&arena[start..]))
            
            deserialize<'a>(bytes) -> Result<&'a T::Archived, StorageError>
                rkyv::access_unchecked(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))

    FlatbuffersSerializer
        impl<T> Serializer<T> for FlatbuffersSerializer
        where T: flatbuffers::Follow<'static> + 'static
            type Output<'a> = T::Inner
            
            serialize(value, _arena) -> Result<Cow<[u8]>, StorageError>
                let mut builder = FlatBufferBuilder::new()
                let root = T::create(&mut builder, value)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?
                builder.finish(root, None)
                Ok(Cow::Owned(builder.finished_data().to_vec()))
            
            deserialize<'a>(bytes) -> Result<T::Inner, StorageError>
                flatbuffers::root::<T>(bytes)
                    .map_err(|e| StorageError::Serialization(e.to_string()))

    RawSerializer
        impl Serializer<&[u8]> for RawSerializer
            type Output<'a> = &'a [u8]
            
            serialize(value, _arena) -> Result<Cow<[u8]>, StorageError>
                Ok(Cow::Borrowed(value))
            
            deserialize<'a>(bytes) -> Result<&'a [u8], StorageError>
                Ok(bytes)

    Table<K, V, S>
        database: &Database
        name: &'static str
        serializer: S
        _phantom: PhantomData<(K, V)>
        
        put(key, value) -> Result<(), StorageError>
            let mut arena = Vec::new()
            let txn = self.database.write_transaction()
            txn.put(self.name, key, value, &self.serializer, &mut arena)?
            txn.commit()
        
        get<F, R>(key, accessor) -> Result<Option<R>, StorageError>
            let mut arena = Vec::new()
            let txn = self.database.read_transaction()
            txn.get(self.name, key, &self.serializer, &mut arena, accessor)
        
        delete(key) -> Result<bool, StorageError>
            let mut arena = Vec::new()
            let txn = self.database.write_transaction()
            let existed = txn.delete(self.name, key, &mut arena)?
            txn.commit()?
            Ok(existed)
        
        scan<F, R>(scanner) -> Result<Vec<R>, StorageError>
            let mut arena = Vec::new()
            let txn = self.database.read_transaction()
            txn.scan(self.name, &self.serializer, &mut arena, scanner)
        
        put_all<I>(items) -> Result<(), StorageError>
            let mut arena = Vec::new()
            let txn = self.database.write_transaction()
            for (key, value) in items {
                txn.put(self.name, key, value, &self.serializer, &mut arena)?
                arena.clear()  // Reuse arena
            }
            txn.commit()
        
        batch_write<F, R>(operation) -> Result<R, StorageError>
            let mut arena = Vec::new()
            let txn = self.database.write_transaction()
            let result = operation(txn, self.name, &self.serializer, &mut arena)?
            txn.commit()?
            Ok(result)

    RedbBackend
        impl Backend for RedbBackend
            db: Database
            
            new(path) -> Result<Self, StorageError>
                let db = Database::create(path)
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                Ok(RedbBackend { db })
            
            get<F, R>(&self, table: &str, key: &[u8], accessor: F) -> Result<Option<R>, StorageError>
                where F: FnOnce(&[u8]) -> R
                let read_txn = self.db.begin_read()
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                let table = read_txn.open_table(table)
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                
                match table.get(key) {
                    Ok(Some(value)) => Ok(Some(accessor(value.value()))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(StorageError::Backend(e.to_string()))
                }
            
            put(&self, table: &str, key: &[u8], value: Vec<u8>) -> Result<(), StorageError>
                let write_txn = self.db.begin_write()
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                let mut table = write_txn.open_table(table)
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                table.insert(key, value.as_slice())
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                write_txn.commit()
                    .map_err(|e| StorageError::Backend(e.to_string()))
            
            delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>
                let write_txn = self.db.begin_write()
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                let mut table = write_txn.open_table(table)
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                let existed = table.remove(key)
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                    .is_some()
                write_txn.commit()
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                Ok(existed)
            
            scan<F>(&self, table: &str, mut callback: F) -> Result<(), StorageError>
                where F: FnMut(&[u8], &[u8]) -> bool
                let read_txn = self.db.begin_read()
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                let table = read_txn.open_table(table)
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                
                for item in table.iter() {
                    let (key, value) = item
                        .map_err(|e| StorageError::Backend(e.to_string()))?
                    if !callback(key.value(), value.value()) {
                        break
                    }
                }
                Ok(())

    impl Database
        rkyv_table<K, V>(name) -> Table<K, V, RkyvSerializer>
            Table::new(self, name, RkyvSerializer)
        
        flatbuffers_table<K, V>(name) -> Table<K, V, FlatbuffersSerializer>
            Table::new(self, name, FlatbuffersSerializer)
        
        raw_table<K>(name) -> Table<K, &[u8], RawSerializer>
            Table::new(self, name, RawSerializer)
        
        table_with_serializer<K, V, S>(name, serializer) -> Table<K, V, S>
            Table::new(self, name, serializer)

usage_direct_transactions()
    database = Database::new("app.redb")
    
    users = database.rkyv_table::<u64, User>("users")
    monsters = database.flatbuffers_table::<u64, Monster>("monsters")
    cache = database.raw_table::<&str>("cache")
    
    // Single operations - minimal overhead
    users.put(123, &user)?
    monsters.put(456, &monster)?
    cache.put("session:abc", b"session_data")?
    
    // Zero-copy access
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
    
    // Scan with arena reuse
    active_users = users.scan(|id, user| {
        if user.active.to_native() {
            Some(*id)
        } else {
            None
        }
    })?
    
    // Batch operations - single transaction
    user_data = vec![(1, user1), (2, user2), (3, user3)]
    users.put_all(user_data)?
    
    // Multi-table batch - manual transaction control
    let mut arena = Vec::new()
    let txn = database.write_transaction()
    txn.put("users", user.id, &user, &rkyv_serializer, &mut arena)?
    txn.delete("users", old_user_id, &mut arena)?
    txn.put("monsters", monster.id, &monster, &flatbuffers_serializer, &mut arena)?
    txn.commit()?
    
    // Custom batch operation
    users.batch_write(|txn, table_name, serializer, arena| {
        for i in 0..1000 {
            let user = generate_user(i);
            txn.put(table_name, i, &user, serializer, arena)?;
            if i % 100 == 0 {
                arena.clear(); // Periodic arena cleanup
            }
        }
        Ok(())
    })?

performance_benefits()
    // Zero allocation for string keys
    cache.put("key", b"value")?  // &str -> Cow::Borrowed
    
    // Arena allocation only for complex keys
    users.put(123u64, &user)?    // u64 -> arena allocation
    
    // Batch operations share arena and transaction
    let items = vec![(1, user1), (2, user2), (3, user3)];
    users.put_all(items)?        // Single transaction, reused arena
    
    // Manual control for complex scenarios
    let mut arena = Vec::new();
    let txn = database.write_transaction();
    for (id, user) in large_dataset {
        txn.put("users", id, &user, &serializer, &mut arena)?;
        if arena.len() > 1024 * 1024 {  // Limit memory usage
            arena.clear();
        }
    }
    txn.commit()?
```
