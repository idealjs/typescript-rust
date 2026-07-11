//! Copy-on-write map and set, ported from `internal/collections/cow.go`.
//!
//! These are used in the binder/checker to support nested scopes that share
//! the parent's backing storage for reads but clone on first write.
//!
//! The Go implementation returns a `func()` from `EnterScope` that restores
//! the previous state when called. In Rust we provide two APIs:
//!   - `with_scope(&mut self, f)`: a callback-based RAII scope (preferred)
//!   - `enter_scope` / `exit_scope`: explicit save/restore for cases where
//!     the scope boundary doesn't map cleanly to a closure

use std::collections::HashMap;
use std::hash::Hash;

/// A copy-on-write map.
///
/// Mirrors `collections.CopyOnWriteMap[K, V]` in Go. The map starts by
/// borrowing its parent's storage; the first mutation clones the storage
/// so the parent is not affected.
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

/// Saved state of a `CopyOnWriteMap`, used to restore a scope on exit.
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

    /// `has` is an alias for `contains_key`, mirroring the Go API name.
    pub fn has(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    /// Insert `key`/`value`, cloning the backing storage first if it is
    /// shared with a parent scope.
    pub fn insert(&mut self, key: K, value: V) {
        self.ensure_owned();
        self.inner.insert(key, value);
    }

    /// `set` is an alias for `insert`, mirroring the Go API name.
    pub fn set(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    fn ensure_owned(&mut self) {
        if self.owned {
            return;
        }
        // Clone the shared storage so we own our own copy.
        self.inner = self.inner.clone();
        self.owned = true;
    }

    /// Run `f` in a new scope. While `f` runs, the map shares its current
    /// backing storage with the parent scope: reads see the inherited
    /// entries, and the first mutation transparently clones the storage so
    /// the parent's view is not modified. When `f` returns, the map is
    /// restored to its pre-scope state.
    ///
    /// This is the Rust-idiomatic replacement for Go's
    /// `defer cleanup := m.EnterScope()`.
    pub fn with_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let state = self.enter_scope();
        let result = f(self);
        self.exit_scope(state);
        result
    }

    /// Explicitly enter a new scope, returning the saved state. The caller
    /// must later call `exit_scope` with the returned state to restore the
    /// map. Prefer `with_scope` where possible.
    pub fn enter_scope(&mut self) -> CowScopeState<K, V> {
        let state = CowScopeState {
            inner: self.inner.clone(),
            owned: self.owned,
        };
        self.owned = false; // share with parent
        state
    }

    /// Restore the map to the state captured by `enter_scope`.
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

/// A copy-on-write set.
///
/// Mirrors `collections.CopyOnWriteSet[K]` in Go.
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

    /// `has` is an alias for `contains`, mirroring the Go API name.
    pub fn has(&self, key: &K) -> bool {
        self.contains(key)
    }

    pub fn insert(&mut self, key: K) {
        self.map.insert(key, ());
    }

    /// `add` is an alias for `insert`, mirroring the Go API name.
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
mod tests {
    use super::*;

    #[test]
    fn scope_restores_state() {
        let mut m = CopyOnWriteMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        m.with_scope(|m| {
            m.insert("c", 3);
            m.insert("a", 10);
            assert_eq!(m.get(&"a"), Some(&10));
            assert_eq!(m.len(), 3);
        });
        // After the scope exits, the map should be back to its pre-scope state.
        assert_eq!(m.get(&"a"), Some(&1));
        assert!(!m.contains_key(&"c"));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn cow_set_scope() {
        let mut s = CopyOnWriteSet::new();
        s.insert("a");
        s.insert("b");
        s.with_scope(|s| {
            s.insert("c");
            assert!(s.contains(&"c"));
            assert_eq!(s.len(), 3);
        });
        assert!(!s.contains(&"c"));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn explicit_enter_exit() {
        let mut m = CopyOnWriteMap::new();
        m.insert("a", 1);
        let state = m.enter_scope();
        m.insert("b", 2);
        assert_eq!(m.len(), 2);
        m.exit_scope(state);
        assert_eq!(m.len(), 1);
        assert!(!m.contains_key(&"b"));
    }
}
