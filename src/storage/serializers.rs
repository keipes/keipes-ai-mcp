use rkyv::rancor;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum StorageError {
    Serialization(String),
}

pub trait ValueSerializer<T> {
    type Output<'a>;
    fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError>;
    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<Self::Output<'a>, StorageError>;
}

// Simplified rkyv serializer
pub struct RkyvSerializer<T> {
    _phantom: PhantomData<T>,
}

impl<T> RkyvSerializer<T> {
    pub fn new() -> Self {
        RkyvSerializer {
            _phantom: PhantomData,
        }
    }
}

impl<T> ValueSerializer<T> for RkyvSerializer<T>
where
    T: for<'a> rkyv::Serialize<
        rkyv::api::high::HighSerializer<
            rkyv::util::AlignedVec,
            rkyv::ser::allocator::ArenaHandle<'a>,
            rancor::Error,
        >,
    >,
    T: rkyv::Archive,
    T::Archived: rkyv::Portable,
{
    type Output<'a> = &'a T::Archived;

    fn serialize(&self, value: &T) -> Result<Vec<u8>, StorageError> {
        let bytes = rkyv::to_bytes::<rancor::Error>(value)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        Ok(bytes.into_vec())
    }

    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<&'a T::Archived, StorageError> {
        unsafe { Ok(rkyv::access_unchecked::<T::Archived>(bytes)) }
    }
}

// FlatBuffers serializer for pre-serialized data
pub struct FlatbuffersSerializer<T>
where
    T: flatbuffers::Follow<'static> + flatbuffers::Verifiable + 'static,
{
    _phantom: PhantomData<T>,
}

impl<T> FlatbuffersSerializer<T>
where
    T: flatbuffers::Follow<'static> + flatbuffers::Verifiable + 'static,
{
    pub fn new() -> Self {
        FlatbuffersSerializer {
            _phantom: PhantomData,
        }
    }
}

impl<T> ValueSerializer<&[u8]> for FlatbuffersSerializer<T>
where
    T: flatbuffers::Follow<'static> + flatbuffers::Verifiable + 'static,
{
    type Output<'a> = T::Inner;

    fn serialize(&self, value: &&[u8]) -> Result<Vec<u8>, StorageError> {
        // Just store the pre-serialized FlatBuffer bytes
        Ok((*value).to_vec())
    }

    fn deserialize<'a>(&self, bytes: &'a [u8]) -> Result<T::Inner, StorageError> {
        flatbuffers::root::<T>(bytes).map_err(|e| StorageError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test data structure for rkyv
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, PartialEq)]
    struct TestUser {
        id: u64,
        name: String,
        age: u32,
        active: bool,
    }

    #[test]
    fn test_rkyv_serializer() {
        let serializer = RkyvSerializer::<TestUser>::new();

        let user = TestUser {
            id: 123,
            name: "Alice".to_string(),
            age: 30,
            active: true,
        };

        // Serialize
        let bytes = serializer.serialize(&user).unwrap();
        println!("Serialized {} bytes", bytes.len());

        // Deserialize with zero-copy access
        let archived = serializer.deserialize(&bytes).unwrap();

        // Direct field access - TRUE ZERO COPY
        assert_eq!(archived.id, 123);
        assert_eq!(archived.name.as_str(), "Alice");
        assert_eq!(archived.age, 30);
        assert_eq!(archived.active, true);

        println!("✅ rkyv zero-copy access works!");
        println!("Name: {} (zero-copy &str)", archived.name.as_str());
        println!("ID: {} (direct u64)", archived.id);
    }
}
