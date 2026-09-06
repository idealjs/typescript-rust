#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Mutex;

struct OwnerCacheEntry<V: Clone> {
    value: V,
    owners: HashSet<u64>,
}

pub struct OwnerCache<K: Eq + Hash + Clone, V: Clone> {
    entries: Mutex<HashMap<K, OwnerCacheEntry<V>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> OwnerCache<K, V> {
    pub fn new() -> Self {
        OwnerCache {
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_and_acquire<F>(&self, identity: K, owner: u64, parse: F) -> V
    where
        F: FnOnce(&K) -> V,
    {
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.entry(identity.clone()).or_insert_with(|| {
            let value = parse(&identity);
            OwnerCacheEntry {
                value,
                owners: HashSet::new(),
            }
        });
        entry.owners.insert(owner);
        entry.value.clone()
    }

    pub fn acquire(&self, identity: K, owner: u64, value: V) {
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.entry(identity).or_insert_with(|| OwnerCacheEntry {
            value: value.clone(),
            owners: HashSet::new(),
        });
        entry.owners.insert(owner);
    }

    pub fn add_owner(&self, identity: &K, owner: u64) {
        let mut entries = self.entries.lock().unwrap();
        match entries.get_mut(identity) {
            None => panic!("OwnerCache.add_owner: entry not found"),
            Some(entry) => {
                if entry.owners.is_empty() {
                    panic!("OwnerCache.add_owner: entry has no owners");
                }
                entry.owners.insert(owner);
            }
        }
    }

    pub fn has(&self, identity: &K) -> bool {
        self.entries.lock().unwrap().contains_key(identity)
    }

    pub fn release(&self, identity: &K, owner: u64) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(identity) {
            entry.owners.remove(&owner);
            if entry.owners.is_empty() {
                entries.remove(identity);
            }
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for OwnerCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
