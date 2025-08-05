
use crate::storage::StorageError;

pub trait KeyFormat<T>: Send + Sync {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError>;

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError>;
}

pub trait ValueFormat<T>: Send + Sync {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError>;

    fn view_bytes<'a>(&self, bytes: &'a [u8]) -> Result<&'a [u8], StorageError> {
        Ok(bytes)
    }
}

pub mod flatbuffers;
pub mod rkyv;
