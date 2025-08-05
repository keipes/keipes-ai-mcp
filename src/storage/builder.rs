use crate::storage::{StorageConfig, StorageError, DatabaseBackend, Recreate};
use std::path::Path;

/// Builder for creating storage instances with fluent API
#[derive(Debug)]
pub struct StorageBuilder {
    config: StorageConfig,
}

impl StorageBuilder {
    /// Create a new storage builder with the given database path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            config: StorageConfig::new(path),
        }
    }
    
    /// Set the database backend (currently only REDB is supported)
    pub fn with_backend(mut self, backend: DatabaseBackend) -> Self {
        self.config = self.config.with_backend(backend);
        self
    }
    
    /// Set the database recreation policy
    pub fn with_recreate(mut self, recreate: Recreate) -> Self {
        self.config = self.config.with_recreate(recreate);
        self
    }
    
    /// Build the storage instance
    pub fn build(self) -> Result<Storage, StorageError> {
        // Validate configuration
        self.config.validate()?;
        
        // Handle recreation policy
        self.handle_recreation()?;
        
        // Create storage based on backend
        match self.config.backend {
            DatabaseBackend::Redb => {
                let backend = crate::storage::backends::redb::RedbStorage::new(self.config)?;
                Ok(Storage::new(backend))
            }
        }
    }
    
    fn handle_recreation(&self) -> Result<(), StorageError> {
        let path = Path::new(&self.config.path);
        
        match self.config.recreate {
            Recreate::Always => {
                if path.exists() {
                    if path.is_dir() {
                        std::fs::remove_dir_all(path)?;
                    } else {
                        std::fs::remove_file(path)?;
                    }
                }
            }
            Recreate::IfMissing => {
                // Do nothing - let the database handle existing files
            }
            Recreate::OnVersionBump | Recreate::OnValidationFail => {
                // Future implementation - for now treat as IfMissing
            }
        }
        
        Ok(())
    }
}

/// Main storage interface (simplified for Phase 1 - REDB only)
pub struct Storage {
    backend: crate::storage::backends::redb::RedbStorage,
}

impl Storage {
    pub(crate) fn new(backend: crate::storage::backends::redb::RedbStorage) -> Self {
        Self { backend }
    }
    
    /// Create a new table builder
    pub fn new_table<K, V>(&self, name: &str) -> crate::storage::table::TableBuilder<K, V> 
    where
        K: Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        crate::storage::table::TableBuilder::new(self, name)
    }
    
    /// Compact the database (if supported by backend)
    pub fn compact(&self) -> Result<(), StorageError> {
        self.backend.compact()
    }
    
    /// Get a reference to the backend for table creation
    pub(crate) fn backend(&self) -> &crate::storage::backends::redb::RedbStorage {
        &self.backend
    }
}
