use std::path::Path;

/// Database backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseBackend {
    /// REDB embedded database
    Redb,
}

impl Default for DatabaseBackend {
    fn default() -> Self {
        DatabaseBackend::Redb
    }
}

/// Database recreation policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recreate {
    /// Always recreate the database (delete existing)
    Always,
    /// Only create if database doesn't exist
    IfMissing,
    /// Recreate on version bump (future feature)
    OnVersionBump,
    /// Recreate if validation fails (future feature)
    OnValidationFail,
}

impl Default for Recreate {
    fn default() -> Self {
        Recreate::IfMissing
    }
}

/// Complete storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Path to the database file or directory
    pub path: String,
    /// Database backend to use
    pub backend: DatabaseBackend,
    /// Recreation policy
    pub recreate: Recreate,
}

impl StorageConfig {
    /// Create a new storage configuration
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().into_owned(),
            backend: DatabaseBackend::default(),
            recreate: Recreate::default(),
        }
    }
    
    /// Set the database backend
    pub fn with_backend(mut self, backend: DatabaseBackend) -> Self {
        self.backend = backend;
        self
    }
    
    /// Set the recreation policy
    pub fn with_recreate(mut self, recreate: Recreate) -> Self {
        self.recreate = recreate;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::storage::StorageError> {
        // Validate path
        if self.path.is_empty() {
            return Err(crate::storage::StorageError::InvalidConfiguration(
                "Database path cannot be empty".to_string()
            ));
        }
        
        // Check if parent directory exists or can be created
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
