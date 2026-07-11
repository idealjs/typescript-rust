//! Insertion-ordered set, ported from `internal/collections/ordered_set.go`.

use super::ordered_map::OrderedMap;
use std::hash::Hash;

/// An insertion-ordered set.
///
/// Mirrors `collections.OrderedSet[T]` in Go. Backed by an `OrderedMap`
/// with `()` values.
#[derive(Debug, Clone)]
pub struct OrderedSet<T: Eq + Hash + Clone> {
    map: OrderedMap<T, ()>,
}

impl<T: Eq + Hash + Clone> Default for OrderedSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> OrderedSet<T> {
    pub fn new() -> Self {
        Self {
            map: OrderedMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: OrderedMap::with_capacity(capacity),
        }
    }

    /// Add `value` to the set.
    pub fn insert(&mut self, value: T) {
        self.map.insert(value, ());
    }

    /// `add` is an alias for `insert`, mirroring the Go API name.
    pub fn add(&mut self, value: T) {
        self.insert(value);
    }

    /// True if the set contains `value`.
    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    /// `has` is an alias for `contains`, mirroring the Go API name.
    pub fn has(&self, value: &T) -> bool {
        self.contains(value)
    }

    /// Remove `value` from the set. Returns true if it was present.
    pub fn remove(&mut self, value: &T) -> bool {
        self.map.remove(value).is_some()
    }

    /// `delete` is an alias for `remove`, mirroring the Go API name.
    pub fn delete(&mut self, value: &T) -> bool {
        self.remove(value)
    }

    /// Iterate over values in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.keys()
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Remove all elements.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Build an `OrderedSet` from an iterator of values.
    pub fn from_iter(items: impl IntoIterator<Item = T>) -> Self {
        let items: Vec<_> = items.into_iter().collect();
        let mut set = Self::with_capacity(items.len());
        for item in items {
            set.insert(item);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_order() {
        let mut s = OrderedSet::new();
        s.insert("b");
        s.insert("a");
        s.insert("b"); // duplicate, no effect
        s.insert("c");
        let values: Vec<&str> = s.iter().copied().collect();
        assert_eq!(values, vec!["b", "a", "c"]);
    }

    #[test]
    fn remove() {
        let mut s = OrderedSet::new();
        s.insert("a");
        s.insert("b");
        s.insert("c");
        assert!(s.remove(&"b"));
        assert!(!s.remove(&"b"));
        let values: Vec<&str> = s.iter().copied().collect();
        assert_eq!(values, vec!["a", "c"]);
    }
}
