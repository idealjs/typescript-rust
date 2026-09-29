use crate::collections::ordered_map::OrderedMap;

impl<K: Eq + std::hash::Hash + Clone, V: Clone> OrderedMap<K, V> {
    pub fn entries(&self) -> impl Iterator<Item = (K, V)> + '_ {
        let mut i = 0;
        std::iter::from_fn(move || {
            let (key, value) = self.entry_at(i)?;
            let entry = (key.clone(), value.clone());
            i += 1;
            Some(entry)
        })
    }
}
