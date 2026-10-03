#![allow(dead_code, unused_imports, unused_variables)]

use std::hash::Hash;

use crate::collections::syncmap::SyncMap;

pub struct SyncSet<T: Eq + Hash + Clone> {
    m: SyncMap<T, ()>,
}

impl<T: Eq + Hash + Clone> Default for SyncSet<T> {
    fn default() -> Self { crate::fntrace::enter("default"); 
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> SyncSet<T> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        SyncSet {
            m: SyncMap::new(),
        }
    }

    pub fn has(&self, key: &T) -> bool { crate::fntrace::enter("has"); 
        self.m.load(key).is_some()
    }

    pub fn add(&self, key: T) { crate::fntrace::enter("add"); 
        self.add_if_absent(&key);
    }

    pub fn add_if_absent(&self, key: &T) -> bool { crate::fntrace::enter("add_if_absent"); 
        let (_, loaded) = self.m.load_or_store(key.clone(), ());
        !loaded
    }

    pub fn delete(&self, key: &T) { crate::fntrace::enter("delete"); 
        self.m.delete(key);
    }

    pub fn for_each<F: FnMut(&T) -> bool>(&self, mut f: F) { crate::fntrace::enter("for_each"); 
        self.m.for_each(|key, _| f(key));
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.m.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.m.is_empty()
    }
}

pub fn to_slice<T: Eq + Hash + Clone>(set: &SyncSet<T>) -> Vec<T> { crate::fntrace::enter("to_slice"); 
    let mut arr = Vec::with_capacity(set.len());
    set.for_each(|key| {
        arr.push(key.clone());
        true
    });
    arr
}
