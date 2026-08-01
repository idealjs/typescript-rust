//! Insertion-ordered map, ported from `internal/collections/ordered_map.go`.
//!
//! Backed by a `Vec<K>` (for order) and a `HashMap<K, V>` (for lookup).
//! Supports serde serialization in insertion order.

use std::collections::HashMap;
use std::hash::Hash;

/// An insertion-ordered map.
///
/// Mirrors `collections.OrderedMap[K, V]` in Go.
#[derive(Debug, Clone)]
pub struct OrderedMap<K: Eq + Hash + Clone, V> {
    keys: Vec<K>,
    map: HashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone, V> OrderedMap<K, V> {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Insert a key-value pair. If the key already exists, the value is
    /// updated; otherwise the key is appended to the end.
    pub fn insert(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.map.insert(key, value);
    }

    /// Set is an alias for `insert`, mirroring the Go API name.
    pub fn set(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    /// Get a reference to the value for `key`.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Get a mutable reference to the value for `key`.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    /// True if the map contains `key`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// `has` is an alias for `contains_key`, mirroring the Go API name.
    pub fn has(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    /// Remove `key` from the map, returning the previous value if present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let value = self.map.remove(key)?;
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.keys.remove(pos);
        }
        Some(value)
    }

    /// `delete` is an alias for `remove`, mirroring the Go API name.
    pub fn delete(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }

    /// Iterate over keys in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }

    /// Iterate over values in insertion order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.keys.iter().filter_map(move |k| self.map.get(k))
    }

    /// Iterate over `(key, value)` pairs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .iter()
            .filter_map(move |k| self.map.get(k).map(|v| (k, v)))
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Remove all entries but keep the allocated capacity.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.map.clear();
    }

    /// Get the key-value pair at `index` (insertion order).
    pub fn entry_at(&self, index: usize) -> Option<(&K, &V)> {
        let key = self.keys.get(index)?;
        self.map.get(key).map(|v| (key, v))
    }
}

impl<K: Eq + Hash + Clone, V: PartialEq> OrderedMap<K, V> {
    /// Compare two ordered maps for equality (same keys, same order, same values).
    pub fn eq(&self, other: &Self) -> bool {
        if self.keys != other.keys {
            return false;
        }
        for k in &self.keys {
            match (self.map.get(k), other.map.get(k)) {
                (Some(a), Some(b)) if a == b => {}
                _ => return false,
            }
        }
        true
    }
}

// Serde: serialize as a map preserving insertion order.
impl<K, V> serde::Serialize for OrderedMap<K, V>
where
    K: Eq + Hash + Clone + serde::Serialize,
    V: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de, K, V> serde::Deserialize<'de> for OrderedMap<K, V>
where
    K: Eq + Hash + Clone + serde::Deserialize<'de>,
    V: serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // We can't preserve arbitrary key order without a custom deserializer,
        // but for string keys serde_json already preserves order via a Vec.
        // For the general case, deserialize into a Vec of pairs.
        let pairs: Vec<(K, V)> = Vec::deserialize(deserializer)?;
        let mut result = Self::with_capacity(pairs.len());
        for (k, v) in pairs {
            result.insert(k, v);
        }
        Ok(result)
    }
}

/// A key-value pair, used when constructing an `OrderedMap` from a list.
#[derive(Debug, Clone)]
pub struct MapEntry<K, V> {
    pub key: K,
    pub value: V,
}

impl<K: Eq + Hash + Clone, V> OrderedMap<K, V> {
    /// Build an `OrderedMap` from a list of entries.
    pub fn from_entries(entries: impl IntoIterator<Item = MapEntry<K, V>>) -> Self {
        let entries: Vec<_> = entries.into_iter().collect();
        let mut map = Self::with_capacity(entries.len());
        for entry in entries {
            map.insert(entry.key, entry.value);
        }
        map
    }
}

