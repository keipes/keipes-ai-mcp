
use crate::storage::StorageError;
use std::marker::PhantomData;

pub struct RkyvFormat<T> {
    _phantom: PhantomData<T>,
}

impl<T> RkyvFormat<T> {
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
        todo!("RkyvFormat::to_bytes will be implemented in Phase 3")
    }
    
    fn from_bytes(&self, _bytes: &[u8]) -> Result<T, StorageError> {
        todo!("RkyvFormat::from_bytes will be implemented in Phase 3")
    }
}

impl<T: Send + Sync> super::ValueFormat<T> for RkyvFormat<T> {
    fn to_bytes(&self, _value: &T) -> Result<Vec<u8>, StorageError> {
        todo!("RkyvFormat::to_bytes will be implemented in Phase 3")
    }
}
