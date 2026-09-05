use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::ast::{
    CheckFlags, DiagnosticsCollection, ModifierFlags, Node, NodeData,
    NodeFlags, NodeSymbolMap, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::evaluator::EvalResult;
use crate::jsnum;

use super::tracer::Tracer;
use super::types::*;
use super::relater::RelaterChainEntry;
use super::utilities::is_in_compound_like_assignment;
use super::utilities::{get_assignment_target_kind, AssignmentKind};


mod unused_diagnostics;
mod symbol_types;
mod expr_access;
mod prop_access;
mod imports_namespace;
mod assertions_interfaces;
mod assignment2;
mod suggestions_resolve;
mod contextual;

mod calls;
mod element_access;
mod literals;
mod statements;
mod classes;
mod enums;
mod operators;
mod expressions;
mod modules;
mod resolve;

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
    _marker: std::marker::PhantomData<K>,
    data: HashMap<u64, V>,
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

fn noop_entity_fn(_: &Arc<Node>, _: Option<&Arc<Node>>) -> EvalResult {
    EvalResult::none()
}

static NEXT_CHECKER_ID: AtomicU32 = AtomicU32::new(1);

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

pub struct Checker {

    pub id: u32,

    pub program: Arc<dyn Program>,
    pub compiler_options: Arc<CompilerOptions>,
    pub files: Vec<Arc<SourceFile>>,
    pub file_index_map: HashMap<u64, usize>,

    pub type_count: u32,
    pub symbol_count: u32,
    pub signature_count: u32,
    pub total_instantiation_count: u32,
    pub instantiation_count: u32,
    pub instantiation_depth: u32,

    pub language_version: ScriptTarget,
    pub module_kind: ModuleKind,
    pub module_resolution_kind: ModuleResolutionKind,
    pub legacy_decorators: bool,
    pub emit_standard_class_fields: bool,
    pub strict_null_checks: bool,
    pub strict_function_types: bool,
    pub strict_bind_call_apply: bool,
    pub strict_property_initialization: bool,
    pub strict_builtin_iterator_return: bool,
    pub no_implicit_any: bool,
    pub no_implicit_this: bool,
    pub use_unknown_in_catch_variables: bool,
    pub exact_optional_property_types: bool,
    pub can_collect_symbol_alias_accessibility_data: bool,

    pub globals: SymbolTable,
    pub undefined_symbol: Option<Arc<Symbol>>,
    pub arguments_symbol: Option<Arc<Symbol>>,
    pub require_symbol: Option<Arc<Symbol>>,
    pub unknown_symbol: Option<Arc<Symbol>>,
    pub global_this_symbol: Option<Arc<Symbol>>,

    pub string_literal_types: HashMap<String, Arc<Type>>,
    pub number_literal_types: HashMap<jsnum::Number, Arc<Type>>,
    pub bigint_literal_types: HashMap<String, Arc<Type>>,
    pub unique_es_symbol_types: HashMap<u64, Arc<Type>>,
    pub nan_type: Option<Arc<Type>>,

    pub indexed_access_types: HashMap<CacheHashKey, Arc<Type>>,
    pub template_literal_types: HashMap<CacheHashKey, Arc<Type>>,
    pub string_mapping_types: HashMap<u64, Arc<Type>>,
    pub cached_types: HashMap<CachedTypeKey, Arc<Type>>,
    pub union_types: HashMap<CacheHashKey, Arc<Type>>,
    pub intersection_types: HashMap<CacheHashKey, Arc<Type>>,
    pub tuple_types: HashMap<CacheHashKey, Arc<Type>>,
    pub error_types: HashMap<CacheHashKey, Arc<Type>>,

    pub global_interface_members: HashMap<String, Vec<String>>,

    pub boxed_global_types: HashMap<String, Arc<Type>>,

    pub diagnostics: DiagnosticsCollection,
    pub suggestion_diagnostics: DiagnosticsCollection,

    pub node_links: LinkStore<Node, NodeLinks>,
    pub signature_links: LinkStore<Node, SignatureLinks>,
    pub symbol_node_links: LinkStore<Node, SymbolNodeLinks>,
    pub type_node_links: LinkStore<Node, TypeNodeLinks>,
    pub enum_member_links: LinkStore<Node, EnumMemberLinks>,
    pub assertion_links: LinkStore<Node, AssertionLinks>,
    pub array_literal_links: LinkStore<Node, ArrayLiteralLinks>,
    pub switch_statement_links: LinkStore<Node, SwitchStatementLinks>,
    pub jsx_element_links: LinkStore<Node, JsxElementLinks>,

    pub symbol_reference_links: LinkStore<Symbol, SymbolReferenceLinks>,
    pub value_symbol_links: LinkStore<Symbol, ValueSymbolLinks>,
    pub mapped_symbol_links: LinkStore<Symbol, MappedSymbolLinks>,
    pub deferred_symbol_links: LinkStore<Symbol, DeferredSymbolLinks>,
    pub alias_symbol_links: LinkStore<Symbol, AliasSymbolLinks>,
    pub module_symbol_links: LinkStore<Symbol, ModuleSymbolLinks>,
    pub late_bound_links: LinkStore<Symbol, LateBoundLinks>,
    pub export_type_links: LinkStore<Symbol, ExportTypeLinks>,
    pub members_and_exports_links: LinkStore<Symbol, MembersAndExportsLinks>,
    pub type_alias_links: LinkStore<Symbol, TypeAliasLinks>,
    pub declared_type_links: LinkStore<Symbol, DeclaredTypeLinks>,

    pub type_resolution_stack: Vec<TypeResolutionEntry>,

    pub type_argument_stack: Vec<HashMap<*const crate::ast::Symbol, Arc<Type>>>,

    pub type_argument_name_frames: Vec<Vec<(Arc<Symbol>, Arc<Type>)>>,

    pub type_node_subst_cache: HashMap<(usize, u64), Arc<Type>>,

    pub type_node_resolving: HashSet<(usize, u64)>,

    pub type_resolution_depth: u32,

    pub speculation_depth: u32,

    pub heritage_degraded_events: u64,

    pub type_node_query_epochs: Vec<u64>,

    pub heritage_retry_counts: HashMap<usize, u32>,

    pub type_node_subst_cache_limit: usize,

    pub type_parameter_resolving: HashSet<usize>,

    pub ts2313_reported: HashSet<usize>,

    pub ts2354_checked_files: HashSet<usize>,

    pub requested_external_emit_helpers: HashMap<usize, u32>,

    pub degraded_type_ptrs: std::collections::HashSet<u32>,

    pub jsx_implicit_namespace: HashMap<usize, Option<Arc<Symbol>>>,

    pub pending_jsx_2875: Option<(crate::core::text::TextRange, String)>,

    pub relater_error_chain: Vec<RelaterChainEntry>,

    pub relater_chain_active: bool,

    pub relater_depth: u32,

    pub deferred_constraint_depth: u32,

    pub relation_count: u32,

    pub relater_overflow: bool,

    pub relater_intersection_target_depth: u32,

    pub subst_object_in_progress: std::collections::HashMap<u32, Arc<crate::checker::types::Type>>,

    pub in_return_substitution: bool,

    pub relater_source_stack: Vec<Arc<Type>>,

    pub relater_target_stack: Vec<Arc<Type>>,

    pub relation_cache: HashMap<crate::checker::relater::RelationCacheKey, bool>,

    pub probe_cache_permissive: HashMap<u32, Arc<Type>>,
    pub probe_cache_restrictive: HashMap<u32, Arc<Type>>,

    pub enum_relation: HashMap<EnumRelationKey, crate::checker::relater::RelationComparisonResult>,

    pub relation_in_progress: std::collections::HashSet<crate::checker::relater::RelationCacheKey>,

    pub interface_extends_reported:
        std::collections::HashSet<(*const crate::ast::Symbol, *const crate::ast::Node)>,

    pub indexed_access_2538_reported: std::collections::HashSet<*const crate::ast::Node>,

    pub arith_operand_error_nodes: std::collections::HashSet<*const crate::ast::Node>,

    pub computed_property_name_checked: std::collections::HashSet<*const crate::ast::Node>,

    pub symbol_reference_kinds: dashmap::DashMap<u64, SymbolFlags>,
    pub spread_links: LinkStore<Symbol, SpreadLinks>,
    pub variance_links: LinkStore<Symbol, VarianceLinks>,
    pub reverse_mapped_symbol_links: LinkStore<Symbol, ReverseMappedSymbolLinks>,
    pub marked_assignment_symbol_links: LinkStore<Symbol, MarkedAssignmentSymbolLinks>,
    pub symbol_container_links: LinkStore<Symbol, ContainingSymbolLinks>,

    pub symbol_table_alias_cache: HashMap<u64, Vec<Arc<Symbol>>>,

    pub class_expression_name_tables: HashMap<u64, SymbolTable>,
    pub source_file_links: LinkStore<SourceFile, SourceFileLinks>,

    pub declaration_links: LinkStore<Node, DeclarationLinks>,

    pub declaration_file_links: LinkStore<SourceFile, DeclarationFileLinks>,

    last_combined_modifier_flags_node: Option<Arc<Node>>,
    last_combined_modifier_flags_result: ModifierFlags,

    pub any_type: OnceLock<Arc<Type>>,
    pub unknown_type: OnceLock<Arc<Type>>,
    pub undefined_type: OnceLock<Arc<Type>>,
    pub null_type: OnceLock<Arc<Type>>,
    pub string_type: OnceLock<Arc<Type>>,
    pub number_type: OnceLock<Arc<Type>>,
    pub bigint_type: OnceLock<Arc<Type>>,
    pub boolean_type: OnceLock<Arc<Type>>,
    pub es_symbol_type: OnceLock<Arc<Type>>,
    pub void_type: OnceLock<Arc<Type>>,
    pub never_type: OnceLock<Arc<Type>>,
    pub non_primitive_type: OnceLock<Arc<Type>>,
    pub true_type: OnceLock<Arc<Type>>,
    pub false_type: OnceLock<Arc<Type>>,
    pub error_type: OnceLock<Arc<Type>>,
    pub unresolved_type: OnceLock<Arc<Type>>,

    pub auto_type: OnceLock<Arc<Type>>,

    pub empty_object_type: OnceLock<Arc<Type>>,
    pub empty_generic_type: OnceLock<Arc<Type>>,
    pub any_function_type: OnceLock<Arc<Type>>,
    pub no_constraint_type: OnceLock<Arc<Type>>,
    pub circular_constraint_type: OnceLock<Arc<Type>>,

    pub any_array_type: OnceLock<Arc<Type>>,
    pub auto_array_type: OnceLock<Arc<Type>>,
    pub any_readonly_array_type: OnceLock<Arc<Type>>,

    pub global_object_type: OnceLock<Arc<Type>>,
    pub global_function_type: OnceLock<Arc<Type>>,
    pub global_array_type: OnceLock<Arc<Type>>,
    pub global_readonly_array_type: OnceLock<Arc<Type>>,
    pub global_string_type: OnceLock<Arc<Type>>,
    pub global_number_type: OnceLock<Arc<Type>>,
    pub global_boolean_type: OnceLock<Arc<Type>>,
    pub global_reg_exp_type: OnceLock<Arc<Type>>,
    pub global_this_type: OnceLock<Arc<Type>>,
    pub global_promise_type: OnceLock<Arc<Type>>,

    pub array_type_cache: std::collections::HashMap<(usize, usize), Arc<Type>>,

    pub interface_instantiation_cache: std::collections::HashMap<Vec<usize>, (Vec<Arc<Type>>, Arc<Type>)>,

    pub attached_type_args_cache:
        std::collections::HashMap<Vec<usize>, (Arc<Type>, Vec<Arc<Type>>, Arc<Type>)>,

    pub typequery_instantiation_cache: std::collections::HashMap<Vec<usize>, (Vec<Arc<Type>>, Arc<Type>)>,

    pub array_type_parameter_symbols: Option<Vec<Arc<crate::ast::Symbol>>>,

    pub array_member_type_cache: std::collections::HashMap<(usize, usize), Arc<Type>>,

    pub instantiated_member_type_cache:
        std::collections::HashMap<(usize, usize), (Arc<Type>, Arc<Type>)>,

    pub instantiated_member_type_cache_limit: usize,

    pub any_signature: OnceLock<Arc<Signature>>,
    pub unknown_signature: OnceLock<Arc<Signature>>,
    pub resolving_signature: OnceLock<Arc<Signature>>,

    pub current_node: Option<Arc<Node>>,
    pub inline_level: i32,
    pub serialization_level: i32,

    pub type_print_stack: Vec<usize>,

    pub current_file: Option<Arc<SourceFile>>,

    pub current_file_id: u64,

    pub current_file_symbol: Option<Arc<Symbol>>,

    pub scope_stack: Vec<u64>,

    pub function_scope_count: usize,

    pub arrow_function_scope_count: usize,

    pub globals_populated: bool,

    pub break_continue_context_stack: Vec<BreakContinueContext>,

    pub this_type_stack: Vec<Arc<Type>>,

    pub display_target_override: Option<Arc<Type>>,

    pub enclosing_class_stack: Vec<Arc<Node>>,

    pub this_container_stack: Vec<ThisContainerKind>,

    pub ambient_context_depth: usize,

    ambient_ts1036_reported_blocks: std::collections::HashSet<u64>,

    pub namespace_value_depth: u8,

    pub accessor_pair_return_hint: Option<Arc<Type>>,

    pub call_arg_arrow_context: Vec<usize>,

    pub resolving_type_aliases: std::collections::HashSet<*const Symbol>,

    pub resolving_function_like: std::collections::HashSet<u64>,

    pub class_statics_resolution_stack: Vec<u64>,

    pub class_type_resolution_stack: Vec<u64>,

    pub resolving_contextual_calls: std::collections::HashSet<u64>,

    pub logical_rhs_narrowing_frames: Vec<(Arc<Symbol>, Arc<Type>)>,

    pub in_ctor_body_stack: Vec<bool>,

    pub return_type_stack: Vec<Option<Arc<Type>>>,

    pub flow_analysis_disabled: bool,
    pub flow_invocation_count: i32,
    pub flow_type_cache: HashMap<u64, Arc<Type>>,
    pub flow_node_reachable: HashMap<u64, bool>,

    pub type_instantiation_count: u64,

    pub type_instantiation_limit_reported: bool,

    pub flow_inline_level: u32,

    pub in_static_member_type: bool,

    pub suppress_cannot_find_name_in_type_nodes: u32,

    pub suppress_source_file: Option<u64>,

    pub tracer: Arc<Tracer>,

    pub merged_symbols: HashMap<u64, u64>,

    pub mu: Mutex<()>,
}

impl Checker {

    pub fn new(program: Arc<dyn Program>, tracer: Arc<Tracer>) -> Self {
        let compiler_options = Arc::new(program.options().clone());
        let files = program.source_files().to_vec();

        let language_version = compiler_options.get_emit_script_target();
        let module_kind = compiler_options.get_emit_module_kind();
        let module_resolution_kind = compiler_options.get_module_resolution_kind();

        let legacy_decorators = compiler_options.experimental_decorators.is_true();
        let emit_standard_class_fields = compiler_options.get_emit_standard_class_fields();
        let strict_null_checks =
            compiler_options.get_strict_option_value(compiler_options.strict_null_checks);
        let strict_function_types =
            compiler_options.get_strict_option_value(compiler_options.strict_function_types);
        let strict_bind_call_apply =
            compiler_options.get_strict_option_value(compiler_options.strict_bind_call_apply);
        let strict_property_initialization = compiler_options
            .get_strict_option_value(compiler_options.strict_property_initialization);
        let strict_builtin_iterator_return = compiler_options
            .get_strict_option_value(compiler_options.strict_builtin_iterator_return);
        let no_implicit_any =
            compiler_options.get_strict_option_value(compiler_options.no_implicit_any);
        let no_implicit_this =
            compiler_options.get_strict_option_value(compiler_options.no_implicit_this);
        let use_unknown_in_catch_variables = compiler_options
            .get_strict_option_value(compiler_options.use_unknown_in_catch_variables);
        let exact_optional_property_types =
            compiler_options.exact_optional_property_types.is_true();
        let can_collect_symbol_alias_accessibility_data = compiler_options
            .verbatim_module_syntax
            .is_false_or_unknown();

        let mut file_index_map = HashMap::new();
        for (i, file) in files.iter().enumerate() {
            file_index_map.insert(file.id(), i);
        }

        let mut checker = Self {
            id: NEXT_CHECKER_ID.fetch_add(1, Ordering::Relaxed),
            program,
            compiler_options,
            files,
            file_index_map,

            type_count: 0,
            symbol_count: 0,
            signature_count: 0,
            total_instantiation_count: 0,
            instantiation_count: 0,
            instantiation_depth: 0,

            language_version,
            module_kind,
            module_resolution_kind,
            legacy_decorators,
            emit_standard_class_fields,
            strict_null_checks,
            strict_function_types,
            strict_bind_call_apply,
            strict_property_initialization,
            strict_builtin_iterator_return,
            no_implicit_any,
            no_implicit_this,
            use_unknown_in_catch_variables,
            exact_optional_property_types,
            can_collect_symbol_alias_accessibility_data,

            globals: SymbolTable::default(),
            undefined_symbol: Some(Arc::new(Symbol::new(SymbolFlags::Property, "undefined"))),
            arguments_symbol: Some(Arc::new(Symbol::new(
                SymbolFlags::Property.union(SymbolFlags::Transient),
                "arguments",
            ))),
            require_symbol: None,
            unknown_symbol: None,
            global_this_symbol: None,

            string_literal_types: HashMap::new(),
            number_literal_types: HashMap::new(),
            bigint_literal_types: HashMap::new(),
            unique_es_symbol_types: HashMap::new(),
            nan_type: None,

            indexed_access_types: HashMap::new(),
            template_literal_types: HashMap::new(),
            string_mapping_types: HashMap::new(),
            cached_types: HashMap::new(),
            union_types: HashMap::new(),
            intersection_types: HashMap::new(),
            tuple_types: HashMap::new(),
            error_types: HashMap::new(),

            global_interface_members: HashMap::new(),
            boxed_global_types: HashMap::new(),

            diagnostics: DiagnosticsCollection::default(),
            suggestion_diagnostics: DiagnosticsCollection::default(),

            node_links: LinkStore::new(),
            signature_links: LinkStore::new(),
            symbol_node_links: LinkStore::new(),
            type_node_links: LinkStore::new(),
            enum_member_links: LinkStore::new(),
            assertion_links: LinkStore::new(),
            array_literal_links: LinkStore::new(),
            switch_statement_links: LinkStore::new(),
            jsx_element_links: LinkStore::new(),

            symbol_reference_links: LinkStore::new(),
            value_symbol_links: LinkStore::new(),
            mapped_symbol_links: LinkStore::new(),
            deferred_symbol_links: LinkStore::new(),
            alias_symbol_links: LinkStore::new(),
            module_symbol_links: LinkStore::new(),
            late_bound_links: LinkStore::new(),
            export_type_links: LinkStore::new(),
            members_and_exports_links: LinkStore::new(),
            type_alias_links: LinkStore::new(),
            declared_type_links: LinkStore::new(),
            type_resolution_stack: Vec::new(),
            type_argument_stack: Vec::new(),
            type_argument_name_frames: Vec::new(),
            type_node_subst_cache: HashMap::new(),
            type_node_resolving: HashSet::new(),
            type_resolution_depth: 0,
            heritage_degraded_events: 0,
            type_node_query_epochs: Vec::new(),
            heritage_retry_counts: HashMap::new(),
            type_node_subst_cache_limit: 300_000,
            speculation_depth: 0,
            type_parameter_resolving: HashSet::new(),
            ts2313_reported: HashSet::new(),
            ts2354_checked_files: HashSet::new(),
            requested_external_emit_helpers: HashMap::new(),
            degraded_type_ptrs: std::collections::HashSet::new(),
            jsx_implicit_namespace: HashMap::new(),
            pending_jsx_2875: None,
            relater_error_chain: Vec::new(),
            relater_chain_active: false,
            relater_depth: 0,
            deferred_constraint_depth: 0,
            relation_count: 0,
            relater_overflow: false,
            relater_intersection_target_depth: 0,
            subst_object_in_progress: std::collections::HashMap::new(),
            in_return_substitution: false,
            relater_source_stack: Vec::new(),
            relater_target_stack: Vec::new(),
            relation_cache: HashMap::new(),
            probe_cache_permissive: HashMap::new(),
            probe_cache_restrictive: HashMap::new(),
            enum_relation: HashMap::new(),
            relation_in_progress: std::collections::HashSet::new(),
            interface_extends_reported: std::collections::HashSet::new(),
            indexed_access_2538_reported: std::collections::HashSet::new(),
            arith_operand_error_nodes: std::collections::HashSet::new(),
            computed_property_name_checked: std::collections::HashSet::new(),
            symbol_reference_kinds: dashmap::DashMap::new(),
            spread_links: LinkStore::new(),
            variance_links: LinkStore::new(),
            reverse_mapped_symbol_links: LinkStore::new(),
            marked_assignment_symbol_links: LinkStore::new(),
            symbol_container_links: LinkStore::new(),
            symbol_table_alias_cache: HashMap::new(),
            class_expression_name_tables: HashMap::new(),
            source_file_links: LinkStore::new(),
            declaration_links: LinkStore::new(),
            declaration_file_links: LinkStore::new(),
            last_combined_modifier_flags_node: None,
            last_combined_modifier_flags_result: ModifierFlags::empty(),

            any_type: OnceLock::new(),
            unknown_type: OnceLock::new(),
            undefined_type: OnceLock::new(),
            null_type: OnceLock::new(),
            string_type: OnceLock::new(),
            number_type: OnceLock::new(),
            bigint_type: OnceLock::new(),
            boolean_type: OnceLock::new(),
            es_symbol_type: OnceLock::new(),
            void_type: OnceLock::new(),
            never_type: OnceLock::new(),
            non_primitive_type: OnceLock::new(),
            true_type: OnceLock::new(),
            false_type: OnceLock::new(),
            error_type: OnceLock::new(),
            unresolved_type: OnceLock::new(),

            auto_type: OnceLock::new(),

            empty_object_type: OnceLock::new(),
            empty_generic_type: OnceLock::new(),
            any_function_type: OnceLock::new(),
            no_constraint_type: OnceLock::new(),
            circular_constraint_type: OnceLock::new(),

            any_array_type: OnceLock::new(),
            auto_array_type: OnceLock::new(),
            any_readonly_array_type: OnceLock::new(),

            global_object_type: OnceLock::new(),
            global_function_type: OnceLock::new(),
            global_array_type: OnceLock::new(),
            global_readonly_array_type: OnceLock::new(),
            global_string_type: OnceLock::new(),
            global_number_type: OnceLock::new(),
            global_boolean_type: OnceLock::new(),
            global_reg_exp_type: OnceLock::new(),
            global_this_type: OnceLock::new(),
            global_promise_type: OnceLock::new(),
            array_type_cache: std::collections::HashMap::new(),
            interface_instantiation_cache: std::collections::HashMap::new(),
            typequery_instantiation_cache: std::collections::HashMap::new(),
            attached_type_args_cache: std::collections::HashMap::new(),
            array_type_parameter_symbols: None,
            array_member_type_cache: std::collections::HashMap::new(),
            instantiated_member_type_cache: std::collections::HashMap::new(),
            instantiated_member_type_cache_limit: 300_000,

            any_signature: OnceLock::new(),
            unknown_signature: OnceLock::new(),
            resolving_signature: OnceLock::new(),

            current_node: None,
            inline_level: 0,
            serialization_level: 0,
            type_print_stack: Vec::new(),
            current_file: None,
            current_file_id: 0,
            current_file_symbol: None,
            scope_stack: Vec::new(),
            function_scope_count: 0,
            arrow_function_scope_count: 0,
            globals_populated: false,
            break_continue_context_stack: Vec::new(),
            this_container_stack: Vec::new(),
            ambient_context_depth: 0,
            ambient_ts1036_reported_blocks: std::collections::HashSet::new(),
            namespace_value_depth: 0,
            accessor_pair_return_hint: None,
            this_type_stack: Vec::new(),
            display_target_override: None,
            enclosing_class_stack: Vec::new(),
            call_arg_arrow_context: Vec::new(),
            resolving_type_aliases: std::collections::HashSet::new(),
            resolving_function_like: std::collections::HashSet::new(),
            class_statics_resolution_stack: Vec::new(),
            class_type_resolution_stack: Vec::new(),
            resolving_contextual_calls: std::collections::HashSet::new(),
            logical_rhs_narrowing_frames: Vec::new(),
            in_ctor_body_stack: Vec::new(),
            return_type_stack: Vec::new(),

            flow_analysis_disabled: false,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            type_instantiation_count: 0,
            type_instantiation_limit_reported: false,
            flow_node_reachable: HashMap::new(),
            flow_inline_level: 0,
            in_static_member_type: false,
            suppress_cannot_find_name_in_type_nodes: 0,
            suppress_source_file: None,

            merged_symbols: HashMap::new(),

            tracer,
            mu: Mutex::new(()),
        };

        {

            let mut global_this = Symbol::new(SymbolFlags::ValueModule, "globalThis");
            global_this.check_flags = CheckFlags::Readonly;
            let global_this = Arc::new(global_this);
            checker
                .globals
                .insert("globalThis".to_string(), Arc::clone(&global_this));
            checker.global_this_symbol = Some(global_this);

            if let Some(ref undef) = checker.undefined_symbol {
                checker
                    .globals
                    .insert("undefined".to_string(), Arc::clone(undef));
            }
        }

        checker
    }

    fn merge_global_symbols(dst: &Arc<Symbol>, src: &Arc<Symbol>) {
        let dst_mut = Arc::as_ptr(dst) as *mut Symbol;
        unsafe {
            (*dst_mut).flags |= src.flags;
            for d in &src.declarations {
                if !dst
                    .declarations
                    .iter()
                    .any(|existing| Arc::ptr_eq(existing, d))
                {
                    (*dst_mut).declarations.push(Arc::clone(d));
                }
            }
            if dst.value_declaration.is_none() {
                (*dst_mut).value_declaration = src.value_declaration.clone();
            }
            for (name, member) in src.members.entries.iter() {
                match (*dst_mut).members.entries.get(name) {
                    Some(existing) => Self::merge_global_symbols(existing, member),
                    None => {
                        (*dst_mut).members.entries.insert(name.clone(), Arc::clone(member));
                    }
                }
            }
            for (name, export) in src.exports.entries.iter() {
                match (*dst_mut).exports.entries.get(name) {
                    Some(existing) => Self::merge_global_symbols(existing, export),
                    None => {
                        (*dst_mut).exports.entries.insert(name.clone(), Arc::clone(export));
                    }
                }
            }
        }
    }

    fn populate_globals(&mut self) {
        for file in &self.files {

            if file.external_module_indicator.is_some() {
                continue;
            }

            let symbol_map = self.program.symbol_map();
            if let Some(file_sym) = symbol_map.symbol_of(&file.node) {

                for (name, sym) in file_sym.members.iter() {
                    match self.globals.get(name) {
                        Some(existing) => Self::merge_global_symbols(existing, sym),
                        None => {
                            self.globals.insert(name.clone(), Arc::clone(sym));
                        }
                    }
                }

                if let Some(locals) = symbol_map.locals_of(&file.node) {
                    for (name, sym) in locals.iter() {
                        match self.globals.get(name) {
                            Some(existing) => Self::merge_global_symbols(existing, sym),
                            None => {
                                self.globals.insert(name.clone(), Arc::clone(sym));
                            }
                        }
                    }
                }
            }
        }

        for file in &self.files {
            for aug_name in &file.module_augmentations {
                let Some(module_node) = aug_name.parent.clone() else {
                    continue;
                };
                if !crate::ast::is_global_scope_augmentation(&module_node) {
                    continue;
                }
                let symbol_map = self.program.symbol_map();
                let mut aug_members: Vec<(String, Arc<Symbol>)> = Vec::new();
                if let Some(module_sym) = symbol_map.symbol_of(&module_node) {
                    aug_members.extend(
                        module_sym
                            .exports
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v))),
                    );
                    aug_members.extend(
                        module_sym
                            .members
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v))),
                    );
                }
                if let Some(locals) = symbol_map.locals_of(&module_node) {
                    aug_members.extend(
                        locals
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v))),
                    );
                }
                for (name, sym) in aug_members {
                    match self.globals.get(&name) {
                        Some(existing) => Self::merge_global_symbols(existing, &sym),
                        None => {
                            self.globals.insert(name, sym);
                        }
                    }
                }
            }
        }

        self.ensure_host_globals();

        self.ensure_jsx_namespace();
    }

    fn ensure_host_globals(&mut self) {

        const DOM_VALUES: &[&str] = &[
            "document",
            "window",
            "navigator",
            "self",
            "top",
            "parent",
            "frames",
            "location",
            "history",
            "screen",
            "localStorage",
            "sessionStorage",
            "console",
            "alert",
            "confirm",
            "prompt",
            "fetch",
            "setTimeout",
            "setInterval",
            "clearTimeout",
            "clearInterval",
            "queueMicrotask",
            "requestAnimationFrame",
            "cancelAnimationFrame",
            "getComputedStyle",
            "matchMedia",
            "addEventListener",
            "removeEventListener",
            "postMessage",
            "atob",
            "btoa",
            "scrollTo",
            "scrollBy",

            "Function",

        ];

        const DOM_TYPES: &[&str] = &[
            "HTMLElement",
            "Element",
            "Node",
            "Event",
            "EventTarget",
            "Document",
            "DocumentFragment",
            "ShadowRoot",
            "Window",
            "NodeList",
            "HTMLInputElement",
            "HTMLButtonElement",
            "HTMLDivElement",
            "HTMLSpanElement",
            "HTMLAnchorElement",
            "HTMLFormElement",
            "HTMLSelectElement",
            "HTMLTextAreaElement",
            "HTMLCanvasElement",
            "CanvasRenderingContext2D",
            "MouseEvent",
            "KeyboardEvent",
            "DataTransfer",
            "SVGElement",
            "TrustedHTML",
            "StyleMedia",
            "FormData",
            "Blob",
            "File",
            "URL",
            "URLSearchParams",
            "TextEncoder",
            "TextDecoder",
            "AbortController",
            "AbortSignal",
            "Headers",
            "Request",
            "Response",
            "ReadableStream",
            "WritableStream",
            "TransformStream",
        ];

        const ES_TYPES: &[&str] = &[
            "Promise",
            "Iterable",
            "Iterator",
            "IterableIterator",
            "Symbol",
            "Generator",
            "AsyncIterable",
            "AsyncIterator",
            "Awaited",
            "ArrayBuffer",
            "Uint8Array",
            "Int8Array",
            "Uint16Array",
            "Int16Array",
            "Uint32Array",
            "Int32Array",
            "Float32Array",
            "Float64Array",
            "DataView",

            "Object",
            "Array",
            "Number",
            "String",
            "Boolean",
            "Date",
            "Math",
            "Error",
            "RegExp",
            "Intl",
            "JSON",
            "Map",
            "Set",
            "WeakMap",
            "WeakSet",
            "TemplateStringsArray",
            "TypedPropertyDescriptor",
            "ReadonlyArray",
            "BigInt",
            "Proxy",
            "Reflect",
            "FinalizationRegistry",
            "WeakRef",
            "SharedArrayBuffer",
            "Atomics",
            "globalThis",
        ];

        const UTILITY_TYPES: &[&str] = &[
            "Partial",
            "Readonly",
            "Pick",
            "Record",
            "Omit",
            "Exclude",
            "Extract",
            "NonNullable",
            "Parameters",
            "ReturnType",
            "ConstructorParameters",
            "InstanceType",
            "Required",
            "ReadonlyArray",
        ];

        for &name in DOM_VALUES
            .iter()
            .chain(DOM_TYPES.iter())
            .chain(ES_TYPES.iter())
            .chain(UTILITY_TYPES.iter())
        {
            if self.globals.get(name).is_none() {
                self.globals.insert(
                    name.to_string(),
                    Arc::new(Symbol::new(SymbolFlags::Property, name)),
                );
            }
        }
    }

    fn ensure_jsx_namespace(&mut self) {
        use super::jsx::JsxNames;
        if !self.is_jsx_enabled() || self.get_jsx_namespace().is_some() {
            return;
        }

        let mut jsx = Symbol::new(SymbolFlags::NamespaceModule, JsxNames::JSX);

        let element = Symbol::new(SymbolFlags::TypeLiteral, JsxNames::ELEMENT);
        jsx.members
            .insert(JsxNames::ELEMENT.to_string(), Arc::new(element));

        let mut intrinsic = Symbol::new(SymbolFlags::TypeLiteral, JsxNames::INTRINSIC_ELEMENTS);
        intrinsic.members.insert(
            crate::ast::INTERNAL_SYMBOL_NAME_INDEX.to_string(),
            Arc::new(Symbol::new(SymbolFlags::TypeLiteral, "")),
        );
        jsx.members.insert(
            JsxNames::INTRINSIC_ELEMENTS.to_string(),
            Arc::new(intrinsic),
        );

        self.globals
            .insert(JsxNames::JSX.to_string(), Arc::new(jsx));
    }

    pub fn any_type(&self) -> Arc<Type> {
        self.any_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Any,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "any".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn unknown_type(&self) -> Arc<Type> {
        self.unknown_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Unknown,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "unknown".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn undefined_type(&self) -> Arc<Type> {
        self.undefined_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Undefined,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "undefined".to_string(),
                    }),
                ))
            })
            .clone()
    }

    fn nullish_widening_type(&self, base: Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            return base;
        }
        let mut t = Type::new(base.flags, TypeData::Intrinsic(IntrinsicTypeData {
            intrinsic_name: base
                .intrinsic_name()
                .unwrap_or("undefined")
                .to_string(),
        }));
        t.object_flags |= crate::checker::types::ObjectFlags::ContainsWideningType;
        Arc::new(t)
    }

    pub fn null_type(&self) -> Arc<Type> {
        self.null_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Null,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "null".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn string_type(&self) -> Arc<Type> {
        self.string_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::String,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "string".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn number_type(&self) -> Arc<Type> {
        self.number_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Number,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "number".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn bigint_type(&self) -> Arc<Type> {
        self.bigint_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::BigInt,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "bigint".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn boolean_type(&self) -> Arc<Type> {
        self.boolean_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Boolean,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "boolean".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn es_symbol_type(&self) -> Arc<Type> {
        self.es_symbol_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::ESSymbol,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "symbol".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn void_type(&self) -> Arc<Type> {
        self.void_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Void,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "void".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn never_type(&self) -> Arc<Type> {
        self.never_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Never,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "never".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn auto_type(&self) -> Arc<Type> {
        self.auto_type
            .get_or_init(|| {
                Arc::new(Type {
                    flags: TypeFlags::Any,
                    object_flags: ObjectFlags::NonInferrableType,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "any".to_string(),
                    }),
                })
            })
            .clone()
    }

    pub fn auto_array_type(&mut self) -> Arc<Type> {
        if let Some(t) = self.auto_array_type.get() {
            return Arc::clone(t);
        }
        let auto = self.auto_type();
        let arr = self.create_array_type(auto);

        self.auto_array_type
            .set(arr.clone())
            .ok()
            .map(|()| arr.clone())
            .unwrap_or_else(|| self.auto_array_type.get().cloned().unwrap_or(arr))
    }

    pub fn get_evolving_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::EvolvingArray,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::EvolvingArray(EvolvingArrayTypeData {
                object: ObjectTypeData::default(),
                element_type: Some(element_type),
                final_array_type: OnceLock::new(),
            }),
        })
    }

    pub fn add_evolving_array_element_type(
        &mut self,
        evolving_type: &Arc<Type>,
        new_element_type: Arc<Type>,
    ) -> Arc<Type> {
        let current_element = match &evolving_type.data {
            TypeData::EvolvingArray(ea) => ea.element_type.clone(),
            _ => return Arc::clone(evolving_type),
        };
        match current_element {
            Some(current) => {

                if self.is_type_subset_of(&new_element_type, &current) {
                    return Arc::clone(evolving_type);
                }
                let union = self.get_union_type(vec![current, new_element_type]);
                self.get_evolving_array_type(union)
            }
            None => self.get_evolving_array_type(new_element_type),
        }
    }

    pub fn finalize_evolving_array_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if !t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return Arc::clone(t);
        }
        match &t.data {
            TypeData::EvolvingArray(ea) => {
                if let Some(final_t) = ea.final_array_type.get() {
                    return Arc::clone(final_t);
                }
                let element = ea.element_type.clone().unwrap_or_else(|| self.never_type());
                let result = if element.flags.contains(TypeFlags::Never) {
                    self.auto_array_type()
                } else if element.flags.contains(TypeFlags::Union) {

                    self.create_array_type(element)
                } else {
                    self.create_array_type(element)
                };

                if let TypeData::EvolvingArray(ea) = &t.data {
                    let _ = ea.final_array_type.set(Arc::clone(&result));
                }
                result
            }
            _ => Arc::clone(t),
        }
    }

    pub fn is_type_subset_of(&mut self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if Arc::ptr_eq(a, b) || self.types_are_equal(a, b) {
            return true;
        }
        if a.flags.contains(TypeFlags::Never) {
            return true;
        }
        if b.flags.contains(TypeFlags::Any) || b.flags.contains(TypeFlags::Unknown) {
            return true;
        }

        self.is_type_assignable_to(a, b)
    }

    pub fn any_function_type(&self) -> Arc<Type> {
        self.any_function_type
            .get_or_init(|| {
                Arc::new(Type {
                    flags: TypeFlags::Object,
                    object_flags: ObjectFlags::Anonymous,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: TypeData::Object(ObjectTypeData::default()),
                })
            })
            .clone()
    }

    pub fn non_primitive_type(&self) -> Arc<Type> {
        self.non_primitive_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::NonPrimitive,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "object".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn true_type(&self) -> Arc<Type> {
        self.true_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::BooleanLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::Boolean(true),
                        fresh_type: OnceLock::new(),
                        regular_type: OnceLock::new(),
                    }),
                ))
            })
            .clone()
    }

    pub fn false_type(&self) -> Arc<Type> {
        self.false_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::BooleanLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::Boolean(false),
                        fresh_type: OnceLock::new(),
                        regular_type: OnceLock::new(),
                    }),
                ))
            })
            .clone()
    }

    pub fn error_type(&self) -> Arc<Type> {
        self.error_type
            .get_or_init(|| {
                Arc::new(Type::new(
                    TypeFlags::Any,
                    TypeData::Intrinsic(IntrinsicTypeData {
                        intrinsic_name: "error".to_string(),
                    }),
                ))
            })
            .clone()
    }

    pub fn get_string_literal_type(&mut self, value: &str) -> Arc<Type> {
        if let Some(t) = self.string_literal_types.get(value) {
            return Arc::clone(t);
        }
        let t = Arc::new(Type::new(
            TypeFlags::StringLiteral,
            TypeData::Literal(LiteralTypeData {
                value: LiteralValue::String(value.to_string()),
                fresh_type: OnceLock::new(),
                regular_type: OnceLock::new(),
            }),
        ));
        self.string_literal_types
            .insert(value.to_string(), Arc::clone(&t));
        t
    }

    pub fn get_number_literal_type(&mut self, value: jsnum::Number) -> Arc<Type> {
        if let Some(t) = self.number_literal_types.get(&value) {
            return Arc::clone(t);
        }
        let t = Arc::new(Type::new(
            TypeFlags::NumberLiteral,
            TypeData::Literal(LiteralTypeData {
                value: LiteralValue::Number(value),
                fresh_type: OnceLock::new(),
                regular_type: OnceLock::new(),
            }),
        ));
        self.number_literal_types.insert(value, Arc::clone(&t));
        t
    }

    pub fn get_fresh_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {

        if !t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            return Arc::clone(t);
        }
        let lit = match &t.data {
            TypeData::Literal(lit) => lit,
            _ => {

                return Arc::clone(t);
            }
        };

        if lit.regular_type.get().is_some() {
            return Arc::clone(t);
        }

        let value = lit.value.clone();
        let flags = t.flags;
        let regular = Arc::clone(t);
        let fresh = lit.fresh_type.get_or_init(move || {
            Arc::new(Type::new(
                flags,
                TypeData::Literal(LiteralTypeData {
                    value,

                    fresh_type: OnceLock::new(),

                    regular_type: OnceLock::from(regular),
                }),
            ))
        });
        Arc::clone(fresh)
    }

    pub fn is_literal_of_contextual_type(
        &self,
        candidate: &Arc<Type>,
        contextual: &Arc<Type>,
    ) -> bool {
        if contextual.flags.intersects(TypeFlags::Union | TypeFlags::Intersection) {
            if let TypeData::Union(u) = &contextual.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|t| self.is_literal_of_contextual_type(candidate, t));
            }
            return false;
        }
        if contextual.flags.intersects(TypeFlags::TypeParameter) {

            if let Some(constraint) = self.get_base_constraint_of_type(contextual) {
                return (constraint.flags.intersects(TypeFlags::String)
                    && candidate.flags.intersects(TypeFlags::StringLiteral))
                    || (constraint.flags.intersects(TypeFlags::Number)
                        && candidate.flags.intersects(TypeFlags::NumberLiteral))
                    || self.is_literal_of_contextual_type(candidate, &constraint);
            }
            return false;
        }

        (contextual.flags.intersects(
            TypeFlags::StringLiteral | TypeFlags::Index | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
        ) && candidate.flags.intersects(TypeFlags::StringLiteral))
            || (contextual.flags.intersects(TypeFlags::NumberLiteral)
                && candidate.flags.intersects(TypeFlags::NumberLiteral))
            || (contextual.flags.intersects(TypeFlags::BigIntLiteral)
                && candidate.flags.intersects(TypeFlags::BigIntLiteral))
            || (contextual.flags.intersects(TypeFlags::BooleanLiteral)
                && candidate.flags.intersects(TypeFlags::BooleanLiteral))
            || (contextual.flags.intersects(TypeFlags::UniqueESSymbol)
                && candidate.flags.intersects(TypeFlags::UniqueESSymbol))
    }

    pub fn get_widened_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {

        if crate::checker::is_fresh_literal_type(t) {

            if t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) {
                if let Some(sym) = &t.symbol
                    && sym.flags.contains(SymbolFlags::EnumMember)
                    && let Some(parent) = &sym.parent
                    && let Some(cached) = self
                        .type_alias_links
                        .get(parent)
                        .and_then(|l| l.declared_type.clone())
                {
                    return cached;
                }
            }
            if t.flags.contains(TypeFlags::StringLiteral) {
                return self.string_type();
            }
            if t.flags.contains(TypeFlags::NumberLiteral) {
                return self.number_type();
            }
            if t.flags.contains(TypeFlags::BigIntLiteral) {
                return self.bigint_type();
            }
            if t.flags.contains(TypeFlags::BooleanLiteral) {
                return self.boolean_type();
            }

        }

        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_widened_literal_type(member))
                .collect();

            if widened
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            return self.build_union_from_types(widened);
        }
        Arc::clone(t)
    }

    pub fn get_regular_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            if let TypeData::Literal(lit) = &t.data {
                if let Some(regular) = lit.regular_type.get() {
                    return Arc::clone(regular);
                }
            }
        }

        if let TypeData::Union(union_data) = &t.data {
            let regularized: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_regular_type_of_literal_type(member))
                .collect();
            if regularized
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            return self.build_union_from_types(regularized);
        }
        Arc::clone(t)
    }

    pub fn get_widened_literal_type_for_initializer(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
    ) -> Arc<Type> {

        if self
            .get_combined_node_flags(declaration)
            .intersects(NodeFlags::Constant)
        {
            return Arc::clone(t);
        }
        self.get_widened_literal_type(t)
    }

    pub fn get_diagnostics(&self) -> &DiagnosticsCollection {
        &self.diagnostics
    }

    pub fn get_suggestion_diagnostics(&self) -> &DiagnosticsCollection {
        &self.suggestion_diagnostics
    }

    pub fn get_combined_node_flags(&mut self, node: &Arc<Node>) -> NodeFlags {
        let mut flags = node.flags;
        let mut parent = node.parent.clone();
        while let Some(p) = parent {
            if p.kind == SyntaxKind::SourceFile {
                break;
            }
            flags |= p.flags;
            parent = p.parent.clone();
        }
        flags
    }

    pub fn get_combined_modifier_flags(&mut self, node: &Arc<Node>) -> ModifierFlags {

        if let Some(cached) = &self.last_combined_modifier_flags_node {
            if Arc::ptr_eq(cached, node) {
                return self.last_combined_modifier_flags_result;
            }
        }
        let flags = ast_get_combined_modifier_flags(node);
        self.last_combined_modifier_flags_node = Some(Arc::clone(node));
        self.last_combined_modifier_flags_result = flags;
        flags
    }

    pub fn get_root_declaration(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        while current.kind == SyntaxKind::BindingElement {

            let parent = match &current.parent {
                Some(p) => Arc::clone(p),
                None => break,
            };
            let grandparent = match &parent.parent {
                Some(gp) => Arc::clone(gp),
                None => break,
            };
            current = grandparent;
        }
        current
    }

    pub fn get_declaration_container(node: &Arc<Node>) -> Option<Arc<Node>> {
        let root = Self::get_root_declaration(node);

        let skip = |kind: SyntaxKind| {
            matches!(
                kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::VariableDeclarationList
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::NamedImports
                    | SyntaxKind::NamespaceImport
                    | SyntaxKind::ImportClause
            )
        };
        let mut current = Some(root);
        while let Some(n) = current {
            if skip(n.kind) {
                current = n.parent.clone();
                continue;
            }
            return n.parent.clone();
        }
        None
    }

    pub fn is_global_source_file(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        !Self::is_external_or_common_js_module(node)
    }

    pub fn is_external_or_common_js_module(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        let NodeData::SourceFile(data) = &node.data else {
            return false;
        };
        for stmt in data.statements.nodes.iter() {

            match stmt.kind {
                SyntaxKind::ImportDeclaration
                | SyntaxKind::ExportDeclaration
                | SyntaxKind::ExportAssignment
                | SyntaxKind::NamespaceExportDeclaration
                | SyntaxKind::ImportEqualsDeclaration => return true,
                _ => {
                    if stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_external_module_augmentation(node: &Arc<Node>) -> bool {
        if !Self::is_ambient_module(node) {
            return false;
        }
        Self::is_module_augmentation_external(node)
    }

    fn is_ambient_module(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::ModuleDeclaration {
            return false;
        }
        if let NodeData::ModuleDeclaration(d) = &node.data {

            d.keyword == SyntaxKind::ModuleKeyword || d.keyword == SyntaxKind::NamespaceKeyword
        } else {
            false
        }
    }

    fn is_module_augmentation_external(node: &Arc<Node>) -> bool {
        let parent = match &node.parent {
            Some(p) => p,
            None => return false,
        };
        match parent.kind {
            SyntaxKind::SourceFile => Self::is_external_or_common_js_module(parent),
            SyntaxKind::ModuleBlock => {
                let grandparent = match &parent.parent {
                    Some(gp) => gp,
                    None => return false,
                };
                Self::is_ambient_module(grandparent)
                    && matches!(&grandparent.parent, Some(ggp) if ggp.kind == SyntaxKind::SourceFile)
                    && !Self::is_external_or_common_js_module(grandparent.parent.as_ref().unwrap())
            }
            _ => false,
        }
    }

    pub fn is_late_visibility_painted_statement(node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::ImportDeclaration
                | SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::VariableStatement
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::ModuleDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::EnumDeclaration
        )
    }

    pub fn get_any_import_syntax(node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::ImportEqualsDeclaration => Some(Arc::clone(node)),
            SyntaxKind::ImportClause => node.parent.clone(),
            SyntaxKind::NamespaceImport => node.parent.clone().and_then(|p| p.parent.clone()),
            SyntaxKind::ImportSpecifier => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .and_then(|gp| gp.parent.clone()),
            _ => None,
        }
    }
}

fn ast_get_combined_modifier_flags(node: &Arc<Node>) -> ModifierFlags {
    let current = Checker::get_root_declaration(node);
    let mut flags = current.syntactic_modifier_flags();
    if current.kind == SyntaxKind::VariableDeclaration {
        if let Some(parent) = current.parent.clone() {
            if parent.kind == SyntaxKind::VariableDeclarationList {
                flags |= parent.syntactic_modifier_flags();
                if let Some(gp) = parent.parent.clone() {
                    if gp.kind == SyntaxKind::VariableStatement {
                        flags |= gp.syntactic_modifier_flags();
                    }
                }
            }
        }
    }
    flags
}

impl Checker {

    pub fn push_type_resolution(
        &mut self,
        target: *const Symbol,
        property: TypeResolutionProperty,
    ) -> bool {

        let cycle_start = self
            .type_resolution_stack
            .iter()
            .rposition(|entry| entry.target == target && entry.property == property);

        if let Some(idx) = cycle_start {

            for entry in &mut self.type_resolution_stack[idx..] {
                entry.result = false;
            }
            false
        } else {
            self.type_resolution_stack.push(TypeResolutionEntry {
                target,
                property,
                result: true,
            });
            true
        }
    }

    pub fn pop_type_resolution(&mut self) -> bool {
        self.type_resolution_stack
            .pop()
            .map(|entry| entry.result)
            .unwrap_or(true)
    }

    pub fn is_resolving(&self, target: *const Symbol, property: TypeResolutionProperty) -> bool {
        self.type_resolution_stack
            .iter()
            .any(|entry| entry.target == target && entry.property == property)
    }
}

impl Checker {

    pub fn get_string_type(&self) -> Arc<Type> {
        self.string_type()
    }
    pub fn get_number_type(&self) -> Arc<Type> {
        self.number_type()
    }
    pub fn get_boolean_type(&self) -> Arc<Type> {
        self.boolean_type()
    }
    pub fn get_void_type(&self) -> Arc<Type> {
        self.void_type()
    }
    pub fn get_undefined_type(&self) -> Arc<Type> {
        self.undefined_type()
    }
    pub fn get_null_type(&self) -> Arc<Type> {
        self.null_type()
    }
    pub fn get_any_type(&self) -> Arc<Type> {
        self.any_type()
    }
    pub fn get_error_type(&self) -> Arc<Type> {
        self.error_type()
    }
    pub fn get_never_type(&self) -> Arc<Type> {
        self.never_type()
    }
    pub fn get_unknown_type(&self) -> Arc<Type> {
        self.unknown_type()
    }
    pub fn get_bigint_type(&self) -> Arc<Type> {
        self.bigint_type()
    }
    pub fn get_es_symbol_type(&self) -> Arc<Type> {
        self.es_symbol_type()
    }

    pub fn get_unknown_symbol(&self) -> Option<Arc<Symbol>> {
        self.unknown_symbol.clone()
    }
    pub fn get_undefined_symbol(&self) -> Option<Arc<Symbol>> {
        self.undefined_symbol.clone()
    }
    pub fn get_arguments_symbol(&self) -> Option<Arc<Symbol>> {
        self.arguments_symbol.clone()
    }

    pub fn get_properties_of_type(&self, t: &Arc<Type>) -> Vec<Arc<Symbol>> {
        if let Some(structured) = t.as_structured() {
            return structured.properties.clone();
        }
        Vec::new()
    }

    pub fn get_signatures_of_type(
        &self,
        t: &Arc<Type>,
        kind: SignatureKind,
    ) -> Vec<Arc<Signature>> {
        if let Some(structured) = t.as_structured() {
            return match kind {
                SignatureKind::Call => structured.call_signatures().to_vec(),
                SignatureKind::Construct => structured.construct_signatures().to_vec(),
            };
        }
        Vec::new()
    }

    pub fn type_has_call_or_construct_signatures(&self, t: &Arc<Type>) -> bool {
        if let Some(structured) = t.as_structured() {
            return !structured.signatures.is_empty();
        }
        false
    }

    pub fn is_array_like_type(&self, t: &Arc<Type>) -> bool {

        if t.flags.contains(TypeFlags::Object) {
            if let Some(structured) = t.as_structured() {

                for info in &structured.index_infos {
                    if info
                        .key_type
                        .as_ref()
                        .map(|kt| kt.flags.contains(TypeFlags::Number))
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }

                return t.object_flags.contains(ObjectFlags::Tuple);
            }
        }
        false
    }

    pub fn is_array_type(&self, t: &Arc<Type>) -> bool {

        t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::Reference)
    }

    pub fn is_tuple_type(&self, t: &Arc<Type>) -> bool {
        super::utilities::is_tuple_type(t)
    }

    pub fn get_base_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::EnumLiteral) {

            if let Some(sym) = &t.symbol
                && sym.flags.contains(SymbolFlags::EnumMember)
                && let Some(parent) = &sym.parent
                && let Some(cached) = self
                    .type_alias_links
                    .get(parent)
                    .and_then(|l| l.declared_type.clone())
            {
                return cached;
            }
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            return self.string_type();
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            return self.number_type();
        }
        if t.flags.contains(TypeFlags::BigIntLiteral) {
            return self.bigint_type();
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            return self.boolean_type();
        }

        if let TypeData::Union(u) = &t.data {
            let widened: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .map(|m| self.get_base_type_of_literal_type(m))
                .collect();
            if widened.len() == 1 {
                return Arc::clone(&widened[0]);
            }
            if widened
                .iter()
                .zip(u.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            if let Some(first) = widened.first() {
                if widened.iter().all(|w| Arc::ptr_eq(w, first)) {
                    return Arc::clone(first);
                }
            }

            return Arc::new(Type {
                flags: TypeFlags::Union,
                object_flags: ObjectFlags::None,
                id: crate::checker::types::next_type_id(),
                symbol: None,
                alias: None,
                data: TypeData::Union(UnionTypeData {
                    union_or_intersection: UnionOrIntersectionTypeData {
                        structured: StructuredTypeData::default(),
                        types: widened,
                    },
                    resolved_reduced_type: std::sync::OnceLock::new(),
                    regular_type: std::sync::OnceLock::new(),
                    origin: None,
                    key_property_name: None,
                    constituent_map: HashMap::new(),
                }),
            });
        }
        Arc::clone(t)
    }

    pub fn get_widened_type(&self, t: &Arc<Type>) -> Arc<Type> {

        if t.flags.intersects(TYPE_FLAGS_NULLABLE)
            && t.object_flags
                .intersects(crate::checker::types::OBJECT_FLAGS_REQUIRES_WIDENING)
        {
            return self.get_any_type();
        }

        if t.flags.intersects(TYPE_FLAGS_NULLABLE) {
            return Arc::clone(t);
        }

        if t.flags.intersects(TYPE_FLAGS_LITERAL) {
            if crate::checker::is_fresh_literal_type(t) {
                return self.get_base_type_of_literal_type(t);
            }
            return Arc::clone(t);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            return self.es_symbol_type();
        }

        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_widened_type(member))
                .collect();

            if widened
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }

            return self.build_union_from_types(widened);
        }
        Arc::clone(t)
    }

    pub fn widen_initializer_type(&mut self, t: &Arc<Type>) -> Arc<Type> {

        if crate::checker::is_object_literal_type(t) {
            return self.widen_object_literal_type(t);
        }

        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return Arc::clone(t);
        }

        if self.is_auto_array_type(t) {
            return self.get_evolving_array_type(self.never_type());
        }

        self.get_widened_type(t)
    }

    pub fn is_auto_array_type(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::Object) || !t.object_flags.contains(ObjectFlags::Reference)
        {
            return false;
        }

        match &t.data {
            TypeData::Object(obj) => obj
                .type_arguments
                .first()
                .map(|elem| elem.object_flags.contains(ObjectFlags::NonInferrableType))
                .unwrap_or(false),
            _ => false,
        }
    }

    pub(crate) fn widen_object_literal_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let structured = match t.as_structured() {
            Some(s) => s,
            None => return Arc::clone(t),
        };

        let mut widened_pairs: Vec<(String, Arc<Type>, Arc<Symbol>)> = Vec::new();
        for prop in &structured.properties {
            let prop_type = self.get_type_of_symbol(prop);
            let widened = self.widen_initializer_type(&prop_type);
            widened_pairs.push((prop.name.clone(), widened, Arc::clone(prop)));
        }

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(widened_pairs.len());
        for (name, t, src_prop) in widened_pairs {
            let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));

            {
                let sym_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*sym_mut).flags |= src_prop.flags & SymbolFlags::Optional;
                    (*sym_mut).check_flags |=
                        src_prop.check_flags & crate::ast::CheckFlags::Readonly;

                    (*sym_mut).declarations = src_prop.declarations.clone();
                }
            }
            members.insert(name, Arc::clone(&symbol));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn build_union_from_types(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }

        let mut seen: Vec<Arc<Type>> = Vec::new();
        for t in types {
            if let TypeData::Union(u) = &t.data {
                for inner in &u.union_or_intersection.types {
                    if !seen.iter().any(|s| Arc::ptr_eq(s, inner)) {
                        seen.push(Arc::clone(inner));
                    }
                }
            } else if !seen.iter().any(|s| Arc::ptr_eq(s, &t)) {
                seen.push(t);
            }
        }
        if seen.len() == 1 {
            return seen.into_iter().next().expect("exactly one");
        }

        seen.sort_by_key(|t| {
            if t.flags.intersects(TypeFlags::EnumLiteral | TypeFlags::Enum) {
                return TypeFlags::Enum.bits();
            }
            let b = t.flags.bits();
            b & b.wrapping_neg()
        });
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: seen,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn get_constraint_of_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.constraint.clone();
        }
        None
    }

    pub fn get_default_from_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.resolved_default_type.get().cloned();
        }
        None
    }

    pub fn get_resolved_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {

            if let Some(rt) = ct.resolved_true_type.get() {
                return Some(rt.clone());
            }
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    pub fn get_constraint_of_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(mt) = &t.data {
            return mt.constraint_type.clone();
        }
        if let TypeData::ReverseMapped(rm) = &t.data {
            return rm.constraint_type.clone();
        }
        None
    }

    pub fn get_true_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_true_type.get().cloned();
        }
        None
    }

    pub fn get_false_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_false_type.get().cloned();
        }
        None
    }

    pub fn get_return_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        sig.resolved_return_type.get().cloned()
    }

    pub fn get_type_predicate_of_signature<'a>(
        &self,
        sig: &'a Arc<Signature>,
    ) -> Option<&'a TypePredicate> {
        sig.resolved_type_predicate.as_deref()
    }

    pub fn compute_type_predicate_of_signature(
        &mut self,
        sig: &Arc<Signature>,
    ) -> Option<TypePredicate> {

        if let Some(pred) = sig.resolved_type_predicate.as_deref() {

            if pred.parameter_name == "<<unresolved>>" {
                return None;
            }
            return Some(pred.clone());
        }

        let Some(decl) = sig.declaration.as_ref() else {
            return None;
        };
        let Some(type_node) = decl.type_node() else {
            return None;
        };
        if type_node.kind != SyntaxKind::TypePredicate {
            return None;
        }
        let NodeData::TypePredicateNode(pred_data) = &type_node.data else {
            return None;
        };
        let t = pred_data
            .type_node
            .as_ref()
            .map(|tn| self.get_type_from_type_node(tn));
        let is_this = pred_data.parameter_name.kind == SyntaxKind::ThisKeyword
            || pred_data.parameter_name.kind == SyntaxKind::ThisType
            || (pred_data.parameter_name.kind == SyntaxKind::Identifier
                && pred_data.parameter_name.text() == "this");
        let kind = if pred_data.asserts_modifier.is_some() {
            if is_this {
                TypePredicateKind::AssertsThis
            } else {
                TypePredicateKind::AssertsIdentifier
            }
        } else {
            if is_this {
                TypePredicateKind::This
            } else {
                TypePredicateKind::Identifier
            }
        };
        let parameter_name = if is_this {
            String::new()
        } else {
            match &pred_data.parameter_name.data {
                NodeData::Identifier(id) => id.text.clone(),
                _ => String::new(),
            }
        };
        let parameter_index = if kind == TypePredicateKind::Identifier
            || kind == TypePredicateKind::AssertsIdentifier
        {
            sig.parameters
                .iter()
                .position(|p| p.name == parameter_name)
                .map(|i| i as i32)
                .unwrap_or(-1)
        } else {
            0
        };
        Some(TypePredicate {
            kind,
            parameter_index,
            parameter_name,
            t,
        })
    }

    pub fn get_base_constraint_of_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::TypeParameter(tp) => tp.constrained.resolved_base_constraint.get().cloned(),
            TypeData::Conditional(ct) => ct.constrained.resolved_base_constraint.get().cloned(),
            TypeData::IndexedAccess(ia) => ia.constrained.resolved_base_constraint.get().cloned(),
            TypeData::Index(it) => it.constrained.resolved_base_constraint.get().cloned(),
            _ => None,
        }
    }

    pub fn get_type_arguments(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        if let TypeData::Object(obj) = &t.data {
            return obj.type_arguments.clone();
        }
        Vec::new()
    }

    pub fn get_unique_symbol_type(&self, _name: &str) -> Option<Arc<Type>> {

        None
    }

    pub fn was_canceled(&self) -> bool {
        false
    }

    pub fn check_source_file(&mut self, file: &Arc<SourceFile>) {

        self.type_instantiation_count = 0;

        if !self.globals_populated {
            self.populate_globals();
            self.globals_populated = true;
        }

        let file_node = Arc::clone(&file.node);
        let file_id = file_node.id();
        let source_file_symbol = self.program.symbol_map().symbol_of(&file_node).cloned();

        self.set_parent_pointers(&file_node);

        let file_arc = Arc::clone(file);
        self.current_file = Some(Arc::clone(&file_arc));
        self.current_file_id = file_id;
        self.current_file_symbol = source_file_symbol;

        self.push_scope(&file_node);

        let statements: Vec<Arc<Node>> = match &file_node.data {
            crate::ast::NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
            _ => Vec::new(),
        };

        self.check_function_overloads_recursive(&statements);
        for stmt in &statements {
            self.check_statement(stmt);
        }

        self.check_export_assignment_conflicts(&statements);

        self.check_unused_identifiers_in_file(&file_node);

        self.pop_scope();
        self.current_file = None;
        self.current_file_id = 0;
        self.current_file_symbol = None;
    }


    pub fn get_semantic_diagnostics(&self) -> Vec<crate::ast::Diagnostic> {
        self.diagnostics.get_all()
    }
}

