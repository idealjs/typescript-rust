#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::tspath::Path;

use super::overlay_fs::FileHandle;
use super::refcount_cache::{RefCountCache, RefCountCacheOptions};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseCacheKey {
    pub file_name: String,
    pub path: Path,
    pub hash_lo: u64,
    pub hash_hi: u64,
    pub script_kind: i32,
}

impl ParseCacheKey {
    pub fn new(
        file_name: String,
        path: Path,
        hash_lo: u64,
        hash_hi: u64,
        script_kind: i32,
    ) -> Self {
        ParseCacheKey {
            file_name,
            path,
            hash_lo,
            hash_hi,
            script_kind,
        }
    }
}

pub struct ParseCache {
    inner: RefCountCache<ParseCacheKey, Arc<SourceFile>>,
}

impl ParseCache {
    pub fn new(options: RefCountCacheOptions) -> Self {
        ParseCache {
            inner: RefCountCache::new(options),
        }
    }

    pub fn acquire<F>(&self, key: &ParseCacheKey, _fh: &dyn FileHandle, parse: F) -> Arc<SourceFile>
    where
        F: FnOnce(&ParseCacheKey) -> Arc<SourceFile>,
    {
        self.inner.acquire(key.clone(), parse)
    }

    pub fn has(&self, key: &ParseCacheKey) -> bool {
        self.inner.has(key)
    }

    pub fn r#ref(&self, key: &ParseCacheKey) {
        self.inner.r#ref(key);
    }

    pub fn deref(&self, key: &ParseCacheKey) {
        self.inner.deref(key);
    }
}
