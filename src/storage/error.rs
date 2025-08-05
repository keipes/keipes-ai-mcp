use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Database(String),

    Serialization(String),

    Deserialization(String),

    InvalidConfiguration(String),

    Io(std::io::Error),

    MissingFormat(String),

    TableOperation(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Database(msg) => write!(f, "Database error: {}", msg),
            StorageError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            StorageError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            StorageError::Io(err) => write!(f, "I/O error: {}", err),
            StorageError::MissingFormat(msg) => write!(f, "Missing format: {}", msg),
            StorageError::TableOperation(msg) => write!(f, "Table operation error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<redb::Error> for StorageError {
    fn from(err: redb::Error) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::DatabaseError> for StorageError {
    fn from(err: redb::DatabaseError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::TransactionError> for StorageError {
    fn from(err: redb::TransactionError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::TableError> for StorageError {
    fn from(err: redb::TableError) -> Self {
        StorageError::TableOperation(err.to_string())
    }
}

impl From<redb::StorageError> for StorageError {
    fn from(err: redb::StorageError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::CompactionError> for StorageError {
    fn from(err: redb::CompactionError) -> Self {
        StorageError::Database(err.to_string())
    }
}