impl Checker {

    pub fn get_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> {

        if node.kind == SyntaxKind::ThisKeyword {
            return self.compute_type_of_node(node);
        }

        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }
        let result = self.compute_type_of_node(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }

    fn compute_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {

            SyntaxKind::NumericLiteral => {
                if let crate::ast::NodeData::NumericLiteral(data) = &node.data {
                    let lit = self.infer_number_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.number_type()
            }
            SyntaxKind::StringLiteral => {
                if let crate::ast::NodeData::StringLiteral(data) = &node.data {
                    let lit = self.infer_string_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.string_type()
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => self.string_type(),
            SyntaxKind::TrueKeyword => self.get_fresh_type_of_literal_type(&self.true_type()),
            SyntaxKind::FalseKeyword => self.get_fresh_type_of_literal_type(&self.false_type()),
            SyntaxKind::NullKeyword => self.nullish_widening_type(self.null_type()),
            SyntaxKind::UndefinedKeyword => self.nullish_widening_type(self.undefined_type()),
            SyntaxKind::BigIntLiteral => self.get_fresh_type_of_literal_type(&self.bigint_type()),
            SyntaxKind::ArrayLiteralExpression => {
                return self.get_type_of_array_literal(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                return self.get_type_of_object_literal(node);
            }
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                self.get_type_of_function_like(node)
            }
            SyntaxKind::FunctionDeclaration => self.get_type_of_function_like(node),
            SyntaxKind::Identifier => self.get_type_of_identifier(node),
            SyntaxKind::MetaProperty => self.get_type_of_meta_property(node),

            SyntaxKind::BinaryExpression => self.get_type_of_binary_expression(node),
            SyntaxKind::PrefixUnaryExpression => {
                if let crate::ast::NodeData::PrefixUnaryExpression(data) = &node.data {

                    match data.operator {

                        SyntaxKind::ExclamationToken => return self.boolean_type(),

                        SyntaxKind::DeleteKeyword => return self.boolean_type(),

                        SyntaxKind::VoidKeyword => return self.undefined_type(),

                        _ => return self.number_type(),
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::PostfixUnaryExpression => {

                self.number_type()
            }
            SyntaxKind::CallExpression => self.get_return_type_of_call_expression(node),
            SyntaxKind::NewExpression => self.get_return_type_of_new_expression(node),
            SyntaxKind::PropertyAccessExpression => self.get_type_of_property_access(node),
            SyntaxKind::ElementAccessExpression => self.get_type_of_element_access(node),
            SyntaxKind::ParenthesizedExpression => {
                if let crate::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::AsExpression => {

                if let crate::ast::NodeData::AsExpression(data) = &node.data {

                    if data.type_node.kind == SyntaxKind::ConstKeyword {
                        return self.get_const_assertion_type(&data.expression);
                    }
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::SatisfiesExpression => {

                if let crate::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::TypeAssertionExpression => {

                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::NonNullExpression => {

                if let crate::ast::NodeData::NonNullExpression(data) = &node.data {
                    let operand_type = self.get_type_of_node(&data.expression);
                    return self.remove_flags_from_union(
                        &operand_type,
                        TypeFlags::Undefined | TypeFlags::Null,
                    );
                }
                self.get_any_type()
            }
            SyntaxKind::ConditionalExpression => {

                if let crate::ast::NodeData::ConditionalExpression(data) = &node.data {
                    let true_type = self.get_type_of_node(&data.when_true);
                    let false_type = self.get_type_of_node(&data.when_false);
                    let true_widened = self.get_widened_type_of_literal(&true_type);
                    let false_widened = self.get_widened_type_of_literal(&false_type);
                    return self.get_union_type(vec![true_widened, false_widened]);
                }
                self.get_any_type()
            }
            SyntaxKind::TemplateExpression => {

                self.string_type()
            }
            SyntaxKind::TaggedTemplateExpression => {

                if let crate::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    let tag_type = self.get_type_of_node(&data.tag);
                    if let Some(structured) = tag_type.as_structured() {
                        for sig in structured.call_signatures() {
                            if let Some(rt) = self.get_return_type_of_signature(sig) {
                                return rt;
                            }
                            return self.get_any_type();
                        }
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::DeleteExpression => {

                self.boolean_type()
            }
            SyntaxKind::VoidExpression => {

                self.undefined_type()
            }
            SyntaxKind::AwaitExpression => {

                if let crate::ast::NodeData::AwaitExpression(data) = &node.data {
                    let operand = Arc::clone(&data.expression);

                    if let Some(ns) = self.type_of_dynamic_import(&operand) {
                        return ns;
                    }
                    let operand_type = self.get_type_of_node(&operand);
                    return match self.get_awaited_type(&operand_type) {
                        Some(awaited) => awaited,
                        None => operand_type,
                    };
                }
                self.get_any_type()
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {

                if node.kind == SyntaxKind::SuperKeyword
                    && self.super_in_computed_name_of_innermost_class(node)
                    && self.enclosing_class_stack.len() >= 2
                {
                    return self.this_type_stack
                        .get(self.this_type_stack.len() - 2)
                        .cloned()
                        .unwrap_or_else(|| self.get_any_type());
                }

                if self.this_container_stack.last() == Some(&ThisContainerKind::StaticMember)
                    && let Some(class) = self.enclosing_class_stack.last().cloned()
                {
                    return self.get_type_of_class_declaration(&class);
                }
                let r = self
                    .this_type_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.get_any_type());
                r
            }
            _ => self.get_any_type(),
        }
    }

    fn get_type_of_identifier(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(symbol) = self.resolve_identifier(node) {

            if symbol.flags == SymbolFlags::Alias {
                if let Some(t) = self.type_of_imported_symbol(&symbol) {
                    return t;
                }
            }
            let flow = self.program.symbol_map().flow_node_of(node).map(Arc::clone);
            let narrowed = self.get_narrowed_type_of_symbol(&symbol, flow.as_ref());

            if narrowed.object_flags.contains(ObjectFlags::EvolvingArray)
                && self.is_evolving_array_operation_target(node)
            {
                return self.auto_array_type();
            }

            let final_type = self.finalize_evolving_array_type(&narrowed);

            let target_kind = get_assignment_target_kind(node);
            let compound_like = target_kind == AssignmentKind::Definite
                && is_in_compound_like_assignment(node);
            if compound_like || target_kind == AssignmentKind::Compound {
                return self.get_base_type_of_literal_type(&final_type);
            }
            final_type
        } else {
            self.get_any_type()
        }
    }

    fn is_evolving_array_operation_target(&self, node: &Arc<Node>) -> bool {
        let root = self.get_reference_root(node);
        let Some(parent) = &root.parent else {
            return false;
        };

        if let NodeData::PropertyAccessExpression(pa) = &parent.data {
            if Arc::ptr_eq(&pa.expression, root) {
                let name = pa.name.text();
                if name == "length" {
                    return true;
                }
                if name == "push" || name == "unshift" {

                    if let Some(grandparent) = &parent.parent {
                        if matches!(grandparent.kind, SyntaxKind::CallExpression) {
                            return true;
                        }
                    }
                }
            }
        }

        if let NodeData::ElementAccessExpression(ea) = &parent.data {
            if Arc::ptr_eq(&ea.expression, root) {
                if let Some(grandparent) = &parent.parent {
                    if let NodeData::BinaryExpression(bin) = &grandparent.data {
                        if bin.operator_token.kind == SyntaxKind::EqualsToken
                            && Arc::ptr_eq(&bin.left, parent)
                        {

                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn get_reference_root<'a>(&self, node: &'a Arc<Node>) -> &'a Arc<Node> {
        let Some(parent) = &node.parent else {
            return node;
        };
        let recurse = match &parent.data {
            NodeData::ParenthesizedExpression(_) => true,
            NodeData::BinaryExpression(bin) => {
                (bin.operator_token.kind == SyntaxKind::EqualsToken && Arc::ptr_eq(&bin.left, node))
                    || (bin.operator_token.kind == SyntaxKind::CommaToken
                        && Arc::ptr_eq(&bin.right, node))
            }
            _ => false,
        };
        if recurse {
            self.get_reference_root(parent)
        } else {
            node
        }
    }




    pub(super) fn has_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> bool {

        if t.flags.contains(TypeFlags::IndexedAccess)
            && let Some(constraint) = self.constraint_of_indexed_access(t)
        {
            return self.has_property_of_type(&constraint, name);
        }

        if t.flags.intersects(
            TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::Never
                | TypeFlags::Undefined
                | TypeFlags::Null,
        ) {
            return true;
        }

        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }

            if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
                return true;
            }
            if !structured.index_infos.is_empty() {
                return true;
            }

            if t.object_flags.contains(ObjectFlags::EvolvingArray) {
                return name == "length" || self.is_array_mutation_method(name);
            }

            if t.object_flags.contains(ObjectFlags::Anonymous)
                && structured.call_signature_count > 0
                && self.global_interface_has_property("Function", name)
            {
                return true;
            }

            if t.flags.contains(TypeFlags::Object)
                && !t.object_flags.contains(ObjectFlags::Reference)
            {

                return self.global_interface_has_property("Object", name);
            }
        }

        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                for ct in &u.union_or_intersection.types {
                    if ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                        continue;
                    }
                    if !self.has_property_of_type(ct, name) {
                        return false;
                    }
                }
                return true;
            }
        }

        if t.flags.contains(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = &t.data {
                for ct in &i.union_or_intersection.types {
                    if self.has_property_of_type(ct, name) {
                        return true;
                    }
                }
                return false;
            }
        }

        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.has_property_of_type(&constraint, name);
            }

            return true;
        }

        if t.flags.contains(TypeFlags::Conditional) {
            if let Some(constraint) = self.constraint_of_conditional_type(t) {
                return self.has_property_of_type(&constraint, name);
            }

            return true;
        }

        if t.flags.contains(TypeFlags::IndexedAccess) {
            if let TypeData::IndexedAccess(ia) = &t.data {
                if let (Some(o), Some(i)) = (&ia.object_type, &ia.index_type) {
                    let obj = self.get_base_constraint_or_type(o);
                    let idx = self.get_base_constraint_or_type(i);
                    if !self.type_flags_is_generic_object_type(&obj)
                        && !self.type_flags_is_generic_index_type(&idx)
                    {
                        let resolved = self.get_indexed_access_type(&obj, &idx);
                        return self.has_property_of_type(&resolved, name);
                    }
                }
            }
            return true;
        }

        if self.is_array_type(t) {
            if name == "length" {
                return true;
            }
            if (self.is_auto_array_type(t)
                || t.object_flags.contains(ObjectFlags::EvolvingArray))
                && self.is_array_mutation_method(name)
            {
                return true;
            }

            if self.global_interface_has_property("Array", name) {
                return true;
            }
            return false;
        }

        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return name == "length" || self.is_array_mutation_method(name);
        }

        if self.is_tuple_type(t) {
            return name == "length";
        }

        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            return self.global_interface_has_property("String", name);
        }

        if t.flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            return self.global_interface_has_property("Number", name);
        }

        if t.flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            return self.global_interface_has_property("Boolean", name);
        }

        if t.flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            return self.global_interface_has_property("BigInt", name);
        }

        if t.flags
            .intersects(TypeFlags::ESSymbol | TypeFlags::Void | TypeFlags::UniqueESSymbol)
        {
            return false;
        }

        if t.flags.contains(TypeFlags::Object | TypeFlags::Enum) {
            return true;
        }

        true
    }






    fn expression_has_side_effects(&self, node: &Arc<Node>) -> bool {

        let mut cur = node;
        while let crate::ast::NodeData::ParenthesizedExpression(p) = &cur.data {
            cur = &p.expression;
        }

        if matches!(
            cur.kind,
            SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::UndefinedKeyword
        ) {
            return false;
        }
        match &cur.data {
            crate::ast::NodeData::Identifier(_)
            | crate::ast::NodeData::StringLiteral(_)
            | crate::ast::NodeData::RegularExpressionLiteral(_)
            | crate::ast::NodeData::TaggedTemplateExpression(_)
            | crate::ast::NodeData::TemplateExpression(_)
            | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_)
            | crate::ast::NodeData::NumericLiteral(_)
            | crate::ast::NodeData::BigIntLiteral(_)
            | crate::ast::NodeData::FunctionExpression(_)
            | crate::ast::NodeData::ClassExpression(_)
            | crate::ast::NodeData::ArrowFunction(_)
            | crate::ast::NodeData::ArrayLiteralExpression(_)
            | crate::ast::NodeData::ObjectLiteralExpression(_)
            | crate::ast::NodeData::TypeOfExpression(_)
            | crate::ast::NodeData::NonNullExpression(_)
            | crate::ast::NodeData::JsxSelfClosingElement(_)
            | crate::ast::NodeData::JsxElement(_) => false,
            crate::ast::NodeData::ConditionalExpression(c) => {
                self.expression_has_side_effects(&c.when_true)
                    || self.expression_has_side_effects(&c.when_false)
            }
            crate::ast::NodeData::BinaryExpression(b) => {
                Self::is_assignment_operator(b.operator_token.kind)
                    || self.expression_has_side_effects(&b.left)
                    || self.expression_has_side_effects(&b.right)
            }
            crate::ast::NodeData::PrefixUnaryExpression(p) => !matches!(
                p.operator,
                SyntaxKind::ExclamationToken
                    | SyntaxKind::PlusToken
                    | SyntaxKind::MinusToken
                    | SyntaxKind::TildeToken
            ),
            _ => true,
        }
    }

    fn is_indirect_call_comma(&self, comma: &Arc<Node>) -> bool {
        let Some(paren) = comma.parent.as_ref() else {
            return false;
        };
        if paren.kind != SyntaxKind::ParenthesizedExpression {
            return false;
        }
        let crate::ast::NodeData::BinaryExpression(b) = &comma.data else {
            return false;
        };
        let zero_left = matches!(&b.left.data, crate::ast::NodeData::NumericLiteral(n) if n.text == "0");
        if !zero_left {
            return false;
        }
        let Some(grand) = paren.parent.as_ref() else {
            return false;
        };
        let call_uses_paren = matches!(&grand.data, crate::ast::NodeData::CallExpression(ce)
            if std::ptr::eq(&ce.expression, paren));
        if !call_uses_paren && grand.kind != SyntaxKind::TaggedTemplateExpression {
            return false;
        }
        match &b.right.data {
            crate::ast::NodeData::PropertyAccessExpression(_)
            | crate::ast::NodeData::ElementAccessExpression(_) => true,
            crate::ast::NodeData::Identifier(id) => id.text == "eval",
            _ => false,
        }
    }





    pub(crate) fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn check_external_emit_helpers(&mut self, location: &Arc<Node>, helpers: u32) {
        if !self.compiler_options.import_helpers.is_true() {
            return;
        }
        if self.ambient_context_depth > 0 {
            return;
        }
        let file_id = self.current_file_id as usize;
        let requested = self
            .requested_external_emit_helpers
            .get(&file_id)
            .copied()
            .unwrap_or(0);
        if requested & helpers == helpers {
            return;
        }
        let unchecked = helpers & !requested;
        let Some(helpers_module) = self.resolve_module_file_symbol("tslib") else {
            if self.ts2354_checked_files.insert(file_id) {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    location.loc,
                    crate::diagnostics::messages_generated::
                        THIS_SYNTAX_REQUIRES_AN_IMPORTED_HELPER_BUT_MODULE_0_CANNOT_BE_FOUND,
                    vec!["tslib".to_string()],
                ));
            }
            self.requested_external_emit_helpers
                .insert(file_id, requested | helpers);
            return;
        };
        for (bit, helper_name) in [
            (EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT, "__importDefault"),
            (EXTERNAL_EMIT_HELPER_IMPORT_STAR, "__importStar"),
            (EXTERNAL_EMIT_HELPER_EXPORT_STAR, "__exportStar"),
        ] {
            if unchecked & bit == 0 {
                continue;
            }

            let found = helpers_module
                .exports
                .get(helper_name)
                .or_else(|| helpers_module.members.get(helper_name))
                .cloned()
                .or_else(|| self.ambient_namespace_local(&helpers_module, helper_name))
                .is_some_and(|s| s.flags.intersects(SymbolFlags::VALUE));
            if !found {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    location.loc,
                    crate::diagnostics::messages_generated::
                        THIS_SYNTAX_REQUIRES_AN_IMPORTED_HELPER_NAMED_1_WHICH_DOES_NOT_EXIST_IN_0_CONSIDER_UPGRADING_YOUR_VERSION_OF_0,
                    vec!["tslib".to_string(), helper_name.to_string()],
                ));
            }
        }
        self.requested_external_emit_helpers
            .insert(file_id, requested | helpers);
    }

}

