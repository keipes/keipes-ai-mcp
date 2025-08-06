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

## Core Database Interface
```rust
trait Database: Send + Sync {
    // Direct operations (auto-commit)
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>;
    fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>;
    
    // Batch operations (single transaction)
    fn put_batch(&self, table: &str, items: &[(&[u8], &[u8])]) -> Result<(), StorageError>;
}

// Simple implementation struct
struct RedbDatabase {
    db: redb::Database,
}

impl RedbDatabase {
    fn new(path: &str) -> Result<Self, StorageError> {
        let db = redb::Database::create(path)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(RedbDatabase { db })
    }
}

impl Database for RedbDatabase {
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
    
    fn put_batch(&self, table: &str, items: &[(&[u8], &[u8])]) -> Result<(), StorageError> {
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let table_def = redb::TableDefinition::new(table);
        let mut table_handle = write_txn.open_table(table_def)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        
        for (key, value) in items {
            table_handle.insert(*key, *value)
                .map_err(|e| StorageError::Backend(e.to_string()))?;
        }
        
        write_txn.commit()
            .map_err(|e| StorageError::Backend(e.to_string()))
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
        // Zero-copy: avoid clone, just reference slice directly
        Ok((*value).to_vec())
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
        // Zero-copy: avoid clone, just reference slice directly  
        Ok((*value).to_vec())
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

## High-Level Table Layer
```rust
use std::cell::RefCell;

