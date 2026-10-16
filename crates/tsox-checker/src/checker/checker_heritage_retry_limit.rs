#![allow(unused_imports)]

use crate::checker::checker::*;

pub const HERITAGE_RETRY_LIMIT: u32 = 100;

pub const ALIAS_SELF_INSTANTIATION_DEPTH: usize = 8;

pub const EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT: u32 = 1 << 0;
pub const EXTERNAL_EMIT_HELPER_IMPORT_STAR: u32 = 1 << 1;
pub const EXTERNAL_EMIT_HELPER_EXPORT_STAR: u32 = 1 << 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TypeResolutionProperty {
    Type,

    DeclaredType,

    ResolvedBaseTypes,

    ResolvedBaseConstructorType,

    ResolvedReturnType,

    ResolvedTypeArguments,

    ResolvedBaseConstraint,

    WriteType,

    InitializerIsUndefined,
}

#[derive(Clone, Copy)]
pub struct TypeResolutionEntry {
    pub target: *const Symbol,

    pub property: TypeResolutionProperty,

    pub result: bool,
}

unsafe impl Send for TypeResolutionEntry {}
unsafe impl Sync for TypeResolutionEntry {}

pub trait Program: Send + Sync {
    fn options(&self) -> &CompilerOptions;
    fn source_files(&self) -> &[Arc<SourceFile>];
    fn bind_source_files(&self);
    fn file_exists(&self, file_name: &str) -> bool;
    fn get_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>>;
    fn is_source_file_default_library(&self, path: &str) -> bool;

    fn symbol_map(&self) -> &NodeSymbolMap;

    fn current_directory(&self) -> &str;

    fn use_case_sensitive_file_names(&self) -> bool;

    fn common_source_directory(&self) -> String;

    fn get_resolved_module(&self, _file_name: &str, _module_name: &str) -> Option<String> { ::tsox_core::fntrace::enter("get_resolved_module");
        None
    }

    // Go checker 的模块符号解析一律走程序级解析记录
    //（resolveExternalModule → GetResolvedModule）；未接记录的实现（如 LSP
    // AliasResolver）由调用方回落文件表解析
    fn records_module_resolutions(&self) -> bool { ::tsox_core::fntrace::enter("records_module_resolutions");
        false
    }

    fn get_resolved_modules(
        &self,
    ) -> std::collections::HashMap<String, Vec<(String, Option<tsox_tsoptions::module::ResolvedModule>)>>
    { ::tsox_core::fntrace::enter("get_resolved_modules"); 
        std::collections::HashMap::new()
    }

    fn read_file(&self, _file_name: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        None
    }

    fn canonicalize_path(&self, path: &str) -> String { ::tsox_core::fntrace::enter("canonicalize_path"); 
        path.to_string()
    }

    fn get_source_file_for_resolved_module(&self, _resolved_path: &str) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file_for_resolved_module"); 
        None
    }

    fn resolve_external_module_path(
        &self,
        _specifier: &str,
        _containing_file: &str,
        _resolution_mode: tsox_core::core::compiler_options::ModuleKind,
    ) -> Option<String> { ::tsox_core::fntrace::enter("resolve_external_module_path"); 
        None
    }

    fn get_emit_module_format_of_file(
        &self,
        _file_name: &str,
    ) -> tsox_core::core::compiler_options::ModuleKind { ::tsox_core::fntrace::enter("get_emit_module_format_of_file"); 
        tsox_core::core::compiler_options::ModuleKind::None
    }

    fn source_file_may_be_emitted(&self, _file_name: &str) -> bool { ::tsox_core::fntrace::enter("source_file_may_be_emitted"); 
        true
    }

    fn diagnostics(&self) -> &[Arc<tsox_frontend::ast::Diagnostic>] { ::tsox_core::fntrace::enter("diagnostics"); 
        &[]
    }

    fn has_semantic_diagnostics(&self) -> bool { ::tsox_core::fntrace::enter("has_semantic_diagnostics"); 
        false
    }
}

// 对齐 tsgo core.PagedLinkStore(linkstore.go):键为全局单调 id(Node/
// Symbol/SourceFile),条目存 256 槽定长页,低页号走 Vec 页表、高页号
// (>=16M id)走 HashMap 兜底;查找为下标算术,零哈希。槽位 Option<V>
// 保留精确 per-key 语义(tsgo 页级 Has 更粗,此处不放宽)。
const LINK_PAGE_SHIFT: u32 = 8;
const LINK_PAGE_SIZE: usize = 1 << LINK_PAGE_SHIFT;
const LINK_PAGE_MASK: u64 = (LINK_PAGE_SIZE - 1) as u64;
const LINK_MAX_PAGE_COUNT: u64 = 65536;

type LinkPage<V> = Box<[Option<V>; LINK_PAGE_SIZE]>;