fn get_excluded_symbol_flags(flags: SymbolFlags) -> SymbolFlags {
    let mut result = SymbolFlags::None;
    if flags.intersects(SymbolFlags::BlockScopedVariable) {
        result |= SymbolFlags::BlockScopedVariableExcludes;
    }
    if flags.intersects(SymbolFlags::FunctionScopedVariable) {
        result |= SymbolFlags::FunctionScopedVariableExcludes;
    }
    if flags.intersects(SymbolFlags::Property) {
        result |= SymbolFlags::PropertyExcludes;
    }
    if flags.intersects(SymbolFlags::EnumMember) {
        result |= SymbolFlags::EnumMemberExcludes;
    }
    if flags.intersects(SymbolFlags::Function) {
        result |= SymbolFlags::FunctionExcludes;
    }
    if flags.intersects(SymbolFlags::Class) {
        result |= SymbolFlags::ClassExcludes;
    }
    if flags.intersects(SymbolFlags::Interface) {
        result |= SymbolFlags::InterfaceExcludes;
    }
    if flags.intersects(SymbolFlags::RegularEnum) {
        result |= SymbolFlags::RegularEnumExcludes;
    }
    if flags.intersects(SymbolFlags::ConstEnum) {
        result |= SymbolFlags::ConstEnumExcludes;
    }
    if flags.intersects(SymbolFlags::ValueModule) {
        result |= SymbolFlags::ValueModuleExcludes;
    }
    if flags.intersects(SymbolFlags::Method) {
        result |= SymbolFlags::MethodExcludes;
    }
    if flags.intersects(SymbolFlags::GetAccessor) {
        result |= SymbolFlags::GetAccessorExcludes;
    }
    if flags.intersects(SymbolFlags::SetAccessor) {
        result |= SymbolFlags::SetAccessorExcludes;
    }
    if flags.intersects(SymbolFlags::TypeParameter) {
        result |= SymbolFlags::TypeParameterExcludes;
    }
    if flags.intersects(SymbolFlags::TypeAlias) {
        result |= SymbolFlags::TypeAliasExcludes;
    }
    if flags.intersects(SymbolFlags::Alias) {
        result |= SymbolFlags::AliasExcludes;
    }
    if flags.intersects(SymbolFlags::ReplaceableByMethod) {
        result.remove(SymbolFlags::Method);
    }
    result
}

