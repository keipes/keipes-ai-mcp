use std::{
    cell::RefCell,
    num::{NonZero, NonZeroUsize},
    sync::Arc,
};

// use dashmap::DashMap;
use lru::LruCache;
use moka::sync::{Cache, CacheBuilder};

use crate::session::{Session, SessionId, UuidSession};

pub trait SessionCache<K, V>
where
    K: SessionId,
    V: Session,
{
    fn get(&self, id: &K) -> Arc<V>;
    fn put(&self, id: K, session: Arc<V>);
}

pub struct LruSessionCache<K, V>
where
    K: SessionId,
    V: Session,
{
    cache: RefCell<LruCache<K, Arc<V>>>,
}

impl<K, V> LruSessionCache<K, V>
where
    K: SessionId,
    V: Session + 'static,
{
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            cache: RefCell::new(LruCache::new(capacity)),
        }
    }
}

impl<K, V> SessionCache<K, V> for LruSessionCache<K, V>
where
    K: SessionId,
    V: Session + 'static,
{
    fn get(&self, id: &K) -> Arc<V> {
        let data = self
            .cache
            .borrow_mut()
            .get_or_insert_mut_ref(id, || Arc::new(V::new()))
            .clone();
        data
    }

    fn put(&self, id: K, session: Arc<V>) {
        self.cache.borrow_mut().put(id, session);
    }
}

pub struct MokaCache<K, V>
where
    K: SessionId,
    V: Session,
{
    cache: Cache<K, Arc<V>>,
}

impl<K, V> MokaCache<K, V>
where
    K: SessionId,
    V: Session + 'static,
{
    fn new() -> Self {
        Self {
            cache: CacheBuilder::new(64).build(),
        }
    }
}

impl<K, V> SessionCache<K, V> for MokaCache<K, V>
where
    K: SessionId,
    V: Session + 'static,
{
    fn get(&self, id: &K) -> Arc<V> {
        let data = self.cache.get_with_by_ref(id, || Arc::new(V::new()));
        data
    }

    fn put(&self, id: K, session: Arc<V>) {
        self.cache.insert(id, session);
    }
}

// unit tests
// write tests below here
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moka_cache() {
        let session_id = "test".to_string();
        let session = Arc::new("123123".to_string());
        let cache = MokaCache::<String, String>::new();
        cache.put(session_id.clone(), Arc::clone(&session));
        let cached_session = cache.get(&session_id);
        assert_eq!(*cached_session, "123123");
        assert_eq!(*cached_session, *session)
    }
}
