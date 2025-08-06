// Integration tests for the simplified storage system

use crate::storage::{core::*, redb_backend::*, typed_table::*};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redb_backend_basic() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        // Create database
        let db = RedbBackend::new(&db_path)?;
        
        // Test basic operations
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            active: true,
        };
        
        // Create typed table with JSON serialization
        let users_table = TypedTable::json(&db, "users");
        
        // Insert user
        users_table.set(&user.id, &user)?;
        
        // Retrieve user
        let retrieved = users_table.get(&user.id)?;
        assert_eq!(retrieved, Some(user.clone()));
        
        // Check existence
        assert!(users_table.contains_key(&user.id)?);
        assert!(!users_table.contains_key(&999u64)?);
        
        // Remove user
        let existed = users_table.remove(&user.id)?;
        assert!(existed);
        
        // Verify removal
        let retrieved = users_table.get(&user.id)?;
        assert_eq!(retrieved, None);
        
        println!("✅ redb backend basic operations test passed!");
        Ok(())
    }

    #[test]
    fn test_multiple_tables() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_multi.db");
        
        let db = RedbBackend::new(&db_path)?;
        
        // Create multiple tables
        let users_table = TypedTable::json(&db, "users");
        let settings_table = TypedTable::json(&db, "settings");
        
        // Insert into different tables
        let user = User {
            id: 1,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            active: true,
        };
        
        let setting_key = "theme".to_string();
        let setting_value = "dark".to_string();
        
        users_table.set(&user.id, &user)?;
        settings_table.set(&setting_key, &setting_value)?;
        
        // Retrieve from different tables
        let retrieved_user = users_table.get(&user.id)?;
        let retrieved_setting = settings_table.get(&setting_key)?;
        
        assert_eq!(retrieved_user, Some(user));
        assert_eq!(retrieved_setting, Some(setting_value));
        
        println!("✅ Multiple tables test passed!");
        Ok(())
    }

    #[test]
    fn test_string_keys() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_string_keys.db");
        
        let db = RedbBackend::new(&db_path)?;
        let config_table = TypedTable::json(&db, "config");
        
        // Test string keys
        let keys_values = vec![
            ("database.host".to_string(), "localhost".to_string()),
            ("database.port".to_string(), "5432".to_string()),
            ("app.name".to_string(), "MyApp".to_string()),
        ];
        
        // Insert all key-value pairs
        for (key, value) in &keys_values {
            config_table.set(key, value)?;
        }
        
        // Retrieve and verify
        for (key, expected_value) in &keys_values {
            let retrieved = config_table.get(key)?;
            assert_eq!(retrieved, Some(expected_value.clone()));
        }
        
        println!("✅ String keys test passed!");
        Ok(())
    }

    #[test]
    fn test_direct_backend_api() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_direct.db");
        
        let db = RedbBackend::new(&db_path)?;
        let serializer = JsonSerializer;
        
        let key = 42u64;
        let value = "Hello, World!".to_string();
        
        // Test direct backend API
        db.set("direct_test", &key, &value, &serializer)?;
        
        let retrieved = db.get("direct_test", &key, &serializer)?;
        assert_eq!(retrieved, Some(value.clone()));
        
        let exists = db.contains_key("direct_test", &key)?;
        assert!(exists);
        
        let removed = db.remove("direct_test", &key)?;
        assert!(removed);
        
        let after_removal: Option<String> = db.get("direct_test", &key, &serializer)?;
        assert_eq!(after_removal, None);
        
        println!("✅ Direct backend API test passed!");
        Ok(())
    }
}