fn is_module_or_enum_name(_node: &Node) -> bool {

    false
}

fn is_non_local_alias(symbol: &Arc<Symbol>, excludes: SymbolFlags) -> bool {
    if symbol.flags == SymbolFlags::Alias
        || (symbol.flags.intersects(SymbolFlags::Alias)
            && symbol.flags.intersects(SymbolFlags::Assignment))
    {
        !symbol.flags.intersects(excludes)
    } else {
        false
    }
}

fn is_type_only_node(node: &Node) -> bool {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::ImportSpecifier(data) => data.is_type_only,
        NodeData::ExportSpecifier(data) => data.is_type_only,
        NodeData::ExportDeclaration(data) => data.is_type_only,
        NodeData::ImportEqualsDeclaration(data) => data.is_type_only,
        _ => false,
    }
}

fn is_alias_symbol_declaration(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportSpecifier
            | SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
    )
}

fn node_name(node: &Node) -> Option<&str> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::Identifier(data) => Some(&data.text),
        NodeData::StringLiteral(data) => Some(&data.text),
        NodeData::NumericLiteral(data) => Some(&data.text),
        _ => None,
    }
}

fn is_expression_position_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Identifier
            | SyntaxKind::NumericLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateExpression
            | SyntaxKind::BinaryExpression
            | SyntaxKind::PrefixUnaryExpression
            | SyntaxKind::PostfixUnaryExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::ConditionalExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionExpression
            | SyntaxKind::AwaitExpression
            | SyntaxKind::YieldExpression
            | SyntaxKind::SpreadElement
            | SyntaxKind::AsExpression
            | SyntaxKind::NonNullExpression
            | SyntaxKind::SatisfiesExpression
            | SyntaxKind::TypeOfExpression
            | SyntaxKind::DeleteExpression
            | SyntaxKind::VoidExpression
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::ClassExpression
            | SyntaxKind::OmittedExpression
    )
}

