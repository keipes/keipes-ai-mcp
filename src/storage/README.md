# Storage Module v7

Type-safe, zero-copy database abstraction with dual API design for maximum performance and convenience.

## Quick Start

```rust
// Create storage
let storage = Storage::new("data.redb")?;

// Get typed tables
let users = storage.rkyv_table::<u64, User>("users");
let monsters = storage.flatbuffers_table::<u64, Monster>("monsters");

// Zero-copy operations
users.put(&123, &user)?;
let name = users.get(&123, |u| u.name.as_str().to_string())?.unwrap();

// Efficient scanning
let active_users = users.filter_map(|_id, user| {
    if user.active.to_native() {
        Some(user.email.as_str().to_string())
    } else {
        None
    }
})?;

// Memory-efficient processing
let total_revenue = companies.scan_with(|scanner| {
    scanner.fold(0.0, |acc, _id, company| {
        Ok(acc + company.revenue.to_native())
    })
})?;
```

## Core API

### Database Interface
```rust
trait Database: Send + Sync {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&self, table: &str, key: &[u8], value: Vec<u8>) -> Result<(), StorageError>;
    fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>;
    fn scan(&self, table: &str, callback: &mut dyn FnMut(&[u8], &[u8]) -> bool) -> Result<(), StorageError>;
    fn put_batch(&self, table: &str, entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), StorageError>;
}
```

### Table Interface
```rust
impl<K, V, S> Table<K, V, S> {
    // CRUD operations
    fn get<F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
    fn put(&self, key: &K, value: &V) -> Result<(), StorageError>
    fn delete(&self, key: &K) -> Result<bool, StorageError>
    
    // Batch operations
    fn put_batch(&self, entries: impl IntoIterator<Item = (K, V)>) -> Result<(), StorageError>
    
    // Scanning - dual API for different use cases
    fn filter_map<F, R>(&self, mapper: F) -> Result<Vec<R>, StorageError>  // Collect results
    fn scan_with<F, R>(&self, processor: F) -> Result<R, StorageError>    // Stream processing
}
```

## Dual API Design

### Collect API - `filter_map`
Best for: Small-medium datasets, chaining operations
```rust
let active_emails = users
    .filter_map(|_id, user| {
        if user.active.to_native() {
            Some(user.email.as_str().to_string())
        } else {
            None
        }
    })?
    .into_iter()
    .filter(|email| email.contains("@company.com"))
    .collect();
```

### Stream API - `scan_with`  
Best for: Large datasets, memory efficiency, aggregations
```rust
let stats = users.scan_with(|scanner| {
    scanner.fold(UserStats::default(), |mut acc, _id, user| {
        acc.total_count += 1;
        if user.active.to_native() {
            acc.active_count += 1;
        }
        Ok(acc)
    })
})?;
```

## Serialization Formats

### rkyv (Maximum Performance)
```rust
let table = storage.rkyv_table::<u64, User>("users");

// Zero-copy string access
table.get(&123, |user| user.name.as_str().to_string())?;

// Direct numeric access  
table.get(&123, |user| user.age.to_native())?;
```
- **Performance**: ~0ns deserialization, direct memory access
- **Safety**: Unsafe by default (use `access()` for validation)
- **Use case**: High-frequency reads, trusted data

### FlatBuffers (Schema Evolution)
```rust  
let table = storage.flatbuffers_table::<u64, Monster>("monsters");

// Validated access with bounds checking
table.get(&456, |monster| monster.hp())?;

// Safe string access
table.get(&456, |monster| monster.name().unwrap_or("Unknown"))?;
```
- **Performance**: ~10ns validation overhead
- **Safety**: Bounds checking, schema validation
- **Use case**: Evolving schemas, untrusted data

## Zero-Copy Details

### Memory Layout
```rust
// Direct field access on archived data
let user_data = table.get(&123, |user| {
    UserSummary {
        name: user.name.as_str().to_string(),     // Zero-copy &str
        age: user.age.to_native(),                // Direct u32 access
        active: user.active.to_native(),          // Direct bool access
        balance: user.balance.to_native(),        // Direct f64 access
    }
})?;
```

### Performance Characteristics
- **rkyv**: Direct memory access, no allocation on read
- **FlatBuffers**: Validated access, minimal allocation
- **Keys**: Zero-copy for `u64`, `&str` - direct byte representation  
- **Callbacks**: Stack-allocated, zero-cost abstractions

## Advanced Patterns

