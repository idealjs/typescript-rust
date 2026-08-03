//! MapBuilder for snapshot cloning.
//! Port of Go's `internal/project/dirty/mapbuilder.go`.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Builder for creating a new map from a base map with modifications.
/// Matches Go's `MapBuilder[K, VBase, VBuilder]`.
pub struct MapBuilder<K: Eq + Hash + Clone, VBase: Clone> {
    base: HashMap<K, VBase>,
    dirty: HashMap<K, VBase>,
    deleted: HashSet<K>,
}

impl<K: Eq + Hash + Clone, VBase: Clone> MapBuilder<K, VBase> {
    pub fn new(base: HashMap<K, VBase>) -> Self {
        MapBuilder {
            base,
            dirty: HashMap::new(),
            deleted: HashSet::new(),
        }
    }

    pub fn set(&mut self, key: K, value: VBase) {
        self.dirty.insert(key.clone(), value);
        self.deleted.remove(&key);
    }

    pub fn delete(&mut self, key: &K) {
        self.deleted.insert(key.clone());
        self.dirty.remove(key);
    }

    pub fn clear(&mut self) {
        self.dirty.clear();
        self.deleted = self.base.keys().cloned().collect();
    }

    pub fn has(&self, key: &K) -> bool {
        if self.deleted.contains(key) {
            return false;
        }
        if self.dirty.contains_key(key) {
            return true;
        }
        self.base.contains_key(key)
    }

    pub fn build(&self) -> HashMap<K, VBase> {
        if self.dirty.is_empty() && self.deleted.is_empty() {
            return self.base.clone();
        }
        let mut result = self.base.clone();
        for key in &self.deleted {
            result.remove(key);
        }
        for (key, value) in &self.dirty {
            result.insert(key.clone(), value.clone());
        }
        result
    }
}
