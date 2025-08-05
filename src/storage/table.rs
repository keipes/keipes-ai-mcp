use anyhow::Result;
use rkyv::{
    access, access_unchecked,
    api::high::{to_bytes_in, HighDeserializer, HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    rancor::Failure,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
    Archive, Serialize as RkyvSerialize,
};
use serde::{de::DeserializeOwned, Serialize};
use std::mem::take;

// Type aliases for consistent high-level API usage (like in the benchmark)
pub type TableSerializer<'a> = HighSerializer<AlignedVec, ArenaHandle<'a>, Failure>;
pub type TableDeserializer = HighDeserializer<Failure>;
pub type TableValidator<'a> = HighValidator<'a, Failure>;

/// Generic adaptable table interface that can work with different key-value types
pub trait Table<K, V>
where
    K: Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Get a value by key
    fn get(&self, key: &K) -> Result<Option<V>>;

    /// Put a key-value pair
    fn put(&self, key: &K, value: &V) -> Result<()>;

    /// Remove a key-value pair
    fn remove(&self, key: &K) -> Result<Option<V>>;
}

/// Enhanced table interface that supports both traditional serde and high-performance rkyv access
/// Inspired by patterns from the rkyv benchmark
pub trait RkyvTable<K, V>
where
    K: Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Traditional get that deserializes the full value
    fn get(&self, key: &K) -> Result<Option<V>>;

    /// High-performance get that works with archived data directly
    /// The reader function gets access to zero-copy archived data
    fn get_archived<F, R>(&self, key: &K, reader: F) -> Result<Option<R>>
    where
        V: Archive,
        V::Archived: for<'a> CheckBytes<TableValidator<'a>>,
        F: FnOnce(&V::Archived) -> Result<R>;

    /// Fast unsafe get for trusted data sources
    fn get_archived_unsafe<F, R>(&self, key: &K, reader: F) -> Result<Option<R>>
    where
        V: Archive,
        F: FnOnce(&V::Archived) -> Result<R>;

    /// Put with automatic serialization choice based on type
    fn put(&self, key: &K, value: &V) -> Result<()>;

    /// Remove a key-value pair
    fn remove(&self, key: &K) -> Result<Option<V>>;

    /// Batch operations for better performance
    fn batch_get_archived<F, R>(&self, keys: &[K], reader: F) -> Result<Vec<Option<R>>>
    where
        V: Archive,
        V::Archived: for<'a> CheckBytes<TableValidator<'a>>,
        F: Fn(&V::Archived) -> Result<R>;
}

/// Buffer-reusable serializer for high-performance scenarios
/// Inspired by the rkyv benchmark's buffer reuse pattern
pub struct RkyvSerializer {
    buffer: AlignedVec,
}

impl RkyvSerializer {
    pub fn new() -> Self {
        Self {
            buffer: AlignedVec::with_capacity(1_000_000), // 1MB default
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: AlignedVec::with_capacity(capacity),
        }
    }

    /// Serialize with buffer reuse - much faster for repeated operations
    pub fn serialize<T>(&mut self, data: &T) -> Result<&[u8]>
    where
        T: for<'a> RkyvSerialize<TableSerializer<'a>>,
    {
        self.buffer.clear();
        self.buffer = to_bytes_in(data, take(&mut self.buffer))
            .map_err(|e| anyhow::anyhow!("Failed to serialize with rkyv: {}", e))?;
        Ok(self.buffer.as_ref())
    }

    /// Get a copy of the serialized bytes (if you need ownership)
    pub fn serialize_to_vec<T>(&mut self, data: &T) -> Result<Vec<u8>>
    where
        T: for<'a> RkyvSerialize<TableSerializer<'a>>,
    {
        Ok(self.serialize(data)?.to_vec())
    }
}

/// Enhanced access patterns inspired by rkyv benchmark
/// Provides both validated and unvalidated access with different performance characteristics

/// Fast unsafe access - maximum performance for trusted data
pub fn access_rkyv_unsafe<T, F, R>(bytes: &[u8], accessor: F) -> Result<R>
where
    T: Archive,
    F: FnOnce(&T::Archived) -> Result<R>,
{
    let archived = unsafe { access_unchecked::<T::Archived>(bytes) };
    accessor(archived)
}

/// Safe validated access - slower but secure for untrusted data
pub fn access_rkyv_safe<T, F, R>(bytes: &[u8], accessor: F) -> Result<R>
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<TableValidator<'a>>,
    F: FnOnce(&T::Archived) -> Result<R>,
{
    let archived = access::<T::Archived, Failure>(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to access archived data: {}", e))?;
    accessor(archived)
}

/// Read-only operation on archived data (like benchmark's read pattern)
pub fn read_rkyv<T, F>(bytes: &[u8], reader: F, validate: bool) -> Result<()>
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<TableValidator<'a>>,
    F: Fn(&T::Archived),
{
    if validate {
        access_rkyv_safe::<T, _, _>(bytes, |archived| {
            reader(archived);
            Ok(())
        })
    } else {
        access_rkyv_unsafe::<T, _, _>(bytes, |archived| {
            reader(archived);
            Ok(())
        })
    }
}

///
/// # Example
/// ```rust
/// use rkyv::{Archive, Serialize as RkyvSerialize};
///
/// #[derive(Archive, RkyvSerialize)]
/// struct MyData {
///     value: i32,
/// }
///
/// let data = MyData { value: 42 };
/// let bytes = serialize_rkyv(&data)?;
/// ```
pub fn serialize_rkyv<T>(data: &T) -> Result<Vec<u8>>
where
    T: for<'a> RkyvSerialize<
        rkyv::rancor::Strategy<
            rkyv::ser::Serializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::ser::sharing::Share,
            >,
            rkyv::rancor::BoxedError,
        >,
    >,
{
    rkyv::to_bytes(data)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| anyhow::anyhow!("Failed to serialize with rkyv: {}", e))
}

