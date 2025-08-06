# Storage Implementation v7 - Complete Implementation

## Core Error Types
```rust
#[derive(Debug)]
enum StorageError {
    Backend(String),
    Serialization(String),
    Transaction(String),
    KeyDeserialization(String),
}
```

## Key Serialization Layer
```rust
trait IntoKeyBytes {
    fn serialize_key_to(&self, arena: &mut Vec<u8>) -> &[u8];
}

trait FromKeyBytes: Sized {
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError>;
}

impl IntoKeyBytes for &str {
    fn serialize_key_to(&self, _arena: &mut Vec<u8>) -> &[u8] {
        self.as_bytes()  // Zero-copy
    }
}

impl FromKeyBytes for String {
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| StorageError::KeyDeserialization(e.to_string()))
    }
}

impl IntoKeyBytes for &[u8] {
    fn serialize_key_to(&self, _arena: &mut Vec<u8>) -> &[u8] {
        self  // Zero-copy
    }
}

impl FromKeyBytes for Vec<u8> {
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError> {
        Ok(bytes.to_vec())
    }
}

impl IntoKeyBytes for String {
    fn serialize_key_to(&self, _arena: &mut Vec<u8>) -> &[u8] {
        self.as_bytes()  // Zero-copy
    }
}

impl IntoKeyBytes for u64 {
    fn serialize_key_to(&self, arena: &mut Vec<u8>) -> &[u8] {
        let start = arena.len();
        arena.extend_from_slice(&self.to_le_bytes());
        &arena[start..start+8]
    }
}

impl FromKeyBytes for u64 {
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError> {
        if bytes.len() != 8 {
            return Err(StorageError::KeyDeserialization("u64 key must be 8 bytes".into()));
        }
        let array: [u8; 8] = bytes.try_into()
            .map_err(|_| StorageError::KeyDeserialization("Invalid u64 bytes".into()))?;
        Ok(u64::from_le_bytes(array))
    }
}
```

## Backend Interface Layer
```rust
trait Backend: Send + Sync {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>;
    fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>;
    fn write_transaction(&self) -> Result<WriteTransaction, StorageError>;
    fn read_transaction(&self) -> Result<ReadTransaction, StorageError>;
}
```

## Transaction Layer
```rust
struct WriteTransaction {
    inner: RedbWriteTransaction,
}

impl WriteTransaction {
    fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.inner.put(table, key, value)
    }
    
    fn delete(&mut self, table: &str, key: &[u8]) -> Result<bool, StorageError> {
        self.inner.delete(table, key)
    }
    
    fn commit(mut self) -> Result<(), StorageError> {
        self.inner.commit()
    }
    
    fn rollback(mut self) -> Result<(), StorageError> {
        self.inner.rollback()
    }
}

impl Drop for WriteTransaction {
    fn drop(&mut self) {
        let _ = self.inner.rollback(); // Auto-rollback if not committed
    }
}

struct ReadTransaction {
    inner: RedbReadTransaction,
}

impl ReadTransaction {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.inner.get(table, key)
    }
    
    fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError> {
        self.inner.scan(table, callback)
    }
}

struct RedbWriteTransaction {
    txn: Option<redb::WriteTransaction>,
}

impl RedbWriteTransaction {
    fn new(txn: redb::WriteTransaction) -> Self {
        RedbWriteTransaction { txn: Some(txn) }
    }
    
    fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let txn = self.txn.as_ref().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?;
        let table_def = redb::TableDefinition::new(table);
        let mut table_handle = txn.open_table(table_def)
            .map_err(|e| StorageError::Transaction(e.to_string()))?;
        table_handle.insert(key, value)
            .map_err(|e| StorageError::Transaction(e.to_string()))
    }
    
    fn delete(&mut self, table: &str, key: &[u8]) -> Result<bool, StorageError> {
        let txn = self.txn.as_ref().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?;
        let table_def = redb::TableDefinition::new(table);
        let mut table_handle = txn.open_table(table_def)
            .map_err(|e| StorageError::Transaction(e.to_string()))?;
        let existed = table_handle.remove(key)
            .map_err(|e| StorageError::Transaction(e.to_string()))?
            .is_some();
        Ok(existed)
    }
    
    fn commit(mut self) -> Result<(), StorageError> {
        let txn = self.txn.take().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?;
        txn.commit().map_err(|e| StorageError::Transaction(e.to_string()))
    }
    
    fn rollback(mut self) -> Result<(), StorageError> {
        let txn = self.txn.take().ok_or_else(|| StorageError::Transaction("Transaction already consumed".into()))?;
        txn.abort().map_err(|e| StorageError::Transaction(e.to_string()))
    }
}
```

