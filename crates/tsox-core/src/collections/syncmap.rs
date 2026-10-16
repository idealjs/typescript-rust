pub(crate) use dashmap::DashMap;
pub(crate) use std::collections::HashMap;
pub(crate) use std::hash::Hash;

#[derive(Debug, Default)]
pub struct SyncMap<K: Eq + Hash + Clone, V: Clone> {
    inner: DashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V: Clone> SyncMap<K, V> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            inner: DashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self { crate::fntrace::enter("with_capacity"); 
        Self {
            inner: DashMap::with_capacity(capacity),
        }
    }

    pub fn load(&self, key: &K) -> Option<V> { crate::fntrace::enter("load"); 
        self.inner.get(key).map(|v| v.clone())
    }

    pub fn store(&self, key: K, value: V) { crate::fntrace::enter("store"); 
        self.inner.insert(key, value);
    }

    pub fn load_or_store(&self, key: K, value: V) -> (V, bool) { crate::fntrace::enter("load_or_store"); 
        match self.inner.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(e) => (e.get().clone(), true),
            dashmap::mapref::entry::Entry::Vacant(e) => {
                e.insert(value.clone());
                (value, false)
            }
        }
    }

    pub fn delete(&self, key: &K) { crate::fntrace::enter("delete"); 
        self.inner.remove(key);
    }

    pub fn clear(&self) { crate::fntrace::enter("clear"); 
        self.inner.clear();
    }

    pub fn for_each<F: FnMut(&K, &V) -> bool>(&self, mut f: F) { crate::fntrace::enter("for_each"); 
        for entry in self.inner.iter() {
            if !f(entry.key(), entry.value()) {
                break;
            }
        }
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.inner.is_empty()
    }

    pub fn to_hash_map(&self) -> HashMap<K, V> { crate::fntrace::enter("to_hash_map"); 
        self.inner
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    pub fn keys(&self) -> Vec<K> { crate::fntrace::enter("keys"); 
        self.inner.iter().map(|e| e.key().clone()).collect()
    }

    pub fn clone_map(&self) -> Self { crate::fntrace::enter("clone_map"); 
        let new = Self::new();
        for entry in self.inner.iter() {
            new.store(entry.key().clone(), entry.value().clone());
        }
        new
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Clone for SyncMap<K, V> {
    fn clone(&self) -> Self { crate::fntrace::enter("clone"); 
        self.clone_map()
    }
}