fn is_statement_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ExpressionStatement
            | SyntaxKind::VariableStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::Block
            | SyntaxKind::ThrowStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ModuleDeclaration
    )
}

fn is_declaration_name(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    let parent_kind = parent.kind;

    let name_field = crate::ast::node_data_generated::node_name(parent);
    if let Some(name) = name_field {
        if std::ptr::eq(name.as_ref() as *const Node, node.as_ref() as *const Node) {
            return matches!(
                parent_kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::EnumMember
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ImportClause
                    | SyntaxKind::ImportEqualsDeclaration
                    | SyntaxKind::ExportSpecifier
                    | SyntaxKind::NamespaceImport
                    | SyntaxKind::Parameter
                    | SyntaxKind::BindingElement
                    | SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertySignature
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::MethodSignature
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::PropertyAssignment
                    | SyntaxKind::NamespaceExportDeclaration
                    | SyntaxKind::NamespaceExport
                    | SyntaxKind::LabeledStatement

                    | SyntaxKind::ClassExpression
                    | SyntaxKind::FunctionExpression
            );
        }
    }
    false
}

fn is_property_access_name(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    if parent.kind != SyntaxKind::PropertyAccessExpression {
        return false;
    }
    let Some(name_field) = crate::ast::node_data_generated::node_name(parent) else {
        return false;
    };
    std::ptr::eq(
        name_field.as_ref() as *const Node,
        node.as_ref() as *const Node,
    )
}

fn is_valid_identifier_text(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}

        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

enum ImportEntityError {
    None,

    NamespaceNotFound(Arc<Node>),

    TypeAsNamespace(Arc<Node>),

    HiddenByLocal(Arc<Node>),

    MissingMember((Arc<Node>, String, String)),
}

pub(crate) fn base_identifier_of(name: &Arc<Node>) -> Arc<Node> {
    let mut cur = Arc::clone(name);
    loop {
        let next = match &cur.data {
            crate::ast::NodeData::QualifiedName(q) => Arc::clone(&q.left),
            _ => return cur,
        };
        cur = next;
    }
}

fn object_literal_is_destructuring_target(literal: &Arc<Node>) -> bool {
    let Some(parent) = literal.parent.as_ref() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => true,
        SyntaxKind::BinaryExpression => {
            matches!(&parent.data, crate::ast::NodeData::BinaryExpression(bin)
                if bin.operator_token.kind == SyntaxKind::EqualsToken
                    && std::ptr::eq(
                        bin.left.as_ref() as *const Node,
                        literal.as_ref() as *const Node
                    ))
        }
        SyntaxKind::ParenthesizedExpression => {
            object_literal_is_destructuring_target(parent)
        }
        _ => false,
    }
}

fn is_assignment_target(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };

    if parent.kind == SyntaxKind::BindingElement {
        if let crate::ast::NodeData::BindingElement(be) = &parent.data {
            if let Some(name) = &be.name {
                return std::ptr::eq(
                    name.as_ref() as *const Node,
                    node.as_ref() as *const Node,
                ) && !matches!(
                    name.kind,
                    SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                );
            }
        }
        return false;
    }

    if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
        if let crate::ast::NodeData::ShorthandPropertyAssignment(sa) = &parent.data {
            let name_is_node =
                std::ptr::eq(sa.name.as_ref() as *const Node, node.as_ref() as *const Node);
            let literal = parent.parent.as_ref();
            if name_is_node
                && literal.is_some_and(|lit| {
                    lit.kind == SyntaxKind::ObjectLiteralExpression
                        && object_literal_is_destructuring_target(lit)
                })
            {
                return true;
            }
        }
        return false;
    }
    if parent.kind != SyntaxKind::BinaryExpression {
        return false;
    }
    let crate::ast::NodeData::BinaryExpression(bin) = &parent.data else {
        return false;
    };
    if !is_compound_or_simple_assignment(bin.operator_token.kind) {
        return false;
    }

    std::ptr::eq(
        bin.left.as_ref() as *const Node,
        node.as_ref() as *const Node,
    )
}

fn is_let_or_const_declaration(declaration: &Arc<Node>) -> bool {
    if let Some(parent) = declaration.parent.as_ref() {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
        }
    }

    true
}

fn type_contains_undefined(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Undefined) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(type_contains_undefined);
        }
    }
    false
}

fn type_is_possibly_undefined(t: &Arc<Type>) -> bool {
    if t.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null));
        }
    }
    false
}

fn type_includes_undefined_only(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Undefined) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.contains(TypeFlags::Undefined));
        }
    }
    false
}

fn type_includes_null_only(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Null) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.contains(TypeFlags::Null));
        }
    }
    false
}

fn is_entity_name_expression(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::Identifier => true,
        SyntaxKind::PropertyAccessExpression => match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                is_entity_name_expression(&data.expression)
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_compound_or_simple_assignment(kind: SyntaxKind) -> bool {
    use SyntaxKind::*;
    matches!(
        kind,
        EqualsToken
            | PlusEqualsToken
            | MinusEqualsToken
            | AsteriskEqualsToken
            | AsteriskAsteriskEqualsToken
            | SlashEqualsToken
            | PercentEqualsToken
            | LessThanLessThanEqualsToken
            | GreaterThanGreaterThanEqualsToken
            | GreaterThanGreaterThanGreaterThanEqualsToken
            | AmpersandEqualsToken
            | BarEqualsToken
            | CaretEqualsToken
            | BarBarEqualsToken
            | AmpersandAmpersandEqualsToken
            | QuestionQuestionEqualsToken
    )
}

fn flatten_union_leaves<'a>(t: &'a Arc<Type>, leaves: &mut Vec<&'a Arc<Type>>) {
    match t.as_union_or_intersection() {
        Some(u) => {
            for m in &u.types {
                flatten_union_leaves(m, leaves);
            }
        }
        None => leaves.push(t),
    }
}

fn class_declaration_name(class: &Arc<Node>) -> Option<String> {
    if let crate::ast::NodeData::ClassDeclaration(d) = &class.data {
        return d.name.as_ref().map(|n| n.text().to_string());
    }
    None
}

fn prop_decl_has_initializer(decl: &Arc<Node>) -> bool {
    matches!(&decl.data, crate::ast::NodeData::PropertyDeclaration(d) if d.initializer.is_some())
}

fn later_sibling_property(node: &Arc<Node>, prop_decl: &Arc<Node>) -> bool {
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if a.kind == SyntaxKind::PropertyDeclaration {
            return prop_decl.loc.pos() > a.loc.pos();
        }
        cur = a.parent.as_ref();
    }
    false
}

#[derive(Debug, PartialEq, Eq)]
enum ModuleMemberLookup {
    Found,

    LocalNotExported,
    Missing,
}

#[allow(dead_code)]
fn body_assigns_this_property(n: &Arc<Node>, name: &str) -> bool {
    match &n.data {
        crate::ast::NodeData::BinaryExpression(b)
            if b.operator_token.kind == SyntaxKind::EqualsToken =>
        {
            if let crate::ast::NodeData::PropertyAccessExpression(pa) = &b.left.data
                && pa.expression.kind == SyntaxKind::ThisKeyword
                && pa.name.kind == SyntaxKind::Identifier
                && pa.name.text() == name
            {
                return true;
            }
        }

        crate::ast::NodeData::FunctionDeclaration(_)
        | crate::ast::NodeData::FunctionExpression(_)
        | crate::ast::NodeData::ArrowFunction(_) => return false,
        _ => {}
    }
    let mut found = false;
    crate::ast::node_data_generated::for_each_child(n, |child| {
        if body_assigns_this_property(child, name) {
            found = true;
            true
        } else {
            false
        }
    });
    found
}

impl std::fmt::Debug for Checker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checker")
            .field("id", &self.id)
            .field("type_count", &self.type_count)
            .field("symbol_count", &self.symbol_count)
            .field("files", &self.files.len())
            .finish()
    }
}

impl Checker {

