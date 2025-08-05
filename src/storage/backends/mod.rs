
use crate::storage::StorageError;

pub trait StorageBackend: Send + Sync {
    fn compact(&self) -> Result<(), StorageError>;
    
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
