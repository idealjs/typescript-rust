use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct CopyOnWriteMap<K: Eq + Hash + Clone, V: Clone> {
    inner: HashMap<K, V>,
    owned: bool,
}

impl<K: Eq + Hash + Clone, V: Clone> Default for CopyOnWriteMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CowScopeState<K: Eq + Hash + Clone, V: Clone> {
    inner: HashMap<K, V>,
    owned: bool,
}

impl<K: Eq + Hash + Clone, V: Clone> CopyOnWriteMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            owned: true,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn has(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.ensure_owned();
        self.inner.insert(key, value);
    }

    pub fn set(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn ensure_owned(&mut self) {
        if self.owned {
            return;
        }

        self.inner = self.inner.clone();
        self.owned = true;
    }

    pub fn with_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let state = self.enter_scope();
        let result = f(self);
        self.exit_scope(state);
        result
    }

    pub fn enter_scope(&mut self) -> CowScopeState<K, V> {
        let state = CowScopeState {
            inner: self.inner.clone(),
            owned: self.owned,
        };
        self.owned = false;
        state
    }

    pub fn exit_scope(&mut self, state: CowScopeState<K, V>) {
        self.inner = state.inner;
        self.owned = state.owned;
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner.iter()
    }
}

#[derive(Debug, Clone)]
pub struct CopyOnWriteSet<K: Eq + Hash + Clone> {
    map: CopyOnWriteMap<K, ()>,
}

impl<K: Eq + Hash + Clone> Default for CopyOnWriteSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone> CopyOnWriteSet<K> {
    pub fn new() -> Self {
        Self {
            map: CopyOnWriteMap::new(),
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn has(&self, key: &K) -> bool {
        self.contains(key)
    }

    pub fn insert(&mut self, key: K) {
        self.map.insert(key, ());
    }

    pub fn add(&mut self, key: K) {
        self.insert(key);
    }

    pub fn with_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let state = self.map.enter_scope();
        let result = f(self);
        self.map.exit_scope(state);
        result
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests;
