use anyhow::Result;
use redb::{Database, ReadOnlyTable, Table, TableDefinition};
use std::{path::Path, sync::Arc};

mod serializers;
pub mod table;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recreate {
    Always,
    IfMissing,
    OnVersionBump,    // probably want this in conjunction with if missing
    OnValidationFail, // maybe need a builder
}
pub struct StorageConfig {
    pub db_path: String,
    pub recreate: Recreate,
}

impl StorageConfig {
    pub fn new(db_path: String, recreate: Recreate) -> Self {
        Self { db_path, recreate }
    }

    pub fn recreate(mut self) -> Self {
        self.recreate = Recreate::Always;
        self
    }
}

pub struct Storage {
    db: Arc<Database>,
}

const CIK_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::<&[u8], &[u8]>::new("cik");
// pub type WriteFn = fn(&mut Table<&[u8], &[u8]>) -> Result<()>;
// pub type ReadFn = fn(&ReadOnlyTable<&[u8], &[u8]>) -> Result<()>;

pub type WriteFn<'a> = dyn FnOnce(&mut Table<&[u8], &[u8]>) -> Result<()> + 'a;
pub type ReadFn<'a> = dyn FnOnce(&ReadOnlyTable<&[u8], &[u8]>) -> Result<()> + 'a;

impl Storage {
    pub fn new(config: StorageConfig) -> Result<Self> {
        if config.recreate == Recreate::Always && Path::new(&config.db_path).exists() {
            std::fs::remove_file(&config.db_path)?;
        }
        let db = Database::create(&config.db_path)?;
        Ok(Self { db: Arc::new(db) })
    }

    fn write<F>(&self, table: TableDefinition<&[u8], &[u8]>, writer: F) -> Result<(), String>
    where
        F: FnOnce(&mut Table<&[u8], &[u8]>) -> Result<()>,
    {
        match self.db.begin_write() {
            Ok(txn) => {
                let write_ok = {
                    let mut table: Table<'_, &[u8], &[u8]> = txn
                        .open_table(table)
                        .map_err(|e| format!("Failed to open table: {}", e))?;
                    writer(&mut table)
                };
                match write_ok {
                    Ok(()) => match txn.commit() {
                        Ok(()) => Ok(()),
                        Err(e) => Err(format!("Failed to write to table: {}", e)),
                    },
                    Err(e) => match txn.abort() {
                        Ok(_) => Err(format!("Failed to write to table: {}", e)),
                        Err(abort_err) => {
                            Err(format!("Failed to abort transaction: {}", abort_err))
                        }
                    },
                }
            }
            Err(e) => {
                eprintln!("Error creating table: {}", e);
                Err("Failed to create table".into())
            }
        }
    }

    pub fn read<F>(&self, table: TableDefinition<&[u8], &[u8]>, reader: F) -> Result<(), String>
    where
        F: FnOnce(&ReadOnlyTable<&[u8], &[u8]>) -> Result<()>,
    {
        match self.db.begin_read() {
            Ok(txn) => {
                let table = txn
                    .open_table(table)
                    .map_err(|e| format!("Failed to open table: {}", e))?;
                reader(&table).map_err(|e| format!("Failed to read from table: {}", e))
            }
            Err(e) => {
                eprintln!("Error creating table: {}", e);
                Err("Failed to create table".into())
            }
        }
    }

    pub fn compact(&mut self) -> Result<()> {
        match Arc::get_mut(&mut self.db) {
            Some(db) => {
                db.compact()?;
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Cannot compact: database has multiple references"
            )),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::storage::{Recreate, Storage, StorageConfig};
    use redb::TableDefinition;

    use rkyv::Archived;
    use rkyv::{Archive, Deserialize, Serialize};

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
    #[rkyv(
    compare(PartialEq),
    derive(Debug), // This is the key addition
)]
    struct Test {
        int: i32,
        string: String,
        option: Option<Vec<u8>>,
    }

    #[test]
    fn test_serialization() {
        let value = Test {
            int: 42,
            string: "hello world".to_string(),
            option: Some(vec![1, 2, 3, 4]),
        };

        // Test basic serialization first
        let bytes = rkyv::to_bytes(&value)
            .map_err(|e: rkyv::rancor::Error| format!("Serialization error: {}", e))
            .unwrap();
        let archived = rkyv::access::<Archived<Test>, rkyv::rancor::Error>(&bytes[..]).unwrap();
        assert_eq!(archived, &value);

        // Test database round-trip
        let db = Storage::new(StorageConfig::new(
            "test_serialization_db.redb".into(),
            Recreate::Always,
        ))
        .expect("Failed to create storage");
        let table_def = TableDefinition::<&[u8], &[u8]>::new("my_table");

        // Write
        let write_result = db.write(table_def, |table| {
            table.insert(b"example_key" as &[u8], bytes.as_slice())?;
            Ok(())
        });
        assert!(write_result.is_ok());

        // Read and verify
        let read_result = db.read(table_def, |table| {
            if let Some(access_guard) = table.get(b"example_key" as &[u8])? {
                let stored_bytes = access_guard.value();

                // Copy the bytes to ensure proper alignment
                let aligned_bytes = stored_bytes.to_vec();
                let stored_archived =
                    rkyv::access::<Archived<Test>, rkyv::rancor::Error>(&aligned_bytes)
                        .map_err(|e| anyhow::anyhow!("Access error: {}", e))?;

                // Compare the archived data with the original value
                assert_eq!(stored_archived, &value);

                Ok(())
            } else {
                Err(anyhow::anyhow!("Key not found"))
            }
        });

        assert!(read_result.is_ok());
    }
}
