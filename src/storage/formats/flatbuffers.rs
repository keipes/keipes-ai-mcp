
use crate::storage::StorageError;
use std::marker::PhantomData;

pub struct FlatbuffersFormat<T> {
    _phantom: PhantomData<T>,
}

impl<T> FlatbuffersFormat<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for FlatbuffersFormat<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync> super::KeyFormat<T> for FlatbuffersFormat<T> {
    fn to_bytes(&self, _value: &T) -> Result<Vec<u8>, StorageError> {
        todo!("FlatbuffersFormat::to_bytes will be implemented in Phase 5")
    }
    
    fn from_bytes(&self, _bytes: &[u8]) -> Result<T, StorageError> {
        todo!("FlatbuffersFormat::from_bytes will be implemented in Phase 5")
    }
}

impl<T: Send + Sync> super::ValueFormat<T> for FlatbuffersFormat<T> {
    fn to_bytes(&self, _value: &T) -> Result<Vec<u8>, StorageError> {
        todo!("FlatbuffersFormat::to_bytes will be implemented in Phase 5")
    }
}
