use crate::core::arena::Arena;
use crate::core::compiler_options_options::CompilerOptions;
use crate::tspath::directory_separator::combine_paths;
use crate::tspath::get_normalized_absolute_path::{EXTENSION_JSON, file_extension_is};
use crate::tspath::get_normalized_absolute_path::for_each_ancestor_directory;
use crate::tspath::supported_ts_extensions_flat::has_ts_file_extension;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub struct LinkStore<K: Eq + std::hash::Hash, V> {
    entries: HashMap<K, *mut V>,
    arena: Arena<V>,
}

impl<K: Eq + std::hash::Hash + Clone, V> LinkStore<K, V> {
    pub fn get(&mut self, key: &K) -> &mut V {
        if let Some(&value) = self.entries.get(key) {
            return unsafe { &mut *value };
        }
        let value: *mut V = self.arena.alloc(unsafe { std::mem::zeroed() });
        self.entries.insert(key.clone(), value);
        unsafe { &mut *value }
    }

    pub fn has(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    pub fn try_get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).map(|&value| unsafe { &*value })
    }
}

const PAGE_SHIFT: u64 = 8;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const PAGE_MASK: u64 = PAGE_SIZE as u64 - 1;
const MAX_PAGE_COUNT: u64 = 65536;

pub struct PagedLinkStore<V> {
    page_map: HashMap<u64, Box<[Option<V>; PAGE_SIZE]>>,
    page_list: Vec<Option<Box<[Option<V>; PAGE_SIZE]>>>,
}

impl<V> Default for PagedLinkStore<V> {
    fn default() -> Self {
        Self {
            page_map: HashMap::new(),
            page_list: Vec::new(),
        }
    }
}

impl<V> PagedLinkStore<V> {
    fn get_or_create_page(&mut self, page_index: u64) -> &mut [Option<V>; PAGE_SIZE] {
        if page_index < MAX_PAGE_COUNT {
            let index = page_index as usize;
            if index >= self.page_list.len() {
                self.page_list.resize_with(index + 1, || None);
            }
            if self.page_list[index].is_none() {
                self.page_list[index] = Some(Box::new(std::array::from_fn(|_| None)));
            }
            self.page_list[index].as_mut().unwrap()
        } else {
            self.page_map
                .entry(page_index)
                .or_insert_with(|| Box::new(std::array::from_fn(|_| None)))
        }
    }

    pub fn get(&mut self, key: u64) -> &mut Option<V> {
        let page_index = key >> PAGE_SHIFT;
        let page = self.get_or_create_page(page_index);
        &mut page[(key & PAGE_MASK) as usize]
    }

    pub fn has(&self, key: u64) -> bool {
        self.try_get(key).is_some()
    }

    pub fn try_get(&self, key: u64) -> Option<&V> {
        let page_index = key >> PAGE_SHIFT;
        let page: Option<&Box<[Option<V>; PAGE_SIZE]>> = if page_index < MAX_PAGE_COUNT {
            let index = page_index as usize;
            self.page_list.get(index).and_then(|p| p.as_ref())
        } else {
            self.page_map.get(&page_index)
        };
        page.and_then(|page| page[(key & PAGE_MASK) as usize].as_ref())
    }
}