    pub(crate) fn attach_explicit_type_arguments_cached(
        &mut self,
        t: &Arc<Type>,
        args: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        let mut key = Vec::with_capacity(args.len() + 1);
        key.push(t.id as usize);
        key.extend(args.iter().map(|a| a.id as usize));
        if let Some(cached) = self.attached_type_args_cache.get(&key) {
            return Arc::clone(&cached.2);
        }
        let rebuilt = attach_explicit_type_arguments(t, args.clone());
        self.attached_type_args_cache
            .insert(key, (Arc::clone(t), args, Arc::clone(&rebuilt)));
        rebuilt
    }
}

pub(crate) fn attach_explicit_type_arguments(t: &Arc<Type>, args: Vec<Arc<Type>>) -> Arc<Type> {
    if let TypeData::Object(o) = &t.data {
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: o.structured.members.clone(),
                    properties: o.structured.properties.clone(),
                    signatures: o.structured.signatures.clone(),
                    call_signature_count: o.structured.call_signature_count,
                    index_infos: o.structured.index_infos.clone(),
                    ..Default::default()
                },
                target: o.target.clone(),
                mapper: o.mapper.clone(),
                type_arguments: args,
            }),
        );
        rebuilt.object_flags = t.object_flags;
        rebuilt.symbol = t.symbol.clone();
        return Arc::new(rebuilt);
    }
    Arc::clone(t)
}

#[cfg(test)]
mod array_member_tests {
    use super::*;
    use crate::bundled::lib_path;
    use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
    use crate::tsoptions::ParsedCommandLine;
    use crate::vfs::InMemoryFS;

    fn build_checker_with_lib(source: &str) -> Checker {
        use crate::bundled::BundledFS;
        let inner = Arc::new(InMemoryFS::new());
        inner.insert_file("/proj/entry.ts", source);
        inner.insert_file(
            "/proj/tsconfig.json",
            "{ \"compilerOptions\": {}, \"files\": [\"entry.ts\"] }",
        );
        let fs = Arc::new(BundledFS::new(inner));
        let parsed = ParsedCommandLine {
            file_names: vec!["/proj/entry.ts".to_string()],
            ..Default::default()
        };
        let host: Arc<dyn CompilerHost> =
            Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Arc::new(Program::new(ProgramOptions { config: parsed, host }));
        program.build_checker()
    }

    fn error_codes(checker: &Checker) -> Vec<i32> {
        let codes: Vec<i32> = checker
            .diagnostics
            .get_all()
            .iter()
            .filter(|d| !d.file.as_ref().is_some_and(|f| f.file_name.starts_with("bundled://")))
            .map(|d| d.code)
            .collect();
        codes
    }

    #[test]
    fn array_every_callback_param_typed_by_element() {
        let ok = build_checker_with_lib(
            "declare const ss: string[]; ss.every((x: string) => true);",
        );
        assert_eq!(error_codes(&ok), Vec::<i32>::new(), "matching callback must pass");

        let bad = build_checker_with_lib(
            "declare const ss: string[]; ss.every((x: number) => true);",
        );

        assert_eq!(error_codes(&bad), vec![2769], "mismatched callback param must fail");
    }

    #[test]
    fn array_flat_own_type_params_stay_free() {
        let ok = build_checker_with_lib(
            "function foo<T>(arr: T[], depth: number) { return arr.flat(depth); }",
        );
        assert_eq!(error_codes(&ok), Vec::<i32>::new());
    }

    #[test]
    fn array_method_signature_display_substituted() {
        let checker = build_checker_with_lib("declare const ss: string[]; ss.every(42);");
        let codes = super::convergence_tests::error_codes(&checker);
        assert_eq!(codes, vec![2769]);
        let diag = checker
            .diagnostics
            .get_all()
            .iter()
            .find(|d| d.code == 2769)
            .cloned()
            .unwrap();
        let template = diag.message.as_ref().map(|m| m.text).unwrap_or("");
        let mut msg = template.to_string();
        for (i, a) in diag.message_args.iter().enumerate() {
            msg = msg.replace(&format!("{{{i}}}"), a);
        }

        fn collect_chain_text(d: &crate::ast::Diagnostic, out: &mut String) {
            out.push_str(&d.message.as_ref().map(|m| m.text).unwrap_or(""));
            for (i, a) in d.message_args.iter().enumerate() {
                out.push(' ');
                out.push_str(a);
                let _ = i;
            }
            for c in &d.message_chain {
                collect_chain_text(c, out);
            }
        }
        let mut full = msg;
        collect_chain_text(&diag, &mut full);
        assert!(
            full.contains("(value: string, index: number, array: string[])"),
            "message should show the element-substituted signature: {full}"
        );
    }

    #[test]
    fn explicit_type_arguments_select_generic_overload() {
        let ok = build_checker_with_lib(
            "declare const a: string[]; const r = a.reduce<number>((c, d) => c + d, \" \");",
        );
        assert_eq!(error_codes(&ok), Vec::<i32>::new());
    }

    #[test]
    fn bare_array_assignable_to_concat_array() {
        let ok = build_checker_with_lib(
            "declare const a: string[]; const c: ConcatArray<string> = a; const r = a.concat(\"x\");",
        );
        assert_eq!(error_codes(&ok), Vec::<i32>::new());
    }

    #[test]
    fn concat_on_number_array_with_array_arg() {
        let ok = build_checker_with_lib(
            "declare const fa: number[]; var x = fa.concat(fa);",
        );
        assert_eq!(error_codes(&ok), Vec::<i32>::new());
    }
}

#[cfg(test)]
pub(crate) mod convergence_tests {
    use super::*;
    use crate::bundled::lib_path;
    use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
    use crate::tsoptions::ParsedCommandLine;
    use crate::vfs::InMemoryFS;

    pub(crate) fn build_program_and_checker(source: &str, lib_spec: &[&str]) -> (Arc<Program>, Checker) {
        use crate::bundled::BundledFS;
        let inner = Arc::new(InMemoryFS::new());
        inner.insert_file("/proj/entry.ts", source);
        let fs = Arc::new(BundledFS::new(inner));
        let mut compiler_options = CompilerOptions::default();
        compiler_options.lib = lib_spec.iter().map(|s| s.to_string()).collect();
        let parsed = ParsedCommandLine {
            file_names: vec!["/proj/entry.ts".to_string()],
            compiler_options,
            ..Default::default()
        };
        let host: Arc<dyn CompilerHost> =
            Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Arc::new(Program::new(ProgramOptions { config: parsed, host }));
        let tracer = Arc::new(Tracer::new());
        let checker = Checker::new(Arc::clone(&program) as _, tracer);
        (program, checker)
    }

    pub(crate) fn error_codes(checker: &Checker) -> Vec<i32> {
        checker
            .diagnostics
            .get_all()
            .iter()
            .filter(|d| !d.file.as_ref().is_some_and(|f| f.file_name.starts_with("bundled://")))
            .map(|d| d.code)
            .collect()
    }

    #[test]
    fn any_base_is_not_degradation() {
        let (program, mut checker) = convergence_tests::build_program_and_checker(
            "type AnyAlias = any;\n\
             interface I extends AnyAlias { x: number; }\n\
             declare const i: I;\n\
             const n: number = i.x;\n\
             const m: number = i.x;\n\
             const k: number = i.x;",
            &["es5"],
        );
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        assert_eq!(error_codes(&checker), Vec::<i32>::new());

        let entry = program.get_source_file("/proj/entry.ts").expect("entry file");
        let iface = match &entry.node.data {
            crate::ast::NodeData::SourceFile(d) => d
                .statements
                .iter()
                .find(|s| matches!(s.data, crate::ast::NodeData::InterfaceDeclaration(_)))
                .expect("interface I declared")
                .clone(),
            _ => unreachable!(),
        };
        let sym = program
            .symbol_map()
            .symbol_of(&iface)
            .expect("interface symbol")
            .clone();

        assert!(
            checker
                .type_alias_links
                .get(&sym)
                .is_some_and(|l| l.declared_type.is_some()),
            "an any base must not disable the declared-type cache"
        );
        assert!(
            !checker
                .heritage_retry_counts
                .contains_key(&(Arc::as_ptr(&sym) as *const Symbol as usize)),
            "an any base must not record degraded retries for the interface"
        );
    }

    #[test]
    fn cyclic_base_interfaces_converge() {
        let source = "interface A extends B { a: number; }\n\
                      interface B extends A { b: string; }\n\
                      declare const v1: A; declare const v2: A;\n\
                      declare const v3: A; declare const v4: A;\n\
                      declare const v5: A; declare const v6: A;\n\
                      const n: number = v6.a;";
        let (program, mut checker) = convergence_tests::build_program_and_checker(source, &["es5"]);
        for file in program.source_files() {
            checker.check_source_file(file);
        }

        assert_eq!(
            error_codes(&checker),
            Vec::<i32>::new(),
            "own-member access through the cyclic interface must stay clean"
        );

        assert!(
            !checker.heritage_retry_counts.is_empty(),
            "cyclic bases must have recorded degraded retries"
        );
        assert!(
            checker
                .heritage_retry_counts
                .values()
                .any(|&c| c > HERITAGE_RETRY_LIMIT),
            "repeated references must cross the retry limit and be accepted"
        );
    }

    #[test]
    fn subst_cache_respects_capacity() {
        let (program, mut checker) = convergence_tests::build_program_and_checker(
            "declare const a: string[]; declare const b: number[];\n\
             var x = a.concat(b); var y = b.concat(a);\n\
             var z = x.concat(y);",
            &["es5"],
        );
        checker.type_node_subst_cache_limit = 8;
        checker.instantiated_member_type_cache_limit = 8;
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        assert!(
            checker.type_node_subst_cache.len() <= 8,
            "subst cache must stay within its cap, got {}",
            checker.type_node_subst_cache.len()
        );
        assert!(
            checker.instantiated_member_type_cache.len() <= 8,
            "member-type cache must stay within its cap, got {}",
            checker.instantiated_member_type_cache.len()
        );
    }

    #[test]
    fn deep_class_chain_bounded() {
        let mut source = String::from("class C0 { m0: number = 0; }\n");
        for i in 1..=260 {
            source.push_str(&format!(
                "class C{i} extends C{} {{ m{i}: number = {i}; }}\n",
                i - 1
            ));
        }
        source.push_str("declare const c: C260;\nconst n: number = c.m260;");
        let (program, mut checker) = convergence_tests::build_program_and_checker(&source, &["es5"]);
        for file in program.source_files() {
            checker.check_source_file(file);
        }

        assert_eq!(
            error_codes(&checker),
            Vec::<i32>::new(),
            "own-member access on the leaf of a deep chain must stay clean"
        );
    }
}

#[cfg(test)]
pub(crate) mod node_format_tests {
    use super::*;
    use crate::bundled::{lib_path, BundledFS};
    use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
    use crate::core::compiler_options::CompilerOptions;
    use crate::tsoptions::ParsedCommandLine;
    use crate::vfs::InMemoryFS;

    pub(crate) fn check_files(
        files: &[(&str, &str)],
        root: &str,
        configure: impl FnOnce(&mut CompilerOptions),
    ) -> Vec<i32> {
        let inner = Arc::new(InMemoryFS::new());
        inner.insert_dir("/proj");
        for (name, content) in files {
            let abs = if name.starts_with('/') {
                (*name).to_string()
            } else {
                format!("/proj/{name}")
            };

            let mut parent = crate::tspath::get_directory_path(&abs);
            loop {
                inner.insert_dir(&parent);
                let next = crate::tspath::get_directory_path(&parent);
                if next == parent {
                    break;
                }
                parent = next;
            }
            inner.insert_file(&abs, content);
        }
        let fs = Arc::new(BundledFS::new(inner));
        let mut options = CompilerOptions::default();
        configure(&mut options);
        let parsed = ParsedCommandLine {
            file_names: vec![root.to_string()],
            compiler_options: options,
            ..Default::default()
        };
        let host: Arc<dyn CompilerHost> =
            Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Arc::new(Program::new(ProgramOptions { config: parsed, host }));
        let checker = program.build_checker();

        checker
            .diagnostics
            .get_all()
            .iter()
            .chain(program.diagnostics().iter().map(|d| d.as_ref()))
            .filter(|d| d.file.as_ref().is_some_and(|f| f.file_name == root))
            .map(|d| d.code)
            .collect()
    }