### Joins
```rust
// Hash join pattern
let users_map: HashMap<u64, String> = users
    .filter_map(|id, user| Some((*id, user.name.as_str().to_string())))?
    .into_iter()
    .collect();

let enriched_orders = orders.filter_map(|_id, order| {
    let user_id = order.user_id.to_native();
    users_map.get(&user_id).map(|name| OrderWithUser {
        amount: order.amount.to_native(),
        user_name: name.clone(),
    })
})?;
```

### Aggregations
```rust
// Group by with fold
let revenue_by_region = companies.scan_with(|scanner| {
    scanner.fold(HashMap::new(), |mut acc, _id, company| {
        let region = company.region.as_str().to_string();
        let revenue = company.revenue.to_native();
        *acc.entry(region).or_insert(0.0) += revenue;
        Ok(acc)
    })
})?;
```

### Time-Series Processing
```rust
// Rolling window analysis
let daily_volumes = trades.scan_with(|scanner| {
    scanner.fold(Vec::new(), |mut acc, _id, trade| {
        acc.push((trade.timestamp.to_native(), trade.volume.to_native()));
        Ok(acc)
    })
})?
.into_iter()
.group_by_day()
.map(|(day, trades)| (day, trades.iter().map(|(_, vol)| vol).sum()))
.collect();
```

## Package Hierarchy

```
src/storage/
├── mod.rs                    # Public API exports
├── error.rs                  # StorageError definitions
├── storage.rs               # Storage facade struct
├── table.rs                 # Table<K,V,S> implementation
├── scanner.rs               # TableScanner implementation
├── key/
│   ├── mod.rs               # Key serialization traits
│   ├── primitives.rs        # u64, String, &str implementations
│   └── custom.rs            # UUID, composite keys (future)
├── serializers/
│   ├── mod.rs               # ValueSerializer trait
│   ├── rkyv.rs              # RkyvSerializer, RkyvBorrowedSerializer
│   ├── flatbuffers.rs       # FlatbuffersSerializer
│   ├── raw.rs               # RawSerializer
│   └── protobuf.rs          # ProtobufSerializer (future)
└── backends/
    ├── mod.rs               # Database trait
    ├── redb.rs              # RedbDatabase implementation
    ├── rocksdb.rs           # RocksDbDatabase (future)
    └── memory.rs            # MemoryDatabase (testing)
```

## Interface Points

### Database Abstraction Layer
```rust
trait Database: Send + Sync {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    // ... other methods
}
```
**Interface for**: Adding new database backends (PostgreSQL, RocksDB, etc.)

### Key Serialization Layer
```rust
trait IntoKeyBytes {
    fn serialize_key_to(&self, arena: &mut Vec<u8>) -> &[u8];
}
trait FromKeyBytes: Sized {
    fn deserialize_key_from(bytes: &[u8]) -> Result<Self, StorageError>;
}
```
**Interface for**: Adding new key types (UUIDs, custom structs, etc.)

### Value Serialization Layer
```rust
trait ValueSerializer<T> {
    type Output<'a>;
    fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>;
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<Self::Output<'a>, StorageError>;
}
```
**Interface for**: Adding new serialization formats (Protobuf, Bincode, etc.)

## Future Extensions

The layered architecture enables easy extension:

### Range Operations (Future)
```rust
// Will be added without breaking changes
trait Database {
    fn scan_range(&self, table: &str, start: &[u8], end: &[u8], ...) -> Result<..>;
    fn first(&self, table: &str) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError>;
    fn last(&self, table: &str) -> Result<Option<(Vec<u8>, Vec<u8>)>, StorageError>;
}
```

### Validation Layer (Future)
```rust
// Add validation without refactoring
trait Validator<T> {
    fn validate(data: &T) -> Result<(), ValidationError>;
}

impl<K, V, S> Table<K, V, S> {
    fn get_validated<Val, F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
    where Val: Validator<S::Output<'_>>
}
```

### Custom Serializers (Future)
```rust
// Extend with new formats
let protobuf_table = storage.protobuf_table::<u64, Message>("messages");
let bincode_table = storage.bincode_table::<u64, Struct>("structs");
```

## Design Philosophy

- **Zero-copy first**: Direct memory access without allocation
- **Callback-based**: Ensure transaction safety and lifetime correctness
- **Dual API**: Balance convenience (collect) vs efficiency (stream)
- **Layered design**: Database abstraction + format specialization
- **Future-proof**: Non-breaking extensibility for range ops, validation, etc.
- **LLM-friendly**: Predictable patterns, clear types, functional composition