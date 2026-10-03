pub(crate) use std::collections::HashSet;
pub(crate) use std::hash::Hash;

#[derive(Debug, Clone, Default)]
pub struct Set<T: Eq + Hash + Clone> {
    inner: HashSet<T>,
}

impl<T: Eq + Hash + Clone> Set<T> {
    pub fn new() -> Self { crate::fntrace::enter("new"); 
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self { crate::fntrace::enter("with_capacity"); 
        Self {
            inner: HashSet::with_capacity(capacity),
        }
    }

    pub fn contains(&self, key: &T) -> bool { crate::fntrace::enter("contains"); 
        self.inner.contains(key)
    }

    pub fn has(&self, key: &T) -> bool { crate::fntrace::enter("has"); 
        self.contains(key)
    }

    pub fn insert(&mut self, key: T) -> bool { crate::fntrace::enter("insert"); 
        self.inner.insert(key)
    }

    pub fn add(&mut self, key: T) { crate::fntrace::enter("add"); 
        self.inner.insert(key);
    }

    pub fn add_if_absent(&mut self, key: T) -> bool { crate::fntrace::enter("add_if_absent"); 
        self.inner.insert(key)
    }

    pub fn remove(&mut self, key: &T) -> bool { crate::fntrace::enter("remove"); 
        self.inner.remove(key)
    }

    pub fn delete(&mut self, key: &T) { crate::fntrace::enter("delete"); 
        self.inner.remove(key);
    }

    pub fn len(&self) -> usize { crate::fntrace::enter("len"); 
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.inner.is_empty()
    }

    pub fn clear(&mut self) { crate::fntrace::enter("clear"); 
        self.inner.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> { crate::fntrace::enter("iter"); 
        self.inner.iter()
    }

    pub fn union(&mut self, other: &Set<T>) { crate::fntrace::enter("union"); 
        for item in other.inner.iter() {
            self.inner.insert(item.clone());
        }
    }

    pub fn unioned_with(&self, other: &Set<T>) -> Set<T> { crate::fntrace::enter("unioned_with"); 
        let mut result = self.clone();
        result.union(other);
        result
    }

    pub fn equals(&self, other: &Set<T>) -> bool { crate::fntrace::enter("equals"); 
        self.inner == other.inner
    }

    pub fn is_subset_of(&self, other: &Set<T>) -> bool { crate::fntrace::enter("is_subset_of"); 
        self.inner.is_subset(&other.inner)
    }

    pub fn intersects(&self, other: &Set<T>) -> bool { crate::fntrace::enter("intersects"); 
        self.inner.iter().any(|x| other.inner.contains(x))
    }

    pub fn from_items(items: impl IntoIterator<Item = T>) -> Self { crate::fntrace::enter("from_items"); 
        let mut set = Self::new();
        for item in items {
            set.insert(item);
        }
        set
    }
}