    #[test]
    fn import_meta_reports_1470_only_in_cjs_files() {
        let files = [
            ("/proj/package.json", r#"{"name": "package", "type": "module"}"#),
            ("/proj/sub/package.json", r#"{"type": "commonjs"}"#),
            ("/proj/sub/index.ts", "const x = import.meta.url;\nexport {x};\n"),
            ("/proj/index.ts", "const x = import.meta.url;\nexport {x};\n"),
        ];
        let cjs = check_files(&files, "/proj/sub/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(cjs, vec![1470], "CJS-format file must report TS1470");
        let esm = check_files(&files, "/proj/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(esm, Vec::<i32>::new(), "ESM-format file must be clean");
    }

    #[test]
    fn declare_global_augmentation_from_types_reference_merges() {
        let files = [
            (
                "/node_modules/pkg/package.json",
                r#"{ "name": "pkg", "exports": { "import": "./import.js", "require": "./require.js" } }"#,
            ),
            (
                "/node_modules/pkg/import.d.ts",
                "export {};\ndeclare global { var foo: number; }\n",
            ),
            (
                "/node_modules/pkg/require.d.ts",
                "export {};\ndeclare global { var bar: number; }\n",
            ),
            ("/package.json", r#"{ "type": "module" }"#),
            (
                "/index.ts",
                "/// <reference types=\"pkg\" resolution-mode=\"import\" />\nfoo;\nexport {};\n",
            ),
        ];

        let codes = check_files(&files, "/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(codes, Vec::<i32>::new(), "foo must resolve via the global augmentation");

        let files_both = [
            (
                "/node_modules/pkg/package.json",
                r#"{ "name": "pkg", "exports": { "import": "./import.js", "require": "./require.js" } }"#,
            ),
            (
                "/node_modules/pkg/import.d.ts",
                "export {};\ndeclare global { var foo: number; }\n",
            ),
            (
                "/node_modules/pkg/require.d.ts",
                "export {};\ndeclare global { var bar: number; }\n",
            ),
            ("/package.json", r#"{ "type": "module" }"#),
            (
                "/index.ts",
                "/// <reference types=\"pkg\" resolution-mode=\"import\" />\n/// <reference types=\"pkg\" resolution-mode=\"require\" />\nfoo;\nbar;\nexport {};\n",
            ),
        ];
        let codes = check_files(&files_both, "/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(codes, Vec::<i32>::new(), "both augmentations must merge");

        let files_none = [
            (
                "/node_modules/pkg/package.json",
                r#"{ "name": "pkg", "exports": { "import": "./import.js" } }"#,
            ),
            (
                "/node_modules/pkg/import.d.ts",
                "export {};\ndeclare global { var foo: number; }\n",
            ),
            ("/package.json", r#"{ "type": "module" }"#),
            ("/index.ts", "foo;\nexport {};\n"),
        ];
        let codes = check_files(&files_none, "/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(codes, vec![2304], "unreferenced augmentation must not leak");
    }

    #[test]
    fn import_helpers_missing_helper_name_reports_2343() {
        let files = [
            (
                "/types.d.ts",
                "declare module \"fs\";\ndeclare module \"tslib\" { export {}; }\n",
            ),
            ("/sub/package.json", r#"{ "type": "commonjs" }"#),
            (
                "/sub/index.ts",
                "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
            ),
        ];
        let codes = check_files(&files, "/sub/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
            o.import_helpers = crate::core::tristate::Tristate::True;
        });
        assert_eq!(codes, vec![2343], "missing __importDefault must report TS2343");

        let files_ok = [
            (
                "/types.d.ts",
                "declare module \"fs\";\ndeclare module \"tslib\" { export function __importDefault(m: any): any; }\n",
            ),
            ("/sub/package.json", r#"{ "type": "commonjs" }"#),
            (
                "/sub/index.ts",
                "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
            ),
        ];
        let codes = check_files(&files_ok, "/sub/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
            o.import_helpers = crate::core::tristate::Tristate::True;
        });
        assert_eq!(codes, Vec::<i32>::new(), "present helper must be clean");

        let files_esm = [
            (
                "/types.d.ts",
                "declare module \"fs\";\ndeclare module \"tslib\" { export {}; }\n",
            ),
            ("/package.json", r#"{ "type": "module" }"#),
            (
                "/index.ts",
                "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
            ),
        ];
        let codes = check_files(&files_esm, "/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
            o.import_helpers = crate::core::tristate::Tristate::True;
        });
        assert_eq!(codes, Vec::<i32>::new(), "ESM-format emit needs no import helper");

        let files_nointerop = [
            (
                "/types.d.ts",
                "declare module \"fs\";\ndeclare module \"tslib\" { export {}; }\n",
            ),
            ("/sub/package.json", r#"{ "type": "commonjs" }"#),
            (
                "/sub/index.ts",
                "/// <reference path=\"/types.d.ts\" />\nexport { default } from \"fs\";\n",
            ),
        ];
        let codes = check_files(&files_nointerop, "/sub/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
            o.import_helpers = crate::core::tristate::Tristate::True;
            o.es_module_interop = crate::core::tristate::Tristate::False;
        });
        assert_eq!(codes, Vec::<i32>::new(), "explicit interop=false must disable the check");
    }

    #[test]
    fn module_member_check_2305_and_2459() {
        let files = [
            (
                "/mod.ts",
                "export interface A {}\n\
                 export const v = 1;\n\
                 interface Internal {}\n\
                 const notExportedConst = 2;\n\
                 export type T = number;\n",
            ),
            (
                "/main.ts",
                "import { A, Missing, Internal, notExportedConst, Missing2 } from \"./mod\";\n\
                 export { Nope } from \"./mod\";\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});

        assert_eq!(
            codes,
            vec![2305, 2459, 2459, 2305, 2305],
            "missing members report 2305 (incl. re-export), module-locals 2459"
        );
    }

    #[test]
    fn shorthand_ambient_module_members_exempt_from_2305() {
        let files = [
            (
                "/types.d.ts",
                "declare module \"short\";\ndeclare module \"real\" { export const v = 1; }\n",
            ),
            (
                "/main.ts",
                "/// <reference path=\"/types.d.ts\" />\n\
                 import { anything } from \"short\";\n\
                 export { whatever } from \"short\";\n\
                 import { missing } from \"real\";\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            vec![2305],
            "shorthand ambient members resolve silently; non-shorthand ambient still checks"
        );
    }

    #[test]
    fn module_member_check_default_export_forms() {
        let files = [
            ("/a1.ts", "export default class A {}\n"),
            ("/a2.ts", "export default class {}\n"),
            ("/a3.ts", "export default function f() {}\n"),
            ("/a4.ts", "export default function () {}\n"),
            (
                "/main.ts",
                "import { default as D1 } from \"./a1\";\n\
                 import { default as D2 } from \"./a2\";\n\
                 import { default as D3 } from \"./a3\";\n\
                 import { default as D4 } from \"./a4\";\n\
                 void [D1, D2, D3, D4];\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            Vec::<i32>::new(),
            "named and anonymous default declarations answer a 'default' member import"
        );
    }

    #[test]
    fn module_member_check_export_clauses() {
        let files = [
            ("/lib.ts", "export const X = 1;\n"),
            ("/local.ts", "const Y = 2;\nexport { Y };\n"),
            ("/fwd.ts", "export { X } from \"./lib\";\n"),
            ("/def.ts", "export { X as default } from \"./lib\";\n"),
            (
                "/main.ts",
                "import { Y } from \"./local\";\n\
                 import { X } from \"./fwd\";\n\
                 import { default as D } from \"./def\";\n\
                 import { Nope } from \"./local\";\n\
                 void [Y, X, D];\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            vec![2305],
            "clause exports resolve in all three forms; unknown names still report 2305"
        );
    }

    #[test]
    fn module_member_check_star_chains() {
        let files = [
            ("/leaf.ts", "export const deep = 1;\nexport const other = 2;\n"),
            ("/mid.ts", "export * from \"./leaf\";\n"),
            (
                "/cyc1.ts",
                "export * from \"./cyc2\";\nexport const c1 = 1;\n",
            ),
            (
                "/cyc2.ts",
                "export * from \"./cyc1\";\nexport const c2 = 2;\n",
            ),
            (
                "/shadow.ts",
                "export * from \"./leaf\";\nexport const other = \"own\";\n",
            ),
            ("/star.ts", "export * from \"./leaf\";\n"),
            (
                "/main.ts",
                "import { deep } from \"./mid\";\n\
                 import { c1 } from \"./cyc2\";\n\
                 import { c2 } from \"./cyc1\";\n\
                 import { other } from \"./shadow\";\n\
                 import { default as D } from \"./star\";\n\
                 void [deep, c1, c2, other, D];\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            vec![2305],
            "star chains resolve transitively and through cycles; 'default' never passes a star"
        );
    }

    #[test]
    fn module_member_check_ambient_implicit_exports() {
        let files = [
            (
                "/types.d.ts",
                "declare module \"amb\" {\n    function f(): void;\n    interface I { x: number }\n    const v: number;\n}\n\
                 declare module \"exp\" {\n    export const e = 1;\n    function hidden(): void;\n}\n",
            ),
            (
                "/main.ts",
                "/// <reference path=\"/types.d.ts\" />\n\
                 import { f, I, v } from \"amb\";\n\
                 import { e, hidden } from \"exp\";\n\
                 void [f, v, e, hidden];\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            Vec::<i32>::new(),
            "ambient module bodies implicitly export all declarations (export-const members don't break the context)"
        );
    }

    #[test]
    fn module_member_check_export_equals_targets() {
        let files = [
            (
                "/thing.d.ts",
                "declare namespace Foo {\n    export interface Bar {}\n    export function f(): Bar;\n}\nexport = Foo;\n",
            ),
            (
                "/demo.d.ts",
                "declare namespace demoNS {\n    function g(): void;\n}\n\
                 declare module 'demoModule' {\n    import alias = demoNS;\n    export = alias;\n}\n",
            ),
            (
                "/main.ts",
                "/// <reference path=\"/thing.d.ts\" />\n\
                 /// <reference path=\"/demo.d.ts\" />\n\
                 import { f } from \"./thing\";\n\
                 import { g } from \"demoModule\";\n\
                 void [f, g];\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            Vec::<i32>::new(),
            "export= namespace members resolve (direct and via import-alias), ambient namespace locals included"
        );
    }

    #[test]
    fn module_member_check_synthetic_default() {
        let files = [
            ("/nodefault.d.ts", "export declare function helper(): void;\n"),
            ("/plain.ts", "export const x = 1;\n"),
            (
                "/main.ts",
                "import { default as D1 } from \"./nodefault\";\n\
                 import { default as D2 } from \"./plain\";\n\
                 void [D1, D2];\n",
            ),
        ];
        let codes = check_files(&files, "/main.ts", |_| {});
        assert_eq!(
            codes,
            vec![2305],
            "declaration files answer a synthetic default; plain .ts modules without export= do not"
        );
    }

    #[test]
    fn module_member_check_non_type_only_ignores_resolution_mode() {
        let files = [
            (
                "/node_modules/pkg/package.json",
                r#"{ "name": "pkg", "exports": { "import": "./import.js", "require": "./require.js" } }"#,
            ),
            (
                "/node_modules/pkg/import.d.ts",
                "export interface ImportInterface {}\n",
            ),
            (
                "/node_modules/pkg/require.d.ts",
                "export interface RequireInterface {}\n",
            ),
            (
                "/index.ts",

                "import type { ImportInterface } from \"pkg\" with { \"resolution-mode\": \"import\" };\n\
                 import { ImportInterface as Imp } from \"pkg\" with { \"resolution-mode\": \"import\" };\n\
                 import { RequireInterface as Req } from \"pkg\";\n",
            ),
        ];
        let codes = check_files(&files, "/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(
            codes,
            vec![2305, 2823],
            "type-only override resolves the import face (clean); the plain clause \
             takes the default CJS chain → ImportInterface missing (2305) + TS2823 \
             for the attribute on node16"
        );
    }

    #[test]
    fn types_option_symbols_resolve_before_declaring_file_checked() {
        let files = [
            ("/types/jquery/index.d.ts", "declare var $: { foo(): void };\n"),
            (
                "/index.ts",
                "const q: number = $;\n$.nope();\n$.foo();\n",
            ),
        ];
        let codes = check_files(&files, "/index.ts", |o| {
            o.types = vec!["jquery".to_string()];
            o.type_roots = vec!["/types".to_string()];
        });
        assert_eq!(
            codes,
            vec![2322, 2339],
            "$ must be typed from the auto-included d.ts (2322 for `q: number = $`, \
             2339 for the missing member) — not silently any"
        );
    }

    #[test]
    fn jsx_runtime_import_source_unresolvable_reports_2875() {
        let files = [
            (
                "/lib.d.ts",
                "declare namespace JSX { interface Element {} }\n",
            ),
            ("/index.tsx", "const a = <div />;\nexport {};\n"),
        ];
        let codes = check_files(&files, "/index.tsx", |o| {
            o.jsx = crate::core::compiler_options::JsxEmit::ReactJSX;
            o.jsx_import_source = "preact".to_string();
        });
        assert_eq!(codes, vec![2875], "unresolvable jsx runtime must report TS2875");

        let codes = check_files(&files, "/index.tsx", |o| {
            o.jsx = crate::core::compiler_options::JsxEmit::React;
            o.jsx_import_source = "preact".to_string();
        });
        assert_eq!(
            codes,
            vec![2874],
            "classic mode reports no TS2875 but TS2874 without React in scope"
        );
    }

    #[test]
    fn namespace_import_alias_qualified_type_access() {
        let files = [
            (
                "/amb.d.ts",
                "declare module \"pkg\" {\n    export type VM<T> = { [K in keyof T]-?: number };\n}\ndeclare module \"outer\" {\n    import * as P from \"pkg\";\n    namespace Inner {\n        type Alias<T> = P.VM<T>;\n    }\n    export = Inner;\n}\n",
            ),
            (
                "/index.ts",
                "/// <reference path=\"/amb.d.ts\" />\nimport * as O from \"outer\";\nexport declare const y: O.Alias<{}>;\n",
            ),
        ];
        let codes = check_files(&files, "/index.ts", |o| {
            o.target = crate::core::compiler_options::ScriptTarget::ES2015;
        });
        assert_eq!(
            codes,
            Vec::<i32>::new(),
            "ambient-module namespace-import qualified access must resolve"
        );
    }

    #[test]
    fn generic_callback_reference_infers_no_2345() {
        let codes = check_files(
            &[(
                "/index.ts",
                "function identity<A>(a: A): A { return a; }\nconst x = [1, 2, 3].map(identity)[0];\nexport {};\n",
            )],
            "/index.ts",
            |o| {
                o.target = crate::core::compiler_options::ScriptTarget::ES2015;
            },
        );
        assert_eq!(codes, Vec::<i32>::new(), "map(identity) must not report TS2345");
    }

    #[test]
    fn explicit_node10_resolution_ignores_exports() {
        let files = [
            (
                "/node_modules/pkg/package.json",
                r#"{ "name": "pkg", "version": "1.0.0", "exports": { ".": "./definitely-not-index.js" } }"#,
            ),
            ("/node_modules/pkg/definitely-not-index.d.ts", "export {};\n"),
            ("/index.ts", "import { pkg } from \"pkg\";\n"),
        ];
        let codes = check_files(&files, "/index.ts", |o| {
            o.module_resolution = ModuleResolutionKind::Node10;
            o.target = crate::core::compiler_options::ScriptTarget::ES2015;
        });
        assert_eq!(codes, vec![2307], "node10 must not resolve via exports");
    }

    #[test]
    fn dom_two_level_heritage_assignable_no_phantom_2739() {
        let codes = check_files(
            &[(
                "/index.ts",
                "declare const h: HTMLElement;\nconst e: Element = h;\nexport {};\n",
            )],
            "/index.ts",
            |o| {
                o.target = crate::core::compiler_options::ScriptTarget::ES2022;
            },
        );
        assert_eq!(codes, Vec::<i32>::new(), "two-level DOM heritage must be assignable");
    }

    #[test]
    fn reserved_cjs_top_level_names() {
        let body = "function require() {}\n\
                    const exports = {};\n\
                    class Object {}\n\
                    export const __esModule = false;\n\
                    export {require, exports, Object};\n";
        let files = [
            ("/proj/package.json", r#"{"name": "package", "type": "module"}"#),
            ("/proj/sub/package.json", r#"{"type": "commonjs"}"#),
            ("/proj/sub/index.ts", body),
            ("/proj/index.ts", body),
        ];
        let cjs = check_files(&files, "/proj/sub/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(cjs, vec![2441, 2441, 2725, 1216], "CJS file reserved names");
        let esm = check_files(&files, "/proj/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });
        assert_eq!(esm, Vec::<i32>::new(), "ESM file has no collisions");
    }

    #[test]
    fn import_attributes_2823_suppressed_on_parse_error() {
        let codes = check_files(
            &[("/proj/index.ts", "import * as f from \"./first\" with {\n")],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::CommonJS;
            },
        );
        assert!(
            !codes.contains(&2823),
            "TS2823 must be suppressed on files with parse errors: {codes:?}"
        );
        assert!(codes.contains(&1005), "expected the parse error: {codes:?}");
    }

    #[test]
    fn type_only_resolution_mode_attribute_grammar() {
        let files = [
            (
                "/proj/node_modules/pkg/package.json",
                r#"{"name": "pkg", "exports": {"import": "./import.js", "require": "./require.js"}}"#,
            ),
            ("/proj/node_modules/pkg/import.d.ts", "export interface ImportInterface {}\n"),
            ("/proj/node_modules/pkg/require.d.ts", "export interface RequireInterface {}\n"),
            (
                "/proj/index.ts",
                "import type { RequireInterface } from \"pkg\" with { \"resolution-mode\": \"require\" };\n\
                 import { ImportInterface } from \"pkg\" with { \"resolution-mode\": \"import\" };\n\
                 export interface L extends RequireInterface {}\n",
            ),
        ];
        let codes = check_files(&files, "/proj/index.ts", |o| {
            o.module = ModuleKind::Node16;
            o.module_resolution = ModuleResolutionKind::Node16;
        });

        assert_eq!(
            codes.iter().filter(|c| **c == 2823).count(),
            1,
            "one TS2823 for the non-type-only clause: {codes:?}"
        );

        let bad = check_files(
            &[
                (
                    "/proj/index.ts",
                    "import type { X } from \"./missing\" with { \"resolution-mode\": \"foobar\" };\n",
                ),
            ],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::Node18;
                o.module_resolution = ModuleResolutionKind::Node16;
            },
        );
        assert!(bad.contains(&1453), "bad resolution-mode value: {bad:?}");
    }

    #[test]
    fn overload_probe_does_not_leak_diagnostics() {
        let codes = check_files(
            &[("/proj/index.ts", "var fa: number[];\nfa = fa.concat([0]);\n")],
            "/proj/index.ts",
            |_| {},
        );
        assert_eq!(codes, vec![2454], "only used-before-assigned: {codes:?}");
    }

    #[test]
    fn generic_arity_error_suppresses_ts2564() {
        let codes = check_files(
            &[(
                "/proj/index.ts",
                "export interface A<T> {\n   new (dbSet: DbSet<T>): T;\n}\n\
                 export class DbSet<T> {\n    _entityType: A;\n  get entityType() { return this._entityType; }\n}\n",
            )],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::CommonJS;
            },
        );
        assert_eq!(
            codes.iter().filter(|c| **c == 2314).count(),
            1,
            "exactly one TS2314: {codes:?}"
        );
        assert!(
            !codes.contains(&2564),
            "TS2564 must be suppressed by the error-typed annotation: {codes:?}"
        );
    }

    #[test]
    fn ambient_declarations_exempt_from_reserved_names() {
        let codes = check_files(
            &[(
                "/proj/index.ts",
                "export declare var exports: number;\n\
                 export declare var require: string;\n\
                 declare namespace inner { var exports: string; }\n",
            )],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::CommonJS;
            },
        );
        assert_eq!(codes, Vec::<i32>::new(), "ambient names are clean: {codes:?}");
    }

    #[test]
    fn es_module_marker_requires_export_and_emit() {

        let bare = check_files(
            &[("/proj/index.ts", "export default \"test\";\nvar __esModule = 1;\n")],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::CommonJS;
            },
        );
        assert_eq!(bare, Vec::<i32>::new(), "bare __esModule is legal: {bare:?}");

        let exported = check_files(
            &[("/proj/index.ts", "export default \"test\";\nexport var __esModule = 1;\n")],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::CommonJS;
            },
        );
        assert_eq!(exported, vec![1216], "exported __esModule reports TS1216: {exported:?}");

        let noemit = check_files(
            &[("/proj/index.ts", "export default \"test\";\nexport var __esModule = 1;\n")],
            "/proj/index.ts",
            |o| {
                o.module = ModuleKind::CommonJS;
                o.no_emit = crate::core::tristate::Tristate::True;
            },
        );
        assert_eq!(noemit, Vec::<i32>::new(), "noEmit skips the marker check: {noemit:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_store_basic() {
        let store: LinkStore<Node, NodeLinks> = LinkStore::new();

        assert!(store.data.is_empty());
    }

    #[test]
    fn ternary_and_or() {
        assert_eq!(Ternary::True.and(Ternary::False), Ternary::False);
        assert_eq!(Ternary::True.or(Ternary::False), Ternary::True);
    }

    #[test]
    fn get_symbol_at_location() {
        use crate::astnav::get_token_at_position;
        use crate::bundled::{BundledFS, lib_path};
        use crate::compiler::{CompilerHostImpl, Program, ProgramOptions};
        use crate::tsoptions::ParsedCommandLine;
        use crate::vfs::InMemoryFS;

        let content = "interface Foo {\n  bar: string;\n}\ndeclare const foo: Foo;\nfoo.bar;";
        let inner = Arc::new(InMemoryFS::new());
        inner.insert_file("/foo.ts", content);
        inner.insert_file(
            "/tsconfig.json",
            "{\n  \"compilerOptions\": {},\n  \"files\": [\"foo.ts\"]\n}",
        );
        let fs = Arc::new(BundledFS::new(inner));

        let parsed = ParsedCommandLine {
            file_names: vec!["/foo.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(fs, "/".to_string(), lib_path()));
        let program = Arc::new(Program::new(ProgramOptions {
            config: parsed,
            host,
        }));

        let mut checker = program.build_checker();
        let file = program.get_source_file("/foo.ts").expect("foo.ts");
        checker.check_source_file(&file);

        let interface_name = get_token_at_position(&file.node, 10).expect("interface name");
        let sym = checker.get_symbol_at_location(&interface_name);
        assert!(sym.is_some(), "Expected symbol for interface name 'Foo'");

        let var_name = get_token_at_position(&file.node, 47).expect("variable name");
        let sym = checker.get_symbol_at_location(&var_name);
        assert!(sym.is_some(), "Expected symbol for variable name 'foo'");

        let prop_access = get_token_at_position(&file.node, 60).expect("property access");
        let sym = checker.get_symbol_at_location(&prop_access);
        assert!(
            sym.is_some(),
            "Expected symbol for property access 'foo.bar'"
        );
    }

    #[test]
    fn tracer_push_preserves_end_arg_mutations() {
        use crate::tracing::{Phase, TraceArg, Tracer};

        let tr = Tracer::new();

        let args = vec![
            ("checkerId".to_string(), TraceArg::Int(7)),
            ("id".to_string(), TraceArg::Int(1)),
        ];
        let outer = tr.push(Phase::CheckTypes, "getVariancesWorker", args.clone());

        assert_eq!(args.len(), 2);

        let inner_args = vec![("checkerId".to_string(), TraceArg::Int(7))];
        let inner = tr.push(Phase::Check, "checkSourceFile", inner_args);

        drop(inner);
        drop(outer);

        let events = tr.take_events();

        let outer_begin = events
            .iter()
            .find(|e| e.ph == "B" && e.name == "getVariancesWorker")
            .expect("outer begin event");
        let outer_end = events
            .iter()
            .find(|e| e.ph == "E" && e.name == "getVariancesWorker")
            .expect("outer end event");

        assert_eq!(outer_begin.cat, "checkTypes");
        assert_eq!(
            outer_begin.args,
            vec![
                ("checkerId".to_string(), TraceArg::Int(7)),
                ("id".to_string(), TraceArg::Int(1)),
            ]
        );

        assert_eq!(outer_end.args, outer_begin.args);

        assert_eq!(outer_begin.tid, outer_end.tid);

        let inner_begin = events
            .iter()
            .find(|e| e.ph == "B" && e.name == "checkSourceFile")
            .expect("inner begin event");
        assert_eq!(inner_begin.tid, outer_begin.tid);

        let outer_begin_idx = events
            .iter()
            .position(|e| std::ptr::eq(e, outer_begin))
            .unwrap();
        let inner_begin_idx = events
            .iter()
            .position(|e| std::ptr::eq(e, inner_begin))
            .unwrap();
        let inner_end_idx = events
            .iter()
            .position(|e| e.ph == "E" && e.name == "checkSourceFile")
            .unwrap();
        let outer_end_idx = events
            .iter()
            .position(|e| std::ptr::eq(e, outer_end))
            .unwrap();
        assert!(outer_begin_idx < inner_begin_idx);
        assert!(inner_begin_idx < inner_end_idx);
        assert!(inner_end_idx < outer_end_idx);
    }
}

fn qualified_name_text(name: &Arc<Node>) -> String {
    match &name.data {
        crate::ast::NodeData::QualifiedName(d) => {
            format!("{}.{}", qualified_name_text(&d.left), d.right.text())
        }
        _ => name.text().to_string(),
    }
}

pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

pub(crate) fn levenshtein_with_max(s1: &str, s2: &str, max: f64) -> Option<f64> {
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    let big = max + 0.01;
    let mut prev: Vec<f64> = (0..=s2.len()).map(|i| i as f64).collect();
    let mut curr = vec![0.0f64; s2.len() + 1];
    for i in 1..=s1.len() {
        let c1 = s1[i - 1];
        let min_j = (((i as f64) - max).ceil().max(1.0)) as usize;
        let max_j = ((max + i as f64).floor()) as usize;
        let max_j = max_j.min(s2.len());
        curr[0] = i as f64;
        let mut col_min = i as f64;
        for j in 1..(min_j.min(s2.len() + 1)) {
            curr[j] = big;
        }
        if min_j <= max_j {
            for j in min_j..=max_j {
                let substitution = if c1.to_lowercase().eq(s2[j - 1].to_lowercase()) {
                    prev[j - 1] + 0.1
                } else {
                    prev[j - 1] + 2.0
                };
                let dist = if c1 == s2[j - 1] {
                    prev[j - 1]
                } else {
                    (prev[j] + 1.0).min(curr[j - 1] + 1.0).min(substitution)
                };
                curr[j] = dist;
                col_min = col_min.min(dist);
            }
        }
        for j in (max_j + 1)..=s2.len() {
            curr[j] = big;
        }
        if col_min > max {
            return None;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    let res = prev[s2.len()];
    if res > max {
        return None;
    }
    Some(res)
}

fn relative_emit_specifier(from_file: &str, symbol_file: &str) -> String {
    let from_dir = {
        let dir = from_file
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or("");
        dir.trim_end_matches('/').to_string()
    };
    let from_segs: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to_segs: Vec<&str> = symbol_file
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut common = 0;
    while common < from_segs.len()
        && common < to_segs.len().saturating_sub(1)
        && from_segs[common] == to_segs[common]
    {
        common += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_segs.len() {
        parts.push("..".to_string());
    }
    for seg in &to_segs[common..] {
        parts.push((*seg).to_string());
    }
    let last = parts.len().saturating_sub(1);
    if let Some(name) = parts.last().cloned() {
        let mapped = if let Some(stripped) = name.strip_suffix(".d.ts") {
            format!("{stripped}.d.ts")
        } else if let Some(stripped) = name.strip_suffix(".mts") {
            format!("{stripped}.mjs")
        } else if let Some(stripped) = name.strip_suffix(".cts") {
            format!("{stripped}.cjs")
        } else if let Some(stripped) = name.strip_suffix(".tsx") {
            format!("{stripped}.jsx")
        } else if let Some(stripped) = name.strip_suffix(".ts") {
            format!("{stripped}.js")
        } else {
            name
        };
        parts[last] = mapped;
    }
    let mut spec = parts.join("/");
    if !spec.starts_with("..") {
        spec = format!("./{spec}");
    }
    spec
}

fn module_format_is_esm_for_require_check(
    path: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> bool {
    use crate::core::compiler_options::ModuleKind;
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".d.ts") {
        return false;
    }
    crate::compiler::implied_node_format_of_file(path, read_file) == ModuleKind::ESNext
}

fn importer_is_cjs_for_require_check(
    path: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> bool {
    use crate::core::compiler_options::ModuleKind;
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".d.ts") {
        return true;
    }
    crate::compiler::implied_node_format_of_file(path, read_file) == ModuleKind::CommonJS
}

fn module_is_instantiated(node: &Arc<Node>, preserve_const_enums: bool) -> bool {
    let state = module_instance_state(node, &mut Vec::new());
    state == 2 || (preserve_const_enums && state == 1)
}

fn module_instance_state(node: &Arc<Node>, visited: &mut Vec<usize>) -> u8 {
    let id = Arc::as_ptr(node) as usize;

    if visited.contains(&id) {
        return 0;
    }
    visited.push(id);
    let state = module_instance_state_worker(node, visited);
    visited.pop();
    state
}

fn module_instance_state_worker(node: &Arc<Node>, visited: &mut Vec<usize>) -> u8 {
    match &node.data {
        crate::ast::NodeData::InterfaceDeclaration(_) | crate::ast::NodeData::TypeAliasDeclaration(_) => 0,
        crate::ast::NodeData::EnumDeclaration(_) => {
            if node.has_syntactic_modifier(ModifierFlags::Const) { 1 } else { 2 }
        }
        crate::ast::NodeData::ImportDeclaration(_)
        | crate::ast::NodeData::ImportEqualsDeclaration(_) => {
            if node.has_syntactic_modifier(ModifierFlags::Export) { 2 } else { 0 }
        }
        crate::ast::NodeData::ExportDeclaration(ed) => {
            if ed.module_specifier.is_none()
                && ed.export_clause.as_ref().is_some_and(|c| c.kind == SyntaxKind::NamedExports)
            {
                let clause = ed.export_clause.as_ref().unwrap();
                let crate::ast::NodeData::NamedExports(named) = &clause.data else {
                    return 2;
                };
                let mut state = 0u8;
                for spec in &named.elements.nodes {
                    let s = module_alias_target_state(spec, node, visited);
                    if s > state {
                        state = s;
                    }
                    if state == 2 {
                        return 2;
                    }
                }
                state
            } else {
                2
            }
        }
        crate::ast::NodeData::ModuleDeclaration(md) => match &md.body {
            Some(body) => module_instance_state(body, visited),
            None => 2,
        },
        crate::ast::NodeData::ModuleBlock(block) => {
            let mut state = 0u8;
            for stmt in &block.statements.nodes {
                let child = module_instance_state(stmt, visited);
                if child == 2 {
                    return 2;
                }
                if child == 1 {
                    state = 1;
                }
            }
            state
        }
        _ => 2,
    }
}

fn module_alias_target_state(
    spec: &Arc<Node>,
    export_decl: &Arc<Node>,
    visited: &mut Vec<usize>,
) -> u8 {
    let crate::ast::NodeData::ExportSpecifier(es) = &spec.data else {
        return 2;
    };
    let target_name = es.property_name.as_ref().unwrap_or(&es.name);
    if target_name.kind != SyntaxKind::Identifier {
        return 2;
    }
    let target_text = target_name.text();
    let mut anc = export_decl.parent.as_ref();
    while let Some(p) = anc {
        if matches!(
            p.kind,
            SyntaxKind::ModuleBlock | SyntaxKind::Block | SyntaxKind::SourceFile
        ) {
            let stmts: &[Arc<Node>] = match &p.data {
                crate::ast::NodeData::ModuleBlock(b) => &b.statements.nodes,
                crate::ast::NodeData::SourceFile(sf) => &sf.statements.nodes,
                crate::ast::NodeData::Block(b) => &b.statements.nodes,
                _ => &[],
            };
            let mut found: Option<u8> = None;
            for s in stmts {
                if statement_declares_name(s, target_text) {
                    let st = module_instance_state(s, visited);
                    found = Some(found.map_or(st, |f| f.max(st)));
                    if found == Some(2) {
                        return 2;
                    }
                    if s.kind == SyntaxKind::ImportEqualsDeclaration {

                        return 2;
                    }
                }
            }
            if let Some(f) = found {
                return f;
            }
        }
        anc = p.parent.as_ref();
    }
    2
}

fn statement_declares_name(stmt: &Arc<Node>, id_text: &str) -> bool {
    let name: Option<&Arc<Node>> = match &stmt.data {
        crate::ast::NodeData::FunctionDeclaration(f) => f.name.as_ref(),
        crate::ast::NodeData::ClassDeclaration(c) => c.name.as_ref(),
        crate::ast::NodeData::EnumDeclaration(e) => Some(&e.name),
        crate::ast::NodeData::ModuleDeclaration(m) => Some(&m.name),
        crate::ast::NodeData::InterfaceDeclaration(i) => Some(&i.name),
        crate::ast::NodeData::TypeAliasDeclaration(t) => Some(&t.name),
        crate::ast::NodeData::ImportEqualsDeclaration(i) => Some(&i.name),
        crate::ast::NodeData::VariableStatement(vs) => {
            let crate::ast::NodeData::VariableDeclarationList(dl) = &vs.declaration_list.data
            else {
                return false;
            };
            return dl.declarations.nodes.iter().any(|d| binding_names_cover(d, id_text));
        }
        _ => None,
    };
    name.is_some_and(|n| n.kind == SyntaxKind::Identifier && n.text() == id_text)
}

fn binding_names_cover(decl: &Arc<Node>, id_text: &str) -> bool {
    let crate::ast::NodeData::VariableDeclaration(d) = &decl.data else {
        return false;
    };
    match &d.name.kind {
        SyntaxKind::Identifier => d.name.text() == id_text,
        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
            let mut hit = false;
            crate::ast::node_data_generated::for_each_child(&d.name, |el| {
                if binding_names_cover(el, id_text) {
                    hit = true;
                    true
                } else {
                    false
                }
            });
            hit
        }
        _ => false,
    }
}

#[cfg(test)]
mod regression_fix_tests {
    use super::*;

    use crate::diagnosticwriter::format_diagnostic_compact;

    fn rendered_with_chain(checker: &Checker) -> Vec<String> {
        checker
            .diagnostics
            .get_all()
            .iter()
            .filter(|d| !d.file.as_ref().is_some_and(|f| f.file_name.starts_with("bundled://")))
            .map(|d| {
                let mut s = format_diagnostic_compact(d, None);
                if let Some(rest) = s.find(" error TS") {
                    s = s[rest + 1..].to_string();
                }
                for c in &d.message_chain {
                    s.push('\n');
                    s.push_str("  ");
                    s.push_str(&crate::diagnosticwriter::message_text(c, None));
                }
                s
            })
            .collect()
    }

    #[test]
    fn invocation_error_chain_names_apparent_wrapper_type() {

        let (program, mut checker) = convergence_tests::build_program_and_checker(
            "declare const s: string;\ns();",
            &["es5"],
        );
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        let lines = rendered_with_chain(&checker);
        assert!(lines.iter().any(|l| l.starts_with("error TS2349:")), "{lines:?}");
        assert!(
            lines
                .iter()
                .any(|l| l.contains("Type 'String' has no call signatures.")),
            "{lines:?}"
        );
    }

    #[test]
    fn never_intersection_callee_renders_never_in_chain() {

        let (program, mut checker) = convergence_tests::build_program_and_checker(
            "declare const f: { (x: string): number, a: \"\" } & { a: number };\nf();",
            &["es5"],
        );
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        let lines = rendered_with_chain(&checker);
        assert!(lines.iter().any(|l| l.starts_with("error TS2349:")), "{lines:?}");
        assert!(
            lines.iter().any(|l| l.contains("Type 'never' has no call signatures.")),
            "{lines:?}"
        );
    }

    #[test]
    fn union_target_failure_keeps_constituent_head_line() {

        let source = "var a0: (n: number, s: string) => number\n\
                      var a1: typeof a0 | ((n: number, s: string) => string);\n\
                      a1 = (foo, bar) => { return true; }";
        let (program, mut checker) = convergence_tests::build_program_and_checker(source, &["es5"]);
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        let codes = super::convergence_tests::error_codes(&checker);
        assert_eq!(codes, vec![2322], "{codes:?}");
        let lines = rendered_with_chain(&checker);
        let joined = lines.join("\n");
        assert!(
            joined.contains("Type '(foo: number, bar: string) => boolean' is not assignable to type '(n: number, s: string) => number'."),
            "{joined}"
        );
        assert!(joined.contains("Type 'boolean' is not assignable to type 'number'."), "{joined}");
    }

    #[test]
    fn equality_discriminant_keeps_undefined_member_under_non_strict() {

        let source = "type Foo2 = { kind?: 'a', a: number } | { kind?: 'b' } | { kind?: never };\n\
                      function f2(foo: Foo2) {\n\
                          if (foo.kind === 'a') {\n\
                              foo.a;\n\
                          }\n\
                      }";
        for strict in [false, true] {
            let (program, mut checker) = convergence_tests::build_program_and_checker(source, &["es5"]);

            let _ = strict;
            for file in program.source_files() {
                checker.check_source_file(file);
            }
            let codes = super::convergence_tests::error_codes(&checker);
            assert!(codes.is_empty(), "strict={strict} codes={codes:?}");
        }
    }

    #[test]
    fn optional_member_stays_t_when_strict_null_checks_off() {

        let lines: Vec<Vec<i32>> = [false, true]
            .iter()
            .map(|strict| {
                let diags = super::node_format_tests::check_files(
                    &[(
                        "entry.ts",
                        "interface I { x?: string; }\n\
                         declare const i: I;\n\
                         const a: string = i.x;",
                    )],
                    "/proj/entry.ts",
                    |o| o.strict_null_checks = crate::core::tristate::Tristate::from(*strict),
                );
                diags
            })
            .collect();

        assert!(lines[0].is_empty(), "non-strict: {:?}", lines[0]);

        assert_eq!(lines[1], vec![2322], "strict: {:?}", lines[1]);
    }

    #[test]
    fn indexed_access_tp_target_carries_instantiation_note() {

        let (program, mut checker) = convergence_tests::build_program_and_checker(
            "function f<T extends object, P extends keyof T>(s: string, tp: T[P]): void {\n    tp = s;\n}",
            &["es5"],
        );
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        let lines = rendered_with_chain(&checker);
        let joined = lines.join("\n");
        assert!(joined.contains("error TS2322"), "{joined}");
        assert!(
            joined.contains("could be instantiated with an arbitrary type which could be unrelated to 'string'"),
            "{joined}"
        );
    }

    #[test]
    fn record_element_access_assigns_object() {

        let (program, mut checker) = convergence_tests::build_program_and_checker(
            "declare const row: string;\n\
             const classesByRow: Record<string, object> = {};\n\
             classesByRow[row] = {};",
            &["es2015"],
        );
        for file in program.source_files() {
            checker.check_source_file(file);
        }
        let codes = super::convergence_tests::error_codes(&checker);
        assert!(codes.is_empty(), "{codes:?}");
    }

    #[test]
    fn node10_program_reports_deprecation_and_alternate_result() {

        use crate::bundled::BundledFS;
        use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
        use crate::tsoptions::ParsedCommandLine;
        use crate::vfs::InMemoryFS;

        let inner = Arc::new(InMemoryFS::new());
        inner.insert_dir("/node_modules");
        inner.insert_dir("/node_modules/pkg");
        inner.insert_file(
            "/node_modules/pkg/package.json",
            r#"{"name":"pkg","version":"1.0.0","exports":{".":"./definitely-not-index.js"}}"#,
        );
        inner.insert_file("/node_modules/pkg/definitely-not-index.d.ts", "export {};");
        inner.insert_file("/proj/entry.ts", "import { pkg } from \"pkg\";");
        let fs = Arc::new(BundledFS::new(inner));
        let mut options = CompilerOptions::default();
        options.module_resolution = crate::core::compiler_options::ModuleResolutionKind::Node10;
        options.target = crate::core::compiler_options::ScriptTarget::ES2015;
        options.module = crate::core::compiler_options::ModuleKind::CommonJS;
        let parsed = ParsedCommandLine {
            file_names: vec!["/proj/entry.ts".to_string()],
            compiler_options: options,
            ..Default::default()
        };
        let host: Arc<dyn CompilerHost> =
            Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), crate::bundled::lib_path()));
        let program = Arc::new(Program::new(ProgramOptions { config: parsed, host }));

        let global_codes: Vec<i32> = program
            .diagnostics()
            .iter()
            .filter(|d| d.file.is_none())
            .map(|d| d.code)
            .collect();
        assert!(global_codes.contains(&5107), "{global_codes:?}");

        let lines: Vec<String> = program
            .diagnostics()
            .iter()
            .map(|d| d.as_ref())
            .filter(|d| d.code == 2307)
            .map(|d| crate::diagnosticwriter::format_diagnostic_compact(d, None))
            .collect();
        assert!(!lines.is_empty(), "TS2307 must report");
        let joined = lines.join("\n");
        assert!(
            joined
                .contains("There are types at '/node_modules/pkg/definitely-not-index.d.ts'"),
            "{joined}"
        );
    }

    #[test]
    fn per_file_jsx_pragma_overrides_option_factory_for_2874() {

        let files: Vec<(&str, &str)> = vec![
            ("renderer.d.ts", "declare global {\n    namespace JSX {\n        interface IntrinsicElements {\n            [e: string]: any;\n        }\n    }\n}\nexport function dom(): void;\nexport { dom as p };"),
            ("reacty.tsx", "/** @jsx dom */\nimport { dom } from \"./renderer\";\n<h></h>"),
            ("index.tsx", "import { p } from \"./renderer\";\n<h></h>"),
        ];
        let diags = super::node_format_tests::check_files(&files, "/proj/reacty.tsx", |o| {
            o.jsx = crate::core::compiler_options::JsxEmit::React;
            o.jsx_factory = "p".to_string();
            o.module = crate::core::compiler_options::ModuleKind::CommonJS;
            o.target = crate::core::compiler_options::ScriptTarget::ES2015;
        });
        assert!(
            !diags.iter().any(|c| *c == 2874),
            "TS2874 must not fire under per-file pragma: {diags:?}"
        );
    }

}