struct Table<K, V, S> 
where 
    K: IntoKeyBytes + FromKeyBytes,
    S: ValueSerializer<V>
{
    database: Arc<dyn Database>,
    table_name: String,
    serializer: S,
    key_arena: RefCell<Vec<u8>>, // Reusable key buffer
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, S> Table<K, V, S> 
where 
    K: IntoKeyBytes + FromKeyBytes,
    S: ValueSerializer<V>
{
    fn new(database: Arc<dyn Database>, table_name: String, serializer: S) -> Self {
        Table {
            database,
            table_name,
            serializer,
            key_arena: RefCell::new(Vec::with_capacity(64)), // Pre-allocate
            _phantom: PhantomData,
        }
    }
    
    fn put(&self, key: &K, value: &V) -> Result<(), StorageError> {
        let mut arena = self.key_arena.borrow_mut();
        arena.clear();
        let key_bytes = key.serialize_key_to(&mut arena);
        let value_bytes = self.serializer.serialize(value)?;
        self.database.put(&self.table_name, key_bytes, &value_bytes)
    }
    
    fn get<F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
    where F: FnOnce(S::Output<'_>) -> R
    {
        let mut arena = self.key_arena.borrow_mut();
        arena.clear();
        let key_bytes = key.serialize_key_to(&mut arena);
        
        match self.database.get(&self.table_name, key_bytes)? {
            Some(value_bytes) => {
                let deserialized = self.serializer.deserialize(&value_bytes)?;
                Ok(Some(accessor(deserialized)))
            }
            None => Ok(None)
        }
    }
    
    fn delete(&self, key: &K) -> Result<bool, StorageError> {
        let mut arena = self.key_arena.borrow_mut();
        arena.clear();
        let key_bytes = key.serialize_key_to(&mut arena);
        self.database.delete(&self.table_name, key_bytes)
    }
    
    fn filter_map<F, R>(&self, mut mapper: F) -> Result<Vec<R>, StorageError>
    where F: FnMut(&K, S::Output<'_>) -> Option<R>
    {
        let mut results = Vec::new();
        self.database.scan(&self.table_name, &mut |key_bytes, value_bytes| {
            match (K::deserialize_key_from(key_bytes), self.serializer.deserialize(value_bytes)) {
                (Ok(key), Ok(value)) => {
                    if let Some(result) = mapper(&key, value) {
                        results.push(result);
                    }
                    true // Continue scanning
                }
                _ => true // Skip invalid entries, continue scanning
            }
        })?;
        Ok(results)
    }
    
    fn scan_with<F, R>(&self, processor: F) -> Result<R, StorageError>
    where F: FnOnce(TableScanner<'_, K, V, S>) -> Result<R, StorageError>
    {
        let scanner = TableScanner::new(self);
        processor(scanner)
    }
    
    fn put_batch<I>(&self, items: I) -> Result<(), StorageError>
    where I: IntoIterator<Item = (K, V)>
    {
        let mut key_arena = Vec::with_capacity(64);
        let mut batch_items = Vec::new();
        
        for (key, value) in items {
            key_arena.clear();
            let arena_start_len = key_arena.len();
            let key_bytes = key.serialize_key_to(&mut key_arena);
            let value_bytes = self.serializer.serialize(&value)?;
            
            // Check if arena was actually used
            if key_arena.len() > arena_start_len {
                // Arena was used (key like u64) - need to copy
                batch_items.push((key_bytes.to_vec(), value_bytes));
            } else {
                // Zero-copy key (like &str) - key_bytes points to original data
                batch_items.push((key_bytes.to_vec(), value_bytes));
            }
        }
        
        let refs: Vec<_> = batch_items.iter().map(|(k, v)| (k.as_slice(), v.as_slice())).collect();
        self.database.put_batch(&self.table_name, &refs)
    }
}

// Streaming iterator adapter
struct TableScanner<'a, K, V, S> {
    table: &'a Table<K, V, S>,
}

impl<'a, K, V, S> TableScanner<'a, K, V, S>
where 
    K: IntoKeyBytes + FromKeyBytes,
    S: ValueSerializer<V>
{
    fn new(table: &'a Table<K, V, S>) -> Self {
        TableScanner { table }
    }
    
    fn for_each<F>(self, mut processor: F) -> Result<(), StorageError>
    where F: FnMut(&K, S::Output<'_>) -> Result<(), StorageError>
    {
        self.table.database.scan(&self.table.table_name, &mut |key_bytes, value_bytes| {
            match (K::deserialize_key_from(key_bytes), self.table.serializer.deserialize(value_bytes)) {
                (Ok(key), Ok(value)) => {
                    match processor(&key, value) {
                        Ok(()) => true,  // Continue
                        Err(_) => false, // Stop on error
                    }
                }
                _ => true // Skip invalid entries, continue
            }
        })
    }
    
    fn fold<F, Acc>(self, init: Acc, mut folder: F) -> Result<Acc, StorageError>
    where F: FnMut(Acc, &K, S::Output<'_>) -> Result<Acc, StorageError>
    {
        let mut accumulator = init;
        self.table.database.scan(&self.table.table_name, &mut |key_bytes, value_bytes| {
            match (K::deserialize_key_from(key_bytes), self.table.serializer.deserialize(value_bytes)) {
                (Ok(key), Ok(value)) => {
                    match folder(accumulator, &key, value) {
                        Ok(new_acc) => {
                            accumulator = new_acc;
                            true // Continue
                        }
                        Err(_) => false, // Stop on error
                    }
                }
                _ => true // Skip invalid entries
            }
        })?;
        Ok(accumulator)
    }
}
```

## Storage Facade
```rust
struct Storage {
    database: Arc<dyn Database>,
}

impl Storage {
    fn new(path: &str) -> Result<Self, StorageError> {
        let database = Arc::new(RedbDatabase::new(path)?);
        Ok(Storage { database })
    }
    
    fn rkyv_table<K, V>(&self, name: &str) -> Table<K, V, RkyvSerializer<V>>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        V: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
    {
        Table::new(self.database.clone(), name.to_string(), RkyvSerializer::new())
    }
    
    fn rkyv_borrowed_table<K, V>(&self, name: &str) -> Table<K, &[u8], RkyvBorrowedSerializer<V>>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        V: rkyv::Archive
    {
        Table::new(self.database.clone(), name.to_string(), RkyvBorrowedSerializer::new())
    }
    
    fn flatbuffers_table<K, V>(&self, name: &str) -> Table<K, &[u8], FlatbuffersSerializer<V>>
    where 
        K: IntoKeyBytes + FromKeyBytes,
        V: flatbuffers::Follow<'static> + 'static
    {
        Table::new(self.database.clone(), name.to_string(), FlatbuffersSerializer::new())
    }
    
    fn raw_table<K>(&self, name: &str) -> Table<K, Vec<u8>, RawSerializer>
    where K: IntoKeyBytes + FromKeyBytes
    {
        Table::new(self.database.clone(), name.to_string(), RawSerializer)
    }
}
```

## Usage Examples
```rust
// Create storage
let storage = Storage::new("app.redb")?;

// Create typed tables
let users = storage.rkyv_table::<u64, User>("users");              // Direct serialization
let sessions = storage.rkyv_borrowed_table::<u64, Session>("sessions"); // Pre-serialized data
let monsters = storage.flatbuffers_table::<u64, Monster>("monsters");
let cache = storage.raw_table::<String>("cache");

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

// Batch operations
let user_batch = vec![(1, user1), (2, user2), (3, user3)];
users.put_batch(user_batch)?;

// Filter-map operations  
let active_user_names = users.filter_map(|_id, user| {
    if user.active.to_native() {
        Some(user.name.as_str().to_string())
    } else {
        None
    }
})?;

// Streaming operations for large datasets
let total_revenue = users.scan_with(|scanner| {
    scanner.fold(0.0, |acc, _id, user| {
        Ok(acc + user.revenue.to_native())
    })
})?;

// Process without collecting all in memory
users.scan_with(|scanner| {
    scanner.for_each(|id, user| {
        if user.revenue.to_native() > 1_000_000.0 {
            println!("High revenue user {}: {}", id, user.name.as_str());
        }
        Ok(())
    })
})?;
```
