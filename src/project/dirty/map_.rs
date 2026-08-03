//! Dirty-tracking map for snapshot cloning.
//! Port of Go's `internal/project/dirty/map.go`.

use std::collections::HashMap;
use std::hash::Hash;

/// An entry in a dirty Map, tracking original and current values.
pub struct MapEntry<K: Clone + Eq + Hash, V: Clone> {
    pub key: K,
    pub original: V,
    pub value: V,
    pub dirty: bool,
    pub delete: bool,
}

impl<K: Clone + Eq + Hash, V: Clone> MapEntry<K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn original(&self) -> &V {
        &self.original
    }

    pub fn value(&self) -> V {
        if self.delete {
            // Return a clone of original (Go returns zero value; we return original)
            self.original.clone()
        } else {
            self.value.clone()
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}

/// Dirty-tracking map. Tracks changes from a base map for copy-on-write semantics.
/// Matches Go's `Map[K, V]`.
pub struct DirtyMap<K: Clone + Eq + Hash, V: Clone> {
    base: HashMap<K, V>,
    dirty: HashMap<K, MapEntry<K, V>>,
}

impl<K: Clone + Eq + Hash, V: Clone> DirtyMap<K, V> {
    pub fn new(base: HashMap<K, V>) -> Self {
        DirtyMap {
            base,
            dirty: HashMap::new(),
        }
    }

    /// Get an entry by key. Returns a clone of the entry.
    pub fn get(&self, key: &K) -> Option<MapEntry<K, V>> {
        if let Some(entry) = self.dirty.get(key) {
            if entry.delete {
                return None;
            }
            return Some(entry.clone_shallow());
        }
        let value = self.base.get(key)?;
        Some(MapEntry {
            key: key.clone(),
            original: value.clone(),
            value: value.clone(),
            dirty: false,
            delete: false,
        })
    }

    /// Add a new entry (mark as dirty without checking base).
    pub fn add(&mut self, key: K, value: V) {
        self.dirty.insert(
            key.clone(),
            MapEntry {
                key,
                original: value.clone(),
                value,
                dirty: true,
                delete: false,
            },
        );
    }

    /// Change a value, panicking if it doesn't exist.
    pub fn change<F>(&mut self, key: &K, apply: F)
    where
        F: FnOnce(&mut V),
    {
        if let Some(entry) = self.get(key) {
            let dirty_key = key.clone();
            if !entry.dirty {
                let mut value = entry.value.clone();
                apply(&mut value);
                self.dirty.insert(
                    dirty_key.clone(),
                    MapEntry {
                        key: dirty_key,
                        original: entry.original.clone(),
                        value,
                        dirty: true,
                        delete: false,
                    },
                );
            } else {
                let mut value = entry.value.clone();
                apply(&mut value);
                self.dirty.insert(
                    dirty_key.clone(),
                    MapEntry {
                        key: dirty_key,
                        original: entry.original.clone(),
                        value,
                        dirty: true,
                        delete: false,
                    },
                );
            }
        } else {
            panic!("tried to change a non-existent entry");
        }
    }

    /// Try to delete an entry. Returns false if not found.
    pub fn try_delete(&mut self, key: &K) -> bool {
        if self.get(key).is_none() {
            return false;
        }
        let entry = self.dirty.entry(key.clone()).or_insert_with(|| {
            let orig = self.base.get(key).cloned();
            let val = orig.clone();
            MapEntry {
                key: key.clone(),
                original: val.unwrap(),
                value: orig.unwrap(),
                dirty: false,
                delete: false,
            }
        });
        entry.delete = true;
        true
    }

    /// Iterate over all entries (dirty + base).
    pub fn range<F>(&self, mut f: F)
    where
        F: FnMut(&MapEntry<K, V>) -> bool,
    {
        let mut seen = std::collections::HashSet::new();
        for (key, entry) in &self.dirty {
            seen.insert(key.clone());
            if !entry.delete && !f(entry) {
                return;
            }
        }
        for (key, value) in &self.base {
            if seen.contains(key) {
                continue;
            }
            let entry = MapEntry {
                key: key.clone(),
                original: value.clone(),
                value: value.clone(),
                dirty: false,
                delete: false,
            };
            if !f(&entry) {
                break;
            }
        }
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.dirty.clear();
        self.base.clear();
    }

    /// Finalize: produce the merged map and whether anything changed.
    pub fn finalize(&self) -> (HashMap<K, V>, bool) {
        if self.dirty.is_empty() {
            return (self.base.clone(), false);
        }
        let mut result = self.base.clone();
        for (key, entry) in &self.dirty {
            if entry.delete {
                result.remove(key);
            } else {
                result.insert(key.clone(), entry.value.clone());
            }
        }
        (result, true)
    }
}

impl<K: Clone + Eq + Hash, V: Clone> MapEntry<K, V> {
    fn clone_shallow(&self) -> Self {
        MapEntry {
            key: self.key.clone(),
            original: self.original.clone(),
            value: self.value.clone(),
            dirty: self.dirty,
            delete: self.delete,
        }
    }
}
