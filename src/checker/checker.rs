use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::ast::{
    CheckFlags, DiagnosticsCollection, ModifierFlags, Node, NodeData,
    NodeFlags, NodeList, NodeSymbolMap, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::core::text::TextRange;
use crate::diagnostics::messages_generated::{
    A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
    A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER, ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1,
    BLOCK_SCOPED_VARIABLE_0_USED_BEFORE_ITS_DECLARATION,
    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
    CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS, CANNOT_FIND_NAME_0,
    EXPECTED_0_ARGUMENTS_BUT_GOT_1, EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1,
    FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
    OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
    PROPERTY_0_HAS_NO_INITIALIZER_AND_IS_NOT_DEFINITELY_ASSIGNED_IN_THE_CONSTRUCTOR,
    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
    PROPERTY_0_IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1,
    THIS_COMPARISON_APPEARS_TO_BE_UNINTENTIONAL_BECAUSE_THE_TYPES_0_AND_1_HAVE_NO_OVERLAP,
    THIS_EXPRESSION_IS_NOT_CALLABLE, THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE,
    EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_CONSTRUCT_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER,
    EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER,
    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CALLABLE,
    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CONSTRUCTABLE,
    NO_CONSTITUENT_OF_TYPE_0_IS_CALLABLE,
    NO_CONSTITUENT_OF_TYPE_0_IS_CONSTRUCTABLE,
    TYPE_0_HAS_NO_CALL_SIGNATURES,
    TYPE_0_HAS_NO_CONSTRUCT_SIGNATURES,
    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1, UNREACHABLE_CODE_DETECTED,
    VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED,
};
use crate::evaluator::{EvalResult, EvalValue};
use crate::jsnum;

use super::tracer::Tracer;
use super::types::*;
use super::relater::RelaterChainEntry;
use super::utilities::is_in_compound_like_assignment;
use super::utilities::{get_assignment_target_kind, AssignmentKind};

use super::inference::{InferenceContext, InferenceInfo};

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

    pub degraded_type_ptrs: std::collections::HashSet<usize>,

    pub jsx_implicit_namespace: HashMap<usize, Option<Arc<Symbol>>>,

    pub pending_jsx_2875: Option<(crate::core::text::TextRange, String)>,

    pub relater_error_chain: Vec<RelaterChainEntry>,

    pub relater_chain_active: bool,

    pub relater_depth: u32,

    pub deferred_constraint_depth: u32,

    pub relation_count: u32,

    pub relater_overflow: bool,

    pub relater_intersection_target_depth: u32,

    pub subst_object_in_progress: std::collections::HashMap<usize, Arc<crate::checker::types::Type>>,

    pub in_return_substitution: bool,

    pub relater_source_stack: Vec<Arc<Type>>,

    pub relater_target_stack: Vec<Arc<Type>>,

    pub relation_cache: HashMap<crate::checker::relater::RelationCacheKey, bool>,

    pub probe_cache_permissive: HashMap<usize, Arc<Type>>,
    pub probe_cache_restrictive: HashMap<usize, Arc<Type>>,

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
                    id: 0,
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
            id: 0,
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
                    id: 0,
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
                id: 0,
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
            id: 0,
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

    fn check_unused_identifiers_in_file(&mut self, file_node: &Arc<Node>) {
        let no_locals = !self.compiler_options.no_unused_locals.is_true();
        let no_params = !self.compiler_options.no_unused_parameters.is_true();
        if no_locals && no_params {
            return;
        }
        let mut containers: Vec<Arc<Node>> = Vec::new();
        Self::collect_unused_check_containers(file_node, &mut containers);
        for container in containers {
            self.check_unused_locals_and_parameters(&container);
        }
    }

    fn collect_unused_check_containers(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
        use SyntaxKind::*;
        match node.kind {
            SourceFile | ModuleDeclaration | Block | CaseBlock | ForStatement
            | ForInStatement | ForOfStatement => out.push(Arc::clone(node)),
            Constructor | FunctionExpression | FunctionDeclaration | ArrowFunction
            | MethodDeclaration | GetAccessor | SetAccessor => {
                if Self::function_like_has_body(node) {
                    out.push(Arc::clone(node));
                }
            }
            _ => {}
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            Self::collect_unused_check_containers(child, out);
            false
        });
    }

    fn function_like_has_body(node: &Arc<Node>) -> bool {
        use crate::ast::NodeData;
        match &node.data {
            NodeData::ConstructorDeclaration(d) => d.body.is_some(),
            NodeData::FunctionDeclaration(d) => d.body.is_some(),

            NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => true,
            NodeData::MethodDeclaration(d) => d.body.is_some(),
            NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
            NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
            _ => false,
        }
    }

    fn check_unused_locals_and_parameters(&mut self, container: &Arc<Node>) {

        let Some(locals) = self.program.symbol_map().locals.get(&container.id()) else {
            return;
        };
        let locals: Vec<Arc<crate::ast::Symbol>> = locals.entries.values().cloned().collect();

        let mut variable_parents: Vec<(Arc<Node>, bool)> = Vec::new();

        let mut import_clauses: Vec<(Arc<Node>, Vec<Arc<Node>>)> = Vec::new();
        for local in locals {
            let reference_kinds = self
                .symbol_reference_kinds
                .get(&local.id())
                .map(|e| *e)
                .unwrap_or(SymbolFlags::empty());

            let variable_bits =
                SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable;
            let skip = if local.flags.contains(SymbolFlags::TypeParameter) {
                !local.flags.intersects(variable_bits)
                    || reference_kinds.intersects(variable_bits)
            } else {
                reference_kinds != SymbolFlags::empty()
                    || local.export_symbol.is_some()
                    || local.flags.contains(SymbolFlags::ModuleExports)
            };
            if skip {
                continue;
            }
            for declaration in &local.declarations {
                match declaration.kind {
                    SyntaxKind::VariableDeclaration
                    | SyntaxKind::Parameter
                    | SyntaxKind::BindingElement => {
                        if let Some(root) = Self::root_declaration(declaration) {
                            if let Some(parent) = root.parent.as_ref() {
                                if !variable_parents
                                    .iter()
                                    .any(|(n, _)| Arc::ptr_eq(n, parent))
                                {
                                    variable_parents.push((Arc::clone(parent), false));
                                }
                            }
                        }
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::NamespaceImport => {
                        if !Self::name_starts_with_underscore(declaration) {
                            let clause =
                                Self::import_clause_from_imported(declaration);
                            match import_clauses
                                .iter_mut()
                                .find(|(c, _)| Arc::ptr_eq(c, &clause))
                            {
                                Some((_, v)) => v.push(Arc::clone(declaration)),
                                None => import_clauses
                                    .push((clause, vec![Arc::clone(declaration)])),
                            }
                        }
                    }
                    _ => {
                        if declaration.kind != SyntaxKind::TypeParameter
                            && declaration.kind != SyntaxKind::ModuleDeclaration
                        {
                            let name = local.name.clone();
                            let is_type_decl =
                                matches!(declaration.kind, SyntaxKind::TypeAliasDeclaration
                                    | SyntaxKind::InterfaceDeclaration
                                    | SyntaxKind::ClassDeclaration
                                    | SyntaxKind::EnumDeclaration);
                            self.report_unused_local(declaration, &name, is_type_decl);
                        }
                    }
                }
            }
        }
        for (parent, _is_param) in variable_parents {
            if parent.kind == SyntaxKind::VariableDeclarationList {
                self.report_unused_variables(&parent);
            } else {
                self.report_unused_parameters(&parent);
            }
        }
        for (clause, unused) in import_clauses {
            self.report_unused_imports(&clause, &unused);
        }
    }

    fn root_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut cursor = Arc::clone(node);
        for _ in 0..100 {
            match cursor.kind {
                SyntaxKind::BindingElement => {
                    cursor = cursor.parent.as_ref()?.clone();
                }
                SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                    cursor = cursor.parent.as_ref()?.clone();
                }
                _ => return Some(cursor),
            }
        }
        None
    }

    fn name_starts_with_underscore(node: &Arc<Node>) -> bool {
        let text = node.text();
        !text.is_empty() && text.starts_with('_')
    }

    fn import_clause_from_imported(node: &Arc<Node>) -> Arc<Node> {
        match node.kind {
            SyntaxKind::ImportClause => Arc::clone(node),
            SyntaxKind::NamespaceImport => node
                .parent
                .clone()
                .unwrap_or_else(|| Arc::clone(node)),
            _ => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .unwrap_or_else(|| Arc::clone(node)),
        }
    }

    fn report_unused_local(&mut self, node: &Arc<Node>, name: &str, is_type_decl: bool) {
        let message: &'static crate::diagnostics::Message = if is_type_decl {
            &crate::diagnostics::messages_generated::X_0_IS_DECLARED_BUT_NEVER_USED
        } else {
            &crate::diagnostics::messages_generated::X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ
        };
        let loc = Self::name_or_node_loc(node);
        let is_param = node.kind == SyntaxKind::Parameter;
        self.report_unused(node, is_param, loc, message, vec![name.to_string()]);
    }

    fn report_unused_variables(&mut self, list: &Arc<Node>) {
        let declarations: Vec<Arc<Node>> = match &list.data {
            crate::ast::NodeData::VariableDeclarationList(d) => {
                d.declarations.iter().cloned().collect()
            }
            _ => return,
        };
        if declarations.len() > 1
            && declarations
                .iter()
                .all(|d| self.is_unreferenced_variable_declaration(d))
        {
            self.report_unused(
                list,
                false,
                list.loc,
                &crate::diagnostics::messages_generated::ALL_VARIABLES_ARE_UNUSED,
                vec![],
            );
        } else {
            self.report_unused_variable_declarations(&declarations);
        }
    }

    fn report_unused_parameters(&mut self, function: &Arc<Node>) {
        let parameters: Vec<Arc<Node>> = match &function.data {
            crate::ast::NodeData::ConstructorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::FunctionDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::FunctionExpression(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::ArrowFunction(d) => d.parameters.iter().cloned().collect(),
            crate::ast::NodeData::MethodDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::GetAccessorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::SetAccessorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            _ => return,
        };
        self.report_unused_variable_declarations(&parameters);
    }

    fn report_unused_variable_declarations(&mut self, declarations: &[Arc<Node>]) {
        for declaration in declarations {
            let (name_node, is_pattern) = match &declaration.data {
                crate::ast::NodeData::VariableDeclaration(d) => {
                    (Some(Arc::clone(&d.name)), Self::is_binding_pattern(&d.name))
                }
                crate::ast::NodeData::ParameterDeclaration(d) => {
                    (Some(Arc::clone(&d.name)), Self::is_binding_pattern(&d.name))
                }
                crate::ast::NodeData::BindingElement(d) => {
                    let n = d.name.clone();
                    let is_pattern = n
                        .as_ref()
                        .is_some_and(|n| Self::is_binding_pattern(n));
                    (n, is_pattern)
                }
                _ => continue,
            };
            let Some(name_node) = name_node else { continue };

            if declaration.kind == SyntaxKind::Parameter {
                if let crate::ast::NodeData::ParameterDeclaration(d) = &declaration.data {
                    if d.modifiers.as_ref().is_some_and(|m| {
                        m.modifier_flags.intersects(
                            ModifierFlags::Public
                                | ModifierFlags::Private
                                | ModifierFlags::Protected
                                | ModifierFlags::Readonly,
                        )
                    }) {
                        continue;
                    }
                    if name_node.kind == SyntaxKind::ThisKeyword {
                        continue;
                    }
                }
            }
            if is_pattern {
                self.report_unused_binding_elements(&name_node);
            } else if self.is_unreferenced_variable_declaration(declaration) {
                let name = name_node.text().to_string();
                self.report_unused(
                    declaration,
                    declaration.kind == SyntaxKind::Parameter,
                    name_node.loc,
                    &crate::diagnostics::messages_generated::
                        X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
                    vec![name],
                );
            }
        }
    }

    fn report_unused_binding_elements(&mut self, pattern: &Arc<Node>) {
        let elements: Vec<Arc<Node>> = match &pattern.data {
            crate::ast::NodeData::BindingPattern(d) => {
                d.elements.iter().cloned().collect()
            }
            _ => return,
        };
        if elements.len() > 1
            && elements
                .iter()
                .all(|e| self.is_unreferenced_variable_declaration(e))
        {
            self.report_unused(
                pattern,
                false,
                pattern.loc,
                &crate::diagnostics::messages_generated::ALL_DESTRUCTURED_ELEMENTS_ARE_UNUSED,
                vec![],
            );
        } else {
            self.report_unused_variable_declarations(&elements);
        }
    }

    fn is_unreferenced_variable_declaration(&self, node: &Arc<Node>) -> bool {
        let name_node = match &node.data {
            crate::ast::NodeData::VariableDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::ParameterDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::BindingElement(d) => d.name.clone(),
            _ => return true,
        };
        let Some(name_node) = name_node else {
            return true;
        };
        if Self::is_binding_pattern(&name_node) {
            let elements: Vec<Arc<Node>> = match &name_node.data {
                crate::ast::NodeData::BindingPattern(d) => {
                    d.elements.iter().cloned().collect()
                }
                _ => return true,
            };
            return elements
                .iter()
                .all(|e| self.is_unreferenced_variable_declaration(e));
        }

        if let Some(sym) = self
            .program
            .symbol_map()
            .symbol_of(node)
        {
            if let Some(kinds) = self.symbol_reference_kinds.get(&sym.id()) {
                if kinds
                    .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
                {
                    return false;
                }
            }
        }

        if node.kind == SyntaxKind::BindingElement {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::ObjectBindingPattern {
                    let elements: Vec<Arc<Node>> = match &parent.data {
                        crate::ast::NodeData::BindingPattern(d) => {
                            d.elements.iter().cloned().collect()
                        }
                        _ => Vec::new(),
                    };
                    let is_last = elements
                        .last()
                        .is_some_and(|last| Arc::ptr_eq(last, node));
                    let last_has_dots = elements.last().is_some_and(|last| {
                        matches!(&last.data,
                            crate::ast::NodeData::BindingElement(d) if d.dot_dot_dot_token.is_some())
                    });
                    let has_property_name = matches!(&node.data,
                        crate::ast::NodeData::BindingElement(d) if d.property_name.is_some());
                    if !is_last && last_has_dots && !has_property_name {
                        return false;
                    }
                }
            }
        }

        let underscore_exempt = match node.kind {
            SyntaxKind::Parameter => true,
            SyntaxKind::VariableDeclaration => {
                let mut in_for = false;
                if let Some(parent) = node.parent.as_ref() {
                    if parent.kind == SyntaxKind::VariableDeclarationList {
                        if let Some(gp) = parent.parent.as_ref() {
                            in_for = matches!(
                                gp.kind,
                                SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                            );
                        }
                    }
                }
                in_for || node.flags.contains(crate::ast::NodeFlags::Using)
            }
            SyntaxKind::BindingElement => {
                let parent_is_object_pattern = node
                    .parent
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ObjectBindingPattern);
                let has_property_name = matches!(&node.data,
                    crate::ast::NodeData::BindingElement(d) if d.property_name.is_some());
                !(parent_is_object_pattern && !has_property_name)
            }
            _ => false,
        };
        if underscore_exempt && Self::name_node_starts_with_underscore(&name_node) {
            return false;
        }
        true
    }

    fn name_node_starts_with_underscore(node: &Arc<Node>) -> bool {
        let text = node.text();
        !text.is_empty() && text.starts_with('_')
    }

    fn is_binding_pattern(node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        )
    }

    fn name_or_node_loc(node: &Arc<Node>) -> crate::core::text::TextRange {
        crate::ast::utilities::get_name_of_declaration(node)
            .map(|n| n.loc)
            .unwrap_or(node.loc)
    }

    fn report_unused_imports(&mut self, clause: &Arc<Node>, unused: &[Arc<Node>]) {
        let mut declaration_count = 0usize;
        let named_bindings: Option<Arc<Node>> = match &clause.data {
            crate::ast::NodeData::ImportClause(d) => {
                if d.name.is_some() {
                    declaration_count += 1;
                }
                d.named_bindings.clone()
            }
            _ => None,
        };
        if let Some(nb) = &named_bindings {
            if nb.kind == SyntaxKind::NamespaceImport {
                declaration_count += 1;
            } else {
                let elements: Vec<Arc<Node>> = match &nb.data {
                    crate::ast::NodeData::NamedImports(d) => {
                        d.elements.iter().cloned().collect()
                    }
                    _ => Vec::new(),
                };
                declaration_count += elements.len();
            }
        }
        if declaration_count > 1 && declaration_count == unused.len() {
            let loc = clause
                .parent
                .as_ref()
                .map(|p| p.loc)
                .unwrap_or(clause.loc);
            self.report_unused(
                clause,
                false,
                loc,
                &crate::diagnostics::messages_generated::
                    ALL_IMPORTS_IN_IMPORT_DECLARATION_ARE_UNUSED,
                vec![],
            );
        } else {
            for u in unused {
                let name = u.text().to_string();
                let is_type_decl = false;
                self.report_unused_local(u, &name, is_type_decl);
            }
        }
    }

    fn report_unused(
        &mut self,
        location: &Arc<Node>,
        is_parameter: bool,
        loc: crate::core::text::TextRange,
        message: &'static crate::diagnostics::Message,
        args: Vec<String>,
    ) {
        let ambient = location.flags.contains(crate::ast::NodeFlags::Ambient)
            || self.ambient_ancestor(location)
            || self
                .get_source_file_of_node(location)
                .is_some_and(|f| f.is_declaration_file);
        if ambient {
            return;
        }
        let is_error = if is_parameter {
            self.compiler_options.no_unused_parameters.is_true()
        } else {
            self.compiler_options.no_unused_locals.is_true()
        };
        if !is_error {
            return;
        }
        let file = self.current_file.clone();
        self.diagnostics
            .add(crate::ast::Diagnostic::new(file, loc, *message, args));
    }

    fn set_parent_pointers(&mut self, node: &Arc<Node>) {
        use crate::ast::node_data_generated::for_each_child;

        let mut children: Vec<Arc<Node>> = Vec::new();
        for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        let parent_clone = Arc::clone(node);
        for child in &children {
            let child_mut = Arc::as_ptr(child) as *mut Node;

            let already = unsafe {
                (*child_mut)
                    .parent
                    .as_ref()
                    .map_or(false, |p| Arc::ptr_eq(p, &parent_clone))
            };
            if !already {
                unsafe {
                    (*child_mut).parent = Some(Arc::clone(&parent_clone));
                }
            }
            self.set_parent_pointers(child);
        }
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

    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if symbol.flags.contains(SymbolFlags::Alias) {
            let target = self.follow_alias(symbol);
            if let Some(target) = target
                && !Arc::ptr_eq(&target, symbol)
            {
                let t = self.get_type_of_symbol(&target);
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return t;
            }
            return self.get_any_type();
        }

        if symbol.flags.contains(SymbolFlags::ValueModule)
            && (symbol.flags.contains(SymbolFlags::Function)
                || symbol.flags.contains(SymbolFlags::Class)
                || symbol.flags.contains(SymbolFlags::RegularEnum)
                || symbol.flags.contains(SymbolFlags::ConstEnum))
        {
            return self.get_type_of_merged_namespace_symbol(symbol);
        }

        if symbol.flags.contains(SymbolFlags::Prototype) {
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
            let result = self.get_type_of_prototype_property(symbol);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(result.clone());
            return result;
        }

        if symbol.flags.intersects(SymbolFlags::Method)
            && let Some(decl) = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::MethodDeclaration)
            && let crate::ast::NodeData::MethodDeclaration(data) = &decl.data
        {
            if let Some(links) = self.value_symbol_links.get(symbol)
                && let Some(ref t) = links.resolved_type
            {
                return Arc::clone(t);
            }
            self.push_scope(decl);
            let return_type = match data.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                &data.parameters,
                return_type,
                 false,
                 None,
                 Some(Arc::clone(decl)),
            );
            self.pop_scope();
            let t = self.create_function_or_constructor_type(vec![sig], false);
            self.value_symbol_links
                .get_or_default(symbol)
                .resolved_type = Some(Arc::clone(&t));
            return t;
        }

        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            || symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            || symbol.flags.contains(SymbolFlags::Function)
            || symbol.flags.contains(SymbolFlags::Class)
            || symbol.flags.contains(SymbolFlags::Property)
            || symbol.flags.contains(SymbolFlags::EnumMember)
        {

            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }

            if let Some(decl) = &symbol.value_declaration {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        return Arc::clone(t);
                    }
                }
            }

            for decl in &symbol.declarations {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        return Arc::clone(t);
                    }
                }
            }

            if let Some(t) = self.resolve_symbol_declared_type_on_demand(symbol) {
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return t;
            }
            self.get_any_type()
        } else if symbol.flags.contains(SymbolFlags::ValueModule) {

            self.resolve_namespace_type(symbol)
        } else if symbol.flags.intersects(SymbolFlags::ENUM) {

            self.resolve_enum_value_type(symbol)
        } else {
            self.get_any_type()
        }
    }

    fn attach_function_expando_type(
        &mut self,
        symbol: &Arc<crate::ast::Symbol>,
        base: Arc<Type>,
    ) -> Arc<Type> {
        let mut entries: Vec<(String, Arc<Node>)> = Vec::new();
        for (name, sym) in symbol.exports.iter() {
            if name == crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT {
                for d in &sym.declarations {

                    let mname = match &d.data {
                        crate::ast::NodeData::BinaryExpression(b) => match &b.left.data {
                            crate::ast::NodeData::ElementAccessExpression(eae) => self
                                .node_source_text(&eae.argument_expression)
                                .map(|t| format!("[{t}]"))
                                .unwrap_or_default(),
                            _ => String::new(),
                        },
                        _ => String::new(),
                    };
                    entries.push((mname, Arc::clone(d)));
                }
            } else if sym.flags.contains(SymbolFlags::Property)
                && !sym.declarations.is_empty()
                && sym
                    .declarations
                    .iter()
                    .all(|d| d.kind == SyntaxKind::BinaryExpression)
            {
                for d in &sym.declarations {
                    entries.push((name.clone(), Arc::clone(d)));
                }
            }
        }
        if entries.is_empty() {
            return base;
        }
        let mut table = crate::ast::SymbolTable::new();
        let mut props: Vec<Arc<crate::ast::Symbol>> = Vec::new();
        for (name, node) in entries {
            if table.entries.contains_key(&name) {
                continue;
            }
            let crate::ast::NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let rhs_type = self.with_declaring_file_context(&node, |c| {
                let t = c.get_type_of_node(&bin.right);
                c.get_widened_type(&t)
            });
            let prop = Arc::new(crate::ast::Symbol::new(
                SymbolFlags::Property,
                name.clone(),
            ));
            self.value_symbol_links.insert(
                &prop,
                ValueSymbolLinks {
                    resolved_type: Some(rhs_type),
                    ..Default::default()
                },
            );
            table.insert(name.clone(), Arc::clone(&prop));
            props.push(prop);
        }
        if props.is_empty() {
            return base;
        }
        let face = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        Arc::new(Type {
            flags: TypeFlags::Intersection,
            object_flags: ObjectFlags::None,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![base, face],
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn add_optional_undefined(&mut self, t: Arc<Type>) -> Arc<Type> {
        if !self.strict_null_checks {
            return t;
        }

        if t.flags.contains(TypeFlags::Any) && t.intrinsic_name() == Some("error") {
            return t;
        }
        let already = t.flags.contains(TypeFlags::Undefined)
            || (t.flags.contains(TypeFlags::Union)
                && t.types()
                    .is_some_and(|ts| ts.iter().any(|c| c.flags.contains(TypeFlags::Undefined))));
        if already {
            return t;
        }
        self.get_union_type(vec![t, self.undefined_type()])
    }

    pub(crate) fn strip_optional_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(ts) = t.types()
        {
            let kept: Vec<Arc<Type>> = ts
                .iter()
                .filter(|c| !c.flags.contains(TypeFlags::Undefined))
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() != ts.len() {
                return if kept.len() == 1 {
                    kept.into_iter().next().expect("nonempty")
                } else {
                    self.get_union_type(kept)
                };
            }
        }
        Arc::clone(t)
    }

    fn resolve_symbol_declared_type_on_demand(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        use crate::ast::NodeData;
        let decl = symbol
            .value_declaration
            .clone()
            .or_else(|| symbol.declarations.first().cloned())?;
        let type_node_and_init: (Option<Arc<Node>>, Option<Arc<Node>>) = match &decl.data {
            NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertySignatureDeclaration(d) => (Some(Arc::clone(&d.type_node)), None),
            NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            _ => return None,
        };
        if type_node_and_init.0.is_none() && type_node_and_init.1.is_none() {

            if decl.kind == SyntaxKind::VariableDeclaration {

                let placeholder = self.get_any_type();
                let existing = self
                    .value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type
                    .replace(placeholder);
                let t = self.initial_type_of_declaration(&decl);
                match &t {
                    Some(t) => {
                        self.value_symbol_links
                            .get_or_default(symbol)
                            .resolved_type = Some(Arc::clone(t));
                    }
                    None => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                    }
                }
                return t;
            }
            return None;
        }

        let placeholder = self.get_any_type();
        let existing = self
            .value_symbol_links
            .get_or_default(symbol)
            .resolved_type
            .replace(placeholder);
        let result = self.with_declaring_file_context(&decl, |checker| {
            let (type_node, initializer) = match &decl.data {
                NodeData::VariableDeclaration(d) => {
                    (d.type_node.clone(), d.initializer.clone())
                }
                NodeData::PropertyDeclaration(d) => {
                    (d.type_node.clone(), d.initializer.clone())
                }
                NodeData::PropertySignatureDeclaration(d) => {
                    (Some(Arc::clone(&d.type_node)), None)
                }
                NodeData::ParameterDeclaration(d) => {
                    (d.type_node.clone(), d.initializer.clone())
                }
                _ => (None, None),
            };
            if let Some(tn) = type_node {
                Some(checker.get_type_from_type_node(&tn))
            } else {

                let owner_class = match &decl.data {
                    NodeData::PropertyDeclaration(_) => decl
                        .parent
                        .as_ref()
                        .filter(|p| {
                            matches!(
                                p.kind,
                                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                            )
                        })
                        .cloned(),
                    _ => None,
                };
                if let Some(class) = owner_class.as_ref() {
                    let this_type = checker.build_class_instance_type_with_base(class);
                    checker.this_type_stack.push(this_type);
                }
                let t = initializer.map(|init| {

                    if !checker
                        .get_combined_node_flags(&decl)
                        .intersects(NodeFlags::Constant)
                        && matches!(
                            init.kind,
                            SyntaxKind::NullKeyword | SyntaxKind::UndefinedKeyword
                        )
                    {
                        return checker.auto_type();
                    }
                    if checker.is_empty_array_literal(&init) {
                        return checker.auto_array_type();
                    }
                    let raw = checker.get_type_of_node(&init);
                    let widened_literal =
                        checker.get_widened_literal_type_for_initializer(&decl, &raw);
                    let regularized =
                        checker.get_regular_type_of_literal_type(&widened_literal);
                    checker.widen_initializer_type(&regularized)
                });
                if owner_class.is_some() {
                    checker.this_type_stack.pop();
                }
                t
            }
        });

        let result = match (&result, &decl.data) {

            (Some(t), NodeData::ParameterDeclaration(pd))
                if pd.question_token.is_some() && pd.initializer.is_none() =>
            {
                Some(self.add_optional_undefined(Arc::clone(t)))
            }

            (Some(t), NodeData::PropertySignatureDeclaration(psd))
                if psd.postfix_token.as_ref().is_some_and(|tk| {
                    tk.kind == SyntaxKind::QuestionToken
                }) =>
            {
                Some(self.get_optional_type(Arc::clone(t)))
            }
            _ => result,
        };
        match &result {
            Some(t) => {
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(t));
            }
            None => {
                self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
            }
        }
        result
    }

    fn with_declaring_file_context<T>(
        &mut self,
        decl: &Arc<Node>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved_file = self.current_file.take();
        let saved_id = self.current_file_id;
        let saved_symbol = self.current_file_symbol.take();
        let mut pushed = 0usize;
        if let Some(file) = self.get_source_file_of_node(decl) {
            self.current_file = Some(Arc::clone(&file));
            self.current_file_id = file.node.id();
            self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();

            let mut chain: Vec<Arc<Node>> = Vec::new();
            let mut cur = decl.parent.clone();
            while let Some(n) = cur {
                if matches!(
                    n.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::ModuleDeclaration
                        | SyntaxKind::Block
                        | SyntaxKind::CatchClause
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::MethodSignature
                        | SyntaxKind::CallSignature
                        | SyntaxKind::ConstructSignature
                        | SyntaxKind::FunctionType
                        | SyntaxKind::ConstructorType
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::InterfaceDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::TypeAliasDeclaration
                        | SyntaxKind::MappedType
                        | SyntaxKind::EnumDeclaration
                ) {
                    chain.push(Arc::clone(&n));
                    if n.kind == SyntaxKind::SourceFile {
                        break;
                    }
                }
                cur = n.parent.clone();
            }
            for scope in chain.iter().rev() {
                self.push_scope(scope);
                pushed += 1;
            }
        }
        let result = f(self);
        for _ in 0..pushed {
            self.pop_scope();
        }
        self.current_file = saved_file;
        self.current_file_id = saved_id;
        self.current_file_symbol = saved_symbol;
        result
    }

    fn get_type_of_merged_namespace_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(cached) = self
            .declared_type_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let value_type = self.get_value_type_of_symbol(symbol);

        let ns_type = self.resolve_namespace_type(symbol);

        let (call_sigs, construct_sigs) = match &value_type.data {
            TypeData::Object(obj) => {
                let cs = obj.structured.call_signatures().to_vec();
                let xs = obj.structured.construct_signatures().to_vec();
                (cs, xs)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let merged = if call_sigs.is_empty() && construct_sigs.is_empty() {

            ns_type
        } else {

            let ns_obj = match &ns_type.data {
                TypeData::Object(obj) => obj,
                _ => {

                    self.declared_type_links.get_or_default(symbol).declared_type =
                        Some(Arc::clone(&value_type));
                    return value_type;
                }
            };
            let ns_structured = &ns_obj.structured;
            let mut structured = StructuredTypeData::default();
            structured.members = ns_structured.members.clone();
            structured.properties = ns_structured.properties.clone();
            structured.index_infos = ns_structured.index_infos.clone();

            let existing_sigs = ns_structured.signatures.clone();
            let existing_call_count = ns_structured.call_signature_count;
            structured.call_signature_count = call_sigs.len() + existing_call_count;
            structured.signatures = call_sigs;
            structured
                .signatures
                .extend(existing_sigs[..existing_call_count].to_vec());
            structured.signatures.extend(construct_sigs);
            structured
                .signatures
                .extend(existing_sigs[existing_call_count..].to_vec());
            Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: ObjectFlags::Anonymous,
                id: 0,
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::Object(ObjectTypeData {
                    structured,
                    target: None,
                    mapper: None,
                    type_arguments: Vec::new(),
                }),
            })
        };

        self.declared_type_links.get_or_default(symbol).declared_type = Some(Arc::clone(&merged));
        merged
    }

    fn get_value_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        if let Some(decl) = &symbol.value_declaration {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }

        for decl in &symbol.declarations {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }
        self.get_any_type()
    }

    fn resolve_enum_value_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        let _ = self.resolve_enum_type(symbol);

        let members: Vec<(String, Arc<Symbol>)> = symbol
            .members
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in &members {
            if name.starts_with("\u{FE}") {
                continue;
            }

            let _ = self.get_type_of_symbol(member_sym);
            symbol_table.insert(name.clone(), Arc::clone(member_sym));
            props.push(Arc::clone(member_sym));
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    constrained: ConstrainedTypeData::default(),
                    members: symbol_table,
                    properties: props,
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    index_infos: Vec::new(),
                    object_type_without_abstract_construct_signatures: std::sync::OnceLock::new(),
                },
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        });
        self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&result));
        result
    }

    fn get_type_of_function_like(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (parameters, body, type_node) = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => {
                (&data.parameters, Some(&data.body), data.type_node.as_ref())
            }
            crate::ast::NodeData::ArrowFunction(data) => {
                (&data.parameters, Some(&data.body), data.type_node.as_ref())
            }
            crate::ast::NodeData::FunctionDeclaration(data) => (
                &data.parameters,
                data.body.as_ref(),
                data.type_node.as_ref(),
            ),
            _ => return self.get_any_type(),
        };

        let contextual_signature: Option<Arc<Signature>> = self
            .get_contextual_signature(node)
            .or_else(|| self.iife_contextual_signature(node));
        let contextual_signature = contextual_signature.as_ref();

        let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
        if is_arrow {
            self.push_arrow_function_scope(node);
        } else {
            self.push_function_scope(node);
        }

        let placeholder = self.get_any_type();
        let _primed = self.build_signature_from_function_like_type_node(
            parameters,
            placeholder,
             false,
            contextual_signature,
             None,
        );

        let return_type = self.infer_function_return_type(body, type_node);
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }

        let sig = self.build_signature_from_function_like_type_node(
            parameters,
            return_type,
             false,
            contextual_signature,
             Some(Arc::clone(node)),
        );

        if !sig.type_parameters.is_empty() && let Some(contextual) = contextual_signature {
            if contextual.type_parameters.is_empty() {
                let inst = self.instantiate_signature_in_context_of(&sig, contextual);
                return self.create_function_or_constructor_type(vec![inst], false);
            }
        }
        self.create_function_or_constructor_type(vec![sig], false)
    }

    fn build_overload_function_type(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {

        let fn_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::FunctionDeclaration)
            .cloned()
            .collect();
        if fn_decls.len() <= 1 {
            return None;
        }

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        for decl in &fn_decls {
            let has_body = match &decl.data {
                crate::ast::NodeData::FunctionDeclaration(data) => data.body.is_some(),
                _ => false,
            };
            if has_body {
                continue;
            }
            let (parameters, type_node) = match &decl.data {
                crate::ast::NodeData::FunctionDeclaration(data) => {
                    (&data.parameters, data.type_node.as_ref())
                }
                _ => continue,
            };

            self.push_scope(decl);
            let return_type = match type_node {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                parameters,
                return_type,
                 false,
                 None,
                 Some(Arc::clone(decl)),
            );
            self.pop_scope();
            signatures.push(sig);
        }
        if signatures.is_empty() {
            return None;
        }
        Some(self.create_function_or_constructor_type(signatures, false))
    }

    pub(crate) fn get_type_of_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let members = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
            _ => return self.get_any_type(),
        };

        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        let node_id = node.id();
        if self.class_type_resolution_stack.contains(&node_id) {
            return self.get_any_type();
        }
        self.class_type_resolution_stack.push(node_id);
        let result = self.build_type_of_class_declaration(node, &members);
        self.class_type_resolution_stack.pop();
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }

    pub(crate) fn build_type_of_class_declaration(
        &mut self,
        node: &Arc<Node>,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {

        self.push_scope(node);

        let instance_type = self.build_class_instance_type_with_base(node);
        let mut construct_sigs: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            if member.kind != SyntaxKind::Constructor {
                continue;
            }
            let params = match &member.data {
                crate::ast::NodeData::ConstructorDeclaration(data) => &data.parameters,
                _ => continue,
            };
            let sig = self.build_signature_from_function_like_type_node(
                params,
                Arc::clone(&instance_type),
                 true,
                 None,
                 Some(Arc::clone(member)),
            );
            construct_sigs.push(sig);
        }
        self.pop_scope();
        if construct_sigs.is_empty() {

            let mut inherited: Option<(Arc<Node>, Arc<Node>)> = None;
            let mut cursor = Arc::clone(node);

            for _ in 0..1000 {
                let Some((base_node, _)) = self.extends_base_of(&cursor) else {
                    break;
                };
                if Arc::ptr_eq(&base_node, &cursor) {
                    break;
                }
                if let crate::ast::NodeData::ClassDeclaration(data) = &base_node.data {
                    if let Some(ctor) = data.members.iter().find(|m| {
                        matches!(m.data, crate::ast::NodeData::ConstructorDeclaration(_))
                    }) {
                        inherited = Some((Arc::clone(ctor), Arc::clone(&base_node)));
                        break;
                    }
                }
                cursor = base_node;
            }
            if let Some((ctor_decl, _)) = inherited {
                if let crate::ast::NodeData::ConstructorDeclaration(data) = &ctor_decl.data {
                    let params = Arc::clone(&data.parameters);
                    let sig = self.build_signature_from_function_like_type_node(
                        &params,
                        Arc::clone(&instance_type),
                         true,
                        None,
                        Some(ctor_decl),
                    );
                    construct_sigs.push(sig);
                }
            }
        }
        if construct_sigs.is_empty() {
            let sig = self.build_signature_from_function_like_type_node(
                &Arc::new(NodeList::default()),
                Arc::clone(&instance_type),
                 true,
                None,
                None,
            );
            construct_sigs.push(sig);
        }

        if node.has_syntactic_modifier(ModifierFlags::Abstract) {
            construct_sigs = construct_sigs
                .into_iter()
                .map(|sig| {
                    let s = crate::checker::types::Signature {
                        id: sig.id,
                        flags: sig.flags
                            | crate::checker::types::SignatureFlags::Abstract,
                        min_argument_count: sig.min_argument_count,
                        resolved_min_argument_count: sig.resolved_min_argument_count,
                        declaration: sig.declaration.clone(),
                        type_parameters: sig.type_parameters.clone(),
                        parameters: sig.parameters.clone(),
                        this_parameter: sig.this_parameter.clone(),
                        resolved_return_type: std::sync::OnceLock::new(),
                        resolved_type_predicate: sig.resolved_type_predicate.clone(),
                        target: None,
                        mapper: sig.mapper.clone(),
                        isolated_signature_type: std::sync::OnceLock::new(),
                        instantiated_parameter_types: sig.instantiated_parameter_types.clone(),
                    };
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let _ = s.resolved_return_type.set(rt.clone());
                    }
                    if let Some(it) = sig.isolated_signature_type.get() {
                        let _ = s.isolated_signature_type.set(it.clone());
                    }
                    Arc::new(s)
                })
                .collect();
        }
        let ctor_type = self.create_function_or_constructor_type(construct_sigs,  true);

        self.attach_class_statics(&ctor_type, node);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let t_mut = Arc::as_ptr(&ctor_type) as *mut crate::checker::types::Type;
            unsafe {
                (*t_mut).symbol = Some(Arc::clone(class_sym));
            }
        }
        ctor_type
    }

    fn attach_class_statics(&mut self, ctor_type: &Arc<Type>, node: &Arc<Node>) {

        let node_id = node.id();
        if self.class_statics_resolution_stack.contains(&node_id)
            || self.class_statics_resolution_stack.len() >= 200
        {
            return;
        }
        self.class_statics_resolution_stack.push(node_id);
        let mut members = SymbolTable::new();
        let mut properties: Vec<Arc<Symbol>> = Vec::new();

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let mut statics: Vec<(String, Arc<Symbol>)> = Vec::new();
            for sym in class_sym.members.entries.values() {
                if sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for sym in class_sym.exports.entries.values() {

                if (sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                    || sym.flags.contains(SymbolFlags::Prototype))
                    && !statics.iter().any(|(n, _)| *n == sym.name)
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for (name, sym) in statics {
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        let class_members: Option<Arc<NodeList>> = match &node.data {
            crate::ast::NodeData::ClassDeclaration(d) => Some(Arc::clone(&d.members)),
            crate::ast::NodeData::ClassExpression(d) => Some(Arc::clone(&d.members)),
            _ => None,
        };
        if let Some(member_list) = class_members {
            for member in member_list.iter() {
                if !member.has_syntactic_modifier(ModifierFlags::Static) {
                    continue;
                }
                let Some(name_node) = member.name() else { continue };
                let name = name_node.text().to_string();
                if name.is_empty() || members.get(&name).is_some() {
                    continue;
                }
                let flags = match member.kind {
                    SyntaxKind::MethodDeclaration => SymbolFlags::Method,
                    SyntaxKind::GetAccessor => SymbolFlags::GetAccessor,
                    SyntaxKind::SetAccessor => SymbolFlags::SetAccessor,
                    _ => SymbolFlags::Property,
                };
                let mut sym = Symbol::new(flags, name.clone());
                sym.declarations.push(Arc::clone(member));
                let sym = Arc::new(sym);

                if let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data
                    && let Some(tn) = &pd.type_node
                {
                    let t = self.get_type_from_type_node(tn);
                    self.value_symbol_links.insert(
                        &sym,
                        crate::checker::types::ValueSymbolLinks {
                            resolved_type: Some(t),
                            ..Default::default()
                        },
                    );
                }
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        if let Some((base_node, _)) = self.extends_base_of(node) {
            let base_ctor = self.get_type_of_class_declaration(&base_node);
            if let Some(base_structured) = base_ctor.as_structured() {
                for (name, sym) in base_structured.members.iter() {
                    if members.get(name).is_none() {
                        members.insert(name.clone(), Arc::clone(sym));
                    }
                }
                for prop in &base_structured.properties {
                    let name = prop.name.clone();
                    if members.get(&name).is_some() && !properties.iter().any(|p| Arc::ptr_eq(p, prop)) {
                        properties.push(Arc::clone(prop));
                    }
                }
            }
        }
        self.class_statics_resolution_stack.pop();
        if members.is_empty() {
            return;
        }
        let t_mut = Arc::as_ptr(ctor_type) as *mut crate::checker::types::Type;
        unsafe {
            if let TypeData::Object(obj) = &mut (*t_mut).data {
                obj.structured.members = members;
                obj.structured.properties = properties;
            }
        }
    }

    fn extends_base_of(&self, class_node: &Arc<Node>) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let heritage = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            crate::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        let extends_expr = heritage?.iter().find_map(|clause| {
            if let crate::ast::NodeData::HeritageClause(hc) = &clause.data {
                if hc.token == SyntaxKind::ExtendsKeyword {
                    return hc.types.iter().next().cloned();
                }
            }
            None
        })?;
        let base_expr = match &extends_expr.data {
            crate::ast::NodeData::ExpressionWithTypeArguments(data) => Arc::clone(&data.expression),
            _ => return None,
        };
        if base_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let symbol = self.resolve_identifier(&base_expr)?;
        if !symbol.flags.contains(SymbolFlags::Class) {
            return None;
        }
        symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned()
            .map(|n| (n, symbol))
    }

    fn get_return_type_of_call_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let callee = match &node.data {
            crate::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(&callee.0);
        if let Some(structured) = callee_type.as_structured() {
            let signatures = structured.call_signatures();
            if signatures.is_empty() {
                return self.get_any_type();
            }

            let matching_idx = if signatures.len() == 1 {
                0
            } else {
                self.find_matching_signature(node, signatures, &callee.1)
            };
            let sig = &signatures[matching_idx];
            if let Some(rt) = self.get_return_type_of_signature(sig) {

                if !sig.type_parameters.is_empty() {
                    let args: Vec<Arc<Node>> = callee.1.iter().cloned().collect();
                    let inferred = self.infer_call_type_arguments(node, sig, &args);
                    self.in_return_substitution = true;
                    let r = self.substitute_infer_type_parameters(
                        &rt,
                        &sig.type_parameters,
                        &inferred,
                    );
                    self.in_return_substitution = false;
                    return r;
                }
                return rt;
            }

            return self.get_any_type();
        }
        self.get_any_type()
    }

    fn get_return_type_of_new_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (callee, args) = match &node.data {
            crate::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(callee);
        if let Some(structured) = callee_type.as_structured() {
            for sig in structured.construct_signatures() {
                if let Some(rt) = self.get_return_type_of_signature(sig) {

                    let rt = if !sig.type_parameters.is_empty() {
                        let arg_vec: Vec<Arc<Node>> = args.iter().cloned().collect();
                        let inferred = self.infer_call_type_arguments(node, sig, &arg_vec);
                        self.substitute_infer_type_parameters(
                            &rt,
                            &sig.type_parameters,
                            &inferred,
                        )
                    } else {
                        rt
                    };

                    if let crate::ast::NodeData::NewExpression(d) = &node.data
                        && let Some(type_args) = &d.type_arguments
                        && let Some(class_sym) = rt.symbol.clone()
                    {
                        let tps = self.declared_type_parameter_types(&class_sym);
                        let arg_types: Vec<Arc<Type>> = type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect();
                        if !tps.is_empty() && tps.len() == arg_types.len() {
                            return self.attach_explicit_type_arguments_cached(&rt, arg_types);
                        }
                    }
                    return rt;
                }
                return self.get_any_type();
            }
        }
        self.get_any_type()
    }

    fn get_type_of_binary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use crate::ast::SyntaxKind::*;
        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            match data.operator_token.kind {

                PlusToken => {
                    let lt = self.get_type_of_node(&data.left);
                    let rt = self.get_type_of_node(&data.right);
                    let string_like = |t: &Arc<Type>| {
                        t.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral)
                    };
                    if string_like(&lt) || string_like(&rt) {
                        self.string_type()
                    } else if lt.flags.contains(TypeFlags::Any)
                        || rt.flags.contains(TypeFlags::Any)
                    {
                        self.get_any_type()
                    } else {
                        self.number_type()
                    }
                }

                MinusToken
                | AsteriskToken
                | SlashToken
                | PercentToken
                | AsteriskAsteriskToken
                | LessThanLessThanToken
                | GreaterThanGreaterThanToken
                | GreaterThanGreaterThanGreaterThanToken
                | AmpersandToken
                | BarToken
                | CaretToken => self.number_type(),

                LessThanToken
                | GreaterThanToken
                | LessThanEqualsToken
                | GreaterThanEqualsToken
                | EqualsEqualsToken
                | ExclamationEqualsToken
                | EqualsEqualsEqualsToken
                | ExclamationEqualsEqualsToken
                | InKeyword
                | InstanceOfKeyword => self.boolean_type(),

                AmpersandAmpersandToken | BarBarToken | QuestionQuestionToken => {
                    self.get_type_of_node(&data.left)
                }

                CommaToken => self.get_type_of_node(&data.right),

                EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken
                | BarEqualsToken
                | CaretEqualsToken
                | BarBarEqualsToken
                | AmpersandAmpersandEqualsToken
                | QuestionQuestionEqualsToken => self.get_type_of_node(&data.right),
                _ => self.get_any_type(),
            }
        } else {
            self.get_any_type()
        }
    }

    fn get_type_of_property_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, name) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => (&data.expression, &data.name),
            _ => return self.get_any_type(),
        };

        if obj_expr.kind == SyntaxKind::Identifier
            && let Some(sym) = self.resolve_identifier(obj_expr)
        {
            let base = self.resolve_alias_base(sym);
            if base.flags.contains(SymbolFlags::ValueModule) {
                let name_text = name.text();
                let member = base
                    .exports
                    .get(name_text)
                    .or_else(|| base.members.get(name_text))
                    .cloned()
                    .or_else(|| self.ambient_namespace_local(&base, name_text));

                if member.is_none() && !self.ambient_namespace_locals_visible(&base) {
                    return self.error_type();
                }
                if let Some(member) = member {
                    if let Some(t) = self
                        .value_symbol_links
                        .get(&member)
                        .and_then(|l| l.resolved_type.clone())
                    {
                        return t;
                    }
                    for decl in &member.declarations {
                        match decl.kind {
                            SyntaxKind::FunctionDeclaration => {
                                return self.get_type_of_function_like(decl);
                            }
                            SyntaxKind::ClassDeclaration => {
                                return self.get_type_of_class_declaration(decl);
                            }

                            SyntaxKind::ImportEqualsDeclaration => {
                                let t = self.type_of_imported_symbol(&member);
                                let resolved = match t {
                                    Some(t)
                                        if !(t.flags.contains(TypeFlags::Any)
                                            && t.intrinsic_name() == Some("any")) =>
                                    {
                                        Some(t)
                                    }
                                    _ => {
                                        let base =
                                            self.resolve_alias_base(Arc::clone(&member));
                                        base.declarations
                                            .iter()
                                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                                            .map(|cd| self.get_type_of_class_declaration(cd))
                                    }
                                };
                                if let Some(t) = resolved {
                                    return t;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        let obj_type = self.get_type_of_node(obj_expr);

        if obj_type.intrinsic_name() == Some("error") {
            return self.error_type();
        }
        let name_text = name.text();

        if obj_type.is_union() {
            let parts: Vec<Arc<Type>> = self
                .constituent_types(&obj_type)
                .into_iter()
                .filter_map(|c| {
                    let sym = self.get_property_of_type(&c, &name_text)?;
                    if let Some(sub) = self.instantiate_array_member_type(&c, &sym) {
                        return Some(sub);
                    }
                    if c.as_object().is_some_and(|o| !o.type_arguments.is_empty()) {
                        return Some(self.substituted_member_type_of(&c, &sym));
                    }
                    Some(self.get_type_of_symbol(&sym))
                })
                .collect();
            if !parts.is_empty() {
                let t = if parts.len() == 1 {
                    parts.into_iter().next().expect("exactly one")
                } else {
                    self.get_union_type(parts)
                };
                return self.flow_type_of_access_expression(node, None, t);
            }
        }

        if (self.is_auto_array_type(&obj_type)
            || obj_type.object_flags.contains(ObjectFlags::EvolvingArray))
            && self.is_array_mutation_method(&name_text)
        {
            return self.get_any_type();
        }
        if let Some(sym) = self.get_property_of_type(&obj_type, &name_text) {

            if let Some(substituted) = self.instantiate_array_member_type(&obj_type, &sym) {
                return self.flow_type_of_access_expression(node, Some(&sym), substituted);
            }

            if obj_type
                .as_object()
                .is_some_and(|o| !o.type_arguments.is_empty())
            {
                let substituted = self.substituted_member_type_of(&obj_type, &sym);
                return self.flow_type_of_access_expression(node, Some(&sym), substituted);
            }
            let prop_type = self.get_type_of_symbol(&sym);
            return self.flow_type_of_access_expression(node, Some(&sym), prop_type);
        }

        if name_text == "length" && self.is_array_type(&obj_type) {
            return self.number_type();
        }
        self.get_any_type()
    }

    fn flow_type_of_access_expression(
        &mut self,
        node: &Arc<Node>,
        prop: Option<&Arc<Symbol>>,
        prop_type: Arc<Type>,
    ) -> Arc<Type> {
        if Self::is_definite_assignment_target(node) {
            return prop_type;
        }
        if let Some(prop) = prop {
            let eligible = prop
                .flags
                .intersects(SymbolFlags::VARIABLE | SymbolFlags::Property | SymbolFlags::ACCESSOR)
                || (prop.flags.contains(SymbolFlags::Method) && prop_type.is_union());
            if !eligible {
                return prop_type;
            }
        }
        self.get_flow_type_of_reference(node, &prop_type)
    }

    fn is_definite_assignment_target(node: &Arc<Node>) -> bool {
        let Some(parent) = &node.parent else {
            return false;
        };
        match &parent.data {
            NodeData::BinaryExpression(bin) => {
                Self::is_assignment_operator(bin.operator_token.kind)
                    && Arc::ptr_eq(&bin.left, node)
            }
            NodeData::PostfixUnaryExpression(unary) => {
                matches!(unary.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken)
                    && Arc::ptr_eq(&unary.operand, node)
            }
            NodeData::PrefixUnaryExpression(unary) => {
                matches!(unary.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken)
                    && Arc::ptr_eq(&unary.operand, node)
            }
            _ => false,
        }
    }

    fn is_assignment_operator(kind: crate::ast::SyntaxKind) -> bool {
        use crate::ast::SyntaxKind::*;
        matches!(
            kind,
            EqualsToken
                | PlusEqualsToken
                | MinusEqualsToken
                | AsteriskEqualsToken
                | SlashEqualsToken
                | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
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

    fn is_block_terminating_statement(stmt: &Arc<Node>) -> bool {
        matches!(
            stmt.kind,
            SyntaxKind::ReturnStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
        )
    }

    #[allow(dead_code)]
    fn property_type_includes_undefined(
        &mut self,
        data: &crate::ast::node_data_generated::PropertyDeclarationData,
    ) -> bool {
        let Some(tn) = &data.type_node else {
            return false;
        };
        let t = self.get_type_from_type_node(tn);
        if t.flags.contains(TypeFlags::Undefined) {
            return true;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u.types.iter().any(|m| m.flags.contains(TypeFlags::Undefined));
        }
        false
    }

    #[allow(dead_code)]
    fn class_constructor_assigns_property(&self, name: &str) -> bool {
        let Some(class) = self.enclosing_class_stack.last() else {
            return false;
        };
        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return false;
        };
        cd.members.iter().any(|member| {
            if member.kind != SyntaxKind::Constructor {
                return false;
            }
            let crate::ast::NodeData::ConstructorDeclaration(ctor) = &member.data else {
                return false;
            };
            ctor.body
                .as_ref()
                .is_some_and(|body| body_assigns_this_property(body, name))
        })
    }

    fn check_call_arg_with_context(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
        arg: &Arc<Node>,
    ) {
        let is_function_arg =
            matches!(arg.kind, SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression);
        if is_function_arg {
            let ctx = self.contextual_param_count_for_arg(callee_expr, arg_index);
            if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
                eprintln!("[ctx-arg] pushed ctx={ctx}");
            }
            self.call_arg_arrow_context.push(ctx);
        }
        self.check_expression(arg);
        if is_function_arg {
            self.call_arg_arrow_context.pop();
        }
    }

    fn contextual_signature_of_arrow(&mut self, node: &Arc<Node>) -> Option<Arc<Signature>> {
        if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
            eprintln!(
                "[arrow-ctx] entered parent={:?}",
                node.parent.as_ref().map(|p| p.kind)
            );
        }
        let t = self.get_contextual_type(node, ContextFlags::None)?;
        if let TypeData::IndexedAccess(ia) = &t.data
            && let (Some(o), Some(i)) = (&ia.object_type, &ia.index_type)
            && o.flags.contains(TypeFlags::TypeParameter)
        {
            let resolved = self.get_indexed_access_type(o, i);
            if !matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
                return self.first_call_signature(&resolved);
            }
        }
        self.first_call_signature(&t)
    }

    fn first_call_signature(&mut self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        if let TypeData::Union(u) = &t.data {
            for constituent in &u.union_or_intersection.types {
                if constituent
                    .flags
                    .intersects(TypeFlags::Undefined | TypeFlags::Null)
                {
                    continue;
                }
                if let Some(sig) = self.first_call_signature(constituent) {
                    return Some(sig);
                }
            }
            return None;
        }
        let structured = t.as_structured()?;
        structured.call_signatures().first().cloned()
    }

    fn contextual_param_count_for_arg(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
    ) -> usize {
        let t = self.get_type_of_node(callee_expr);
        if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
            eprintln!(
                "[ctx-arg] callee={:?} intr={:?} union={} structured={}",
                callee_expr.kind,
                t.intrinsic_name(),
                matches!(&t.data, TypeData::Union(_)),
                t.as_structured()
                    .map(|s| s.call_signatures().len())
                    .unwrap_or(usize::MAX),
            );
        }
        if t.flags.contains(TypeFlags::Any) {

            if let crate::ast::NodeData::PropertyAccessExpression(data) = &callee_expr.data {
                let method = data.name.text().to_string();
                const ARRAY_CALLBACK_SIGS: &[(&str, usize)] = &[
                    ("map", 3),
                    ("filter", 3),
                    ("forEach", 3),
                    ("every", 3),
                    ("some", 3),
                    ("find", 3),
                    ("findIndex", 3),
                    ("findLast", 3),
                    ("findLastIndex", 3),
                    ("flatMap", 3),
                    ("reduce", 4),
                    ("reduceRight", 4),
                    ("sort", 2),
                ];
                if let Some((_, count)) = ARRAY_CALLBACK_SIGS.iter().find(|(m, _)| *m == method) {
                    let recv_type = self.get_type_of_node(&data.expression);
                    if self.is_array_type(&recv_type) {
                        return *count;
                    }
                }
            }
            return 0;
        }

        let t = if let TypeData::Union(u) = &t.data {
            match u.union_or_intersection.types.iter().find(|c| {
                !c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null)
                    && c.as_structured()
                        .is_some_and(|s| !s.call_signatures().is_empty())
            }) {
                Some(c) => Arc::clone(c),
                None => return 0,
            }
        } else {
            t
        };
        let Some(structured) = t.as_structured() else {
            return 0;
        };

        let Some(sig) = structured
            .call_signatures()
            .iter()
            .find(|s| s.parameters.len() > arg_index)
            .or_else(|| structured.call_signatures().first())
        else {
            return 0;
        };
        let Some(param) = sig.parameters.get(arg_index) else {
            return 0;
        };
        let param_type = self.get_type_of_symbol(param);
        if param_type.flags.contains(TypeFlags::Any) {
            return 0;
        }
        let Some(param_structured) = param_type.as_structured() else {
            return 0;
        };
        param_structured
            .call_signatures()
            .first()
            .map_or(0, |callback_sig| callback_sig.parameters.len())
    }

    fn symbol_is_abstract_class(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if decl.kind == SyntaxKind::ClassDeclaration
                && decl.has_syntactic_modifier(ModifierFlags::Abstract)
            {
                return true;
            }
        }
        false
    }

    fn type_includes_abstract_constructor(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Any) {
            return false;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u.types.iter().any(|m| self.type_includes_abstract_constructor(m));
        }

        if t.flags.contains(TypeFlags::Object) {
            if let Some(s) = t.as_structured()
                && s.construct_signatures().iter().any(|sig| {
                    sig.flags.contains(crate::checker::types::SignatureFlags::Abstract)
                })
            {
                return true;
            }
        }
        if let Some(symbol) = &t.symbol {
            return self.symbol_is_abstract_class(symbol);
        }
        false
    }

    fn declaring_class_of_member(&self, member_symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
        self.declaring_class_of_private_member(member_symbol)
            .or_else(|| {
                for decl in &member_symbol.declarations {
                    if matches!(
                        decl.kind,
                        SyntaxKind::PropertyDeclaration | SyntaxKind::MethodDeclaration
                    ) {
                        if let Some(parent) = &decl.parent {
                            if parent.kind == SyntaxKind::ClassDeclaration {
                                return Some(Arc::clone(parent));
                            }
                        }
                    }
                }
                None
            })
    }

    fn declaring_class_of_private_member(
        &self,
        member_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        for decl in &member_symbol.declarations {
            if matches!(
                decl.kind,
                SyntaxKind::PropertyDeclaration
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                if let Some(parent) = &decl.parent {
                    if matches!(
                        parent.kind,
                        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                    ) {
                        return Some(Arc::clone(parent));
                    }
                }
            }
        }
        None
    }

    fn lookup_private_identifier_declaration(
        &self,
        text: &str,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        let mut current = Some(Arc::clone(location));
        while let Some(n) = current {
            if matches!(n.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
                if let Some(sym) = symbol_map.symbol_of(&n) {
                    if let Some(prop) = sym.members.get(text) {
                        return Some(Arc::clone(prop));
                    }
                    if let Some(prop) = sym.exports.get(text) {
                        return Some(Arc::clone(prop));
                    }
                }
            }
            current = n.parent.clone();
        }
        None
    }

    fn is_ancestor_class_of(&self, node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
        let mut current = Some(Arc::clone(node));
        while let Some(n) = current {
            if Arc::ptr_eq(&n, ancestor) {
                return true;
            }
            current = n.parent.clone();
        }
        false
    }

    fn check_private_identifier_access(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        name_text: &str,
        obj_type: &Arc<Type>,
    ) -> bool {
        let assignment_kind = crate::checker::utilities::get_assignment_target_kind(node);
        let lexical = self.lookup_private_identifier_declaration(name_text, name);

        if assignment_kind != crate::checker::utilities::AssignmentKind::None
            && let Some(lx) = &lexical
            && lx.declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::MethodDeclaration)
        {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    CANNOT_ASSIGN_TO_PRIVATE_METHOD_0_PRIVATE_METHODS_ARE_NOT_WRITABLE,
                vec![name_text.to_string()],
            ));
        }

        let type_member: Option<Arc<Symbol>> = obj_type
            .as_structured()
            .and_then(|s| s.members.get(name_text))
            .map(Arc::clone);
        let resolved = match (&lexical, &type_member) {
            (Some(lx), Some(m)) => {
                let same_decl = lx.declarations.iter().any(|ld| {
                    m.declarations.iter().any(|d| d.id() == ld.id())
                });
                let same_class = lx
                    .declarations
                    .first()
                    .and_then(|ld| ld.parent.clone())
                    .zip(m.declarations.first().and_then(|d| d.parent.clone()))
                    .is_some_and(|(a, b)| a.id() == b.id());
                let synthetic_same_class = m.declarations.is_empty()
                    && lx.declarations
                        .first()
                        .and_then(|d| d.parent.clone())
                        .and_then(|class| self.program.symbol_map().symbol_of(&class))
                        .zip(obj_type.symbol.clone())
                        .is_some_and(|(a, b)| Arc::ptr_eq(&a, &b));
                (same_decl || same_class || synthetic_same_class).then(|| Arc::clone(m))
            }
            _ => None,
        };

        if resolved.is_none() {

            let property_on_type = type_member.as_ref().filter(|m| {
                m.declarations.iter().any(|d| {
                    d.name()
                        .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
                })
            });
            if let Some(property) = property_on_type {
                let type_class = self.declaring_class_of_private_member(property);
                if let (Some(lx), Some(type_class)) = (&lexical, &type_class) {

                    let lexical_class = self.declaring_class_of_private_member(lx);
                    if lexical_class.is_some_and(|lc| self.is_ancestor_class_of(&lc, type_class))
                    {
                        let type_str = self.type_to_string(obj_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name.loc,
                            crate::diagnostics::messages_generated::THE_PROPERTY_0_CANNOT_BE_ACCESSED_ON_TYPE_1_WITHIN_THIS_CLASS_BECAUSE_IT_IS_SHADOWED_BY_ANOTHER_PRIVATE_IDENTIFIER_WITH_THE_SAME_SPELLING,
                            vec![name_text.to_string(), type_str],
                        ));
                        return true;
                    }
                }
                let class_name = type_class.map_or_else(
                    || "(anonymous)".to_string(),
                    |c| match &c.data {
                        crate::ast::NodeData::ClassDeclaration(d) => d
                            .name
                            .as_ref()
                            .map(|n| n.text().to_string())
                            .unwrap_or_else(|| "(anonymous)".to_string()),
                        _ => "(anonymous)".to_string(),
                    },
                );
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::PROPERTY_0_IS_NOT_ACCESSIBLE_OUTSIDE_CLASS_1_BECAUSE_IT_HAS_A_PRIVATE_IDENTIFIER,
                    vec![name_text.to_string(), class_name],
                ));
                return true;
            }
            return false;
        }

        let setonly = resolved.as_ref().is_some_and(|m| {
            m.flags.contains(SymbolFlags::SetAccessor) && !m.flags.contains(SymbolFlags::GetAccessor)
        });
        if setonly && assignment_kind != crate::checker::utilities::AssignmentKind::Definite {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                crate::diagnostics::messages_generated::PRIVATE_ACCESSOR_WAS_DEFINED_WITHOUT_A_GETTER,
                vec![],
            ));
        }
        false
    }

    fn is_within_declaring_class(&self, class_node: &Arc<Node>) -> bool {
        self.enclosing_class_stack
            .iter()
            .any(|c| Arc::ptr_eq(c, class_node))
    }

    fn super_in_computed_name_of_innermost_class(&self, node: &Arc<Node>) -> bool {
        let Some(innermost) = self.enclosing_class_stack.last() else {
            return false;
        };
        let mut in_computed_name = false;
        let mut cur = node.parent.as_ref();
        while let Some(c) = cur {
            if Arc::ptr_eq(c, innermost) {
                return in_computed_name;
            }
            if c.kind == SyntaxKind::ComputedPropertyName {
                in_computed_name = true;
            }
            if matches!(c.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {

                return false;
            }
            cur = c.parent.as_ref();
        }
        false
    }

    fn function_body_definitely_returns(&self, body: &Arc<Node>) -> bool {
        if body.kind != SyntaxKind::Block {
            return false;
        }
        if let crate::ast::NodeData::Block(data) = &body.data {
            if let Some(last) = data.statements.nodes.last() {
                return self.statement_always_returns(last);
            }
        }
        false
    }

    fn statement_always_returns(&self, stmt: &Arc<Node>) -> bool {
        match stmt.kind {
            SyntaxKind::ReturnStatement | SyntaxKind::ThrowStatement => true,
            SyntaxKind::Block => {
                if let crate::ast::NodeData::Block(data) = &stmt.data {
                    if let Some(last) = data.statements.nodes.last() {
                        return self.statement_always_returns(last);
                    }
                }
                false
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &stmt.data {
                    let then_returns = self.statement_always_returns(&data.then_statement);
                    let else_returns = data
                        .else_statement
                        .as_ref()
                        .map_or(false, |e| self.statement_always_returns(e));
                    then_returns && else_returns
                } else {
                    false
                }
            }

            SyntaxKind::WhileStatement | SyntaxKind::DoStatement => {
                let (condition, body) = match &stmt.data {
                    crate::ast::NodeData::WhileStatement(data) => {
                        (&data.expression, &data.statement)
                    }
                    crate::ast::NodeData::DoStatement(data) => {
                        (&data.expression, &data.statement)
                    }
                    _ => return false,
                };
                condition.kind == SyntaxKind::TrueKeyword
                    && !Self::loop_has_escaping_break(body, true)
            }

            SyntaxKind::ForStatement => {
                if let crate::ast::NodeData::ForStatement(data) = &stmt.data {
                    data.condition
                        .as_ref()
                        .map_or(true, |c| c.kind == SyntaxKind::TrueKeyword)
                        && !Self::loop_has_escaping_break(&data.statement, true)
                } else {
                    false
                }
            }

            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &stmt.data
                    && let crate::ast::NodeData::CaseBlock(block) = &data.case_block.data
                {
                    let has_default = block.clauses.iter().any(|c| {
                        c.kind == SyntaxKind::DefaultClause
                    });
                    if !has_default {
                        return false;
                    }
                    block.clauses.iter().all(|c| {
                        match &c.data {
                            crate::ast::NodeData::CaseOrDefaultClause(cd) => cd
                                .statements
                                .nodes
                                .last()
                                .map_or(true, |l| self.statement_always_returns(l)),
                            _ => false,
                        }
                    })
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn is_property_readonly(&self, t: &Arc<Type>, name: &str) -> bool {
        let Some(structured) = t.as_structured() else {
            return false;
        };
        let Some(symbol) = structured.members.get(name) else {
            return false;
        };

        for decl in &symbol.declarations {
            let modifiers = match &decl.data {
                crate::ast::NodeData::PropertyDeclaration(d) => &d.modifiers,
                crate::ast::NodeData::PropertySignatureDeclaration(d) => &d.modifiers,
                crate::ast::NodeData::ParameterDeclaration(d) => &d.modifiers,
                _ => continue,
            };
            if let Some(m) = modifiers {
                if m.modifier_flags.contains(ModifierFlags::Readonly) {
                    return true;
                }
            }
        }

        if symbol.check_flags.contains(CheckFlags::Readonly) {
            return true;
        }
        false
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

    pub fn is_array_mutation_method(&self, name: &str) -> bool {
        matches!(name, "push" | "unshift")
    }

    pub fn boxed_apparent_type_of_primitive(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        use crate::checker::types::TYPE_FLAGS_ENUM_LIKE;
        let name = if t.flags.intersects(
            TypeFlags::String | TypeFlags::StringLiteral | TypeFlags::Index | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
        ) {
            "String"
        } else if t.flags.intersects(
            TypeFlags::Number | TypeFlags::NumberLiteral | TypeFlags::EnumLiteral,
        ) || (t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) && !t.flags.intersects(TypeFlags::String))
        {
            "Number"
        } else if t.flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral) {
            "Boolean"
        } else if t.flags.intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral) {
            "BigInt"
        } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
            "Symbol"
        } else {
            return None;
        };
        if let Some(cached) = self.boxed_global_types.get(name) {
            return Some(Arc::clone(cached));
        }

        let mut matching: Vec<Arc<Node>> = Vec::new();
        for file in &self.files {
            let statements = match &file.node.data {
                NodeData::SourceFile(data) => &data.statements,
                _ => continue,
            };
            for stmt in statements.iter() {
                if let NodeData::InterfaceDeclaration(d) = &stmt.data
                    && d.name.text() == name
                {
                    matching.push(Arc::clone(stmt));
                }
            }
        }
        let mut all_members: Vec<Arc<Node>> = Vec::new();
        for stmt in &matching {
            let NodeData::InterfaceDeclaration(d) = &stmt.data else {
                continue;
            };
            all_members.extend(d.members.iter().cloned());

            self.collect_boxed_heritage_members(stmt, &mut all_members, &mut Vec::new(), 0);
        }
        if all_members.is_empty() {
            return None;
        }
        let members = Arc::new(crate::ast::NodeList::new(all_members));
        let built = self.build_interface_type_from_members(&members);
        self.boxed_global_types
            .insert(name.to_string(), Arc::clone(&built));
        Some(built)
    }

    fn collect_boxed_heritage_members(
        &mut self,
        iface_stmt: &Arc<Node>,
        out: &mut Vec<Arc<Node>>,
        visited: &mut Vec<*const Node>,
        depth: usize,
    ) {
        if depth >= 6 {
            return;
        }
        if visited.contains(&Arc::as_ptr(iface_stmt)) {
            return;
        }
        visited.push(Arc::as_ptr(iface_stmt));
        let heritage = match &iface_stmt.data {
            NodeData::InterfaceDeclaration(d) => d.heritage_clauses.clone(),
            _ => return,
        };
        let Some(heritage) = heritage else {
            return;
        };
        for clause in heritage.iter() {
            let NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            if hc.token != SyntaxKind::ExtendsKeyword {
                continue;
            }
            for type_ref in hc.types.iter() {
                let expr = match &type_ref.data {
                    NodeData::ExpressionWithTypeArguments(ewa) => Arc::clone(&ewa.expression),
                    _ => continue,
                };
                let base_decls: Vec<Arc<Node>> =
                    self.with_declaring_file_context(iface_stmt, |c| match expr.kind {
                        SyntaxKind::Identifier => c
                            .resolve_identifier(&expr)
                            .map(|s| s.declarations.clone())
                            .unwrap_or_default(),
                        _ => c
                            .resolve_qualified_symbol(&expr)
                            .map(|s| s.declarations.clone())
                            .unwrap_or_default(),
                    });
                for decl in base_decls {
                    if let NodeData::InterfaceDeclaration(bd) = &decl.data {
                        out.extend(bd.members.iter().cloned());
                        self.collect_boxed_heritage_members(&decl, out, visited, depth + 1);
                    }
                }
            }
        }
    }

    fn global_interface_has_property(&mut self, symbol_name: &str, prop_name: &str) -> bool {
        if !self.global_interface_members.contains_key(symbol_name) {
            let names = self.collect_global_interface_member_names(symbol_name);
            self.global_interface_members
                .insert(symbol_name.to_string(), names);
        }
        self.global_interface_members
            .get(symbol_name)
            .map(|names| names.iter().any(|n| n == prop_name))
            .unwrap_or(false)
    }

    fn collect_global_interface_member_names(&self, interface_name: &str) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for file in &self.files {
            let statements: Vec<Arc<Node>> = match &file.node.data {
                NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
                _ => continue,
            };
            for stmt in &statements {
                let members = match &stmt.data {
                    NodeData::InterfaceDeclaration(d) if d.name.text() == interface_name => {
                        &d.members
                    }
                    _ => continue,
                };
                for member in members.iter() {
                    let member_name = match &member.data {
                        NodeData::PropertySignatureDeclaration(d) => {
                            self.get_property_name_from_node(&d.name)
                        }
                        NodeData::MethodSignatureDeclaration(d) => {
                            self.get_property_name_from_node(&d.name)
                        }
                        _ => continue,
                    };
                    if !member_name.is_empty() && !names.iter().any(|n| n == &member_name) {
                        names.push(member_name);
                    }
                }
            }
        }
        names
    }

    fn check_property_access(&mut self, node: &Arc<Node>) {
        let (obj_expr, question_dot, name) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => (
                &data.expression,
                data.question_dot_token.is_some(),
                &data.name,
            ),
            _ => return,
        };
        let obj_type = self.get_type_of_node(obj_expr);
        let name_text = name.text();

        let lookup_type;
        if !question_dot
            && self.strict_null_checks
            && type_is_possibly_undefined(&obj_type)
        {
            self.report_possibly_null_or_undefined(obj_expr, &obj_type, false);
            lookup_type = self.get_non_nullable_type_of(&obj_type);
        } else {
            lookup_type = obj_type;
        }
        let obj_type = lookup_type;

        if name.kind == SyntaxKind::PrivateIdentifier
            && self.check_private_identifier_access(node, name, name_text, &obj_type)
        {
            return;
        }

        if let Some(structured) = obj_type.as_structured() {
            if let Some(member_symbol) = structured.members.get(name_text) {

                let in_ctor = self.in_ctor_body_stack.last() == Some(&true);
                let in_prop_init = !in_ctor && self.access_in_property_initializer(node);
                if obj_expr.kind == SyntaxKind::ThisKeyword
                    && (in_ctor || in_prop_init)
                    && let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
                        d.kind == SyntaxKind::PropertyDeclaration
                            && d.has_syntactic_modifier(ModifierFlags::Abstract)
                    })
                    && let Some(parent) = &abstract_decl.parent
                    && parent.kind == SyntaxKind::ClassDeclaration
                    && let Some(class_name) = class_declaration_name(parent)
                {
                    let file = self.current_file.clone();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        name.loc,
                        crate::diagnostics::messages_generated::
                            ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
                        vec![name_text.to_string(), class_name],
                    );
                    self.diagnostics.add(diagnostic);
                }

                if in_prop_init
                    && obj_expr.kind == SyntaxKind::ThisKeyword
                    && get_assignment_target_kind(node) == AssignmentKind::None
                    && let Some(prop_decl) = member_symbol.declarations.iter().find(|d| {
                        d.kind == SyntaxKind::PropertyDeclaration
                            && !d.has_syntactic_modifier(ModifierFlags::Static)
                    })
                {

                    let asserted = matches!(
                        &prop_decl.data,
                        crate::ast::NodeData::PropertyDeclaration(d) if d.postfix_token.is_some()
                    );
                    let uninitialized = !prop_decl_has_initializer(prop_decl) && !asserted;
                    let later = later_sibling_property(node, prop_decl);
                    if uninitialized || later {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IS_USED_BEFORE_ITS_INITIALIZATION,
                            vec![name_text.to_string()],
                        ));
                    }
                }
                if let Some(declaring_class) = self.declaring_class_of_member(member_symbol) {

                    let is_private = super::exports::
                        get_declaration_modifier_flags_from_symbol_ex(member_symbol, false)
                        .contains(ModifierFlags::Private);
                    if is_private && !self.is_within_declaring_class(&declaring_class) {
                        let class_name = match &declaring_class.data {
                            crate::ast::NodeData::ClassDeclaration(d) => d
                                .name
                                .as_ref()
                                .map(|n| n.text().to_string())
                                .unwrap_or_default(),
                            _ => String::new(),
                        };
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name.loc,
                            PROPERTY_0_IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1,
                            vec![name_text.to_string(), class_name],
                        ));
                        return;
                    }
                }
            }
        }

        if !obj_type.flags.contains(TypeFlags::Never)
            && self.has_property_of_type(&obj_type, name_text)
        {
            return;
        }

        if self.global_constructor_value_has_property(obj_expr, name_text) {
            return;
        }

        if obj_expr.kind == SyntaxKind::Identifier
            && let Some(sym) = self.resolve_identifier(obj_expr)
        {
            let base = self.resolve_alias_base(sym);
            if base.flags.contains(SymbolFlags::ValueModule) {
                let found = base.exports.entries.contains_key(name_text)
                    || base.members.entries.contains_key(name_text)
                    || self.ambient_namespace_local(&base, name_text).is_some();
                if found {
                    return;
                }
            }
        }
        let file = self.current_file.clone();

        let display_type = if obj_type.flags.contains(TypeFlags::IndexedAccess) {
            self.constraint_of_indexed_access(&obj_type)
                .unwrap_or_else(|| Arc::clone(&obj_type))
        } else {
            Arc::clone(&obj_type)
        };
        let type_str = self.type_to_string(&display_type);

        let suggestion = display_type.as_structured().and_then(|st| {
            let rune_len = name_text.chars().count();
            let maximum_length_difference = 2.max((rune_len as f64 * 0.34) as usize);
            let mut best_distance = (rune_len as f64 * 0.4).floor() + 0.9;
            let mut best: Option<String> = None;
            let mut members: Vec<&String> = st.members.entries.keys().collect();
            members.sort();
            for cand in members {
                let cand = cand.as_str();
                if cand.is_empty()
                    || cand.starts_with('"')
                    || cand.starts_with('\'')
                    || cand.starts_with('`')
                    || cand.starts_with('\u{FE}')
                {
                    continue;
                }
                let cand_len = cand.chars().count();

                if cand_len < 3 && !cand.eq_ignore_ascii_case(name_text) {
                    continue;
                }
                if rune_len.max(cand_len) - rune_len.min(cand_len) > maximum_length_difference {
                    continue;
                }
                if cand == name_text {
                    continue;
                }
                let Some(d) = levenshtein_with_max(name_text, cand, best_distance) else {
                    continue;
                };
                if d < best_distance {
                    best_distance = d;
                    best = Some(cand.to_string());
                }
            }
            best
        });
        if let Some(sugg) = suggestion {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2,
                vec![name_text.to_string(), type_str, sugg],
            ));
        } else {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![name_text.to_string(), type_str],
            ));
        }
    }

    fn report_possibly_null_or_undefined(
        &mut self,
        node: &Arc<Node>,
        t: &Arc<Type>,
        invoke_form: bool,
    ) -> bool {
        if !self.strict_null_checks || !type_is_possibly_undefined(t) {
            return false;
        }
        let possibly_undefined = type_includes_undefined_only(t);
        let possibly_null = type_includes_null_only(t);
        let entity_text = if is_entity_name_expression(node) {

            let text = if node.kind == SyntaxKind::Identifier {
                node.text().to_string()
            } else {
                self.node_source_text(node).unwrap_or_default()
            };
            if !text.is_empty() && text.len() < 100 {
                Some(text)
            } else {
                None
            }
        } else {
            None
        };
        let (message, args): (crate::diagnostics::Message, Vec<String>) = if invoke_form {

            (
                if possibly_undefined {
                    if possibly_null {
                        crate::diagnostics::messages_generated::
                            CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL_OR_UNDEFINED
                    } else {
                        crate::diagnostics::messages_generated::
                            CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_UNDEFINED
                    }
                } else {
                    crate::diagnostics::messages_generated::
                        CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL
                },
                Vec::new(),
            )
        } else if let Some(text) = entity_text {
            if possibly_undefined {
                if possibly_null {
                    (
                        crate::diagnostics::messages_generated::
                            X_0_IS_POSSIBLY_NULL_OR_UNDEFINED,
                        vec![text],
                    )
                } else {
                    (
                        crate::diagnostics::messages_generated::X_0_IS_POSSIBLY_UNDEFINED,
                        vec![text],
                    )
                }
            } else {
                (
                    crate::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL,
                    vec![text],
                )
            }
        } else if possibly_undefined {
            if possibly_null {
                (
                    crate::diagnostics::messages_generated::
                        OBJECT_IS_POSSIBLY_NULL_OR_UNDEFINED,
                    Vec::new(),
                )
            } else {
                (
                    crate::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_UNDEFINED,
                    Vec::new(),
                )
            }
        } else {
            (
                crate::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL,
                Vec::new(),
            )
        };
        self.diagnostics.add(crate::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            message,
            args,
        ));
        true
    }

    fn global_constructor_value_has_property(&mut self, obj_expr: &Arc<Node>, name: &str) -> bool {
        if obj_expr.kind != SyntaxKind::Identifier {
            return false;
        }

        let resolved = match self.resolve_identifier(obj_expr) {
            Some(sym) => sym,
            None => return false,
        };
        let interface_name = match resolved.name.as_str() {
            "Object" => {

                match self.globals.get("Object") {
                    Some(global_sym) if Arc::ptr_eq(&resolved, global_sym) => "ObjectConstructor",
                    _ => return false,
                }
            }
            _ => return false,
        };
        self.global_interface_has_property(interface_name, name)
    }

    #[allow(dead_code)]
    fn property_exists_on_non_nullable_part(&mut self, t: &Arc<Type>, name: &str) -> bool {
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                for ct in &u.union_or_intersection.types {
                    if ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                        continue;
                    }
                    if self.has_property_of_type(ct, name) {
                        return true;
                    }
                }
                return false;
            }
        }

        self.has_property_of_type(t, name)
    }

    fn check_call_arguments(&mut self, node: &Arc<Node>, is_new: bool) {
        let (callee_expr, arguments) = match &node.data {
            crate::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            crate::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return,
        };

        if !is_new && callee_expr.kind == SyntaxKind::SuperKeyword {
            let Some(base_ctor_type) = self.resolve_base_class_constructor_type() else {
                return;
            };
            self.check_call_arguments_against(
                node,
                &base_ctor_type,
                &arguments,
                callee_expr,
                 true,
            );
            return;
        }
        let callee_type = self.get_type_of_node(callee_expr);

        if !is_new {
            let optional_call = matches!(
                &node.data,
                crate::ast::NodeData::CallExpression(d) if d.question_dot_token.is_some()
            );
            if !optional_call {
                self.report_possibly_null_or_undefined(callee_expr, &callee_type, true);
            }
        }
        self.check_call_arguments_against(node, &callee_type, &arguments, callee_expr, is_new);
    }

    fn report_invocation_error(
        &mut self,
        callee_expr: &Arc<Node>,
        callee_type: &Arc<Type>,
        is_new: bool,
    ) {
        let head = if is_new {
            THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE
        } else {
            THIS_EXPRESSION_IS_NOT_CALLABLE
        };
        let no_sigs = if is_new {
            TYPE_0_HAS_NO_CONSTRUCT_SIGNATURES
        } else {
            TYPE_0_HAS_NO_CALL_SIGNATURES
        };
        let chain = if callee_type.flags.contains(TypeFlags::Union)
            && let Some(u) = callee_type.as_union_or_intersection()
        {

            let union_str = self.type_to_string(callee_type);
            let mut has_signatures = false;
            let mut first_without: Option<String> = None;
            for c in u.types.iter() {
                let n = if is_new {
                    c.as_structured()
                        .map(|s| s.construct_signatures().len())
                        .unwrap_or(0)
                } else {
                    c.as_structured()
                        .map(|s| s.call_signatures().len())
                        .unwrap_or(0)
                };
                if n != 0 {
                    has_signatures = true;
                    if first_without.is_some() {
                        break;
                    }
                } else if first_without.is_none() {
                    first_without = Some(self.type_to_string(c));
                }
            }
            let msg = if !has_signatures {
                if is_new {
                    NO_CONSTITUENT_OF_TYPE_0_IS_CONSTRUCTABLE
                } else {
                    NO_CONSTITUENT_OF_TYPE_0_IS_CALLABLE
                }
            } else if first_without.is_some() {
                if is_new {
                    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CONSTRUCTABLE
                } else {
                    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CALLABLE
                }
            } else if is_new {
                EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_CONSTRUCT_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
            } else {
                EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
            };
            let mut outer = crate::ast::Diagnostic::new(
                self.current_file.clone(),
                callee_expr.loc,
                msg,
                vec![union_str],
            );
            if let Some(first) = first_without.filter(|_| has_signatures) {
                outer.message_chain = vec![crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    callee_expr.loc,
                    no_sigs,
                    vec![first],
                )];
            }
            vec![outer]
        } else {

            let apparent_str = if callee_type.flags.contains(TypeFlags::Intersection)
                && self.is_never_intersection(callee_type)
            {
                "never".to_string()
            } else {
                match self.primitive_apparent_name(callee_type) {
                    Some(name) => name.to_string(),
                    None => self.type_to_string(callee_type),
                }
            };
            vec![crate::ast::Diagnostic::new(
                self.current_file.clone(),
                callee_expr.loc,
                no_sigs,
                vec![apparent_str],
            )]
        };
        let mut diag = crate::ast::Diagnostic::new(
            self.current_file.clone(),
            callee_expr.loc,
            head,
            vec![],
        );
        diag.message_chain = chain;
        self.diagnostics.add(diag);
    }

    fn primitive_apparent_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        let name = if t.flags.intersects(
            TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) {
            "String"
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            "Number"
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            "Boolean"
        } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
            "Symbol"
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            "BigInt"
        } else {
            return None;
        };
        self.globals.get(name).map(|_| name)
    }

    fn is_never_intersection(&mut self, t: &Arc<Type>) -> bool {
        let Some(ui) = t.as_union_or_intersection() else {
            return false;
        };
        let domain = |t: &Arc<Type>| -> u8 {
            if t.flags.intersects(
                TypeFlags::String
                    | TypeFlags::StringLiteral
                    | TypeFlags::TemplateLiteral
                    | TypeFlags::StringMapping,
            ) {
                1
            } else if t.flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral) {
                2
            } else if t
                .flags
                .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
            {
                3
            } else if t
                .flags
                .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
            {
                4
            } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
                5
            } else if t.flags.contains(TypeFlags::Undefined) {
                6
            } else if t.flags.contains(TypeFlags::Null) {
                7
            } else {
                0
            }
        };
        let disjoint = |a: &Arc<Type>, b: &Arc<Type>| -> bool {
            let (da, db) = (domain(a), domain(b));
            if da == 0 || db == 0 {
                return false;
            }
            if da != db {
                return true;
            }
            match (a.literal_value(), b.literal_value()) {
                (Some(x), Some(y)) => x != y,
                _ => false,
            }
        };
        for (i, c) in ui.types.iter().enumerate() {
            let Some(cs) = c.as_structured() else {
                continue;
            };
            for prop in &cs.properties {
                for (j, other) in ui.types.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let Some(os) = other.as_structured() else {
                        continue;
                    };
                    if let Some(other_prop) = os
                        .properties
                        .iter()
                        .find(|p| p.name == prop.name)
                        .cloned()
                    {
                        let pt = self.get_type_of_symbol(prop);
                        let ot = self.get_type_of_symbol(&other_prop);
                        if disjoint(&pt, &ot) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn check_call_arguments_against(
        &mut self,
        node: &Arc<Node>,
        callee_type: &Arc<Type>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) {

        if callee_type.flags.contains(TypeFlags::Any) {
            return;
        }

        let cond_constraint;
        let callee_type: &Arc<Type> = if callee_type.flags.contains(TypeFlags::Conditional) {
            match self.deferred_default_constraint_of_conditional(callee_type) {
                Some(constraint) => {
                    cond_constraint = constraint;
                    &cond_constraint
                }
                None => callee_type,
            }
        } else {
            callee_type
        };

        let mut union_signatures: Vec<Arc<Signature>> = Vec::new();
        let signatures: &[Arc<Signature>] =
            if callee_type.as_union_or_intersection().is_some() {

                let mut leaves: Vec<&Arc<Type>> = Vec::new();
                flatten_union_leaves(callee_type, &mut leaves);
                if is_new {

                    let all_constructable = !leaves.is_empty()
                        && leaves.iter().all(|m| {
                            m.as_structured()
                                .is_some_and(|s| !s.construct_signatures().is_empty())
                        });
                    if all_constructable {
                        for m in &leaves {
                            if let Some(s) = m.as_structured() {
                                union_signatures
                                    .extend(s.construct_signatures().iter().cloned());
                            }
                        }
                        &union_signatures
                    } else {

                        self.report_invocation_error(callee_expr, callee_type, is_new);
                        return;
                    }
                } else {

                    let mut expanded_leaves: Vec<Arc<Type>> = Vec::new();
                    for m in leaves.iter().copied() {
                        if m.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                            continue;
                        }
                        if m.flags.contains(TypeFlags::Conditional) {
                            if let Some(constraint) = self
                                .deferred_default_constraint_of_conditional(m)
                            {
                                if let Some(u) = constraint.as_union_or_intersection() {
                                    for c in u.types.iter() {
                                        if !c.flags.intersects(
                                            TypeFlags::Undefined | TypeFlags::Null,
                                        ) && !c.flags.contains(TypeFlags::Never)
                                        {
                                            expanded_leaves.push(Arc::clone(c));
                                        }
                                    }
                                } else if !constraint.flags.intersects(
                                    TypeFlags::Undefined | TypeFlags::Null,
                                ) && !constraint.flags.contains(TypeFlags::Never)
                                {
                                    expanded_leaves.push(constraint);
                                }
                                continue;
                            }
                        }
                        expanded_leaves.push(Arc::clone(m));
                    }
                    let all_callable = !expanded_leaves.is_empty()
                        && expanded_leaves.iter().all(|m| {
                            m.as_structured()
                                .is_some_and(|s| !s.call_signatures().is_empty())
                        });
                    if all_callable {
                        for m in &expanded_leaves {
                            if let Some(s) = m.as_structured() {
                                union_signatures
                                    .extend(s.call_signatures().iter().cloned());
                            }
                        }
                        &union_signatures
                    } else {

                        self.report_invocation_error(callee_expr, callee_type, is_new);
                        return;
                    }
                }
            } else if let Some(structured) = callee_type.as_structured() {
                if is_new {
                    structured.construct_signatures()
                } else {
                    structured.call_signatures()
                }
            } else {

                if !is_new && self.report_get_accessor_call(callee_expr) {
                    return;
                }
                self.report_invocation_error(callee_expr, callee_type, is_new);
                return;
            };

        let type_arg_filtered: Vec<Arc<Signature>>;
        let signatures: &[Arc<Signature>] = {
            let provided = Self::explicit_type_argument_count(node);
            if provided != 0 && signatures.len() > 1 {
                type_arg_filtered = signatures
                    .iter()
                    .filter(|s| s.type_parameters.len() == provided)
                    .cloned()
                    .collect();
                if !type_arg_filtered.is_empty() {
                    &type_arg_filtered
                } else {
                    signatures
                }
            } else {
                signatures
            }
        };
        if signatures.is_empty() {
            if !is_new {

                if callee_expr.kind == SyntaxKind::Identifier
                    && let Some(structured) = callee_type.as_structured()
                    && !structured.construct_signatures().is_empty()
                {
                    let type_str = self.type_to_string(callee_type);
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        callee_expr.loc,
                        crate::diagnostics::messages_generated::
                            VALUE_OF_TYPE_0_IS_NOT_CALLABLE_DID_YOU_MEAN_TO_INCLUDE_NEW,
                        vec![type_str],
                    ));
                    return;
                }
            }
            if is_new {

                if let Some(structured) = callee_type.as_structured() {
                    let call_sigs: &[Arc<Signature>] = structured.call_signatures();
                    if !call_sigs.is_empty() {
                        if !self.no_implicit_any {
                            let matching = self.find_matching_signature(node, call_sigs, &arguments);
                            let ret_is_void = self
                                .get_return_type_of_signature(&call_sigs[matching])
                                .is_some_and(|t| t.flags.contains(TypeFlags::Void));
                            if !ret_is_void {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        ONLY_A_VOID_FUNCTION_CAN_BE_CALLED_WITH_THE_NEW_KEYWORD,
                                    Vec::new(),
                                ));
                            }
                        }
                        self.check_call_arguments_against(
                            node,
                            callee_type,
                            &arguments,
                            callee_expr,
                             false,
                        );
                        return;
                    }
                }
            }

            if !is_new && self.report_get_accessor_call(callee_expr) {
                return;
            }
            self.report_invocation_error(callee_expr, callee_type, is_new);
            return;
        }

        let matching_idx = if signatures.len() == 1 {
            0
        } else {

            let no_match = {
                self.speculation_depth += 1;
                let r = !signatures
                    .iter()
                    .any(|s| self.signature_accepts_arguments(node, s, &arguments));
                self.speculation_depth -= 1;
                r
            };
            if no_match && self.report_no_overload_matches(node, signatures, &arguments) {
                return;
            }
            self.find_matching_signature(node, signatures, &arguments)
        };
        let sig = Arc::clone(&signatures[matching_idx]);

        if !self.check_call_arity(node, &sig, &arguments, callee_expr, is_new) {
            return;
        }
        let _file = self.current_file.clone();

        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {

            let ret = match self.try_get_type_at_position(&sig, rest_index) {
                Some(t) => Some(t),
                None => {
                    let rest_param_type =
                        self.get_type_of_symbol(&sig.parameters[rest_index]);
                    Some(self.get_array_element_type(&rest_param_type))
                }
            };
            ret
        } else {
            None
        };

        if !sig.type_parameters.is_empty() || Self::has_explicit_type_arguments(node) {
            let provided = Self::explicit_type_argument_count(node);

            let expected = if is_new {
                self.get_return_type_of_signature(&sig)
                    .and_then(|rt| rt.symbol.clone())
                    .map(|class_sym| {
                        let tps = self.declared_type_parameter_types(&class_sym);
                        if tps.is_empty() {
                            sig.type_parameters.len()
                        } else {
                            tps.len()
                        }
                    })
                    .unwrap_or_else(|| sig.type_parameters.len())
            } else {
                sig.type_parameters.len()
            };
            if provided != 0
                && provided != expected

                && !callee_type.flags.contains(TypeFlags::Any)
            {
                let loc = match &node.data {
                    crate::ast::NodeData::CallExpression(d) => d
                        .type_arguments
                        .as_ref()
                        .and_then(|t| t.iter().next())
                        .map(|t| t.loc)
                        .unwrap_or(node.loc),
                    crate::ast::NodeData::NewExpression(d) => d
                        .type_arguments
                        .as_ref()
                        .and_then(|t| t.iter().next())
                        .map(|t| t.loc)
                        .unwrap_or(node.loc),
                    _ => node.loc,
                };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    loc,
                    crate::diagnostics::messages_generated::EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                    vec![expected.to_string(), provided.to_string()],
                ));
            }
        }
        let inferred_types = self.infer_call_type_arguments(node, &sig, &arguments.nodes);

        let new_explicit_subst: Option<(Vec<Arc<Type>>, Vec<Arc<Type>>)> = if is_new {
            self.get_return_type_of_signature(&sig)
                .and_then(|rt| rt.symbol.clone())
                .and_then(|class_sym| {
                    let tps = self.declared_type_parameter_types(&class_sym);
                    if tps.is_empty() {
                        return None;
                    }

                    let args: Option<Vec<Arc<Type>>> = match &node.data {
                        crate::ast::NodeData::NewExpression(d) => d
                            .type_arguments
                            .as_ref()
                            .map(|ta| ta.iter().map(|t| self.get_type_from_type_node(t)).collect()),
                        _ => None,
                    };
                    let args = match args {
                        Some(a) if a.len() == tps.len() => Some(a),

                        _ if callee_expr.kind == SyntaxKind::SuperKeyword => self
                            .heritage_type_arguments_for_base(&class_sym)
                            .filter(|a| a.len() == tps.len()),
                        _ => None,
                    };
                    args.map(|args| (tps, args))
                })
        } else {
            None
        };
        if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
            eprintln!(
                "[infer] sig params={} tp={}",
                sig.parameters.len(),
                sig.type_parameters.len()
            );
            for (i, t) in inferred_types.iter().enumerate() {
                eprintln!("[infer]   {} -> {}", i, self.type_to_string(t));
            }
        }
        for (i, arg) in arguments.iter().enumerate() {

            let base_param_type = if has_rest && i >= rest_index {

                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {

                self.try_get_type_at_position(&sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[i]))
            } else {

                continue;
            };

            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &base_param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else if let Some((tps, args)) = new_explicit_subst.as_ref() {
                self.substitute_infer_type_parameters(&base_param_type, tps, args)
            } else {
                Arc::clone(&base_param_type)
            };

            let inference_empty =
                !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }

            if matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) {
                let pt = Arc::clone(&param_type);
                self.check_contextual_elements(arg, &pt, arg.loc);
            }
            let arg_type = self.get_type_of_node(arg);

            let display_param = if i < sig.parameters.len() {
                let param_optional = sig.parameters[i]
                    .flags
                    .contains(crate::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    });
                if param_optional {
                    Some(self.strip_optional_undefined(&param_type))
                } else {
                    None
                }
            } else {
                None
            };

            let elements_reported = matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) && self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc.pos() >= arg.loc.pos() && d.loc.end() <= arg.loc.end());
            if elements_reported {
                continue;
            }
            let ok = self.check_type_related_to_and_elaborate_display(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
                Some(arg),
                None,
                Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
                None,
                display_param.as_ref(),
            );

            if !ok {
                break;
            }
        }
    }

    fn report_no_overload_matches(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> bool {
        let saved = self.diagnostics.take_inner();
        let mut entries: Vec<crate::ast::Diagnostic> = Vec::new();
        let mut all_failed = true;
        for sig in signatures.iter() {
            match self.probe_first_argument_error(node, sig, arguments) {
                Some(d) => entries.push(d),
                None => {
                    all_failed = false;
                    break;
                }
            }
        }
        let _probe_only = self.diagnostics.take_inner();
        self.diagnostics.set_inner(saved);
        if !all_failed {
            return false;
        }
        let file = self.current_file.clone();
        let anchor = entries
            .first()
            .map(|d| d.loc)
            .unwrap_or(node.loc);
        let mut chain: Vec<crate::ast::Diagnostic> = Vec::new();
        for (i, (entry, sig)) in entries.into_iter().zip(signatures.iter()).enumerate() {
            let sig_str = self.signature_display_colon(sig, "");
            let mut d = crate::ast::Diagnostic::new(
                file.clone(),
                anchor,
                crate::diagnostics::messages_generated::
                    OVERLOAD_0_OF_1_2_GAVE_THE_FOLLOWING_ERROR,
                vec![(i + 1).to_string(), signatures.len().to_string(), sig_str],
            );
            d.message_chain = vec![entry];
            chain.push(d);
        }
        let mut head = crate::ast::Diagnostic::new(
            file,
            anchor,
            crate::diagnostics::messages_generated::NO_OVERLOAD_MATCHES_THIS_CALL,
            Vec::new(),
        );
        head.message_chain = chain;
        self.diagnostics.add(head);
        true
    }

    fn probe_first_argument_error(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> Option<crate::ast::Diagnostic> {

        let arg_count = arguments.len();
        let max_params = if sig.has_rest_parameter() {
            usize::MAX
        } else {
            sig.parameters.len()
        };
        if arg_count > max_params || arg_count < sig.min_argument_count.max(0) as usize {
            return None;
        }
        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {

            match self.signature_instantiated_param_type(sig, rest_index) {
                Some(arr) => Some(self.get_array_element_type(&arr)),
                None => match self.try_get_type_at_position(sig, rest_index) {
                    Some(t) => Some(t),
                    None => {
                        let rest_param_type =
                            self.get_type_of_symbol(&sig.parameters[rest_index]);
                        Some(self.get_array_element_type(&rest_param_type))
                    }
                },
            }
        } else {
            None
        };
        let inferred_types = self.infer_call_type_arguments(node, sig, &arguments.nodes);
        for (i, arg) in arguments.iter().enumerate() {
            let base_param_type = if has_rest && i >= rest_index {
                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {
                self.signature_instantiated_param_type(sig, i)
                    .or_else(|| self.try_get_type_at_position(sig, i))
                    .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[i]))
            } else {
                continue;
            };
            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &base_param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else {
                base_param_type
            };
            let inference_empty = !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }
            let arg_type = self.get_type_of_node(arg);
            if self.is_type_related_to(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
            ) {
                continue;
            }
            let param_optional = i < sig.parameters.len()
                && (sig.parameters[i]
                    .flags
                    .contains(crate::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    }));
            let display_param = if param_optional {
                Some(self.strip_optional_undefined(&param_type))
            } else {
                None
            };
            let mut out: Vec<crate::ast::Diagnostic> = Vec::new();
            self.check_type_related_to_and_elaborate_display(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
                Some(arg),
                Some(arg),
                Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
                Some(&mut out),
                display_param.as_ref(),
            );
            return out.into_iter().next();
        }
        None
    }

    pub(crate) fn infer_call_type_arguments(
        &mut self,
        node: &Arc<Node>,
        signature: &Arc<Signature>,
        args: &[Arc<Node>],
    ) -> Vec<Arc<Type>> {
        if signature.type_parameters.is_empty() {
            return Vec::new();
        }
        let inferences: Vec<InferenceInfo> = signature
            .type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(signature));
        self.infer_type_arguments(node, signature, args, &mut context)
    }

    fn check_call_arity(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) -> bool {
        let arg_count = arguments.len();

        if let Some(spread_idx) = arguments
            .nodes
            .iter()
            .position(|a| matches!(a.data, crate::ast::NodeData::SpreadElement(_)))
        {

            let min_count = self.get_min_argument_count(sig);
            let max_count = self.get_parameter_count(sig);
            let has_rest = self.has_effective_rest_parameter(sig);
            let spread_ok = spread_idx >= min_count && (has_rest || spread_idx < max_count);
            if !spread_ok {
                let file = self.current_file.clone();
                let spread_node = Arc::clone(&arguments.nodes[spread_idx]);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    spread_node.loc,
                    A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER,
                    vec![],
                ));
                return false;
            }

            return true;
        }

        let min_count = self.get_min_argument_count(sig);
        let max_count = self.get_parameter_count(sig);
        let has_rest = self.has_effective_rest_parameter(sig);

        if !has_rest && arg_count > max_count {
            let file = self.current_file.clone();
            let loc = self.extra_arguments_range(arguments, max_count);
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                loc,
                EXPECTED_0_ARGUMENTS_BUT_GOT_1,
                vec![min_count.to_string(), arg_count.to_string()],
            ));
            return false;
        }

        if arg_count < min_count {
            let file = self.current_file.clone();

            let error_loc = if is_new {
                node.loc
            } else if let crate::ast::NodeData::PropertyAccessExpression(d) = &callee_expr.data
            {
                d.name.loc
            } else {
                callee_expr.loc
            };
            let message = if has_rest {
                EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1
            } else {
                EXPECTED_0_ARGUMENTS_BUT_GOT_1
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                error_loc,
                message,
                vec![min_count.to_string(), arg_count.to_string()],
            ));
            return false;
        }

        true
    }

    fn extra_arguments_range(&self, arguments: &Arc<NodeList>, max_count: usize) -> TextRange {
        if max_count >= arguments.nodes.len() {

            return arguments.loc;
        }
        let start = arguments.nodes[max_count].loc.pos;
        let mut end = arguments
            .nodes
            .last()
            .map(|a| a.loc.end)
            .unwrap_or(arguments.loc.end);
        if end < start {
            end = start;
        }
        TextRange { pos: start, end }
    }

    fn signature_accepts_arguments(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> bool {

        if arguments.len() < sig.min_argument_count.max(0) as usize {
            return false;
        }

        let inferred_types = if sig.type_parameters.is_empty() {
            Vec::new()
        } else {
            self.infer_call_type_arguments(node, sig, &arguments.nodes)
        };

        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for (i, arg) in arguments.iter().enumerate() {
            let param_type = if has_rest && i >= rest_index {
                match self.try_get_type_at_position(sig, i) {
                    Some(t) => t,
                    None => {
                        let rt = self.get_type_of_symbol(&sig.parameters[rest_index]);
                        match self.get_array_element_type_of(&rt) {
                            Some(e) => e,
                            None => rt,
                        }
                    }
                }
            } else if i < sig.parameters.len() {
                match self.try_get_type_at_position(sig, i) {
                    Some(t) => t,

                    None => continue,
                }
            } else {

                return false;
            };
            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else {
                param_type
            };

            if param_type.flags.contains(TypeFlags::Any) {
                continue;
            }
            let arg_type = self.get_type_of_node(arg);
            if !self.is_type_assignable_to(&arg_type, &param_type) {
                return false;
            }
        }
        true
    }

    fn find_matching_signature(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> usize {

        self.speculation_depth += 1;
        let result = (|| {
            for (idx, sig) in signatures.iter().enumerate() {
                if self.signature_accepts_arguments(node, sig, arguments) {
                    return idx;
                }
            }

            let arg_count = arguments.len();
            for (idx, sig) in signatures.iter().enumerate() {
                let max_params = if sig.has_rest_parameter() {
                    usize::MAX
                } else {
                    sig.parameters.len()
                };
                if arg_count <= max_params
                    && arg_count >= sig.min_argument_count.max(0) as usize
                {
                    return idx;
                }
            }
            0
        })();
        self.speculation_depth -= 1;
        result
    }

    fn enclosing_function_is_generator(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.clone();
        while let Some(n) = cur {

            let in_name_of_current = crate::ast::node_data_generated::node_name(&n).is_some_and(
                |name| {
                    name.loc.pos() <= node.loc.pos() && node.loc.end() <= name.loc.end()
                },
            );
            if in_name_of_current {
                cur = n.parent.clone();
                continue;
            }
            match &n.data {
                crate::ast::NodeData::FunctionDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }
                crate::ast::NodeData::FunctionExpression(d) => {
                    return d.asterisk_token.is_some();
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }

                crate::ast::NodeData::ArrowFunction(_)
                | crate::ast::NodeData::GetAccessorDeclaration(_)
                | crate::ast::NodeData::SetAccessorDeclaration(_)
                | crate::ast::NodeData::ConstructorDeclaration(_) => return false,
                _ => {}
            }
            cur = n.parent.clone();
        }
        false
    }

    fn get_type_of_element_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, arg_expr) = match &node.data {
            crate::ast::NodeData::ElementAccessExpression(data) => {
                (&data.expression, &data.argument_expression)
            }
            _ => return self.get_any_type(),
        };

        {
            let arg_type = self.get_type_of_node(arg_expr);

            let is_type_param_or_union_of = arg_type.is_type_parameter()
                || (arg_type.is_union()
                    && arg_type
                        .types()
                        .is_some_and(|ts| ts.iter().all(|t| t.is_type_parameter())));
            if !arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                && !is_type_param_or_union_of
            {
                let parts: Vec<Arc<Type>> = if arg_type.is_union() {
                    arg_type
                        .types()
                        .map(|ts| ts.to_vec())
                        .unwrap_or_default()
                } else {
                    vec![Arc::clone(&arg_type)]
                };
                for p in parts {
                    if p.flags.intersects(
                        TypeFlags::Any
                            | TypeFlags::Never
                            | TypeFlags::String
                            | TypeFlags::StringLiteral
                            | TypeFlags::Number
                            | TypeFlags::NumberLiteral
                            | TypeFlags::ESSymbol
                            | TypeFlags::EnumLiteral
                            | TypeFlags::StringMapping,
                    ) {
                        continue;
                    }
                    let type_str = self.type_to_string(&p);
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        arg_expr.loc,
                        crate::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec![type_str],
                    ));
                }
            }
        }
        let obj_type = self.get_type_of_node(obj_expr);

        if obj_type.flags.contains(TypeFlags::Union)
            && let Some(members) = obj_type.types().map(|ts| ts.to_vec())
        {
            let mut elem_types: Vec<Arc<Type>> = Vec::new();
            for m in &members {
                if m.flags.contains(TypeFlags::Any) {
                    continue;
                }
                let t = self.element_access_result_type(node, m, arg_expr);
                if !t.flags.contains(TypeFlags::Any) {
                    elem_types.push(t);
                }
            }
            if !elem_types.is_empty() {
                return self.get_union_type(elem_types);
            }
            return self.get_any_type();
        }
        self.element_access_result_type(node, &obj_type, arg_expr)
    }

    fn element_access_result_type(
        &mut self,
        node: &Arc<Node>,
        obj_type: &Arc<Type>,
        arg_expr: &Arc<Node>,
    ) -> Arc<Type> {

        if self.is_tuple_type(obj_type) {
            if let Some(index) = self.get_constant_numeric_value(arg_expr) {
                if let Some(t) = self.get_tuple_element_type(obj_type, index as usize) {
                    return t;
                }
            }

            return self.get_any_type();
        }

        if self.is_array_type(obj_type) {
            return self.get_array_element_type(obj_type);
        }

        if let Some(member_name) = self.literal_element_access_name(arg_expr) {
            if let Some(sym) = self.get_property_of_type(obj_type, &member_name) {
                if let Some(substituted) = self.instantiate_array_member_type(obj_type, &sym) {
                    return self.flow_type_of_access_expression(node, Some(&sym), substituted);
                }
                let prop_type = self.get_type_of_symbol(&sym);
                return self.flow_type_of_access_expression(node, Some(&sym), prop_type);
            }
        }

        if let Some(structured) = obj_type.as_structured() {
            for info in &structured.index_infos {
                if let Some(key_type) = &info.key_type {
                    if key_type.flags.contains(crate::checker::TypeFlags::String)
                        || key_type.flags.contains(crate::checker::TypeFlags::Number)
                    {
                        if let Some(val_type) = &info.value_type {
                            let val_type = Arc::clone(val_type);
                            return self.flow_type_of_access_expression(node, None, val_type);
                        }
                    }
                }
            }
        }

        self.get_any_type()
    }

    fn literal_element_access_name(&self, arg: &Arc<Node>) -> Option<String> {
        match &arg.data {
            crate::ast::NodeData::StringLiteral(data) => Some(data.text.clone()),
            crate::ast::NodeData::NumericLiteral(data) => Some(data.text.clone()),
            _ => None,
        }
    }

    pub(crate) fn get_array_element_type(&self, t: &Arc<Type>) -> Arc<Type> {
        match &t.data {
            crate::checker::TypeData::Object(obj) => {

                if let Some(elem) = obj.type_arguments.first() {
                    return Arc::clone(elem);
                }
                self.get_any_type()
            }
            crate::checker::TypeData::EvolvingArray(ea) => ea
                .element_type
                .clone()
                .unwrap_or_else(|| self.get_any_type()),
            _ => self.get_any_type(),
        }
    }

    fn get_constant_numeric_value(&self, node: &Arc<Node>) -> Option<f64> {
        match &node.data {
            crate::ast::NodeData::NumericLiteral(data) => data.text.parse::<f64>().ok(),
            _ => None,
        }
    }

    fn get_type_of_array_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let elements = match &node.data {
            crate::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return self.get_any_type(),
        };
        if elements.is_empty() {

            let elem = if self.strict_null_checks {
                self.never_type()
            } else {
                self.undefined_type()
            };
            return self.create_array_type(elem);
        }

        let mut element_types: Vec<Arc<Type>> = Vec::new();
        for elem in elements.iter() {

            if elem.kind == SyntaxKind::SpreadElement {
                return self.create_array_type(self.get_any_type());
            }
            let t = self.get_type_of_node(elem);

            let widened = if crate::checker::is_object_literal_type(&t) {
                self.widen_initializer_type(&t)
            } else {
                self.get_widened_type_of_literal(&t)
            };
            element_types.push(widened);
        }

        let first = &element_types[0];
        let all_same = element_types[1..]
            .iter()
            .all(|t| Arc::ptr_eq(t, first) || self.types_are_equal(t, first));
        if all_same {
            return self.create_array_type(Arc::clone(first));
        }

        let elem_union = self.get_union_type(element_types);
        self.create_array_type(elem_union)
    }

    pub(crate) fn is_empty_array_literal(&self, node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
        )
    }

    fn get_const_assertion_type(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        match expr.kind {
            SyntaxKind::ArrayLiteralExpression => {

                let elements = match &expr.data {
                    crate::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
                    _ => return self.get_any_type(),
                };
                let mut element_types: Vec<Arc<Type>> = Vec::new();
                for elem in elements.iter() {
                    if elem.kind == SyntaxKind::SpreadElement {

                        let t = self.get_type_of_node(elem);
                        element_types.push(t);
                    } else {
                        element_types.push(self.get_type_of_node(elem));
                    }
                }
                self.create_tuple_type(element_types)
            }
            _ => {

                self.get_type_of_node(expr)
            }
        }
    }

    fn get_type_of_object_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let properties = match &node.data {
            crate::ast::NodeData::ObjectLiteralExpression(data) => &data.properties,
            _ => return self.get_any_type(),
        };

        let contextual =
            self.get_contextual_type(node, ContextFlags::empty());
        let mut prop_pairs: Vec<(String, Arc<Type>, Option<Arc<Node>>)> = Vec::new();
        let mut fell_back_to_any = false;
        for prop in properties.iter() {
            match &prop.data {
                NodeData::PropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }

                    let mut t = self.get_type_of_node(&data.initializer);
                    if let Some(ctx) = &contextual
                        && let Some(prop_ctx) = self.get_type_of_property_of_type(ctx, &name)
                        && crate::checker::is_fresh_literal_type(&t)
                    {

                        if !self.is_literal_of_contextual_type(&t, &prop_ctx) {
                            t = self.get_widened_literal_type(&t);
                        } else {
                            t = self.get_regular_type_of_literal_type(&t);
                        }
                    }
                    prop_pairs.push((name, t, Some(Arc::clone(prop))));
                }
                NodeData::ShorthandPropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }

                    let t = self.get_type_of_node(&data.name);
                    prop_pairs.push((name, t, Some(Arc::clone(prop))));
                }
                NodeData::SpreadAssignment(_) => {

                    fell_back_to_any = true;
                    break;
                }
                _ => {
                    fell_back_to_any = true;
                    break;
                }
            }
        }
        if fell_back_to_any {
            return self.get_any_type();
        }

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(prop_pairs.len());
        for (name, t, decl) in prop_pairs {
            let mut sym = Symbol::new(SymbolFlags::Property, name.clone());
            if let Some(d) = decl {
                sym.declarations.push(d);
            }
            let symbol = Arc::new(sym);
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
            id: 0,
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

    fn get_excess_property_name(&self, source: &Arc<Type>, target: &Arc<Type>) -> Option<String> {

        if !crate::checker::is_object_literal_type(source) {
            return None;
        }
        let source_struct = source.as_structured()?;
        let target_struct = target.as_structured()?;

        if !target_struct.index_infos.is_empty() {
            return None;
        }
        for prop in &source_struct.properties {

            if !self.target_has_property(target, &prop.name) {
                return Some(prop.name.clone());
            }
        }
        None
    }

    fn target_has_property(&self, t: &Arc<Type>, name: &str) -> bool {

        if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
            return true;
        }
        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }

            if !structured.index_infos.is_empty() {
                return true;
            }
        }

        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }

        if t.flags.contains(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = &t.data {
                return i
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }
        false
    }

    pub(crate) fn get_missing_required_properties(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Vec<String> {
        let Some(source_struct) = source.as_structured() else {
            return Vec::new();
        };
        let Some(target_struct) = target.as_structured() else {
            return Vec::new();
        };
        let mut missing = Vec::new();
        for target_prop in &target_struct.properties {
            if target_prop.flags.contains(SymbolFlags::Optional) {
                continue;
            }
            if source_struct.members.get(&target_prop.name).is_none() {
                missing.push(target_prop.name.clone());
            }
        }
        missing
    }

    fn find_object_literal_property_name_node(
        &self,
        init: &Arc<Node>,
        prop_name: &str,
    ) -> Option<TextRange> {
        let crate::ast::NodeData::ObjectLiteralExpression(data) = &init.data else {
            return None;
        };
        for prop in data.properties.iter() {
            let name = match &prop.data {
                NodeData::PropertyAssignment(p) => &p.name,
                NodeData::ShorthandPropertyAssignment(p) => &p.name,
                _ => continue,
            };
            if self.get_property_name_from_node(name) == prop_name {
                return Some(name.loc);
            }
        }
        None
    }

    pub(crate) fn get_property_name_from_node(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(id) => id.text.clone(),
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => n.text.clone(),
            NodeData::ComputedPropertyName(_) => {

                let file = self
                    .get_source_file_of_node(node)
                    .or_else(|| self.current_file.clone());
                let Some(file) = file else {
                    return String::new();
                };
                let pos = node.loc.pos();
                let end = node.loc.end();
                if pos < end && end <= file.text.len() {
                    file.text[pos..end].to_string()
                } else {
                    String::new()
                }
            }
            _ => node.text().to_string(),
        }
    }

    pub fn get_widened_type_of_literal(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(crate::checker::TypeFlags::StringLiteral)
            || t.flags.contains(crate::checker::TypeFlags::NumberLiteral)
            || t.flags.contains(crate::checker::TypeFlags::BigIntLiteral)
            || t.flags.contains(crate::checker::TypeFlags::BooleanLiteral)
        {
            return self.get_base_type_of_literal_type(t);
        }
        Arc::clone(t)
    }

    fn types_are_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        if a.flags != b.flags {
            return false;
        }

        match (&a.data, &b.data) {
            (crate::checker::TypeData::Intrinsic(a), crate::checker::TypeData::Intrinsic(b)) => {
                a.intrinsic_name == b.intrinsic_name
            }
            _ => false,
        }
    }

    fn infer_number_literal_type(&mut self, text: &str) -> Arc<Type> {

        let num = crate::jsnum::Number::from_string(text);
        if num.is_nan() {
            return self.number_type();
        }
        self.get_number_literal_type(num)
    }

    fn infer_string_literal_type(&mut self, text: &str) -> Arc<Type> {
        self.get_string_literal_type(text)
    }

    pub fn check_statement(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));

        self.type_instantiation_count = 0;

        if self.ambient_context_depth > 0
            && !matches!(
                node.kind,
                SyntaxKind::VariableStatement
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::ImportDeclaration
                    | SyntaxKind::ImportEqualsDeclaration
                    | SyntaxKind::ExportDeclaration
                    | SyntaxKind::ExportAssignment
                    | SyntaxKind::NamespaceExportDeclaration
            )
            && node.parent.as_ref().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::Block | SyntaxKind::ModuleBlock | SyntaxKind::SourceFile
                )
            })
            && !Self::inside_function_body(node)
        {
            let block_id = node.parent.as_ref().unwrap().id();
            if !self.ambient_ts1036_reported_blocks.contains(&block_id) {
                self.ambient_ts1036_reported_blocks.insert(block_id);
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::
                        STATEMENTS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                    Vec::new(),
                ));
            }
        }
        match node.kind {
            SyntaxKind::ExpressionStatement => {
                if let crate::ast::NodeData::ExpressionStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::VariableStatement => {
                if let crate::ast::NodeData::VariableStatement(data) = &node.data {

                    self.check_grammar_variable_declaration_list(&data.declaration_list);
                    self.check_variable_declaration_list(&data.declaration_list);

                    self.check_grammar_modifiers(node);

                    if let crate::ast::NodeData::VariableDeclarationList(list) =
                        &data.declaration_list.data
                    {
                        let decls = list.declarations.clone();
                        for d in decls.iter() {
                            if let crate::ast::NodeData::VariableDeclaration(vd) = &d.data {
                                self.check_cjs_reserved_top_level_name(d, &vd.name);
                            }
                        }
                    }

                    self.check_declaration_nameability(node);
                }
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.check_statement(&data.then_statement);
                    if let Some(else_stmt) = &data.else_statement {
                        self.check_statement(else_stmt);
                    }
                }
            }
            SyntaxKind::WhileStatement => {
                if let crate::ast::NodeData::WhileStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
            }
            SyntaxKind::DoStatement => {
                if let crate::ast::NodeData::DoStatement(data) = &node.data {
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                }
            }
            SyntaxKind::ForStatement => {
                self.push_scope(node);
                if let crate::ast::NodeData::ForStatement(data) = &node.data {
                    if let Some(init) = &data.initializer {
                        self.check_for_initializer(init);
                    }
                    if let Some(cond) = &data.condition {
                        self.check_expression(cond);
                        self.check_truthiness_of_type(cond);
                    }
                    if let Some(incr) = &data.incrementor {
                        self.check_expression(incr);
                    }
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
                self.pop_scope();
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.push_scope(node);
                if let crate::ast::NodeData::ForInOrOfStatement(data) = &node.data {
                    self.check_for_initializer(&data.initializer);
                    self.check_expression(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
                self.pop_scope();
            }
            SyntaxKind::ReturnStatement => {

                if self.function_scope_count == 0 && self.arrow_function_scope_count == 0 {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            A_RETURN_STATEMENT_CAN_ONLY_BE_USED_WITHIN_A_FUNCTION_BODY,
                        Vec::new(),
                    ));
                }
                if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);

                        let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                        if let Some(expected) = expected {
                            let actual = self.get_type_of_node(expr);

                            if !actual.flags.contains(TypeFlags::Any)
                                && !self.is_type_assignable_to(&actual, &expected)
                            {

                                let display_type =
                                    if crate::checker::is_literal_type(&actual) {
                                        self.get_base_type_of_literal_type(&actual)
                                    } else {
                                        actual.clone()
                                    };
                                let ok = self.check_type_related_to_and_optionally_elaborate(
                                    &display_type,
                                    &expected,
                                    crate::checker::relater::RelationKind::Assignable,
                                    Some(node),
                                    Some(expr),
                                    None,
                                    None,
                                );
                                if ok {

                                }
                            }
                        }
                    } else {

                        let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                        if let Some(expected) = expected {
                            if !expected.flags.contains(TypeFlags::Void)
                                && !expected.flags.contains(TypeFlags::Undefined)
                                && !expected.flags.contains(TypeFlags::Any)
                            {
                                let expected_str = self.type_to_string(&expected);
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                    vec!["undefined".to_string(), expected_str],
                                ));
                            }
                        }
                    }
                }
            }
            SyntaxKind::Block => {
                self.push_scope(node);
                if let crate::ast::NodeData::Block(data) = &node.data {

                    let mut after_terminator = false;
                    for stmt in data.statements.iter() {

                        let is_hoistable_decl = matches!(
                            stmt.kind,
                            SyntaxKind::EnumDeclaration
                                | SyntaxKind::FunctionDeclaration
                                | SyntaxKind::ClassDeclaration
                        );
                        if after_terminator && !is_hoistable_decl {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                stmt.loc,
                                UNREACHABLE_CODE_DETECTED,
                                vec![],
                            ));
                        }
                        self.check_statement(stmt);
                        if Self::is_block_terminating_statement(stmt) {
                            after_terminator = true;
                        }
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::ThrowStatement => {
                if let crate::ast::NodeData::ThrowStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Switch,
                            label: None,
                            is_iteration: false,
                        });

                    if let crate::ast::NodeData::CaseBlock(case_block) = &data.case_block.data {
                        self.push_scope(&data.case_block);
                        for case in case_block.clauses.iter() {
                            self.check_case_clause(case);
                        }
                        self.pop_scope();
                    }
                    self.break_continue_context_stack.pop();
                }
            }

            SyntaxKind::FunctionDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        self.check_cjs_reserved_top_level_name(node, name);
                    }
                }

                self.check_duplicate_function_implementations(node);

                self.check_overload_implementation_follows(node);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(tps) = &data.type_parameters {
                        let _ = tps;
                    }
                    self.check_grammar_parameter_list(&data.parameters);

                    self.check_parameter_property_modifiers(&data.parameters, false);

                    self.check_parameter_implicit_any(node, &data.parameters, 0);
                    for p in data.parameters.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = &data.type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any
                        && data.type_node.is_none()
                        && data.body.is_none()
                        && let Some(name) = &data.name
                        && name.kind == SyntaxKind::Identifier
                    {
                        let file = self.current_file.clone();
                        let diagnostic = crate::ast::Diagnostic::new(
                            file,
                            name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![name.text().to_string(), "any".to_string()],
                        );
                        self.diagnostics.add(diagnostic);
                    }
                }

                self.check_unmatched_jsdoc_parameters(node);

                let fn_type = self.get_type_of_function_like(node);

                let fn_symbol = match &node.data {
                    crate::ast::NodeData::FunctionDeclaration(data) => data
                        .name
                        .as_ref()
                        .and_then(|n| self.resolve_identifier(n)),
                    _ => None,
                };
                let fn_type = match &fn_symbol {
                    Some(sym) => self.attach_function_expando_type(sym, fn_type),
                    None => fn_type,
                };
                self.type_node_links.get_or_default(node).resolved_type = Some(fn_type.clone());
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        if let Some(symbol) = self.resolve_identifier(name) {

                            let symbol_type = match self.build_overload_function_type(&symbol) {
                                Some(overload_type) => overload_type,
                                None => fn_type.clone(),
                            };
                            self.value_symbol_links
                                .get_or_default(&symbol)
                                .resolved_type = Some(symbol_type.clone());
                            self.type_node_links.get_or_default(name).resolved_type =
                                Some(symbol_type);
                        }
                    }
                }

                self.push_function_scope(node);
                self.break_continue_context_stack
                    .push(BreakContinueContext {
                        kind: BreakContinueContextKind::Function,
                        label: None,
                        is_iteration: false,
                    });

                let declared_return = match &node.data {
                    crate::ast::NodeData::FunctionDeclaration(data) => {
                        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                        data.type_node
                            .as_ref()
                            .map(|tn| self.get_type_from_type_node(tn))
                            .map(|t| self.unwrap_async_return_type(t, is_async))
                    }
                    _ => None,
                };
                self.return_type_stack.push(declared_return.clone());
                self.in_ctor_body_stack.push(false);

                self.this_container_stack
                    .push(ThisContainerKind::PlainFunction);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.this_container_stack.pop();

                if let Some(ret_type) = &declared_return {
                    if !ret_type.flags.contains(TypeFlags::Void)
                        && !ret_type.flags.contains(TypeFlags::Undefined)
                        && !ret_type.flags.contains(TypeFlags::Any)
                    {
                        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                            if let Some(body) = &data.body {
                                if !self.function_body_definitely_returns(body) {
                                    if !Self::function_body_has_explicit_return(body) {

                                        let loc = data
                                            .type_node
                                            .as_ref()
                                            .map_or(node.loc, |tn| tn.loc);
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            loc,
                                            A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                            vec![],
                                        ));
                                    } else {

                                        let loc = data
                                            .type_node
                                            .as_ref()
                                            .map_or(node.loc, |tn| tn.loc);
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            loc,
                                            FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                                            vec![],
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                self.return_type_stack.pop();
                self.in_ctor_body_stack.pop();
                self.break_continue_context_stack.pop();
                self.pop_function_scope();
            }
            SyntaxKind::ClassDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        self.check_reserved_type_name(
                            name,
                            &crate::diagnostics::messages_generated::CLASS_NAME_CANNOT_BE_0,
                        );

                        self.check_cjs_reserved_top_level_name(node, name);
                    }
                }

                self.push_scope(node);

                let this_type = self.build_class_instance_type_with_base(node);
                self.this_type_stack.push(this_type);

                self.enclosing_class_stack.push(Arc::clone(node));

                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(heritage) = &data.heritage_clauses {
                        for clause in heritage.iter() {
                            self.check_heritage_clause(clause);
                        }
                    }

                    if !node.has_syntactic_modifier(ModifierFlags::Ambient)
                        && self.ambient_context_depth == 0
                        && !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file)
                    {
                        self.check_class_member_overloads(&data.members);
                    }

                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }

                    if let Some(this_type) = self.this_type_stack.last().cloned() {
                        self.check_index_constraints(&this_type, node);
                    }
                    self.check_class_heritage_members(node);

                    self.check_property_initialization(node);
                }
                self.pop_scope();
                self.this_type_stack.pop();
                self.enclosing_class_stack.pop();

                let class_type = self.get_type_of_class_declaration(node);
                self.type_node_links.get_or_default(node).resolved_type = Some(class_type.clone());
                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        if let Some(symbol) = self.resolve_identifier(name) {
                            self.value_symbol_links
                                .get_or_default(&symbol)
                                .resolved_type = Some(class_type);
                        }
                    }
                }
            }
            SyntaxKind::InterfaceDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::InterfaceDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::INTERFACE_NAME_CANNOT_BE_0,
                    );
                    self.check_interface_members(&data.members);
                }

                let iface_sym = self.program.symbol_map().symbol_of(node).cloned();
                if let Some(sym) = iface_sym {
                    let iface_type = self.resolve_interface_type(&sym, None);

                    self.check_index_constraints(&iface_type, node);
                }
            }
            SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier => {

                if matches!(
                    node.kind,
                    SyntaxKind::ImportDeclaration | SyntaxKind::ExportDeclaration
                ) && self.ambient_context_depth == 0
                    && self
                        .current_file
                        .as_ref()
                        .is_none_or(|f| !f.file_name.starts_with("bundled://"))
                {
                    self.check_module_specifier_members(node);
                    self.check_module_export_names(node);
                }

                if matches!(
                    node.kind,
                    SyntaxKind::ImportDeclaration
                        | SyntaxKind::ExportDeclaration
                        | SyntaxKind::ImportEqualsDeclaration
                ) && self.ambient_context_depth == 0
                    && self
                        .current_file
                        .as_ref()
                        .is_none_or(|f| !f.file_name.starts_with("bundled://"))
                {
                    self.check_module_format_mismatch(node);
                }

                if node.kind == SyntaxKind::TypeAliasDeclaration
                    && let crate::ast::NodeData::TypeAliasDeclaration(d) = &node.data
                {
                    self.check_type_annotation(&d.type_node);

                    if !self.current_file.as_ref().is_some_and(|f| {
                        f.file_name.starts_with("bundled://")
                    }) {
                        let _ = self.get_type_from_type_node(&d.type_node);
                    }
                }

                {
                    use crate::core::compiler_options::ModuleKind;
                    let module_ok = matches!(
                        self.compiler_options.module,
                        ModuleKind::ESNext
                            | ModuleKind::Node18
                            | ModuleKind::Node20
                            | ModuleKind::NodeNext
                            | ModuleKind::Preserve
                    );
                    let attributes = match &node.data {
                        crate::ast::NodeData::ImportDeclaration(d) => d.attributes.clone(),
                        crate::ast::NodeData::ExportDeclaration(d) => d.attributes.clone(),
                        _ => None,
                    };
                    if let Some(attrs) = attributes {
                        let file_has_parse_errors = self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.has_parse_diagnostics);
                        if !file_has_parse_errors {
                            let file = self.current_file.clone();
                            let is_type_only = match &node.data {
                                crate::ast::NodeData::ImportDeclaration(d) => d
                                    .import_clause
                                    .as_ref()
                                    .is_some_and(|c| {
                                        matches!(
                                            &c.data,
                                            crate::ast::NodeData::ImportClause(ic)
                                                if ic.phase_modifier
                                                    == Some(SyntaxKind::TypeKeyword)
                                        )
                                    }),
                                crate::ast::NodeData::ExportDeclaration(d) => {
                                    d.is_type_only
                                }
                                _ => false,
                            };
                            let override_mode =
                                self.get_resolution_mode_override(&attrs, is_type_only);
                            let exempt = is_type_only && override_mode.is_some();
                            if !exempt {

                                let emit_commonjs = file
                                    .as_ref()
                                    .map(|f| {
                                        self.program
                                            .get_emit_module_format_of_file(&f.file_name)
                                            == ModuleKind::CommonJS
                                    })
                                    .unwrap_or(false);
                                if !module_ok {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            IMPORT_ATTRIBUTES_ARE_ONLY_SUPPORTED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ESNEXT_NODE18_NODE20_NODENEXT_OR_PRESERVE,
                                        Vec::new(),
                                    ));
                                } else if emit_commonjs {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            IMPORT_ATTRIBUTES_ARE_NOT_ALLOWED_ON_STATEMENTS_THAT_COMPILE_TO_COMMONJS_REQUIRE_CALLS,
                                        Vec::new(),
                                    ));
                                } else if is_type_only {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            IMPORT_ATTRIBUTES_CANNOT_BE_USED_WITH_TYPE_ONLY_IMPORTS_OR_EXPORTS,
                                        Vec::new(),
                                    ));
                                } else if override_mode.is_some() {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        attrs.loc,
                                        crate::diagnostics::messages_generated::
                                            X_RESOLUTION_MODE_CAN_ONLY_BE_SET_FOR_TYPE_ONLY_IMPORTS,
                                        Vec::new(),
                                    ));
                                }
                            }
                        }
                    }
                }

                if self.ambient_context_depth == 0 {
                    let emit_format_cjs = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| {
                            self.program
                                .get_emit_module_format_of_file(&f.file_name)
                                < crate::core::compiler_options::ModuleKind::System
                        });
                    let interop = self.compiler_options.es_module_interop.is_true_or_unknown();
                    if emit_format_cjs {
                        match &node.data {
                            crate::ast::NodeData::ExportDeclaration(d)
                                if d.module_specifier.is_some() =>
                            {
                                match d.export_clause.as_ref().map(|c| c.kind) {

                                    Some(SyntaxKind::NamespaceExport) if interop => {
                                        self.check_external_emit_helpers(
                                            node,
                                            EXTERNAL_EMIT_HELPER_IMPORT_STAR,
                                        );
                                    }

                                    None => {
                                        self.check_external_emit_helpers(
                                            node,
                                            EXTERNAL_EMIT_HELPER_EXPORT_STAR,
                                        );
                                    }

                                    Some(SyntaxKind::NamedImports | SyntaxKind::NamedExports) => {
                                        let elements = d.export_clause.as_ref().and_then(|c| {
                                            match &c.data {
                                                crate::ast::NodeData::NamedExports(ne) => {
                                                    Some(ne.elements.clone())
                                                }
                                                crate::ast::NodeData::NamedImports(ni) => {
                                                    Some(ni.elements.clone())
                                                }
                                                _ => None,
                                            }
                                        });
                                        if interop
                                            && let Some(elements) = elements
                                        {
                                            for spec in elements.nodes.iter() {
                                                if let crate::ast::NodeData::ExportSpecifier(es) =
                                                    &spec.data
                                                {
                                                    let pn = es
                                                        .property_name
                                                        .as_ref()
                                                        .unwrap_or(&es.name);
                                                    if pn.kind == SyntaxKind::DefaultKeyword
                                                        || pn.text() == "default"
                                                    {
                                                        self.check_external_emit_helpers(
                                                            spec,
                                                            EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            crate::ast::NodeData::ImportDeclaration(d) => {
                                if let Some(clause) = &d.import_clause
                                    && let crate::ast::NodeData::ImportClause(ic) = &clause.data
                                {

                                    if interop
                                        && matches!(
                                            ic.named_bindings.as_ref().map(|b| b.kind),
                                            Some(SyntaxKind::NamespaceImport)
                                        )
                                    {
                                        self.check_external_emit_helpers(
                                            node,
                                            EXTERNAL_EMIT_HELPER_IMPORT_STAR,
                                        );
                                    }

                                    if interop
                                        && let Some(nb) = &ic.named_bindings
                                        && let crate::ast::NodeData::NamedImports(ni) = &nb.data
                                    {
                                        for spec in ni.elements.nodes.iter() {
                                            if let crate::ast::NodeData::ImportSpecifier(is) =
                                                &spec.data
                                            {
                                                let pn =
                                                    is.property_name.as_ref().unwrap_or(&is.name);
                                                if pn.kind == SyntaxKind::DefaultKeyword
                                                    || pn.text() == "default"
                                                {
                                                    self.check_external_emit_helpers(
                                                        spec,
                                                        EXTERNAL_EMIT_HELPER_IMPORT_DEFAULT,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if matches!(
                    node.kind,
                    SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration
                ) && self.ambient_context_depth > 0
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let spec = match &node.data {
                        crate::ast::NodeData::ImportDeclaration(d) => {
                            Some(d.module_specifier.text().to_string())
                        }
                        crate::ast::NodeData::ImportEqualsDeclaration(d) => {
                            if let crate::ast::NodeData::ExternalModuleReference(ext) =
                                &d.module_reference.data
                            {
                                Some(ext.expression.text().to_string())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(spec) = spec {
                        let relative = spec.starts_with("./")
                            || spec.starts_with("../")
                            || spec.starts_with(".\\")
                            || spec.starts_with("..\\");
                        if relative {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    IMPORT_OR_EXPORT_DECLARATION_IN_AN_AMBIENT_MODULE_DECLARATION_CANNOT_REFERENCE_MODULE_THROUGH_RELATIVE_MODULE_NAME,
                                vec![],
                            ));

                            let spec_loc = match &node.data {
                                crate::ast::NodeData::ImportDeclaration(d) => {
                                    d.module_specifier.loc
                                }
                                crate::ast::NodeData::ImportEqualsDeclaration(d) => {

                                    if let crate::ast::NodeData::ExternalModuleReference(ext) =
                                        &d.module_reference.data
                                    {
                                        ext.expression.loc
                                    } else {
                                        d.module_reference.loc
                                    }
                                }
                                _ => node.loc,
                            };
                            let spec_trimmed = spec.trim_matches(['"', '\'', '`']).to_string();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                spec_loc,
                                crate::diagnostics::messages_generated::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
                                vec![spec_trimmed],
                            ));
                        }
                    }
                }

                if node.kind == SyntaxKind::ImportEqualsDeclaration
                    && let crate::ast::NodeData::ImportEqualsDeclaration(d) = &node.data
                    && matches!(
                        d.module_reference.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {
                    let entity_ok = match &d.module_reference.data {
                        crate::ast::NodeData::Identifier(id) => {
                            is_valid_identifier_text(&id.text)
                                && !matches!(id.text.as_str(), "null" | "true" | "false")
                        }
                        _ => true,
                    };

                    let ns_hit = self
                        .resolve_identifier_with_meaning(
                            &base_identifier_of(&d.module_reference),
                            SymbolFlags::NAMESPACE,
                        )
                        .map(|s| self.resolve_alias_base(s));
                    let base_is_namespace = match &d.module_reference.data {
                        crate::ast::NodeData::Identifier(_) => ns_hit
                            .as_ref()
                            .is_some_and(|b| b.flags.intersects(SymbolFlags::NAMESPACE)),
                        _ => true,
                    };
                    let traced_err = if entity_ok && !base_is_namespace {

                        let base = base_identifier_of(&d.module_reference);
                        let any_hit = self
                            .resolve_identifier(&base)
                            .map(|s| self.resolve_alias_base(s));
                        let masked = any_hit.as_ref().is_some_and(|s| {
                            !s.flags.intersects(SymbolFlags::NAMESPACE)
                                && ns_hit
                                    .as_ref()
                                    .is_some_and(|n| n.flags.intersects(SymbolFlags::VALUE))
                        });
                        if masked {
                            ImportEntityError::HiddenByLocal(base)
                        } else if any_hit
                            .as_ref()
                            .is_some_and(|s| s.flags.intersects(SymbolFlags::TYPE))
                        {
                            ImportEntityError::TypeAsNamespace(base)
                        } else {
                            ImportEntityError::NamespaceNotFound(base)
                        }
                    } else if entity_ok {
                        match self.resolve_qualified_symbol_traced(&d.module_reference) {
                            Err((segment, ns_path, _member)) if ns_path.is_empty() => {

                                let any_hit = self
                                    .resolve_identifier(&segment)
                                    .map(|s| self.resolve_alias_base(s));
                                if any_hit
                                    .as_ref()
                                    .is_some_and(|s| s.flags.intersects(SymbolFlags::TYPE))
                                {
                                    ImportEntityError::TypeAsNamespace(segment)
                                } else {
                                    ImportEntityError::NamespaceNotFound(segment)
                                }
                            }
                            Err(e) => ImportEntityError::MissingMember(e),
                            Ok(_) => ImportEntityError::None,
                        }
                    } else {
                        ImportEntityError::None
                    };
                    if !matches!(traced_err, ImportEntityError::None)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        match traced_err {
                            ImportEntityError::None => {}
                            ImportEntityError::NamespaceNotFound(seg) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                                    vec![seg.text().to_string()],
                                ));
                            }
                            ImportEntityError::TypeAsNamespace(seg) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::
                                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                    vec![seg.text().to_string()],
                                ));
                            }
                            ImportEntityError::HiddenByLocal(seg) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::
                                        MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME,
                                    vec![seg.text().to_string()],
                                ));
                            }
                            ImportEntityError::MissingMember((seg, ns_path, member)) => {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    seg.loc,
                                    crate::diagnostics::messages_generated::
                                        NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                    vec![ns_path, member],
                                ));
                            }
                        }
                    }

                    if entity_ok
                        && let Some(ns) = ns_hit.as_ref()
                        && ns.flags.intersects(SymbolFlags::VALUE)
                    {
                        let base = base_identifier_of(&d.module_reference);
                        let masked = self
                            .resolve_identifier_with_meaning(
                                &base,
                                SymbolFlags::VALUE | SymbolFlags::NAMESPACE,
                            )

                            .map(|s| self.resolve_alias_base(s))
                            .is_some_and(|s| !s.flags.intersects(SymbolFlags::NAMESPACE));
                        if masked {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                base.loc,
                                crate::diagnostics::messages_generated::
                                    MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME,
                                vec![base.text().to_string()],
                            ));
                        }
                    }

                    if node.kind == SyntaxKind::ImportEqualsDeclaration
                        && let crate::ast::NodeData::ImportEqualsDeclaration(d) = &node.data
                    {
                        if let Some(alias_sym) =
                            self.program.symbol_map().symbol_of(node).cloned()
                        {
                            let target = self.resolve_alias_base(Arc::clone(&alias_sym));

                            let target_resolved = !Arc::ptr_eq(&target, &alias_sym)
                                || !target.flags.intersects(SymbolFlags::Alias);
                            if target_resolved && target.flags.intersects(SymbolFlags::TYPE) {
                                self.check_reserved_type_name(
                                    &d.name,
                                    &crate::diagnostics::messages_generated::IMPORT_NAME_CANNOT_BE_0,
                                );
                            }

                            let non_alias_flags =
                                alias_sym.flags.difference(SymbolFlags::Alias);
                            let has_local_conflict = target_resolved
                                && alias_sym
                                    .declarations
                                    .iter()
                                    .any(|dd| dd.id() != node.id())
                                && !non_alias_flags.is_empty()
                                && {
                                    let value_side =
                                        non_alias_flags.intersects(SymbolFlags::VALUE);
                                    let type_side =
                                        non_alias_flags.intersects(SymbolFlags::TYPE);
                                    (value_side && target.flags.intersects(SymbolFlags::VALUE))
                                        || (type_side && target.flags.intersects(SymbolFlags::TYPE))
                                };
                            if has_local_conflict {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        IMPORT_DECLARATION_CONFLICTS_WITH_LOCAL_DECLARATION_OF_0,
                                    vec![d.name.text().to_string()],
                                ));
                            }
                        }
                    }
                }
            }
            SyntaxKind::EnumDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::ENUM_NAME_CANNOT_BE_0,
                    );

                    if let Some(sym) = self.program.symbol_map().symbol_of(node) {
                        let enum_decls: Vec<&Arc<Node>> = sym
                            .declarations
                            .iter()
                            .filter(|d| d.kind == SyntaxKind::EnumDeclaration)
                            .collect();
                        if enum_decls.len() > 1 {
                            let is_first_decl =
                                enum_decls.first().is_some_and(|d| Arc::ptr_eq(d, &node));

                            let first_decl_starts_uninit = enum_decls.first().and_then(|d| {
                                let NodeData::EnumDeclaration(ed) = &d.data else {
                                    return None;
                                };
                                ed.members.iter().next().and_then(|m| {
                                    matches!(&m.data, crate::ast::NodeData::EnumMember(em) if em.initializer.is_none())
                                        .then_some(())
                                })
                            }) == Some(());
                            if !is_first_decl && first_decl_starts_uninit {
                                let first_member = data.members.iter().next();
                                let uninit = first_member.is_some_and(|m| {
                                    matches!(
                                        &m.data,
                                        crate::ast::NodeData::EnumMember(em)
                                            if em.initializer.is_none()
                                    )
                                });
                                if uninit {
                                    let loc = first_member
                                        .and_then(|m| m.name())
                                        .map(|n| n.loc)
                                        .unwrap_or(node.loc);
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        loc,
                                        crate::diagnostics::messages_generated::
                                            IN_AN_ENUM_WITH_MULTIPLE_DECLARATIONS_ONLY_ONE_DECLARATION_CAN_OMIT_AN_INITIALIZER_FOR_ITS_FIRST_ENUM_ELEMENT,
                                        Vec::new(),
                                    ));
                                }
                            }
                        }
                    }
                }

                self.push_scope(node);
                if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
                    for member in data.members.iter() {
                        self.check_enum_member(member);
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::ExportAssignment => {

                if let crate::ast::NodeData::ExportAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ModuleDeclaration => {

                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                    && data.name.kind == SyntaxKind::Identifier
                    && !is_valid_identifier_text(data.name.text())
                {
                    if let Some(msg) = Self::cannot_find_name_message_for("module") {
                        let file = self.current_file.clone();
                        let kw = crate::core::text::TextRange::new(
                            node.loc.pos(),
                            (node.loc.pos() + 6).min(node.loc.end()),
                        );
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            kw,
                            *msg,
                            vec!["module".to_string()],
                        ));
                    }
                }

                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                    && data.name.kind == SyntaxKind::StringLiteral
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let raw = data.name.text();
                    let module_name = raw.trim_matches(['"', '\'']);
                    let relative = module_name.starts_with("./")
                        || module_name.starts_with("../")
                        || module_name.starts_with(".\\")
                        || module_name.starts_with("..\\");
                    let ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file);

                    if relative && ambient {

                        let is_decl_name_direct = !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.external_module_indicator.is_some());
                        if is_decl_name_direct {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                data.name.loc,
                                crate::diagnostics::messages_generated::
                                    AMBIENT_MODULE_DECLARATION_CANNOT_SPECIFY_RELATIVE_MODULE_NAME,
                                vec![],
                            ));
                        }
                    }
                }

                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                    && data.name.kind == SyntaxKind::StringLiteral
                    && self.current_file.as_ref().is_some_and(|f| {
                        f.external_module_indicator.is_some()
                            && !f.file_name.starts_with("bundled://")
                    })
                {
                    let module_name = data.name.text().trim_matches(['"', '\'']).to_string();
                    let resolvable = self.resolve_module_file_symbol(&module_name).is_some();
                    if !resolvable {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                INVALID_MODULE_NAME_IN_AUGMENTATION_MODULE_0_CANNOT_BE_FOUND,
                            vec![module_name],
                        ));
                    }
                }

                if let crate::ast::NodeData::ModuleDeclaration(mdd) = &node.data
                    && mdd.name.kind == SyntaxKind::Identifier
                    && !node.has_syntactic_modifier(ModifierFlags::Ambient)
                    && self.ambient_context_depth == 0
                    && !self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file)
                    && let Some(sym) = self.program.symbol_map().symbol_of(node)
                {
                if sym.flags.contains(SymbolFlags::ValueModule)
                    && sym.declarations.len() > 1
                    && module_is_instantiated(
                        node,
                        self.compiler_options.should_preserve_const_enums(),
                    )
                {

                    let first_non_ambient = sym.declarations.iter().find(|d| {
                        let bodied_fn = matches!(
                            &d.data,
                            crate::ast::NodeData::FunctionDeclaration(fd)
                                if fd.body.is_some()
                        );
                        (matches!(d.kind, SyntaxKind::ClassDeclaration) || bodied_fn)
                            && !d.has_syntactic_modifier(ModifierFlags::Ambient)
                            && !self
                                .get_source_file_of_node(d)
                                .is_some_and(|f| f.is_declaration_file)
                    });
                    if let Some(fc) = first_non_ambient
                        && node.loc.pos() < fc.loc.pos()
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            mdd.name.loc,
                            crate::diagnostics::messages_generated::
                                A_NAMESPACE_DECLARATION_CANNOT_BE_LOCATED_PRIOR_TO_A_CLASS_OR_FUNCTION_WITH_WHICH_IT_IS_MERGED,
                            Vec::new(),
                        ));
                    }
                }
                }

                let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient);
                if is_ambient {
                    self.ambient_context_depth += 1;
                }
                self.push_scope(node);
                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.pop_scope();
                if is_ambient {
                    self.ambient_context_depth -= 1;
                }
            }
            SyntaxKind::EmptyStatement => {

            }
            SyntaxKind::LabeledStatement => {

                if let crate::ast::NodeData::LabeledStatement(data) = &node.data {
                    let label_text = data.label.text().to_string();
                    let is_iteration = matches!(
                        data.statement.kind,
                        SyntaxKind::WhileStatement
                            | SyntaxKind::DoStatement
                            | SyntaxKind::ForStatement
                            | SyntaxKind::ForInStatement
                            | SyntaxKind::ForOfStatement
                    );
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Labeled,
                            label: Some(label_text),
                            is_iteration,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
            }
            SyntaxKind::BreakStatement | SyntaxKind::ContinueStatement => {

                self.check_grammar_break_or_continue_statement(node);
            }
            SyntaxKind::VariableDeclaration => {
                self.check_variable_declaration(node);
            }

            SyntaxKind::ModuleBlock => {
                if let crate::ast::NodeData::ModuleBlock(data) = &node.data {
                    for stmt in data.statements.iter() {
                        self.check_statement(stmt);
                    }
                }
            }
            _ => {

                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }

    fn declaration_is_ambient(&self, node: &Arc<Node>) -> bool {
        if self.ambient_context_depth > 0 {
            return true;
        }

        if self
            .get_source_file_of_node(node)
            .or(self.current_file.clone())
            .is_some_and(|f| f.is_declaration_file)
        {
            return true;
        }
        let mut cur = Some(node);
        while let Some(n) = cur {
            if n.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }

            if matches!(n.kind, SyntaxKind::VariableStatement | SyntaxKind::ClassDeclaration | SyntaxKind::FunctionDeclaration) {
                break;
            }
            cur = n.parent.as_ref();
        }
        false
    }

    fn check_cjs_reserved_top_level_name(&mut self, node: &Arc<Node>, name: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;
        if !matches!(name.kind, SyntaxKind::Identifier) {
            return;
        }

        if self.compiler_options.no_emit.is_true() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let is_module = file.external_module_indicator.is_some()
            || file.common_js_module_indicator.is_some();
        if !is_module {
            return;
        }

        let mut top_level = false;
        let mut p = node.parent.as_ref();
        while let Some(parent) = p {
            match parent.kind {
                SyntaxKind::SourceFile => {
                    top_level = true;
                    break;
                }
                SyntaxKind::VariableDeclarationList | SyntaxKind::VariableStatement => {
                    p = parent.parent.as_ref();
                }
                _ => break,
            }
        }
        if !top_level {
            return;
        }

        if self.declaration_is_ambient(node) {
            return;
        }
        let emit_format = self.program.get_emit_module_format_of_file(&file.file_name);
        let text = name.text().to_string();
        if text == "require" || text == "exports" {

            if emit_format >= ModuleKind::ES2015 {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    DUPLICATE_IDENTIFIER_0_COMPILER_RESERVES_NAME_1_IN_TOP_LEVEL_SCOPE_OF_A_MODULE,
                vec![text.clone(), text],
            ));
        } else if text == "__esModule" {

            let var_stmt = node
                .parent
                .as_ref()
                .and_then(|list| list.parent.as_ref())
                .filter(|stmt| stmt.kind == SyntaxKind::VariableStatement);
            let exported = var_stmt.is_some_and(|stmt| stmt.has_syntactic_modifier(ModifierFlags::Export));
            if !exported || emit_format >= ModuleKind::System {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    IDENTIFIER_EXPECTED_ESMODULE_IS_RESERVED_AS_AN_EXPORTED_MARKER_WHEN_TRANSFORMING_ECMASCRIPT_MODULES,
                Vec::new(),
            ));
        } else if text == "Object" && node.kind == SyntaxKind::ClassDeclaration {

            if emit_format != ModuleKind::CommonJS {
                return;
            }
            let module_str = match self.compiler_options.module {
                ModuleKind::Node16 => "Node16".to_string(),
                ModuleKind::Node18 => "Node18".to_string(),
                ModuleKind::Node20 => "Node20".to_string(),
                ModuleKind::NodeNext => "NodeNext".to_string(),
                ModuleKind::CommonJS => "CommonJS".to_string(),
                ModuleKind::AMD => "AMD".to_string(),
                ModuleKind::UMD => "UMD".to_string(),
                ModuleKind::System => "System".to_string(),
                ModuleKind::ES2015 => "es2015".to_string(),
                ModuleKind::ES2020 => "es2020".to_string(),
                ModuleKind::ES2022 => "es2022".to_string(),
                _ => "esnext".to_string(),
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    CLASS_NAME_CANNOT_BE_OBJECT_WHEN_TARGETING_ES5_AND_ABOVE_WITH_MODULE_0,
                vec![module_str],
            ));
        }
    }

    fn check_for_initializer(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::VariableDeclarationList => {
                self.check_variable_declaration_list(node);
            }
            _ => self.check_expression(node),
        }
    }

    fn check_binding_pattern_computed_names(&mut self, name: &Arc<Node>) {
        if !matches!(
            name.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        ) {
            return;
        }
        let mut stack = vec![Arc::clone(name)];
        while let Some(n) = stack.pop() {
            match &n.data {
                crate::ast::NodeData::BindingPattern(data) => {
                    for element in data.elements.iter() {
                        stack.push(Arc::clone(element));
                    }
                }
                crate::ast::NodeData::BindingElement(data) => {
                    if let Some(pn) = &data.property_name {
                        if pn.kind == SyntaxKind::ComputedPropertyName {

                            self.check_computed_property_name(pn);
                            if let crate::ast::NodeData::ComputedPropertyName(cd) = &pn.data {
                                self.check_expression(&cd.expression);

                                let expr_type = self.get_type_of_node(&cd.expression);
                                let is_any = match &expr_type.data {
                                    crate::checker::types::TypeData::Union(u) => u
                                        .union_or_intersection
                                        .types
                                        .iter()
                                        .any(|t| t.flags.contains(TypeFlags::Any)),
                                    _ => expr_type.flags.contains(TypeFlags::Any),
                                };
                                if is_any {
                                    let file = self.current_file.clone();
                                    let type_str = self.type_to_string(&expr_type);
                                    let diagnostic = crate::ast::Diagnostic::new(
                                        file,
                                        cd.expression.loc,
                                        crate::diagnostics::messages_generated::
                                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                                        vec![type_str],
                                    );
                                    self.diagnostics.add(diagnostic);
                                }
                            }
                        }
                    }
                    if let Some(inner) = &data.name {
                        if matches!(
                            inner.kind,
                            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                        ) {
                            stack.push(Arc::clone(inner));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn check_variable_declaration_list(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclarationList(data) = &node.data {
            for decl in data.declarations.iter() {

                if let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data
                    && let Some(init) = &vd.initializer
                    && (node.has_syntactic_modifier(ModifierFlags::Ambient)
                        || node.parent.as_ref().is_some_and(|p| {
                            p.has_syntactic_modifier(ModifierFlags::Ambient)
                        })
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file))
                    && self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                {
                    let is_const = node.flags.contains(NodeFlags::Const);
                    let is_simple_literal = match &init.data {
                        crate::ast::NodeData::StringLiteral(_)
                        | crate::ast::NodeData::NumericLiteral(_)
                        | crate::ast::NodeData::BigIntLiteral(_)
                        | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_) => true,
                        _ if matches!(init.kind, SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword) => {
                            true
                        }

                        crate::ast::NodeData::PropertyAccessExpression(_)
                        | crate::ast::NodeData::ElementAccessExpression(_) => true,
                        _ => false,
                    };
                    let message = if is_const && vd.type_node.is_none() {
                        if is_simple_literal {
                            None
                        } else {
                            Some(
                                crate::diagnostics::messages_generated::
                                    A_CONST_INITIALIZER_IN_AN_AMBIENT_CONTEXT_MUST_BE_A_STRING_OR_NUMERIC_LITERAL_OR_LITERAL_ENUM_REFERENCE,
                            )
                        }
                    } else {
                        Some(
                            crate::diagnostics::messages_generated::
                                INITIALIZERS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                        )
                    };
                    if let Some(message) = message {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            init.loc,
                            message,
                            vec![],
                        ));
                    }
                }

                if let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data
                    && vd.name.kind == SyntaxKind::Identifier
                    && matches!(vd.name.text(), "eval" | "arguments")
                    && self.in_strict_context()
                {
                    let is_module = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                    let message = if is_module {
                        crate::diagnostics::messages_generated::
                            INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
                    } else {
                        crate::diagnostics::messages_generated::INVALID_USE_OF_0_IN_STRICT_MODE
                    };
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        vd.name.loc,
                        message,
                        vec![vd.name.text().to_string()],
                    ));
                }
                self.check_variable_declaration(decl);
            }
        }
    }

    fn in_strict_context(&self) -> bool {
        if self.program.options().always_strict.is_true() {
            return true;
        }
        self.current_file.as_ref().is_some_and(|f| {
            f.external_module_indicator.is_some()
                || f.text.trim_start().starts_with("\"use strict\"")
                || f.text.trim_start().starts_with("'use strict'")
        })
    }

    fn report_abstract_property_access_in_ctor(
        &mut self,
        name_node: &Arc<Node>,
        prop_text: &str,
        this_type: &Arc<Type>,
    ) {
        let Some(structured) = this_type.as_structured() else {
            return;
        };
        let Some(member_symbol) = structured.members.get(prop_text) else {
            return;
        };
        let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
            d.kind == SyntaxKind::PropertyDeclaration
                && d.has_syntactic_modifier(ModifierFlags::Abstract)
        }) else {
            return;
        };
        let Some(parent) = &abstract_decl.parent else { return };
        let Some(class_name) = class_declaration_name(parent) else {
            return;
        };
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            name_node.loc,
            crate::diagnostics::messages_generated::
                ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
            vec![prop_text.to_string(), class_name],
        ));
    }

    fn access_in_property_initializer(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.as_ref();
        while let Some(a) = cur {
            match a.kind {
                SyntaxKind::PropertyDeclaration => return true,
                SyntaxKind::Constructor
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression => return false,
                _ => {}
            }
            cur = a.parent.as_ref();
        }
        false
    }

    fn check_this_destructuring_abstract_properties(
        &mut self,
        pattern: &Arc<Node>,
        this_type: &Arc<Type>,
    ) {
        let Some(structured) = this_type.as_structured() else {
            return;
        };
        let crate::ast::NodeData::BindingPattern(data) = &pattern.data else {
            return;
        };
        for element in data.elements.iter() {
            let crate::ast::NodeData::BindingElement(el) = &element.data else {
                continue;
            };

            let Some(prop_name_node) = el
                .property_name
                .as_ref()
                .or(el.name.as_ref())
                .filter(|n| n.kind == SyntaxKind::Identifier)
            else {
                continue;
            };
            let prop_text = prop_name_node.text();
            let Some(member_symbol) = structured.members.get(prop_text) else {
                continue;
            };
            let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
                d.kind == SyntaxKind::PropertyDeclaration
                    && d.has_syntactic_modifier(ModifierFlags::Abstract)
            }) else {
                continue;
            };
            let Some(parent) = &abstract_decl.parent else { continue };
            let Some(class_name) = class_declaration_name(parent) else {
                continue;
            };
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                prop_name_node.loc,
                crate::diagnostics::messages_generated::
                    ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
                vec![prop_text.to_string(), class_name],
            );
            self.diagnostics.add(diagnostic);
        }
    }

    fn check_variable_declaration(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclaration(data) = &node.data {

            if data.initializer.is_none() {
                let is_const = node
                    .parent
                    .as_ref()
                    .is_some_and(|list| list.flags.contains(NodeFlags::Const));
                let in_for_in_of = node.parent.as_ref().and_then(|l| l.parent.as_ref())
                    .is_some_and(|g| {
                        matches!(
                            g.kind,
                            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                        )
                    });
                let is_ambient = self.ambient_context_depth > 0
                    || node.flags.contains(NodeFlags::Ambient)
                    || node
                        .parent
                        .as_ref()
                        .and_then(|p| p.parent.as_ref())
                        .is_some_and(|stmt| {
                            stmt.has_syntactic_modifier(ModifierFlags::Ambient)
                        })
                    || {

                        let mut anc = node.parent.as_ref();
                        let mut found = false;
                        while let Some(a) = anc {
                            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                                found = true;
                                break;
                            }
                            anc = a.parent.as_ref();
                        }
                        found
                    }
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                if is_const && !in_for_in_of && !is_ambient {
                    let file = self.current_file.clone();
                    let name_loc = data.name.loc;
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        name_loc,
                        crate::diagnostics::messages_generated::X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                        vec!["const".to_string()],
                    ));
                }
            }

            if data.initializer.is_some() && data.name.kind == SyntaxKind::Identifier {
                let list_is_var = node.parent.as_ref().is_none_or(|l| {
                    !(l.flags.contains(NodeFlags::Let) || l.flags.contains(NodeFlags::Const))
                });
                let is_param = node
                    .parent
                    .as_ref()
                    .is_some_and(|l| l.kind == SyntaxKind::Parameter);
                if list_is_var && !is_param {
                    let own = self.program.symbol_map().symbol_of(node).cloned();
                    if let Some(local) = self.resolve_identifier(&data.name)
                        && own.as_ref().is_none_or(|o| !Arc::ptr_eq(o, &local))
                        && local.flags.contains(SymbolFlags::BlockScopedVariable)
                        && let Some(vd) = local.value_declaration.clone()
                        && vd.kind == SyntaxKind::VariableDeclaration
                        && let Some(list) = vd.parent.as_ref()
                        && list.kind == SyntaxKind::VariableDeclarationList
                    {
                        let container = list.parent.as_ref().and_then(|s| s.parent.as_ref());
                        let names_share_scope = container.is_some_and(|c| {
                            c.kind == SyntaxKind::ModuleBlock
                                || c.kind == SyntaxKind::ModuleDeclaration
                                || c.kind == SyntaxKind::SourceFile
                                || (c.kind == SyntaxKind::Block
                                    && c.parent.as_ref().is_some_and(|p| {
                                        matches!(
                                            p.kind,
                                            SyntaxKind::FunctionDeclaration
                                                | SyntaxKind::FunctionExpression
                                                | SyntaxKind::ArrowFunction
                                                | SyntaxKind::MethodDeclaration
                                                | SyntaxKind::Constructor
                                                | SyntaxKind::GetAccessor
                                                | SyntaxKind::SetAccessor
                                        )
                                    }))
                        });
                        if !names_share_scope {
                            let name_text = data.name.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_INITIALIZE_OUTER_SCOPED_VARIABLE_0_IN_THE_SAME_SCOPE_AS_BLOCK_SCOPED_DECLARATION_1,
                                vec![name_text.clone(), name_text],
                            ));
                        }
                    }
                }
            }

            self.check_binding_pattern_computed_names(&data.name);

            if data.name.kind == SyntaxKind::ObjectBindingPattern
                && self.in_ctor_body_stack.last() == Some(&true)
                && let Some(init) = &data.initializer
                && init.kind == SyntaxKind::ThisKeyword
            {
                let this_type = self.get_type_of_node(init);
                self.check_this_destructuring_abstract_properties(&data.name, &this_type);
            }
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }

            let resolved_type = match (&data.type_node, &data.initializer) {
                (Some(type_node), Some(init)) => {
                    let annotation_type = self.get_type_from_type_node(type_node);

                    if init.kind == SyntaxKind::ArrayLiteralExpression {
                        let at = Arc::clone(&annotation_type);
                        self.check_contextual_elements(init, &at, init.loc);
                    }
                    let init_type = self.get_type_of_node(init);
                    let assignable = self.is_type_assignable_to(&init_type, &annotation_type);
                    let mut reported_error = false;

                    if let Some(excess_name) =
                        self.get_excess_property_name(&init_type, &annotation_type)
                    {
                        let file = self.current_file.clone();
                        let annot_str = self.type_to_string(&annotation_type);

                        let loc = self
                            .find_object_literal_property_name_node(init, &excess_name)
                            .unwrap_or(node.loc);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            loc,
                            OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                            vec![excess_name, annot_str],
                        ));
                        reported_error = true;
                    }

                    if !assignable && !reported_error {

                        self.check_type_assignable_to_and_optionally_elaborate(
                            &init_type,
                            &annotation_type,
                            Some(node),
                            Some(init),
                            None,
                            None,
                        );
                    }
                    annotation_type
                }
                (Some(type_node), None) => self.get_type_from_type_node(type_node),
                (None, Some(init)) => {

                    if data.name.kind == SyntaxKind::ArrayBindingPattern {
                        let init_type = if init.kind == SyntaxKind::Identifier
                            && let Some(sym) = self.resolve_identifier(init)
                        {
                            let flow = self
                                .program
                                .symbol_map()
                                .flow_node_of(init)
                                .map(Arc::clone);
                            self.get_narrowed_type_of_symbol(&sym, flow.as_ref())
                        } else {
                            self.get_type_of_node(init)
                        };
                        if init_type.flags.contains(TypeFlags::Never) {
                            let type_str = self.type_to_string(&init_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                data.name.loc,
                                crate::diagnostics::messages_generated::
                                    TYPE_0_MUST_HAVE_A_SYMBOL_ITERATOR_METHOD_THAT_RETURNS_AN_ITERATOR,
                                vec![type_str],
                            ));
                        }
                    }

                    let is_const_decl = self
                        .get_combined_node_flags(node)
                        .intersects(NodeFlags::Constant);
                    if !is_const_decl
                        && matches!(
                            init.kind,
                            SyntaxKind::NullKeyword | SyntaxKind::UndefinedKeyword
                        )
                    {
                        self.auto_type()
                    } else if self.is_empty_array_literal(init) {

                        self.auto_array_type()
                    } else {

                        let init_type = self.get_type_of_node(init);
                        let widened_literal =
                            self.get_widened_literal_type_for_initializer(node, &init_type);
                        let regularized = self.get_regular_type_of_literal_type(&widened_literal);
                        self.widen_initializer_type(&regularized)
                    }
                }
                (None, None) => {

                    match self.initial_type_of_declaration(node) {
                        Some(t) => t,
                        None => self.auto_type(),
                    }
                }
            };

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                let primary = symbol.value_declaration.clone();
                if let Some(primary) = primary
                    && !Arc::ptr_eq(&primary, node)
                    && symbol.declarations.len() > 1
                    && primary.kind == SyntaxKind::VariableDeclaration
                    && symbol
                        .flags
                        .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
                {
                    let auto_to_any = |t: &Arc<Type>| -> Arc<Type> {
                        if t.intrinsic_name() == Some("auto") {
                            self.get_any_type()
                        } else {
                            Arc::clone(t)
                        }
                    };
                    let primary_type = self
                        .type_node_links
                        .get(&primary)
                        .and_then(|l| l.resolved_type.clone())
                        .map(|t| auto_to_any(&t));
                    let this_type = auto_to_any(&resolved_type);
                    if let Some(primary_type) = primary_type
                        && !matches!(primary_type.intrinsic_name(), Some("error"))
                        && !matches!(this_type.intrinsic_name(), Some("error"))
                        && !self
                            .compare_types_identical(&primary_type, &this_type)
                            .is_true()
                    {
                        let name_text = data.name.text().to_string();
                        let first_str = self.type_to_string(&primary_type);
                        let next_str = self.type_to_string(&this_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                SUBSEQUENT_VARIABLE_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_VARIABLE_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                            vec![name_text, first_str, next_str],
                        ));
                    }
                }
            }

            self.type_node_links.get_or_default(node).resolved_type = Some(resolved_type.clone());

            self.type_node_links
                .get_or_default(&data.name)
                .resolved_type = Some(resolved_type.clone());

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                self.value_symbol_links
                    .get_or_default(&symbol)
                    .resolved_type = Some(resolved_type);
            }
        }
    }

    fn check_case_clause(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::CaseOrDefaultClause(data) = &node.data {
            if data.expression.kind != SyntaxKind::UnknownKeyword {
                self.check_expression(&data.expression);
            }
            for stmt in data.statements.iter() {
                self.check_statement(stmt);
            }
        }
    }

    fn check_heritage_clause(&mut self, node: &Arc<Node>) {

        let data = match &node.data {
            crate::ast::NodeData::HeritageClause(d) => d,
            _ => return,
        };
        if data.token == SyntaxKind::ExtendsKeyword {

            if data.types.len() > 1 {
                for type_ref in data.types.iter().skip(1) {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        type_ref.loc,
                        crate::diagnostics::messages_generated::
                            CLASSES_CAN_ONLY_EXTEND_A_SINGLE_CLASS,
                        Vec::new(),
                    ));
                }
            }

            for type_ref in data.types.iter() {
                if let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data {

                    if ewa.expression.kind == SyntaxKind::Identifier {
                        if let Some(sym) = self.resolve_identifier(&ewa.expression)
                            && sym.flags == SymbolFlags::Interface
                        {
                            let name = ewa.expression.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                ewa.expression.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_EXTEND_AN_INTERFACE_0_DID_YOU_MEAN_IMPLEMENTS,
                                vec![name],
                            ));
                        }
                    }

                    self.push_ts2304_suppression();
                    let _ = self.get_type_from_type_node(&ewa.expression);
                    self.pop_ts2304_suppression();
                }
            }
            return;
        }
        if data.token != SyntaxKind::ImplementsKeyword {
            return;
        }

        let class_node = match node.parent.as_ref() {
            Some(p) => p,
            None => return,
        };
        let class_data = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(d) => d,
            _ => return,
        };
        let class_name = class_data
            .name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();

        let instance_type = self.build_class_instance_type_with_base(class_node);

        for type_ref in data.types.iter() {
            let interface_type = self.get_type_from_heritage_type_reference(type_ref);
            if interface_type.flags.contains(TypeFlags::Any) {

                continue;
            }
            if !self.is_type_assignable_to(&instance_type, &interface_type) {

                let mut issued_member_error = false;
                for member in class_data.members.iter() {
                    if member.has_syntactic_modifier(ModifierFlags::Static) {
                        continue;
                    }
                    let name_node = match &member.data {
                        crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                        crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                        crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                        crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                        _ => continue,
                    };
                    let prop_name = name_node.text().to_string();
                    if prop_name.is_empty() {
                        continue;
                    }
                    let Some(prop) = self.get_property_of_type(&instance_type, &prop_name)
                    else {
                        continue;
                    };
                    let Some(base_prop) = self.get_property_of_type(&interface_type, &prop_name)
                    else {
                        continue;
                    };
                    let prop_type = self.get_type_of_symbol(&prop);
                    let base_type = self.get_type_of_symbol(&base_prop);
                    if !self.is_type_assignable_to(&prop_type, &base_type) {
                        let class_str = self.type_to_string(&instance_type);
                        let iface_str = self.type_to_string(&interface_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_node.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                            vec![prop_name, class_str, iface_str],
                        ));
                        issued_member_error = true;
                        break;
                    }
                }
                if !issued_member_error {
                    let iface_name = self.type_to_string(&interface_type);
                    self.grammar_error_on_node_with_args(
                        class_node,
                        &crate::diagnostics::messages_generated::CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1,
                        &[class_name.clone(), iface_name],
                    );
                }
            }
        }
    }

    #[allow(dead_code)]
    fn build_class_instance_type(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        self.build_interface_type_from_members(members)
    }

    pub(crate) fn build_class_instance_type_with_base(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (members, heritage_clauses) = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }

            crate::ast::NodeData::ClassExpression(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.build_interface_type_from_members(&Arc::new(NodeList::default())),
        };

        let own_type = self.build_interface_type_from_members(members);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let own_mut = Arc::as_ptr(&own_type) as *mut crate::checker::types::Type;
            unsafe {
                (*own_mut).symbol = Some(Arc::clone(class_sym));
            }
        }

        let mut base_type: Option<Arc<Type>> = None;
        if let Some(ref heritage) = heritage_clauses {
            for clause in heritage.iter() {
                if let crate::ast::NodeData::HeritageClause(hc) = &clause.data {
                    if hc.token == SyntaxKind::ExtendsKeyword {
                        if let Some(type_ref) = hc.types.iter().next() {
                            base_type = Some(self.resolve_base_class_instance_type(type_ref));
                        }
                        break;
                    }
                }
            }
        }
        match base_type {
            Some(base) => self.merge_instance_types(&own_type, &base),
            None => own_type,
        }
    }

    fn resolve_base_class_constructor_type(&mut self) -> Option<Arc<Type>> {
        let (base_node, symbol) = self.base_class_node_of_enclosing_class()?;

        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return None;
        }
        let ctor_type = self.get_type_of_class_declaration(&base_node);
        self.resolving_type_aliases.remove(&key);
        Some(ctor_type)
    }

    fn base_class_node_of_enclosing_class(&self) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let class_node = self.enclosing_class_stack.last().cloned()?;
        self.extends_base_of(&class_node)
    }

    fn resolve_base_class_instance_type(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {

        if let crate::ast::NodeData::ExpressionWithTypeArguments(data) = &type_ref.data {
            if data.expression.kind == SyntaxKind::Identifier {
                if let Some(symbol) = self.resolve_identifier(&data.expression) {
                    if symbol.flags.contains(SymbolFlags::Class) {

                        if self.type_resolution_stack.len() >= 200 {
                            return self.get_any_type();
                        }

                        if let Some(class_node) = symbol
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                            .cloned()
                        {

                            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
                            if !self.push_type_resolution(
                                key,
                                TypeResolutionProperty::ResolvedBaseTypes,
                            ) {
                                return self.get_any_type();
                            }

                            let heritage_args = data.type_arguments.clone();
                            let base_tps: Vec<Arc<crate::ast::Symbol>> = match &class_node.data {
                                crate::ast::NodeData::ClassDeclaration(cd) => {
                                    match &cd.type_parameters {
                                        Some(tps) => tps
                                            .iter()
                                            .filter_map(|tp| {
                                                self.program
                                                    .symbol_map()
                                                    .symbol_of(tp)
                                                    .map(Arc::clone)
                                            })
                                            .collect(),
                                        None => Vec::new(),
                                    }
                                }
                                _ => Vec::new(),
                            };
                            let pushed = if let Some(args) = &heritage_args
                                && !base_tps.is_empty()
                            {
                                let arg_types: Vec<Arc<Type>> = args
                                    .iter()
                                    .map(|a| self.get_type_from_type_node(a))
                                    .collect();
                                let mut mapping = HashMap::new();
                                let mut name_frame: Vec<(Arc<Symbol>, Arc<Type>)> = Vec::new();
                                for (i, tp_sym) in base_tps.iter().enumerate() {
                                    if i < arg_types.len() {
                                        mapping.insert(
                                            Arc::as_ptr(tp_sym) as *const crate::ast::Symbol,
                                            Arc::clone(&arg_types[i]),
                                        );
                                        name_frame
                                            .push((Arc::clone(tp_sym), Arc::clone(&arg_types[i])));
                                    }
                                }
                                self.type_argument_stack.push(mapping);
                                self.type_argument_name_frames.push(name_frame);
                                true
                            } else {
                                false
                            };
                            let instance = {

                                self.push_scope(&class_node);
                                let i = self.build_class_instance_type_with_base(&class_node);
                                self.pop_scope();
                                i
                            };
                            if pushed {
                                self.type_argument_stack.pop();
                                self.type_argument_name_frames.pop();
                            }
                            self.pop_type_resolution();
                            return instance;
                        }
                    }
                }
            }
        }

        let t = self.get_type_from_type_node(type_ref);
        if t.flags.contains(TypeFlags::Any) {
            return self.get_any_type();
        }

        if t.flags.contains(TypeFlags::Object) {
            return t;
        }
        self.get_any_type()
    }

    fn merge_instance_types(&mut self, derived: &Arc<Type>, base: &Arc<Type>) -> Arc<Type> {
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };

        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();

        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }

        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }

        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());

        let mut call_signatures: Vec<Arc<Signature>> =
            derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,

            symbol: derived.symbol.clone(),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count
                        + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn get_type_from_heritage_type_reference(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {
        self.get_type_from_type_node(type_ref)
    }

    fn check_property_initialization(&mut self, class_node: &Arc<Node>) {
        if !self.strict_null_checks || !self.strict_property_initialization {
            return;
        }

        if class_node.has_syntactic_modifier(ModifierFlags::Ambient)
            || self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        let members = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            _ => return,
        };

        let constructor = members.iter().find(|m| m.kind == SyntaxKind::Constructor);
        for member in members.iter() {
            if member.kind != SyntaxKind::PropertyDeclaration {
                continue;
            }

            let mods = self.get_combined_modifier_flags(member);
            if mods.contains(ModifierFlags::Ambient) || mods.contains(ModifierFlags::Static) {
                continue;
            }

            if mods.contains(ModifierFlags::Abstract) {
                continue;
            }
            let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data else {
                continue;
            };

            if pd.initializer.is_some() || pd.postfix_token.is_some() {
                continue;
            }

            let name_node = &pd.name;
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier
                    | SyntaxKind::PrivateIdentifier
                    | SyntaxKind::ComputedPropertyName
            ) {
                continue;
            }

            let Some(type_node) = &pd.type_node else {
                continue;
            };
            let prop_type = self.get_type_from_type_node(type_node);
            if prop_type
                .flags
                .intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Undefined)
                || type_contains_undefined(&prop_type)
            {
                continue;
            }

            if let Some(ctor) = constructor {
                if self.is_property_assigned_in_constructor(name_node, ctor) {
                    continue;
                }
            }

            let name_text = self.node_text(name_node);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_node.loc,
                PROPERTY_0_HAS_NO_INITIALIZER_AND_IS_NOT_DEFINITELY_ASSIGNED_IN_THE_CONSTRUCTOR,
                vec![name_text],
            ));
        }
    }

    fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            crate::ast::NodeData::Identifier(d) => d.text.clone(),
            crate::ast::NodeData::PrivateIdentifier(d) => d.text.clone(),
            crate::ast::NodeData::ComputedPropertyName(_) => {

                let Some(file) = &self.current_file else {
                    return String::new();
                };
                let pos = node.loc.pos();
                let end = node.loc.end();
                if pos < end && end <= file.text.len() {
                    file.text[pos..end].to_string()
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    #[allow(dead_code)]
    fn resolve_property_name(
        &mut self,
        _member: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {

        self.resolve_identifier(name)
    }

    fn is_property_assigned_in_constructor(&self, name_node: &Arc<Node>, ctor: &Arc<Node>) -> bool {
        let name_text = match &name_node.data {
            crate::ast::NodeData::Identifier(d) => d.text.as_str(),
            _ => return false,
        };

        let body = match &ctor.data {
            crate::ast::NodeData::ConstructorDeclaration(d) => &d.body,
            _ => return false,
        };
        let Some(body) = body else {
            return false;
        };
        Self::node_contains_this_assignment(body, name_text)
    }

    fn node_contains_this_assignment(node: &Arc<Node>, name: &str) -> bool {

        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            if data.operator_token.kind == SyntaxKind::EqualsToken {
                if Self::is_this_property_access(&data.left, name) {
                    return true;
                }
            }
        }

        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if Self::node_contains_this_assignment(child, name) {
                found = true;
                return true;
            }
            false
        });
        found
    }

    fn is_this_property_access(node: &Arc<Node>, name: &str) -> bool {
        match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let crate::ast::NodeData::Identifier(id) = &data.name.data {
                        return id.text == name;
                    }
                }
                false
            }
            crate::ast::NodeData::ElementAccessExpression(data) => {
                if data.expression.kind == SyntaxKind::ThisKeyword {
                    if let crate::ast::NodeData::StringLiteral(sl) = &data.argument_expression.data
                    {
                        return sl.text == name;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn class_member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            _ => None,
        }
    }

    fn class_member_name_text(node: &Arc<Node>) -> Option<String> {
        if matches!(node.kind, SyntaxKind::Constructor) {
            return Some("constructor".to_string());
        }
        let name = Self::class_member_name_node(node)?;
        match name.kind {

            SyntaxKind::Identifier | SyntaxKind::NumericLiteral => {
                let text = name.text().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            SyntaxKind::StringLiteral => Some(format!("\"{}\"", name.text())),
            _ => None,
        }
    }

    fn class_member_has_body(node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::MethodDeclaration(d) if d.body.is_some()
        ) || matches!(
            &node.data,
            crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some()
        )
    }

    fn function_like_params_and_return(
        node: &Arc<Node>,
    ) -> Option<(&Arc<NodeList>, Option<&Arc<Node>>)> {
        match &node.data {
            crate::ast::NodeData::FunctionDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            crate::ast::NodeData::MethodDeclaration(d) => {
                Some((&d.parameters, d.type_node.as_ref()))
            }
            crate::ast::NodeData::ConstructorDeclaration(d) => Some((&d.parameters, None)),
            _ => None,
        }
    }

    fn overload_signature_compatible_with_implementation(
        &mut self,
        overload: &Arc<Node>,
        implementation: &Arc<Node>,
    ) -> bool {
        let Some((ov_params, ov_return)) = Self::function_like_params_and_return(overload)
            .map(|(p, r)| (Arc::clone(p), r.cloned()))
        else {
            return true;
        };
        let Some((im_params, im_return)) = Self::function_like_params_and_return(implementation)
            .map(|(p, r)| (Arc::clone(p), r.cloned()))
        else {
            return true;
        };

        let return_ok = match (ov_return, im_return) {
            (Some(ovn), Some(imn)) => {
                let ov_t = self.get_type_from_type_node(&ovn);
                let im_t = self.get_type_from_type_node(&imn);
                ov_t.flags.contains(TypeFlags::Void)
                    || self.is_type_assignable_to(&ov_t, &im_t)
                    || self.is_type_assignable_to(&im_t, &ov_t)
            }
            _ => true,
        };
        if !return_ok {
            return false;
        }

        let n = ov_params.len().min(im_params.len());
        for i in 0..n {
            let ov_tn = match &ov_params.nodes[i].data {
                crate::ast::NodeData::ParameterDeclaration(p) => p.type_node.as_ref(),
                _ => None,
            };
            let im_tn = match &im_params.nodes[i].data {
                crate::ast::NodeData::ParameterDeclaration(p) => p.type_node.as_ref(),
                _ => None,
            };
            let (Some(o), Some(m)) = (ov_tn, im_tn) else {
                continue;
            };
            let ov_t = self.get_type_from_type_node(&o);
            let im_t = self.get_type_from_type_node(&m);
            if !self.is_type_assignable_to(&ov_t, &im_t)
                && !self.is_type_assignable_to(&im_t, &ov_t)
            {
                return false;
            }
        }
        true
    }

    fn check_class_member_overloads(&mut self, members: &NodeList) {

        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, m) in members.iter().enumerate() {
            if !matches!(m.kind, SyntaxKind::Constructor | SyntaxKind::MethodDeclaration) {
                continue;
            }
            if let Some(name) = Self::class_member_name_text(m) {
                groups.entry(name).or_default().push(idx);
            }
        }
        for (_, idxs) in groups {

            let mut prev: Option<usize> = None;
            let mut has_body = false;
            for &idx in &idxs {
                let node = &members.nodes[idx];
                if !Self::class_member_has_body(node) {
                    if let Some(p) = prev {
                        if p + 1 != idx {
                            self.report_implementation_expected_error(members, p);
                        }
                    }
                } else {
                    has_body = true;
                }
                prev = Some(idx);
            }
            let last = idxs[idxs.len() - 1];
            if !has_body {
                let node = &members.nodes[last];
                let exempt = node.has_syntactic_modifier(ModifierFlags::Abstract)
                    || matches!(
                        &node.data,
                        crate::ast::NodeData::MethodDeclaration(d) if d.postfix_token.is_some()
                    );
                if !exempt {
                    self.report_implementation_expected_error(members, last);
                }
            } else {

                let impl_idx = idxs
                    .iter()
                    .copied()
                    .find(|&i| Self::class_member_has_body(&members.nodes[i]))
                    .unwrap_or(last);
                let impl_node = Arc::clone(&members.nodes[impl_idx]);
                for &i in &idxs {
                    if i == impl_idx {
                        continue;
                    }
                    let overload = Arc::clone(&members.nodes[i]);
                    if !self.overload_signature_compatible_with_implementation(&overload, &impl_node)
                        && let Some(name_node) = crate::ast::utilities::get_name_of_declaration(
                            &overload,
                        )
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_node.loc,
                            crate::diagnostics::messages_generated::
                                THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                            Vec::new(),
                        ));
                    }
                }
            }
        }
    }

    fn report_implementation_expected_error(&mut self, members: &NodeList, idx: usize) {
        let node = Arc::clone(&members.nodes[idx]);
        let name_text = Self::class_member_name_text(&node);
        if let Some(sib) = members.nodes.get(idx + 1) {
            if sib.kind == node.kind {
                let sib_name = Self::class_member_name_text(sib);
                let same_name = match (&name_text, &sib_name) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                };

                if same_name {
                    return;
                }
                if Self::class_member_has_body(sib) {

                    let file = self.current_file.clone();
                    let loc = Self::class_member_name_node(sib)
                        .map(|n| n.loc)
                        .unwrap_or(sib.loc);
                    let display_name = name_text.unwrap_or_default();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        loc,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![display_name],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }

        let file = self.current_file.clone();
        let (loc, message): (crate::core::text::TextRange, crate::diagnostics::Message) =
            if matches!(node.kind, SyntaxKind::Constructor) {
                (
                    node.loc,
                    crate::diagnostics::messages_generated::CONSTRUCTOR_IMPLEMENTATION_IS_MISSING,
                )
            } else {
                (
                    Self::class_member_name_node(&node)
                        .map(|n| n.loc)
                        .unwrap_or(node.loc),
                    crate::diagnostics::messages_generated::
                        FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                )
            };
        let diagnostic = crate::ast::Diagnostic::new(file, loc, message, Vec::new());
        self.diagnostics.add(diagnostic);
    }

    fn check_parameter_property_modifiers(&mut self, params: &NodeList, is_ctor_impl: bool) {
        for param in params.iter() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };

            if pd.modifiers.is_some() {
                self.check_grammar_modifiers(param);
            }
            let Some(modifiers) = &pd.modifiers else { continue };
            if is_ctor_impl {
                continue;
            }
            if modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) {
                let file = self.current_file.clone();
                let diagnostic = crate::ast::Diagnostic::new(
                    file,
                    param.loc,
                    crate::diagnostics::messages_generated::
                        A_PARAMETER_PROPERTY_IS_ONLY_ALLOWED_IN_A_CONSTRUCTOR_IMPLEMENTATION,
                    Vec::new(),
                );
                self.diagnostics.add(diagnostic);
            }
        }
    }

    fn check_parameter_implicit_any(
        &mut self,
        node: &Arc<Node>,
        params: &NodeList,
        contextual_param_count: usize,
    ) {
        if !self.no_implicit_any {
            return;
        }
        for (i, param) in params.iter().enumerate() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.type_node.is_some() || pd.initializer.is_some() {
                continue;
            }
            let name = &pd.name;
            if name.kind != SyntaxKind::Identifier || name.text() == "this" {
                continue;
            }

            if i < contextual_param_count {
                continue;
            }

            if self.param_has_typed_jsdoc_tag(node, name.text()) {
                continue;
            }
            let file = self.current_file.clone();
            let name_text = name.text().to_string();
            let diagnostic = if pd.dot_dot_dot_token.is_some() {
                crate::ast::Diagnostic::new(
                    file,
                    param.loc,
                    crate::diagnostics::messages_generated::
                        REST_PARAMETER_0_IMPLICITLY_HAS_AN_ANY_TYPE,
                    vec![name_text],
                )
            } else {
                crate::ast::Diagnostic::new(
                    file,
                    param.loc,
                    crate::diagnostics::messages_generated::PARAMETER_0_IMPLICITLY_HAS_AN_1_TYPE,
                    vec![name_text, "any".to_string()],
                )
            };
            self.diagnostics.add(diagnostic);
        }
    }

    fn param_has_typed_jsdoc_tag(&self, node: &Arc<Node>, param_name: &str) -> bool {
        let Some(file) = &self.current_file else {
            return false;
        };
        for jsdoc in file.resolve_jsdoc(node) {
            let crate::ast::NodeData::JSDoc(d) = &jsdoc.data else {
                continue;
            };
            let Some(tags) = &d.tags else { continue };
            for tag in tags.iter() {
                if let crate::ast::NodeData::JSDocParameterOrPropertyTag(td) = &tag.data
                    && td.name.kind == SyntaxKind::Identifier
                    && td.name.text() == param_name
                    && td.type_expression.is_some()
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_type_annotation(&mut self, tn: &Arc<Node>) {

        self.with_declaring_file_context(tn, |c| c.check_type_annotation_inner(tn));
    }

    fn check_type_annotation_inner(&mut self, tn: &Arc<Node>) {
        match tn.kind {
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType => {
                let (params, return_type): (&NodeList, Option<&Arc<Node>>) = match &tn.data {
                    crate::ast::NodeData::FunctionTypeNode(d) => {
                        (&d.parameters, d.type_node.as_ref())
                    }
                    crate::ast::NodeData::ConstructorTypeNode(d) => {
                        (&d.parameters, d.type_node.as_ref())
                    }
                    _ => return,
                };
                self.check_parameter_property_modifiers(params, false);
                self.check_parameter_implicit_any(tn, params, 0);
                for p in params.iter() {
                    if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                        && let Some(pt) = &pd.type_node
                    {
                        self.check_type_annotation(pt);
                    }
                }
                if let Some(rt) = return_type {
                    self.check_type_annotation(rt);
                }
            }
            SyntaxKind::TypeReference => {
                if let crate::ast::NodeData::TypeReferenceNode(d) = &tn.data
                    && let Some(args) = &d.type_arguments
                {
                    for a in args.iter() {
                        self.check_type_annotation(a);
                    }
                }
            }
            SyntaxKind::UnionType | SyntaxKind::IntersectionType => {
                if let crate::ast::NodeData::UnionTypeNode(d) = &tn.data {
                    for t in d.types.iter() {
                        self.check_type_annotation(t);
                    }
                }
                if let crate::ast::NodeData::IntersectionTypeNode(d) = &tn.data {
                    for t in d.types.iter() {
                        self.check_type_annotation(t);
                    }
                }
            }
            SyntaxKind::ParenthesizedType => {
                if let crate::ast::NodeData::ParenthesizedTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.type_node);
                }
            }
            SyntaxKind::ArrayType | SyntaxKind::TypeOperator => {
                if let crate::ast::NodeData::ArrayTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.element_type);
                }
                if let crate::ast::NodeData::TypeOperatorNode(d) = &tn.data {
                    self.check_type_annotation(&d.type_node);
                }
            }
            SyntaxKind::TupleType => {
                if let crate::ast::NodeData::TupleTypeNode(d) = &tn.data {
                    for t in d.elements.iter() {
                        self.check_type_annotation(t);
                    }
                }
            }
            SyntaxKind::IndexedAccessType => {
                if let crate::ast::NodeData::IndexedAccessTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.object_type);
                    self.check_type_annotation(&d.index_type);

                    self.check_indexed_access_index_type(tn);
                }
            }
            SyntaxKind::TypeLiteral => {

                if let crate::ast::NodeData::TypeLiteralNode(d) = &tn.data {
                    for member in d.members.iter() {
                        if matches!(
                            member.kind,
                            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                        ) {
                            self.check_accessor_in_type_context(member);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_indexed_access_index_type(&mut self, node: &Arc<Node>) {
        use crate::checker::types::{TypeData, TypeFlags};
        let t = self.get_type_from_type_node(node);

        if !self.type_argument_stack.is_empty() {
            return;
        }

        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
        {
            return;
        }
        let (object_type, index_type) = match &t.data {
            TypeData::IndexedAccess(d) => match (&d.object_type, &d.index_type) {
                (Some(o), Some(i)) => (Arc::clone(o), Arc::clone(i)),
                _ => return,
            },
            _ => return,
        };

        if object_type
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown)
        {
            return;
        }

        if self.type_flags_is_generic_object_type(&object_type) {
            return;
        }

        let object_index_type = self.get_index_type(&object_type);
        let has_number_index_info = self
            .get_index_info_of_type(&object_type, &self.number_type())
            .is_some();

        let constituents: Vec<Arc<Type>> = if index_type.flags.contains(TypeFlags::Union) {
            match &index_type.data {
                TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![Arc::clone(&index_type)],
            }
        } else {
            vec![Arc::clone(&index_type)]
        };
        for c in &constituents {
            let mut ok = self.is_type_assignable_to(c, &object_index_type);
            if !ok && has_number_index_info {

                ok = self.is_type_assignable_to(c, &self.number_type());
            }
            if ok {
                continue;
            }
            if object_type.object_flags.intersects(
                crate::checker::types::ObjectFlags::IsGenericObjectType,
            ) {

                if let Some(name) = self.property_name_from_index(c) {
                    if let Some(sym) = self.get_constituent_property(&object_type, &name) {
                        let non_public = sym
                            .value_declaration
                            .as_ref()
                            .map(|d| {
                                self.get_combined_modifier_flags(d).intersects(
                                    crate::ast::ModifierFlags::NonPublicAccessibilityModifier,
                                )
                            })
                            .unwrap_or(false);
                        if non_public {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    PRIVATE_OR_PROTECTED_MEMBER_0_CANNOT_BE_ACCESSED_ON_A_TYPE_PARAMETER,
                                vec![name],
                            ));
                            return;
                        }
                    }
                }
            }
            let index_display = self.type_to_string(&index_type);
            let object_display = self.type_to_string(&object_type);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::TYPE_0_CANNOT_BE_USED_TO_INDEX_TYPE_1,
                vec![index_display, object_display],
            ));
            return;
        }
    }

    fn property_name_from_index(&mut self, t: &Arc<Type>) -> Option<String> {
        use crate::checker::types::{TypeData, TypeFlags};
        if t.flags.intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral) {
            if let TypeData::Literal(l) = &t.data {
                return match &l.value {
                    crate::checker::types::LiteralValue::String(s) => Some(s.clone()),
                    crate::checker::types::LiteralValue::Number(n) => Some(n.to_string()),
                    _ => None,
                };
            }
        }
        None
    }

    pub(crate) fn get_constituent_property(
        &mut self,
        object_type: &Arc<Type>,
        name: &str,
    ) -> Option<std::sync::Arc<crate::ast::Symbol>> {
        let apparent = self.get_apparent_type(object_type);
        let parts: Vec<Arc<Type>> = if apparent.flags.contains(
            crate::checker::types::TypeFlags::Union,
        ) {
            match &apparent.data {
                crate::checker::types::TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![apparent],
            }
        } else {
            vec![apparent]
        };
        for p in parts {
            if let Some(sym) = self.get_property_of_type(&p, name) {
                return Some(sym);
            }
        }
        None
    }

    fn loop_has_escaping_break(n: &Arc<Node>, direct: bool) -> bool {
        match n.kind {
            SyntaxKind::BreakStatement => {
                matches!(
                    &n.data,
                    crate::ast::NodeData::BreakStatement(d) if d.label.is_some()
                ) || direct
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => false,
            _ => {

                let nested = matches!(
                    n.kind,
                    SyntaxKind::WhileStatement
                        | SyntaxKind::DoStatement
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::SwitchStatement
                );
                let mut found = false;
                crate::ast::node_data_generated::for_each_child(n, |child| {
                    if Self::loop_has_escaping_break(child, direct && !nested) {
                        found = true;
                        true
                    } else {
                        false
                    }
                });
                found
            }
        }
    }

    fn function_body_has_explicit_return(body: &Arc<Node>) -> bool {
        fn walk(n: &Arc<Node>) -> bool {
            match n.kind {
                SyntaxKind::ReturnStatement => return true,

                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return false,
                _ => {}
            }
            let mut found = false;
            crate::ast::node_data_generated::for_each_child(n, |child| {
                if walk(child) {
                    found = true;
                    true
                } else {
                    false
                }
            });
            found
        }
        walk(body)
    }

    fn report_get_accessor_call(&mut self, callee_expr: &Arc<Node>) -> bool {
        let crate::ast::NodeData::PropertyAccessExpression(pa) = &callee_expr.data else {
            return false;
        };
        if pa.name.kind != SyntaxKind::Identifier {
            return false;
        }
        let target_type = self.get_type_of_node(&pa.expression);
        let name = pa.name.text().to_string();
        let is_getter = target_type
            .as_structured()
            .and_then(|s| s.properties.iter().find(|p| p.name == name))
            .is_some_and(|sym| sym.flags.contains(SymbolFlags::GetAccessor));
        if !is_getter {
            return false;
        }
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,

            pa.name.loc,
            crate::diagnostics::messages_generated::
                THIS_EXPRESSION_IS_NOT_CALLABLE_BECAUSE_IT_IS_A_GET_ACCESSOR_DID_YOU_MEAN_TO_USE_IT_WITHOUT,
            vec![],
        ));
        true
    }

    pub(crate) fn has_same_named_type_symbol(&self, name: &str) -> bool {
        let type_meaning = SymbolFlags::Interface
            | SymbolFlags::Class
            | SymbolFlags::TypeParameter
            | SymbolFlags::TypeAlias
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum;
        let symbol_map = self.program.symbol_map();
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && sym.flags.intersects(type_meaning)
            {
                return true;
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id)
                && (container_sym
                    .members
                    .get(name)
                    .is_some_and(|s| s.flags.intersects(type_meaning))
                    || container_sym
                        .exports
                        .get(name)
                        .is_some_and(|s| s.flags.intersects(type_meaning)))
            {
                return true;
            }
        }
        self.globals
            .get(name)
            .is_some_and(|s| s.flags.intersects(type_meaning))
    }

    pub(crate) fn namespace_usable_as_value(&mut self, namespace: &Arc<Symbol>) -> bool {
        let state_instantiated = namespace
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .any(|d| {
                module_is_instantiated(d, self.compiler_options.should_preserve_const_enums())
            });
        state_instantiated || self.namespace_has_value_side(namespace)
    }

    fn namespace_has_value_side(&mut self, namespace: &Arc<Symbol>) -> bool {
        let value_flags = SymbolFlags::Function
            | SymbolFlags::Class
            | SymbolFlags::FunctionScopedVariable
            | SymbolFlags::BlockScopedVariable
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum
            | SymbolFlags::Method;
        let has_value_member = |table: &crate::ast::SymbolTable| {
            table.iter().any(|(name, s)| {
                name != "export="
                    && s.flags.intersects(value_flags)

                    && s.declarations.iter().any(|d| {
                        !matches!(
                            d.kind,
                            SyntaxKind::Parameter | SyntaxKind::MethodSignature
                        )
                    })
            })
        };
        if has_value_member(&namespace.exports) || has_value_member(&namespace.members) {
            return true;
        }

        for d in &namespace.declarations {
            if d.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            let entries: Vec<(String, Arc<Symbol>)> = self
                .program
                .symbol_map()
                .locals
                .get(&d.id())
                .map(|table| {
                    table
                        .iter()
                        .map(|(k, v)| (k.clone(), Arc::clone(v)))
                        .collect()
                })
                .unwrap_or_default();
            if entries.iter().any(|(name, s)| {
                name != "export="
                    && (s.flags.intersects(value_flags)
                        || (s.flags.contains(SymbolFlags::ValueModule)
                            && self.namespace_has_value_side(s)))
            }) {
                return true;
            }
        }

        if self.namespace_value_depth < 4 {
            self.namespace_value_depth += 1;
            let nested = namespace
                .exports
                .iter()
                .chain(namespace.members.iter())
                .any(|(name, s)| {
                    name != "export="
                        && s.flags.contains(SymbolFlags::ValueModule)
                        && self.namespace_has_value_side(s)
                });
            self.namespace_value_depth -= 1;
            if nested {
                return true;
            }
        }

        for decl in &namespace.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals.iter().any(|(name, s)| {
                    name != "export=" && s.flags.intersects(value_flags)
                })
            {
                return true;
            }
        }

        if let Some(export_equals) = namespace.exports.get("export=") {
            for decl in &export_equals.declarations {
                if let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {

                    let scope_decl = namespace
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(scope_decl) = scope_decl {
                        self.push_scope(&scope_decl);
                        let target = self.resolve_qualified_symbol(&ea.expression);
                        self.pop_scope();
                        if let Some(target) = target {
                            if target.flags.intersects(value_flags) {
                                return true;
                            }
                            if target.flags.contains(SymbolFlags::ValueModule) {
                                return self.namespace_has_value_side(&target);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn resolve_module_member_symbol(
        &mut self,
        module_sym: &Arc<Symbol>,
        name: &str,
        depth: usize,
    ) -> Option<Arc<Symbol>> {
        if depth == 0 {
            return None;
        }
        let sym = self.namespace_member_recursive(module_sym, name);
        if let Some(sym) = &sym {

            if let Some(target) = &sym.export_symbol
                && !Arc::ptr_eq(target, &sym)
            {
                return Some(Arc::clone(target));
            }
        }

        let mut clause_hits: Vec<(String, Option<String>)> = Vec::new();
        self.for_each_module_statement(module_sym, |stmt| {
            if let crate::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let crate::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let crate::ast::NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        let imported = spec
                            .property_name
                            .as_ref()
                            .unwrap_or(&spec.name)
                            .text()
                            .trim_matches(['"', '\'', '`'])
                            .to_string();
                        let module_text = d.module_specifier.as_ref().map(|module_spec| {
                            module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string()
                        });
                        clause_hits.push((imported, module_text));
                        return true;
                    }
                }
            }
            false
        });
        for (imported, module_text) in clause_hits {
            let target_module = match module_text {

                None => Arc::clone(module_sym),
                Some(text) => match self.resolve_module_spec_from(module_sym, &text) {
                    Some(m) => m,
                    None => continue,
                },
            };
            if let Some(target) =
                self.resolve_module_member_symbol(&target_module, &imported, depth - 1)
            {
                return Some(target);
            }
        }
        sym
    }

    pub(crate) fn resolve_module_spec_from(
        &self,
        base_module: &Arc<Symbol>,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {
            return self.resolve_module_file_symbol(specifier);
        }
        let dir = base_module
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|f| {
                f.file_name
                    .rfind('/')
                    .map(|i| f.file_name[..i].to_string())
                    .unwrap_or_default()
            })?;
        self.resolve_module_file_symbol_in(&dir, specifier)
    }

    fn type_of_dynamic_import(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        let spec = self.spec_of_dynamic_import_call(node)?;
        if spec.is_empty() {
            return None;
        }
        let cur = self.current_file.clone()?;

        let module_sym = match self.resolve_module_file_symbol(&spec) {
            Some(s) => s,
            None => {
                let path = self.program.resolve_external_module_path(
                    &spec,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::ESNext,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()?
            }
        };
        Some(self.resolve_namespace_type(&module_sym))
    }

    fn spec_of_dynamic_import_call(&self, node: &Arc<Node>) -> Option<String> {
        if node.kind != SyntaxKind::CallExpression {
            return None;
        }
        let (callee, args) = match &node.data {
            NodeData::CallExpression(d) => (&d.expression, &d.arguments),
            _ => return None,
        };
        if callee.kind != SyntaxKind::ImportKeyword {
            return None;
        }
        let spec_node = args.iter().next()?;
        if spec_node.kind != SyntaxKind::StringLiteral {
            return None;
        }
        Some(spec_node.text().trim_matches(['"', '\'', '`']).to_string())
    }

    pub(crate) fn type_of_imported_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {

        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &decl.data else {
                return None;
            };
            if ied.module_reference.kind == SyntaxKind::ExternalModuleReference {
                let ext = &ied.module_reference;
                let crate::ast::NodeData::ExternalModuleReference(emr) = &ext.data else {
                    return None;
                };
                let module_spec = emr.expression.text().to_string();
                let module_text_trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let module_sym = match self.resolve_module_file_symbol(&module_spec) {
                    Some(s) => s,
                    None => {
                        let Some(cur) = self.current_file.clone() else {
                            return None;
                        };
                        let Some(path) = self.program.resolve_external_module_path(
                            &module_text_trimmed,
                            &cur.file_name,
                            crate::core::compiler_options::ModuleKind::None,
                        ) else {
                            return None;
                        };
                        let Some(sf) = self.program.get_source_file(&path) else {
                            return None;
                        };
                        let Some(sym) =
                            self.program.symbol_map().symbol_of(&sf.node).cloned()
                        else {
                            return None;
                        };
                        sym
                    }
                };

                if let Some(eq) =
                    module_sym.exports.get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                {
                    let entity_decl = eq
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .cloned();
                    let scope_decl = module_sym
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(export_decl) = entity_decl
                        && let crate::ast::NodeData::ExportAssignment(ea) = &export_decl.data
                        && ea.is_export_equals
                        && matches!(
                            ea.expression.kind,
                            SyntaxKind::Identifier | SyntaxKind::QualifiedName
                        )
                    {
                        if let Some(scope) = scope_decl {
                            self.push_scope(&scope);
                            let target = self.resolve_qualified_symbol(&ea.expression);
                            self.pop_scope();
                            if let Some(t) = target {
                                return Some(self.get_type_of_symbol(&t));
                            }
                        } else {

                            let mut segments: Vec<String> = Vec::new();
                            let mut cur = &ea.expression;
                            loop {
                                match &cur.data {
                                    crate::ast::NodeData::Identifier(id) => {
                                        segments.push(id.text.clone());
                                        break;
                                    }
                                    crate::ast::NodeData::QualifiedName(q) => {
                                        segments.push(q.right.text().to_string());
                                        cur = &q.left;
                                    }
                                    _ => break,
                                }
                            }
                            segments.reverse();
                            if let Some(first) = segments.first()
                                && let Some(mut target) =
                                    self.resolve_module_member_symbol(&module_sym, first, 8)
                            {
                                let mut ok = true;
                                for seg in segments.iter().skip(1) {
                                    match target
                                        .exports
                                        .get(seg)
                                        .or_else(|| target.members.get(seg))
                                        .cloned()
                                    {
                                        Some(next) => target = next,
                                        None => {
                                            ok = false;
                                            break;
                                        }
                                    }
                                }
                                if ok {
                                    return Some(self.get_type_of_symbol(&target));
                                }
                            }
                        }
                    }
                }
                return Some(self.resolve_namespace_type(&module_sym));
            }

            let target = &ied.module_reference;
            let t = self.get_type_of_node(target);
            if t.flags.contains(TypeFlags::Any)
                && t.intrinsic_name() == Some("any")
                && self.resolve_identifier(target).is_none()
            {
                return None;
            }
            return Some(t);
        }
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportSpecifier)?;
        let name = match &decl.data {

            crate::ast::NodeData::ImportSpecifier(d) => d
                .property_name
                .as_ref()
                .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
            _ => return None,
        };

        let mut import_decl = decl.parent.as_ref()?;
        while !matches!(import_decl.data, crate::ast::NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            crate::ast::NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_text_trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
        let module_sym = match self.resolve_module_file_symbol(&module_spec) {
            Some(s) => s,
            None => {

                let Some(cur) = self.current_file.clone() else {
                    return None;
                };
                let Some(path) = self.program.resolve_external_module_path(
                    &module_text_trimmed,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                ) else {
                    return None;
                };
                let Some(sf) = self.program.get_source_file(&path) else {
                    return None;
                };
                let Some(sym) = self.program.symbol_map().symbol_of(&sf.node).cloned() else {
                    return None;
                };
                sym
            }
        };
        let Some(member) = self.resolve_module_member_symbol(&module_sym, &name, 8) else {

            if name == "default"
                && self.program.options().allow_synthetic_default_imports.is_true()
            {
                return Some(self.get_any_type());
            }
            return None;
        };
        if let Some(t) = self
            .value_symbol_links
            .get(&member)
            .and_then(|l| l.resolved_type.clone())
        {
            return Some(t);
        }
        for d in &member.declarations {
            match d.kind {
                SyntaxKind::FunctionDeclaration => {
                    return Some(self.get_type_of_function_like(d));
                }
                SyntaxKind::ClassDeclaration => {
                    return Some(self.get_type_of_class_declaration(d));
                }
                _ => {}
            }
        }

        Some(self.get_type_of_symbol(&member))
    }

    fn object_literal_export_member(
        &self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let ea_sym = namespace.exports.get("export=")?;
        for d in &ea_sym.declarations {
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
                && let crate::ast::NodeData::ObjectLiteralExpression(ol) = &ea.expression.data
            {
                for prop in ol.properties.iter() {
                    if prop.text() == name
                        && let Some(s) = self.program.symbol_map().symbol_of(prop)
                    {
                        return Some(Arc::clone(s));
                    }
                }
            }
        }
        None
    }

    fn heritage_type_arguments_for_base(
        &mut self,
        base_sym: &Arc<Symbol>,
    ) -> Option<Vec<Arc<Type>>> {
        let class_node = self.enclosing_class_stack.last().cloned()?;
        let heritage = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        for clause in heritage?.iter() {
            let crate::ast::NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            if hc.token != SyntaxKind::ExtendsKeyword {
                continue;
            }
            for type_ref in hc.types.iter() {
                let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data
                else {
                    continue;
                };
                let type_args = ewa.type_arguments.as_ref()?;
                if ewa.expression.kind == SyntaxKind::Identifier
                    && let Some(sym) = self.resolve_identifier(&ewa.expression)
                    && Arc::ptr_eq(&sym, base_sym)
                {
                    return Some(
                        type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect(),
                    );
                }
            }
        }
        None
    }

    fn namespace_member_recursive(
        &mut self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if let Some(s) = namespace.exports.get(name).or_else(|| namespace.members.get(name)) {
            return Some(Arc::clone(s));
        }
        for d in &namespace.declarations {
            if d.kind == SyntaxKind::ModuleDeclaration
                && let Some(s) = self
                    .program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
            {
                return Some(Arc::clone(s));
            }
        }

        let export_equals = namespace.exports.get("export=")?;
        for d in &export_equals.declarations {
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
            {

                if let crate::ast::NodeData::ObjectLiteralExpression(ol) = &ea.expression.data {
                    for prop in ol.properties.iter() {
                        if prop.text() == name
                            && let Some(s) = self.program.symbol_map().symbol_of(prop)
                        {
                            return Some(Arc::clone(s));
                        }
                    }
                    continue;
                }
                if matches!(
                    ea.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
                {
                let scope_decl = namespace
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .cloned();
                let target = scope_decl.and_then(|scope_decl| {
                    self.push_scope(&scope_decl);
                    let t = self.resolve_qualified_symbol(&ea.expression);
                    self.pop_scope();
                    t
                });
                if let Some(mut target) = target {

                    for _ in 0..4 {
                        if target.flags.contains(SymbolFlags::ValueModule) {
                            break;
                        }
                        if target.flags != SymbolFlags::Alias {
                            break;
                        }
                        let next = target
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                            .and_then(|d| {
                                if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                                    && matches!(
                                        ied.module_reference.kind,
                                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                    )
                                {
                                    Some(self.resolve_qualified_symbol(&ied.module_reference))
                                } else {
                                    None
                                }
                            })
                            .flatten();
                        match next {
                            Some(n) => target = n,
                            None => break,
                        }
                    }
                    if target.flags.contains(SymbolFlags::ValueModule) {
                        return self.namespace_member_recursive(&target, name);
                    }
                    return Some(target);
                }
                }
            }
        }
        None
    }

    fn namespace_full_path(symbol: &Arc<Symbol>) -> String {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ModuleDeclaration);
        let Some(decl) = decl else {
            return symbol.name.clone();
        };
        let mut parts: Vec<String> = Vec::new();
        let mut current: Option<&Arc<Node>> = Some(decl);
        while let Some(n) = current {
            if let crate::ast::NodeData::ModuleDeclaration(md) = &n.data {
                parts.push(md.name.text().trim_matches(['"', '\'']).to_string());
            }
            current = n.parent.as_ref();
        }
        parts.reverse();
        parts.join(".")
    }

    fn check_assertion_overlap(&mut self, node: &Arc<Node>, expr: &Arc<Node>, type_node: &Arc<Node>) {

        if type_node.kind == SyntaxKind::TypeReference && type_node.text() == "const" {
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        let target_type = self.get_type_from_type_node(type_node);
        let error_type = self.error_type();
        let exempt = |t: &Arc<Type>| {
            Arc::ptr_eq(t, &error_type)
                || t.flags.contains(TypeFlags::Any)
                || t.flags.contains(TypeFlags::Unknown)
                || t.flags.contains(TypeFlags::Never)
        };
        if exempt(&expr_type) || exempt(&target_type) {
            return;
        }
        let expr_base = if crate::checker::is_literal_type(&expr_type) {
            self.get_base_type_of_literal_type(&expr_type)
        } else {
            expr_type
        };

        let comparable = self.is_type_comparable_to(&expr_base, &target_type)
            || self.is_type_comparable_to(&target_type, &expr_base);
        if !comparable {
            let source_str = self.type_to_string(&expr_base);
            let target_str = self.type_to_string(&target_type);
            let file = self.current_file.clone();
            let mut diag = crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST,
                vec![source_str, target_str],
            );

            if let Some((prop_loc, prop_name, elem_target_str)) =
                self.assertion_excess_detail(&expr, &expr_base, &target_type)
            {
                diag.loc = prop_loc;
                diag.message_chain.push(crate::ast::Diagnostic::new(
                    None,
                    prop_loc,
                    crate::diagnostics::messages_generated::
                        OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![prop_name, elem_target_str],
                ));
            }
            self.diagnostics.add(diag);
        }
    }

    fn assertion_excess_detail(
        &mut self,
        expr: &Arc<Node>,
        expr_type: &Arc<Type>,
        target_type: &Arc<Type>,
    ) -> Option<(TextRange, String, String)> {

        let (elem_source, elem_target, literal_node) = match &expr.data {
            NodeData::ObjectLiteralExpression(_) => {
                (Arc::clone(expr_type), Arc::clone(target_type), Arc::clone(expr))
            }
            NodeData::ArrayLiteralExpression(d) => {
                let first_obj = d.elements.iter().find(|e| {
                    matches!(&e.data, NodeData::ObjectLiteralExpression(_))
                })?;
                let st = self.element_type_of(expr_type)?;
                let tt = self.element_type_of(target_type)?;
                (st, tt, Arc::clone(first_obj))
            }
            _ => return None,
        };
        let prop_name = self.get_excess_property_name(&elem_source, &elem_target)?;
        let prop_loc =
            self.find_object_literal_property_name_node(&literal_node, &prop_name)?;
        let elem_target_str = self.type_to_string(&elem_target);
        Some((prop_loc, prop_name, elem_target_str))
    }

    fn element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Object) {
            if let TypeData::Object(obj) = &t.data
                && !obj.type_arguments.is_empty()
            {
                return Some(Arc::clone(&obj.type_arguments[0]));
            }
        }
        None
    }

    fn check_accessor_in_type_context(&mut self, member: &Arc<Node>) {
        let body = match &member.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => d.body.clone(),
            crate::ast::NodeData::SetAccessorDeclaration(d) => d.body.clone(),
            _ => return,
        };
        if let Some(body) = body {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                body.loc,
                crate::diagnostics::messages_generated::
                    AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                vec![],
            ));
        }
    }

    fn check_interface_members(&mut self, members: &NodeList) {

        {
            let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                std::collections::HashMap::new();
            for member in members.iter() {
                if let Some(name_node) = member.name() {
                    let name = match name_node.kind {
                        SyntaxKind::StringLiteral
                        | SyntaxKind::NumericLiteral
                        | SyntaxKind::Identifier
                        | SyntaxKind::PrivateIdentifier => name_node.text().to_string(),
                        _ => continue,
                    };
                    seen.entry(name).or_default().push(member);
                }
            }
            for (_, group) in seen.iter() {

                let all_methods = group
                    .iter()
                    .all(|m| m.kind == SyntaxKind::MethodSignature);
                let accessor_pair = group.iter().all(|m| {
                    matches!(
                        m.kind,
                        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                    )
                }) && group.iter().any(|m| m.kind == SyntaxKind::GetAccessor)
                    && group.iter().any(|m| m.kind == SyntaxKind::SetAccessor);
                if group.len() > 1 && !all_methods && !accessor_pair {
                    for m in group {
                        if let Some(name_node) = m.name() {
                            let name = name_node.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_node.loc,
                                crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                vec![name],
                            ));
                        }
                    }
                }
            }
        }
        for member in members.iter() {
            match member.kind {
                SyntaxKind::MethodSignature => {
                    let crate::ast::NodeData::MethodSignatureDeclaration(d) = &member.data
                    else {
                        continue;
                    };
                    self.check_parameter_property_modifiers(&d.parameters, false);
                    self.check_parameter_implicit_any(member, &d.parameters, 0);
                    for p in d.parameters.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = &d.type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any
                        && d.type_node.is_none()
                        && d.name.kind == SyntaxKind::Identifier
                    {
                        let file = self.current_file.clone();
                        let diagnostic = crate::ast::Diagnostic::new(
                            file,
                            d.name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![d.name.text().to_string(), "any".to_string()],
                        );
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::ConstructSignature | SyntaxKind::CallSignature => {
                    let (params, type_node) = match &member.data {
                        crate::ast::NodeData::ConstructSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        crate::ast::NodeData::CallSignatureDeclaration(d) => {
                            (&d.parameters, d.type_node.as_ref())
                        }
                        _ => continue,
                    };
                    self.check_parameter_property_modifiers(params, false);
                    self.check_parameter_implicit_any(member, params, 0);
                    for p in params.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                        }
                    }
                    if let Some(tn) = type_node {
                        self.check_type_annotation(tn);
                    }

                    if self.no_implicit_any && type_node.is_none() {
                        let message = if member.kind == SyntaxKind::ConstructSignature {
                            crate::diagnostics::messages_generated::
                                CONSTRUCT_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        } else {
                            crate::diagnostics::messages_generated::
                                CALL_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                        };
                        let file = self.current_file.clone();
                        let diagnostic =
                            crate::ast::Diagnostic::new(file, member.loc, message, vec![]);
                        self.diagnostics.add(diagnostic);
                    }
                }
                SyntaxKind::PropertySignature => {
                    if let crate::ast::NodeData::PropertySignatureDeclaration(d) = &member.data {
                        self.check_type_annotation(&d.type_node);
                    }
                }

                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                    self.check_accessor_in_type_context(member);
                }
                _ => {}
            }
        }
    }

    fn check_class_function_merge(&mut self, statements: &[Arc<Node>]) {
        let mut groups: std::collections::BTreeMap<String, Vec<Arc<Node>>> =
            std::collections::BTreeMap::new();
        for s in statements {
            match &s.data {
                crate::ast::NodeData::ClassDeclaration(d) => {
                    if let Some(n) = &d.name
                        && n.kind == SyntaxKind::Identifier
                    {
                        groups.entry(n.text().to_string()).or_default().push(Arc::clone(s));
                    }
                }
                crate::ast::NodeData::FunctionDeclaration(d) => {
                    if let Some(n) = &d.name
                        && n.kind == SyntaxKind::Identifier
                    {
                        groups.entry(n.text().to_string()).or_default().push(Arc::clone(s));
                    }
                }
                _ => {}
            }
        }
        for (name, decls) in groups {
            let has_non_ambient_class = decls.iter().any(|d| {
                d.kind == SyntaxKind::ClassDeclaration
                    && self.ambient_context_depth == 0
                    && !d.has_syntactic_modifier(ModifierFlags::Ambient)
            });
            let has_function = decls
                .iter()
                .any(|d| d.kind == SyntaxKind::FunctionDeclaration);
            if !(has_non_ambient_class && has_function) {
                continue;
            }
            for d in decls {
                let (name_node, message): (Option<&Arc<Node>>, _) = match &d.data {
                    crate::ast::NodeData::ClassDeclaration(cd) => (
                        cd.name.as_ref(),
                        crate::diagnostics::messages_generated::
                            CLASS_DECLARATION_CANNOT_IMPLEMENT_OVERLOAD_LIST_FOR_0,
                    ),
                    crate::ast::NodeData::FunctionDeclaration(fd) => (
                        fd.name.as_ref(),
                        crate::diagnostics::messages_generated::
                            FUNCTION_WITH_BODIES_CAN_ONLY_MERGE_WITH_CLASSES_THAT_ARE_AMBIENT,
                    ),
                    _ => continue,
                };
                let Some(name_node) = name_node else { continue };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_node.loc,
                    message,
                    vec![name.clone()],
                ));
            }
        }
    }

    fn check_function_overloads_recursive(&mut self, statements: &[Arc<Node>]) {
        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        self.check_statement_function_overloads(statements);
        self.check_class_function_merge(statements);
        for s in statements {
            match &s.data {
                crate::ast::NodeData::Block(d) => {
                    self.check_function_overloads_recursive(&d.statements.nodes);
                }
                crate::ast::NodeData::ModuleDeclaration(d) => {

                    if d
                        .modifiers
                        .as_ref()
                        .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
                    {
                        continue;
                    }
                    if let Some(body) = &d.body
                        && matches!(body.kind, SyntaxKind::Block | SyntaxKind::ModuleBlock)
                        && let crate::ast::NodeData::Block(bd) = &body.data
                    {
                        self.check_function_overloads_recursive(&bd.statements.nodes);
                    }
                    if let Some(body) = &d.body
                        && body.kind == SyntaxKind::ModuleBlock
                        && let crate::ast::NodeData::ModuleBlock(bd) = &body.data
                    {
                        self.check_function_overloads_recursive(&bd.statements.nodes);
                    }
                }
                _ => {}
            }
        }
    }

    fn check_statement_function_overloads(&mut self, statements: &[Arc<Node>]) {

        let ambient_context = self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        let statements: Vec<Arc<Node>> = statements
            .iter()
            .filter(|s| {
                !matches!(s.kind, SyntaxKind::FunctionDeclaration)
                    || !(ambient_context || s.has_syntactic_modifier(ModifierFlags::Ambient))
            })
            .cloned()
            .collect();
        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, s) in statements.iter().enumerate() {
            if s.kind != SyntaxKind::FunctionDeclaration {
                continue;
            }
            if let crate::ast::NodeData::FunctionDeclaration(d) = &s.data
                && let Some(n) = &d.name
                && n.kind == SyntaxKind::Identifier
            {
                groups.entry(n.text().to_string()).or_default().push(idx);
            }
        }
        for (_, idxs) in groups {
            let mut prev: Option<usize> = None;
            let mut has_body = false;
            for &idx in &idxs {
                let body = matches!(
                    &statements[idx].data,
                    crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some()
                );
                if !body {
                    if let Some(p) = prev {
                        if p + 1 != idx {
                            self.report_function_impl_expected(&statements, p);
                        }
                    }
                } else {
                    has_body = true;
                }
                prev = Some(idx);
            }
            if !has_body {
                let last = idxs[idxs.len() - 1];
                if !statements[last].has_syntactic_modifier(ModifierFlags::Ambient) {
                    self.report_function_impl_expected(&statements, last);
                }
            } else {

                let fn_params = |f: &Arc<Node>| -> (usize, bool) {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data {
                        let mut rest = false;
                        for p in d.parameters.iter() {
                            if p.kind == SyntaxKind::Parameter {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    if pd.dot_dot_dot_token.is_some() {
                                        rest = true;
                                        break;
                                    }

                                    let _ = pd.question_token.is_none();
                                }
                            }
                        }
                        (d.parameters.nodes.len(), rest)
                    } else {
                        (0, false)
                    }
                };
                let impl_idx = idxs
                    .iter()
                    .copied()
                    .find(|&i| {
                        matches!(
                            &statements[i].data,
                            crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some()
                        )
                    })
                    .unwrap_or_else(|| idxs[idxs.len() - 1]);
                let (_impl_total, impl_rest) = fn_params(&statements[impl_idx]);
                let impl_required = {

                    let mut n = 0;
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &statements[impl_idx].data
                    {
                        for p in d.parameters.iter() {
                            if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                                && pd.dot_dot_dot_token.is_none()
                                && pd.question_token.is_none()
                            {
                                n += 1;
                            }
                        }
                    }
                    n
                };
                if !impl_rest {

                    let mut seen_shapes: Vec<String> = Vec::new();
                    for &i in &idxs {
                        if i == impl_idx {
                            continue;
                        }
                        let (overload_count, _) = fn_params(&statements[i]);
                        let shape = if let crate::ast::NodeData::FunctionDeclaration(d) =
                            &statements[i].data
                        {
                            let mut parts = Vec::new();
                            for p in d.parameters.iter() {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    let t = pd
                                        .type_node
                                        .as_ref()
                                        .map(|tn| tn.text())
                                        .unwrap_or_default();
                                    parts.push(format!(
                                        "{t}{}",
                                        if pd.question_token.is_some() { "?" } else { "" }
                                    ));
                                }
                            }
                            let ret = d
                                .type_node
                                .as_ref()
                                .map(|tn| tn.text())
                                .unwrap_or_default();
                            format!("({})=>{}", parts.join(","), ret)
                        } else {
                            String::new()
                        };
                        if seen_shapes.contains(&shape) {
                            continue;
                        }
                        seen_shapes.push(shape);
                        let arity_bad = !impl_rest && overload_count < impl_required;
                        let overload_node = Arc::clone(&statements[i]);
                        let impl_node = Arc::clone(&statements[impl_idx]);
                        let compat = self
                            .overload_signature_compatible_with_implementation(
                                &overload_node, &impl_node,
                            );
                        if (arity_bad || !compat)
                            && let crate::ast::NodeData::FunctionDeclaration(d) = &statements[i].data
                            && let Some(n) = &d.name
                        {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                n.loc,
                                crate::diagnostics::messages_generated::
                                    THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                                Vec::new(),
                            ));
                        }
                    }
                }
            }
        }
    }

    fn report_function_impl_expected(&mut self, statements: &[Arc<Node>], idx: usize) {
        let node = Arc::clone(&statements[idx]);
        let (name_text, name_loc) = match &node.data {
            crate::ast::NodeData::FunctionDeclaration(d) => match &d.name {
                Some(n) => (n.text().to_string(), n.loc),
                None => return,
            },
            _ => return,
        };
        if let Some(sib) = statements.get(idx + 1) {
            if sib.kind == SyntaxKind::FunctionDeclaration {
                let sib_name = match &sib.data {
                    crate::ast::NodeData::FunctionDeclaration(d) => match &d.name {
                        Some(n) => (n.text().to_string(), n.loc, d.body.is_some()),
                        None => (String::new(), sib.loc, false),
                    },
                    _ => (String::new(), sib.loc, false),
                };
                if sib_name.0 == name_text {
                    return;
                }
                if sib_name.2 {
                    let file = self.current_file.clone();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        sib_name.1,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![name_text],
                    );
                    self.diagnostics.add(diagnostic);
                    return;
                }
            }
        }
        let file = self.current_file.clone();
        let diagnostic = crate::ast::Diagnostic::new(
            file,
            name_loc,
            crate::diagnostics::messages_generated::
                FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
            Vec::new(),
        );
        self.diagnostics.add(diagnostic);
    }

    fn check_export_assignment_conflicts(&mut self, statements: &[Arc<Node>]) {
        let export_equals = statements.iter().find(|s| {
            matches!(
                &s.data,
                crate::ast::NodeData::ExportAssignment(d) if d.is_export_equals
            )
        });
        let Some(eq_decl) = export_equals else { return };
        let has_other_value_export = statements.iter().any(|s| {
            if Arc::ptr_eq(s, eq_decl) {
                return false;
            }
            let value_declaring = matches!(
                s.kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::VariableStatement
                    | SyntaxKind::ModuleDeclaration
            );
            value_declaring && s.has_syntactic_modifier(ModifierFlags::Export)
        });
        if has_other_value_export {
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                eq_decl.loc,
                crate::diagnostics::messages_generated::
                    AN_EXPORT_ASSIGNMENT_CANNOT_BE_USED_IN_A_MODULE_WITH_OTHER_EXPORTED_ELEMENTS,
                Vec::new(),
            );
            self.diagnostics.add(diagnostic);
        }
    }

    fn check_reserved_type_name(&mut self, name: &Arc<Node>, message: &'static crate::diagnostics::Message) {
        const RESERVED: &[&str] = &[
            "any", "unknown", "never", "number", "bigint", "boolean", "string", "symbol",
            "void", "object", "undefined",
        ];
        let text = name.text();
        if RESERVED.contains(&text) {
            let file = self.current_file.clone();
            let diagnostic = crate::ast::Diagnostic::new(
                file,
                name.loc,
                *message,
                vec![text.to_string()],
            );
            self.diagnostics.add(diagnostic);
        }
    }

    fn is_type_assignable_to_kind_snf(&mut self, source: &Arc<Type>, kind: TypeFlags) -> bool {
        if source.flags.intersects(kind) {
            return true;
        }
        let number = self.number_type();
        if kind.intersects(crate::checker::types::TYPE_FLAGS_NUMBER_LIKE)
            && self.is_type_assignable_to(source, &number)
        {
            return true;
        }
        let string = self.string_type();
        if kind.intersects(crate::checker::types::TYPE_FLAGS_STRING_LIKE)
            && self.is_type_assignable_to(source, &string)
        {
            return true;
        }
        let symbol = self.es_symbol_type();
        if kind.intersects(TypeFlags::ESSymbol) && self.is_type_assignable_to(source, &symbol) {
            return true;
        }
        false
    }

    fn check_computed_property_name(&mut self, name: &Arc<Node>) {
        if name.kind != SyntaxKind::ComputedPropertyName {
            return;
        }
        if !self.computed_property_name_checked.insert(Arc::as_ptr(name)) {
            return;
        }
        let expr = match &name.data {
            crate::ast::NodeData::ComputedPropertyName(data) => Arc::clone(&data.expression),
            _ => return,
        };

        let invalid_in_form = matches!(&expr.data, crate::ast::NodeData::BinaryExpression(b)
            if b.operator_token.kind == SyntaxKind::InKeyword)
            && name.parent.as_ref().is_some_and(|member| {
                !matches!(
                    member.kind,
                    SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                ) && member
                    .parent
                    .as_ref()
                    .is_some_and(|container| {
                        matches!(
                            container.kind,
                            SyntaxKind::TypeLiteral
                                | SyntaxKind::ClassDeclaration
                                | SyntaxKind::ClassExpression
                                | SyntaxKind::InterfaceDeclaration
                        )
                    })
            });
        if invalid_in_form {
            return;
        }

        self.check_expression(&expr);
        let t = self.get_type_of_node(&expr);

        let kind = crate::checker::types::TYPE_FLAGS_STRING_LIKE
            | crate::checker::types::TYPE_FLAGS_NUMBER_LIKE
            | crate::checker::types::TYPE_FLAGS_ES_SYMBOL_LIKE;
        let bad = t.flags.intersects(crate::checker::types::TYPE_FLAGS_NULLABLE)
            || (!self.is_type_assignable_to_kind_snf(&t, kind) && {
                let target = self.get_union_type(vec![
                    self.string_type(),
                    self.number_type(),
                    self.es_symbol_type(),
                ]);
                !self.is_type_assignable_to(&t, &target)
            });
        if bad {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    A_COMPUTED_PROPERTY_NAME_MUST_BE_OF_TYPE_STRING_NUMBER_SYMBOL_OR_ANY,
                vec![],
            ));
        }
    }

    fn member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::MethodSignatureDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::PropertyDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::PropertySignatureDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::PropertyAssignment(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::ShorthandPropertyAssignment(d) => Some(Arc::clone(&d.name)),
            _ => None,
        }
    }

    fn property_name_key_type(&mut self, name: &Arc<Node>) -> Option<Arc<Type>> {
        match &name.data {
            crate::ast::NodeData::ComputedPropertyName(data) => {
                let expr = &data.expression;
                match &expr.data {
                    crate::ast::NodeData::StringLiteral(s) => {
                        Some(self.get_string_literal_type(&s.text))
                    }
                    crate::ast::NodeData::NumericLiteral(n) => {
                        Some(self.get_number_literal_type(jsnum::Number::from_string(&n.text)))
                    }
                    _ => {

                        Some(self.get_type_of_node(expr))
                    }
                }
            }
            crate::ast::NodeData::Identifier(data) => {
                if let Ok(_) = data.text.parse::<f64>() {
                    Some(self.get_number_literal_type(jsnum::Number::from_string(&data.text)))
                } else {
                    Some(self.get_string_literal_type(&data.text))
                }
            }
            crate::ast::NodeData::StringLiteral(data) => {
                Some(self.get_string_literal_type(&data.text))
            }
            crate::ast::NodeData::NumericLiteral(data) => Some(
                self.get_number_literal_type(jsnum::Number::from_string(&data.text)),
            ),
            _ => None,
        }
    }

    fn property_name_display(&self, name: &Arc<Node>) -> String {
        if name.kind == SyntaxKind::ComputedPropertyName {
            if let Some(text) = self.node_source_text(name) {
                let inner = text
                    .strip_prefix('[')
                    .and_then(|t| t.strip_suffix(']'))
                    .unwrap_or(&text);
                return format!("[{inner}]");
            }
        }
        name.text().to_string()
    }

    pub(crate) fn node_source_text(&self, node: &Arc<Node>) -> Option<String> {
        let mut root: &Arc<Node> = node;
        while let Some(p) = root.parent.as_ref() {
            root = p;
        }
        for f in &self.files {
            if Arc::ptr_eq(&f.node, root) {
                return f
                    .text
                    .get(node.loc.pos()..node.loc.end())
                    .map(|s| s.to_string());
            }
        }
        None
    }

    fn member_declared_type_for_index_check(&mut self, member: &Arc<Node>) -> Option<Arc<Type>> {
        match &member.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(
                self.infer_function_return_type(d.body.as_ref(), d.type_node.as_ref()),
            ),
            crate::ast::NodeData::SetAccessorDeclaration(d) => {
                let tn = d.parameters.iter().next().and_then(|p| {
                    match &p.data {
                        crate::ast::NodeData::ParameterDeclaration(pd) => pd.type_node.clone(),
                        _ => None,
                    }
                });
                match tn {
                    Some(t) => Some(self.get_type_from_type_node(&t)),
                    None => Some(self.any_type()),
                }
            }
            crate::ast::NodeData::PropertyDeclaration(d) => {
                if let Some(t) = &d.type_node {
                    Some(self.get_type_from_type_node(t))
                } else if let Some(init) = &d.initializer {
                    let init_t = self.get_type_of_node(init);
                    Some(self.widen_initializer_type(&init_t))
                } else {
                    None
                }
            }
            crate::ast::NodeData::PropertySignatureDeclaration(d) => {
                Some(self.get_type_from_type_node(&d.type_node))
            }
            _ => None,
        }
    }

    fn check_index_constraints(&mut self, t: &Arc<Type>, declaration: &Arc<Node>) {
        let index_infos = self.get_index_infos_of_type(t);
        if index_infos.is_empty() {
            return;
        }

        let local_index: Option<Arc<crate::checker::IndexInfo>> = index_infos
            .iter()
            .find(|info| {
                info.declaration
                    .as_ref()
                    .and_then(|d| d.parent.as_ref())
                    .is_some_and(|p| Arc::ptr_eq(p, declaration))
            })
            .cloned();
        let is_interface = declaration.kind == SyntaxKind::InterfaceDeclaration;

        for prop in self.get_properties_of_type(t) {
            let Some(first_decl) = prop.declarations.first().cloned() else {
                continue;
            };
            if first_decl
                .parent
                .as_ref()
                .is_some_and(|p| Arc::ptr_eq(p, declaration))
            {
                continue;
            }
            let Some(name) = Self::member_name_node(&first_decl) else {
                continue;
            };
            if name.kind == SyntaxKind::ComputedPropertyName {
                continue;
            }
            let Some(key_type) = self.property_name_key_type(&name) else {
                continue;
            };
            let prop_type = self.get_type_of_symbol(&prop);
            let display = self.property_name_display(&name);
            self.check_index_constraint_for_property(
                t,
                &key_type,
                &prop_type,
                &name,
                &display,
                None,
                local_index.clone(),
                is_interface.then(|| Arc::clone(declaration)),
                &index_infos,
            );
        }

        let props_by_name: std::collections::HashMap<String, Arc<Symbol>> = self
            .get_properties_of_type(t)
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        let members: Vec<Arc<Node>> = match &declaration.data {
            crate::ast::NodeData::ClassDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
            crate::ast::NodeData::InterfaceDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
            _ => Vec::new(),
        };
        for member in &members {
            if member.kind == SyntaxKind::IndexSignature {
                continue;
            }
            let Some(name) = Self::member_name_node(member) else {
                continue;
            };
            let member_symbol = self.program.symbol_map().symbol_of(member).cloned();
            let Some(key_type) = self.property_name_key_type(&name) else {
                continue;
            };
            let prop_type = if name.kind != SyntaxKind::ComputedPropertyName {

                match props_by_name.get(name.text()) {
                    Some(sym) => self.get_type_of_symbol(sym),
                    None => match self.member_declared_type_for_index_check(member) {
                        Some(t) => t,
                        None => continue,
                    },
                }
            } else {

                match self.member_declared_type_for_index_check(member) {
                    Some(t) => t,
                    None => match &member_symbol {
                        Some(sym) => self.get_type_of_symbol(sym),
                        None => continue,
                    },
                }
            };
            let display = self.property_name_display(&name);
            let local_name_node = Some(Arc::clone(&name));
            self.check_index_constraint_for_property(
                t,
                &key_type,
                &prop_type,
                &name,
                &display,
                local_name_node,
                local_index.clone(),
                is_interface.then(|| Arc::clone(declaration)),
                &index_infos,
            );
        }

        let mut bases: Vec<Arc<Node>> = Vec::new();
        let mut worklist: Vec<Arc<Node>> = vec![Arc::clone(declaration)];
        let mut guard = 0;
        while let Some(d) = worklist.pop() {
            guard += 1;
            if guard > 32 {
                break;
            }
            let heritage = match &d.data {
                crate::ast::NodeData::ClassDeclaration(cd) => {
                    cd.heritage_clauses.clone()
                }
                crate::ast::NodeData::InterfaceDeclaration(id) => id.heritage_clauses.clone(),
                _ => continue,
            };
            let Some(clauses) = heritage else { continue };
            for clause in clauses.iter() {
                let crate::ast::NodeData::HeritageClause(hc) = &clause.data else {
                    continue;
                };
                for type_ref in hc.types.iter() {
                    let base_expr = match &type_ref.data {
                        crate::ast::NodeData::ExpressionWithTypeArguments(e) => {
                            Arc::clone(&e.expression)
                        }
                        _ => continue,
                    };
                    let base_symbol = if base_expr.kind == SyntaxKind::Identifier {
                        self.resolve_identifier(&base_expr)
                    } else {
                        None
                    };
                    let Some(base_symbol) = base_symbol else {
                        continue;
                    };
                    for bd in &base_symbol.declarations {
                        if matches!(
                            bd.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::InterfaceDeclaration
                        ) && !bases.iter().any(|b| Arc::ptr_eq(b, bd))
                            && !Arc::ptr_eq(bd, &d)
                        {
                            bases.push(Arc::clone(bd));
                            worklist.push(Arc::clone(bd));
                        }
                    }
                }
            }
        }
        for base in &bases {
            let base_members: Vec<Arc<Node>> = match &base.data {
                crate::ast::NodeData::ClassDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                crate::ast::NodeData::InterfaceDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                _ => continue,
            };
            for member in base_members {
                let Some(name) = Self::member_name_node(&member) else {
                    continue;
                };
                if name.kind != SyntaxKind::ComputedPropertyName {
                    continue;
                }
                let Some(key_type) = self.property_name_key_type(&name) else {
                    continue;
                };
                let Some(symbol) = self.program.symbol_map().symbol_of(&member).cloned()
                else {
                    continue;
                };
                let prop_type = self
                    .member_declared_type_for_index_check(&member)
                    .unwrap_or_else(|| self.get_type_of_symbol(&symbol));
                let display = self.property_name_display(&name);
                let index_for_error = local_index.clone();
                let iface_decl = is_interface.then(|| Arc::clone(declaration));
                self.check_index_constraint_for_property(
                    t,
                    &key_type,
                    &prop_type,
                    &name,
                    &display,
                    None,
                    index_for_error,
                    iface_decl,
                    &index_infos,
                );
            }
        }
    }

    fn check_index_constraint_for_property(
        &mut self,
        _t: &Arc<Type>,
        key_type: &Arc<Type>,
        prop_type: &Arc<Type>,
        name: &Arc<Node>,
        display: &str,
        local_name: Option<Arc<Node>>,
        local_index: Option<Arc<crate::checker::IndexInfo>>,
        interface_decl: Option<Arc<Node>>,
        index_infos: &[Arc<crate::checker::IndexInfo>],
    ) {
        for info in index_infos {
            let Some(info_key) = info.key_type.clone() else {
                continue;
            };
            if !self.is_applicable_index_type(key_type, &info_key) {
                continue;
            }
            let info_value = match info.value_type.clone() {
                Some(v) => v,
                None => continue,
            };
            if self.is_type_assignable_to(prop_type, &info_value) {
                continue;
            }

            let (error_loc, related_index_decl) = if let Some(n) = &local_name {
                (n.loc, None)
            } else if let Some(idx) = &local_index {
                (
                    idx.declaration
                        .as_ref()
                        .map(|d| d.loc)
                        .unwrap_or(name.loc),
                    idx.declaration.clone(),
                )
            } else if let Some(idecl) = &interface_decl {
                (idecl.loc, None)
            } else {
                continue;
            };
            let file = self.current_file.clone();
            let mut diagnostic = crate::ast::Diagnostic::new(
                file,
                error_loc,
                crate::diagnostics::messages_generated::
                    PROPERTY_0_OF_TYPE_1_IS_NOT_ASSIGNABLE_TO_2_INDEX_TYPE_3,
                vec![
                    display.to_string(),
                    self.type_to_string(prop_type),
                    self.type_to_string(&info_key),
                    self.type_to_string(&info_value),
                ],
            );
            if let Some(idx_decl) = related_index_decl {
                diagnostic.related_information = vec![crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    idx_decl.loc,
                    crate::diagnostics::messages_generated::X_0_IS_DECLARED_HERE,
                    vec![display.to_string()],
                )];
            }
            self.diagnostics.add(diagnostic);
        }
    }

    fn check_class_member(&mut self, node: &Arc<Node>) {

        self.check_grammar_modifiers(node);

        if node.kind == SyntaxKind::Constructor {
            self.check_multiple_constructor_implementations(node);
        }

        {
            let class_node = node.parent.clone();
            if let Some(cls) = &class_node
                && matches!(cls.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression)
                && let crate::ast::NodeData::ClassDeclaration(cd) = &cls.data
            {
                let (my_name, my_loc) = match &node.data {
                    crate::ast::NodeData::PropertyDeclaration(d) => {
                        (d.name.text().to_string(), d.name.loc)
                    }
                    crate::ast::NodeData::MethodDeclaration(d) => {
                        (d.name.text().to_string(), d.name.loc)
                    }
                    _ => (String::new(), node.loc),
                };
                if !my_name.is_empty() && my_name.starts_with('#') {

                    let i_am_static = node.has_syntactic_modifier(ModifierFlags::Static);
                    let conflict = cd.members.iter().any(|m| {
                        if m.loc.pos() >= node.loc.pos() {
                            return false;
                        }
                        let Some(mn) = m.name() else { return false };
                        mn.kind == SyntaxKind::PrivateIdentifier
                            && mn.text() == my_name
                            && m.has_syntactic_modifier(ModifierFlags::Static) != i_am_static
                    });
                    if conflict {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            my_loc,
                            crate::diagnostics::messages_generated::
                                DUPLICATE_IDENTIFIER_0_STATIC_AND_INSTANCE_ELEMENTS_CANNOT_SHARE_THE_SAME_PRIVATE_NAME,
                            vec![my_name.clone()],
                        ));
                    }
                }
                if !my_name.is_empty() && !my_name.starts_with('#') {

                    let kind_of = |m: &Arc<Node>| match &m.data {
                        crate::ast::NodeData::PropertyDeclaration(_) => "prop",
                        crate::ast::NodeData::MethodDeclaration(d) => {
                            if d.body.is_some() { "method-body" } else { "method-sig" }
                        }
                        crate::ast::NodeData::GetAccessorDeclaration(_) => "get",
                        crate::ast::NodeData::SetAccessorDeclaration(_) => "set",
                        _ => "",
                    };
                    let mine = kind_of(node);
                    let mut theirs_all_prop = true;
                    let dup = cd.members.iter().any(|m| {
                        if Arc::ptr_eq(m, node) || m.loc.pos() >= node.loc.pos() {
                            return false;
                        }
                        let name_match = match &m.data {
                            crate::ast::NodeData::PropertyDeclaration(d) => {
                                d.name.text() == my_name
                            }
                            crate::ast::NodeData::MethodDeclaration(d) => {
                                d.name.text() == my_name
                            }
                            _ => false,
                        };
                        if !name_match {
                            return false;
                        }
                        let theirs = kind_of(m);
                        if theirs != "prop" {
                            theirs_all_prop = false;
                        }
                        match (mine, theirs) {
                            ("prop", "prop") => true,
                            ("prop", "method-body") | ("prop", "method-sig") => true,
                            ("method-body", "prop") | ("method-sig", "prop") => true,

                            ("method-body", "method-body") => true,
                            _ => false,
                        }
                    });

                    let earlier_has_prop = cd.members.iter().any(|m| {
                        m.loc.pos() < node.loc.pos()
                            && matches!(&m.data, crate::ast::NodeData::PropertyDeclaration(d) if d.name.text() == my_name)
                    });

                    let earlier_has_method = cd.members.iter().any(|m| {
                        m.loc.pos() < node.loc.pos()
                            && matches!(&m.data, crate::ast::NodeData::MethodDeclaration(d) if d.name.text() == my_name)
                    });
                    let report_here = match mine {
                        "prop" => true,
                        "method-body" | "method-sig" => earlier_has_prop,
                        _ => false,
                    };
                    let _ = earlier_has_method;
                    if dup && report_here {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            my_loc,
                            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![my_name.clone()],
                        ));
                    }

                    if dup && mine == "prop" && !report_here {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            my_loc,
                            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![my_name.clone()],
                        ));
                    }

                    if dup && matches!(mine, "method-body" | "method-sig") && theirs_all_prop {
                        if let Some(earlier) = cd.members.iter().find(|m| {
                            m.loc.pos() < node.loc.pos()
                                && matches!(&m.data, crate::ast::NodeData::PropertyDeclaration(d) if d.name.text() == my_name)
                        }) {
                            let earlier_loc = earlier
                                .name()
                                .map(|n| n.loc)
                                .unwrap_or(earlier.loc);
                            let already = self
                                .diagnostics
                                .get_all()
                                .iter()
                                .any(|d| d.code == 2300 && d.loc == earlier_loc);
                            if !already {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    earlier_loc,
                                    crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                    vec![my_name.clone()],
                                ));
                            }
                        }
                    }

                    if dup && mine == "prop" {
                        let first = cd.members.iter().find(|m| {
                            m.loc.pos() < node.loc.pos()
                                && match &m.data {
                                    crate::ast::NodeData::PropertyDeclaration(d) => {
                                        d.name.text() == my_name
                                    }
                                    crate::ast::NodeData::MethodDeclaration(d) => {
                                        d.name.text() == my_name
                                    }
                                    _ => false,
                                }
                        });
                        if let Some(first) = first {
                            let first_type = match &first.data {
                                crate::ast::NodeData::PropertyDeclaration(d) => {
                                    let tn = d.type_node.clone();
                                    tn.map(|tn| {
                                        let t = self.get_type_from_type_node(&tn);
                                        self.type_to_string(&t)
                                    })
                                }
                                _ => None,
                            };
                            let first_sig = match &first.data {
                                crate::ast::NodeData::MethodDeclaration(d) => {
                                    let ret = d
                                        .type_node
                                        .as_ref()
                                        .map(|tn| self.get_type_from_type_node(tn))
                                        .unwrap_or_else(|| self.any_type());
                                    let sig = self
                                        .build_signature_from_function_like_type_node(
                                            &d.parameters,
                                            ret,
                                            false,
                                            None,
                                            Some(Arc::clone(first)),
                                        );
                                    Some(self.type_to_string(
                                        &self.create_function_or_constructor_type(
                                            vec![sig],
                                            false,
                                        ),
                                    ))
                                }
                                _ => None,
                            };
                            let later_type = match &node.data {
                                crate::ast::NodeData::PropertyDeclaration(d) => {
                                    let tn = d.type_node.clone();
                                    tn.map(|tn| {
                                        let t = self.get_type_from_type_node(&tn);
                                        self.type_to_string(&t)
                                    })
                                }
                                _ => None,
                            };
                            if let (Some(f), Some(l)) = (first_type.or(first_sig), later_type) {
                                if f != l {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        my_loc,
                                        crate::diagnostics::messages_generated::
                                            SUBSEQUENT_PROPERTY_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_PROPERTY_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                                        vec![my_name.clone(), f, l],
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        match node.kind {
            SyntaxKind::PropertyDeclaration => {

                if let crate::ast::NodeData::PropertyDeclaration(data) = &node.data {

                    self.check_computed_property_name(&data.name);

                    if node.has_syntactic_modifier(ModifierFlags::Abstract)
                        && data.initializer.is_some()
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_CANNOT_HAVE_AN_INITIALIZER_BECAUSE_IT_IS_MARKED_ABSTRACT,
                            vec![data.name.text().to_string()],
                        ));
                    }

                    if node.has_syntactic_modifier(ModifierFlags::Static) {
                        if let Some(type_node) = &data.type_node {
                            let prev = self.in_static_member_type;
                            self.in_static_member_type = true;
                            let _ = self.get_type_from_type_node(type_node);
                            self.in_static_member_type = prev;
                        }
                    }
                    if let Some(init) = &data.initializer {

                        let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                        self.this_container_stack.push(if is_static {
                            ThisContainerKind::StaticMember
                        } else {
                            ThisContainerKind::InstanceMember
                        });
                        self.check_expression(init);
                        self.this_container_stack.pop();

                        if let Some(tn) = &data.type_node {
                            let target = self.get_type_from_type_node(tn);
                            let anchor = data.name.loc;
                            self.check_contextual_elements(init, &target, anchor);
                        }
                    }
                }
            }
            SyntaxKind::PropertySignature => {

                if let crate::ast::NodeData::PropertySignatureDeclaration(data) = &node.data {
                    self.check_computed_property_name(&data.name);
                }
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                if let crate::ast::NodeData::ClassStaticBlockDeclaration(data) = &node.data {

                    self.this_container_stack
                        .push(ThisContainerKind::StaticMember);
                    self.check_statement(&data.body);
                    self.this_container_stack.pop();
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {

                if node.kind != SyntaxKind::Constructor
                    && let Some(name) = Self::member_name_node(node)
                {
                    self.check_computed_property_name(&name);
                }

                let (body, type_node, parameters): (
                    Option<Arc<Node>>,
                    Option<Arc<Node>>,
                    Option<Arc<NodeList>>,
                ) = match &node.data {
                    crate::ast::NodeData::MethodDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    crate::ast::NodeData::ConstructorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    crate::ast::NodeData::GetAccessorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    crate::ast::NodeData::SetAccessorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone(), Some(Arc::clone(&d.parameters)))
                    }
                    _ => (None, None, None),
                };

                if body.is_some()
                    && (self
                        .enclosing_class_stack
                        .last()
                        .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Ambient))
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file))
                    && let Some(body) = &body
                {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        crate::core::text::TextRange::new(body.loc.pos(), body.loc.pos() + 1),
                        crate::diagnostics::messages_generated::
                            AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                        vec![],
                    ));
                }

                if matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor) {
                    let ambient = self
                        .enclosing_class_stack
                        .last()
                        .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Ambient))
                        || self.ambient_context_depth > 0
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file);
                    let is_abstract = node.has_syntactic_modifier(ModifierFlags::Abstract);
                    if node.kind == SyntaxKind::SetAccessor
                        && let Some(params) = &parameters
                        && let Some(first) = params.iter().next()
                        && let crate::ast::NodeData::ParameterDeclaration(pd) = &first.data
                    {
                        if let Some(rest) = &pd.dot_dot_dot_token {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                rest.loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_CANNOT_HAVE_REST_PARAMETER,
                                vec![],
                            ));
                        }
                        if let Some(question) = &pd.question_token {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                question.loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_CANNOT_HAVE_AN_OPTIONAL_PARAMETER,
                                vec![],
                            ));
                        }
                        if pd.initializer.is_some() {
                            let name_loc = Self::class_member_name_node(node)
                                .map(|n| n.loc)
                                .unwrap_or(node.loc);
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
                                vec![],
                            ));
                        }
                    }
                    if body.is_none() && !ambient && !is_abstract && node.loc.end() > 0 {

                        let file = self.current_file.clone();
                        let mut p = node.loc.end();
                        if let Some(f) = file.as_ref() {
                            while p > node.loc.pos()
                                && matches!(
                                    f.text.as_bytes()[p - 1],
                                    b'\r' | b'\n' | b' ' | b'\t'
                                )
                            {
                                p -= 1;
                            }
                        }
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            crate::core::text::TextRange::new(p - 1, p),
                            crate::diagnostics::messages_generated::X_0_EXPECTED,
                            vec!["{".to_string()],
                        ));
                    }

                    if body.is_some() && is_abstract {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            node.loc,
                            crate::diagnostics::messages_generated::
                                AN_ABSTRACT_ACCESSOR_CANNOT_HAVE_AN_IMPLEMENTATION,
                            vec![],
                        ));
                    }

                    if node.kind == SyntaxKind::GetAccessor
                        && let Some(class) = self.enclosing_class_stack.last().cloned()
                        && let crate::ast::NodeData::GetAccessorDeclaration(gd) = &node.data
                        && gd.name.kind == SyntaxKind::Identifier
                    {
                        let setter = Self::class_members_of(&class).iter().find_map(|m| {
                            if let crate::ast::NodeData::SetAccessorDeclaration(sd) = &m.data
                                && sd.name.kind == SyntaxKind::Identifier
                                && sd.name.text() == gd.name.text()
                            {
                                Some((Arc::clone(m), sd.name.loc))
                            } else {
                                None
                            }
                        });
                        if let Some((setter_node, setter_name_loc)) = setter {
                            let getter_abstract = is_abstract;
                            let setter_abstract =
                                setter_node.has_syntactic_modifier(ModifierFlags::Abstract);
                            if getter_abstract != setter_abstract {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file.clone(),
                                    gd.name.loc,
                                    crate::diagnostics::messages_generated::
                                        ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                    vec![],
                                ));
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    setter_name_loc,
                                    crate::diagnostics::messages_generated::
                                        ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                    vec![],
                                ));
                            }

                            let setter_param_type_node =
                                if let crate::ast::NodeData::SetAccessorDeclaration(sd) =
                                    &setter_node.data
                                {
                                    sd.parameters.iter().next().and_then(|p| {
                                        if let crate::ast::NodeData::ParameterDeclaration(pd) =
                                            &p.data
                                        {
                                            pd.type_node.clone()
                                        } else {
                                            None
                                        }
                                    })
                                } else {
                                    None
                                };
                            if gd.type_node.is_none() && let Some(setter_tn) = setter_param_type_node
                            {
                                self.accessor_pair_return_hint =
                                    Some(self.get_type_from_type_node(&setter_tn));
                            }
                        }
                    }

                        if node.kind == SyntaxKind::SetAccessor
                            && let Some(class) = self.enclosing_class_stack.last().cloned()
                            && let crate::ast::NodeData::SetAccessorDeclaration(sd) = &node.data
                            && sd.name.kind == SyntaxKind::Identifier
                            && let Some(param) = sd.parameters.iter().next()
                            && let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
                            && pd.type_node.is_none()
                            && let Some(param_name) = (if pd.name.kind == SyntaxKind::Identifier {
                                Some(pd.name.text().to_string())
                            } else {
                                None
                            })
                        {
                            let getter_type = Self::class_members_of(&class)
                                .iter()
                                .find_map(|m| {
                                    if let crate::ast::NodeData::GetAccessorDeclaration(gd) =
                                        &m.data
                                        && gd.name.kind == SyntaxKind::Identifier
                                        && gd.name.text() == sd.name.text()
                                        && let Some(tn) = &gd.type_node
                                    {
                                        Some(self.get_type_from_type_node(tn))
                                    } else {
                                        None
                                    }
                                });
                            if let (Some(expected), Some(body)) = (getter_type, &sd.body) {
                                for (lhs_loc, rhs) in
                                    Self::assignments_to_name(body, &param_name)
                                {
                                    let actual = self.get_type_of_node(&rhs);
                                    if !actual.flags.contains(TypeFlags::Any)
                                        && !self.is_type_assignable_to(&actual, &expected)
                                    {
                                        let display_type =
                                            if crate::checker::is_literal_type(&actual) {
                                                self.get_base_type_of_literal_type(&actual)
                                            } else {
                                                actual.clone()
                                            };
                                        let actual_str = self.type_to_string(&display_type);
                                        let expected_str = self.type_to_string(&expected);
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            lhs_loc,
                                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                            vec![actual_str, expected_str],
                                        ));
                                    }
                                }
                            }
                        }

                    if !(body.is_none() && !is_abstract && !ambient) {
                        let name_loc = Self::class_member_name_node(node)
                            .map(|n| n.loc)
                            .unwrap_or(node.loc);
                        fn first_param_is_this(params: &Arc<NodeList>) -> bool {
                            params.iter().next().is_some_and(|p| {
                                matches!(
                                    &p.data,
                                    crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.name.kind == SyntaxKind::Identifier
                    && pd.name.text() == "this")
                            })
                        }
                        let (has_type_params, params, set_has_return) = match &node.data {
                            crate::ast::NodeData::GetAccessorDeclaration(d) => (
                                d.type_parameters.is_some(),
                                Some(&d.parameters),
                                false,
                            ),
                            crate::ast::NodeData::SetAccessorDeclaration(d) => (
                                d.type_parameters.is_some(),
                                Some(&d.parameters),
                                d.type_node.is_some(),
                            ),
                            _ => (false, None, false),
                        };
                        let param_count =
                            params.map_or(0, |p| p.iter().count());
                        let first_is_this =
                            params.is_some_and(first_param_is_this);
                        let expected = if node.kind == SyntaxKind::GetAccessor {
                            0
                        } else {
                            1
                        };
                        let count_correct = param_count == expected
                            || (first_is_this && param_count == expected + 1);
                        let file = self.current_file.clone();
                        if has_type_params {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    AN_ACCESSOR_CANNOT_HAVE_TYPE_PARAMETERS,
                                vec![],
                            ));
                        } else if !count_correct {
                            let message = if node.kind == SyntaxKind::GetAccessor {
                                crate::diagnostics::messages_generated::
                                    A_GET_ACCESSOR_CANNOT_HAVE_PARAMETERS
                            } else {
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_MUST_HAVE_EXACTLY_ONE_PARAMETER
                            };
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                message,
                                vec![],
                            ));
                        } else if node.kind == SyntaxKind::SetAccessor && set_has_return {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    A_SET_ACCESSOR_CANNOT_HAVE_A_RETURN_TYPE_ANNOTATION,
                                vec![],
                            ));
                        }

                        if node.kind == SyntaxKind::GetAccessor
                            && !ambient
                            && let Some(body_node) = &body
                            && !self.function_body_definitely_returns(body_node)
                            && !Self::function_body_has_explicit_return(body_node)
                        {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    A_GET_ACCESSOR_MUST_RETURN_A_VALUE,
                                vec![],
                            ));
                        }
                    }
                }

                if let Some(params) = &parameters {
                    let is_ctor_impl =
                        matches!(node.kind, SyntaxKind::Constructor) && body.is_some();
                    self.check_parameter_property_modifiers(params, is_ctor_impl);

                    if matches!(node.kind, SyntaxKind::MethodDeclaration | SyntaxKind::Constructor)
                    {
                        self.check_parameter_implicit_any(node, params, 0);
                    }
                    for p in params.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);

                            if matches!(
                                node.kind,
                                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                            ) {
                                let _ = self.get_type_from_type_node(pt);
                            }
                        }
                    }
                }
                if let Some(tn) = &type_node {
                    self.check_type_annotation(tn);
                }

                if self.no_implicit_any
                    && matches!(node.kind, SyntaxKind::MethodDeclaration)
                    && type_node.is_none()
                    && body.is_none()
                {
                    if let Some(name) = Self::class_member_name_node(node) {
                        if name.kind == SyntaxKind::Identifier {
                            let file = self.current_file.clone();
                            let diagnostic = crate::ast::Diagnostic::new(
                                file,
                                name.loc,
                                crate::diagnostics::messages_generated::
                                    X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                                vec![name.text().to_string(), "any".to_string()],
                            );
                            self.diagnostics.add(diagnostic);
                        }
                    }
                }
                if let Some(body) = body {

                    if node.kind == SyntaxKind::Constructor
                        && self
                            .enclosing_class_stack
                            .last()
                            .is_some_and(|c| self.extends_base_of(c).is_some())
                    {
                        self.check_super_before_this(&body);
                    }

                    let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                    self.this_container_stack.push(if is_static {
                        ThisContainerKind::StaticMember
                    } else {
                        ThisContainerKind::InstanceMember
                    });
                    self.push_function_scope(node);

                    self.in_ctor_body_stack
                        .push(node.kind == SyntaxKind::Constructor);

                    let declared_return = if node.kind == SyntaxKind::GetAccessor
                        && type_node.is_none()
                        && let Some(hint) = self.accessor_pair_return_hint.take()
                    {
                        Some(hint)
                    } else {
                        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                        type_node
                            .as_ref()
                            .map(|tn| self.get_type_from_type_node(tn))
                            .map(|t| self.unwrap_async_return_type(t, is_async))
                    };
                    self.return_type_stack.push(declared_return.clone());
                    match body.kind {
                        SyntaxKind::Block => self.check_statement(&body),
                        _ => self.check_expression(&body),
                    }
                    self.return_type_stack.pop();
                    self.in_ctor_body_stack.pop();
                    self.pop_function_scope();
                    self.this_container_stack.pop();

                    if let Some(ret_type) = &declared_return
                        && !ret_type.flags.contains(TypeFlags::Void)
                        && !ret_type.flags.contains(TypeFlags::Undefined)
                        && !ret_type.flags.contains(TypeFlags::Any)
                        && body.kind == SyntaxKind::Block
                        && !self.function_body_definitely_returns(&body)
                    {
                        let loc = type_node
                            .as_ref()
                            .map_or(node.loc, |tn| tn.loc);
                        if matches!(node.kind, SyntaxKind::MethodDeclaration) {
                            if Self::function_body_has_explicit_return(&body) {

                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                                    vec![],
                                ));
                            } else {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                    vec![],
                                ));
                            }
                        } else if node.kind == SyntaxKind::GetAccessor {

                            let tgt = self.type_to_string(ret_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec!["undefined".to_string(), tgt],
                            ));
                        }
                    }
                }
            }
            _ => {

            }
        }
    }

    fn check_enum_member(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::EnumMember(data) = &node.data {
            if let Some(init) = &data.initializer {
                self.check_expression(init);

                let ambient = node
                    .parent
                    .as_ref()
                    .is_some_and(|p| p.has_syntactic_modifier(ModifierFlags::Ambient))
                    || self.ambient_context_depth > 0
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                if ambient && !Self::is_constant_enum_initializer(init) {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        init.loc,
                        crate::diagnostics::messages_generated::
                            IN_AMBIENT_ENUM_DECLARATIONS_MEMBER_INITIALIZER_MUST_BE_CONSTANT_EXPRESSION,
                        vec![],
                    ));
                }
            }
        }
    }

    fn is_constant_enum_initializer(init: &Arc<Node>) -> bool {
        match &init.data {
            crate::ast::NodeData::NumericLiteral(_)
            | crate::ast::NodeData::StringLiteral(_)
            | crate::ast::NodeData::NoSubstitutionTemplateLiteral(_) => true,
            crate::ast::NodeData::Identifier(_) => true,
            crate::ast::NodeData::PrefixUnaryExpression(u) => {
                matches!(u.operator, SyntaxKind::PlusToken | SyntaxKind::MinusToken | SyntaxKind::TildeToken)
                    && Self::is_constant_enum_initializer(&u.operand)
            }
            crate::ast::NodeData::BinaryExpression(b) => {
                matches!(
                    b.operator_token.kind,
                    SyntaxKind::PlusToken
                        | SyntaxKind::MinusToken
                        | SyntaxKind::AsteriskToken
                        | SyntaxKind::SlashToken
                        | SyntaxKind::PercentToken
                        | SyntaxKind::LessThanLessThanToken
                        | SyntaxKind::GreaterThanGreaterThanToken
                        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
                        | SyntaxKind::AmpersandToken
                        | SyntaxKind::BarToken
                        | SyntaxKind::CaretToken
                ) && Self::is_constant_enum_initializer(&b.left)
                    && Self::is_constant_enum_initializer(&b.right)
            }
            crate::ast::NodeData::ParenthesizedExpression(p) => {
                Self::is_constant_enum_initializer(&p.expression)
            }
            _ => false,
        }
    }

    pub fn get_declaration_of_kind(
        &self,
        symbol: &Arc<Symbol>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> {
        symbol.declarations.iter().find(|d| d.kind == kind).cloned()
    }

    pub fn get_enum_member_value(&mut self, node: &Arc<Node>) -> EvalResult {
        if let Some(parent) = node.parent.as_ref() {
            self.compute_enum_member_values(parent);
        }
        self.enum_member_links
            .get(node)
            .map(|l| l.value.clone())
            .unwrap_or_else(EvalResult::none)
    }

    fn compute_enum_member_values(&mut self, node: &Arc<Node>) {
        let already = self
            .node_links
            .get(node)
            .map(|l| l.flags.contains(NodeCheckFlags::EnumValuesComputed))
            .unwrap_or(false);
        if already {
            return;
        }
        self.node_links.get_or_default(node).flags |= NodeCheckFlags::EnumValuesComputed;

        let members: Vec<Arc<Node>> = match &node.data {
            NodeData::EnumDeclaration(data) => data.members.iter().cloned().collect(),
            _ => return,
        };

        let mut auto_value: Option<f64> = Some(0.0);
        let mut previous: Option<Arc<Node>> = None;
        for member in &members {
            let result = self.compute_enum_member_value(member, auto_value, previous.as_ref());
            self.enum_member_links.get_or_default(member).value = result.clone();
            if let Some(EvalValue::Number(n)) = &result.value {
                auto_value = Some(n.0 + 1.0);
            } else {
                auto_value = None;
            }
            previous = Some(Arc::clone(member));
        }
    }

    fn compute_enum_member_value(
        &mut self,
        member: &Arc<Node>,
        auto_value: Option<f64>,
        _previous: Option<&Arc<Node>>,
    ) -> EvalResult {
        let has_initializer =
            matches!(&member.data, NodeData::EnumMember(d) if d.initializer.is_some());
        if has_initializer {
            return self.compute_constant_enum_member_value(member);
        }
        match auto_value {
            Some(v) => EvalResult::new(
                Some(EvalValue::Number(jsnum::Number(v))),
                false,
                false,
                false,
            ),
            None => EvalResult::none(),
        }
    }

    fn compute_constant_enum_member_value(&mut self, member: &Arc<Node>) -> EvalResult {
        let initializer = match &member.data {
            NodeData::EnumMember(d) => match &d.initializer {
                Some(init) => Arc::clone(init),
                None => return EvalResult::none(),
            },
            _ => return EvalResult::none(),
        };
        crate::evaluator::evaluate_expression(&initializer, Some(member), noop_entity_fn)
    }

    fn explicit_type_argument_count(node: &Arc<Node>) -> usize {
        match &node.data {
            crate::ast::NodeData::CallExpression(d) => d
                .type_arguments
                .as_ref()
                .map(|t| t.len())
                .unwrap_or(0),
            crate::ast::NodeData::NewExpression(d) => d
                .type_arguments
                .as_ref()
                .map(|t| t.len())
                .unwrap_or(0),
            _ => 0,
        }
    }

    fn has_explicit_type_arguments(node: &Arc<Node>) -> bool {
        Self::explicit_type_argument_count(node) > 0
    }

    fn op_display(kind: crate::ast::SyntaxKind) -> &'static str {
        use crate::ast::SyntaxKind::*;
        match kind {
            AsteriskToken => "*",
            AsteriskAsteriskToken => "**",
            AsteriskEqualsToken => "*=",
            AsteriskAsteriskEqualsToken => "**=",
            SlashToken => "/",
            SlashEqualsToken => "/=",
            PercentToken => "%",
            PercentEqualsToken => "%=",
            MinusToken => "-",
            MinusEqualsToken => "-=",
            PlusToken => "+",
            PlusEqualsToken => "+=",
            LessThanLessThanToken => "<<",
            LessThanLessThanEqualsToken => "<<=",
            GreaterThanGreaterThanToken => ">>",
            GreaterThanGreaterThanEqualsToken => ">>=",
            GreaterThanGreaterThanGreaterThanToken => ">>>",
            GreaterThanGreaterThanGreaterThanEqualsToken => ">>>=",
            BarToken => "|",
            BarEqualsToken => "|=",
            CaretToken => "^",
            CaretEqualsToken => "^=",
            AmpersandToken => "&",
            AmpersandEqualsToken => "&=",
            _ => "?",
        }
    }

    fn nonvariable_assignment_target_type(
        &mut self,
        operand: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        if operand.kind != SyntaxKind::Identifier {
            return None;
        }

        if !Self::is_definite_assignment_target(operand) {
            return None;
        }
        let sym = self.resolve_identifier(operand)?;
        let base = self.resolve_alias_base(sym);
        if base.flags.intersects(SymbolFlags::VARIABLE) {
            return None;
        }
        Some(self.error_type())
    }

    fn check_binary_arith_pre(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;
        let op = data.operator_token.kind;
        let arith_nonplus = matches!(
            op,
            AsteriskToken
                | AsteriskAsteriskToken
                | AsteriskEqualsToken
                | AsteriskAsteriskEqualsToken
                | SlashToken
                | SlashEqualsToken
                | PercentToken
                | PercentEqualsToken
                | MinusToken
                | MinusEqualsToken
                | LessThanLessThanToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | BarToken
                | BarEqualsToken
                | CaretToken
                | CaretEqualsToken
                | AmpersandToken
                | AmpersandEqualsToken
        );
        let plus = op == PlusToken || op == PlusEqualsToken;
        if !arith_nonplus && !plus {
            return;
        }
        for operand in [&data.left, &data.right] {
            if matches!(operand.kind, NullKeyword | UndefinedKeyword) {
                let word = if operand.kind == NullKeyword { "null" } else { "undefined" };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    operand.loc,
                    crate::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                    vec![word.to_string()],
                ));
            }
        }
        if !arith_nonplus {
            return;
        }

        let lt = self
            .nonvariable_assignment_target_type(&data.left)
            .unwrap_or_else(|| self.get_type_of_node(&data.left));
        let rt = self.get_type_of_node(&data.right);
        let boolean_like =
            |t: &Arc<Type>| t.flags.intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral);
        if boolean_like(&lt) && boolean_like(&rt) {
            let suggested = match op {
                AmpersandToken | AmpersandEqualsToken => Some("&&"),
                BarToken | BarEqualsToken => Some("||"),
                CaretToken | CaretEqualsToken => Some("!=="),
                _ => None,
            };
            if let Some(sugg) = suggested {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::
                        THE_0_OPERATOR_IS_NOT_ALLOWED_FOR_BOOLEAN_TYPES_CONSIDER_USING_1_INSTEAD,
                    vec![Self::op_display(op).to_string(), sugg.to_string()],
                ));
            }
            return;
        }
        fn ok_number(c: &mut Checker, t: &Arc<Type>) -> bool {
            let n = c.number_type();
            if c.is_type_assignable_to(t, &n) {
                return true;
            }
            let b = c.bigint_type();
            c.is_type_assignable_to(t, &b)
        }

        let left_is_literal = matches!(data.left.kind, NullKeyword | UndefinedKeyword);
        let right_is_literal = matches!(data.right.kind, NullKeyword | UndefinedKeyword);
        if !left_is_literal && !ok_number(self, &lt) {
            self.arith_operand_error_nodes
                .insert(Arc::as_ptr(node) as *const crate::ast::Node);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                data.left.loc,
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ARITHMETIC_OPERATION_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                Vec::new(),
            ));
        }
        if !right_is_literal && !ok_number(self, &rt) {
            self.arith_operand_error_nodes
                .insert(Arc::as_ptr(node) as *const crate::ast::Node);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                data.right.loc,
                crate::diagnostics::messages_generated::
                    THE_RIGHT_HAND_SIDE_OF_AN_ARITHMETIC_OPERATION_MUST_BE_OF_TYPE_ANY_NUMBER_BIGINT_OR_AN_ENUM_TYPE,
                Vec::new(),
            ));
        }
    }

    fn check_binary_plus_operator_error(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;
        let op = data.operator_token.kind;
        if op != PlusToken && op != PlusEqualsToken {
            return;
        }

        let lt = self
            .nonvariable_assignment_target_type(&data.left)
            .unwrap_or_else(|| self.get_type_of_node(&data.left));
        let rt = self.get_type_of_node(&data.right);
        let number_like = |t: &Arc<Type>| {

            (!self.strict_null_checks
                && t.flags.intersects(
                    TypeFlags::Undefined | TypeFlags::Null,
                ))
                || t.flags.contains(TypeFlags::Never)
                || t.flags.intersects(
                    TypeFlags::Number
                        | TypeFlags::NumberLiteral
                        | TypeFlags::EnumLiteral
                        | TypeFlags::Union,
                )
        };
        let bigint_like = |t: &Arc<Type>| {
            t.flags.intersects(
                TypeFlags::BigInt | TypeFlags::BigIntLiteral | TypeFlags::Union,
            )
        };
        let string_like =
            |t: &Arc<Type>| t.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral);
        let valid = (number_like(&lt) && number_like(&rt))
            || (bigint_like(&lt) && bigint_like(&rt))
            || string_like(&lt)
            || string_like(&rt)
            || lt.flags.contains(TypeFlags::Any)
            || rt.flags.contains(TypeFlags::Any);
        if !valid {
            let lt_str = self.type_to_string(&lt);
            let rt_str = self.type_to_string(&rt);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPES_1_AND_2,
                vec!["+".to_string(), lt_str, rt_str],
            ));
        }
    }

    fn logical_rhs_frame(
        &mut self,
        operator: crate::ast::SyntaxKind,
        target: &Arc<Node>,
    ) -> Option<(Arc<Symbol>, Arc<Type>)> {
        use crate::ast::SyntaxKind::*;
        if !matches!(target.data, crate::ast::NodeData::Identifier(_)) {
            return None;
        }
        let left_type = self.assignment_target_type(target)?;
        let frame = match operator {
            QuestionQuestionEqualsToken => {

                let parts: Vec<Arc<Type>> = self
                    .flow_constituents_public(&left_type)
                    .into_iter()
                    .filter(|c| c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(self.build_union_from_types(parts))
                }
            }
            BarBarEqualsToken => {

                let parts: Vec<Arc<Type>> = self
                    .flow_constituents_public(&left_type)
                    .into_iter()
                    .filter(|c| self.flow_constituent_definitely_falsy(c))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(self.build_union_from_types(parts))
                }
            }
            AmpersandAmpersandEqualsToken => {

                let parts: Vec<Arc<Type>> = self
                    .flow_constituents_public(&left_type)
                    .into_iter()
                    .filter(|c| !self.flow_constituent_definitely_falsy(c))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(self.build_union_from_types(parts))
                }
            }
            _ => None,
        }?;
        let sym = self.resolve_identifier(target)?;
        Some((sym, frame))
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

    fn check_assignment_compat(
        &mut self,
        node: &Arc<Node>,
        data: &crate::ast::node_data_generated::BinaryExpressionData,
    ) {
        use crate::ast::SyntaxKind::*;

        if data.operator_token.kind == EqualsToken
            && matches!(
                data.left.kind,
                ObjectLiteralExpression | ArrayLiteralExpression
            )
        {
            return;
        }

        let mut target: &Arc<Node> = &data.left;
        loop {
            match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    target = &p.expression;
                }
                crate::ast::NodeData::NonNullExpression(n) => {
                    target = &n.expression;
                }
                _ => break,
            }
        }

        let optional_chain = match &target.data {
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                pa.question_dot_token.is_some()
            }
            crate::ast::NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_some()
            }
            _ => false,
        };
        let is_reference = matches!(
            target.kind,
            Identifier | PropertyAccessExpression | ElementAccessExpression
        );
        if !is_reference || optional_chain {
            let message = if optional_chain {
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS
            } else {
                crate::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS
            };

            let loc = if data.left.kind == SyntaxKind::ParenthesizedExpression {
                node.loc
            } else {
                target.loc
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                Vec::new(),
            ));

            self.check_expression(&data.left);
            return;
        }
        let Some(left_type) = self.assignment_target_type(target) else {
            return;
        };

        if target.kind == Identifier {
            if let Some(sym) = self.resolve_identifier(target) {
                let base = self.resolve_alias_base(sym);
                if base.flags.intersects(
                    SymbolFlags::Class | SymbolFlags::ENUM | SymbolFlags::ValueModule,
                ) && !base.flags.intersects(
                    SymbolFlags::VARIABLE
                        | SymbolFlags::PROPERTY_OR_ACCESSOR
                        | SymbolFlags::Function,
                ) {
                    return;
                }

                if self.symbol_is_const_variable(&base) {
                    return;
                }
            }
        }

        if left_type.flags.contains(TypeFlags::Any)
            && left_type.intrinsic_name() == Some("error")
        {
            return;
        }

        if self.assignment_target_is_readonly(target) {
            return;
        }
        let right_type = match data.operator_token.kind {
            EqualsToken => self.get_type_of_node(&data.right),

            AmpersandAmpersandEqualsToken | BarBarEqualsToken
            | QuestionQuestionEqualsToken => {
                match self.logical_rhs_frame(data.operator_token.kind, target) {
                    Some((sym, t)) => {
                        self.logical_rhs_narrowing_frames.push((sym, t));
                        let rt = self.get_type_of_node(&data.right);
                        self.logical_rhs_narrowing_frames.pop();
                        rt
                    }
                    None => self.get_type_of_node(&data.right),
                }
            }

            _ => {
                if self
                    .arith_operand_error_nodes
                    .contains(&(Arc::as_ptr(node) as *const crate::ast::Node))
                {
                    return;
                }
                self.get_type_of_node(node)
            }
        };

        let _ = self.check_type_assignable_to_and_optionally_elaborate(
            &right_type,
            &left_type,
            Some(target),
            Some(&data.right),
            None,
            None,
        );
    }

    fn write_type_of_property_symbol(
        &mut self,
        prop: &Arc<crate::ast::Symbol>,
    ) -> Arc<Type> {
        if prop.flags.contains(SymbolFlags::SetAccessor)
            && let Some(setter) = prop
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SetAccessor)
            && let crate::ast::NodeData::SetAccessorDeclaration(sd) = &setter.data
            && let Some(param) = sd.parameters.iter().next()
            && let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        self.get_type_of_symbol(prop)
    }

    pub(crate) fn assignment_target_type(&mut self, target: &Arc<Node>) -> Option<Arc<Type>> {
        match &target.data {
            crate::ast::NodeData::Identifier(_) => {
                let sym = self.resolve_identifier(target)?;
                let declared = self.get_type_of_symbol(&sym);

                let target_kind = get_assignment_target_kind(target);
                let compound_like = target_kind == AssignmentKind::Definite
                    && is_in_compound_like_assignment(target);
                if compound_like || target_kind == AssignmentKind::Compound {
                    Some(self.get_base_type_of_literal_type(&declared))
                } else {
                    Some(declared)
                }
            }
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);

                self.get_property_of_type(&obj_type, &pa.name.text())
                    .map(|sym| self.write_type_of_property_symbol(&sym))
            }
            crate::ast::NodeData::ElementAccessExpression(ea) => {

                if ea.argument_expression.kind == SyntaxKind::StringLiteral {
                    let obj_type = self.get_type_of_node(&ea.expression);
                    let name = ea.argument_expression.text();
                    if let Some(prop) = self.get_property_of_type(&obj_type, name) {
                        return Some(self.write_type_of_property_symbol(&prop));
                    }
                }
                let obj_type = self.get_type_of_node(&ea.expression);
                let index_type = self.get_type_of_node(&ea.argument_expression);
                Some(self.get_indexed_access_type(&obj_type, &index_type))
            }
            _ => None,
        }
    }

    fn assignment_target_is_readonly(&mut self, target: &Arc<Node>) -> bool {
        match &target.data {
            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let obj_type = self.get_type_of_node(&pa.expression);
                if let Some(sym) = self.get_property_of_type(&obj_type, &pa.name.text())
                    && (sym.check_flags.contains(crate::ast::CheckFlags::Readonly)
                        || sym
                            .declarations
                            .iter()
                            .any(|d| d.has_syntactic_modifier(ModifierFlags::Readonly)))
                {
                    return true;
                }
                self.namespace_const_member(&pa.expression, &pa.name.text())
                    .is_some()
            }

            crate::ast::NodeData::ElementAccessExpression(ea)
                if ea.argument_expression.kind == SyntaxKind::StringLiteral =>
            {
                self.namespace_const_member(
                    &ea.expression,
                    ea.argument_expression.text(),
                )
                .is_some()
            }
            _ => false,
        }
    }

    fn namespace_const_member(
        &mut self,
        obj_expr: &Arc<Node>,
        name: &str,
    ) -> Option<Arc<crate::ast::Symbol>> {
        if obj_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(obj_expr)?;
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return None;
        }
        let member = base
            .exports
            .get(name)
            .or_else(|| base.members.get(name))
            .cloned()
            .or_else(|| {
                base.declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .find_map(|d| {
                        self.program
                            .symbol_map()
                            .locals
                            .get(&d.id())
                            .and_then(|l| l.get(name).cloned())
                    })
            });
        member.filter(|m| self.symbol_is_const_variable(m))
    }

    fn get_type_of_meta_property(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use crate::core::compiler_options::ModuleKind;
        let (keyword_token, name) = match &node.data {
            crate::ast::NodeData::MetaProperty(d) => (d.keyword_token, &d.name),
            _ => return self.error_type(),
        };
        match keyword_token {
            SyntaxKind::NewKeyword => {

                self.any_type()
            }
            SyntaxKind::ImportKeyword => {
                if name.text() == "defer" {
                    return self.error_type();
                }
                if name.text() == "meta" {
                    match self.compiler_options.module {
                        ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20
                        | ModuleKind::NodeNext => {
                            let esm = self
                                .current_file
                                .as_ref()
                                .map(|f| {
                                    self.program_implied_format(&f.file_name)
                                        == ModuleKind::ESNext
                                })
                                .unwrap_or(false);
                            if !esm {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        THE_IMPORT_META_META_PROPERTY_IS_NOT_ALLOWED_IN_FILES_WHICH_WILL_BUILD_INTO_COMMONJS_OUTPUT,
                                    Vec::new(),
                                ));
                            }
                        }
                        m => {

                            let es2020_or_later = matches!(
                                m,
                                ModuleKind::ES2020
                                    | ModuleKind::ES2022
                                    | ModuleKind::ESNext
                                    | ModuleKind::Preserve
                                    | ModuleKind::Node16
                                    | ModuleKind::Node18
                                    | ModuleKind::Node20
                                    | ModuleKind::NodeNext
                            );
                            if !es2020_or_later && m != ModuleKind::System {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        THE_IMPORT_META_META_PROPERTY_IS_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_ES2020_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_OR_NODENEXT,
                                    Vec::new(),
                                ));
                            }
                        }
                    }

                    if let Some(sym) = self.globals.get("ImportMeta").cloned() {
                        return self.resolve_interface_type(&sym, None);
                    }
                    self.any_type()
                } else {

                    self.error_type()
                }
            }
            _ => self.error_type(),
        }
    }

    fn program_implied_format(&self, file_name: &str) -> crate::core::compiler_options::ModuleKind {
        use crate::core::compiler_options::ModuleKind;
        match self.program.get_emit_module_format_of_file(file_name) {
            ModuleKind::None => {

                crate::compiler::implied_node_format_of_file(file_name, &|p| {
                    self.program.read_file(p)
                })
            }
            ModuleKind::ES2020 | ModuleKind::ESNext => ModuleKind::ESNext,
            _ => ModuleKind::CommonJS,
        }
    }

    fn syntactic_truthy_semantics(&mut self, node: &Arc<Node>) -> (bool, bool) {
        let mut n: Arc<Node> = Arc::clone(node);
        loop {
            match &n.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    n = Arc::clone(&p.expression)
                }
                crate::ast::NodeData::NonNullExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::AsExpression(p) => n = Arc::clone(&p.expression),
                crate::ast::NodeData::TypeAssertion(p) => n = Arc::clone(&p.expression),
                _ => break,
            }
        }
        use SyntaxKind::*;
        match n.kind {
            NumericLiteral => {
                let t = n.text();
                if t == "0" || t == "1" {
                    (true, true)
                } else {
                    (true, false)
                }
            }
            ArrayLiteralExpression
            | ArrowFunction
            | BigIntLiteral
            | ClassExpression
            | FunctionExpression
            | JsxElement
            | JsxSelfClosingElement
            | ObjectLiteralExpression
            | RegularExpressionLiteral => (true, false),
            VoidExpression | NullKeyword => (false, true),
            NoSubstitutionTemplateLiteral | StringLiteral => {
                if !n.text().is_empty() {
                    (true, false)
                } else {
                    (false, true)
                }
            }
            ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(d) = &n.data {
                    let (a1, n1) = self.syntactic_truthy_semantics(&d.when_true);
                    let (a2, n2) = self.syntactic_truthy_semantics(&d.when_false);
                    (a1 || a2, n1 || n2)
                } else {
                    (true, true)
                }
            }
            Identifier => {

                if let Some(sym) = self.resolve_identifier(&n)
                    && self.is_undefined_symbol(&sym)
                {
                    return (false, true);
                }
                (true, true)
            }
            _ => (true, true),
        }
    }

    fn check_truthiness_of_type(&mut self, node: &Arc<Node>) {
        let (always, never) = self.syntactic_truthy_semantics(node);
        let message = if always && !never {
            crate::diagnostics::messages_generated::THIS_KIND_OF_EXPRESSION_IS_ALWAYS_TRUTHY
        } else if never && !always {
            crate::diagnostics::messages_generated::THIS_KIND_OF_EXPRESSION_IS_ALWAYS_FALSY
        } else {
            return;
        };
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            message,
            Vec::new(),
        ));
    }

    pub fn check_expression(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));

        self.type_instantiation_count = 0;
        match node.kind {
            SyntaxKind::Identifier => {
                self.check_identifier_reference(node);
            }
            SyntaxKind::NumericLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => {

            }
            SyntaxKind::MetaProperty => {

                let _ = self.get_type_of_node(node);
            }
            SyntaxKind::BinaryExpression => {
                if let crate::ast::NodeData::BinaryExpression(data) = &node.data {

                    self.check_binary_arith_pre(node, data);

                    if data.operator_token.kind == SyntaxKind::CommaToken
                        && !self.is_indirect_call_comma(node)
                        && !self.expression_has_side_effects(&data.left)
                        && !self.diagnostics.get_all().iter().any(|d| {
                            d.code == 2695
                                && d.file
                                    .as_ref()
                                    .map(|f| Arc::ptr_eq(f, self.current_file.as_ref().unwrap_or(&f)))
                                    .unwrap_or(false)
                                && d.loc == data.left.loc
                        })
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.left.loc,
                            crate::diagnostics::messages_generated::
                                LEFT_SIDE_OF_COMMA_OPERATOR_IS_UNUSED_AND_HAS_NO_SIDE_EFFECTS,
                            Vec::new(),
                        ));
                    }
                    self.check_expression(&data.left);

                    if matches!(
                        data.operator_token.kind,
                        crate::ast::SyntaxKind::AmpersandAmpersandToken
                            | crate::ast::SyntaxKind::BarBarToken
                    ) {
                        self.check_truthiness_of_type(&data.left);
                    }

                    let rhs_frame = {
                        let mut lhs: &Arc<Node> = &data.left;
                        loop {
                            match &lhs.data {
                                crate::ast::NodeData::ParenthesizedExpression(p) => {
                                    lhs = &p.expression;
                                }
                                crate::ast::NodeData::NonNullExpression(n) => {
                                    lhs = &n.expression;
                                }
                                _ => break,
                            }
                        }
                        if matches!(
                            data.operator_token.kind,
                            crate::ast::SyntaxKind::QuestionQuestionEqualsToken
                                | crate::ast::SyntaxKind::BarBarEqualsToken
                                | crate::ast::SyntaxKind::AmpersandAmpersandEqualsToken
                        ) {
                            self.logical_rhs_frame(data.operator_token.kind, lhs)
                        } else {
                            None
                        }
                    };
                    match rhs_frame {
                        Some((sym, t)) => {
                            self.logical_rhs_narrowing_frames.push((sym, t));
                            self.check_expression(&data.right);
                            self.logical_rhs_narrowing_frames.pop();
                        }
                        None => self.check_expression(&data.right),
                    }
                    self.check_binary_plus_operator_error(node, data);
                    use crate::ast::SyntaxKind::*;

                    if data.operator_token.kind == EqualsToken
                        && data.left.kind == SyntaxKind::PropertyAccessExpression
                    {
                        if let crate::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                        {
                            let obj_type = self.get_type_of_node(&pa.expression);
                            let name_text = pa.name.text();
                            if self.is_property_readonly(&obj_type, name_text) {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    pa.name.loc,
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                                    vec![name_text.to_string()],
                                ));
                            }
                        }
                    }
                    let mut assigned_target_blocks_type_check = false;

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && data.left.kind == SyntaxKind::PropertyAccessExpression
                        && let crate::ast::NodeData::PropertyAccessExpression(pa) = &data.left.data
                        && pa.expression.kind == SyntaxKind::Identifier
                        && let Some(enum_sym) = self.resolve_identifier(&pa.expression)
                        && self
                            .resolve_alias_base(enum_sym)
                            .flags
                            .intersects(SymbolFlags::ENUM)
                    {
                        let name_text = pa.name.text();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            pa.name.loc,
                            CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                            vec![name_text.to_string()],
                        ));

                        assigned_target_blocks_type_check = true;
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        let name_text = data.left.text().to_string();
                        if let Some(sym) = self.resolve_identifier(&data.left)
                            && let base = self.resolve_alias_base(sym)
                        {
                            let msg = if base.flags.contains(SymbolFlags::Class) {
                                Some(crate::diagnostics::messages_generated::
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CLASS)
                            } else if base.flags.intersects(SymbolFlags::ENUM) {
                                Some(crate::diagnostics::messages_generated::
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_ENUM)
                            } else if base.flags.contains(SymbolFlags::Function) {
                                Some(crate::diagnostics::messages_generated::
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_FUNCTION)
                            } else {
                                None
                            };
                            if let Some(msg) = msg {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    data.left.loc,
                                    msg,
                                    vec![name_text],
                                ));

                                assigned_target_blocks_type_check = true;
                            }
                        }
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        if let Some(symbol) = self.resolve_identifier(&data.left) {
                            if self.symbol_is_const_variable(&symbol) {
                                let name_text = data.left.text();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    data.left.loc,
                                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
                                    vec![name_text.to_string()],
                                ));
                            }
                        }
                    }

                    if data.operator_token.kind == EqualsToken
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        if let Some(target) = self.declared_annotation_type_of(&data.left) {

                            if matches!(
                                data.right.kind,
                                SyntaxKind::ObjectLiteralExpression
                                    | SyntaxKind::ArrayLiteralExpression
                                    | SyntaxKind::TypeAssertionExpression
                                    | SyntaxKind::AsExpression
                            ) {
                                self.check_contextual_elements(
                                    &data.right,
                                    &target,
                                    data.right.loc,
                                );
                            }
                        }
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && matches!(
                            data.left.kind,
                            SyntaxKind::PropertyAccessExpression
                                | SyntaxKind::ElementAccessExpression
                        )
                    {
                        self.check_const_property_assignment(&data.left);
                    }

                    if Self::is_assignment_operator(data.operator_token.kind)
                        && !assigned_target_blocks_type_check
                    {
                        self.check_assignment_compat(node, data);
                    }

                    let is_equality_op = matches!(
                        data.operator_token.kind,
                        EqualsEqualsToken
                            | ExclamationEqualsToken
                            | EqualsEqualsEqualsToken
                            | ExclamationEqualsEqualsToken
                    );
                    if is_equality_op {
                        let left_type = self.get_type_of_node(&data.left);
                        let right_type = self.get_type_of_node(&data.right);

                        let skip_flags = TypeFlags::Any
                            .union(TypeFlags::Unknown)
                            .union(TypeFlags::Never)
                            .union(TypeFlags::Null)
                            .union(TypeFlags::Undefined);
                        if !left_type.flags.intersects(skip_flags)
                            && !right_type.flags.intersects(skip_flags)
                            && !self.are_types_comparable(&left_type, &right_type)
                        {
                            let left_str = self.type_to_string(&left_type);
                            let right_str = self.type_to_string(&right_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                node.loc,
                                THIS_COMPARISON_APPEARS_TO_BE_UNINTENTIONAL_BECAUSE_THE_TYPES_0_AND_1_HAVE_NO_OVERLAP,
                                vec![left_str, right_str],
                            ));
                        }
                    }
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let crate::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);

                    if data.operator == SyntaxKind::ExclamationToken {
                        self.check_truthiness_of_type(&data.operand);
                    }

                    if matches!(data.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken) {
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let crate::ast::NodeData::PostfixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);
                    if matches!(data.operator, SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken) {
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::ParenthesizedExpression => {
                if let crate::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ClassExpression => {

                if let crate::ast::NodeData::ClassExpression(data) = &node.data {
                    self.enclosing_class_stack.push(Arc::clone(node));

                    self.push_scope(node);

                    let this_type = self.build_class_instance_type_with_base(node);
                    self.this_type_stack.push(this_type);
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                    self.this_type_stack.pop();
                    self.pop_scope();
                    self.enclosing_class_stack.pop();
                }
            }
            SyntaxKind::CallExpression => {
                if let crate::ast::NodeData::CallExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    for (i, arg) in data.arguments.iter().enumerate() {
                        self.check_call_arg_with_context(&data.expression, i, arg);
                    }
                }
                self.check_call_arguments(node,  false);
            }
            SyntaxKind::NewExpression => {
                if let crate::ast::NodeData::NewExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    if let Some(args) = &data.arguments {
                        for (i, arg) in args.iter().enumerate() {
                            self.check_call_arg_with_context(&data.expression, i, arg);
                        }
                    }

                    let mut reported_abstract = false;
                    if data.expression.kind == SyntaxKind::Identifier {
                        if let Some(symbol) = self.resolve_identifier(&data.expression) {
                            if self.symbol_is_abstract_class(&symbol) {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                                    vec![],
                                ));
                                reported_abstract = true;
                            }
                        }
                    }

                    if !reported_abstract {
                        let callee_type = self.get_type_of_node(&data.expression);
                        if self.type_includes_abstract_constructor(&callee_type) {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                node.loc,
                                CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                                vec![],
                            ));
                        }
                    }
                }
                self.check_call_arguments(node,  true);
            }
            SyntaxKind::PropertyAccessExpression => {

                if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
                self.check_property_access(node);
            }
            SyntaxKind::ElementAccessExpression => {
                if let crate::ast::NodeData::ElementAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_expression(&data.argument_expression);

                    if data.question_dot_token.is_none() {
                        let obj_type = self.get_type_of_node(&data.expression);
                        self.report_possibly_null_or_undefined(
                            &data.expression,
                            &obj_type,
                            false,
                        );
                    }
                }
            }
            SyntaxKind::ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(data) = &node.data {
                    self.check_expression(&data.condition);
                    self.check_truthiness_of_type(&data.condition);
                    self.check_expression(&data.when_true);
                    self.check_expression(&data.when_false);
                }
            }
            SyntaxKind::ArrayLiteralExpression => {
                if let crate::ast::NodeData::ArrayLiteralExpression(data) = &node.data {
                    for elem in data.elements.iter() {
                        self.check_expression(elem);
                    }
                }
            }
            SyntaxKind::ObjectLiteralExpression => {
                if let crate::ast::NodeData::ObjectLiteralExpression(data) = &node.data {

                    let is_destructuring_assignment_target = node.parent.as_ref().is_some_and(
                        |p| match &p.data {
                            crate::ast::NodeData::BinaryExpression(b) => {
                                b.operator_token.kind == SyntaxKind::EqualsToken
                                    && Arc::ptr_eq(&b.left, node)
                            }
                            _ => false,
                        },
                    );
                    if is_destructuring_assignment_target
                        && self.in_ctor_body_stack.last() == Some(&true)
                        && let Some(rhs) = node.parent.as_ref().and_then(|p| {
                            match &p.data {
                                crate::ast::NodeData::BinaryExpression(b) => {
                                    Some(Arc::clone(&b.right))
                                }
                                _ => None,
                            }
                        })
                        && rhs.kind == SyntaxKind::ThisKeyword
                    {
                        let this_type = self.get_type_of_node(&rhs);
                        for prop in data.properties.iter() {
                            let Some(name_node) = prop.name() else { continue };
                            if name_node.kind == SyntaxKind::ComputedPropertyName {
                                continue;
                            }
                            let prop_text = name_node.text().to_string();
                            self.report_abstract_property_access_in_ctor(
                                &name_node,
                                &prop_text,
                                &this_type,
                            );
                        }
                    }

                    if !is_destructuring_assignment_target {
                        {
                            let mut seen: std::collections::HashMap<String, Vec<&Arc<Node>>> =
                                std::collections::HashMap::new();
                        for prop in data.properties.iter() {
                            let Some(name_node) = prop.name() else {
                                continue;
                            };
                            let name = if name_node.kind == SyntaxKind::ComputedPropertyName {

                                let expr = match &name_node.data {
                                    crate::ast::NodeData::ComputedPropertyName(c) => {
                                        Arc::clone(&c.expression)
                                    }
                                    _ => Arc::clone(name_node),
                                };
                                match expr.kind {
                                    SyntaxKind::NumericLiteral
                                    | SyntaxKind::StringLiteral
                                    | SyntaxKind::Identifier => expr.text().to_string(),
                                    SyntaxKind::PrefixUnaryExpression => {
                                        let crate::ast::NodeData::PrefixUnaryExpression(u) =
                                            &expr.data
                                        else {
                                            continue;
                                        };
                                        let sign = if u.operator == SyntaxKind::MinusToken {
                                            "-"
                                        } else {
                                            ""
                                        };
                                        match &u.operand.data {
                                            crate::ast::NodeData::NumericLiteral(n) => {
                                                format!("{sign}{}", n.text)
                                            }
                                            _ => continue,
                                        }
                                    }
                                    SyntaxKind::PropertyAccessExpression => {

                                        let sym = self.resolve_qualified_symbol(&expr);
                                        match sym.as_ref().and_then(|s| s.value_declaration.clone())
                                        {
                                            Some(decl) => match self.get_constant_value(&decl) {
                                                Some(v) => v,
                                                None => continue,
                                            },
                                            None => continue,
                                        }
                                    }
                                    _ => continue,
                                }
                            } else {
                                match name_node.kind {
                                    SyntaxKind::StringLiteral
                                    | SyntaxKind::NumericLiteral
                                    | SyntaxKind::Identifier => name_node.text().to_string(),
                                    _ => continue,
                                }
                            };
                            seen.entry(name).or_default().push(prop);
                        }
                        for (_, group) in seen.iter() {

                            let accessor_pair = group.iter().all(|p| {
                                matches!(p.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
                            }) && group.len() == 2;
                            if group.len() > 1 && !accessor_pair {
                                for (i, prop) in group.iter().enumerate() {
                                    if i == 0 {
                                        continue;
                                    }
                                    if let Some(name_node) = prop.name() {
                                        let name = name_node.text().to_string();
                                        let file = self.current_file.clone();
                                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                            file,
                                            name_node.loc,
                                            crate::diagnostics::messages_generated::
                                                AN_OBJECT_LITERAL_CANNOT_HAVE_MULTIPLE_PROPERTIES_WITH_THE_SAME_NAME,
                                            vec![name],
                                        ));
                                    }
                                }
                            }
                        }
                        }
                    }

                    for prop in data.properties.iter() {

                        let has_setter = data.properties.iter().any(|p| {
                            p.kind == SyntaxKind::SetAccessor
                                && p.name().is_some_and(|n| {
                                    n.text()
                                        == prop.name().map(|n| n.text()).unwrap_or_default()
                                })
                        });
                        if prop.kind == SyntaxKind::GetAccessor
                            && !has_setter
                            && self.no_implicit_any
                            && let crate::ast::NodeData::GetAccessorDeclaration(gd) = &prop.data
                            && gd.type_node.is_none()
                            && self.getter_return_reaches_this(prop)
                        {
                            let name_loc = Self::member_name_node(prop)
                                .map(|n| n.loc)
                                .unwrap_or(prop.loc);
                            let name = Self::member_name_node(prop)
                                .map(|n| n.text().to_string())
                                .unwrap_or_default();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                name_loc,
                                crate::diagnostics::messages_generated::
                                    X_0_IMPLICITLY_HAS_RETURN_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_RETURN_TYPE_ANNOTATION_AND_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ONE_OF_ITS_RETURN_EXPRESSIONS,
                                vec![name],
                            ));
                        }
                    }

                    let this_typed = self.no_implicit_this
                        || self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| {
                                f.file_name.ends_with(".js") || f.file_name.ends_with(".jsx")
                            });

                    let mut contextual_this: Option<Arc<Type>> = None;
                    {
                        let mut literal = Arc::clone(node);
                        loop {
                            let ctx = self.get_contextual_type(&literal, ContextFlags::None);
                            if let Some(t) = ctx
                                .as_ref()
                                .and_then(|t| self.this_type_marker_argument(t, 0))
                            {
                                contextual_this = Some(t);
                                break;
                            }
                            match &literal.parent.as_ref().map(|p| (p.kind, p.parent.clone())) {
                                Some((SyntaxKind::PropertyAssignment, Some(pp))) => {
                                    literal = Arc::clone(pp);
                                }
                                _ => break,
                            }
                        }
                    }
                    let literal_this = match contextual_this {
                        Some(t) => t,
                        None => self.build_object_literal_this_type(node),
                    };
                    for prop in data.properties.iter() {

                        let method_like = matches!(
                            prop.kind,
                            SyntaxKind::MethodDeclaration
                                | SyntaxKind::GetAccessor
                                | SyntaxKind::SetAccessor
                        );

                        if let Some(name) = Self::member_name_node(prop) {
                            self.check_computed_property_name(&name);
                        }
                        if method_like && this_typed {
                            self.this_type_stack.push(Arc::clone(&literal_this));
                        }
                        self.check_object_literal_element(prop);
                        if method_like && this_typed {
                            self.this_type_stack.pop();
                        }
                    }
                }
            }
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression => {

                let mut contextual_param_count = self
                    .call_arg_arrow_context
                    .last_mut()
                    .map(|v| std::mem::replace(v, 0))
                    .unwrap_or(0);
                if contextual_param_count == 0 {

                    contextual_param_count = self
                        .contextual_signature_of_arrow(node)
                        .map_or(0, |sig| sig.parameters.len());
                }
                match &node.data {
                    crate::ast::NodeData::ArrowFunction(d) => {
                        self.check_parameter_property_modifiers(&d.parameters, false);
                        self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);

                        for param in d.parameters.iter() {
                            self.check_parameter_default_initializer(param);
                        }
                    }
                    crate::ast::NodeData::FunctionExpression(d) => {
                        self.check_parameter_property_modifiers(&d.parameters, false);
                        self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);
                        for param in d.parameters.iter() {
                            self.check_parameter_default_initializer(param);
                        }
                    }
                    _ => {}
                }

                if matches!(node.data, crate::ast::NodeData::FunctionExpression(_)) {
                    self.this_container_stack
                        .push(ThisContainerKind::PlainFunction);
                }
                self.check_function_like_body(node);
                if matches!(node.data, crate::ast::NodeData::FunctionExpression(_)) {
                    self.this_container_stack.pop();
                }
            }
            SyntaxKind::TemplateExpression => {
                if let crate::ast::NodeData::TemplateExpression(data) = &node.data {
                    for span in data.template_spans.iter() {
                        if let crate::ast::NodeData::TemplateSpan(span_data) = &span.data {
                            self.check_expression(&span_data.expression);
                        }
                    }
                }
            }
            SyntaxKind::AwaitExpression => {
                if let crate::ast::NodeData::AwaitExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::YieldExpression => {
                if let crate::ast::NodeData::YieldExpression(data) = &node.data {

                    if !self.enclosing_function_is_generator(node) {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            node.loc,
                            crate::diagnostics::messages_generated::
                                A_YIELD_EXPRESSION_IS_ONLY_ALLOWED_IN_A_GENERATOR_BODY,
                            vec![],
                        ));
                    }
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            SyntaxKind::SpreadElement => {
                if let crate::ast::NodeData::SpreadElement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::AsExpression => {

                if let crate::ast::NodeData::AsExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(
                        node,
                        &data.expression,
                        &data.type_node,
                    );

                    if Self::is_const_type_node(&data.type_node)
                        && !self.is_valid_const_assertion_argument(&data.expression)
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.expression.loc,
                            crate::diagnostics::messages_generated::
                                A_CONST_ASSERTION_CAN_ONLY_BE_APPLIED_TO_REFERENCES_TO_ENUM_MEMBERS_OR_STRING_NUMBER_BOOLEAN_ARRAY_OR_OBJECT_LITERALS,
                            vec![],
                        ));
                    }
                }
            }
            SyntaxKind::TypeAssertionExpression => {

                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(
                        node,
                        &data.expression,
                        &data.type_node,
                    );
                }
            }
            SyntaxKind::NonNullExpression => {
                if let crate::ast::NodeData::NonNullExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SatisfiesExpression => {
                if let crate::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TypeOfExpression => {
                if let crate::ast::NodeData::TypeOfExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::DeleteExpression => {
                if let crate::ast::NodeData::DeleteExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_delete_operand(&data.expression);
                }
            }
            SyntaxKind::VoidExpression => {
                if let crate::ast::NodeData::VoidExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TaggedTemplateExpression => {
                if let crate::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    self.check_expression(&data.tag);
                    self.check_expression(&data.template);
                }
            }
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {

                let opening = match node.kind {
                    SyntaxKind::JsxElement => match &node.data {
                        crate::ast::NodeData::JsxElement(d) => Some(Arc::clone(&d.opening_element)),
                        _ => None,
                    },
                    SyntaxKind::JsxSelfClosingElement => Some(Arc::clone(node)),
                    SyntaxKind::JsxFragment => match &node.data {
                        crate::ast::NodeData::JsxFragment(d) => {
                            Some(Arc::clone(&d.opening_fragment))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(opening) = opening {
                    self.check_jsx_opening_like_element(&opening);
                }

                if node.kind == SyntaxKind::JsxElement {
                    if let crate::ast::NodeData::JsxElement(d) = &node.data
                        && crate::checker::jsx::is_jsx_intrinsic_tag_name(
                            &crate::checker::jsx::jsx_tag_name(&d.closing_element)
                                .unwrap_or_else(|| d.closing_element.clone()),
                        )
                    {
                        self.check_jsx_intrinsic_element(&d.closing_element);
                    }
                }
                self.check_jsx_element(node);
            }
            SyntaxKind::JsxExpression => {
                if let crate::ast::NodeData::JsxExpression(data) = &node.data {

                    self.check_grammar_jsx_expression(node);
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            _ => {

                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }

    fn collect_return_expressions(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
        crate::ast::node_data_generated::for_each_child(node, |child| {
            match child.kind {
                SyntaxKind::ReturnStatement => {
                    if let crate::ast::NodeData::ReturnStatement(r) = &child.data
                        && let Some(expr) = &r.expression
                    {
                        out.push(Arc::clone(expr));
                    }
                    false
                }
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => false,
                _ => {
                    Self::collect_return_expressions(child, out);
                    false
                }
            }
        });
    }

    fn subtree_contains_this(node: &Arc<Node>) -> bool {
        let mut found = false;
        fn walk(root: &Arc<Node>, n: &Arc<Node>, found: &mut bool) {
            if *found {
                return;
            }
            if n.kind == SyntaxKind::ThisKeyword {
                *found = true;
                return;
            }

            if !Arc::ptr_eq(n, root)
                && matches!(
                    n.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                )
            {
                return;
            }
            crate::ast::node_data_generated::for_each_child(n, |c| {
                walk(root, c, found);
                *found
            });
        }

        if matches!(
            node.kind,
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
        ) {
            return false;
        }
        walk(node, node, &mut found);
        found
    }

    fn getter_return_reaches_this(&mut self, accessor: &Arc<Node>) -> bool {
        let crate::ast::NodeData::GetAccessorDeclaration(gd) = &accessor.data else {
            return false;
        };
        let Some(body) = &gd.body else {
            return false;
        };
        let mut returns = Vec::new();
        Self::collect_return_expressions(&body, &mut returns);
        if returns.is_empty() {
            return false;
        }

        let mut this_aliases: Vec<String> = Vec::new();
        crate::ast::node_data_generated::for_each_child(&body, |stmt| {
            if stmt.kind == SyntaxKind::VariableStatement
                && let crate::ast::NodeData::VariableStatement(vs) = &stmt.data
                && let crate::ast::NodeData::VariableDeclarationList(vdl) =
                    &vs.declaration_list.data
            {
                for decl in vdl.declarations.iter() {
                    if let (Some(name), Some(init)) = (decl.name(), {
                            match &decl.data {
                                crate::ast::NodeData::VariableDeclaration(vd) => vd.initializer.clone(),
                                _ => None,
                            }
                        }) {
                        if name.kind == SyntaxKind::Identifier
                            && Self::subtree_contains_this(&init)
                        {
                            this_aliases.push(name.text().to_string());
                        }
                    }
                }
            }
            false
        });
        returns.iter().any(|r| {
            Self::subtree_contains_this(r)
                || {
                    let mut hit = false;
                    fn walk(n: &Arc<Node>, aliases: &[String], hit: &mut bool) {
                        if *hit {
                            return;
                        }
                        if n.kind == SyntaxKind::Identifier
                            && aliases.iter().any(|a| a == n.text())
                        {
                            *hit = true;
                            return;
                        }
                        crate::ast::node_data_generated::for_each_child(n, |c| {
                            walk(c, aliases, hit);
                            *hit
                        });
                    }
                    walk(r, &this_aliases, &mut hit);
                    hit
                }
        })
    }

    fn this_type_marker_argument(&self, t: &Arc<Type>, depth: usize) -> Option<Arc<Type>> {
        if depth > 4 {
            return None;
        }
        let constituent_types: Option<Vec<Arc<Type>>> = match &t.data {
            TypeData::Union(u) => Some(u.union_or_intersection.types.to_vec()),
            TypeData::Intersection(i) => Some(i.union_or_intersection.types.to_vec()),
            _ => None,
        };
        if let Some(types) = constituent_types {
            return types
                .iter()
                .find_map(|c| self.this_type_marker_argument(c, depth + 1));
        }
        let obj = t.as_object()?;
        if obj.type_arguments.len() == 1
            && t.symbol.as_ref().is_some_and(|s| s.name == "ThisType")
        {
            return Some(Arc::clone(&obj.type_arguments[0]));
        }
        None
    }

    fn build_object_literal_this_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let crate::ast::NodeData::ObjectLiteralExpression(data) = &node.data else {
            return self.get_any_type();
        };
        let mut symbol_table = crate::ast::SymbolTable::new();
        let mut props: Vec<Arc<crate::ast::Symbol>> = Vec::new();
        for prop in data.properties.iter() {
            let Some(name_node) = Self::member_name_node(prop) else {
                continue;
            };
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
            ) {
                continue;
            }
            let name = name_node.text().to_string();
            let (member_type, readonly) = match &prop.data {
                crate::ast::NodeData::PropertyAssignment(pa) => {

                    let t = self.get_type_of_node(&pa.initializer);
                    (self.get_widened_type_of_literal(&t), false)
                }
                crate::ast::NodeData::ShorthandPropertyAssignment(sa) => {
                    let t = self.get_type_of_node(&sa.name);
                    (t, false)
                }
                crate::ast::NodeData::GetAccessorDeclaration(gd) => {
                    let t = match &gd.type_node {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    (t, true)
                }

                crate::ast::NodeData::MethodDeclaration(_) => {
                    let mut method_sym = crate::ast::Symbol::new(
                        crate::ast::SymbolFlags::Method,
                        name.clone(),
                    );
                    method_sym.declarations = vec![Arc::clone(prop)];
                    let method_sym = Arc::new(method_sym);
                    symbol_table.insert(name, Arc::clone(&method_sym));
                    props.push(method_sym);
                    continue;
                }
                _ => continue,
            };
            let prop_sym = Arc::new(crate::ast::Symbol::new(
                crate::ast::SymbolFlags::Property,
                name.clone(),
            ));
            if readonly {

                let sym_mut = Arc::as_ptr(&prop_sym) as *mut crate::ast::Symbol;
                unsafe {
                    (*sym_mut).check_flags |= crate::ast::CheckFlags::Readonly;
                }
            }
            self.value_symbol_links.insert(
                &prop_sym,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name, Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Anonymous
                | crate::checker::types::ObjectFlags::ObjectLiteral,
            id: 0,
            symbol: None,
            alias: None,
            data: crate::checker::types::TypeData::Object(
                crate::checker::types::ObjectTypeData {
                    structured: crate::checker::types::StructuredTypeData {
                        members: symbol_table,
                        properties: props,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
        })
    }

    fn check_object_literal_element(&mut self, node: &Arc<Node>) {

        if let Some(name) = node.name()
            && name.kind == SyntaxKind::PrivateIdentifier
        {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                vec![],
            ));
            return;
        }

        match node.kind {
            SyntaxKind::PropertyAssignment => {
                if let crate::ast::NodeData::PropertyAssignment(data) = &node.data {

                    self.check_expression(&data.initializer);
                }
            }
            SyntaxKind::ShorthandPropertyAssignment => {

                if let crate::ast::NodeData::ShorthandPropertyAssignment(data) = &node.data {
                    self.check_identifier_reference(&data.name);
                }
            }
            SyntaxKind::SpreadAssignment => {
                if let crate::ast::NodeData::SpreadAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {

                self.check_class_member(node);
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
    }

    fn check_function_like_body(&mut self, node: &Arc<Node>) {

        self.get_type_of_node(node);

        self.in_ctor_body_stack.push(false);
        let (body, type_node): (Option<Arc<Node>>, Option<Arc<Node>>) = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => {
                (Some(data.body.clone()), data.type_node.clone())
            }
            crate::ast::NodeData::ArrowFunction(data) => {
                (Some(data.body.clone()), data.type_node.clone())
            }
            _ => (None, None),
        };
        if let Some(body) = body {

            let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
            if is_arrow {
                self.push_arrow_function_scope(node);
            } else {
                self.push_function_scope(node);
            }

            let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
            let declared_return = type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn))
                .map(|t| self.unwrap_async_return_type(t, is_async));
            self.return_type_stack.push(declared_return);
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => {

                    self.check_expression(&body);
                    if let Some(expected) =
                        self.return_type_stack.last().and_then(|opt| opt.clone())
                    {
                        let actual = self.get_type_of_node(&body);
                        if !actual.flags.contains(TypeFlags::Any)
                            && !self.is_type_assignable_to(&actual, &expected)
                        {
                            let actual_str = self.type_to_string(&actual);
                            let expected_str = self.type_to_string(&expected);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                body.loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![actual_str, expected_str],
                            ));
                        }
                    }
                }
            }
            self.return_type_stack.pop();
            self.in_ctor_body_stack.pop();
            if is_arrow {
                self.pop_arrow_function_scope();
            } else {
                self.pop_function_scope();
            }
        }
    }

    fn walk_children_for_expressions(&mut self, node: &Arc<Node>) {

        let children: Vec<Arc<Node>> = {
            let mut collected = Vec::new();
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collected.push(Arc::clone(child));
                false
            });
            collected
        };
        for child in &children {

            if is_expression_position_kind(child.kind) {
                self.check_expression(child);
            } else if is_statement_kind(child.kind) {
                self.check_statement(child);
            }

        }
    }

    fn check_jsx_element(&mut self, node: &Arc<Node>) {

        let opening_element: Option<Arc<Node>> = match &node.data {
            crate::ast::NodeData::JsxElement(data) => Some(Arc::clone(&data.opening_element)),
            crate::ast::NodeData::JsxSelfClosingElement(_) => Some(Arc::clone(node)),
            _ => None,
        };
        let children: Vec<Arc<Node>> = match &node.data {
            crate::ast::NodeData::JsxElement(data) => data.children.iter().cloned().collect(),
            crate::ast::NodeData::JsxFragment(data) => data.children.iter().cloned().collect(),
            _ => Vec::new(),
        };

        if let Some(opening) = opening_element {
            let attributes: Option<Arc<Node>> = match &opening.data {
                crate::ast::NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.attributes)),
                crate::ast::NodeData::JsxSelfClosingElement(data) => {
                    Some(Arc::clone(&data.attributes))
                }
                _ => None,
            };
            if let Some(attrs) = attributes {
                if let crate::ast::NodeData::JsxAttributes(data) = &attrs.data {
                    for attr in data.properties.iter() {
                        self.check_jsx_attribute(attr);
                    }
                }
            }

        }

        for child in &children {
            self.check_jsx_child(child);
        }
    }

    fn check_jsx_attribute(&mut self, node: &Arc<Node>) {
        match &node.data {
            crate::ast::NodeData::JsxAttribute(data) => {
                if let Some(init) = &data.initializer {
                    self.check_expression(init);
                }
            }
            crate::ast::NodeData::JsxSpreadAttribute(data) => {
                self.check_expression(&data.expression);
            }
            _ => {}
        }
    }

    fn check_jsx_child(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                self.check_expression(node);
            }
            SyntaxKind::JsxExpression => {
                self.check_expression(node);
            }

            _ => {}
        }
    }

    fn cannot_find_name_message_for(name: &str) -> Option<&'static crate::diagnostics::Message> {
        use crate::diagnostics::messages_generated as mg;
        match name {
            "document" | "console" => Some(
                &mg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM,
            ),
            "process" | "require" | "Buffer" | "module" | "NodeJS" => Some(
                &mg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            ),
            _ => None,
        }
    }

    fn check_parameter_default_initializer(&mut self, param: &Arc<Node>) {
        if let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(init) = &pd.initializer
        {
            self.check_expression(init);
        }
    }

    fn check_identifier_reference(&mut self, node: &Arc<Node>) {

        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return,
        };

        if name.is_empty() {
            return;
        }

        if !is_valid_identifier_text(name) {
            return;
        }

        if is_declaration_name(node) {
            return;
        }

        if is_property_access_name(node) {
            return;
        }

        if self.check_invalid_initializer_reference(node, name) {
            return;
        }

        if !self.ts2304_reporting_allowed_for(node) {
            return;
        }

        if let Some(symbol) = self.resolve_identifier(node) {

            if name == "arguments"
                && self.arguments_symbol.is_some()
                && Arc::ptr_eq(&symbol, self.arguments_symbol.as_ref().unwrap())
            {
                let mut cur = node.parent.as_ref();
                let mut in_initializer_or_static_block = false;
                while let Some(a) = cur {
                    match a.kind {
                        SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor => break,
                        SyntaxKind::ArrowFunction => {
                            cur = a.parent.as_ref();
                            continue;
                        }
                        SyntaxKind::PropertyDeclaration
                        | SyntaxKind::ClassStaticBlockDeclaration => {
                            in_initializer_or_static_block = true;
                            break;
                        }
                        _ => {}
                    }
                    cur = a.parent.as_ref();
                }
                if in_initializer_or_static_block {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        crate::diagnostics::messages_generated::
                            X_ARGUMENTS_CANNOT_BE_REFERENCED_IN_PROPERTY_INITIALIZERS_OR_CLASS_STATIC_INITIALIZATION_BLOCKS,
                        Vec::new(),
                    ));
                    return;
                }
            }

            let is_export_assignment_name = node
                .parent
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::ExportAssignment);
            let base = self.resolve_alias_base(Arc::clone(&symbol));

            let is_true_namespace = base
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ModuleDeclaration
                    && d.name().is_some_and(|n| {
                        !matches!(n.kind, SyntaxKind::StringLiteral)
                    }));
            if !is_export_assignment_name
                && base.flags.contains(SymbolFlags::ValueModule)
                && is_true_namespace
                && !self.namespace_usable_as_value(&base)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                    vec![name.to_string()],
                ));
                return;
            }

            self.check_block_scoped_variable_used_before_declaration(node, &symbol, name);

            self.check_variable_used_before_assigned(node, &symbol, name);
            return;
        }

        let file = self.current_file.clone();

        {
            let is_primitive_type_name = matches!(
                name,
                "any" | "string" | "number" | "boolean" | "never" | "unknown"
            );
            let reported = if is_primitive_type_name {

                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file.clone(),
                    node.loc,
                    crate::diagnostics::messages_generated::
                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                    vec![name.to_string()],
                ));
                true
            } else {

                let type_hit = self
                    .resolve_identifier_with_meaning(node, SymbolFlags::TYPE)
                    .map(|s| self.resolve_alias_base(s));
                if let Some(sym) = type_hit
                    && !sym.flags.intersects(SymbolFlags::VALUE)
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file.clone(),
                        node.loc,
                        crate::diagnostics::messages_generated::
                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                        vec![name.to_string()],
                    ));
                    true
                } else {
                    false
                }
            };
            if reported {
                return;
            }
        }

        let diagnostic = if let Some(class) = self.enclosing_class_stack.last().cloned() {
            let class_name = Self::class_name_text(&class);
            if let Some(is_member_static) = self.class_member_static_by_name(&class, name) {
                if is_member_static {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_STATIC_MEMBER_1_0,
                        vec![name.to_string(), class_name],
                    )
                } else if self.this_container_stack.last() == Some(&ThisContainerKind::InstanceMember)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_INSTANCE_MEMBER_THIS_0,
                        vec![name.to_string()],
                    )
                } else if let Some(suggestion) =
                    self.find_name_suggestion(name, SymbolFlags::VALUE)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else if let Some(suggestion) = self.find_name_suggestion(
                    name,
                    SymbolFlags::VALUE,
                ) {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        CANNOT_FIND_NAME_0,
                        vec![name.to_string()],
                    )
                }
            } else if let Some(suggestion) = self.find_name_suggestion(
                name,
                SymbolFlags::VALUE,
            ) {
                crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                    vec![name.to_string(), suggestion],
                )
            } else {
                crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    CANNOT_FIND_NAME_0,
                    vec![name.to_string()],
                )
            }
        } else if let Some(msg) = Self::cannot_find_name_message_for(name) {
            crate::ast::Diagnostic::new(file, node.loc, *msg, vec![name.to_string()])
        } else if let Some(suggestion) =
            self.find_name_suggestion(name, SymbolFlags::VALUE)
        {
            crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                vec![name.to_string(), suggestion],
            )
        } else {
            crate::ast::Diagnostic::new(
                file,
                node.loc,
                *Self::cannot_find_name_message_for(name).unwrap_or(&CANNOT_FIND_NAME_0),
                vec![name.to_string()],
            )
        };
        self.diagnostics.add(diagnostic);
    }

    fn check_super_before_this(&mut self, body: &Arc<Node>) {
        fn visit(
            c: &mut Checker,
            n: &Arc<Node>,
            super_seen: &mut bool,
        ) {
            if n.kind == SyntaxKind::ThisKeyword {
                if !*super_seen {
                    let file = c.current_file.clone();
                    c.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        n.loc,
                        crate::diagnostics::messages_generated::
                            X_SUPER_MUST_BE_CALLED_BEFORE_ACCESSING_THIS_IN_THE_CONSTRUCTOR_OF_A_DERIVED_CLASS,
                        vec![],
                    ));
                }
                return;
            }

            if n.kind == SyntaxKind::CallExpression
                && let crate::ast::NodeData::CallExpression(call) = &n.data
                && call.expression.kind == SyntaxKind::SuperKeyword
            {
                for arg in call.arguments.iter() {
                    visit(c, arg, super_seen);
                }
                *super_seen = true;
                return;
            }

            if matches!(
                n.kind,
                SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return;
            }

            if matches!(n.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
                return;
            }
            crate::ast::node_data_generated::for_each_child(n, |child| {
                visit(c, child, super_seen);
                false
            });
        }
        let mut super_seen = false;
        visit(self, body, &mut super_seen);
    }

    pub(crate) fn find_name_suggestion(&self, name: &str, meaning: SymbolFlags) -> Option<String> {

        let mut candidates: Vec<&Arc<Symbol>> = Vec::new();
        let symbol_map = self.program.symbol_map();
        fn push_symbol<'a>(
            cands: &mut Vec<&'a Arc<Symbol>>,
            sym: &'a Arc<Symbol>,
            meaning: SymbolFlags,
        ) {
            if sym.flags.intersects(meaning) {
                cands.push(sym);
            }
        }

        if let Some(file) = self.current_file.as_ref() {
            let fid = file.id();
            if let Some(locals) = symbol_map.locals.get(&fid) {
                for sym in locals.entries.values() {
                    push_symbol(&mut candidates, sym, meaning);
                }
            }

            if let Some(sym) = symbol_map.symbols.get(&fid) {
                for sub in sym.members.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
                for sub in sym.exports.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
            }
        }
        for &container_id in self.scope_stack.iter() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                for sym in locals.entries.values() {
                    push_symbol(&mut candidates, sym, meaning);
                }
            }
            if let Some(sym) = symbol_map.symbols.get(&container_id) {
                for sub in sym.members.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
                for sub in sym.exports.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
            }
        }
        for sym in self.globals.entries.values() {
            push_symbol(&mut candidates, sym, meaning);
        }

        let rune_len = name.chars().count();
        let maximum_length_difference = ((rune_len as f64) * 0.34) as usize;
        let maximum_length_difference = maximum_length_difference.max(2);
        let mut best_distance = ((rune_len as f64) * 0.4).floor() + 0.9;
        let mut best: Option<((usize, usize), &String)> = None;
        for sym in candidates {
            let cand: &String = &sym.name;

            if cand.is_empty()
                || cand.starts_with('"')
                || cand.starts_with('\'')
                || cand.starts_with('`')
                || cand.starts_with('\u{FE}')
            {
                continue;
            }
            let cand_len = cand.chars().count();
            if cand_len < 3 && !cand.eq_ignore_ascii_case(name) {
                continue;
            }
            if rune_len.max(cand_len) - rune_len.min(cand_len) > maximum_length_difference {
                continue;
            }
            if cand == name {
                continue;
            }
            let Some(d) = levenshtein_with_max(name, cand, best_distance) else {
                continue;
            };

            let key = self.suggestion_order_key(sym);
            let replace = match &best {
                None => true,
                Some((bkey, _)) => {
                    if d < best_distance {
                        true
                    } else {
                        key < *bkey
                    }
                }
            };
            if d < best_distance {
                best_distance = d;
            }
            if replace {
                best = Some((key, cand));
            }
        }
        best.map(|(_, c)| c.clone())
    }

    fn suggestion_order_key(&self, sym: &Arc<Symbol>) -> (usize, usize) {
        let Some(decl) = sym.declarations.first() else {
            return (usize::MAX, usize::MAX);
        };
        let Some(sf) = self.get_source_file_of_node(decl) else {
            return (usize::MAX, usize::MAX);
        };
        let idx = self
            .files
            .iter()
            .position(|f| f.node.id() == sf.node.id())
            .unwrap_or(usize::MAX);
        (idx, decl.loc.pos())
    }

    fn inside_function_body(node: &Arc<Node>) -> bool {
        let mut anc = node.parent.as_ref();
        while let Some(a) = anc {
            match a.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return true,
                SyntaxKind::ModuleBlock | SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                    return false
                }
                _ => {}
            }
            anc = a.parent.as_ref();
        }
        false
    }

    fn check_class_heritage_members(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::ClassDeclaration(data) = &node.data else {
            return;
        };
        let Some((base_node, _base_sym)) = self.extends_base_of(node) else {
            return;
        };
        let class_name = data
            .name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        let base_name = Self::class_name_text(&base_node);

        if !node.has_syntactic_modifier(ModifierFlags::Abstract) {
            let mut missing: Vec<String> = Vec::new();
            Self::collect_unimplemented_abstract_members(node, &base_node, &mut missing);
            missing.dedup();
            if !missing.is_empty() {
                let file = self.current_file.clone();
                let name_loc = data
                    .name
                    .as_ref()
                    .map(|n| n.loc)
                    .unwrap_or(node.loc);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_loc,
                    crate::diagnostics::messages_generated::
                        NON_ABSTRACT_CLASS_0_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_1_COLON_2,
                    vec![
                        class_name.clone(),
                        base_name.clone(),
                        missing
                            .iter()
                            .map(|m| format!("'{m}'"))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ],
                ));
            }
        }

        for member in data.members.iter() {
            let (name_node, own_type): (&Arc<Node>, Option<Arc<Type>>) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(pd) => {
                    if pd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }
                    let t = if let Some(tn) = &pd.type_node {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        pd.initializer
                            .as_ref()
                            .map(|init| self.get_type_of_node(init))
                    };
                    (&pd.name, t)
                }
                crate::ast::NodeData::GetAccessorDeclaration(gd) => {
                    if gd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }

                    let t = if let Some(tn) = &gd.type_node {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        Self::first_return_expression(gd.body.as_ref())
                            .map(|e| self.get_type_of_node(&e))
                    };
                    (&gd.name, t)
                }
                _ => continue,
            };
            let Some(own_type) = own_type else { continue };
            let prop_name = name_node.text().to_string();
            let Some(base_member) = Self::find_class_member_by_name(&base_node, &prop_name)
            else {
                continue;
            };
            let base_tn = match &base_member.data {
                crate::ast::NodeData::PropertyDeclaration(pd) => pd.type_node.clone(),
                crate::ast::NodeData::GetAccessorDeclaration(gd) => gd.type_node.clone(),
                crate::ast::NodeData::SetAccessorDeclaration(sd) => sd
                    .parameters
                    .iter()
                    .next()
                    .and_then(|p| {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                            pd.type_node.clone()
                        } else {
                            None
                        }
                    }),
                _ => None,
            };
            let Some(base_tn) = base_tn else {
                continue;
            };
            let base_type = self.get_type_from_type_node(&base_tn);
            if !own_type.flags.contains(TypeFlags::Any)
                && !self.is_type_assignable_to(&own_type, &base_type)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_node.loc,
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                    vec![
                        prop_name,
                        class_name.clone(),
                        base_name.clone(),
                    ],
                ));
            }
        }
    }

    fn class_members_of(class: &Arc<Node>) -> &Arc<NodeList> {
        match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            crate::ast::NodeData::ClassExpression(d) => &d.members,
            _ => {
                static EMPTY: std::sync::OnceLock<Arc<NodeList>> = std::sync::OnceLock::new();
                EMPTY.get_or_init(|| Arc::new(NodeList::default()))
            }
        }
    }

    fn find_class_member_by_name(class: &Arc<Node>, name: &str) -> Option<Arc<Node>> {
        Self::class_members_of(class)
            .iter()
            .find(|m| {
                let n = match &m.data {
                    crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                    crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                    crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                    crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                    _ => return false,
                };
                n.kind == SyntaxKind::Identifier && n.text() == name
            })
            .cloned()
    }

    fn collect_unimplemented_abstract_members(
        class: &Arc<Node>,
        base: &Arc<Node>,
        out: &mut Vec<String>,
    ) {
        for member in Self::class_members_of(base).iter() {
            let (name_node, is_abstract_member) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind != SyntaxKind::Identifier {
                continue;
            }
            let name = name_node.text();
            if is_abstract_member {

                if !Self::chain_implements(class, name) {
                    out.push(name.to_string());
                }
            } else if out.iter().any(|m| m == name) {

                out.retain(|m| m != name);
            }
        }
    }

    fn first_return_expression(body: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        fn walk(n: &Arc<Node>) -> Option<Arc<Node>> {
            if let crate::ast::NodeData::ReturnStatement(d) = &n.data
                && let Some(e) = &d.expression
            {
                return Some(Arc::clone(e));
            }
            let mut found: Option<Arc<Node>> = None;
            crate::ast::node_data_generated::for_each_child(n, |child| {
                if found.is_none() {
                    found = walk(child);
                }
                found.is_some()
            });
            found
        }
        body.and_then(walk)
    }

    fn chain_implements(class: &Arc<Node>, name: &str) -> bool {
        for member in Self::class_members_of(class).iter() {
            let (name_node, is_abstract) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind == SyntaxKind::Identifier
                && name_node.text() == name
                && !is_abstract
            {
                return true;
            }
        }

        false
    }

    fn assignments_to_name(
        body: &Arc<Node>,
        name: &str,
    ) -> Vec<(crate::core::text::TextRange, Arc<Node>)> {
        let mut found = Vec::new();
        fn walk(n: &Arc<Node>, name: &str, found: &mut Vec<(crate::core::text::TextRange, Arc<Node>)>) {
            if let crate::ast::NodeData::BinaryExpression(data) = &n.data
                && data.operator_token.kind == SyntaxKind::EqualsToken
                && data.left.kind == SyntaxKind::Identifier
                && data.left.text() == name
            {
                found.push((data.left.loc, Arc::clone(&data.right)));
            }
            crate::ast::node_data_generated::for_each_child(n, |child| {
                walk(child, name, found);
                false
            });
        }
        walk(body, name, &mut found);
        found
    }

    pub fn resolve_qualified_symbol(&mut self, name: &Arc<Node>) -> Option<Arc<Symbol>> {
        match self.resolve_qualified_symbol_traced(name) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn resolve_qualified_symbol_traced(
        &mut self,
        name: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        match &name.data {
            crate::ast::NodeData::Identifier(_) => match self.resolve_identifier(name) {
                Some(s) => Ok(s),
                None => Err((Arc::clone(name), String::new(), String::new())),
            },
            crate::ast::NodeData::QualifiedName(data) => {
                self.resolve_qualified_tail(&data.left, &data.right)
            }

            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let mut base = &pa.expression;
                while let crate::ast::NodeData::ParenthesizedExpression(p) = &base.data {
                    base = &p.expression;
                }
                if matches!(
                    base.kind,
                    SyntaxKind::Identifier
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::PropertyAccessExpression
                ) {
                    self.resolve_qualified_tail(base, &pa.name)
                } else {
                    Err((Arc::clone(name), String::new(), String::new()))
                }
            }
            _ => Err((Arc::clone(name), String::new(), String::new())),
        }
    }

    fn resolve_qualified_tail(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        {
            let mut symbol = self.resolve_qualified_symbol_traced(left)?;
            let path_so_far = qualified_name_text(left);
            symbol = self.resolve_alias_base(symbol);

            if symbol.flags == SymbolFlags::Alias
                && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
            {
                symbol = module_sym;
            }

            let text = right.text();
            let mut next = symbol
                .exports
                .get(text)
                .or_else(|| symbol.members.get(text))
                .cloned()
                .or_else(|| self.ambient_namespace_local(&symbol, text))

                .or_else(|| self.object_literal_export_member(&symbol, text));

                if next.is_none()
                    && let Some(ea_sym) = symbol.exports.get("export=")
                    && let Some(decl) = ea_sym
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                    && let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {

                    let scope = symbol
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(scope) = scope {
                        self.push_scope(&scope);
                        let target = self.resolve_identifier(&ea.expression);
                        self.pop_scope();
                        if let Some(target) = target
                            && target.flags.contains(SymbolFlags::ValueModule)
                        {
                            next = target
                                .exports
                                .get(text)
                                .or_else(|| target.members.get(text))
                                .cloned()
                                .or_else(|| self.ambient_namespace_local(&target, text));
                        }
                    }
                }

                let base_is_unresolved_require_alias = symbol.flags == SymbolFlags::Alias
                    && symbol
                        .declarations
                        .iter()
                        .any(|d| {
                            if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                                && let crate::ast::NodeData::ExternalModuleReference(ext) =
                                    &ied.module_reference.data
                                && ext.expression.kind == SyntaxKind::StringLiteral
                            {
                                self.resolve_module_file_symbol(&ext.expression.text()).is_none()
                            } else {
                                false
                            }
                        });
                if base_is_unresolved_require_alias {
                    return Ok(symbol);
                }
                match next {
                    Some(next) => {

                        let resolved = if next.flags.intersects(SymbolFlags::Alias) {
                            let scope = symbol
                                .declarations
                                .iter()
                                .find(|d| {
                                    d.kind == SyntaxKind::ModuleDeclaration
                                        || d.kind == SyntaxKind::SourceFile
                                })
                                .cloned();
                            if let Some(ref scope) = scope {
                                self.push_scope(scope);
                            }
                            let base = self.resolve_alias_base(Arc::clone(&next));
                            if scope.is_some() {
                                self.pop_scope();
                            }
                            base
                        } else {
                            match self.follow_alias(&next) {
                                Some(f) => f,
                                None => next,
                            }
                        };
                        Ok(resolved)
                    }
                    None => {
                        let _ = path_so_far;
                        Err((
                            Arc::clone(right),
                            Self::namespace_full_path(&symbol),
                            text.to_string(),
                        ))
                    }
                }
            }
    }

    fn ambient_ancestor(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.as_ref();
        while let Some(a) = cur {
            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }
            cur = a.parent.as_ref();
        }
        false
    }

    pub(crate) fn ambient_namespace_locals_visible(&self, ns: &Arc<Symbol>) -> bool {
        if std::env::var_os("TSOX_NO_AMBIENT").is_some() {
            return false;
        }
        ns.declarations.iter().any(|d| {
            d.kind == SyntaxKind::ModuleDeclaration
                && (d.has_syntactic_modifier(ModifierFlags::Ambient)
                    || self.ambient_ancestor(d)
                    || self
                        .get_source_file_of_node(d)
                        .is_some_and(|f| f.is_declaration_file))
                && !crate::binder::Binder::has_export_declarations(d)
        })
    }

    pub(crate) fn ambient_namespace_local(&self, ns: &Arc<Symbol>, name: &str) -> Option<Arc<Symbol>> {
        if !self.ambient_namespace_locals_visible(ns) {
            return None;
        }
        ns.declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .find_map(|d| {
                self.program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
                    .cloned()
            })
    }

    pub(crate) fn resolve_alias_base(&mut self, symbol: Arc<Symbol>) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return symbol;
        }

        if symbol
            .declarations
            .iter()
            .any(|d| matches!(d.kind, SyntaxKind::NamespaceImport | SyntaxKind::NamespaceExport))
            && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
        {
            return module_sym;
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            if let crate::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {

                if let crate::ast::NodeData::ExternalModuleReference(ext) =
                    &data.module_reference.data
                    && ext.expression.kind == SyntaxKind::StringLiteral
                    && let Some(module_sym) =
                        self.resolve_module_file_symbol(&ext.expression.text())
                {
                    if let Some(export_eq) =
                        module_sym.exports.get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                    {

                        let entity_decl = export_eq
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ExportAssignment)
                            .cloned();
                        let scope_decl = module_sym
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                            .cloned();
                        if let (Some(export_decl), Some(scope)) = (entity_decl, scope_decl)
                            && let crate::ast::NodeData::ExportAssignment(ea) = &export_decl.data
                            && ea.is_export_equals
                            && matches!(
                                ea.expression.kind,
                                SyntaxKind::Identifier | SyntaxKind::QualifiedName
                            )
                        {
                            self.push_scope(&scope);
                            let target = self.resolve_qualified_symbol(&ea.expression);
                            self.pop_scope();
                            if let Some(target) = target {
                                return target;
                            }
                        }
                    }
                    return module_sym;
                }

                if matches!(
                    data.module_reference.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    let mut current = Arc::clone(&symbol);
                    for _ in 0..4 {
                        let next = current
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                            .and_then(|d| {
                                if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                                    && matches!(
                                        ied.module_reference.kind,
                                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                    )
                                {
                                    Some(self.resolve_qualified_symbol(&ied.module_reference))
                                } else {
                                    None
                                }
                            })
                            .flatten();
                        match next {
                            Some(n) => current = n,
                            None => break,
                        }
                        if !current.flags.intersects(SymbolFlags::Alias) {
                            return current;
                        }
                    }
                    return current;
                }
            }
        }
        symbol
    }

    pub(crate) fn resolve_module_file_symbol(&self, specifier: &str) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {

            for file in self.program.source_files() {

                if file.external_module_indicator.is_some() {
                    continue;
                }
                if let crate::ast::NodeData::SourceFile(sf) = &file.node.data {
                    for stmt in sf.statements.iter() {
                        if let crate::ast::NodeData::ModuleDeclaration(md) = &stmt.data
                            && md.name.kind == SyntaxKind::StringLiteral
                            && md.name.text().trim_matches(['"', '\'']) == specifier
                        {
                            return self.program.symbol_map().symbol_of(stmt).cloned();
                        }
                    }
                }
            }
            return None;
        }
        let current = self.current_file.as_ref()?;
        let dir = match current.file_name.rfind('/') {
            Some(i) => &current.file_name[..i],
            None => "",
        };
        self.resolve_module_file_symbol_in(dir, specifier)
    }

    fn resolve_module_file_symbol_in(
        &self,
        dir: &str,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        let stem = specifier.strip_prefix("./").unwrap_or(specifier);

        let stem = stem
            .strip_suffix(".js")
            .or_else(|| stem.strip_suffix(".jsx"))
            .unwrap_or(stem);
        let symbol_map = self.program.symbol_map();
        for cand in [
            format!("{dir}/{stem}.ts"),
            format!("{dir}/{stem}.tsx"),
            format!("{dir}/{stem}.d.ts"),
            format!("{dir}/{stem}/index.ts"),
            format!("{dir}/{stem}/index.d.ts"),
        ] {
            if let Some(sf) = self
                .program
                .source_files()
                .iter()
                .find(|f| f.file_name == cand)
            {
                if let Some(sym) = symbol_map.symbol_of(&sf.node) {
                    return Some(Arc::clone(sym));
                }
            }
        }
        None
    }

    fn check_module_format_mismatch(&mut self, node: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;
        if !matches!(self.module_kind, ModuleKind::Node16 | ModuleKind::Node18) {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        if file.file_name.starts_with("bundled://") {
            return;
        }
        let (spec_node, attrs, is_import_equals): (Arc<Node>, Option<Arc<Node>>, bool) =
            match &node.data {
                NodeData::ImportDeclaration(d) => (
                    Arc::clone(&d.module_specifier),
                    d.attributes.clone(),
                    false,
                ),
                NodeData::ExportDeclaration(d) => match &d.module_specifier {
                    Some(spec) => (Arc::clone(spec), d.attributes.clone(), false),
                    None => return,
                },
                NodeData::ImportEqualsDeclaration(d) => {
                    match &d.module_reference.data {
                        NodeData::ExternalModuleReference(ext) => {
                            (Arc::clone(&ext.expression), None, true)
                        }
                        _ => return,
                    }
                }
                _ => return,
            };

        if let Some(attrs) = &attrs
            && self.get_resolution_mode_override(attrs, false).is_some()
        {
            return;
        }
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        if spec_text.is_empty() {
            return;
        }
        let read = |p: &str| self.program.read_file(p);
        let target_path = match self.program.resolve_external_module_path(
            &spec_text,
            &file.file_name,
            ModuleKind::None,
        ) {
            Some(p) => p,
            None => return,
        };

        if !module_format_is_esm_for_require_check(&target_path, &read) {
            return;
        }
        if is_import_equals {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                crate::diagnostics::messages_generated::
                    MODULE_0_CANNOT_BE_IMPORTED_USING_THIS_CONSTRUCT_THE_SPECIFIER_ONLY_RESOLVES_TO_AN_ES_MODULE_WHICH_CANNOT_BE_IMPORTED_WITH_REQUIRE_USE_AN_ECMASCRIPT_IMPORT_INSTEAD,
                vec![spec_text.clone()],
            ));
        } else if importer_is_cjs_for_require_check(&file.file_name, &read) {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                crate::diagnostics::messages_generated::
                    THE_CURRENT_FILE_IS_A_COMMONJS_MODULE_WHOSE_IMPORTS_WILL_PRODUCE_REQUIRE_CALLS_HOWEVER_THE_REFERENCED_FILE_IS_AN_ECMASCRIPT_MODULE_AND_CANNOT_BE_IMPORTED_WITH_REQUIRE_CONSIDER_WRITING_A_DYNAMIC_IMPORT_0_CALL_INSTEAD,
                vec![spec_text],
            ));
        }
    }

    fn check_declaration_nameability(&mut self, stmt: &Arc<Node>) {
        if !self.program.options().declaration.is_true() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        if file.file_name.starts_with("bundled://") || file.is_declaration_file {
            return;
        }

        if file.file_name.contains("/node_modules/") {
            return;
        }
        let crate::ast::NodeData::VariableStatement(data) = &stmt.data else {
            return;
        };
        let has_export = stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
        if !has_export {
            return;
        }

        let mut imported_files: Vec<String> = Vec::new();
        let mut spec_names: Vec<String> = Vec::new();
        let NodeData::SourceFile(sfd) = &file.node.data else {
            return;
        };
        for st in sfd.statements.iter() {
            let spec = match &st.data {
                NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
                NodeData::ExportDeclaration(d) => match &d.module_specifier {
                    Some(s) => s.text().to_string(),
                    None => continue,
                },
                _ => continue,
            };
            let text = spec.trim_matches(['"', '\'', '`']).to_string();
            if text.is_empty() {
                continue;
            }
            spec_names.push(text.clone());
            if let Some(p) = self.program.resolve_external_module_path(
                &text,
                &file.file_name,
                crate::core::compiler_options::ModuleKind::None,
            ) {
                imported_files.push(p);
            }
        }
        let crate::ast::NodeData::VariableDeclarationList(list) = &data.declaration_list.data
        else {
            return;
        };
        for d in list.declarations.iter() {
            let crate::ast::NodeData::VariableDeclaration(vd) = &d.data else {
                continue;
            };

            if let Some(init) = &vd.initializer {
                let mut import_expr = Some(Arc::clone(init));
                if let Some(inner) = import_expr.take() {
                    let unwrapped = match &inner.data {
                        NodeData::AwaitExpression(a) => Some(Arc::clone(&a.expression)),
                        _ => Some(inner),
                    };
                    if let Some(call) = unwrapped
                        && call.kind == SyntaxKind::CallExpression
                        && let Some(spec) = self.spec_of_dynamic_import_call(&call)
                        && let Some(path) = self.program.resolve_external_module_path(
                            &spec,
                            &file.file_name,
                            crate::core::compiler_options::ModuleKind::ESNext,
                        )
                        && !imported_files.contains(&path)
                    {
                        imported_files.push(path);
                    }
                }
            }

            if vd.type_node.is_some() {
                continue;
            }
            let Some(sym) = self.program.symbol_map().symbol_of(d).cloned() else {
                continue;
            };
            let var_name = vd.name.text().to_string();
            let t = self.get_type_of_symbol(&sym);
            let Some(target) = t.symbol.clone() else {
                continue;
            };
            let Some(target_file) = target
                .declarations
                .first()
                .and_then(|dn| self.get_source_file_of_node(dn))
            else {
                continue;
            };
            if target_file.file_name == file.file_name
                || !target_file.file_name.contains("/node_modules/")
                || imported_files.contains(&target_file.file_name)
            {
                continue;
            }

            if self.symbol_in_ambient_module_named(&target, &spec_names) {
                continue;
            }

            let spec = relative_emit_specifier(&file.file_name, &target_file.file_name);
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file.clone()),
                vd.name.loc,
                crate::diagnostics::messages_generated::
                    THE_INFERRED_TYPE_OF_0_CANNOT_BE_NAMED_WITHOUT_A_REFERENCE_TO_2_FROM_1_THIS_IS_LIKELY_NOT_PORTABLE_A_TYPE_ANNOTATION_IS_NECESSARY,
                vec![var_name, spec, target.name.clone()],
            ));
        }
    }

    fn symbol_in_ambient_module_named(
        &self,
        symbol: &Arc<Symbol>,
        imported_specs: &[String],
    ) -> bool {
        if imported_specs.is_empty() {
            return false;
        }
        for decl in &symbol.declarations {
            let mut cur = decl.parent.as_ref();
            while let Some(n) = cur {
                if let NodeData::ModuleDeclaration(md) = &n.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    let module_name = md.name.text().trim_matches(['"', '\'']).to_string();
                    return imported_specs.iter().any(|s| *s == module_name);
                }
                if n.kind == SyntaxKind::SourceFile {
                    break;
                }
                cur = n.parent.as_ref();
            }
        }
        false
    }

    fn check_module_export_names(&mut self, node: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;

        let mut names: Vec<(Arc<Node>, bool)> = Vec::new();
        match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else { return };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                let Some(named) = &ic.named_bindings else { return };
                let NodeData::NamedImports(ni) = &named.data else {
                    return;
                };
                for el in ni.elements.iter() {
                    if let NodeData::ImportSpecifier(spec) = &el.data {
                        if let Some(pn) = &spec.property_name {
                            names.push((Arc::clone(pn), true));
                        }
                    }
                }
            }
            NodeData::ExportDeclaration(d) => {
                let has_module_specifier = d.module_specifier.is_some();
                match &d.export_clause {
                    Some(clause) => match &clause.data {
                        NodeData::NamedExports(ne) => {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    if let Some(pn) = &spec.property_name {
                                        names.push((Arc::clone(pn), has_module_specifier));
                                    }
                                    names.push((Arc::clone(&spec.name), true));
                                }
                            }
                        }
                        NodeData::NamespaceExport(ne) => {
                            names.push((Arc::clone(&ne.name), true));
                        }
                        _ => {}
                    },
                    None => {}
                }
            }
            _ => return,
        }
        if names.is_empty() {
            return;
        }
        let declaration_file = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        for (name, string_allowed) in names {
            if name.kind != SyntaxKind::StringLiteral {
                continue;
            }
            if !string_allowed {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::IDENTIFIER_EXPECTED,
                    vec![],
                ));
            } else if matches!(self.module_kind, ModuleKind::ES2015 | ModuleKind::ES2020)
                && !declaration_file
            {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::
                        STRING_LITERAL_IMPORT_AND_EXPORT_NAMES_ARE_NOT_SUPPORTED_WHEN_THE_MODULE_FLAG_IS_SET_TO_ES2015_OR_ES2020,
                    vec![],
                ));
            }
        }
    }

    fn check_module_specifier_members(&mut self, node: &Arc<Node>) {
        use crate::ast::NodeData;

        let (spec_node, attrs, exclusively_type_only, elements): (
            Arc<Node>,
            Option<Arc<Node>>,
            bool,
            Arc<crate::ast::NodeList>,
        ) = match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else { return };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                let Some(named) = &ic.named_bindings else { return };
                let NodeData::NamedImports(ni) = &named.data else {
                    return;
                };
                (
                    Arc::clone(&d.module_specifier),
                    d.attributes.clone(),
                    ic.phase_modifier == Some(SyntaxKind::TypeKeyword),
                    Arc::clone(&ni.elements),
                )
            }
            NodeData::ExportDeclaration(d) => {
                let Some(spec) = &d.module_specifier else {
                    return;
                };
                let Some(clause) = &d.export_clause else {
                    return;
                };
                let NodeData::NamedExports(ne) = &clause.data else {
                    return;
                };
                (
                    Arc::clone(spec),
                    d.attributes.clone(),
                    d.is_type_only,
                    Arc::clone(&ne.elements),
                )
            }
            _ => return,
        };
        if elements.is_empty() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();

        let mode = match (&attrs, exclusively_type_only) {
            (Some(attrs), true) => self
                .get_resolution_mode_override(attrs, false)
                .unwrap_or(crate::core::compiler_options::ModuleKind::None),
            _ => crate::core::compiler_options::ModuleKind::None,
        };

        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(&spec_text, &file.file_name, mode)
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        let module_symbol = if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        };
        let Some(module_symbol) = module_symbol else {
            return;
        };

        let shorthand_ambient = module_symbol.value_declaration.as_ref().is_some_and(|d| {
            matches!(&d.data, NodeData::ModuleDeclaration(md) if md.body.is_none())
        });
        if shorthand_ambient {
            return;
        }
        for element in elements.iter() {
            let (property_name, name) = match &element.data {
                NodeData::ImportSpecifier(d) => (d.property_name.clone(), d.name.clone()),
                NodeData::ExportSpecifier(d) => (d.property_name.clone(), d.name.clone()),
                _ => continue,
            };
            let member_name = property_name
                .as_ref()
                .unwrap_or(&name)
                .text()
                .trim_matches(['"', '\'', '`'])
                .to_string();
            let error_node = property_name.clone().unwrap_or_else(|| Arc::clone(&name));
            match self.module_member_lookup(&module_symbol, &member_name) {
                ModuleMemberLookup::Found => {}

                ModuleMemberLookup::LocalNotExported => {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        crate::diagnostics::messages_generated::
                            MODULE_0_DECLARES_1_LOCALLY_BUT_IT_IS_NOT_EXPORTED,
                        vec![format!("\"{spec_text}\""), member_name],
                    ));
                }
                ModuleMemberLookup::Missing => {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        crate::diagnostics::messages_generated::MODULE_0_HAS_NO_EXPORTED_MEMBER_1,

                        vec![format!("\"{spec_text}\""), member_name],
                    ));
                }
            }
        }
    }

    fn module_member_lookup(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
    ) -> ModuleMemberLookup {
        use ModuleMemberLookup as M;

        if let Some(export_equals) = module_symbol.exports.get("export=") {
            let target = self.resolve_export_equals_target(export_equals);
            if std::env::var_os("TSOX_DEBUG_MODULE").is_some() {
                eprintln!(
                    "[mod-lookup] export= chain: module={:?} target={:?} exports={} members={}",
                    module_symbol.name,
                    target.name,
                    target.exports.len(),
                    target.members.len()
                );
            }
            if self.module_target_has_member(&target, name)
                || module_symbol.exports.get(name).is_some()
            {
                return M::Found;
            }

            if self.module_star_chain_exports(module_symbol, name)
                || (name == "default"
                    && self.module_can_have_synthetic_default(module_symbol))
            {
                return M::Found;
            }
            return M::Missing;
        }
        if module_symbol.exports.get(name).is_some() {
            return M::Found;
        }
        if std::env::var_os("TSOX_DEBUG_MODULE").is_some() {
            eprintln!(
                "[mod-lookup] plain: name={name} exports={:?} members_with={:?} decls={:?}",
                module_symbol.exports.iter().take(12).map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                module_symbol
                    .members
                    .get(name)
                    .map(|s| (s.export_symbol.is_some(), s.flags)),
                module_symbol.declarations.iter().map(|d| d.kind).collect::<Vec<_>>()
            );
        }

        if self.module_has_export_clause(module_symbol, name) {
            return M::Found;
        }

        if name == "default" && self.module_has_syntactic_default(module_symbol) {
            return M::Found;
        }
        if let Some(sym) = module_symbol.members.get(name) {
            return if sym.export_symbol.is_some() {
                M::Found
            } else {
                M::LocalNotExported
            };
        }

        if let Some(file_node) = module_symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
        {
            if let Some(locals) = self.program.symbol_map().locals.get(&file_node.id())
                && let Some(sym) = locals.get(name)
            {
                return if sym.export_symbol.is_some() {
                    M::Found
                } else {
                    M::LocalNotExported
                };
            }
        }

        if self.module_is_ambient_export_context(module_symbol)
            && self.module_ambient_locals_contain(module_symbol, name)
        {
            return M::Found;
        }

        if name != "default" && self.module_star_chain_exports(module_symbol, name) {
            return M::Found;
        }

        if name == "default" && self.module_can_have_synthetic_default(module_symbol) {
            return M::Found;
        }
        M::Missing
    }

    pub(crate) fn for_each_module_statement(
        &self,
        module_symbol: &Arc<Symbol>,
        mut f: impl FnMut(&Arc<Node>) -> bool,
    ) {
        use crate::ast::NodeData;
        for decl in &module_symbol.declarations {
            let statements: Option<&Arc<crate::ast::NodeList>> = match &decl.data {
                NodeData::SourceFile(sf) => Some(&sf.statements),
                NodeData::ModuleDeclaration(md) => match &md.body {
                    Some(body) => match &body.data {
                        NodeData::ModuleBlock(b) => Some(&b.statements),
                        _ => None,
                    },
                    None => None,
                },
                _ => None,
            };
            if let Some(list) = statements {
                for s in list.iter() {
                    if f(s) {
                        return;
                    }
                }
            }
        }
    }

    fn module_has_export_clause(&self, module_symbol: &Arc<Symbol>, name: &str) -> bool {
        use crate::ast::NodeData;
        let mut found = false;
        self.for_each_module_statement(module_symbol, |stmt| {
            if let NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        found = true;
                        return true;
                    }
                }
            }
            false
        });
        found
    }

    fn module_has_syntactic_default(&self, module_symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData;
        let mut found = false;
        self.for_each_module_statement(module_symbol, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(d) if !d.is_export_equals => found = true,
                _ => {
                    if stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Default) {
                        found = true;
                    }
                }
            }
            found
        });
        found
    }

    fn module_is_ambient_export_context(&self, module_symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData;
        let mut is_ambient = false;
        let mut has_export_declaration = false;
        for decl in &module_symbol.declarations {
            let ambient = match &decl.data {
                NodeData::ModuleDeclaration(_) => {
                    decl.has_syntactic_modifier(crate::ast::ModifierFlags::Ambient)
                        || self
                            .get_source_file_of_node(decl)
                            .is_some_and(|f| f.is_declaration_file)
                }
                NodeData::SourceFile(_) => self
                    .get_source_file_of_node(decl)
                    .is_some_and(|f| f.is_declaration_file),
                _ => false,
            };
            is_ambient |= ambient;
        }
        if !is_ambient {
            return false;
        }
        self.for_each_module_statement(module_symbol, |stmt| match &stmt.data {
            NodeData::ExportDeclaration(_) => {
                has_export_declaration = true;
                true
            }
            NodeData::ExportAssignment(_) => {
                has_export_declaration = true;
                true
            }
            _ => false,
        });
        !has_export_declaration
    }

    fn module_ambient_locals_contain(&self, module_symbol: &Arc<Symbol>, name: &str) -> bool {

        for decl in &module_symbol.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals.get(name).is_some()
            {
                return true;
            }
        }
        false
    }

    fn module_star_chain_exports(&mut self, module_symbol: &Arc<Symbol>, name: &str) -> bool {
        if name == "default" {
            return false;
        }
        let stars = self.module_star_specs(module_symbol);
        let mut visited: Vec<*const Symbol> = vec![Arc::as_ptr(module_symbol)];
        for (spec, file) in &stars {
            if let Some(target) = self.resolve_module_symbol_from(spec, file)
                && self.star_target_exports(&target, name, &mut visited, 0)
            {
                return true;
            }
        }
        false
    }

    fn module_star_specs(
        &self,
        module_symbol: &Arc<Symbol>,
    ) -> Vec<(Arc<Node>, Arc<crate::ast::SourceFile>)> {
        use crate::ast::NodeData;
        let mut stars = Vec::new();
        self.for_each_module_statement(module_symbol, |stmt| {
            if let NodeData::ExportDeclaration(d) = &stmt.data
                && d.export_clause.is_none()
                && let Some(spec) = &d.module_specifier
                && let Some(file) = self.get_source_file_of_node(stmt)
            {
                stars.push((Arc::clone(spec), file));
            }
            false
        });
        stars
    }

    fn star_target_exports(
        &mut self,
        target: &Arc<Symbol>,
        name: &str,
        visited: &mut Vec<*const Symbol>,
        depth: usize,
    ) -> bool {
        if depth >= 8 || visited.contains(&Arc::as_ptr(target)) {
            return false;
        }
        visited.push(Arc::as_ptr(target));

        let face = match target.exports.get("export=") {
            Some(ee) => self.resolve_export_equals_target(ee),
            None => Arc::clone(target),
        };
        if face.exports.get(name).is_some()
            || self.module_has_export_clause(&face, name)
            || face
                .members
                .get(name)
                .is_some_and(|s| s.export_symbol.is_some())
            || (self.module_is_ambient_export_context(&face)
                && self.module_ambient_locals_contain(&face, name))
        {
            return true;
        }
        let stars = self.module_star_specs(&face);
        for (spec, file) in &stars {
            if let Some(next) = self.resolve_module_symbol_from(spec, file)
                && self.star_target_exports(&next, name, visited, depth + 1)
            {
                return true;
            }
        }
        false
    }

    fn resolve_module_symbol_from(
        &mut self,
        spec_node: &Arc<Node>,
        file: &Arc<crate::ast::SourceFile>,
    ) -> Option<Arc<Symbol>> {
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(
                    &spec_text,
                    &file.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                )
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        }
    }

    fn resolve_export_equals_target(&mut self, export_equals: &Arc<Symbol>) -> Arc<Symbol> {
        let mut target = self.resolve_alias_base(Arc::clone(export_equals));
        for decl in export_equals.declarations.clone() {
            if let crate::ast::NodeData::ExportAssignment(d) = &decl.data
                && matches!(
                    d.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
            {
                if let Some(t) = self.with_declaring_file_context(&decl, |c| {
                    c.resolve_qualified_symbol(&d.expression)
                }) {

                    target = if t.flags.intersects(SymbolFlags::Alias) {
                        self.resolve_alias_base(t)
                    } else {
                        t
                    };
                }
                break;
            }
        }
        target
    }

    fn module_target_has_member(&self, target: &Arc<Symbol>, name: &str) -> bool {
        use crate::ast::NodeData;
        if target.exports.get(name).is_some() || target.members.get(name).is_some() {
            return true;
        }

        let mut has_export_declaration = false;
        let mut ambient = false;
        let mut locals_hit = false;
        for decl in &target.declarations {
            if decl.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            if decl.has_syntactic_modifier(crate::ast::ModifierFlags::Ambient)
                || self
                    .get_source_file_of_node(decl)
                    .is_some_and(|f| f.is_declaration_file)
            {
                ambient = true;
            }
            let body = match &decl.data {
                NodeData::ModuleDeclaration(md) => md.body.clone(),
                _ => None,
            };
            if let Some(body) = body
                && let NodeData::ModuleBlock(b) = &body.data
            {
                for s in b.statements.iter() {
                    match &s.data {
                        NodeData::ExportDeclaration(_) | NodeData::ExportAssignment(_) => {
                            has_export_declaration = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if self
                .program
                .symbol_map()
                .locals
                .get(&decl.id())
                .is_some_and(|l| l.get(name).is_some())
            {
                locals_hit = true;
            }
        }
        ambient && !has_export_declaration && locals_hit
    }

    fn module_can_have_synthetic_default(&mut self, module_symbol: &Arc<Symbol>) -> bool {
        if self.module_has_syntactic_default(module_symbol) {
            return false;
        }
        if module_symbol.exports.get("__esModule").is_some() {
            return false;
        }
        let is_ambient_or_declaration = module_symbol.declarations.iter().any(|d| {
            match &d.data {
                crate::ast::NodeData::ModuleDeclaration(_) => true,
                crate::ast::NodeData::SourceFile(_) => self
                    .get_source_file_of_node(d)
                    .is_some_and(|f| f.is_declaration_file),
                _ => false,
            }
        });
        if is_ambient_or_declaration {
            return true;
        }
        module_symbol.exports.get("export=").is_some()
    }

    fn declaring_dir_of(&self, node: &Arc<Node>) -> Option<String> {
        self.get_source_file_of_node(node)
            .or_else(|| self.current_file.clone())
            .map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            })
    }

    fn resolve_import_alias_module(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::NamespaceImport
                        | SyntaxKind::ImportSpecifier
                        | SyntaxKind::NamespaceExport
                )
            })?
            .clone();

        let mut cur = decl;
        for _ in 0..4 {
            let Some(parent) = cur.parent.clone() else {
                return None;
            };
            if parent.kind == SyntaxKind::ExportDeclaration {

                if let crate::ast::NodeData::ExportDeclaration(d) = &parent.data {
                    let Some(specifier) = &d.module_specifier else {
                        return None;
                    };
                    let spec = specifier.text();
                    if !spec.starts_with('.') {
                        return self.resolve_module_file_symbol(&spec);
                    }
                    let dir = self.declaring_dir_of(&parent)?;
                    return self.resolve_module_file_symbol_in(&dir, &spec);
                }
                return None;
            }
            if parent.kind == SyntaxKind::ImportDeclaration {
                if let crate::ast::NodeData::ImportDeclaration(d) = &parent.data {
                    let spec = d.module_specifier.text();
                    if !spec.starts_with('.') {
                        return self.resolve_module_file_symbol(&spec);
                    }

                    let dir = self
                        .get_source_file_of_node(&parent)
                        .map(|f| {
                            match f.file_name.rfind('/') {
                                Some(i) => f.file_name[..i].to_string(),
                                None => String::new(),
                            }
                        })
                        .or_else(|| {
                            self.current_file.as_ref().map(|f| {
                                match f.file_name.rfind('/') {
                                    Some(i) => f.file_name[..i].to_string(),
                                    None => String::new(),
                                }
                            })
                        })?;
                    return self.resolve_module_file_symbol_in(&dir, &spec);
                }
                return None;
            }
            cur = parent;
        }
        None
    }

    fn class_name_text(class: &Arc<Node>) -> String {
        match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => {
                d.name.as_ref().map(|n| n.text().to_string()).unwrap_or_default()
            }
            crate::ast::NodeData::ClassExpression(d) => {
                d.name.as_ref().map(|n| n.text().to_string()).unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    fn class_member_static_by_name(&self, class: &Arc<Node>, name: &str) -> Option<bool> {
        let members = match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            crate::ast::NodeData::ClassExpression(d) => &d.members,
            _ => return None,
        };
        for member in members.iter() {
            let member_name = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                _ => continue,
            };
            if member_name.kind == SyntaxKind::Identifier && member_name.text() == name {
                return Some(member.has_syntactic_modifier(ModifierFlags::Static));
            }
        }
        None
    }

    fn check_duplicate_function_implementations(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::FunctionDeclaration(data) = &node.data else {
            return;
        };
        let Some(name) = &data.name else { return };
        if name.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let stmts = match &parent.data {
            crate::ast::NodeData::SourceFile(sf) => Some(&sf.statements),
            crate::ast::NodeData::ModuleBlock(mb) => Some(&mb.statements),
            _ => None,
        };
        let Some(stmts) = stmts else {
            return;
        };
        let is_ambient = node.flags.contains(NodeFlags::Ambient)
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        let fns: Vec<&Arc<Node>> = stmts
            .iter()
            .filter(|s| {
                s.kind == SyntaxKind::FunctionDeclaration
                    && matches!(&s.data, crate::ast::NodeData::FunctionDeclaration(d) if d
                        .name
                        .as_ref()
                        .is_some_and(|n| n.text() == name.text()))
            })
            .collect();

        if fns.first().is_none_or(|first| !Arc::ptr_eq(first, node)) {
            return;
        }
        let bodied = fns
            .iter()
            .filter(|f| {
                matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            })
            .count();
        let file = self.current_file.clone();
        if bodied >= 2 && !is_ambient {
            for f in &fns {
                if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data
                    && let Some(fname) = &d.name
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file.clone(),
                        fname.loc,
                        crate::diagnostics::messages_generated::DUPLICATE_FUNCTION_IMPLEMENTATION,
                        vec![],
                    ));
                }
            }
        }

        let is_ambient_decl = |f: &Arc<Node>| {
            f.has_syntactic_modifier(ModifierFlags::Ambient)
                || f.flags.contains(NodeFlags::Ambient)
        };
        let canonical = fns
            .iter()
            .find(|f| {
                matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            })
            .or_else(|| fns.first());
        if let Some(canonical) = canonical {
            let canonical_ambient = is_ambient_decl(canonical);
            for f in &fns {
                let has_body =
                    matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some());
                if !has_body && is_ambient_decl(f) != canonical_ambient {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data
                        && let Some(fname) = &d.name
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file.clone(),
                            fname.loc,
                            crate::diagnostics::messages_generated::
                                OVERLOAD_SIGNATURES_MUST_ALL_BE_AMBIENT_OR_NON_AMBIENT,
                            vec![],
                        ));
                    }
                }
            }
        }
    }

    fn check_overload_implementation_follows(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::FunctionDeclaration(data) = &node.data else {
            return;
        };
        if data.body.is_some() {
            return;
        }
        let Some(name) = &data.name else { return };
        if name.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let stmts = match &parent.data {
            crate::ast::NodeData::SourceFile(sf) => Some(&sf.statements),
            crate::ast::NodeData::ModuleBlock(mb) => Some(&mb.statements),
            _ => None,
        };
        let Some(stmts) = stmts else { return };
        let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
            || node.flags.contains(NodeFlags::Ambient)
            || self.ambient_context_depth > 0

            || node
                .parent
                .as_ref()
                .is_some_and(|_| {
                    let mut anc = node.parent.as_ref();
                    let mut found = false;
                    while let Some(a) = anc {
                        if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                            found = true;
                            break;
                        }
                        anc = a.parent.as_ref();
                    }
                    found
                })
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        if is_ambient {
            return;
        }
        let next = stmts.iter().enumerate().find_map(|(i, s)| {
            if Arc::ptr_eq(s, node) {
                stmts.nodes.get(i + 1).cloned()
            } else {
                None
            }
        });

        if next.as_ref().is_some_and(|n| {
            matches!(&n.data, crate::ast::NodeData::FunctionDeclaration(d) if d
                .name
                .as_ref()
                .is_some_and(|n2| n2.text() == name.text()))
        }) {
            return;
        }

        if let Some(n) = &next
            && matches!(&n.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            && n.kind == SyntaxKind::FunctionDeclaration
        {
            if let crate::ast::NodeData::FunctionDeclaration(d) = &n.data
                && let Some(next_name) = &d.name
                && next_name.kind == SyntaxKind::Identifier
                && next_name.text() != name.text()
            {

                let already = self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == 2389 && d.loc == next_name.loc);
                if !already {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        next_name.loc,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![name.text().to_string()],
                    ));
                }
                return;
            }
        }

        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 2391 && d.loc == name.loc);
        if !already {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                vec![],
            ));
        }
    }

    fn check_multiple_constructor_implementations(&mut self, node: &Arc<Node>) {
        let Some(class) = node.parent.as_ref() else {
            return;
        };
        if !matches!(class.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
            return;
        }
        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return;
        };
        let ctors: Vec<&Arc<Node>> = cd
            .members
            .iter()
            .filter(|m| m.kind == SyntaxKind::Constructor)
            .collect();
        if ctors.first().is_none_or(|first| !Arc::ptr_eq(first, node)) {
            return;
        }
        let bodied = ctors
            .iter()
            .filter(|c| {
                matches!(&c.data, crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some())
            })
            .count();
        if bodied < 2 {
            return;
        }
        let file = self.current_file.clone();
        for ctor in ctors {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file.clone(),
                ctor.loc,
                crate::diagnostics::messages_generated::
                    MULTIPLE_CONSTRUCTOR_IMPLEMENTATIONS_ARE_NOT_ALLOWED,
                vec![],
            ));
        }
    }

    fn check_invalid_initializer_reference(&mut self, node: &Arc<Node>, name: &str) -> bool {
        if self.emit_standard_class_fields {
            return false;
        }
        let Some(parent) = node.parent.as_ref() else {
            return false;
        };
        let Some(property) = crate::ast::utilities::find_ancestor(parent, |n| {
            n.kind == SyntaxKind::PropertyDeclaration
        }) else {
            return false;
        };

        if let Some(sym) = self.resolve_identifier(node) {
            let binds_in_initializer_fn = sym.declarations.iter().any(|d| {
                let mut cur = d.parent.as_ref();
                while let Some(a) = cur {
                    if Arc::ptr_eq(a, &property) {
                        return false;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::FunctionExpression
                            | SyntaxKind::ArrowFunction
                    ) {
                        return true;
                    }
                    cur = a.parent.as_ref();
                }
                false
            });
            if binds_in_initializer_fn {
                return false;
            }
        }
        if property.has_syntactic_modifier(ModifierFlags::Static) {
            return false;
        }
        let Some(class) = property.parent.as_ref() else {
            return false;
        };
        if !matches!(class.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
            return false;
        }

        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return false;
        };
        let ctor = cd.members.iter().find(|m| {
            m.kind == SyntaxKind::Constructor
                && matches!(&m.data, crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some())
        });
        let Some(ctor) = ctor else {
            return false;
        };
        let symbol_map = self.program.symbol_map();
        let ctor_has_name = symbol_map
            .locals
            .get(&ctor.id())
            .is_some_and(|locals| {
                locals
                    .get(name)
                    .is_some_and(|sym| sym.flags.intersects(SymbolFlags::VALUE))
            });
        if !ctor_has_name {
            return false;
        }
        let file = self.current_file.clone();
        let property_name = property
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            crate::diagnostics::messages_generated::
                INITIALIZER_OF_INSTANCE_MEMBER_VARIABLE_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_IN_THE_CONSTRUCTOR,
            vec![property_name, name.to_string()],
        ));
        true
    }

    fn check_contextual_elements(
        &mut self,
        expr: &Arc<Node>,
        target: &Arc<Type>,
        missing_anchor: TextRange,
    ) {
        if target.flags.contains(TypeFlags::Any) {
            return;
        }
        if expr.kind == SyntaxKind::ArrayLiteralExpression {
            let crate::ast::NodeData::ArrayLiteralExpression(data) = &expr.data else {
                return;
            };

            let elem_t = if self.is_array_type(target)
                || matches!(target.data, TypeData::EvolvingArray(_))
            {
                self.get_array_element_type(target)
            } else {
                self.get_any_type()
            };
            if elem_t.flags.contains(TypeFlags::Any) {

                let index_source = target.symbol.as_ref().and_then(|sym| {
                    let args = target.as_object()?.type_arguments.clone();
                    if sym.flags.contains(SymbolFlags::Interface) && !args.is_empty() {
                        Some(self.resolve_interface_type_ex(sym, Some(args)))
                    } else {
                        None
                    }
                });
                let index_source = index_source.unwrap_or_else(|| Arc::clone(target));
                let indexed = index_source.as_structured().and_then(|s| {
                    s.index_infos
                        .iter()
                        .find(|info| {
                            info.key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                        })
                        .and_then(|info| info.value_type.clone())
                });
                let Some(elem_t) = indexed else {
                    return;
                };
                if elem_t.flags.contains(TypeFlags::Any) {
                    return;
                }
                let mut inner = Vec::new();
                for el in data.elements.iter() {
                    if el.kind == SyntaxKind::SpreadElement {
                        continue;
                    }
                    inner.push(Arc::clone(el));
                }
                for el in inner {
                    let loc = el.loc;
                    self.check_contextual_elements(&el, &elem_t, loc);
                }
                return;
            }
            for el in data.elements.iter() {
                if el.kind == SyntaxKind::SpreadElement {
                    continue;
                }
                self.check_contextual_elements(el, &elem_t, el.loc);
            }
            return;
        }

        if matches!(
            expr.kind,
            SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression
        ) {
            let target = Arc::clone(target);
            let anchor = expr.loc;
            let assertion_type = match &expr.data {
                crate::ast::NodeData::TypeAssertion(d) => {
                    self.get_type_from_type_node(&d.type_node)
                }
                crate::ast::NodeData::AsExpression(d) => {
                    self.get_type_from_type_node(&d.type_node)
                }
                _ => return,
            };
            let missing =
                self.get_missing_required_properties(&assertion_type, &target);
            let file = self.current_file.clone();
            let src_str = self.type_to_string(&assertion_type);
            let tgt_str = self.type_to_string(&target);
            if missing.len() == 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    anchor,
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), src_str, tgt_str],
                ));
            } else if missing.len() > 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    anchor,
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![src_str, tgt_str, missing.join(", ")],
                ));
            }
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        if expr.kind == SyntaxKind::ObjectLiteralExpression {
            if let Some(excess) = self.get_excess_property_name(&expr_type, target) {
                let loc = self
                    .find_object_literal_property_name_node(expr, &excess)
                    .unwrap_or(expr.loc);
                let tgt_str = self.type_to_string(target);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![excess, tgt_str],
                ));
                return;
            }
            let missing = self.get_missing_required_properties(&expr_type, target);
            let file = self.current_file.clone();
            let src_str = self.type_to_string(&expr_type);
            let tgt_str = self.type_to_string(target);
            if missing.len() == 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    missing_anchor,
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), src_str, tgt_str],
                ));
            } else if missing.len() > 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    missing_anchor,
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![src_str, tgt_str, missing.join(", ")],
                ));
            }
            return;
        }

        if matches!(
            expr.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
        ) && !self.is_type_assignable_to(&expr_type, target)
        {
            let display_type = if crate::checker::is_literal_type(&expr_type) {
                self.get_base_type_of_literal_type(&expr_type)
            } else {
                expr_type.clone()
            };
            let src_str = self.type_to_string(&display_type);
            let tgt_str = self.type_to_string(target);

            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2322 && d.loc == expr.loc);
            if already {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                expr.loc,
                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                vec![src_str, tgt_str],
            ));
        }
    }

    fn unwrap_async_return_type(&self, declared: Arc<Type>, is_async: bool) -> Arc<Type> {
        if !is_async {
            return declared;
        }

        let is_promise = declared
            .symbol
            .as_ref()
            .is_some_and(|s| s.name == "Promise");
        if is_promise {
            if let crate::checker::TypeData::Object(obj) = &declared.data {
                if let Some(t) = obj.type_arguments.first() {
                    return Arc::clone(t);
                }
            }
            return self.get_any_type();
        }
        declared
    }

    pub fn get_awaited_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_awaited_type_with_depth(t, 0)
    }

    fn get_awaited_type_with_depth(
        &mut self,
        t: &Arc<Type>,
        depth: usize,
    ) -> Option<Arc<Type>> {

        if depth > 50 {
            return None;
        }

        if t.flags.contains(TypeFlags::Any) {
            return Some(Arc::clone(t));
        }

        if let crate::checker::TypeData::Union(u) = &t.data {
            let mut mapped: Vec<Arc<Type>> = Vec::with_capacity(u.union_or_intersection.types.len());
            for constituent in &u.union_or_intersection.types {
                let awaited = self
                    .get_awaited_type_with_depth(constituent, depth + 1)
                    .unwrap_or_else(|| Arc::clone(constituent));
                mapped.push(awaited);
            }
            return Some(self.get_union_type(mapped));
        }
        if let Some(promised) = self.get_promised_type_of_promise(t) {
            if Arc::ptr_eq(&promised, t) {

                return None;
            }
            return self.get_awaited_type_with_depth(&promised, depth + 1);
        }

        Some(Arc::clone(t))
    }

    fn get_promised_type_of_promise(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {

        if t.symbol.as_ref().is_some_and(|s| s.name == "Promise") {
            if let crate::checker::TypeData::Object(obj) = &t.data {
                if let Some(first) = obj.type_arguments.first() {
                    return Some(Arc::clone(first));
                }
            }
            return None;
        }

        if !t.flags.contains(TypeFlags::Object) {
            return None;
        }
        let then_fn = self.get_property_of_type(t, "then")?;
        let then_type = self.get_type_of_symbol(&then_fn);
        if then_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let then_signatures = self.get_signatures_of_type(&then_type, SignatureKind::Call);
        let then_sig = then_signatures.first()?;
        let onfulfilled = then_sig.parameters.first()?;
        let callback_type = self.get_type_of_symbol(onfulfilled);
        if callback_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let callback_signatures =
            self.get_signatures_of_type(&callback_type, SignatureKind::Call);
        let callback_sig = callback_signatures.first()?;
        let value_param = callback_sig.parameters.first()?;
        Some(self.get_type_of_symbol(value_param))
    }

    fn declared_annotation_type_of(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        if node.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(node)?;
        let decl = sym.value_declaration.clone()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data else {
            return None;
        };
        let tn = vd.type_node.as_ref()?;
        Some(self.get_type_from_type_node(tn))
    }

    fn is_valid_const_assertion_argument(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::TemplateExpression => true,
            SyntaxKind::ParenthesizedExpression => match &node.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => {
                    self.is_valid_const_assertion_argument(&p.expression)
                }
                _ => false,
            },
            SyntaxKind::PrefixUnaryExpression => match &node.data {
                crate::ast::NodeData::PrefixUnaryExpression(p) => {
                    let arg_kind = p.operand.kind;
                    (p.operator == SyntaxKind::MinusToken
                        && matches!(
                            arg_kind,
                            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
                        ))
                        || (p.operator == SyntaxKind::PlusToken
                            && arg_kind == SyntaxKind::NumericLiteral)
                }
                _ => false,
            },
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {

                let (obj, _name) = match &node.data {
                    crate::ast::NodeData::PropertyAccessExpression(d) => {
                        (Some(d.expression.clone()), d.name.text().to_string())
                    }
                    crate::ast::NodeData::ElementAccessExpression(d) => {
                        let arg = &d.argument_expression;
                        if arg.kind == SyntaxKind::StringLiteral {
                            (Some(d.expression.clone()), arg.text().to_string())
                        } else {
                            (None, String::new())
                        }
                    }
                    _ => (None, String::new()),
                };
                match obj {
                    Some(obj) if obj.kind == SyntaxKind::Identifier => {
                        self.resolve_qualified_symbol(node)
                            .or_else(|| self.resolve_identifier(&obj))
                            .map(|sym| {
                                sym.flags
                                    .intersects(SymbolFlags::ENUM | SymbolFlags::EnumMember)
                            })
                            .unwrap_or(false)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn is_const_type_node(type_node: &Arc<Node>) -> bool {
        type_node.kind == SyntaxKind::ConstKeyword
    }

    fn check_delete_operand(&mut self, operand: &Arc<Node>) {
        let mut target = operand;
        while target.kind == SyntaxKind::ParenthesizedExpression {
            let inner = match &target.data {
                crate::ast::NodeData::ParenthesizedExpression(p) => &p.expression,
                _ => break,
            };
            target = inner;
        }
        match target.kind {
            SyntaxKind::Identifier => {

                let strict =
                    self.program.options().get_strict_option_value(
                        self.program.options().always_strict,
                    ) || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                if strict {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            X_DELETE_CANNOT_BE_CALLED_ON_AN_IDENTIFIER_IN_STRICT_MODE,
                        vec![],
                    ));
                }
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    target.loc,
                    crate::diagnostics::messages_generated::
                        THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_A_PROPERTY_REFERENCE,
                    vec![],
                ));
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                let (obj_expr, name, _name_loc) = match &target.data {
                    crate::ast::NodeData::PropertyAccessExpression(d) => {
                        (&d.expression, d.name.text().to_string(), d.name.loc)
                    }
                    crate::ast::NodeData::ElementAccessExpression(d) => {
                        let arg = &d.argument_expression;
                        if arg.kind == SyntaxKind::StringLiteral {
                            (&d.expression, arg.text().to_string(), arg.loc)
                        } else {
                            return;
                        }
                    }
                    _ => return,
                };

                if matches!(&target.data, crate::ast::NodeData::PropertyAccessExpression(d) if d.name.kind == SyntaxKind::PrivateIdentifier)
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_PRIVATE_IDENTIFIER,
                        vec![],
                    ));
                    return;
                }

                let obj_type = self.get_type_of_node(obj_expr);
                if obj_type.flags.contains(TypeFlags::Any) {
                    return;
                }
                if self.is_property_readonly(&obj_type, &name) {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        target.loc,
                        crate::diagnostics::messages_generated::
                            THE_OPERAND_OF_A_DELETE_OPERATOR_CANNOT_BE_A_READ_ONLY_PROPERTY,
                        vec![name],
                    ));
                    return;
                }

                if let Some(structured) = obj_type.as_structured() {
                    let readonly_index = structured.index_infos.iter().any(|info| {
                        info.is_readonly
                            && info
                                .key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::String))
                    });
                    if readonly_index {
                        let type_name = self.type_to_string(&obj_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            target.loc,
                            crate::diagnostics::messages_generated::
                                INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                            vec![type_name],
                        ));
                        return;
                    }
                }

                if self.strict_null_checks && self.has_property_of_type(&obj_type, &name) {

                    let prop = obj_type.as_structured().and_then(|s| {
                        s.properties
                            .iter()
                            .find(|p| p.name == name)
                            .map(|p| Arc::clone(p))
                    });
                    let deletable = prop.as_ref().is_some_and(|p| {
                        if p.flags.contains(SymbolFlags::Optional) {
                            return true;
                        }
                        let t = self.get_type_of_symbol(p);
                        t.flags.intersects(
                            TypeFlags::Undefined
                                | TypeFlags::Any
                                | TypeFlags::Unknown
                                | TypeFlags::Never,
                        ) || match &t.data {
                            crate::checker::TypeData::Union(u) => u
                                .union_or_intersection
                                .types
                                .iter()
                                .any(|m| {
                                    m.flags.intersects(
                                        TypeFlags::Undefined
                                            | TypeFlags::Any
                                            | TypeFlags::Unknown
                                            | TypeFlags::Never,
                                    )
                                }),
                            _ => false,
                        }
                    }) || obj_type.as_structured().is_some_and(|s| {

                        s.index_infos.iter().any(|info| {
                            info.key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::String))
                        })
                    });
                    if !deletable {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            target.loc,
                            crate::diagnostics::messages_generated::
                                THE_OPERAND_OF_A_DELETE_OPERATOR_MUST_BE_OPTIONAL,
                            vec![],
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn check_const_assignment_target(&mut self, operand: &Arc<Node>) {

        let mut target = operand;
        loop {
            target = match &target.data {

                crate::ast::NodeData::ParenthesizedExpression(p) => &p.expression,
                crate::ast::NodeData::NonNullExpression(n) => &n.expression,
                _ => break,
            };
        }
        let operand = target;
        if operand.kind == SyntaxKind::PropertyAccessExpression
            || operand.kind == SyntaxKind::ElementAccessExpression
        {
            self.check_const_property_assignment(operand);
            return;
        }
        if operand.kind != SyntaxKind::Identifier {
            return;
        }
        if let Some(symbol) = self.resolve_identifier(operand)
            && self.symbol_is_const_variable(&symbol)
        {
            let name_text = operand.text();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                operand.loc,
                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT,
                vec![name_text.to_string()],
            ));
        }
    }

    fn check_const_property_assignment(&mut self, node: &Arc<Node>) {

        let (obj_expr, name, name_loc) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                (&data.expression, &data.name, data.name.loc)
            }
            crate::ast::NodeData::ElementAccessExpression(data) => {
                let arg = &data.argument_expression;
                if arg.kind != SyntaxKind::StringLiteral {
                    return;
                }
                (&data.expression, arg, arg.loc)
            }
            _ => return,
        };
        if obj_expr.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(sym) = self.resolve_identifier(obj_expr) else {
            return;
        };
        let base = self.resolve_alias_base(sym);
        if !base.flags.contains(SymbolFlags::ValueModule) {
            return;
        }
        let name_text = name.text();
        let member = base
            .exports
            .get(name_text)
            .or_else(|| base.members.get(name_text))
            .cloned()
            .or_else(|| {
                base.declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .find_map(|d| {
                        self.program
                            .symbol_map()
                            .locals
                            .get(&d.id())
                            .and_then(|l| l.get(name_text).cloned())
                    })
            });
        if member.is_some_and(|m| self.symbol_is_const_variable(&m)) {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_loc,
                CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                vec![name_text.to_string()],
            ));
        }
    }

    fn check_block_scoped_variable_used_before_declaration(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {

        {
            let decl = symbol
                .value_declaration
                .as_ref()
                .or_else(|| symbol.declarations.first());
            if let Some(mut current) = decl {
                loop {
                    match current.kind {
                        SyntaxKind::VariableDeclaration => {
                            let is_var = current
                                .parent
                                .as_ref()
                                .is_some_and(|parent| {
                                    parent.kind == SyntaxKind::VariableDeclarationList
                                        && !parent.flags.intersects(
                                            crate::ast::NodeFlags::Let
                                                | crate::ast::NodeFlags::Const,
                                        )
                                });
                            if is_var {
                                return;
                            }
                            break;
                        }
                        SyntaxKind::BindingElement
                        | SyntaxKind::ObjectBindingPattern
                        | SyntaxKind::ArrayBindingPattern => match current.parent.as_ref() {
                            Some(parent) => current = parent,
                            None => break,
                        },
                        _ => break,
                    }
                }
            }
        }

        let mut enum_decl_count = 0;
        let is_const_enum = symbol
            .declarations
            .iter()
            .filter(|d| {
                if d.kind == SyntaxKind::EnumDeclaration {
                    enum_decl_count += 1;
                    true
                } else {
                    false
                }
            })
            .all(|d| {
                let Some(f) = self
                    .get_source_file_of_node(d)
                    .or_else(|| self.current_file.clone())
                else {
                    return false;
                };
                let text = &f.text;
                let start = d.loc.pos();

                let lo = start.saturating_sub(8);
                let window = &text[lo.min(text.len())..(start + 6).min(text.len())];
                window.contains("const")
            });
        if is_const_enum
            && enum_decl_count > 0
            && !self.compiler_options.isolated_modules.is_true()
        {
            return;
        }

        {

            let in_tp_default = {
                let mut cur = node.parent.as_ref();
                let mut hit = false;
                while let Some(a) = cur {
                    if a.kind == SyntaxKind::TypeParameter {
                        hit = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::ClassDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::Block
                            | SyntaxKind::SourceFile
                    ) {
                        break;
                    }
                    cur = a.parent.as_ref();
                }
                hit
            };
            if in_tp_default {
                return;
            }
            let in_type_position = {
                let mut cur = node.parent.as_ref();
                let mut hit = false;
                while let Some(a) = cur {
                    if matches!(
                        a.kind,
                        SyntaxKind::TypeReference
                            | SyntaxKind::TypeParameter
                            | SyntaxKind::ArrayType
                            | SyntaxKind::UnionType
                            | SyntaxKind::IntersectionType
                            | SyntaxKind::ParenthesizedType
                            | SyntaxKind::TupleType
                            | SyntaxKind::TypeLiteral
                            | SyntaxKind::FunctionType
                            | SyntaxKind::ConstructorType
                            | SyntaxKind::QualifiedName
                            | SyntaxKind::HeritageClause
                    ) {
                        hit = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::ClassDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::Block
                            | SyntaxKind::SourceFile
                    ) {
                        break;
                    }
                    cur = a.parent.as_ref();
                }
                hit
            };
            if in_type_position {
                return;
            }
        }
        if !symbol.flags.intersects(
            SymbolFlags::BlockScopedVariable
                | SymbolFlags::Class
                | SymbolFlags::ENUM,
        ) {
            return;
        }

        let declaration_for_scope = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::EnumDeclaration
            )
        });
        if let Some(declaration_for_scope) = declaration_for_scope {

            let is_fn_like = |n: &Arc<Node>| {
                matches!(
                    n.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                )
            };
            let immediately_invoked = |n: &Arc<Node>| -> bool {
                let Some(p) = n.parent.as_ref() else {
                    return false;
                };
                match &p.data {
                    crate::ast::NodeData::CallExpression(_) => true,
                    crate::ast::NodeData::ParenthesizedExpression(_) => {

                        let mut cur = p.parent.as_ref();
                        while let Some(a) = cur {
                            if matches!(&a.data, crate::ast::NodeData::CallExpression(_)) {
                                return true;
                            }
                            if matches!(&a.data, crate::ast::NodeData::ParenthesizedExpression(_)) {
                                cur = a.parent.as_ref();
                                continue;
                            }
                            break;
                        }
                        false
                    }
                    _ => false,
                }
            };

            let mut dc = declaration_for_scope.parent.as_ref();
            let mut decl_container: Option<Arc<Node>> = None;
            while let Some(a) = dc {
                if is_fn_like(a) {
                    decl_container = Some(Arc::clone(a));
                    break;
                }
                dc = a.parent.as_ref();
            }
            let mut cur = node.parent.as_ref();
            let mut exempt = false;
            while let Some(a) = cur {
                if let Some(dcont) = &decl_container {
                    if Arc::ptr_eq(a, dcont) {
                        break;
                    }
                }
                if is_fn_like(a) {
                    if immediately_invoked(a) {
                        cur = a.parent.as_ref();
                        continue;
                    }
                    exempt = true;
                    break;
                }

                if a.kind == SyntaxKind::PropertyDeclaration {

                    let in_initializer = matches!(&a.data, crate::ast::NodeData::PropertyDeclaration(pd) if pd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())));
                    let is_static_prop = a.has_syntactic_modifier(ModifierFlags::Static);
                    let is_decl_instance_prop = declaration_for_scope.kind
                        == SyntaxKind::PropertyDeclaration
                        && !declaration_for_scope.has_syntactic_modifier(ModifierFlags::Static);
                    if in_initializer && !is_static_prop && !is_decl_instance_prop {
                        exempt = true;
                        break;
                    }
                }
                cur = a.parent.as_ref();
            }
            if exempt {
                return;
            }
        }

        let declaration = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::EnumDeclaration
            )
        });
        let Some(declaration) = declaration else {
            return;
        };

        if declaration.kind == SyntaxKind::VariableDeclaration
            && !is_let_or_const_declaration(declaration)
        {
            return;
        }

        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
        {
            return;
        }

        let decl_name_pos = match &declaration.data {
            crate::ast::NodeData::VariableDeclaration(d) => d.name.pos(),
            crate::ast::NodeData::BindingElement(d) => d
                .name
                .as_ref()
                .map(|n| n.pos())
                .unwrap_or(declaration.pos()),
            _ => declaration.pos(),
        };
        if decl_name_pos <= node.pos() {

            let inside_own_initializer = {

                let mut cur = declaration.parent.as_ref();
                let mut found = false;
                while let Some(a) = cur {
                    if matches!(&a.data, crate::ast::NodeData::VariableDeclaration(vdd)
                        if vdd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())))
                    {
                        found = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::BindingElement
                            | SyntaxKind::ArrayBindingPattern
                            | SyntaxKind::ObjectBindingPattern
                    ) {
                        cur = a.parent.as_ref();
                        continue;
                    }
                    break;
                }
                found
            };
            if !inside_own_initializer {
                return;
            }
        }

        let decl_file = self.get_source_file_of_node(declaration);
        let use_file = self.get_source_file_of_node(node);
        if let (Some(df), Some(uf)) = (&decl_file, &use_file) {
            if df.file_name != uf.file_name {
                return;
            }
        }
        let file = self.current_file.clone();

        let message = if symbol.flags.contains(SymbolFlags::Class) {
            crate::diagnostics::messages_generated::CLASS_0_USED_BEFORE_ITS_DECLARATION
        } else if symbol.flags.intersects(SymbolFlags::RegularEnum)
            || (symbol.flags.intersects(SymbolFlags::ConstEnum)
                && self.compiler_options.isolated_modules.is_true())
        {
            crate::diagnostics::messages_generated::ENUM_0_USED_BEFORE_ITS_DECLARATION
        } else {
            BLOCK_SCOPED_VARIABLE_0_USED_BEFORE_ITS_DECLARATION
        };
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == message.code && d.loc == node.loc);
        if !already {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                message,
                vec![name.to_string()],
            ));
        }
    }

    fn check_variable_used_before_assigned(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {

        if is_assignment_target(node) {
            return;
        }

        if !self.strict_null_checks {
            return;
        }

        let is_plain_var = symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            && symbol
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::VariableDeclaration);
        if !symbol.flags.contains(SymbolFlags::BlockScopedVariable) && !is_plain_var {
            return;
        }

        let declaration = symbol.value_declaration.as_ref().or_else(|| {
            symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::VariableDeclaration)
        });
        let Some(declaration) = declaration else {
            return;
        };

        let crate::ast::NodeData::VariableDeclaration(vd) = &declaration.data else {
            return;
        };

        if vd.type_node.is_none() && vd.initializer.is_none() {
            return;
        }

        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
            || vd.exclamation_token.is_some()
        {
            return;
        }

        let declared_type = self.get_type_of_symbol(symbol);
        if declared_type.flags.contains(TypeFlags::Any)
            || type_contains_undefined(&declared_type)
        {
            return;
        }

        let flow_container_of = |n: &Arc<Node>| -> Option<Arc<Node>> {
            let mut current = Arc::clone(n);
            loop {
                if matches!(
                    current.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::ModuleDeclaration

                        | SyntaxKind::PropertyDeclaration
                        | SyntaxKind::PropertySignature
                ) {
                    return Some(current);
                }
                current = Arc::clone(current.parent.as_ref()?);
            }
        };
        let same_scope = match (flow_container_of(node), flow_container_of(declaration)) {
            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
            _ => true,
        };
        if !same_scope {
            return;
        }

        if node
            .parent
            .as_ref()
            .is_some_and(|p| p.kind == SyntaxKind::NonNullExpression)
        {
            return;
        }

        if !self.strict_null_checks {
            return;
        }
        if let Some(flow_type) = self.get_definite_assignment_flow_type(symbol, node) {
            if type_contains_undefined(&flow_type) {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED,
                    vec![name.to_string()],
                ));
            }
        }
    }

    pub(crate) fn push_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes += 1;
        if self.suppress_source_file.is_none() {
            self.suppress_source_file = self.current_file.as_ref().map(|f| f.node.id());
        }
    }

    pub(crate) fn pop_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes = self
            .suppress_cannot_find_name_in_type_nodes
            .saturating_sub(1);
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            self.suppress_source_file = None;
        }
    }

    pub(crate) fn ts2304_reporting_allowed_for(&self, node: &Arc<Node>) -> bool {
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            return true;
        }
        match (
            self.get_source_file_of_node(node),
            self.suppress_source_file,
        ) {
            (Some(f), Some(origin)) => {
                if f.node.id() == origin {

                    false
                } else {

                    !f.file_name.starts_with("bundled://")
                }
            }
            _ => false,
        }
    }

    pub(crate) fn push_scope(&mut self, node: &Arc<Node>) {
        self.scope_stack.push(node.id());
    }

    fn push_function_scope(&mut self, node: &Arc<Node>) {
        self.function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    fn pop_function_scope(&mut self) {
        self.function_scope_count -= 1;
        self.scope_stack.pop();
    }

    fn push_arrow_function_scope(&mut self, node: &Arc<Node>) {
        self.arrow_function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    fn pop_arrow_function_scope(&mut self) {
        self.arrow_function_scope_count -= 1;
        self.scope_stack.pop();
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

    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let result = self.resolve_identifier_with_meaning_inner(node, meaning);

        if let Some(sym) = &result {
            let mut bits = meaning;
            if meaning.intersects(SymbolFlags::VALUE) {
                bits |= SymbolFlags::FunctionScopedVariable
                    | SymbolFlags::BlockScopedVariable;
            }
            self.record_symbol_reference(sym, bits);
        }
        result
    }

    fn record_symbol_reference(&self, symbol: &Arc<Symbol>, bits: SymbolFlags) {
        self.symbol_reference_kinds
            .entry(symbol.id())
            .and_modify(|f| *f |= bits)
            .or_insert(bits);
    }

    fn alias_chain_hits_meaning(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool {
        if !sym.flags.intersects(SymbolFlags::Alias) {
            return false;
        }
        match self.follow_alias(sym) {
            Some(target) if !Arc::ptr_eq(&target, sym) => target.flags.intersects(meaning),
            _ => true,
        }
    }

    fn resolve_identifier_with_meaning_inner(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        for &container_id in self.scope_stack.iter().rev() {

            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }

            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {

                if !container_sym.flags.intersects(SymbolFlags::Class)
                    || container_sym.flags.intersects(SymbolFlags::Function)
                {
                    if let Some(sym) = container_sym.members.get(name) {
                        if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::MODULE)
                    && !container_sym.flags.intersects(SymbolFlags::Class)
                {
                    if let Some(sym) = container_sym.exports.get(name) {

                        let is_export_specifier = sym.flags == SymbolFlags::Alias
                            && sym
                                .declarations
                                .iter()
                                .any(|d| d.kind == SyntaxKind::ExportSpecifier);
                        if !is_export_specifier {
                            return self.follow_alias(sym);
                        }
                    }

                    if let Some(merged) = self.globals.get(container_sym.name.as_str()) {
                        if !Arc::ptr_eq(merged, container_sym)
                            && merged.flags.intersects(SymbolFlags::MODULE)
                        {
                            if let Some(sym) = merged.exports.get(name) {
                                if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                                    return self.follow_alias(sym);
                                }
                            }
                            if let Some(sym) = self.ambient_namespace_local(merged, name) {
                                if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                                    return self.follow_alias(&sym);
                                }
                            }
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }

                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning & SymbolFlags::TYPE) || self.alias_chain_hits_meaning(&sym, meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }
        }

        {

            const ANCESTRY_CONTAINERS: &[SyntaxKind] = &[
                SyntaxKind::SourceFile,
                SyntaxKind::ModuleDeclaration,
                SyntaxKind::Block,
                SyntaxKind::CatchClause,
                SyntaxKind::ForStatement,
                SyntaxKind::ForInStatement,
                SyntaxKind::ForOfStatement,
                SyntaxKind::FunctionDeclaration,
                SyntaxKind::FunctionExpression,
                SyntaxKind::ArrowFunction,
                SyntaxKind::MethodDeclaration,

                SyntaxKind::MethodSignature,

                SyntaxKind::CallSignature,
                SyntaxKind::ConstructSignature,
                SyntaxKind::FunctionType,
                SyntaxKind::ConstructorType,
                SyntaxKind::MappedType,
                SyntaxKind::Constructor,
                SyntaxKind::GetAccessor,
                SyntaxKind::SetAccessor,
                SyntaxKind::InterfaceDeclaration,
                SyntaxKind::ClassDeclaration,
                SyntaxKind::ClassExpression,
                SyntaxKind::TypeAliasDeclaration,
                SyntaxKind::EnumDeclaration,
            ];
            let mut ancestor = node.parent.as_ref();
            while let Some(a) = ancestor {
                if !ANCESTRY_CONTAINERS.contains(&a.kind) {
                    ancestor = a.parent.as_ref();
                    continue;
                }
                let aid = a.id();
                if let Some(locals) = symbol_map.locals.get(&aid) {
                    if let Some(sym) = locals.get(name)
                        && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return self.follow_alias(sym);
                    }
                }
                if let Some(a_sym) = symbol_map.symbols.get(&aid) {
                    if !a_sym.flags.intersects(SymbolFlags::Class) {
                        if let Some(sym) = a_sym.members.get(name)
                            && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return self.follow_alias(sym);
                        }
                        if a_sym.flags.intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                            && let Some(sym) = a_sym.exports.get(name)
                            && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return self.follow_alias(sym);
                        }

                        if a_sym.flags.intersects(SymbolFlags::MODULE) {
                            if let Some(merged) = self.globals.get(a_sym.name.as_str()) {
                                if !Arc::ptr_eq(merged, a_sym)
                                    && merged.flags.intersects(SymbolFlags::MODULE)
                                {
                                    if let Some(sym) = merged.exports.get(name)
                                        && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                                    {
                                        return self.follow_alias(sym);
                                    }
                                    if let Some(sym) =
                                    self.ambient_namespace_local(merged, name)
                                    && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                                {
                                    return self.follow_alias(&sym);
                                }
                            }
                        }
                    }
                    }

                    if let Some(sym) = a_sym.members.get(name)
                        && (sym.flags.intersects(meaning & SymbolFlags::TYPE) || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return self.follow_alias(sym);
                    }
                }
                ancestor = a.parent.as_ref();
            }
        }

        if self.function_scope_count > 0
            && name == "arguments"
            && meaning.intersects(SymbolFlags::VARIABLE)
        {
            if let Some(ref sym) = self.arguments_symbol {
                return Some(Arc::clone(sym));
            }
        }

        if let Some(sym) = self.globals.get(name) {
            if sym
                .flags
                .intersects(meaning.union(SymbolFlags::GlobalLookup))
            {
                return Some(Arc::clone(sym));
            }
        }

        None
    }

    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {

        if symbol.flags.intersects(SymbolFlags::Alias) {
            self.record_symbol_reference(
                symbol,
                SymbolFlags::VALUE
                    | SymbolFlags::TYPE
                    | SymbolFlags::NAMESPACE
                    | SymbolFlags::FunctionScopedVariable
                    | SymbolFlags::BlockScopedVariable,
            );
        }
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(Arc::clone(symbol));
        }

        let is_pure_alias = symbol.flags == SymbolFlags::Alias
            || (symbol.flags.intersects(SymbolFlags::Alias)
                && symbol.flags.intersects(SymbolFlags::Assignment));
        if !is_pure_alias {
            return Some(Arc::clone(symbol));
        }

        let mut current = Arc::clone(symbol);
        let mut seen: Vec<*const Symbol> = vec![Arc::as_ptr(symbol)];
        loop {
            if let Some(ref target) = current.export_symbol {
                let target_ptr = Arc::as_ptr(target);
                if seen.contains(&target_ptr) {

                    return Some(Arc::clone(&current));
                }
                let is_pure = target.flags == SymbolFlags::Alias
                    || (target.flags.intersects(SymbolFlags::Alias)
                        && target.flags.intersects(SymbolFlags::Assignment));
                if is_pure {
                    seen.push(target_ptr);
                    current = Arc::clone(target);
                    continue;
                }
                return Some(Arc::clone(target));
            } else {

                return Some(Arc::clone(&current));
            }
        }
    }

    fn get_referenced_value_symbol(
        &self,
        node: &Node,
        start_in_declaration_container: bool,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

        if let Some(sym) = symbol_map.symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        let location = if start_in_declaration_container {

            node
        } else {
            node
        };

        let meaning = SymbolFlags::ExportValue
            .union(SymbolFlags::VALUE)
            .union(SymbolFlags::Alias);
        self.resolve_identifier_at_location(location, node_name(node)?, meaning)
    }

    #[allow(dead_code)]
    fn find_parent_declaration_container(&self, _node: &Node) -> Option<u64> {

        for &container_id in self.scope_stack.iter().rev() {
            let symbol_map = self.program.symbol_map();
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if container_sym
                    .flags
                    .intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                {
                    return Some(container_id);
                }
            }
        }
        None
    }

    pub fn get_referenced_export_container(&self, node: &Node, prefix_locals: bool) -> Option<u64> {

        let start_in_declaration_container = is_module_or_enum_name(node);
        if let Some(symbol) = self.get_referenced_value_symbol(node, start_in_declaration_container)
        {
            if symbol.flags.intersects(SymbolFlags::ExportValue) {
                if let Some(ref export_symbol) = symbol.export_symbol {
                    let merged = self.get_merged_symbol(export_symbol);
                    if !prefix_locals
                        && merged.flags.intersects(SymbolFlags::EXPORT_HAS_LOCAL)
                        && !merged.flags.intersects(SymbolFlags::VARIABLE)
                    {
                        return None;
                    }

                    if let Some(parent) = &merged.parent {
                        if parent.flags.intersects(SymbolFlags::ValueModule)
                            && parent.value_declaration.is_some()
                        {
                            return Some(parent.value_declaration.as_ref().unwrap().id());
                        }

                        for &container_id in self.scope_stack.iter().rev() {
                            let symbol_map = self.program.symbol_map();
                            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                                if Arc::ptr_eq(container_sym, parent) {
                                    return Some(container_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_referenced_import_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {

            if is_non_local_alias(&symbol, SymbolFlags::VALUE)
                && !self.is_type_only_alias_declaration(&symbol)
            {
                return self.get_declaration_of_alias_symbol(&symbol);
            }
        }
        None
    }

    pub fn get_referenced_value_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
            return export_sym.value_declaration.clone();
        }
        None
    }

    pub fn get_referenced_value_declarations(&self, node: &Node) -> Vec<Arc<Node>> {
        let mut declarations = Vec::new();
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
            for decl in export_sym.declarations.iter() {
                match decl.kind {
                    SyntaxKind::VariableDeclaration
                    | SyntaxKind::Parameter
                    | SyntaxKind::BindingElement
                    | SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertyAssignment
                    | SyntaxKind::ShorthandPropertyAssignment
                    | SyntaxKind::EnumMember
                    | SyntaxKind::ObjectLiteralExpression
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::ModuleDeclaration => {
                        declarations.push(Arc::clone(decl));
                    }
                    _ => {}
                }
            }
        }
        declarations
    }

    pub fn get_element_access_expression_name(&self, expression: &Node) -> Option<String> {
        if expression.kind == SyntaxKind::ElementAccessExpression {
            if let crate::ast::NodeData::ElementAccessExpression(data) = &expression.data {

                if let crate::ast::NodeData::StringLiteral(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }

                if let crate::ast::NodeData::NumericLiteral(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }

                if let crate::ast::NodeData::Identifier(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }
            }
        }
        None
    }

    pub fn get_referenced_member_value_declaration(&self, node: &Node) -> Option<Arc<Node>> {

        let symbol_map = self.program.symbol_map();
        let s = symbol_map.symbol_of(node).map(|s| Arc::clone(s));
        if s.is_none() {

            if let Some(sym) = symbol_map.symbol_of(node) {
                let merged = self.get_merged_symbol(sym);
                let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&merged);
                return export_sym.value_declaration.clone();
            }
        }
        if let Some(ref s) = s {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(s);
            return export_sym.value_declaration.clone();
        }
        None
    }

    pub fn get_merged_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(_target_id) = self.merged_symbols.get(&symbol.id()) {

        }
        Arc::clone(symbol)
    }

    fn get_export_symbol_of_value_symbol_if_exported(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        let mut result = Arc::clone(symbol);
        if symbol.flags.intersects(SymbolFlags::ExportValue) {
            if let Some(ref export_sym) = symbol.export_symbol {
                result = self.get_merged_symbol(export_sym);
            }
        }
        result
    }

    fn is_type_only_alias_declaration(&self, symbol: &Arc<Symbol>) -> bool {
        if let Some(node) = self.get_declaration_of_alias_symbol(symbol) {
            let current = Some(Arc::clone(&node));
            while let Some(ref n) = current {
                match n.kind {
                    SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ExportDeclaration => {
                        return is_type_only_node(n);
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier => {
                        if is_type_only_node(n) {
                            return true;
                        }

                        break;
                    }
                    _ => break,
                }
            }
        }
        false
    }

    fn get_declaration_of_alias_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> {

        symbol
            .declarations
            .iter()
            .filter(|d| is_alias_symbol_declaration(d))
            .last()
            .cloned()
    }

    fn resolve_identifier_at_location(
        &self,
        _location: &Node,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {

        let symbol_map = self.program.symbol_map();

        for &container_id in self.scope_stack.iter().rev() {

            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }

            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::MODULE) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        let is_export_specifier = sym.flags == SymbolFlags::Alias
                            && sym
                                .declarations
                                .iter()
                                .any(|d| d.kind == SyntaxKind::ExportSpecifier);
                        if !is_export_specifier {
                            return self.follow_alias(sym);
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }
            }
        }

        if let Some(sym) = self.globals.get(name) {
            if sym
                .flags
                .intersects(meaning.union(SymbolFlags::GlobalLookup))
            {
                return Some(Arc::clone(sym));
            }
        }

        None
    }

    pub fn merge_symbol_table(
        &mut self,
        target: &mut SymbolTable,
        source: &SymbolTable,
        unidirectional: bool,
        merged_parent: Option<u64>,
    ) {

        let entries: Vec<(String, Arc<Symbol>)> = source
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        for (name, source_symbol) in entries {
            if let Some(target_symbol) = target.entries.get_mut(&name) {

                let merged = self.merge_symbol(target_symbol, &source_symbol, unidirectional);
                let is_transient = merged.flags.intersects(SymbolFlags::Transient);
                *target_symbol = merged;
                if let Some(_parent_id) = merged_parent {
                    if is_transient {

                    }
                }
            } else {

                let merged = self.get_merged_symbol(&source_symbol);
                target.insert(name, merged);
            }
        }
    }

    pub fn merge_symbol(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
        unidirectional: bool,
    ) -> Arc<Symbol> {
        let excluded = get_excluded_symbol_flags(source.flags);
        if target.flags.intersects(excluded) == false
            || (source.flags | target.flags).intersects(SymbolFlags::Assignment)
        {
            if Arc::ptr_eq(target, source) {
                return Arc::clone(target);
            }

            let effective_target = if !target.flags.intersects(SymbolFlags::Transient) {
                let resolved_target = self.resolve_symbol(target);
                if resolved_target
                    .flags
                    .intersects(get_excluded_symbol_flags(source.flags))
                    == false
                    || (source.flags | resolved_target.flags).intersects(SymbolFlags::Assignment)
                {
                    if let Some(cloned) = self.clone_symbol(&resolved_target) {
                        cloned
                    } else {
                        return Arc::clone(source);
                    }
                } else {

                    return Arc::clone(source);
                }
            } else {
                Arc::clone(target)
            };

            let mut source_flags = source.flags;
            if !effective_target
                .flags
                .intersects(SymbolFlags::ConstEnumOnlyModule)
            {
                source_flags.remove(SymbolFlags::ConstEnumOnlyModule);
            }
            let merged_flags = effective_target.flags | source_flags;

            let mut merged = Symbol::new(merged_flags, &effective_target.name);

            merged.value_declaration = source
                .value_declaration
                .clone()
                .or_else(|| effective_target.value_declaration.clone());

            merged.declarations = effective_target.declarations.clone();
            merged
                .declarations
                .extend(source.declarations.iter().cloned());

            merged.parent = effective_target.parent.clone();

            merged.members = SymbolTable {
                entries: effective_target.members.entries.clone(),
            };
            merged.exports = SymbolTable {
                entries: effective_target.exports.entries.clone(),
            };

            let result = Arc::new(merged);

            let mut result_mut = Symbol::new(result.flags, &result.name);
            result_mut.value_declaration = result.value_declaration.clone();
            result_mut.declarations = result.declarations.clone();
            result_mut.parent = result.parent.clone();
            result_mut.members = SymbolTable {
                entries: result.members.entries.clone(),
            };
            result_mut.exports = SymbolTable {
                entries: result.exports.entries.clone(),
            };

            let source_members: Vec<(String, Arc<Symbol>)> = source
                .members
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_members {
                if let Some(target_sym) = result_mut.members.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.members.insert(name, merged);
                }
            }

            let source_exports: Vec<(String, Arc<Symbol>)> = source
                .exports
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_exports {
                if let Some(target_sym) = result_mut.exports.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.exports.insert(name, merged);
                }
            }

            let final_result = Arc::new(result_mut);

            if !unidirectional {
                self.record_merged_symbol(&final_result, source);
            }

            final_result
        } else {

            self.report_merge_symbol_error(target, source);
            Arc::clone(target)
        }
    }

    fn report_merge_symbol_error(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        let is_either_enum =
            target.flags.contains(SymbolFlags::ENUM) || source.flags.contains(SymbolFlags::ENUM);
        let is_either_block_scoped = target
            .flags
            .intersects(SymbolFlags::BlockScopedVariable)
            || source.flags.intersects(SymbolFlags::BlockScopedVariable);
        let message = if is_either_enum {
            crate::diagnostics::messages_generated::
                ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS
        } else if is_either_block_scoped {
            crate::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0
        };
        let name = source.name.clone();
        let mut locs: Vec<crate::core::text::TextRange> = Vec::new();
        for sym in [target, source] {
            for d in &sym.declarations {
                let name_node =
                    crate::ast::utilities::get_name_of_declaration(d).unwrap_or_else(|| Arc::clone(d));
                locs.push(name_node.loc);
            }
        }
        for loc in locs {

            if self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc == loc && d.code == message.code)
            {
                continue;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                vec![name.clone()],
            ));
        }
    }

    pub fn record_merged_symbol(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        self.merged_symbols.insert(source.id(), target.id());
    }

    pub fn clone_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let mut cloned = Symbol::new(symbol.flags | SymbolFlags::Transient, &symbol.name);
        cloned.declarations = symbol.declarations.clone();
        cloned.parent = symbol.parent.clone();
        cloned.value_declaration = symbol.value_declaration.clone();
        cloned.members = SymbolTable {
            entries: symbol.members.entries.clone(),
        };
        cloned.exports = SymbolTable {
            entries: symbol.exports.entries.clone(),
        };
        let result = Arc::new(cloned);
        Some(result)
    }

    pub fn resolve_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(result) = self.follow_alias(symbol) {
            result
        } else {
            Arc::clone(symbol)
        }
    }

    pub fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {

        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        if node.kind == crate::ast::SyntaxKind::Identifier {
            let mut current = node.parent.as_ref();
            while let Some(parent) = current {
                if let Some(sym) = self.program.symbol_map().symbol_of(parent) {
                    return Some(Arc::clone(sym));
                }
                current = parent.parent.as_ref();
            }
        }

        if node.kind == crate::ast::SyntaxKind::PropertyAccessExpression {
            if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                if let Some(links) = self.type_node_links.get(&data.expression) {
                    if let Some(ref t) = links.resolved_type {
                        return self.get_property_of_type(t, data.name.text());
                    }
                }
            }
        }

        None
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
        key.push(Arc::as_ptr(t) as *const Type as usize);
        key.extend(args.iter().map(|a| Arc::as_ptr(a) as *const Type as usize));
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
