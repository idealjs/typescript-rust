use dashmap::DashMap;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Default)]
pub struct SyncMap<K: Eq + Hash + Clone, V: Clone> {
    inner: DashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V: Clone> SyncMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DashMap::with_capacity(capacity),
        }
    }

    pub fn load(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|v| v.clone())
    }

    pub fn store(&self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    pub fn load_or_store(&self, key: K, value: V) -> (V, bool) {
        match self.inner.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(e) => (e.get().clone(), true),
            dashmap::mapref::entry::Entry::Vacant(e) => {
                e.insert(value.clone());
                (value, false)
            }
        }
    }

    pub fn delete(&self, key: &K) {
        self.inner.remove(key);
    }

    pub fn clear(&self) {
        self.inner.clear();
    }

    pub fn for_each<F: FnMut(&K, &V) -> bool>(&self, mut f: F) {
        for entry in self.inner.iter() {
            if !f(entry.key(), entry.value()) {
                break;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn to_hash_map(&self) -> HashMap<K, V> {
        self.inner
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    pub fn keys(&self) -> Vec<K> {
        self.inner.iter().map(|e| e.key().clone()).collect()
    }

    pub fn clone_map(&self) -> Self {
        let new = Self::new();
        for entry in self.inner.iter() {
            new.store(entry.key().clone(), entry.value().clone());
        }
        new
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Clone for SyncMap<K, V> {
    fn clone(&self) -> Self {
        self.clone_map()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let m = SyncMap::new();
        m.store("a", 1);
        m.store("b", 2);
        assert_eq!(m.load(&"a"), Some(1));
        assert_eq!(m.load(&"c"), None);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn load_or_store() {
        let m = SyncMap::new();
        let (v, loaded) = m.load_or_store("a", 1);
        assert_eq!(v, 1);
        assert!(!loaded);
        let (v, loaded) = m.load_or_store("a", 2);
        assert_eq!(v, 1);
        assert!(loaded);
    }

    #[test]
    fn test_sync_map_with_nil() {

        let m: SyncMap<String, Option<()>> = SyncMap::new();

        let got1 = m.load(&"foo".to_string());
        assert_eq!(got1, None);

        m.store("foo".to_string(), None);

        let got2 = m.load(&"foo".to_string());
        assert_eq!(got2, Some(None));

        let (too, loaded) = m.load_or_store("too".to_string(), None);
        assert!(!loaded);
        assert_eq!(too, None);

        m.for_each(|_, _| true);
    }
}
