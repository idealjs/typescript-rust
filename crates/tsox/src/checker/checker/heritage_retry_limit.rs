#![allow(unused_imports)]

use super::*;

pub const HERITAGE_RETRY_LIMIT: u32 = 2;

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

    fn get_resolved_module(&self, _file_name: &str, _module_name: &str) -> Option<String> {
        None
    }

    fn read_file(&self, _file_name: &str) -> Option<String> {
        None
    }

    fn get_source_file_for_resolved_module(&self, _resolved_path: &str) -> Option<Arc<SourceFile>> {
        None
    }

    fn resolve_external_module_path(
        &self,
        _specifier: &str,
        _containing_file: &str,
        _resolution_mode: crate::core::compiler_options::ModuleKind,
    ) -> Option<String> {
        None
    }

    fn get_emit_module_format_of_file(
        &self,
        _file_name: &str,
    ) -> crate::core::compiler_options::ModuleKind {
        crate::core::compiler_options::ModuleKind::None
    }

    fn source_file_may_be_emitted(&self, _file_name: &str) -> bool {
        true
    }
}

#[derive(Debug, Default)]
pub struct LinkStore<K, V> {
    pub(crate) _marker: std::marker::PhantomData<K>,
    pub(crate) data: HashMap<u64, V>,
}

impl<K, V> LinkStore<K, V>
where
    K: HasId,
    V: Default,
{
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            data: HashMap::new(),
        }
    }

    pub fn get_or_default(&mut self, key: &K) -> &mut V {
        self.data.entry(key.id()).or_default()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(&key.id())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(&key.id())
    }

    pub fn insert(&mut self, key: &K, value: V) {
        self.data.insert(key.id(), value);
    }
}

pub trait HasId {
    fn id(&self) -> u64;
}

impl HasId for Node {
    fn id(&self) -> u64 {
        Node::id(self)
    }
}

impl HasId for Symbol {
    fn id(&self) -> u64 {
        self.id()
    }
}

impl HasId for SourceFile {
    fn id(&self) -> u64 {
        self.id()
    }
}

pub(crate) fn noop_entity_fn(_: &Arc<Node>, _: Option<&Arc<Node>>) -> EvalResult {
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
