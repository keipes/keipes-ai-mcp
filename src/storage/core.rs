// Core storage traits and implementations

use anyhow::Result;

/// Key serialization
pub trait IntoKeyBytes {
    fn into_key_bytes(&self) -> Vec<u8>;
}

/// Key deserialization  
pub trait FromKeyBytes: Sized {
    fn from_key_bytes(bytes: &[u8]) -> Result<Self>;
}

/// Value serialization
pub trait ValueSerializer<T> {
    fn serialize(&self, value: &T) -> Result<Vec<u8>>;
    fn deserialize(&self, bytes: &[u8]) -> Result<T>;
}

// Implement key serialization for common types
impl IntoKeyBytes for u64 {
    fn into_key_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl FromKeyBytes for u64 {
    fn from_key_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 8 {
            return Err(anyhow::anyhow!("Invalid u64 key length: {}", bytes.len()));
        }
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_be_bytes(array))
    }
}

impl IntoKeyBytes for String {
    fn into_key_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl FromKeyBytes for String {
    fn from_key_bytes(bytes: &[u8]) -> Result<Self> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in string key: {}", e))
    }
}

impl IntoKeyBytes for &str {
    fn into_key_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

/// Simple JSON value serializer for any serde type
pub struct JsonSerializer;

impl<T> ValueSerializer<T> for JsonSerializer
where T: serde::Serialize + for<'de> serde::Deserialize<'de>
{
    fn serialize(&self, value: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(value)
            .map_err(|e| anyhow::anyhow!("JSON serialization failed: {}", e))
    }
    
    fn deserialize(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes)
            .map_err(|e| anyhow::anyhow!("JSON deserialization failed: {}", e))
    }
}
