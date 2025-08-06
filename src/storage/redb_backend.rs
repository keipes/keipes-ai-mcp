// Simplified redb backend implementation

use crate::storage::core::{IntoKeyBytes, ValueSerializer};
use anyhow::Result;
use redb::{Database as RedbDatabase, TableDefinition};
use std::path::Path;

/// Simplified redb database wrapper
pub struct RedbBackend {
    db: RedbDatabase,
}

impl RedbBackend {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = RedbDatabase::create(path)?;
        Ok(RedbBackend { db })
    }
    
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = RedbDatabase::open(path)?;
        Ok(RedbBackend { db })
    }
    
    /// Get a value by key using a serializer
    pub fn get<K, V, S>(&self, table_name: &str, key: &K, serializer: &S) -> Result<Option<V>>
    where
        K: IntoKeyBytes,
        S: ValueSerializer<V>,
    {
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(table_def)?;
        
        let key_bytes = key.into_key_bytes();
        if let Some(value_guard) = table.get(key_bytes.as_slice())? {
            let value_bytes = value_guard.value();
            let value = serializer.deserialize(value_bytes)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    /// Set a value by key using a serializer
    pub fn set<K, V, S>(&self, table_name: &str, key: &K, value: &V, serializer: &S) -> Result<()>
    where
        K: IntoKeyBytes,
        S: ValueSerializer<V>,
    {
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(table_def)?;
            let key_bytes = key.into_key_bytes();
            let value_bytes = serializer.serialize(value)?;
            table.insert(key_bytes.as_slice(), value_bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
    
    /// Remove a value by key
    pub fn remove<K>(&self, table_name: &str, key: &K) -> Result<bool>
    where
        K: IntoKeyBytes,
    {
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        let write_txn = self.db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(table_def)?;
            let key_bytes = key.into_key_bytes();
            let result = table.remove(key_bytes.as_slice())?;
            result.is_some()
        };
        write_txn.commit()?;
        Ok(existed)
    }
    
    /// Check if a key exists
    pub fn contains_key<K>(&self, table_name: &str, key: &K) -> Result<bool>
    where
        K: IntoKeyBytes,
    {
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(table_def)?;
        
        let key_bytes = key.into_key_bytes();
        Ok(table.get(key_bytes.as_slice())?.is_some())
    }
}
