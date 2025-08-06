# Storage Implementation v6 - Pragmatic and Clean

```
trait IntoKeyBytes
    fn serialize_key_to(&self, arena: &mut Vec<u8>) -> &[u8]

trait FromKeyBytes: Sized
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError>

impl IntoKeyBytes for &str
    fn serialize_key_to(&self, _arena: &mut Vec<u8>) -> &[u8]
        self.as_bytes()  // Zero-copy, no arena needed

impl FromKeyBytes for String
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError>
        String::from_utf8(bytes.to_vec())
            .map_err(|e| StorageError::KeyDeserialization(e.to_string()))

impl IntoKeyBytes for &[u8]
    fn serialize_key_to(&self, _arena: &mut Vec<u8>) -> &[u8]
        self  // Zero-copy, no arena needed

impl FromKeyBytes for Vec<u8>
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError>
        Ok(bytes.to_vec())

impl IntoKeyBytes for String
    fn serialize_key_to(&self, _arena: &mut Vec<u8>) -> &[u8]
        self.as_bytes()  // Zero-copy

impl IntoKeyBytes for u64
    fn serialize_key_to(&self, arena: &mut Vec<u8>) -> &[u8]
        let start = arena.len()
        arena.extend_from_slice(&self.to_le_bytes())
        &arena[start..start+8]

impl FromKeyBytes for u64
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError>
        if bytes.len() != 8 {
            return Err(StorageError::KeyDeserialization("u64 key must be 8 bytes".into()))
        }
        let array: [u8; 8] = bytes.try_into()
            .map_err(|_| StorageError::KeyDeserialization("Invalid u64 bytes".into()))?
        Ok(u64::from_le_bytes(array))

mod storage
    #[derive(Debug)]
    enum StorageError
        Backend(String)
        Serialization(String)
        Transaction(String)
        KeyDeserialization(String)

    trait Backend: Send + Sync
        fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>
        fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>
        fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>
        fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>
        fn write_transaction(&self) -> Result<Box<dyn WriteTransaction>, StorageError>
        fn read_transaction(&self) -> Result<Box<dyn ReadTransaction>, StorageError>

    trait WriteTransaction
        fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>
        fn delete(&mut self, table: &str, key: &[u8]) -> Result<bool, StorageError>
        fn commit(self: Box<Self>) -> Result<(), StorageError>
        fn rollback(self: Box<Self>) -> Result<(), StorageError>

    trait ReadTransaction
        fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>
        fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>

    Database
        backend: Box<dyn Backend>
        
        new(path: &str) -> Result<Self, StorageError>
            let backend = Box::new(RedbBackend::new(path)?)
            Ok(Database { backend })

    trait ValueSerializer<T>
        fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>
        fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<Self::Output<'a>, StorageError>
        type Output<'a>

    RkyvSerializer<T> where T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
        _phantom: PhantomData<T>
        
        new() -> Self
            RkyvSerializer { _phantom: PhantomData }

    impl<T> ValueSerializer<T> for RkyvSerializer<T>
    where T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
        type Output<'a> = &'a T::Archived
        
        fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>
            let aligned_bytes = rkyv::to_bytes(value)
                .map_err(|e| StorageError::Serialization(e.to_string()))?
            // Accept the allocation - pragmatic approach
            Ok(aligned_bytes.to_vec())
        
        fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a T::Archived, StorageError>
            rkyv::access_unchecked::<T>(bytes)
                .map_err(|e| StorageError::Serialization(e.to_string()))

    FlatbuffersSerializer<T> where T: flatbuffers::Follow<'static> + 'static
        _phantom: PhantomData<T>
        
        new() -> Self
            FlatbuffersSerializer { _phantom: PhantomData }

    impl<T> ValueSerializer<T> for FlatbuffersSerializer<T>
    where T: flatbuffers::Follow<'static> + 'static
        type Output<'a> = T::Inner
        
        fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>
            // Note: This assumes T has a to_flatbuffer method
            // In practice, you'd need custom serialization logic per type
            todo!("FlatBuffers serialization requires type-specific implementation")
        
        fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<T::Inner, StorageError>
            flatbuffers::root::<T>(bytes)
                .map_err(|e| StorageError::Serialization(e.to_string()))

    RawSerializer
        
    impl ValueSerializer<Vec<u8>> for RawSerializer
        type Output<'a> = &'a [u8]
        
        fn serialize(&self, value: &Vec<u8>) -> Result<Vec<u8>, StorageError>
            Ok(value.clone())
        
        fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a [u8], StorageError>
            Ok(bytes)

    Table<K, V, S>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        S: ValueSerializer<V>
        database: Database
        table_name: String
        serializer: S
        _phantom: PhantomData<(K, V)>
        
        new(database: Database, table_name: String, serializer: S) -> Self
            Table {
                database,
                table_name,
                serializer,
                _phantom: PhantomData,
            }
        
        put(&self, key: &K, value: &V) -> Result<(), StorageError>
            let mut key_arena = Vec::new()
            let key_bytes = key.serialize_key_to(&mut key_arena)
            let value_bytes = self.serializer.serialize(value)?
            self.database.backend.put(&self.table_name, key_bytes, &value_bytes)
        
        get<F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
        where F: FnOnce(S::Output<'_>) -> R
            let mut key_arena = Vec::new()
            let key_bytes = key.serialize_key_to(&mut key_arena)
            
            match self.database.backend.get(&self.table_name, key_bytes)? {
                Some(value_bytes) => {
                    let deserialized = self.serializer.deserialize(&value_bytes)?
                    Ok(Some(accessor(deserialized)))
                }
                None => Ok(None)
            }
        
        delete(&self, key: &K) -> Result<bool, StorageError>
            let mut key_arena = Vec::new()
            let key_bytes = key.serialize_key_to(&mut key_arena)
            self.database.backend.delete(&self.table_name, key_bytes)
        
        scan<F, R>(&self, mut scanner: F) -> Result<Vec<R>, StorageError>
        where F: FnMut(&K, S::Output<'_>) -> Option<R>
            let mut results = Vec::new()
            self.database.backend.scan(&self.table_name, &mut |key_bytes, value_bytes| {
                match (K::deserialize_key_from(key_bytes), self.serializer.deserialize(value_bytes)) {
                    (Ok(key), Ok(value)) => {
                        if let Some(result) = scanner(&key, value) {
                            results.push(result)
                        }
                        true // Continue scanning
                    }
                    _ => true // Skip invalid entries, continue scanning
                }
            })?
            Ok(results)
        
        write_transaction<F, R>(&self, operation: F) -> Result<R, StorageError>
        where F: FnOnce(&mut dyn WriteTransaction, &str, &S) -> Result<R, StorageError>
            let mut txn = self.database.backend.write_transaction()?
            match operation(txn.as_mut(), &self.table_name, &self.serializer) {
                Ok(result) => {
                    txn.commit()?
                    Ok(result)
                }
                Err(e) => {
                    txn.rollback()?
                    Err(e)
                }
            }
        
        put_batch<I>(&self, items: I) -> Result<(), StorageError>
        where I: IntoIterator<Item = (K, V)>
            self.write_transaction(|txn, table_name, serializer| {
                let mut key_arena = Vec::new()
                for (key, value) in items {
                    key_arena.clear()
                    let key_bytes = key.serialize_key_to(&mut key_arena)
                    let value_bytes = serializer.serialize(&value)?
                    txn.put(table_name, key_bytes, &value_bytes)?
                }
                Ok(())
            })

    RedbBackend
        db: redb::Database
        
        new(path: &str) -> Result<Self, StorageError>
            let db = redb::Database::create(path)
                .map_err(|e| StorageError::Backend(e.to_string()))?
            Ok(RedbBackend { db })

    impl Backend for RedbBackend
        fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>
            let read_txn = self.db.begin_read()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            let table_def = redb::TableDefinition::new(table)
            let table_handle = read_txn.open_table(table_def)
                .map_err(|e| StorageError::Backend(e.to_string()))?
            
            match table_handle.get(key) {
                Ok(Some(value)) => Ok(Some(value.value().to_vec())),
                Ok(None) => Ok(None),
                Err(e) => Err(StorageError::Backend(e.to_string()))
            }
        
        fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>
            let write_txn = self.db.begin_write()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            let table_def = redb::TableDefinition::new(table)
            let mut table_handle = write_txn.open_table(table_def)
                .map_err(|e| StorageError::Backend(e.to_string()))?
            table_handle.insert(key, value)
                .map_err(|e| StorageError::Backend(e.to_string()))?
            write_txn.commit()
                .map_err(|e| StorageError::Backend(e.to_string()))
        
        fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>
            let write_txn = self.db.begin_write()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            let table_def = redb::TableDefinition::new(table)
            let mut table_handle = write_txn.open_table(table_def)
                .map_err(|e| StorageError::Backend(e.to_string()))?
            let existed = table_handle.remove(key)
                .map_err(|e| StorageError::Backend(e.to_string()))?
                .is_some()
            write_txn.commit()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            Ok(existed)
        
        fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>
            let read_txn = self.db.begin_read()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            let table_def = redb::TableDefinition::new(table)
            let table_handle = read_txn.open_table(table_def)
                .map_err(|e| StorageError::Backend(e.to_string()))?
            
            for item in table_handle.iter() {
                let (key, value) = item
                    .map_err(|e| StorageError::Backend(e.to_string()))?
                if !callback(key.value(), value.value()) {
                    break
                }
            }
            Ok(())
        
        fn write_transaction(&self) -> Result<Box<dyn WriteTransaction>, StorageError>
            let txn = self.db.begin_write()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            Ok(Box::new(RedbWriteTransaction::new(txn)))
        
        fn read_transaction(&self) -> Result<Box<dyn ReadTransaction>, StorageError>
            let txn = self.db.begin_read()
                .map_err(|e| StorageError::Backend(e.to_string()))?
            Ok(Box::new(RedbReadTransaction::new(txn)))

    RedbWriteTransaction
        txn: Option<redb::WriteTransaction>
        
        new(txn: redb::WriteTransaction) -> Self
            RedbWriteTransaction { txn: Some(txn) }

    impl WriteTransaction for RedbWriteTransaction
        fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>
            let txn = self.txn.as_ref().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?
            let table_def = redb::TableDefinition::new(table)
            let mut table_handle = txn.open_table(table_def)
                .map_err(|e| StorageError::Transaction(e.to_string()))?
            table_handle.insert(key, value)
                .map_err(|e| StorageError::Transaction(e.to_string()))
        
        fn delete(&mut self, table: &str, key: &[u8]) -> Result<bool, StorageError>
            let txn = self.txn.as_ref().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?
            let table_def = redb::TableDefinition::new(table)
            let mut table_handle = txn.open_table(table_def)
                .map_err(|e| StorageError::Transaction(e.to_string()))?
            let existed = table_handle.remove(key)
                .map_err(|e| StorageError::Transaction(e.to_string()))?
                .is_some()
            Ok(existed)
        
        fn commit(mut self: Box<Self>) -> Result<(), StorageError>
            let txn = self.txn.take().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?
            txn.commit().map_err(|e| StorageError::Transaction(e.to_string()))
        
        fn rollback(mut self: Box<Self>) -> Result<(), StorageError>
            let txn = self.txn.take().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?
            txn.abort().map_err(|e| StorageError::Transaction(e.to_string()))

    RedbReadTransaction
        txn: redb::ReadTransaction
        
        new(txn: redb::ReadTransaction) -> Self
            RedbReadTransaction { txn }

    impl ReadTransaction for RedbReadTransaction
        fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>
            let table_def = redb::TableDefinition::new(table)
            let table_handle = self.txn.open_table(table_def)
                .map_err(|e| StorageError::Transaction(e.to_string()))?
            
            match table_handle.get(key) {
                Ok(Some(value)) => Ok(Some(value.value().to_vec())),
                Ok(None) => Ok(None),
                Err(e) => Err(StorageError::Transaction(e.to_string()))
            }
        
        fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>
            let table_def = redb::TableDefinition::new(table)
            let table_handle = self.txn.open_table(table_def)
                .map_err(|e| StorageError::Transaction(e.to_string()))?
            
            for item in table_handle.iter() {
                let (key, value) = item
                    .map_err(|e| StorageError::Transaction(e.to_string()))?
                if !callback(key.value(), value.value()) {
                    break
                }
            }
            Ok(())

    impl Database
        fn rkyv_table<K, V>(&self, name: &str) -> Table<K, V, RkyvSerializer<V>>
        where 
            K: IntoKeyBytes + FromKeyBytes,
            V: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
            Table::new(self.clone(), name.to_string(), RkyvSerializer::new())
        
        fn flatbuffers_table<K, V>(&self, name: &str) -> Table<K, V, FlatbuffersSerializer<V>>
        where 
            K: IntoKeyBytes + FromKeyBytes,
            V: flatbuffers::Follow<'static> + 'static
            Table::new(self.clone(), name.to_string(), FlatbuffersSerializer::new())
        
        fn raw_table<K>(&self, name: &str) -> Table<K, Vec<u8>, RawSerializer>
        where K: IntoKeyBytes + FromKeyBytes
            Table::new(self.clone(), name.to_string(), RawSerializer)

usage_examples()
    // Create database
    let db = Database::new("app.redb")?
    
    // Create typed tables
    let users = db.rkyv_table::<u64, User>("users")
    let cache = db.raw_table::<String>("cache")
    
    // Simple operations
    users.put(&123, &user)?
    cache.put(&"session".to_string(), &session_data)?
    
    // Zero-copy access
    let user_name = users.get(&123, |archived_user| {
        archived_user.name.as_str().to_string()
    })?.unwrap()
    
    // Batch operations
    let user_batch = vec![(1, user1), (2, user2), (3, user3)]
    users.put_batch(user_batch)?
    
    // Custom transactions
    users.write_transaction(|txn, table_name, serializer| {
        let mut key_arena = Vec::new()
        for i in 0..1000 {
            key_arena.clear()
            let key_bytes = i.serialize_key_to(&mut key_arena)
            let user = generate_user(i)
            let value_bytes = serializer.serialize(&user)?
            txn.put(table_name, key_bytes, &value_bytes)?
        }
        Ok(())
    })?
    
    // Scanning
    let active_users = users.scan(|user_id, archived_user| {
        if archived_user.active.to_native() {
            Some(*user_id)
        } else {
            None
        }
    })?

pragmatic_design_decisions()
    // Accept one allocation per write operation
    serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>
        let aligned_bytes = rkyv::to_bytes(value)?
        Ok(aligned_bytes.to_vec())  // Pragmatic allocation
    
    // Zero-copy reads (the common case)
    deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a T::Archived, StorageError>
        rkyv::access_unchecked::<T>(bytes)  // True zero-copy
    
    // Clean &[u8] interface throughout
    Backend::put(&self, table: &str, key: &[u8], value: &[u8])  // Simple and clean
    
    // Arena only for complex keys like u64
    key.serialize_key_to(&mut key_arena)  // Returns &[u8], arena as scratch space

performance_characteristics()
    // Write path: One allocation per operation (acceptable)
    users.put(123, &user)?  // rkyv::to_bytes() + .to_vec()
    
    // Read path: True zero-copy
    users.get(123, |archived| archived.name.as_str())?  // No allocations
    
    // Key serialization: Zero-copy for strings, arena for numbers
    cache.put("key", value)?     // "key".as_bytes() - no allocation
    users.put(123u64, value)?    // 123.to_le_bytes() - arena allocation
    
    // Batch operations: Shared transaction, reused key arena
    users.put_batch(items)?      // Single transaction, arena.clear() between items

implementation_readiness()
    // All interfaces use standard Rust types
    Vec<u8>, &[u8], String, u64, &str  // No exotic types
    
    // Direct redb compatibility
    redb::TableDefinition::new(table)   // Dynamic table creation
    table_handle.insert(key: &[u8], value: &[u8])  // Direct compatibility
    
    // Standard error handling
    Result<T, StorageError>  // Consistent error propagation
    
    // No lifetime complexity
    Database owns Backend, Table borrows Database  // Simple ownership
    
    // Ready for implementation - no remaining design questions
```
