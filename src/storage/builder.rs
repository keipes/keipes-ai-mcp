use crate::storage::{StorageConfig, StorageError, DatabaseBackend, Recreate};
use std::path::Path;

#[derive(Debug)]
pub struct StorageBuilder {
    config: StorageConfig,
}

impl StorageBuilder {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            config: StorageConfig::new(path),
        }
    }
    
    pub fn with_backend(mut self, backend: DatabaseBackend) -> Self {
        self.config = self.config.with_backend(backend);
        self
    }
    
    pub fn with_recreate(mut self, recreate: Recreate) -> Self {
        self.config = self.config.with_recreate(recreate);
        self
    }
    
    pub fn build(self) -> Result<Storage, StorageError> {
        self.config.validate()?;
        
        self.handle_recreation()?;
        
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
            }
            Recreate::OnVersionBump | Recreate::OnValidationFail => {
            }
        }
        
        Ok(())
    }
}

pub struct Storage {
    backend: crate::storage::backends::redb::RedbStorage,
}

impl Storage {
    pub(crate) fn new(backend: crate::storage::backends::redb::RedbStorage) -> Self {
        Self { backend }
    }
    
    pub fn new_table<K, V>(&self, name: &str) -> crate::storage::table::TableBuilder<K, V> 
    where
        K: Send + Sync + 'static,
        V: Send + Sync + 'static,
    {
        crate::storage::table::TableBuilder::new(self, name)
    }
    
    pub fn compact(&self) -> Result<(), StorageError> {
        self.backend.compact()
    }
    
    pub(crate) fn backend(&self) -> &crate::storage::backends::redb::RedbStorage {
        &self.backend
    }
}
