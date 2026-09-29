use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};

use tsox_core::collections::syncmap::SyncMap;

type DirtyShared<K, V> = SyncMap<K, Arc<SyncMapEntry<K, V>>>;

#[derive(Clone)]
struct EntryState<K: Eq + Hash + Clone, V: Clone> {
    key: K,
    original: V,
    value: V,
    dirty: bool,
    delete: bool,
    proxy_for: Option<Arc<SyncMapEntry<K, V>>>,
}

pub struct SyncMapEntry<K: Eq + Hash + Clone, V: Clone> {
    m: Arc<DirtySyncMap<K, V>>,
    inner: Mutex<EntryState<K, V>>,
}

pub struct DirtySyncMap<K: Eq + Hash + Clone, V: Clone> {
    pub base: HashMap<K, V>,
    pub dirty: DirtyShared<K, V>,
}

pub struct LockedEntry<'a, K: Eq + Hash + Clone, V: Clone> {
    guard: MutexGuard<'a, EntryState<K, V>>,
    entry: &'a SyncMapEntry<K, V>,
}

impl<K: Eq + Hash + Clone, V: Clone> SyncMapEntry<K, V> {
    pub fn value(&self) -> Option<V> {
        let state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.clone() {
            drop(state);
            return proxy.value();
        }
        self.value_locked(&state)
    }

    pub fn value_locked(&self, state: &EntryState<K, V>) -> Option<V> {
        if state.delete {
            None
        } else {
            Some(state.value.clone())
        }
    }

    pub fn dirty(&self) -> bool {
        let state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.clone() {
            drop(state);
            return proxy.dirty();
        }
        state.dirty
    }

    pub fn locked<R>(&self, f: impl FnOnce(&LockedEntry<K, V>) -> R) -> R {
        let state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.clone() {
            drop(state);
            return proxy.locked(f);
        }
        let locked = LockedEntry {
            guard: state,
            entry: self,
        };
        f(&locked)
    }

    pub fn change(&self, apply: impl FnOnce(&mut V)) {
        let mut state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.as_ref() {
            let proxy = proxy.clone();
            drop(state);
            proxy.change(apply);
            return;
        }
        self.change_locked(&mut state, apply);
    }

    pub fn change_locked(&self, state: &mut EntryState<K, V>, apply: impl FnOnce(&mut V)) {
        if state.dirty {
            apply(&mut state.value);
            return;
        }
        let (entry, loaded) = self
            .m
            .dirty
            .load_or_store(state.key.clone(), self_arc(self));
        if loaded {
            let mut entry_state = entry.inner.lock().unwrap();
            if !entry_state.dirty {
                entry_state.value = entry_state.value.clone();
                entry_state.dirty = true;
            }
            state.proxy_for = Some(entry.clone());
            state.value = entry_state.value.clone();
            state.dirty = true;
            state.delete = entry_state.delete;
            apply(&mut entry_state.value);
        } else {
            state.value = state.value.clone();
            state.dirty = true;
            apply(&mut state.value);
        }
    }

    pub fn change_if(&self, cond: impl FnOnce(&V) -> bool, apply: impl FnOnce(&mut V)) -> bool {
        let mut state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.as_ref() {
            let proxy = proxy.clone();
            drop(state);
            return proxy.change_if(cond, apply);
        }
        if cond(&state.value) {
            self.change_locked(&mut state, apply);
            true
        } else {
            false
        }
    }

    pub fn delete(&self) {
        let mut state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.as_ref() {
            let proxy = proxy.clone();
            drop(state);
            proxy.delete();
            return;
        }
        if state.dirty {
            state.delete = true;
            return;
        }
        let (entry, loaded) = self
            .m
            .dirty
            .load_or_store(state.key.clone(), self_arc(self));
        if loaded {
            state.delete = true;
            entry.inner.lock().unwrap().delete = true;
        } else {
            state.delete = true;
        }
    }

