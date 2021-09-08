use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tokio::time::Duration;
use tokio::sync::Mutex;
use std::borrow::Borrow;

pub struct AutoRemoveHashMap<K, V> {
    inner: Mutex<HashMap<K, V>>,
    expire: Duration,
}

impl <K, V> AutoRemoveHashMap<K, V> {
    #[inline]
    pub fn new(expire: Duration) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::new()),
            expire,
        })
    }

    #[inline]
    pub fn with_capacity(expire: Duration, capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::with_capacity(capacity)),
            expire,
        })
    }
}

impl<K, V> AutoRemoveHashMap<K, V>
    where
        K: Clone + Eq + Hash + Send + 'static,
        V: Clone + Send + 'static,
{
    #[inline]
    pub async fn get<Q: ?Sized>(&self, k: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.inner.lock().await.get(k).map(Clone::clone)
    }

    #[inline]
    pub async fn insert(self: &Arc<Self>, k: K, v: V) -> Option<V> {
        let ret = self.inner.lock().await.insert(k.clone(), v);
        let this = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(this.expire).await;
            this.inner.lock().await.remove(&k);
        });
        ret
    }

    #[inline]
    pub async fn remove<Q: ?Sized>(&self, k: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.inner.lock().await.remove(k)
    }

    #[inline]
    pub async fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
        where
            K: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.inner.lock().await.contains_key(k)
    }

}