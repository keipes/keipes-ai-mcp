# Storage Implementation v2 - Transaction-Scoped Zero-Copy Keys

```
trait IntoKeyBytes<'a>
    into_key_bytes(self, arena) -> Cow<'a, [u8]>

impl IntoKeyBytes<'a> for &'a str
    into_key_bytes(self, _arena) -> Cow::Borrowed(self.as_bytes())

impl IntoKeyBytes<'a> for &'a [u8]
    into_key_bytes(self, _arena) -> Cow::Borrowed(self)

impl IntoKeyBytes<'a> for u64
    into_key_bytes(self, arena) ->
        start = arena.len()
        arena.extend_from_slice(&self.to_le_bytes())
        Cow::Borrowed(&arena[start..start+8])

impl IntoKeyBytes<'a> for String
    into_key_bytes(self, _arena) -> Cow::Owned(self.into_bytes())

mod storage
    Storage
        backend: Arc<RedbBackend>
        
        new(path) -> Result<Self>
        put<K, V>(table, key, value) -> Result<()>
        get<K, F, R>(table, key, accessor) -> Result<Option<R>>
        delete<K>(table, key) -> Result<bool>
        write_batch<F>(operation) -> Result<()>
        table<K, V>(name) -> Table<K, V>

    WriteBatch<'txn>
        backend: &'txn RedbBackend
        key_arena: Vec<u8>
        buffer: Vec<(Cow<'txn, [u8]>, Vec<u8>)>
        table_name: &'txn str
        
        put<K, V>(key, value) -> Result<()>
            key_bytes = key.into_key_bytes(&mut self.key_arena)
            value_bytes = rkyv::to_bytes(value)?.into_vec()
            self.buffer.push((key_bytes, value_bytes))
        
        put_raw<K>(key, value_bytes) -> Result<()>
            key_bytes = key.into_key_bytes(&mut self.key_arena)
            self.buffer.push((key_bytes, value_bytes))

    Table<K, V>
        storage: Arc<Storage>
        name: String
        _phantom: PhantomData<(K, V)>
        
        put(key, value) -> Result<()>
        put_batch<I>(items) -> Result<()>
        get<F, R>(key, accessor) -> Result<Option<R>>
        scan<F, R>(scanner) -> Result<Vec<R>>

mod backend
    RedbBackend
        db: Database
        buffer_pool: RefCell<Vec<(Vec<u8>, Vec<u8>)>>
        
        put_batch<I>(table, items) -> Result<()>
            buffer = self.buffer_pool.borrow_mut()
            buffer.clear()
            for (key, value) in items
                buffer.push((key.into_owned(), value))
            
            txn = db.begin_write()
            tbl = txn.open_table(table)
            for (k, v) in &*buffer
                tbl.insert(k.as_slice(), v.as_slice())
            txn.commit()
        
        get<F, R>(table, key, accessor) -> Result<Option<R>>
        delete(table, key) -> Result<bool>
        scan<F>(table, callback) -> Result<()>

usage()
    storage = Storage::new("db.redb")
    
    storage.put("users", "alice", &user)
    storage.put("users", 42u64, &user)
    
    storage.write_batch(|mut batch| {
        batch.put("alice", &user1)
        batch.put(123u64, &user2)
        batch.put(&bytes, &user3)
    })
    
    users = storage.table::<&str, User>("users")
    users.put("charlie", &user)
    
    user = users.get("alice", |archived| archived.name.as_str())
    
    items = users.scan(|key, data| {
        if data.active.to_native()
            Some(key.clone())
        else
            None
    })
```
