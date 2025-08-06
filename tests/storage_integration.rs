use keipes_ai_mcp::storage::{IntoKeyBytes, Storage, Table};
use std::borrow::Cow;

#[test]
fn test_storage_imports() {
    // Test that we can create a Storage instance
    let storage = Storage::new("test.db").unwrap();

    // Test that we can create a Table
    let _table: Table<&str, String> = storage.table("test_table");

    println!("✓ Storage imports working");
}

#[test]
fn test_key_conversion() {
    let mut arena = Vec::new();

    // Test string key conversion (zero-copy)
    let str_key = "test_key";
    let str_bytes: Cow<[u8]> = str_key.into_key_bytes(&mut arena);
    assert_eq!(str_bytes.as_ref(), b"test_key");

    // Test u64 key conversion (arena allocation)
    let num_key = 42u64;
    let num_bytes: Cow<[u8]> = num_key.into_key_bytes(&mut arena);
    assert_eq!(num_bytes.as_ref(), &42u64.to_le_bytes());

    println!("✓ Key conversion working");
}