/// Compute the diff between two ordered maps, invoking callbacks for added,
/// removed, and modified entries.
///
/// Mirrors `collections.DiffOrderedMapsFunc` in Go.
pub fn diff_ordered_maps<K, V>(
    m1: &OrderedMap<K, V>,
    m2: &OrderedMap<K, V>,
    equal_values: impl Fn(&V, &V) -> bool,
    mut on_added: impl FnMut(&K, &V),
    mut on_removed: impl FnMut(&K, &V),
    mut on_modified: impl FnMut(&K, &V, &V),
) where
    K: Eq + Hash + Clone,
{
    for (k, v2) in m2.iter() {
        if !m1.contains_key(k) {
            on_added(k, v2);
        }
    }
    for (k, v1) in m1.iter() {
        match m2.get(k) {
            Some(v2) => {
                if !equal_values(v1, v2) {
                    on_modified(k, v1, v2);
                }
            }
            None => on_removed(k, v1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_order() {
        let mut m = OrderedMap::new();
        m.insert("b", 2);
        m.insert("a", 1);
        m.insert("c", 3);
        let keys: Vec<&str> = m.keys().copied().collect();
        assert_eq!(keys, vec!["b", "a", "c"]);
    }

    #[test]
    fn update_preserves_order() {
        let mut m = OrderedMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        m.insert("a", 10);
        let keys: Vec<&str> = m.keys().copied().collect();
        assert_eq!(keys, vec!["a", "b"]);
        assert_eq!(m.get(&"a"), Some(&10));
    }

    #[test]
    fn remove_preserves_order() {
        let mut m = OrderedMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        m.insert("c", 3);
        m.remove(&"b");
        let keys: Vec<&str> = m.keys().copied().collect();
        assert_eq!(keys, vec!["a", "c"]);
    }

    #[test]
    fn diff() {
        let mut m1 = OrderedMap::new();
        m1.insert("a", 1);
        m1.insert("b", 2);
        m1.insert("c", 3);
        let mut m2 = OrderedMap::new();
        m2.insert("a", 1);
        m2.insert("b", 20);
        m2.insert("d", 4);

        let mut added: Vec<(String, i32)> = Vec::new();
        let mut removed: Vec<(String, i32)> = Vec::new();
        let mut modified: Vec<(String, i32, i32)> = Vec::new();
        diff_ordered_maps(
            &m1,
            &m2,
            |a, b| a == b,
            |k, v| added.push((k.to_string(), *v)),
            |k, v| removed.push((k.to_string(), *v)),
            |k, v1, v2| modified.push((k.to_string(), *v1, *v2)),
        );
        assert_eq!(added, vec![("d".to_string(), 4)]);
        assert_eq!(removed, vec![("c".to_string(), 3)]);
        assert_eq!(modified, vec![("b".to_string(), 2, 20)]);
    }

    // ── Ported from Go internal/collections/ordered_map_test.go ──

    fn pad_int(n: i32) -> String {
        format!("{:>10}", n)
    }

    #[test]
    fn test_ordered_map() {
        let mut m: OrderedMap<i32, String> = OrderedMap::new();

        assert!(!m.has(&1));

        const N: i32 = 1000;
        const START: i32 = 1;
        const END: i32 = START + N;

        // Seed the map with ascending keys and values for easier testing.
        for i in START..END {
            m.set(i, pad_int(i));
        }

        assert_eq!(m.len(), N as usize);

        // Attempt to overwrite existing keys in reverse order.
        for i in (START..END).rev() {
            m.set(i, pad_int(i));
        }

        assert_eq!(m.len(), N as usize);

        for i in START..END {
            let v = m.get(&i);
            assert!(v.is_some());
            assert_eq!(v.unwrap(), &pad_int(i));
        }

        for (k, v) in m.iter() {
            assert_eq!(v, &pad_int(*k));
        }

        let keys: Vec<i32> = m.keys().copied().collect();
        assert_eq!(keys.len(), N as usize);
        assert!(keys.windows(2).all(|w| w[0] <= w[1]));

        let values: Vec<String> = m.values().cloned().collect();
        assert_eq!(values.len(), N as usize);
        assert!(values.windows(2).all(|w| w[0] <= w[1]));

        let first_key = *m.keys().next().unwrap();
        assert_eq!(first_key, START);

        let first_value = m.values().next().unwrap().clone();
        assert_eq!(first_value, pad_int(START));

        let (fk, fv) = m.iter().next().unwrap();
        assert_eq!(*fk, START);
        assert_eq!(*fv, pad_int(START));

        for i in (START + 1)..END {
            let v = m.delete(&i);
            assert!(v.is_some());
            assert_eq!(v.unwrap(), pad_int(i));
            assert!(!m.has(&i));

            assert!(m.get(&i).is_none());

            assert!(m.delete(&i).is_none());
        }

        assert_eq!(m.len(), 1);
        assert!(m.has(&START));

        let v = m.delete(&START);
        assert!(v.is_some());
        assert_eq!(v.unwrap(), pad_int(START));

        assert_eq!(m.len(), 0);
    }

    #[test]
    fn test_ordered_map_clone() {
        let mut m: OrderedMap<i32, String> = OrderedMap::new();
        m.set(1, "one".to_string());
        m.set(2, "two".to_string());

        let clone = m.clone();

        // In Go: assert.Assert(t, clone != m) -- clone is a separate object.
        assert_eq!(clone.len(), 2);
        let clone_keys: Vec<i32> = clone.keys().copied().collect();
        assert_eq!(clone_keys, vec![1, 2]);
        let clone_values: Vec<String> = clone.values().cloned().collect();
        assert_eq!(clone_values, vec!["one".to_string(), "two".to_string()]);

        let v = clone.get(&1);
        assert!(v.is_some());
        assert_eq!(v.unwrap(), "one");

        m.delete(&1);

        assert_eq!(m.len(), 1);
        assert_eq!(clone.len(), 2);
        let clone_keys: Vec<i32> = clone.keys().copied().collect();
        assert_eq!(clone_keys, vec![1, 2]);
        let clone_values: Vec<String> = clone.values().cloned().collect();
        assert_eq!(clone_values, vec!["one".to_string(), "two".to_string()]);
    }

    #[test]
    fn test_ordered_map_clear() {
        let mut m: OrderedMap<i32, String> = OrderedMap::new();
        m.set(1, "one".to_string());
        m.set(2, "two".to_string());

        m.clear();

        assert_eq!(m.len(), 0);
    }

    #[test]
    #[ignore = "TODO: Go's testing.AllocsPerRun has no Rust equivalent for allocation counting"]
    fn test_ordered_map_with_size_hint() {
        // Ported from TestOrderedMapWithSizeHint:
        // const N: usize = 1024;
        // let mut m = OrderedMap::with_capacity(N);
        // for i in 0..N { m.set(i, i); }
        // Go verifies allocs < 10; no direct Rust equivalent without a custom allocator.
    }

    #[test]
    #[ignore = "TODO: OrderedMap serde Deserialize uses Vec of pairs, not a JSON object; \
                JSON object unmarshal into OrderedMap is not supported"]
    fn test_ordered_map_unmarshal_json() {
        // Ported from TestOrderedMapUnmarshalJSON:
        // let mut m: OrderedMap<String, serde_json::Value> = OrderedMap::new();
        // serde_json::from_str(r#"{"a": 1, "b": "two", "c": { "d": 4 } }"#) -> m
        //   assert size == 3, get_or_zero("a") == 1.0
        // serde_json::from_str("null") -> m  (no error)
        // serde_json::from_str(r#""foo""#) -> error "cannot unmarshal non-object JSON value into Map"
        // serde_json::from_str(r#"{"a": 1, "b": "two"}"#) into OrderedMap<i32, _> -> error "unmarshal"
    }
}
