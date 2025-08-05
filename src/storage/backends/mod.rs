//! Database backend abstractions

use crate::storage::StorageError;

/// Trait for database backend implementations
pub trait StorageBackend: Send + Sync {
    /// Compact the database (optimize storage)
    fn compact(&self) -> Result<(), StorageError>;
    
    /// Create a new table with the given name and formats
    fn create_table<K, V>(
        &self,
        name: &str,
        key_format: Box<dyn crate::storage::formats::KeyFormat<K>>,
        value_format: Box<dyn crate::storage::formats::ValueFormat<V>>,
    ) -> Result<Box<dyn crate::storage::table::Table<K, V>>, StorageError>
    where
        K: Send + Sync + 'static,
        V: Send + Sync + 'static;
}

pub mod redb;
