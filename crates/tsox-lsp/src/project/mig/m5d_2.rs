#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use tsox_core::tspath::Path;
use tsox_tsoptions::vfs::FS;

use crate::project::overlay_fs::{hash_string_128, Hash128};
use crate::project::parse_cache::ParseCacheKey;
use crate::project::parse_cache::ParseCache;
use crate::project::program_counter::ProgramCounter;
use crate::project::refcount_cache::{RefCountCache, RefCountCacheOptions};
use crate::project::overlay_fs::script_kind_from_file_name;

pub struct OwnerCacheEntry<V: Clone> {
    value: Mutex<Option<V>>,
    owners: Mutex<HashSet<u64>>,
}

pub struct OwnerCache<K: Eq + std::hash::Hash + Clone, V: Clone, LoadArgs: Clone> {
    entries: Mutex<HashMap<K, Arc<OwnerCacheEntry<V>>>>,
    is_expired: Option<Box<dyn Fn(&K, &V, &LoadArgs) -> bool + Send + Sync>>,
    parse: Box<dyn Fn(&K, &LoadArgs) -> V + Send + Sync>,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone, LoadArgs: Clone> OwnerCache<K, V, LoadArgs> {
    pub fn new(
        parse: impl Fn(&K, &LoadArgs) -> V + Send + Sync + 'static,
        is_expired: Option<impl Fn(&K, &V, &LoadArgs) -> bool + Send + Sync + 'static>,
    ) -> Self { ::tsox_core::fntrace::enter("new"); 
        OwnerCache {
            entries: Mutex::new(HashMap::new()),
            is_expired: is_expired
                .map(|f| Box::new(f) as Box<dyn Fn(&K, &V, &LoadArgs) -> bool + Send + Sync>),
            parse: Box::new(parse),
        }
    }

    pub fn load_and_acquire(&self, identity: K, owner: u64, load_args: LoadArgs) -> V { ::tsox_core::fntrace::enter("load_and_acquire"); 
        let (entry, loaded) = self.load_or_store_locked_entry(identity.clone());
        let mut value = entry.value.lock().unwrap();
        let expired = loaded
            && self
                .is_expired
                .as_ref()
                .map(|f| f.as_ref()(&identity, value.as_ref().unwrap(), &load_args))
                .unwrap_or(false);
        if !loaded || expired {
            *value = Some((self.parse)(&identity, &load_args));
        }
        entry.owners.lock().unwrap().insert(owner);
        value.clone().unwrap()
    }

    pub fn acquire(&self, identity: K, owner: u64, value: V) { ::tsox_core::fntrace::enter("acquire"); 
        let (entry, loaded) = self.load_or_store_locked_entry(identity);
        if !loaded {
            *entry.value.lock().unwrap() = Some(value);
        }
        entry.owners.lock().unwrap().insert(owner);
    }

    pub fn load_or_store_locked_entry(&self, key: K) -> (Arc<OwnerCacheEntry<V>>, bool) { ::tsox_core::fntrace::enter("load_or_store_locked_entry"); 
        let mut entries = self.entries.lock().unwrap();
        if let Some(existing) = entries.get(&key) {
            return (existing.clone(), true);
        }
        let entry = Arc::new(OwnerCacheEntry {
            value: Mutex::new(None),
            owners: Mutex::new(HashSet::new()),
        });
        entries.insert(key, entry.clone());
        (entry, false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SourceFileParseOptions {
    pub file_name: String,
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentMappedParseCacheKey {
    pub options: SourceFileParseOptions,
    pub hash: Hash128,
}

pub fn new_parse_cache_key(
    options: SourceFileParseOptions,
    hash: Hash128,
    script_kind: i32,
) -> ParseCacheKey { ::tsox_core::fntrace::enter("new_parse_cache_key"); 
    let mut kind = script_kind;
    if kind == 0 {
        kind = script_kind_from_file_name(&options.file_name);
    }
    ParseCacheKey::new(options.file_name, options.path, hash.lo, hash.hi, kind)
}

pub fn content_mapped_parse_cache_key(
    options: SourceFileParseOptions,
    raw_hash: Hash128,
    transform_identity: Hash128,
    diagnostic_locale: &str,
) -> ContentMappedParseCacheKey { ::tsox_core::fntrace::enter("content_mapped_parse_cache_key"); 
    let mut buf = Vec::with_capacity(32 + diagnostic_locale.len());
    buf.extend_from_slice(&raw_hash.hi.to_le_bytes());
    buf.extend_from_slice(&raw_hash.lo.to_le_bytes());
    buf.extend_from_slice(&transform_identity.hi.to_le_bytes());
    buf.extend_from_slice(&transform_identity.lo.to_le_bytes());
    buf.extend_from_slice(diagnostic_locale.as_bytes());
    ContentMappedParseCacheKey {
        options,
        hash: hash_bytes_128(&buf),
    }
}

pub fn source_file_parse_options(file_name: &str) -> SourceFileParseOptions { ::tsox_core::fntrace::enter("source_file_parse_options"); 
    SourceFileParseOptions {
        file_name: file_name.to_string(),
        path: tsox_core::tspath::to_path(file_name, "", false),
    }
}

fn duplicate_hash_128(hash: u128) -> Hash128 { ::tsox_core::fntrace::enter("duplicate_hash_128"); 
    Hash128 {
        lo: hash as u64,
        hi: (hash >> 64) as u64,
    }
}

pub fn parse_cache_key_for_file(file: &tsox_frontend::ast::SourceFile) -> ParseCacheKey { ::tsox_core::fntrace::enter("parse_cache_key_for_file"); 
    new_parse_cache_key(
        source_file_parse_options(&file.file_name),
        crate::project::overlay_fs::hash_string_128(&file.text),
        file.script_kind as i32,
    )
}

pub fn content_mapped_parse_cache_key_for_file(
    file: &tsox_frontend::ast::SourceFile,
) -> ContentMappedParseCacheKey { ::tsox_core::fntrace::enter("content_mapped_parse_cache_key_for_file"); 
    ContentMappedParseCacheKey {
        options: source_file_parse_options(&file.file_name),
        hash: crate::project::overlay_fs::hash_string_128(&file.text),
    }
}

pub fn parse_cache_key_for_duplicate(
    file: &tsox_compile::compiler::DuplicateSourceFile,
) -> ParseCacheKey { ::tsox_core::fntrace::enter("parse_cache_key_for_duplicate"); 
    new_parse_cache_key(
        source_file_parse_options(&file.file_name),
        duplicate_hash_128(file.hash),
        file.script_kind as i32,
    )
}

pub fn content_mapped_parse_cache_key_for_duplicate(
    file: &tsox_compile::compiler::DuplicateSourceFile,
) -> ContentMappedParseCacheKey { ::tsox_core::fntrace::enter("content_mapped_parse_cache_key_for_duplicate"); 
    ContentMappedParseCacheKey {
        options: source_file_parse_options(&file.file_name),
        hash: duplicate_hash_128(file.hash),
    }
}

pub fn hash_bytes_128(bytes: &[u8]) -> Hash128 { ::tsox_core::fntrace::enter("hash_bytes_128"); 
    use std::hash::Hasher;
    use xxhash_rust::xxh3::Xxh3;
    let mut hasher = Xxh3::new();
    hasher.write(bytes);
    let lo = hasher.finish();
    let mut hasher2 = Xxh3::new();
    hasher2.write(bytes);
    hasher2.write(&[0x42]);
    let hi = hasher2.finish();
    Hash128 { lo, hi }
}

pub fn new_parse_cache(options: RefCountCacheOptions) -> ParseCache { ::tsox_core::fntrace::enter("new_parse_cache"); 
    ParseCache::new(options)
}

pub struct ContentMappedParseCache {
    pub inner: RefCountCache<ContentMappedParseCacheKey, tsox_compile::mig::m3l_cm_2::SourceFiles>,
}

impl ContentMappedParseCache {
    pub fn new(options: RefCountCacheOptions) -> Self { ::tsox_core::fntrace::enter("new"); 
        ContentMappedParseCache {
            inner: RefCountCache::new(options),
        }
    }
}

pub fn new_content_mapped_parse_cache(
    options: RefCountCacheOptions,
) -> ContentMappedParseCache { ::tsox_core::fntrace::enter("new_content_mapped_parse_cache"); 
    ContentMappedParseCache::new(options)
}

impl ProgramCounter {
    pub fn ref_program(&mut self, program: &Arc<tsox_compile::compiler::Program>) { ::tsox_core::fntrace::enter("ref_program"); 
        self.r#ref(program);
    }
}

pub fn new_ref_count_cache<K: Eq + std::hash::Hash + Clone, V: Clone>(
    options: RefCountCacheOptions,
) -> RefCountCache<K, V> { ::tsox_core::fntrace::enter("new_ref_count_cache"); 
    RefCountCache::new(options)
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> RefCountCache<K, V> {
    pub fn acquire_or_error<F>(&self, identity: K, produce: F) -> Result<V, String>
    where
        F: FnOnce() -> Result<V, String>,
    { ::tsox_core::fntrace::enter("acquire_or_error"); 
        if self.has(&identity) {
            return Ok(self.acquire(identity, |_| {
                panic!("cache entry disappeared between has and acquire")
            }));
        }
        match produce() {
            Ok(value) => {
                self.acquire(identity, |_| value.clone());
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    pub fn ref_entry(&self, identity: &K) { ::tsox_core::fntrace::enter("ref_entry"); 
        self.r#ref(identity);
    }
}