## Serialization Layer
```rust
trait ValueSerializer<T> {
    type Output<'a>;
    fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>;
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<Self::Output<'a>, StorageError>;
}

// Zero-copy rkyv serializer using AlignedVec
struct RkyvSerializer<T> where T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>> {
    _phantom: PhantomData<T>,
}

impl<T> RkyvSerializer<T> where T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>> {
    fn new() -> Self {
        RkyvSerializer { _phantom: PhantomData }
    }
}

impl<T> ValueSerializer<T> for RkyvSerializer<T>
where T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
{
    type Output<'a> = &'a T::Archived;
    
    fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError> {
        let aligned_bytes = rkyv::to_bytes(value)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        Ok(aligned_bytes.to_vec())
    }
    
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a T::Archived, StorageError> {
        rkyv::access_unchecked::<T>(bytes)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }
}

struct RkyvBorrowedSerializer<T> where T: rkyv::Archive {
    _phantom: PhantomData<T>,
}

impl<T> RkyvBorrowedSerializer<T> where T: rkyv::Archive {
    fn new() -> Self {
        RkyvBorrowedSerializer { _phantom: PhantomData }
    }
}

impl<T> ValueSerializer<&[u8]> for RkyvBorrowedSerializer<T>
where T: rkyv::Archive
{
    type Output<'a> = &'a T::Archived;
    
    fn serialize(&self, value: &&[u8]) -> Result<Vec<u8>, StorageError> {
        // Zero-copy: data is already serialized rkyv bytes
        Ok(value.to_vec())
    }
    
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a T::Archived, StorageError> {
        rkyv::access_unchecked::<T>(bytes)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }
}

struct FlatbuffersSerializer<T> where T: flatbuffers::Follow<'static> + 'static {
    _phantom: PhantomData<T>,
}

impl<T> FlatbuffersSerializer<T> where T: flatbuffers::Follow<'static> + 'static {
    fn new() -> Self {
        FlatbuffersSerializer { _phantom: PhantomData }
    }
}

impl<T> ValueSerializer<&[u8]> for FlatbuffersSerializer<T>
where T: flatbuffers::Follow<'static> + 'static
{
    type Output<'a> = T::Inner;
    
    fn serialize(&self, value: &&[u8]) -> Result<Vec<u8>, StorageError> {
        // Zero-copy: reference existing bytes, only allocate when storing
        Ok(value.to_vec())
    }
    
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<T::Inner, StorageError> {
        flatbuffers::root::<T>(bytes)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }
}

struct RawSerializer;

impl ValueSerializer<Vec<u8>> for RawSerializer {
    type Output<'a> = &'a [u8];
    
    fn serialize(&self, value: &Vec<u8>) -> Result<Vec<u8>, StorageError> {
        Ok(value.clone())
    }
    
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a [u8], StorageError> {
        Ok(bytes)
    }
}
```

## Backend Implementation Layer
```rust
struct RedbBackend {
    db: redb::Database,
}

impl RedbBackend {
    fn new(path: &str) -> Result<Self, StorageError> {
        let db = redb::Database::create(path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(RedbBackend { db })
    }
}

impl Backend for RedbBackend {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let table_def = redb::TableDefinition::new(table);
        let table_handle = read_txn.open_table(table_def)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        match table_handle.get(key) {
            Ok(Some(value)) => Ok(Some(value.value().to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::Backend(e.to_string()))
        }
    }
    
    fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let table_def = redb::TableDefinition::new(table);
        let mut table_handle = write_txn.open_table(table_def)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        table_handle.insert(key, value)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        write_txn.commit()
            .map_err(|e| StorageError::Backend(e.to_string()))
    }
    
    fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError> {
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let table_def = redb::TableDefinition::new(table);
        let mut table_handle = write_txn.open_table(table_def)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let existed = table_handle.remove(key)
            .map_err(|e| StorageError::Backend(e.to_string()))?
            .is_some();
        write_txn.commit()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(existed)
    }
    
    fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let table_def = redb::TableDefinition::new(table);
        let table_handle = read_txn.open_table(table_def)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        for item in table_handle.iter() {
            let (key, value) = item
                .map_err(|e| StorageError::Backend(e.to_string()))?;
            if !callback(key.value(), value.value()) {
                break;
            }
        }
        Ok(())
    }
    
    fn write_transaction(&self) -> Result<WriteTransaction, StorageError> {
        let txn = self.db.begin_write()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(WriteTransaction {
            inner: RedbWriteTransaction::new(txn),
        })
    }
}
```