    pub fn delete_locked(&self, state: &mut EntryState<K, V>) {
        if state.dirty {
            state.delete = true;
            return;
        }
        let (entry, loaded) = self
            .m
            .dirty
            .load_or_store(state.key.clone(), self_arc(self));
        if loaded {
            let mut entry_state = entry.inner.lock().unwrap();
            state.proxy_for = Some(entry.clone());
            state.value = entry_state.value.clone();
            state.delete = true;
            state.dirty = entry_state.dirty;
            entry_state.delete = true;
        } else {
            state.delete = true;
        }
    }

    pub fn delete_if(&self, cond: impl FnOnce(&V) -> bool) {
        let mut state = self.inner.lock().unwrap();
        if let Some(proxy) = state.proxy_for.as_ref() {
            let proxy = proxy.clone();
            drop(state);
            proxy.delete_if(cond);
            return;
        }
        if cond(&state.value) {
            self.delete_locked(&mut state);
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> LockedEntry<'_, K, V> {
    pub fn value(&self) -> Option<V> {
        self.entry.value_locked(&self.guard)
    }

    pub fn original(&self) -> V {
        self.guard.original.clone()
    }

    pub fn dirty(&self) -> bool {
        self.guard.dirty
    }

    pub fn change(&mut self, apply: impl FnOnce(&mut V)) {
        apply(&mut self.guard.value);
    }

    pub fn change_if(&mut self, cond: impl FnOnce(&V) -> bool, apply: impl FnOnce(&mut V)) -> bool {
        if cond(&self.guard.value) {
            apply(&mut self.guard.value);
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self) {
        self.entry.delete_locked(&mut self.guard);
    }

    pub fn locked(&mut self, f: impl FnOnce(&mut LockedEntry<K, V>)) {
        f(self)
    }
}

impl<K: Eq + Hash + Clone, V: Clone> DirtySyncMap<K, V> {
    pub fn range(&self, mut f: impl FnMut(&Arc<SyncMapEntry<K, V>>) -> bool) {
        let mut seen = std::collections::HashSet::new();
        self.dirty.for_each(|key, entry| {
            seen.insert(key.clone());
            let not_deleted = !entry.inner.lock().unwrap().delete;
            not_deleted && f(entry)
        });
        for (key, value) in &self.base {
            if seen.contains(key) {
                continue;
            }
            let entry = Arc::new(SyncMapEntry {
                m: arc_of(self),
                inner: Mutex::new(EntryState {
                    key: key.clone(),
                    original: value.clone(),
                    value: value.clone(),
                    dirty: false,
                    delete: false,
                    proxy_for: None,
                }),
            });
            if !f(&entry) {
                break;
            }
        }
    }

    pub fn finalize(&self) -> (HashMap<K, V>, bool) {
        self.finalize_with(|_key, _value| {})
    }

    pub fn finalize_with(
        &self,
        mut on_discard: impl FnMut(&K, &V),
    ) -> (HashMap<K, V>, bool) {
        if self.dirty.is_empty() {
            return (self.base.clone(), false);
        }
        let mut result = self.base.clone();
        self.dirty.for_each(|key, entry| {
            let state = entry.inner.lock().unwrap();
            if state.delete {
                result.remove(key);
                on_discard(key, &state.original);
            } else {
                result.insert(key.clone(), state.value.clone());
            }
            true
        });
        (result, true)
    }
}

fn self_arc<K: Eq + Hash + Clone, V: Clone>(_e: &SyncMapEntry<K, V>) -> Arc<SyncMapEntry<K, V>> {
    unreachable!("requires Arc back-reference; see handoff")
}

fn arc_of<K: Eq + Hash + Clone, V: Clone>(_m: &DirtySyncMap<K, V>) -> Arc<DirtySyncMap<K, V>> {
    unreachable!("requires Arc back-reference; see handoff")
}