/// Deserialize data using rkyv (unsafe - no validation)
/// The deserializer function is called with the archived data to limit scope
///
/// # Example
/// ```rust
/// use rkyv::{Archive, Serialize as RkyvSerialize};
///
/// #[derive(Archive, RkyvSerialize)]
/// struct MyData {
///     value: i32,
/// }
///
/// let result = deserialize_rkyv_unsafe::<MyData, _, _>(&bytes, |archived| {
///     Ok(archived.value)
/// })?;
/// ```
///
/// # Safety
/// This function uses `access_unchecked` and does not validate the archived data.
/// Only use with trusted data sources.
pub fn deserialize_rkyv_unsafe<T, F, R>(bytes: &[u8], deserializer: F) -> Result<R>
where
    T: Archive,
    F: FnOnce(&T::Archived) -> Result<R>,
{
    let archived = unsafe { rkyv::access_unchecked::<T::Archived>(bytes) };
    deserializer(archived)
}

// ==== USAGE EXAMPLES inspired by rkyv benchmark patterns ====

#[cfg(test)]
mod examples {
    use super::*;
    use rkyv::{Archive, Serialize as RkyvSerialize};

    #[derive(Archive, RkyvSerialize, Debug, PartialEq)]
    struct UserProfile {
        id: u64,
        name: String,
        scores: Vec<i32>,
    }

    /// Example 1: High-performance serializer with buffer reuse
    #[allow(dead_code)]
    fn example_buffer_reuse() -> Result<()> {
        let mut serializer = RkyvSerializer::with_capacity(10_000);

        let user1 = UserProfile {
            id: 1,
            name: "Alice".to_string(),
            scores: vec![100, 95, 88],
        };

        let user2 = UserProfile {
            id: 2,
            name: "Bob".to_string(),
            scores: vec![92, 87, 94],
        };

        // Serialize multiple items reusing the same buffer (much faster)
        let bytes1 = serializer.serialize(&user1)?;
        println!("User1 serialized to {} bytes", bytes1.len());

        let bytes2 = serializer.serialize(&user2)?;
        println!("User2 serialized to {} bytes", bytes2.len());

        Ok(())
    }

    /// Example 2: Zero-copy access patterns
    #[allow(dead_code)]
    fn example_zero_copy_access() -> Result<()> {
        let mut serializer = RkyvSerializer::new();
        let user = UserProfile {
            id: 42,
            name: "Charlie".to_string(),
            scores: vec![100, 99, 98, 97],
        };

        let bytes = serializer.serialize_to_vec(&user)?;

        // Fast unsafe access (like benchmark's unvalidated path)
        let total_score = access_rkyv_unsafe::<UserProfile, _, _>(&bytes, |archived| {
            let sum: i32 = archived.scores.iter().map(|x| i32::from(*x)).sum();
            Ok(sum)
        })?;
        println!("Total score (unsafe): {}", total_score);

        // Safe validated access (like benchmark's validated path)
        let max_score = access_rkyv_safe::<UserProfile, _, _>(&bytes, |archived| {
            let max = archived
                .scores
                .iter()
                .map(|x| i32::from(*x))
                .max()
                .unwrap_or(0);
            Ok(max)
        })?;
        println!("Max score (safe): {}", max_score);

        // Read pattern (like benchmark's read operations)
        read_rkyv::<UserProfile, _>(
            &bytes,
            |archived| {
                println!(
                    "User: {} has {} scores",
                    archived.name,
                    archived.scores.len()
                );
            },
            false,
        )?; // false = no validation for speed

        Ok(())
    }

    /// Example 3: How this could enhance a table implementation
    #[allow(dead_code)]
    fn example_enhanced_table_usage() {
        // Pseudocode showing how RkyvTable could be used:

        // let table: Box<dyn RkyvTable<u64, UserProfile>> = /* some implementation */;

        // // Traditional access
        // let user = table.get(&42)?;

        // // High-performance zero-copy access
        // let score_sum = table.get_archived(&42, |archived| {
        //     Ok(archived.scores.iter().sum::<i32>())
        // })?;

        // // Ultra-fast unsafe access for trusted scenarios
        // let name_len = table.get_archived_unsafe(&42, |archived| {
        //     Ok(archived.name.len())
        // })?;

        // // Batch operations
        // let all_scores = table.batch_get_archived(&[1, 2, 3], |archived| {
        //     Ok(archived.scores.clone())
        // })?;
    }
}