pub const UNPREFIXED_NODE_CORE_MODULES: &[&str] = &[
    "assert",
    "assert/strict",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "dns/promises",
    "domain",
    "events",
    "fs",
    "fs/promises",
    "http",
    "http2",
    "https",
    "inspector",
    "inspector/promises",
    "module",
    "net",
    "os",
    "path",
    "path/posix",
    "path/win32",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "readline/promises",
    "repl",
    "stream",
    "stream/consumers",
    "stream/promises",
    "stream/web",
    "string_decoder",
    "sys",
    "timers",
    "timers/promises",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "util/types",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

pub const EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES: &[&str] = &[
    "node:quic",
    "node:sea",
    "node:sqlite",
    "node:test",
    "node:test/reporters",
];

pub fn node_core_modules() -> &'static HashMap<&'static str, bool> {
    static NODE_CORE_MODULES: OnceLock<HashMap<&'static str, bool>> = OnceLock::new();
    NODE_CORE_MODULES.get_or_init(|| {
        let mut modules = HashMap::new();
        for unprefixed in UNPREFIXED_NODE_CORE_MODULES {
            modules.insert(*unprefixed, true);
            let prefixed: &'static str = Box::leak(format!("node:{}", unprefixed).into_boxed_str());
            modules.insert(prefixed, true);
        }
        for exclusively_prefixed in EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES {
            modules.insert(*exclusively_prefixed, true);
        }
        modules
    })
}

pub fn non_relative_module_name_for_typing_cache(module_name: &str) -> String {
    if node_core_modules().get(module_name).copied().unwrap_or(false) {
        return "node".to_string();
    }
    module_name.to_string()
}

pub fn find_best_pattern_match<T: Clone>(
    values: &[T],
    get_pattern: impl Fn(&T) -> &crate::core::core::Pattern,
    candidate: &str,
) -> Option<T> {
    let mut best_pattern: Option<T> = None;
    let mut longest_match_prefix_length: isize = -1;
    for value in values {
        let pattern = get_pattern(value);
        if (pattern.star_index == -1 || pattern.star_index > longest_match_prefix_length)
            && pattern.matches(candidate)
        {
            best_pattern = Some(value.clone());
            longest_match_prefix_length = pattern.star_index;
        }
    }
    best_pattern
}

pub fn resolve_project_reference_path(project_reference: &crate::core::project_reference::ProjectReference) -> String {
    resolve_config_file_name_of_project_reference(&project_reference.path)
}

pub fn resolve_config_file_name_of_project_reference(path: &str) -> String {
    if file_extension_is(path, EXTENSION_JSON) {
        return path.to_string();
    }
    combine_paths(path, &["tsconfig.json"])
}

impl CompilerOptions {
    pub fn get_effective_type_roots(&self, current_directory: &str) -> (Vec<String>, bool) {
        if !self.type_roots.is_empty() {
            return (self.type_roots.clone(), true);
        }
        let base_dir = if !self.config_file_path.is_empty() {
            crate::tspath::directory_separator::get_directory_path(&self.config_file_path)
        } else {
            if current_directory.is_empty() {
                panic!(
                    "cannot get effective type roots without a config file path or current directory"
                );
            }
            current_directory.to_string()
        };
        let mut type_roots = Vec::with_capacity(base_dir.matches('/').count());
        for_each_ancestor_directory(&base_dir, &mut |dir: &str| {
            type_roots.push(combine_paths(dir, &["node_modules", "@types"]));
            false
        });
        (type_roots, false)
    }

    pub fn get_paths_base_path(&self, current_directory: &str) -> String {
        if self.paths.as_ref().is_none_or(|paths| paths.is_empty()) {
            return String::new();
        }
        if !self.paths_base_path.is_empty() {
            return self.paths_base_path.clone();
        }
        current_directory.to_string()
    }
}

pub fn get_new_line_kind(s: &str) -> crate::core::compiler_options_kinds::NewLineKind {
    match s {
        "\r\n" => crate::core::compiler_options_kinds::NewLineKind::CRLF,
        "\n" => crate::core::compiler_options_kinds::NewLineKind::LF,
        _ => crate::core::compiler_options_kinds::NewLineKind::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckerLifetime {
    Temporary,
    Diagnostics,
    Api,
}

#[derive(Clone, Default)]
pub struct RequestContext {
    pub request_id: String,
    pub checker_lifetime: Option<CheckerLifetime>,
    after_funcs: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl std::fmt::Debug for RequestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestContext")
            .field("request_id", &self.request_id)
            .field("checker_lifetime", &self.checker_lifetime)
            .finish()
    }
}

impl RequestContext {
    pub fn after_func(&self, f: Box<dyn FnOnce() + Send>) {
        self.after_funcs.lock().unwrap().push(f);
    }

    pub fn finish_request(&self) {
        let funcs = std::mem::take(&mut *self.after_funcs.lock().unwrap());
        for f in funcs {
            f();
        }
    }
}

pub fn with_request_id(ctx: &RequestContext, id: &str) -> RequestContext {
    RequestContext {
        request_id: id.to_string(),
        checker_lifetime: ctx.checker_lifetime,
        after_funcs: Arc::clone(&ctx.after_funcs),
    }
}

pub fn get_request_id(ctx: &RequestContext) -> String {
    ctx.request_id.clone()
}

pub fn with_checker_lifetime(
    ctx: &RequestContext,
    lifetime: CheckerLifetime,
) -> RequestContext {
    RequestContext {
        request_id: ctx.request_id.clone(),
        checker_lifetime: Some(lifetime),
        after_funcs: Arc::clone(&ctx.after_funcs),
    }
}

pub fn get_checker_lifetime(ctx: &RequestContext) -> CheckerLifetime {
    ctx.checker_lifetime
        .unwrap_or(CheckerLifetime::Temporary)
}
