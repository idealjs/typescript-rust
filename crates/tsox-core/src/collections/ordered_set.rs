pub(crate) use super::ordered_map::OrderedMap;
pub(crate) use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct OrderedSet<T: Eq + Hash + Clone> {
    map: OrderedMap<T, ()>,
}

impl<T: Eq + Hash + Clone> Default for OrderedSet<T> {
    fn default() -> Self { crate::fntrace::enter("default"); 
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> OrderedSet<T> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            map: OrderedMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self { crate::fntrace::enter("with_capacity"); 
        Self {
            map: OrderedMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, value: T) { crate::fntrace::enter("insert"); 
        self.map.insert(value, ());
    }

    pub fn add(&mut self, value: T) { crate::fntrace::enter("add"); 
        self.insert(value);
    }

    pub fn contains(&self, value: &T) -> bool { crate::fntrace::enter("contains"); 
        self.map.contains_key(value)
    }

    pub fn has(&self, value: &T) -> bool { crate::fntrace::enter("has"); 
        self.contains(value)
    }

    pub fn remove(&mut self, value: &T) -> bool { crate::fntrace::enter("remove"); 
        self.map.remove(value).is_some()
    }

    pub fn delete(&mut self, value: &T) -> bool { crate::fntrace::enter("delete"); 
        self.remove(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> { crate::fntrace::enter("iter"); 
        self.map.keys()
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.map.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.map.is_empty()
    }

    pub fn clear(&mut self) { crate::fntrace::enter("clear"); 
        self.map.clear();
    }

    pub fn from_iter(items: impl IntoIterator<Item = T>) -> Self { crate::fntrace::enter("from_iter"); 
        let items: Vec<_> = items.into_iter().collect();
        let mut set = Self::with_capacity(items.len());
        for item in items {
            set.insert(item);
        }
        set
    }
}
