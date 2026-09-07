use std::collections::HashMap;
use std::hash::Hash;

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

    pub fn insert(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.map.insert(key, value);
    }

    pub fn set(&mut self, key: K, value: V) {
        self.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn has(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let value = self.map.remove(key)?;
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.keys.remove(pos);
        }
        Some(value)
    }

    pub fn delete(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.keys.iter().filter_map(move |k| self.map.get(k))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .iter()
            .filter_map(move |k| self.map.get(k).map(|v| (k, v)))
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.map.clear();
    }

    pub fn entry_at(&self, index: usize) -> Option<(&K, &V)> {
        let key = self.keys.get(index)?;
        self.map.get(key).map(|v| (key, v))
    }
}

impl<K: Eq + Hash + Clone, V: PartialEq> OrderedMap<K, V> {
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
        struct OrderedMapVisitor<K, V>(std::marker::PhantomData<(K, V)>);

        impl<'de, K, V> serde::de::Visitor<'de> for OrderedMapVisitor<K, V>
        where
            K: Eq + Hash + Clone + serde::Deserialize<'de>,
            V: serde::Deserialize<'de>,
        {
            type Value = OrderedMap<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut map = OrderedMap::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((key, value)) = access.next_entry()? {
                    map.insert(key, value);
                }
                Ok(map)
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(OrderedMap::new())
            }
        }

        deserializer.deserialize_any(OrderedMapVisitor(std::marker::PhantomData))
    }
}

#[derive(Debug, Clone)]
pub struct MapEntry<K, V> {
    pub key: K,
    pub value: V,
}

impl<K: Eq + Hash + Clone, V> OrderedMap<K, V> {
    pub fn from_entries(entries: impl IntoIterator<Item = MapEntry<K, V>>) -> Self {
        let entries: Vec<_> = entries.into_iter().collect();
        let mut map = Self::with_capacity(entries.len());
        for entry in entries {
            map.insert(entry.key, entry.value);
        }
        map
    }
}

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
mod tests;
