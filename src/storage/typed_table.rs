// High-level typed table wrapper for simplified backend

use crate::storage::{core::*, redb_backend::RedbBackend};
use anyhow::Result;
use std::marker::PhantomData;

/// Type-safe table wrapper for RedbBackend
pub struct TypedTable<'db, K, V, S> 
where
    K: IntoKeyBytes,
    V: Clone,
    S: ValueSerializer<V>,
{
    db: &'db RedbBackend,
    table_name: String,
    serializer: S,
    _phantom: PhantomData<(K, V)>,
}

impl<'db, K, V, S> TypedTable<'db, K, V, S>
where
    K: IntoKeyBytes,
    V: Clone,
    S: ValueSerializer<V>,
{
    pub fn new(db: &'db RedbBackend, table_name: impl Into<String>, serializer: S) -> Self {
        TypedTable {
            db,
            table_name: table_name.into(),
            serializer,
            _phantom: PhantomData,
        }
    }
    
    /// Get a value by key
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        self.db.get(&self.table_name, key, &self.serializer)
    }
    
    /// Insert or update a value
    pub fn set(&self, key: &K, value: &V) -> Result<()> {
        self.db.set(&self.table_name, key, value, &self.serializer)
    }
    
    /// Remove a value by key
    pub fn remove(&self, key: &K) -> Result<bool> {
        self.db.remove(&self.table_name, key)
    }
    
    /// Check if a key exists
    pub fn contains_key(&self, key: &K) -> Result<bool> {
        self.db.contains_key(&self.table_name, key)
    }
}

// Convenience constructors for common serializers
impl<'db, K, V> TypedTable<'db, K, V, JsonSerializer>
where
    K: IntoKeyBytes,
    V: Clone + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    pub fn json(db: &'db RedbBackend, table_name: impl Into<String>) -> Self {
        TypedTable::new(db, table_name, JsonSerializer)
    }
}
