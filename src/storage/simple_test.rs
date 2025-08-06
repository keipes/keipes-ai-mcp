use rkyv::{Archive, Serialize, Deserialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
struct TestUser {
    id: u64,
    name: String,
    active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_rkyv() {
        let user = TestUser {
            id: 123,
            name: "Alice".to_string(),
            active: true,
        };

        // Serialize using rkyv high-level API
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&user).unwrap();
        println!("Serialized {} bytes", bytes.len());

        // Deserialize with zero-copy access
        let archived = unsafe { rkyv::access_unchecked::<rkyv::Archived<TestUser>>(&bytes) };
        
        // Test zero-copy access
        assert_eq!(archived.id, 123);
        assert_eq!(archived.name.as_str(), "Alice");
        assert_eq!(archived.active, true);
        
        println!("✅ rkyv zero-copy test passed!");
        println!("Name: {} (zero-copy &str)", archived.name.as_str());
        println!("ID: {} (direct u64)", archived.id);
    }
    
    #[test]
    fn test_rkyv_with_validation() {
        let user = TestUser {
            id: 42,
            name: "Bob".to_string(),
            active: false,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&user).unwrap();
        
        // Safe validated access
        let archived = rkyv::access::<rkyv::Archived<TestUser>, rkyv::rancor::Failure>(&bytes).unwrap();
        
        assert_eq!(archived.id, 42);
        assert_eq!(archived.name.as_str(), "Bob");
        assert_eq!(archived.active, false);
        
        println!("✅ rkyv with validation test passed!");
    }
}
