//! Serialization format abstractions

use crate::storage::StorageError;

/// Trait for key serialization/deserialization
pub trait KeyFormat<T>: Send + Sync {
    /// Serialize a key to bytes
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError>;

    /// Deserialize a key from bytes
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError>;
}

/// Trait for value serialization with zero-copy deserialization
pub trait ValueFormat<T>: Send + Sync {
    /// Serialize a value to bytes
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError>;

    /// Create a zero-copy view of the value (returns raw bytes for now)
    fn view_bytes<'a>(&self, bytes: &'a [u8]) -> Result<&'a [u8], StorageError> {
        // For Phase 1, just return the raw bytes
        // Real implementations will deserialize in Phase 3+
        Ok(bytes)
    }
}

pub mod flatbuffers;
pub mod rkyv;
