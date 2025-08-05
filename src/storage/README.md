# Storage Module

A type-safe, zero-copy database abstraction layer with pluggable backends and serialization formats.

## Project Layout

```
src/storage/
    mod.rs                      // Public API exports
    builder.rs                  // StorageBuilder implementation
    config.rs                   // StorageConfig, DatabaseBackend, Recreate enums
    
    backends/
        mod.rs                  // Backend trait definitions
        redb.rs                 // REDB backend + table implementation
        sled.rs                 // Sled backend + table implementation
        rocksdb.rs              // RocksDB backend + table implementation
    
    formats/
        mod.rs                  // Format trait definitions (KeyFormat, ValueFormat)
        rkyv.rs                 // RkyvFormat, RkyvGuard implementations
        flatbuffers.rs          // FlatbuffersFormat, FlatbuffersGuard implementations
    
    table/
        mod.rs                  // Table trait and TableBuilder
        guards.rs               // Base guard traits and utilities
        stream.rs               // Table stream implementations
    
    error.rs                    // Storage-specific error types
    tests/
        mod.rs
        integration.rs          // Cross-backend integration tests
        format_tests.rs         // Format compatibility tests
        guard_tests.rs          // Guard lifetime tests
```

## Complete API (Pseudocode)

### Core Types

```
StorageBuilder
    new(path: string) -> StorageBuilder
    with_backend(backend: DatabaseBackend) -> StorageBuilder
    with_recreate(recreate: Recreate) -> StorageBuilder
    build() -> Storage

DatabaseBackend
    Redb
    Sled
    RocksDb

Recreate
    Always
    IfMissing
    OnVersionBump
    OnValidationFail

Storage
    new_table<K, V>(name: string) -> TableBuilder<K, V>
    compact() -> Result
```

### Table API

```
TableBuilder<K, V>
    with_key_format<F>(format: F) -> TableBuilder<K, V>
        where F: KeyFormat<K>
    with_value_format<F>(format: F) -> TableBuilder<K, V>
        where F: ValueFormat<V>
    build() -> Table<K, V>

Table<K, V>
    get(key: &K) -> Result<Option<V::Guard<'_>>>
    with_stream<F, R>(reader: F) -> Result<R>
        where F: FnOnce(impl Stream<Item = (K, V::Guard<'_>)>) -> R
    put(key: &K, value: &V) -> Result
    put_all(items: Iterator<(&K, &V)>) -> Result
    put_stream<S>(stream: S) -> Result
        where S: Stream<Item = (&K, &V)>
    delete(key: &K) -> Result<bool>
```

### Format System

```
KeyFormat<T>
    to_bytes(value: &T) -> Vec<u8>
    from_bytes(bytes: &[u8]) -> T

ValueFormat<T>
    to_bytes(value: &T) -> Vec<u8>
    view_guard(bytes: &[u8]) -> Result<Self::Guard<'_>>
    type Guard<'a>: Deref<Target = Self::View<'a>>
    type View<'a>: ?Sized

ValueGuard<'txn, T>
    implements Deref<Target = T>
    implements Drop
```

### Format Implementations

```
RkyvFormat<T>
    implements KeyFormat<T> + ValueFormat<T>
    type Guard<'a> = RkyvGuard<'a, T>
    type View<'a> = T::Archived

FlatbuffersFormat<T>
    implements KeyFormat<T> + ValueFormat<T>
    type Guard<'a> = FlatbuffersGuard<'a, T>
    type View<'a> = T::Inner

BincodeFormat<T>
    implements KeyFormat<T> + ValueFormat<T>
    type Guard<'a> = BincodeGuard<'a, T>
    type View<'a> = T

PostcardFormat<T>
    implements KeyFormat<T> + ValueFormat<T>
    type Guard<'a> = PostcardGuard<'a, T>
    type View<'a> = T
```

## Usage Examples

### Basic Setup

```rust
// Create storage with REDB backend
let storage = StorageBuilder::new("db.redb")
    .with_backend(DatabaseBackend::Redb)
    .build()?;

// Create table with rkyv serialization
let cik_table = storage.new_table::<u64, CikData>("ciks")
    .with_key_format(RkyvFormat::new())
    .with_value_format(RkyvFormat::new())
    .build()?;
```

### Zero-Copy Reads

```rust
// Get with zero-copy guard
let guard = cik_table.get(&123123)?;
if let Some(data) = guard {
    println!("Name: {}", data.name()); // Direct access to archived data
    // Guard automatically manages transaction lifetime
}
```

### Stream Processing

```rust
// Stream-based iteration for large datasets
let companies = cik_table.with_stream(|stream| {
    stream
        .filter(|(_, data)| data.revenue() > 1000.0)
        .map(|(cik, guard)| (cik, guard.name().to_string()))
        .take(100)
        .collect::<Vec<_>>()
})?;
```

### Batch Operations

```rust
// Synchronous batch insert
let batch_data = vec![
    (&111111, &cik_data1),
    (&222222, &cik_data2),
];
cik_table.put_all(batch_data.into_iter())?;

// Asynchronous streaming insert
let data_stream = stream::iter(large_dataset.iter());
cik_table.put_stream(data_stream).await?;
```

### Mixed Formats

```rust
// Different serialization per table
let company_table = storage.new_table::<String, Company>("companies")
    .with_key_format(PostcardFormat::new())      // Compact string keys
    .with_value_format(FlatbuffersFormat::new()) // Zero-copy values
    .build()?;

let metrics_table = storage.new_table::<u64, Metrics>("metrics")
    .with_key_format(RkyvFormat::new())     // Fast numeric keys
    .with_value_format(BincodeFormat::new()) // Simple values
    .build()?;
```

## Key Features

### Zero-Copy Access
- **Guards manage transaction lifetimes** automatically
- **Direct access to archived data** without copying
- **Format-specific optimizations** (rkyv, flatbuffers)

### Type Safety
- **Compile-time serialization guarantees**
- **Format compatibility checking**
- **Memory-safe guard lifetimes**

### Flexibility
- **Pluggable database backends** (REDB, Sled, RocksDB)
- **Pluggable serialization formats** (rkyv, flatbuffers, bincode, postcard)
- **Mix and match** formats per table

### Performance
- **Zero-copy reads** where possible
- **Batch operations** for bulk writes
- **Streaming support** for large datasets
- **Efficient transaction management**

### Developer Experience
- **Clean, Rust-native API**
- **No manual transaction management**
- **Automatic resource cleanup**
- **Stream-based iteration**

## Architecture Principles

1. **Separation of Concerns**: Database backends, serialization formats, and table API are independent
2. **Zero-Copy First**: Prioritize zero-copy access through guard pattern
3. **Type Safety**: Leverage Rust's type system for correctness
4. **Resource Management**: Automatic cleanup through RAII
5. **Streaming**: First-class support for large datasets
6. **Flexibility**: Easy to swap backends and formats

## Implementation Status

This is a design document. The actual implementation will follow this API design.
