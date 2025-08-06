# Storage Module

A type-safe, zero-copy database abstraction layer with pluggable backends and serialization formats.

## API Overview

```rust
// Storage creation (simple)
let storage = Storage::new("db.redb")?;

// Rkyv table creation (zero-copy)
let rkyv_table = storage.new_rkyv_table::<u64, CikData>("ciks")?;

// Flatbuffers table creation (typed, validated)
let fb_table = storage.new_flatbuffers_table::<u64, Monster>("monsters")?;

// Zero-copy operations with rkyv
rkyv_table.put(&123, &data)?;
let result = rkyv_table.get(&123, |data| {
    format!("Name: {}, Revenue: {}", 
            data.name.as_str(),           // Direct &str access - TRUE ZERO COPY
            data.revenue.to_native())     // Direct u64 access
})?;
if let Some(info) = result {
    println!("{}", info);
}

// Schema validation with flatbuffers
fb_table.put(&456, &monster_data)?; // Table handles FlatBuffer building
let result = fb_table.get(&456, |monster| { // Direct typed access
    format!("HP: {}, Name: {:?}", monster.hp(), monster.name())
})?;
if let Some(info) = result {
    println!("{}", info);
}

// Scanning with zero-copy access
let rkyv_results = rkyv_table.scan(|key, data| {
    if data.revenue.to_native() > 1000.0 {
        Some((key, data.name.as_str().to_string()))
    } else {
        None
    }
})?;

// FlatBuffers scanning with validation
let fb_results = fb_table.scan(|key, monster| {
    if monster.hp() > 50 {
        Some((key, monster.name().unwrap_or("Unknown").to_string()))
    } else {
        None
    }
})?;
```

## Core Traits

```rust
/// Common backend interface - provides zero-copy access to stored bytes
trait Backend {
    fn get<F, R>(&self, table: &str, key: &[u8], accessor: F) -> Result<Option<R>, StorageError>
        where F: FnOnce(&[u8]) -> R; // Zero-copy access to bytes
    fn put(&self, table: &str, key: &[u8], value: Vec<u8>) -> Result<(), StorageError>;
    fn delete(&self, table: &str, key: &[u8]) -> Result<bool, StorageError>;
    fn scan<F>(&self, table: &str, callback: F) -> Result<(), StorageError>
        where F: FnMut(&[u8], &[u8]) -> bool; // key, value -> continue (zero-copy)
}

/// Format-specific table interfaces
trait RkyvTable<K, V> where 
    K: AsRef<[u8]> + Clone,
    V: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<256>>
{
    fn get<F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
        where F: FnOnce(&V::Archived) -> R;
    fn put(&self, key: &K, value: &V) -> Result<(), StorageError>;
    fn delete(&self, key: &K) -> Result<bool, StorageError>;
    fn scan<F, R>(&self, scanner: F) -> Result<Vec<R>, StorageError>
        where F: Fn(&K, &V::Archived) -> Option<R>;
}

trait FlatbuffersTable<K, V> where 
    K: AsRef<[u8]> + Clone,
    V: flatbuffers::Follow<'static> + 'static
{
    fn get<F, R>(&self, key: &K, accessor: F) -> Result<Option<R>, StorageError>
        where F: FnOnce(V::Inner) -> R; // Provides typed FlatBuffer access
    fn put(&self, key: &K, value: &V) -> Result<(), StorageError>; // Takes typed value, serializes internally
    fn delete(&self, key: &K) -> Result<bool, StorageError>;
    fn scan<F, R>(&self, scanner: F) -> Result<Vec<R>, StorageError>
        where F: Fn(&K, V::Inner) -> Option<R>; // Scan with typed FlatBuffer access
}

// Zero-copy data flow:
// 1. Backend provides &[u8] access to stored data
// 2. rkyv: unsafe { rkyv::access_unchecked(&bytes) } → &V::Archived
// 3. FlatBuffers: root_as_*(&bytes) → V::Inner (typed FlatBuffer access)
// 4. Tables handle serialization/deserialization internally
```

## Key Features

- **True Zero-Copy Access**: Callback-based access provides direct access to archived data without allocation
- **Format-Specific Optimization**: Rkyv for performance, Flatbuffers for schema validation
- **Type Safety**: Compile-time serialization guarantees with concrete table types
- **Pluggable Backends**: REDB implemented, others can be added
- **Zero-Copy Keys**: Direct memory access for u64, &str keys without allocation
- **Efficient Scanning**: Callback-based operations with zero-copy data access

## Zero-Copy Details

### Rkyv Format
- **String access**: `archived_string.as_str()` returns `&str` (zero allocation)
- **Primitive access**: `archived_u64.to_native()` returns `u64` (direct memory access)
- **Complex types**: Direct field access on archived structs
- **Performance**: Fastest possible access, unsafe by default

### Flatbuffers Format  
- **Validated access**: Bounds checking and schema validation
- **Direct returns**: `data.hp()` returns `i32`, `data.name()` returns `Option<&str>`
- **Schema evolution**: Forward/backward compatibility built-in
- **Safety**: Validated access, safe by default

### Key Design Decisions
- **Bytes as common format**: Both rkyv and flatbuffers serialize to `Vec<u8>`
- **Format-specific APIs**: Leverage unique strengths of each format  
- **Zero-copy keys**: Direct byte representation for database indexing
- **Backend abstraction**: Database stores bytes, formats handle access patterns

## Implementation Plan

1. **Phase 1**: Error types, simple configuration ✅
2. **Phase 2**: REDB backend with callback interface ✅  
3. **Phase 3**: Rkyv table with zero-copy access ✅
4. **Phase 4**: FlatBuffers table with validation
5. **Phase 5**: Storage facade and scanning operations

Target: Single file under 300 lines for core functionality.

## Architecture

- **Callback-Based Access**: Direct access to archived data within transaction scope
- **True Zero-Copy**: Direct memory access via rkyv archived types
- **Transaction Safety**: Callback lifetime ensures proper resource cleanup
- **Key Optimization**: Zero-allocation key access for database indexing
- **Backend Abstraction**: Database-agnostic storage with format specialization

## Usage Examples

### Data Definition
```rust
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct CikData {
    name: String,
    revenue: f64,
    employees: u32,
}

// Zero-copy access patterns:
let result = table.get(&cik, |data| {
    (
        data.name.as_str().to_string(),        // Zero-copy string access
        data.revenue.to_native(),              // Direct numeric access  
        data.employees.to_native()             // Direct numeric access
    )
})?;
if let Some((name, revenue, employees)) = result {
    println!("Company: {}, Revenue: {}, Employees: {}", name, revenue, employees);
}
```

### Performance Characteristics
- **Rkyv**: ~0ns deserialization, direct memory access, callback-based
- **Flatbuffers**: Validated access, schema-aware, ~10ns overhead
- **Keys**: Zero-copy for u64, &str - direct byte representation
- **Callbacks**: Stack-allocated, zero-cost abstractions
