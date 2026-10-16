use super::super::ordered_map::OrderedMap;

impl<K: Eq + std::hash::Hash + Clone, V: Default + Clone> OrderedMap<K, V> {
    pub fn get_or_zero(&self, key: &K) -> V { crate::fntrace::enter("get_or_zero"); 
        self.get(key).cloned().unwrap_or_default()
    }
}
