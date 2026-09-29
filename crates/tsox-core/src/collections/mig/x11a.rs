#![allow(dead_code, unused_imports, unused_variables)]

use std::hash::Hash;

use crate::collections::syncmap::SyncMap;

pub struct SyncSet<T: Eq + Hash + Clone> {
    m: SyncMap<T, ()>,
}

impl<T: Eq + Hash + Clone> Default for SyncSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> SyncSet<T> {
    pub fn new() -> Self {
        SyncSet {
            m: SyncMap::new(),
        }
    }

    pub fn has(&self, key: &T) -> bool {
        self.m.load(key).is_some()
    }

    pub fn add(&self, key: T) {
        self.add_if_absent(&key);
    }

    pub fn add_if_absent(&self, key: &T) -> bool {
        let (_, loaded) = self.m.load_or_store(key.clone(), ());
        !loaded
    }

    pub fn delete(&self, key: &T) {
        self.m.delete(key);
    }

    pub fn for_each<F: FnMut(&T) -> bool>(&self, mut f: F) {
        self.m.for_each(|key, _| f(key));
    }

    pub fn len(&self) -> usize {
        self.m.len()
    }

    pub fn is_empty(&self) -> bool {
        self.m.is_empty()
    }
}

pub fn to_slice<T: Eq + Hash + Clone>(set: &SyncSet<T>) -> Vec<T> {
    let mut arr = Vec::with_capacity(set.len());
    set.for_each(|key| {
        arr.push(key.clone());
        true
    });
    arr
}
