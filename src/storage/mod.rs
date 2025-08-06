use redb::{Database, TableDefinition};
use std::borrow::Cow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

// Add missing From implementations for commit errors
impl From<redb::CommitError> for StorageError {
    fn from(err: redb::CommitError) -> Self {
        StorageError::Database(err.to_string())
    }
}

// Error types
#[derive(Debug)]
pub enum StorageError {
    Database(String),
    Serialization(String),
    InvalidConfiguration(String),
    Io(std::io::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Database(msg) => write!(f, "Database error: {}", msg),
            StorageError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            StorageError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<redb::DatabaseError> for StorageError {
    fn from(err: redb::DatabaseError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::TransactionError> for StorageError {
    fn from(err: redb::TransactionError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::TableError> for StorageError {
    fn from(err: redb::TableError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<redb::StorageError> for StorageError {
    fn from(err: redb::StorageError) -> Self {
        StorageError::Database(err.to_string())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

type Result<T> = std::result::Result<T, StorageError>;

// Backend implementation
pub struct RedbBackend {
    db: Database,
    buffer_pool: RefCell<Vec<(Vec<u8>, Vec<u8>)>>,
}

impl RedbBackend {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;
        Ok(RedbBackend {
            db,
            buffer_pool: RefCell::new(Vec::new()),
        })
    }

    pub fn get<F, R>(&self, table_name: &str, key: &[u8], accessor: F) -> Result<Option<R>>
    where
        F: FnOnce(&[u8]) -> R,
    {
        let txn = self.db.begin_read()?;
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        let table = txn.open_table(table_def)?;

        if let Some(value) = table.get(key)? {
            Ok(Some(accessor(value.value())))
        } else {
            Ok(None)
        }
    }

    pub fn put_batch<I>(&self, table_name: &str, items: I) -> Result<()>
    where
        I: IntoIterator<Item = (Cow<'static, [u8]>, Vec<u8>)>,
    {
        let mut buffer = self.buffer_pool.borrow_mut();
        buffer.clear();

        // Convert Cow to owned bytes for storage
        for (key, value) in items {
            buffer.push((key.into_owned(), value));
        }

        // Write to database
        let txn = self.db.begin_write()?;
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        {
            let mut table = txn.open_table(table_def)?;
            for (key, value) in &*buffer {
                table.insert(key.as_slice(), value.as_slice())?;
            }
        }
        txn.commit()?;

        Ok(())
    }

    pub fn delete(&self, table_name: &str, key: &[u8]) -> Result<bool> {
        let txn = self.db.begin_write()?;
        let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new(table_name);
        let existed = {
            let mut table = txn.open_table(table_def)?;
            let result = table.remove(key)?.is_some();
            result
        };
        txn.commit()?;
        Ok(existed)
    }
}
pub trait IntoKeyBytes<'a> {
    fn into_key_bytes(self, arena: &'a mut Vec<u8>) -> Cow<'a, [u8]>;
}

pub struct Storage {
    backend: Arc<RedbBackend>,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let backend = Arc::new(RedbBackend::new(path)?);
        Ok(Storage { backend })
    }

    pub fn table<K, V>(&self, name: &str) -> Table<K, V> {
        Table {
            storage: Arc::new(self.clone()),
            name: name.to_string(),
            _phantom: PhantomData,
        }
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Storage {
            backend: Arc::clone(&self.backend),
        }
    }
}

pub struct Table<K, V> {
    storage: Arc<Storage>,
    name: String,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Table<K, V> {
    pub fn put(&self, _key: K, _value: &V) -> Result<()> {
        // TODO: Implement actual put logic
        Ok(())
    }
}

// WriteBatch for transaction-scoped operations
pub struct WriteBatch {
    backend: Arc<RedbBackend>,
    table_name: String,
    key_arena: Vec<u8>,
    buffer: Vec<(Vec<u8>, Vec<u8>)>,
}

impl WriteBatch {
    pub fn new(backend: Arc<RedbBackend>, table_name: String) -> Self {
        WriteBatch {
            backend,
            table_name,
            key_arena: Vec::new(),
            buffer: Vec::new(),
        }
    }

    pub fn put_raw<K>(&mut self, key: K, value_bytes: Vec<u8>) -> Result<()>
    where
        K: for<'a> IntoKeyBytes<'a>,
    {
        let key_bytes = key.into_key_bytes(&mut self.key_arena).into_owned();
        self.buffer.push((key_bytes, value_bytes));
        Ok(())
    }

    pub fn commit(self) -> Result<()> {
        let items = self.buffer.into_iter().map(|(k, v)| (Cow::Owned(k), v));
        self.backend.put_batch(&self.table_name, items)
    }
}

// Basic implementations for common key types
impl<'a> IntoKeyBytes<'a> for &'a str {
    fn into_key_bytes(self, _arena: &'a mut Vec<u8>) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}

impl<'a> IntoKeyBytes<'a> for u64 {
    fn into_key_bytes(self, arena: &'a mut Vec<u8>) -> Cow<'a, [u8]> {
        let start = arena.len();
        arena.extend_from_slice(&self.to_le_bytes());
        Cow::Borrowed(&arena[start..start + 8])
    }
}
