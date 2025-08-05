use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseBackend {
    Redb,
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        DatabaseBackend::Redb
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recreate {
    Always,
    IfMissing,
    OnVersionBump,
    OnValidationFail,
}

impl Default for Recreate {
    fn default() -> Self {
        Recreate::IfMissing
    }
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub path: String,
    pub backend: DatabaseBackend,
    pub recreate: Recreate,
}

impl StorageConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().into_owned(),
            backend: DatabaseBackend::default(),
            recreate: Recreate::default(),
        }
    }
    
    pub fn with_backend(mut self, backend: DatabaseBackend) -> Self {
        self.backend = backend;
        self
    }
    
    pub fn with_recreate(mut self, recreate: Recreate) -> Self {
        self.recreate = recreate;
        self
    }
    
    pub fn validate(&self) -> Result<(), crate::storage::StorageError> {
        if self.path.is_empty() {
            return Err(crate::storage::StorageError::InvalidConfiguration(
                "Database path cannot be empty".to_string()
            ));
        }
        
        if let Some(parent) = Path::new(&self.path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| crate::storage::StorageError::InvalidConfiguration(
                        format!("Cannot create parent directory {}: {}", parent.display(), e)
                    ))?;
            }
        }
        
        Ok(())
    }
}
