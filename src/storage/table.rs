use anyhow::Result;
use redb::{Database, Key, Table, TableDefinition, Value, WriteTransaction};
use rkyv::{
    access, access_unchecked,
    api::high::{to_bytes_in, HighDeserializer, HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    rancor::Failure,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
    Archive, Serialize as RkyvSerialize,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, mem::take, sync::Arc};

pub fn iter_ciks() {
    // Replace with a valid file path for your database file
    let db = Arc::new(Database::create("mydb.redb").unwrap());
    match db.begin_write() {
        Ok(txn) => {
            // Use the table
            let mut table = txn.open_table(table_def).unwrap();
            let key: &[u8] = b"key";
            let value: &[u8] = b"value";
            table.insert(key, value);
        }
        Err(e) => {
            eprintln!("Error creating table: {}", e);
        }
    }
}

const table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::<&[u8], &[u8]>::new("my_table");

pub type WriteFn = fn(&mut Table<&[u8], &[u8]>) -> Result<()>;

// simpler access to table
// txn commits on Ok return from writefn
pub fn write(
    db: Database,
    table: TableDefinition<&[u8], &[u8]>,
    writer: WriteFn,
) -> Result<(), String> {
    match db.begin_write() {
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
                    Err(abort_err) => Err(format!("Failed to abort transaction: {}", abort_err)),
                },
            }
        }
        Err(e) => {
            eprintln!("Error creating table: {}", e);
            Err("Failed to create table".into())
        }
    }
}

pub fn write_some_stuff() -> Result<(), String> {
    let db = Database::create("mydb.redb").unwrap();
    let table_defs = TableDefinition::<&[u8], &[u8]>::new("my_table");
    write(db, table_defs, |table| {
        // Example write operation
        table.insert(b"example_key" as &[u8], b"example_value" as &[u8])?;
        Ok(())
    })
}

//test
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_write_some_stuff() {
        let result = write_some_stuff();
        assert!(result.is_ok());
    }
}
