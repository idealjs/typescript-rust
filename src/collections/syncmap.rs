//! Concurrent map, ported from `internal/collections/syncmap.go`.
//!
//! The Go implementation wraps `sync.Map`. In Rust we use `dashmap::DashMap`
//! which provides a fine-grained-locked concurrent map with a similar API.

use dashmap::DashMap;
use std::collections::HashMap;
use std::hash::Hash;

/// A concurrent map.
///
/// Mirrors `collections.SyncMap[K, V]` in Go.
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

    /// Get a cloned value for `key` (since we can't easily return a reference
    /// across the lock boundary).
    ///
    /// Mirrors `SyncMap.Load`.
    pub fn load(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|v| v.clone())
    }

    /// Store `value` for `key`.
    pub fn store(&self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    /// Load the existing value for `key`, or store `value` and return it.
    /// Returns `(value, loaded)` where `loaded` is true if the value already
    /// existed.
    ///
    /// Mirrors `SyncMap.LoadOrStore`.
    pub fn load_or_store(&self, key: K, value: V) -> (V, bool) {
        match self.inner.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(e) => (e.get().clone(), true),
            dashmap::mapref::entry::Entry::Vacant(e) => {
                e.insert(value.clone());
                (value, false)
            }
        }
    }

    /// Remove `key` from the map.
    pub fn delete(&self, key: &K) {
        self.inner.remove(key);
    }

    /// Remove all entries.
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Iterate over all entries, calling `f` for each. If `f` returns false,
    /// iteration stops.
    ///
    /// Mirrors `SyncMap.Range`.
    pub fn for_each<F: FnMut(&K, &V) -> bool>(&self, mut f: F) {
        for entry in self.inner.iter() {
            if !f(entry.key(), entry.value()) {
                break;
            }
        }
    }

    /// Approximate number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Collect all entries into a `HashMap`.
    ///
    /// Mirrors `SyncMap.ToMap`.
    pub fn to_hash_map(&self) -> HashMap<K, V> {
        self.inner.iter().map(|e| (e.key().clone(), e.value().clone())).collect()
    }

    /// Collect all keys into a `Vec`.
    pub fn keys(&self) -> Vec<K> {
        self.inner.iter().map(|e| e.key().clone()).collect()
    }

    /// Clone into a new `SyncMap`.
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
}
