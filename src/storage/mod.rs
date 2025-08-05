//! Storage module providing type-safe, zero-copy database abstraction.

pub use builder::{Storage, StorageBuilder};
pub use config::{DatabaseBackend, Recreate, StorageConfig};
pub use error::StorageError;

// Re-export format types for convenience
pub use formats::{KeyFormat, ValueFormat};
pub use formats::rkyv::RkyvFormat;
pub use formats::flatbuffers::FlatbuffersFormat;

// Re-export table types
pub use table::{Table, TableBuilder};

pub mod builder;
pub mod config;
pub mod error;

pub mod backends;
pub mod formats;
pub mod table;
