use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tsox_tsoptions::vfs::FS;

use super::super::dirty_box_::DirtyBox;
use super::super::dirty_map_::{DirtyMap, MapEntry};
use super::super::dirty_map_builder::MapBuilder;
use super::super::extended_config_cache::{ExtendedConfigCacheEntry, ExtendedConfigParseArgs};
use super::super::logging_logger::LoggerImpl;
use super::m5d_2::hash_bytes_128;

pub fn extended_config_entry_hash(
    args: &ExtendedConfigParseArgs,
    extended_source_files: &[String],
    fs: &dyn FS,
) -> (u64, u64) { ::tsox_core::fntrace::enter("extended_config_entry_hash"); 
    let mut buf = Vec::new();
    buf.extend_from_slice(args.content.as_bytes());
    for file_name in extended_source_files {
        let Some(content) = fs.read_file(file_name) else {
            return (0, 0);
        };
        buf.extend_from_slice(content.as_bytes());
    }
    let hash = hash_bytes_128(&buf);
    (hash.lo, hash.hi)
}

pub fn new_test_logger() -> LoggerImpl { ::tsox_core::fntrace::enter("new_test_logger"); 
    LoggerImpl::new(Box::new(std::io::sink()))
}

pub trait CloneableMap<K, V> {
    fn clone_map(&self) -> Self;
}

impl<K, V> CloneableMap<K, V> for HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn clone_map(&self) -> Self { ::tsox_core::fntrace::enter("clone_map"); 
        self.clone()
    }
}

impl<T: Clone + Default> DirtyBox<T> {
    pub fn locked<F: FnOnce(&mut DirtyBox<T>)>(&mut self, f: F) { ::tsox_core::fntrace::enter("locked"); 
        f(self)
    }

    pub fn finalize_owned(&self) -> (T, bool) { ::tsox_core::fntrace::enter("finalize_owned"); 
        DirtyBox::finalize(self)
    }
}

impl<K: Clone + Eq + Hash, V: Clone> MapEntry<K, V> {
    pub fn change_if_map_entry<C, A>(&mut self, cond: C, mut apply: A) -> bool
    where
        C: FnOnce(&V) -> bool,
        A: FnMut(&mut V),
    { ::tsox_core::fntrace::enter("change_if_map_entry"); 
        if self.delete {
            panic!("tried to change a deleted entry");
        }
        if cond(&self.value) {
            self.dirty = true;
            apply(&mut self.value);
            true
        } else {
            false
        }
    }

    pub fn replace(&mut self, new_value: V) { ::tsox_core::fntrace::enter("replace"); 
        if self.delete {
            panic!("tried to change a deleted entry");
        }
        self.dirty = true;
        self.value = new_value;
    }

    pub fn delete_entry(&mut self) { ::tsox_core::fntrace::enter("delete_entry"); 
        self.delete = true;
    }

    pub fn locked<F: FnOnce(&mut MapEntry<K, V>)>(&mut self, f: F) { ::tsox_core::fntrace::enter("locked"); 
        f(self)
    }
}

impl<K: Clone + Eq + Hash, V: Clone> DirtyMap<K, V> {
    pub fn delete(&mut self, key: &K) { ::tsox_core::fntrace::enter("delete"); 
        if !self.try_delete(key) {
            panic!("tried to delete a non-existent entry");
        }
    }
}

pub struct MapBuilderFull<K: Eq + Hash + Clone, VBase: Clone, VBuilder: Clone> {
    base: HashMap<K, VBase>,
    dirty: HashMap<K, VBuilder>,
    deleted: std::collections::HashSet<K>,
    to_builder: Arc<dyn Fn(&VBase) -> VBuilder + Send + Sync>,
    build: Arc<dyn Fn(&VBuilder) -> VBase + Send + Sync>,
}

impl<K: Eq + Hash + Clone, VBase: Clone, VBuilder: Clone> MapBuilderFull<K, VBase, VBuilder> {
    pub fn new(
        base: HashMap<K, VBase>,
        to_builder: Arc<dyn Fn(&VBase) -> VBuilder + Send + Sync>,
        build: Arc<dyn Fn(&VBuilder) -> VBase + Send + Sync>,
    ) -> Self { ::tsox_core::fntrace::enter("new"); 
        MapBuilderFull {
            base,
            dirty: HashMap::new(),
            deleted: std::collections::HashSet::new(),
            to_builder,
            build,
        }
    }

    pub fn set(&mut self, key: K, value: VBuilder) { ::tsox_core::fntrace::enter("set"); 
        self.dirty.insert(key.clone(), value);
        self.deleted.remove(&key);
    }

    pub fn delete(&mut self, key: &K) { ::tsox_core::fntrace::enter("delete"); 
        self.deleted.insert(key.clone());
        self.dirty.remove(key);
    }

    pub fn clear(&mut self) { ::tsox_core::fntrace::enter("clear"); 
        self.dirty = HashMap::new();
        self.deleted = self.base.keys().cloned().collect();
    }

    pub fn has(&self, key: &K) -> bool { ::tsox_core::fntrace::enter("has"); 
        if self.deleted.contains(key) {
            return false;
        }
        if self.dirty.contains_key(key) {
            return true;
        }
        self.base.contains_key(key)
    }

    pub fn build(&self) -> HashMap<K, VBase> { ::tsox_core::fntrace::enter("build"); 
        if self.dirty.is_empty() && self.deleted.is_empty() {
            return self.base.clone();
        }
        let mut result = self.base.clone();
        for key in &self.deleted {
            result.remove(key);
        }
        for (key, value) in &self.dirty {
            result.insert(key.clone(), (self.build)(value));
        }
        result
    }
}

pub fn clone_map_if_nil<K, V, T>(
    dirty: &T,
    original: Option<&T>,
    get_map: impl Fn(&T) -> Option<&HashMap<K, V>>,
) -> HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{ ::tsox_core::fntrace::enter("clone_map_if_nil"); 
    if let Some(dirty_map) = get_map(dirty) {
        return dirty_map.clone();
    }
    match original {
        None => HashMap::new(),
        Some(original) => match get_map(original) {
            None => HashMap::new(),
            Some(original_map) => original_map.clone(),
        },
    }
}

impl<K: Eq + Hash + Clone, VBase: Clone> MapBuilder<K, VBase> {
    pub fn with_converters<VBuilder>(
        base: HashMap<K, VBase>,
        to_builder: impl Fn(&VBase) -> VBuilder + Send + Sync + 'static,
        build: impl Fn(&VBuilder) -> VBase + Send + Sync + 'static,
    ) -> MapBuilderFull<K, VBase, VBuilder>
    where
        VBase: 'static,
        VBuilder: Clone + 'static,
    { ::tsox_core::fntrace::enter("with_converters"); 
        MapBuilderFull::new(base, Arc::new(to_builder), Arc::new(build))
    }
}
