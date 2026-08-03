//! Parse cache for source files (1:1 port of Go's `internal/project/parsecache.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::tspath::Path;

use super::overlay_fs::FileHandle;
use super::refcount_cache::{RefCountCache, RefCountCacheOptions};

/// A key identifying a cached parsed source file.
///
/// Go: `type ParseCacheKey struct { ... }`.
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

/// A ref-counted cache of parsed source files.
///
/// Go: `type ParseCache = RefCountCache[ParseCacheKey, *ast.SourceFile, FileHandle]`.
///
/// In Rust we store `Arc<SourceFile>` values keyed by `ParseCacheKey`.
pub struct ParseCache {
    inner: RefCountCache<ParseCacheKey, Arc<SourceFile>>,
}

impl ParseCache {
    pub fn new(options: RefCountCacheOptions) -> Self {
        ParseCache {
            inner: RefCountCache::new(options),
        }
    }

    /// Acquire (or create) a source file for the given key and file handle.
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
