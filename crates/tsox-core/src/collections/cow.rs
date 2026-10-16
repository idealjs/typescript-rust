pub(crate) use std::collections::HashMap;
pub(crate) use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct CopyOnWriteMap<K: Eq + Hash + Clone, V: Clone> {
    inner: HashMap<K, V>,
    owned: bool,
}

impl<K: Eq + Hash + Clone, V: Clone> Default for CopyOnWriteMap<K, V> {
    fn default() -> Self { crate::fntrace::enter("default"); 
        Self::new()
    }
}

pub struct CowScopeState<K: Eq + Hash + Clone, V: Clone> {
    inner: HashMap<K, V>,
    owned: bool,
}

impl<K: Eq + Hash + Clone, V: Clone> CopyOnWriteMap<K, V> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            inner: HashMap::new(),
            owned: true,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> { crate::fntrace::enter("get"); 
        self.inner.get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool { crate::fntrace::enter("contains_key"); 
        self.inner.contains_key(key)
    }

    pub fn has(&self, key: &K) -> bool { crate::fntrace::enter("has"); 
        self.contains_key(key)
    }

    pub fn insert(&mut self, key: K, value: V) { crate::fntrace::enter("insert"); 
        self.ensure_owned();
        self.inner.insert(key, value);
    }

    pub fn set(&mut self, key: K, value: V) { crate::fntrace::enter("set"); 
        self.insert(key, value);
    }

    fn ensure_owned(&mut self) { crate::fntrace::enter("ensure_owned"); 
        if self.owned {
            return;
        }

        self.inner = self.inner.clone();
        self.owned = true;
    }

    pub fn with_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R { crate::fntrace::enter("with_scope"); 
        let state = self.enter_scope();
        let result = f(self);
        self.exit_scope(state);
        result
    }

    pub fn enter_scope(&mut self) -> CowScopeState<K, V> { crate::fntrace::enter("enter_scope"); 
        let state = CowScopeState {
            inner: self.inner.clone(),
            owned: self.owned,
        };
        self.owned = false;
        state
    }

    pub fn exit_scope(&mut self, state: CowScopeState<K, V>) { crate::fntrace::enter("exit_scope"); 
        self.inner = state.inner;
        self.owned = state.owned;
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> { crate::fntrace::enter("iter"); 
        self.inner.iter()
    }
}

#[derive(Debug, Clone)]
pub struct CopyOnWriteSet<K: Eq + Hash + Clone> {
    map: CopyOnWriteMap<K, ()>,
}

impl<K: Eq + Hash + Clone> Default for CopyOnWriteSet<K> {
    fn default() -> Self { crate::fntrace::enter("default"); 
        Self::new()
    }
}

impl<K: Eq + Hash + Clone> CopyOnWriteSet<K> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            map: CopyOnWriteMap::new(),
        }
    }

    pub fn contains(&self, key: &K) -> bool { crate::fntrace::enter("contains"); 
        self.map.contains_key(key)
    }

    pub fn has(&self, key: &K) -> bool { crate::fntrace::enter("has"); 
        self.contains(key)
    }

    pub fn insert(&mut self, key: K) { crate::fntrace::enter("insert"); 
        self.map.insert(key, ());
    }

    pub fn add(&mut self, key: K) { crate::fntrace::enter("add"); 
        self.insert(key);
    }

    pub fn with_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R { crate::fntrace::enter("with_scope"); 
        let state = self.map.enter_scope();
        let result = f(self);
        self.map.exit_scope(state);
        result
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.map.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.map.is_empty()
    }
}
