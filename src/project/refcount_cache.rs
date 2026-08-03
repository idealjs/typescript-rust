//! Generic ref-counted cache (1:1 port of Go's `internal/project/refcountcache.go`).

#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

/// Options for [`RefCountCache`].
#[derive(Debug, Clone, Default)]
pub struct RefCountCacheOptions {
    /// Prevents entries from being removed from the cache. Used for testing.
    pub disable_deletion: bool,
}

struct RefCountCacheEntry<V: Clone> {
    value: V,
    ref_count: i32,
}

/// A generic cache that tracks reference counts per entry.
///
/// Go: `type RefCountCache[K comparable, V any, AcquireArgs any] struct`.
///
/// In Rust we parameterize on `K`, `V`, and the `parse` closure type.
pub struct RefCountCache<K: Eq + Hash + Clone, V: Clone> {
    pub options: RefCountCacheOptions,
    entries: Mutex<HashMap<K, RefCountCacheEntry<V>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> RefCountCache<K, V> {
    pub fn new(options: RefCountCacheOptions) -> Self {
        RefCountCache {
            options,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Retrieves or creates a cache entry for the given identity. If an entry
    /// exists, its refcount is incremented and the cached value is returned.
    /// Otherwise, `parse` is called to create the value.
    pub fn acquire<F>(&self, identity: K, parse: F) -> V
    where
        F: FnOnce(&K) -> V,
    {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(&identity) {
            entry.ref_count += 1;
            return entry.value.clone();
        }
        let value = parse(&identity);
        entries.insert(
            identity,
            RefCountCacheEntry {
                value: value.clone(),
                ref_count: 1,
            },
        );
        value
    }

    /// Returns true if the cache has an entry for `identity`.
    pub fn has(&self, identity: &K) -> bool {
        self.entries.lock().unwrap().contains_key(identity)
    }

    /// Increments the reference count for an existing entry. Panics if the
    /// entry does not exist.
    pub fn r#ref(&self, identity: &K) {
        let mut entries = self.entries.lock().unwrap();
        match entries.get_mut(identity) {
            None => panic!("cache entry not found"),
            Some(entry) => {
                if entry.ref_count <= 0 && !self.options.disable_deletion {
                    // Entry was deleted; re-add with refcount 1
                    entry.ref_count = 1;
                } else {
                    entry.ref_count += 1;
                }
            }
        }
    }

    /// Decrements the reference count. When the refcount reaches zero, the
    /// entry is removed (unless `DisableDeletion` is set).
    pub fn deref(&self, identity: &K) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(identity) {
            entry.ref_count -= 1;
            if entry.ref_count <= 0 && !self.options.disable_deletion {
                entries.remove(identity);
            }
        }
    }
}
