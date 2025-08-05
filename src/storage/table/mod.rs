//! Table abstractions and builder

use crate::storage::{Storage, StorageError};
use std::marker::PhantomData;

/// Trait for table operations (simplified for Phase 1)
pub trait Table<K, V>: Send + Sync {
    /// Get a value by key (returns raw bytes for now)
    fn get(&self, key: &K) -> Result<Option<Vec<u8>>, StorageError>;
    
    /// Put a key-value pair
    fn put(&self, key: &K, value: &V) -> Result<(), StorageError>;
    
    /// Delete a key-value pair, returns true if the key existed
    fn delete(&self, key: &K) -> Result<bool, StorageError>;
}

/// Builder for creating tables with specific formats
pub struct TableBuilder<K, V> {
    storage: *const Storage,
    name: String,
    key_format: Option<Box<dyn crate::storage::formats::KeyFormat<K>>>,
    value_format: Option<Box<dyn crate::storage::formats::ValueFormat<V>>>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> TableBuilder<K, V> 
where 
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    /// Create a new table builder
    pub(crate) fn new(storage: &Storage, name: &str) -> Self {
        Self {
            storage: storage as *const Storage,
            name: name.to_string(),
            key_format: None,
            value_format: None,
            _phantom: PhantomData,
        }
    }
    
    /// Set the key format
    pub fn with_key_format<F>(mut self, format: F) -> Self
    where
        F: crate::storage::formats::KeyFormat<K> + 'static,
    {
        self.key_format = Some(Box::new(format));
        self
    }
    
    /// Set the value format
    pub fn with_value_format<F>(mut self, format: F) -> Self
    where
        F: crate::storage::formats::ValueFormat<V> + 'static,
    {
        self.value_format = Some(Box::new(format));
        self
    }
    
    /// Build the table
    pub fn build(self) -> Result<Box<dyn Table<K, V>>, StorageError> {
        let storage = unsafe { &*self.storage };
        
        let key_format = self.key_format.ok_or_else(|| {
            StorageError::MissingFormat("Key format is required".to_string())
        })?;
        
        let value_format = self.value_format.ok_or_else(|| {
            StorageError::MissingFormat("Value format is required".to_string())
        })?;
        
        // For Phase 1, directly use the concrete REDB backend
        storage.backend().create_table(&self.name, key_format, value_format)
    }
}

pub mod guards;
pub mod stream;
