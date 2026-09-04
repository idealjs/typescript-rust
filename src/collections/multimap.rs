use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Default)]
pub struct MultiMap<K: Eq + Hash + Clone, V: Clone + PartialEq> {
    inner: HashMap<K, Vec<V>>,
}

impl<K: Eq + Hash + Clone, V: Clone + PartialEq> MultiMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn has(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    pub fn get(&self, key: &K) -> &[V] {
        self.inner.get(key).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn add(&mut self, key: K, value: V) {
        self.inner.entry(key).or_default().push(value);
    }

    pub fn remove(&mut self, key: &K, value: &V) {
        if let Some(values) = self.inner.get_mut(key) {
            if let Some(pos) = values.iter().position(|v| v == value) {
                values.remove(pos);
                if values.is_empty() {
                    self.inner.remove(key);
                }
            }
        }
    }

    pub fn remove_all(&mut self, key: &K) {
        self.inner.remove(key);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.inner.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Vec<V>> {
        self.inner.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &Vec<V>)> {
        self.inner.iter()
    }
}

pub fn group_by<K, V, F>(items: &[V], group_id: F) -> MultiMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone + PartialEq,
    F: Fn(&V) -> K,
{
    let mut m = MultiMap::new();
    for item in items {
        m.add(group_id(item), item.clone());
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_get() {
        let mut m = MultiMap::new();
        m.add("a", 1);
        m.add("a", 2);
        m.add("b", 3);
        assert_eq!(m.get(&"a"), &[1, 2]);
        assert_eq!(m.get(&"b"), &[3]);
        assert_eq!(m.get(&"c"), &[] as &[i32]);
    }

    #[test]
    fn remove() {
        let mut m = MultiMap::new();
        m.add("a", 1);
        m.add("a", 2);
        m.remove(&"a", &1);
        assert_eq!(m.get(&"a"), &[2]);
        m.remove(&"a", &2);
        assert!(!m.has(&"a"));
    }

    #[test]
    fn group_by_works() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let m = group_by(&items, |x| x % 2);
        assert_eq!(m.get(&0), &[2, 4, 6]);
        assert_eq!(m.get(&1), &[1, 3, 5]);
    }
}
