use anyhow::Result;
use flatbuffers::FlatBufferBuilder;

use crate::generated::{Test, TestArgs, finish_test_buffer, root_as_test};
use crate::storage::{Storage, StorageConfig, Recreate};
use redb::TableDefinition;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatbuffers_basic_serialization() -> Result<()> {
        // Create a FlatBuffer
        let mut builder = FlatBufferBuilder::new();
        
        // Create the strings and byte vector
        let string_val = builder.create_string("Hello FlatBuffers!");
        let option_val = builder.create_vector(&[1u8, 2, 3, 4, 5]);
        
        // Create the Test object
        let test = Test::create(&mut builder, &TestArgs {
            int_val: 42,
            string_val: Some(string_val),
            option_val: Some(option_val),
        });
        
        // Finish the buffer
        finish_test_buffer(&mut builder, test);
        
        // Get the bytes
        let bytes = builder.finished_data();
        
        // Read back the data
        let test_read = root_as_test(bytes)?;
        
        // Verify the data
        assert_eq!(test_read.int_val(), 42);
        assert_eq!(test_read.string_val(), Some("Hello FlatBuffers!"));
        
        if let Some(option_vec) = test_read.option_val() {
            let vec_data: Vec<u8> = option_vec.iter().collect();
            assert_eq!(vec_data, vec![1, 2, 3, 4, 5]);
        } else {
            panic!("option_val should not be None");
        }
        
        println!("FlatBuffers serialization test passed!");
        
        Ok(())
    }

    #[test]
    fn test_flatbuffers_with_database() -> Result<()> {
        // Create a storage instance
        let storage = Storage::new(StorageConfig::new(
            "test_flatbuffers_db.redb".into(),
            Recreate::Always,
        ))?;

        let table_def = TableDefinition::<&[u8], &[u8]>::new("flatbuffers_test");

        // Create a FlatBuffer
        let mut builder = FlatBufferBuilder::new();
        let string_val = builder.create_string("Database FlatBuffers Test");
        let option_val = builder.create_vector(&[10u8, 20, 30]);
        
        let test = Test::create(&mut builder, &TestArgs {
            int_val: 123,
            string_val: Some(string_val),
            option_val: Some(option_val),
        });
        
        finish_test_buffer(&mut builder, test);
        let bytes = builder.finished_data();

        // Store in database
        storage.write(table_def, |table| {
            table.insert(&b"flatbuffer_test_key"[..], bytes)?;
            Ok(())
        }).map_err(|e| anyhow::anyhow!(e))?;

        // Read from database and deserialize
        storage.read(table_def, |table| {
            if let Some(access_guard) = table.get(&b"flatbuffer_test_key"[..])? {
                let stored_bytes = access_guard.value();
                let test_read = root_as_test(stored_bytes)
                    .map_err(|e| anyhow::anyhow!("FlatBuffer validation failed: {:?}", e))?;

                // Verify the data
                assert_eq!(test_read.int_val(), 123);
                assert_eq!(test_read.string_val(), Some("Database FlatBuffers Test"));

                if let Some(option_vec) = test_read.option_val() {
                    let vec_data: Vec<u8> = option_vec.iter().collect();
                    assert_eq!(vec_data, vec![10, 20, 30]);
                } else {
                    panic!("option_val should not be None");
                }

                println!("FlatBuffers database integration test passed!");
                Ok(())
            } else {
                Err(anyhow::anyhow!("Key not found in database"))
            }
        }).map_err(|e| anyhow::anyhow!(e))
    }
}
