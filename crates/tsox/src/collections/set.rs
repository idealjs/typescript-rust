use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, Default)]
pub struct Set<T: Eq + Hash + Clone> {
    inner: HashSet<T>,
}

impl<T: Eq + Hash + Clone> Set<T> {
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashSet::with_capacity(capacity),
        }
    }

    pub fn contains(&self, key: &T) -> bool {
        self.inner.contains(key)
    }

    pub fn has(&self, key: &T) -> bool {
        self.contains(key)
    }

    pub fn insert(&mut self, key: T) -> bool {
        self.inner.insert(key)
    }

    pub fn add(&mut self, key: T) {
        self.inner.insert(key);
    }

    pub fn add_if_absent(&mut self, key: T) -> bool {
        self.inner.insert(key)
    }

    pub fn remove(&mut self, key: &T) -> bool {
        self.inner.remove(key)
    }

    pub fn delete(&mut self, key: &T) {
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

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter()
    }

    pub fn union(&mut self, other: &Set<T>) {
        for item in other.inner.iter() {
            self.inner.insert(item.clone());
        }
    }

    pub fn unioned_with(&self, other: &Set<T>) -> Set<T> {
        let mut result = self.clone();
        result.union(other);
        result
    }

    pub fn equals(&self, other: &Set<T>) -> bool {
        self.inner == other.inner
    }

    pub fn is_subset_of(&self, other: &Set<T>) -> bool {
        self.inner.is_subset(&other.inner)
    }

    pub fn intersects(&self, other: &Set<T>) -> bool {
        self.inner.iter().any(|x| other.inner.contains(x))
    }

    pub fn from_items(items: impl IntoIterator<Item = T>) -> Self {
        let mut set = Self::new();
        for item in items {
            set.insert(item);
        }
        set
    }
}

#[cfg(test)]
mod tests;
