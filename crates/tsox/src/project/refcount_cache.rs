#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct RefCountCacheOptions {
    pub disable_deletion: bool,
}

struct RefCountCacheEntry<V: Clone> {
    value: V,
    ref_count: i32,
}

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

    pub fn has(&self, identity: &K) -> bool {
        self.entries.lock().unwrap().contains_key(identity)
    }

    pub fn r#ref(&self, identity: &K) {
        let mut entries = self.entries.lock().unwrap();
        match entries.get_mut(identity) {
            None => panic!("cache entry not found"),
            Some(entry) => {
                if entry.ref_count <= 0 && !self.options.disable_deletion {
                    entry.ref_count = 1;
                } else {
                    entry.ref_count += 1;
                }
            }
        }
    }

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
