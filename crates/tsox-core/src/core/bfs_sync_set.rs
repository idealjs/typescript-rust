use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct SyncSet<K: Eq + Hash + Clone> {
    inner: Mutex<HashSet<K>>,
}

impl<K: Eq + Hash + Clone> SyncSet<K> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            inner: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_if_absent(&self, key: &K) -> bool { crate::fntrace::enter("add_if_absent"); 
        self.inner.lock().unwrap().insert(key.clone())
    }

    pub fn has(&self, key: &K) -> bool { crate::fntrace::enter("has"); 
        self.inner.lock().unwrap().contains(key)
    }

    pub fn delete(&self, key: &K) { crate::fntrace::enter("delete"); 
        self.inner.lock().unwrap().remove(key);
    }
}
