//! Rkyv format implementation (stub for Phase 3)

use crate::storage::StorageError;
use std::marker::PhantomData;

/// Rkyv serialization format
pub struct RkyvFormat<T> {
    _phantom: PhantomData<T>,
}

impl<T> RkyvFormat<T> {
    /// Create a new rkyv format instance
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for RkyvFormat<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync> super::KeyFormat<T> for RkyvFormat<T> {
    fn to_bytes(&self, _value: &T) -> Result<Vec<u8>, StorageError> {
        // TODO: Implement in Phase 3
        todo!("RkyvFormat::to_bytes will be implemented in Phase 3")
    }
    
    fn from_bytes(&self, _bytes: &[u8]) -> Result<T, StorageError> {
        // TODO: Implement in Phase 3
        todo!("RkyvFormat::from_bytes will be implemented in Phase 3")
    }
}

impl<T: Send + Sync> super::ValueFormat<T> for RkyvFormat<T> {
    fn to_bytes(&self, _value: &T) -> Result<Vec<u8>, StorageError> {
        // TODO: Implement in Phase 3
        todo!("RkyvFormat::to_bytes will be implemented in Phase 3")
    }
}
