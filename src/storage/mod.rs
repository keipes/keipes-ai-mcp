use anyhow::Result;
use redb::{Database, TableDefinition};
use std::path::Path;
pub mod table;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recreate {
    Always,
    IfMissing,
}
pub struct StorageConfig {
    pub db_path: String,
    pub recreate: Recreate,
}

impl StorageConfig {
    pub fn new(db_path: String, recreate: Recreate) -> Self {
        Self { db_path, recreate }
    }

    pub fn recreate(mut self) -> Self {
        self.recreate = Recreate::Always;
        self
    }
}

// Example table definitions - customize for your use case
const SESSIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("sessions");
const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("cache");

pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        // Remove the file if recreate is requested
        if config.recreate == Recreate::Always && Path::new(&config.db_path).exists() {
            std::fs::remove_file(&config.db_path)?;
        }

        // Create or open the database
        let db = Database::create(&config.db_path)?;

        // Initialize tables (they're created automatically when first accessed)
        let write_txn = db.begin_write()?;
        {
            // Just open the tables to ensure they exist
            let _sessions = write_txn.open_table(SESSIONS_TABLE)?;
            let _cache = write_txn.open_table(CACHE_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db })
    }

    // Example methods for session storage
    pub fn store_session(&self, session_id: &str, data: &str) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            table.insert(session_id, data)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SESSIONS_TABLE)?;

        if let Some(data) = table.get(session_id)? {
            Ok(Some(data.value().to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn remove_session(&self, session_id: &str) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SESSIONS_TABLE)?;
            table.remove(session_id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // Example methods for cache storage
    pub fn store_cache(&self, key: &str, data: &[u8]) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            table.insert(key, data)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_cache(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CACHE_TABLE)?;

        if let Some(data) = table.get(key)? {
            Ok(Some(data.value().to_vec()))
        } else {
            Ok(None)
        }
    }

    // Manually trigger compaction (redb does this automatically, but you can force it)
    pub fn compact(&mut self) -> Result<()> {
        self.db.compact()?;
        Ok(())
    }

    // Simple info about the database
    pub fn info(&self) -> String {
        "redb database ready".to_string()
    }
}

// Async wrapper function
pub async fn create_storage(config: StorageConfig) -> Result<Storage> {
    // redb is sync, but we can wrap it in spawn_blocking if needed
    let storage = tokio::task::spawn_blocking(move || Storage::new(config)).await??;
    Ok(storage)
}
