#![allow(dead_code)]

use tsox_core::tspath::Path;
use tsox_tsoptions::mig::m5j_2::ParsedTsconfig;

use super::owner_cache::OwnerCache;

#[derive(Clone)]
pub struct ExtendedConfigParseArgs {
    pub file_name: String,
    pub content: String,
    pub resolution_stack: Vec<Path>,
}

#[derive(Clone)]
pub struct ExtendedConfigCacheEntry {
    pub command_line: Option<ParsedTsconfig>,
    pub hash_lo: u64,
    pub hash_hi: u64,
}

pub struct ExtendedConfigCache {
    inner: OwnerCache<Path, ExtendedConfigCacheEntry>,
}

impl ExtendedConfigCache {
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        ExtendedConfigCache {
            inner: OwnerCache::new(),
        }
    }

    pub fn load_and_acquire<F>(&self, path: &Path, owner: u64, parse: F) -> ExtendedConfigCacheEntry
    where
        F: FnOnce(&Path) -> ExtendedConfigCacheEntry,
    { ::tsox_core::fntrace::enter("load_and_acquire"); 
        self.inner.load_and_acquire(path.clone(), owner, parse)
    }

    pub fn add_owner(&self, path: &Path, owner: u64) { ::tsox_core::fntrace::enter("add_owner"); 
        self.inner.add_owner(path, owner);
    }

    pub fn has(&self, path: &Path) -> bool { ::tsox_core::fntrace::enter("has"); 
        self.inner.has(path)
    }

    pub fn len(&self) -> usize { ::tsox_core::fntrace::enter("len"); 
        self.inner.len()
    }

    pub fn release(&self, path: &Path, owner: u64) { ::tsox_core::fntrace::enter("release"); 
        self.inner.release(path, owner);
    }
}

impl Default for ExtendedConfigCache {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self::new()
    }
}
