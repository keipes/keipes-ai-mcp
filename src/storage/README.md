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
    formats/
        mod.rs                  // Format trait definitions (KeyFormat, ValueFormat)
        rkyv.rs                 // RkyvFormat, RkyvGuard implementations
        flatbuffers.rs          // FlatbuffersFormat, FlatbuffersGuard implementations
    table/
        mod.rs                  // Table trait and TableBuilder
        guards.rs               // Base guard traits and utilities
        stream.rs               // Table stream implementations
    error.rs                    // Storage-specific error types
```

## API Overview

```rust
// Storage creation
let storage = StorageBuilder::new("db.redb")
    .with_backend(DatabaseBackend::Redb)
    .build()?;

// Table creation with formats
let table = storage.new_table::<u64, CikData>("ciks")
    .with_key_format(RkyvFormat::new())
    .with_value_format(RkyvFormat::new())
    .build()?;

// Zero-copy operations
table.put(&123, &data)?;
let guard = table.get(&123)?;
if let Some(data) = guard {
    println!("Name: {}", data.name()); // Direct access to archived data
}

// Streaming
let results = table.with_stream(|stream| {
    stream.filter(|(_, data)| data.revenue() > 1000.0)
          .take(100)
          .collect::<Vec<_>>()
})?;
```

## Core Traits

```rust
trait KeyFormat<T> {
    fn to_bytes(&self, value: &T) -> Vec<u8>;
    fn from_bytes(&self, bytes: &[u8]) -> T;
}

trait ValueFormat<T> {
    type Guard<'a>: Deref<Target = Self::View<'a>>;
    type View<'a>: ?Sized;
    fn to_bytes(&self, value: &T) -> Vec<u8>;
    fn view_guard(&self, bytes: &[u8]) -> Result<Self::Guard<'_>>;
}

trait Table<K, V> {
    fn get(&self, key: &K) -> Result<Option<V::Guard<'_>>>;
    fn put(&self, key: &K, value: &V) -> Result<()>;
    fn delete(&self, key: &K) -> Result<bool>;
    fn with_stream<F, R>(&self, reader: F) -> Result<R>
        where F: FnOnce(impl Stream<Item = (K, V::Guard<'_>)>) -> R;
}
```

## Key Features

- **Zero-Copy Access**: Guards manage transaction lifetimes automatically
- **Type Safety**: Compile-time serialization guarantees
- **Pluggable Backends**: REDB implemented, others can be added
- **Format Flexibility**: Mix rkyv and flatbuffers per table
- **Streaming Support**: Efficient iteration over large datasets

## Implementation Plan

1. **Phase 1**: Error types, configuration, storage builder
2. **Phase 2**: REDB backend foundation and table infrastructure  
3. **Phase 3**: Format system and rkyv implementation
4. **Phase 4**: REDB table operations and guard system
5. **Phase 5**: Flatbuffers format integration
6. **Phase 6**: Streaming operations and batch writes
7. **Phase 7**: Advanced features (async streaming)

## Architecture

- **Backend Abstraction**: Database-agnostic table interface
- **Zero-Copy First**: Direct access to serialized data via guards
- **RAII**: Automatic transaction cleanup through guard Drop
- **Format Independence**: Trait-based serialization system
