//! Extended config cache (1:1 port of Go's `internal/project/extendedconfigcache.go`).

#![allow(dead_code)]

use crate::tsoptions::ParsedCommandLine;
use crate::tspath::Path;

use super::owner_cache::OwnerCache;

/// Arguments for parsing an extended config file.
///
/// Go: `type ExtendedConfigParseArgs struct { ... }`.
#[derive(Clone)]
pub struct ExtendedConfigParseArgs {
    pub file_name: String,
    pub content: String,
    pub resolution_stack: Vec<Path>,
}

/// A cache entry wrapping a parsed extended config with a content hash.
///
/// Go: `type ExtendedConfigCacheEntry struct { ... }`.
#[derive(Clone)]
pub struct ExtendedConfigCacheEntry {
    pub command_line: Option<ParsedCommandLine>,
    pub hash_lo: u64,
    pub hash_hi: u64,
}

/// An owner-tracked cache of extended tsconfig entries.
///
/// Go: `type ExtendedConfigCache = OwnerCache[tspath.Path, *ExtendedConfigCacheEntry, ExtendedConfigParseArgs]`.
pub struct ExtendedConfigCache {
    inner: OwnerCache<Path, ExtendedConfigCacheEntry>,
}

impl ExtendedConfigCache {
    pub fn new() -> Self {
        ExtendedConfigCache {
            inner: OwnerCache::new(),
        }
    }

    pub fn load_and_acquire<F>(&self, path: &Path, owner: u64, parse: F) -> ExtendedConfigCacheEntry
    where
        F: FnOnce(&Path) -> ExtendedConfigCacheEntry,
    {
        self.inner.load_and_acquire(path.clone(), owner, parse)
    }

    pub fn add_owner(&self, path: &Path, owner: u64) {
        self.inner.add_owner(path, owner);
    }

    pub fn has(&self, path: &Path) -> bool {
        self.inner.has(path)
    }

    pub fn release(&self, path: &Path, owner: u64) {
        self.inner.release(path, owner);
    }
}

impl Default for ExtendedConfigCache {
    fn default() -> Self {
        Self::new()
    }
}