#[derive(Debug, Default)]
pub struct LinkStore<K, V> {
    pub(crate) _marker: std::marker::PhantomData<K>,
    low_pages: Vec<Option<LinkPage<V>>>,
    high_pages: HashMap<u64, LinkPage<V>>,
}

impl<K, V> LinkStore<K, V>
where
    K: HasId,
    V: Default,
{
    pub fn new() -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            _marker: std::marker::PhantomData,
            low_pages: Vec::new(),
            high_pages: HashMap::new(),
        }
    }

    fn page(&self, page_index: u64) -> Option<&LinkPage<V>> {
        if page_index < LINK_MAX_PAGE_COUNT {
            self.low_pages.get(page_index as usize).and_then(|p| p.as_ref())
        } else {
            self.high_pages.get(&page_index)
        }
    }

    fn page_mut(&mut self, page_index: u64) -> &mut LinkPage<V> {
        if page_index < LINK_MAX_PAGE_COUNT {
            let i = page_index as usize;
            if i >= self.low_pages.len() {
                self.low_pages.resize_with(i + 1, || None);
            }
            self.low_pages[i]
                .get_or_insert_with(|| Box::new(std::array::from_fn(|_| None)))
        } else {
            self.high_pages
                .entry(page_index)
                .or_insert_with(|| Box::new(std::array::from_fn(|_| None)))
        }
    }

    pub fn get_or_default(&mut self, key: &K) -> &mut V { ::tsox_core::fntrace::enter("get_or_default"); 
        let id = key.id();
        self.page_mut(id >> LINK_PAGE_SHIFT)[(id & LINK_PAGE_MASK) as usize]
            .get_or_insert_with(V::default)
    }

    pub fn get(&self, key: &K) -> Option<&V> { ::tsox_core::fntrace::enter("get"); 
        let id = key.id();
        self.page(id >> LINK_PAGE_SHIFT)?[(id & LINK_PAGE_MASK) as usize].as_ref()
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> { ::tsox_core::fntrace::enter("get_mut"); 
        let id = key.id();
        if id >> LINK_PAGE_SHIFT < LINK_MAX_PAGE_COUNT {
            let i = (id >> LINK_PAGE_SHIFT) as usize;
            if i >= self.low_pages.len() {
                return None;
            }
            self.low_pages[i]
                .as_mut()?[(id & LINK_PAGE_MASK) as usize]
                .as_mut()
        } else {
            self.high_pages
                .get_mut(&(id >> LINK_PAGE_SHIFT))?[(id & LINK_PAGE_MASK) as usize]
                .as_mut()
        }
    }

    pub fn insert(&mut self, key: &K, value: V) { ::tsox_core::fntrace::enter("insert"); 
        let id = key.id();
        self.page_mut(id >> LINK_PAGE_SHIFT)[(id & LINK_PAGE_MASK) as usize] = Some(value);
    }

    /// 移除不分配页(tsgo 无对应;本侧 inference 重算路径需要)
    pub fn remove(&mut self, key: &K) -> Option<V> { ::tsox_core::fntrace::enter("remove"); 
        let id = key.id();
        let page = if id >> LINK_PAGE_SHIFT < LINK_MAX_PAGE_COUNT {
            let i = (id >> LINK_PAGE_SHIFT) as usize;
            if i >= self.low_pages.len() {
                return None;
            }
            self.low_pages.get_mut(i)?.as_mut()?
        } else {
            self.high_pages.get_mut(&(id >> LINK_PAGE_SHIFT))?
        };
        page[(id & LINK_PAGE_MASK) as usize].take()
    }
}



pub trait HasId {
    fn id(&self) -> u64;
}

impl HasId for Node {
    fn id(&self) -> u64 { ::tsox_core::fntrace::enter("id"); 
        Node::id(self)
    }
}

impl HasId for Symbol {
    fn id(&self) -> u64 { ::tsox_core::fntrace::enter("id"); 
        self.id()
    }
}

impl HasId for SourceFile {
    fn id(&self) -> u64 { ::tsox_core::fntrace::enter("id"); 
        self.id()
    }
}

pub(crate) fn noop_entity_fn(_: &Arc<Node>, _: Option<&Arc<Node>>) -> EvalResult { ::tsox_core::fntrace::enter("noop_entity_fn"); 
    EvalResult::none()
}

pub(crate) static NEXT_CHECKER_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakContinueContextKind {
    Loop,

    Switch,

    Function,

    Labeled,
}

#[derive(Debug, Clone)]
pub struct BreakContinueContext {
    pub kind: BreakContinueContextKind,

    pub label: Option<String>,

    pub is_iteration: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThisContainerKind {
    StaticMember,

    InstanceMember,

    PlainFunction,
}