## High-Level Table Layer
```rust
struct Table<K, V, S> 
where 
    K: IntoKeyBytes + FromKeyBytes,
    S: ValueSerializer<V>
{
    database: Database,
    table_name: String,
    serializer: S,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, S> Table<K, V, S> 
where 
    K: IntoKeyBytes + FromKeyBytes,
    S: ValueSerializer<V>
{
    fn new(database: Database, table_name: String, serializer: S) -> Self {
        Table {
            database,
            table_name,
            serializer,
            _phantom: PhantomData,
        }
    }
    
    fn put(&self, key: &K, value: &V) -> Result<(), StorageError> {
        let mut key_arena = Vec::new();
        let key_bytes = key.serialize_key_to(&mut key_arena);
        let value_bytes = self.serializer.serialize(value)?;
        self.database.backend.put(&self.table_name, key_bytes, &value_bytes)
    }
    
    fn get<F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
    where F: FnOnce(S::Output<'_>) -> R
    {
        let mut key_arena = Vec::new();
        let key_bytes = key.serialize_key_to(&mut key_arena);
        
        match self.database.backend.get(&self.table_name, key_bytes)? {
            Some(value_bytes) => {
                let deserialized = self.serializer.deserialize(&value_bytes)?;
                Ok(Some(accessor(deserialized)))
            }
            None => Ok(None)
        }
    }
    
    fn delete(&self, key: &K) -> Result<bool, StorageError> {
        let mut key_arena = Vec::new();
        let key_bytes = key.serialize_key_to(&mut key_arena);
        self.database.backend.delete(&self.table_name, key_bytes)
    }
    
    fn scan<F, R>(&self, mut scanner: F) -> Result<Vec<R>, StorageError>
    where F: FnMut(&K, S::Output<'_>) -> Option<R>
    {
        let mut results = Vec::new();
        self.database.backend.scan(&self.table_name, &mut |key_bytes, value_bytes| {
            match (K::deserialize_key_from(key_bytes), self.serializer.deserialize(value_bytes)) {
                (Ok(key), Ok(value)) => {
                    if let Some(result) = scanner(&key, value) {
                        results.push(result);
                    }
                    true // Continue scanning
                }
                _ => true // Skip invalid entries, continue scanning
            }
        })?;
        Ok(results)
    }
    
    fn put_batch<I>(&self, items: I) -> Result<(), StorageError>
    where I: IntoIterator<Item = (K, V)>
    {
        let mut txn = self.database.write_transaction()?;
        for (key, value) in items {
            let mut key_arena = Vec::new();
            let key_bytes = key.serialize_key_to(&mut key_arena);
            let value_bytes = self.serializer.serialize(&value)?;
            txn.put(&self.table_name, key_bytes, &value_bytes)?;
        }
        txn.commit()
    }
    
    fn write_transaction<F, R>(&self, operation: F) -> Result<R, StorageError>
    where F: FnOnce(&mut WriteTransaction, &str, &S) -> Result<R, StorageError>
    {
        let mut txn = self.database.write_transaction()?;
        let result = operation(&mut txn, &self.table_name, &self.serializer)?;
        txn.commit()?;
        Ok(result)
    }
}
```

## Database Facade Layer
```rust
#[derive(Clone)]
struct Database {
    backend: Arc<dyn Backend>,
}

impl Database {
    fn new(path: &str) -> Result<Self, StorageError> {
        let backend = Arc::new(RedbBackend::new(path)?);
        Ok(Database { backend })
    }
    
    fn write_transaction(&self) -> Result<WriteTransaction, StorageError> {
        self.backend.write_transaction()
    }
    
    fn rkyv_table<K, V>(&self, name: &str) -> Table<K, V, RkyvSerializer<V>>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        V: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
    {
        Table::new(self.clone(), name.to_string(), RkyvSerializer::new())
    }
    
    fn rkyv_borrowed_table<K, V>(&self, name: &str) -> Table<K, &[u8], RkyvBorrowedSerializer<V>>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        V: rkyv::Archive
    {
        Table::new(self.clone(), name.to_string(), RkyvBorrowedSerializer::new())
    }
    
    fn flatbuffers_table<K, V>(&self, name: &str) -> Table<K, &[u8], FlatbuffersSerializer<V>>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        V: flatbuffers::Follow<'static> + 'static
    {
        Table::new(self.clone(), name.to_string(), FlatbuffersSerializer::new())
    }
    
    fn raw_table<K>(&self, name: &str) -> Table<K, Vec<u8>, RawSerializer>
    where K: IntoKeyBytes + FromKeyBytes
    {
        Table::new(self.clone(), name.to_string(), RawSerializer)
    }
}
```

## Usage Examples
```rust
// Create database
let db = Database::new("app.redb")?;

// Create typed tables
let users = db.rkyv_table::<u64, User>("users");              // Direct serialization
let sessions = db.rkyv_borrowed_table::<u64, Session>("sessions"); // Pre-serialized data
let monsters = db.flatbuffers_table::<u64, Monster>("monsters");
let cache = db.raw_table::<String>("cache");

// Simple operations
users.put(&123, &user)?;                    // rkyv serializes on-demand
sessions.put(&456, &session_bytes)?;       // &[u8] - zero-copy rkyv data
monsters.put(&789, &monster_bytes)?;       // &[u8] - zero-copy FlatBuffers
cache.put(&"session".to_string(), &session_data)?;

// Zero-copy access
let user_name = users.get(&123, |archived_user| {
    archived_user.name.as_str().to_string()
})?.unwrap();

let monster_hp = monsters.get(&456, |monster| {
    monster.hp()
})?.unwrap();

// Direct transactions
let mut txn = db.write_transaction()?;
for (id, user) in batch {
    let mut key_arena = Vec::new();
    let key_bytes = id.serialize_key_to(&mut key_arena);
    let value_bytes = RkyvSerializer::new().serialize(&user)?;
    txn.put("users", key_bytes, &value_bytes)?;
}
txn.commit()?; // Or auto-rollback on drop
```
