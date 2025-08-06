use redb::{Database, TableDefinition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::create("test_batch.redb")?;
    let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("test");
    
    // Test what methods are available on Table
    let txn = db.begin_write()?;
    {
        let mut table = txn.open_table(table_def)?;
        
        // Available methods:
        // table.insert(key, value)?;      // Single insert
        // table.remove(key)?;             // Single remove  
        // table.get(key)?;                // Single get
        // table.iter()?;                  // Iterator over all
        // table.range(..)?;               // Range iterator
        
        // Test batch inserts - check if there's any special batch API
        for i in 0..1000u32 {
            let key = i.to_be_bytes();
            let value = format!("value_{}", i).into_bytes();
            table.insert(key.as_slice(), value.as_slice())?;
        }
        
        // Note: No special batch insert method - just repeated insert() calls
        // But they're all in the same transaction, so it's efficient
    }
    txn.commit()?;
    
    println!("Batch insert completed - redb uses transaction-level batching");
    Ok(())
}
