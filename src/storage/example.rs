// Complete example demonstrating the storage system

use crate::storage::{core::*, redb_backend::*, typed_table::*};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Customer {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub created_at: u64, // Unix timestamp
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Order {
    pub id: u64,
    pub customer_id: u64,
    pub items: Vec<String>,
    pub total: f64,
    pub created_at: u64,
}

/// Example application using the storage system
pub struct CustomerStore {
    db: RedbBackend,
}

impl CustomerStore {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = RedbBackend::new(db_path)?;
        Ok(CustomerStore { db })
    }
    
    pub fn open(db_path: &str) -> Result<Self> {
        let db = RedbBackend::open(db_path)?;
        Ok(CustomerStore { db })
    }
    
    // Customer operations
    pub fn add_customer(&self, customer: &Customer) -> Result<()> {
        let customers: TypedTable<u64, Customer, JsonSerializer> = TypedTable::json(&self.db, "customers");
        customers.set(&customer.id, customer)
    }
    
    pub fn get_customer(&self, id: u64) -> Result<Option<Customer>> {
        let customers: TypedTable<u64, Customer, JsonSerializer> = TypedTable::json(&self.db, "customers");
        customers.get(&id)
    }
    
    pub fn update_customer(&self, customer: &Customer) -> Result<()> {
        let customers: TypedTable<u64, Customer, JsonSerializer> = TypedTable::json(&self.db, "customers");
        customers.set(&customer.id, customer)
    }
    
    pub fn remove_customer(&self, id: u64) -> Result<bool> {
        let customers: TypedTable<u64, Customer, JsonSerializer> = TypedTable::json(&self.db, "customers");
        customers.remove(&id)
    }
    
    // Order operations
    pub fn add_order(&self, order: &Order) -> Result<()> {
        let orders: TypedTable<u64, Order, JsonSerializer> = TypedTable::json(&self.db, "orders");
        orders.set(&order.id, order)
    }
    
    pub fn get_order(&self, id: u64) -> Result<Option<Order>> {
        let orders: TypedTable<u64, Order, JsonSerializer> = TypedTable::json(&self.db, "orders");
        orders.get(&id)
    }
    
    // Settings operations using string keys
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let settings: TypedTable<String, String, JsonSerializer> = TypedTable::json(&self.db, "settings");
        settings.set(&key.to_string(), &value.to_string())
    }
    
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let settings: TypedTable<String, String, JsonSerializer> = TypedTable::json(&self.db, "settings");
        settings.get(&key.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_customer_store_example() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("customer_store.db");
        
        let store = CustomerStore::new(db_path.to_str().unwrap())?;
        
        // Create a customer
        let customer = Customer {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            created_at: 1691280000, // Unix timestamp
            active: true,
        };
        
        // Add customer
        store.add_customer(&customer)?;
        
        // Retrieve customer
        let retrieved = store.get_customer(1)?;
        assert_eq!(retrieved, Some(customer.clone()));
        
        // Update customer
        let mut updated_customer = customer.clone();
        updated_customer.email = "alice.johnson@example.com".to_string();
        store.update_customer(&updated_customer)?;
        
        let retrieved_updated = store.get_customer(1)?;
        assert_eq!(retrieved_updated, Some(updated_customer));
        
        // Create an order
        let order = Order {
            id: 100,
            customer_id: 1,
            items: vec!["Widget A".to_string(), "Widget B".to_string()],
            total: 29.99,
            created_at: 1691280060,
        };
        
        store.add_order(&order)?;
        let retrieved_order = store.get_order(100)?;
        assert_eq!(retrieved_order, Some(order));
        
        // Settings
        store.set_setting("app.version", "1.0.0")?;
        store.set_setting("feature.new_ui", "enabled")?;
        
        let version = store.get_setting("app.version")?;
        assert_eq!(version, Some("1.0.0".to_string()));
        
        let ui_feature = store.get_setting("feature.new_ui")?;
        assert_eq!(ui_feature, Some("enabled".to_string()));
        
        // Test removal
        let removed = store.remove_customer(1)?;
        assert!(removed);
        
        let after_removal = store.get_customer(1)?;
        assert_eq!(after_removal, None);
        
        println!("✅ Complete customer store example test passed!");
        println!("📊 Demonstrated:");
        println!("  - Customer CRUD operations");
        println!("  - Order management");
        println!("  - Settings with string keys");
        println!("  - Multiple tables in same database");
        println!("  - Type-safe JSON serialization");
        
        Ok(())
    }

    #[test]
    fn test_performance_demo() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("perf_test.db");
        
        let store = CustomerStore::new(db_path.to_str().unwrap())?;
        
        let start = std::time::Instant::now();
        
        // Insert 1000 customers
        for i in 0..1000 {
            let customer = Customer {
                id: i,
                name: format!("Customer {}", i),
                email: format!("customer{}@example.com", i),
                created_at: 1691280000 + i,
                active: i % 2 == 0,
            };
            store.add_customer(&customer)?;
        }
        
        let insert_time = start.elapsed();
        
        let start = std::time::Instant::now();
        
        // Read back all customers
        for i in 0..1000 {
            let customer = store.get_customer(i)?;
            assert!(customer.is_some());
        }
        
        let read_time = start.elapsed();
        
        println!("✅ Performance demo completed!");
        println!("📈 Results:");
        println!("  - Inserted 1000 customers in {:?}", insert_time);
        println!("  - Read 1000 customers in {:?}", read_time);
        println!("  - Avg insert: {:?}/op", insert_time / 1000);
        println!("  - Avg read: {:?}/op", read_time / 1000);
        
        Ok(())
    }
}
