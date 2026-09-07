use super::ordered_map::OrderedMap;
use std::hash::Hash;

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

    pub fn insert(&mut self, value: T) {
        self.map.insert(value, ());
    }

    pub fn add(&mut self, value: T) {
        self.insert(value);
    }

    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    pub fn has(&self, value: &T) -> bool {
        self.contains(value)
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.map.remove(value).is_some()
    }

    pub fn delete(&mut self, value: &T) -> bool {
        self.remove(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.keys()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

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
mod tests;
