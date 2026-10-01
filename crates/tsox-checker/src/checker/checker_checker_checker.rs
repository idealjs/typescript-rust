#![allow(unused_imports)]
use crate::checker::checker_checker::*;
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
    /// type_of_imported_symbol 访问中环守卫（Go symbolLinks 解析中缓存）
    pub imported_type_resolution: Vec<u64>,
    pub alias_type_resolution_stack: Vec<u64>,
    /// CS 实参部分代入后的回声推断域：此域内自引用候选（T←T）拦截生效
    pub cs_echo_inference: bool,
    /// 推断期实参检查的活动上下文型（node id → 类型）：Go
    /// checkExpressionWithContextualType 的 pushContextualType 等价物，
    /// 值为按当前推断候选代入后的参数型
    pub active_inferential_contextual: Option<(u64, std::sync::Arc<crate::checker::types::Type>)>,
    /// JSX 属性上下文型两阶段推断进行中标志（元素 node id → 当前阶段
    /// 参数型）：重入查询返回当前阶段参数型，阻断递归
    pub jsx_attr_ctx_guard: Option<(u64, std::sync::Arc<crate::checker::types::Type>)>,
    /// Go discriminatedContextualTypes（checker.go:643）：对象字面量按
    /// 成员判别收窄 memo（node id, contextual type id → 成功判别结果；
    /// Go nil 写入为 no-op，失败不驻留）
    pub discriminated_contextual_types:
        std::collections::HashMap<(u64, u32), std::sync::Arc<crate::checker::types::Type>>,
    pub language_version: ScriptTarget,
    pub module_kind: ModuleKind,
    pub module_resolution_kind: ModuleResolutionKind,
    pub legacy_decorators: bool,
    pub emit_standard_class_fields: bool,
    pub strict_null_checks: bool,
    pub allow_unreachable_code: tsox_core::core::tristate::Tristate,
    pub allow_unused_labels: tsox_core::core::tristate::Tristate,
    pub within_unreachable_code: bool,
    pub interface_build_depth: usize,
    pub reported_unreachable_nodes: std::collections::HashSet<usize>,
    /// this 引用位置诊断已报节点（Go checkThisExpression 单入口，Rust 分
    /// check/type 两入口，此集合保证位置类错误每节点只报一次）
    pub this_location_errors_reported: std::collections::HashSet<u64>,
    /// heritage 左端标识符已按 Namespace 含义预解析（Go resolveQualifiedName
    /// 左端 Namespace 解析 + symbolLinks 缓存），值位复检跳过 TS2708
    pub namespace_value_suppressed_nodes: std::collections::HashSet<u64>,
    pub strict_function_types: bool,
    pub strict_bind_call_apply: bool,
    pub strict_property_initialization: bool,
    pub strict_builtin_iterator_return: bool,
    pub no_implicit_any: bool,
    pub no_implicit_this: bool,
    pub use_unknown_in_catch_variables: bool,
    pub exact_optional_property_types: bool,
    pub no_unchecked_indexed_access: bool,
    pub can_collect_symbol_alias_accessibility_data: bool,
    pub globals: SymbolTable,
    pub undefined_symbol: Option<Arc<Symbol>>,
    pub arguments_symbol: Option<Arc<Symbol>>,
    pub require_symbol: Option<Arc<Symbol>>,
    pub unknown_symbol: Option<Arc<Symbol>>,
    pub global_this_symbol: Option<Arc<Symbol>>,
    pub string_literal_types: HashMap<String, Arc<Type>>,
    pub number_literal_types: HashMap<tsox_core::jsnum::Number, Arc<Type>>,
    pub bigint_literal_types: HashMap<String, Arc<Type>>,
    pub unique_es_symbol_types: HashMap<u64, Arc<Type>>,
    pub nan_type: Option<Arc<Type>>,
    pub indexed_access_types: HashMap<CacheHashKey, Arc<Type>>,
    pub template_literal_types: HashMap<CacheHashKey, Arc<Type>>,
    pub string_mapping_types: HashMap<u64, Arc<Type>>,
    pub intrinsic_marker_type: OnceLock<Arc<Type>>,
    pub marker_super_type: OnceLock<Arc<Type>>,
    pub marker_sub_type: OnceLock<Arc<Type>>,
    pub marker_other_type: OnceLock<Arc<Type>>,
    pub marker_types: std::collections::HashSet<u32>,
    pub variance_stack: Vec<usize>,
    pub reliability_flags: u8,
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
    pub class_instance_type_cache: HashMap<u64, Arc<Type>>,
    pub base_ctor_type_cache: HashMap<u32, Arc<Type>>,
    pub this_type_cache: HashMap<u64, Arc<Type>>,
    pub type_resolution_stack: Vec<TypeResolutionEntry>,
    pub variable_type_frame_depth: usize,
    pub signature_return_resolutions: Vec<(*const tsox_frontend::ast::Node, bool, bool)>,
    pub partial_fn_type_builds: std::collections::HashSet<*const tsox_frontend::ast::Node>,
    pub rt_infer_boundary_marks: Vec<usize>,
    pub call_return_query_depth: u32,
    pub callee_resolution_depth: u32,
    pub in_flight_object_literal_types: HashMap<u64, Arc<Type>>,
    pub type_argument_stack: Vec<HashMap<*const tsox_frontend::ast::Symbol, Arc<Type>>>,
    pub erase_signature_strict: bool,
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
    pub pending_jsx_2875: Option<(tsox_core::core::text::TextRange, String)>,
    pub relater_error_chain: Vec<RelaterChainEntry>,
    pub relater_excess_error_node: Option<Arc<tsox_frontend::ast::Node>>,
    pub relater_error_node: Option<Arc<tsox_frontend::ast::Node>>,
    pub relater_chain_active: bool,
    pub relater_no_common_self_reports: u32,
    pub relater_pending_primitive_source: bool,
    pub property_lookup_skips_index_synthesis: bool,
    pub relater_depth: u32,
    pub deferred_constraint_depth: u32,
    pub deferred_conditional_root_stack: Vec<u64>,
    pub relation_count: u32,
    pub relater_overflow: bool,
    pub relater_intersection_target_depth: u32,
    pub relation_reference_canon:
        HashMap<(usize, Vec<u32>), u32>,
    pub subst_object_in_progress: std::collections::HashMap<u32, Arc<crate::checker::types::Type>>,
    pub subst_reference_shell_cache:
        std::collections::HashMap<(usize, Vec<usize>), Arc<crate::checker::types::Type>>,
    pub in_return_substitution: bool,
    pub relater_source_stack: Vec<Arc<Type>>,
    pub relater_target_stack: Vec<Arc<Type>>,
    pub relation_cache: HashMap<crate::checker::relater::RelationCacheKey, bool>,
    pub probe_cache_permissive: HashMap<u32, Arc<Type>>,
    pub probe_cache_restrictive: HashMap<u32, Arc<Type>>,
    pub enum_relation: HashMap<EnumRelationKey, crate::checker::relater::RelationComparisonResult>,
    pub relation_in_progress: std::collections::HashSet<crate::checker::relater::RelationCacheKey>,
    pub relation_maybe_keys: Vec<crate::checker::relater::RelationCacheKey>,
    pub relation_maybe_key_set: std::collections::HashSet<crate::checker::relater::RelationCacheKey>,
    pub relater_bail_maybe: bool,
    pub new_call_fallback_signature: bool,
    pub interface_extends_reported: std::collections::HashSet<(
        *const tsox_frontend::ast::Symbol,
        *const tsox_frontend::ast::Node,
    )>,
    pub interface_simultaneous_reported: std::collections::HashSet<*const tsox_frontend::ast::Symbol>,
    pub indexed_access_2538_reported: std::collections::HashSet<*const tsox_frontend::ast::Node>,
    pub arith_operand_error_nodes: std::collections::HashSet<*const tsox_frontend::ast::Node>,
    pub computed_property_name_checked: std::collections::HashSet<*const tsox_frontend::ast::Node>,
    pub symbol_reference_kinds: dashmap::DashMap<u64, SymbolFlags>,
    pub spread_links: LinkStore<Symbol, SpreadLinks>,
    pub variance_links: LinkStore<Symbol, VarianceLinks>,
    pub reverse_mapped_symbol_links: LinkStore<Symbol, ReverseMappedSymbolLinks>,
    pub reverse_mapped_cache: HashMap<(u32, u32, u32), Option<Arc<Type>>>,
    pub pending_annotated_param_inferences: Vec<(Arc<Type>, Arc<Type>)>,
    pub inference_loop_depth: usize,
    pub generic_index_type_cache: HashMap<u32, bool>,
    pub could_contain_type_variables_cache: HashMap<u32, bool>,
    pub primitive_apparent_types: HashMap<&'static str, Arc<Type>>,
    pub template_resolving_ids: std::collections::HashSet<u32>,
    pub mapped_shell_resolving: std::collections::HashSet<u32>,
    pub template_resolution_letway: bool,
    pub deferred_indexed_access_cache: HashMap<(u32, u32), Arc<Type>>,
    pub index_type_cache: HashMap<u32, Arc<Type>>,
    pub interface_shell_reify_cache: HashMap<usize, Arc<Type>>,
    pub reverse_mapped_print_stack: Vec<Arc<Symbol>>,
    pub alias_args_resolution_stack: Vec<(usize, Vec<u32>)>,
    pub alias_type_instantiation_stack: Vec<(usize, Vec<u32>)>,
    pub infer_subst_ancestor_stack: Vec<usize>,
    pub infer_subst_memo: Vec<crate::checker::relater_probing_checker_4::InferSubstMemoFrame>,
    pub alias_instantiation_cache: HashMap<(usize, Vec<u32>), Arc<Type>>,
    pub current_alias_frame: Option<(usize, Vec<u32>)>,
    pub reverse_mapped_depth: Vec<u32>,
    pub reverse_mapped_target_depth: Vec<u32>,
    pub marked_assignment_symbol_links: LinkStore<Symbol, MarkedAssignmentSymbolLinks>,
    pub symbol_container_links: LinkStore<Symbol, ContainingSymbolLinks>,
    pub symbol_table_alias_cache: HashMap<u64, Vec<Arc<Symbol>>>,
    pub class_expression_name_tables: HashMap<u64, SymbolTable>,
    pub source_file_links: LinkStore<SourceFile, SourceFileLinks>,
    pub declaration_links: LinkStore<Node, DeclarationLinks>,
    pub declaration_file_links: LinkStore<SourceFile, DeclarationFileLinks>,
    pub(crate) last_combined_modifier_flags_node: Option<Arc<Node>>,
    pub(crate) last_combined_modifier_flags_result: ModifierFlags,
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
    pub silent_never_type: OnceLock<Arc<Type>>,
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
    pub typeof_type: OnceLock<Arc<Type>>,
    pub global_this_type: OnceLock<Arc<Type>>,
    pub global_promise_type: OnceLock<Arc<Type>>,
    pub array_type_cache: std::collections::HashMap<(usize, usize), Arc<Type>>,
    pub interface_instantiation_cache:
        std::collections::HashMap<Vec<usize>, (Vec<Arc<Type>>, Arc<Type>)>,
    pub merged_ns_instance_type_cache: std::collections::HashMap<u64, Arc<Type>>,
    pub pending_interface_shells: std::collections::HashMap<usize, Arc<Type>>,
    pub pending_arg_shells: std::collections::HashMap<Vec<usize>, (Arc<Type>, u64)>,
    pub arg_shell_seq: u64,
    pub arg_shell_resolves: u64,
    pub attached_type_args_cache:
        std::collections::HashMap<Vec<usize>, (Arc<Type>, Vec<Arc<Type>>, Arc<Type>)>,
    pub filling_class_members: std::collections::HashSet<u64>,
    pub class_build_in_progress: std::collections::HashSet<u64>,
    pub pending_base_merges: Vec<(u64, Arc<Type>)>,
    pub typequery_instantiation_cache:
        std::collections::HashMap<Vec<usize>, (Vec<Arc<Type>>, Arc<Type>)>,
    pub fn_typequery_shells: std::collections::HashMap<usize, Arc<Type>>,
    pub fn_typequery_shell_ptrs: std::collections::HashSet<usize>,
    pub array_type_parameter_symbols: Option<Vec<Arc<tsox_frontend::ast::Symbol>>>,
    pub array_member_type_cache: std::collections::HashMap<(usize, usize), Arc<Type>>,
    pub array_type_intern_cache: std::collections::HashMap<(u32, bool), Arc<Type>>,
    pub intersection_intern_cache: std::collections::HashMap<Vec<TypeId>, Arc<Type>>,
    pub instantiated_member_type_cache: std::collections::HashMap<
        (usize, usize),
        (Arc<Type>, Arc<tsox_frontend::ast::Symbol>, Arc<Type>),
    >,
    pub instantiated_member_type_cache_limit: usize,
    pub instantiated_member_owner: std::collections::HashMap<usize, u64>,
    pub any_signature: OnceLock<Arc<Signature>>,
    pub unknown_signature: OnceLock<Arc<Signature>>,
    pub resolving_signature: OnceLock<Arc<Signature>>,
    pub current_node: Option<Arc<Node>>,
    pub inline_level: i32,
    pub serialization_level: i32,
    pub type_print_stack: Vec<usize>,
    pub display_approximate_length: usize,
    pub display_truncating: bool,
    pub current_file: Option<Arc<SourceFile>>,
    pub current_file_id: u64,
    pub display_enclosing_file: Option<Arc<SourceFile>>,
    pub module_display_specifiers: std::collections::HashMap<u64, String>,
    pub display_enclosing_node: Option<Arc<Node>>,
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
    pub(crate) ambient_statement_reported: std::collections::HashSet<u64>,
    pub namespace_value_depth: u8,
    pub accessor_pair_return_hint: Option<Arc<Type>>,
    pub call_arg_arrow_context: Vec<usize>,
    pub resolving_type_aliases: std::collections::HashSet<*const Symbol>,
    pub alias_resolution_stack: Vec<Arc<Symbol>>,
    pub alias_circular_frames: std::collections::HashSet<*const Symbol>,
    pub alias_circular_reported: std::collections::HashSet<u64>,
    pub resolving_function_like: std::collections::HashSet<u64>,
    pub class_statics_resolution_stack: Vec<u64>,
    pub class_type_resolution_stack: Vec<u64>,
    pub inference_constraint_in_flight: Vec<u64>,
    pub resolving_contextual_calls: std::collections::HashSet<u64>,
    pub logical_rhs_narrowing_frames: Vec<(Arc<Symbol>, Arc<Type>)>,
    pub in_ctor_body_stack: Vec<bool>,
    pub return_type_stack: Vec<Option<Arc<Type>>>,
    pub flow_analysis_disabled: bool,
    pub definite_assignment_check_depth: u32,
    pub flow_invocation_count: i32,
    pub flow_type_cache: HashMap<u64, Arc<Type>>,
    pub union_or_intersection_property_cache: HashMap<(u64, String), Arc<Symbol>>,
    pub flow_node_reachable: HashMap<u64, bool>,
    pub switch_exhaustive_state: HashMap<u64, u8>,
    pub type_instantiation_count: u64,
    pub type_instantiation_limit_reported: bool,
    pub flow_inline_level: u32,
    pub in_static_member_type: bool,
    pub suppress_cannot_find_name_in_type_nodes: u32,
    pub suppress_source_file: Option<u64>,
    pub tracer: Arc<Tracer>,
    pub narrowable_reference_query_stack: Vec<u64>,
    pub binding_pattern_narrowing_stack: Vec<u64>,
    pub merged_symbols: HashMap<u64, u64>,
    pub merged_symbol_targets: HashMap<u64, Arc<tsox_frontend::ast::Symbol>>,
    pub awaited_type_stack: Vec<Arc<Type>>,
    pub flow_loop_stack: Vec<FlowLoopInfo>,
    pub flow_loop_cache: HashMap<FlowLoopKey, Arc<Type>>,
    pub cached_signatures: HashMap<CachedSignatureKey, Arc<Signature>>,
    pub substitution_types: HashMap<SubstitutionTypeKey, Arc<Type>>,
    pub undefined_properties: HashMap<String, Arc<Symbol>>,
    pub this_expando_kinds: HashMap<u64, crate::checker::mig::wc3::ThisAssignmentDeclarationKind>,
    pub this_expando_locations: HashMap<u64, Option<Arc<Node>>>,
    pub conditional_constraint_depth: u32,
    pub contextual_infos: Vec<crate::checker::mig::m2b_2::ContextualInfo>,
    pub inference_context_infos: Vec<crate::checker::mig::m2b_2::InferenceContextInfo>,
    pub active_mappers: Vec<Arc<crate::checker::types_impl_chunk::TypeMapper>>,
    pub active_type_mappers_caches: Vec<HashMap<CacheHashKey, Arc<Type>>>,
    pub deferred_diagnostic_callbacks: Vec<Box<dyn FnOnce() + Send>>,
    pub type_resolutions: Vec<TypeResolution>,
    pub resolution_start: usize,
    pub factory: tsox_frontend::ast::mig::m3c::NodeFactory,
    pub wildcard_type: Arc<Type>,
    pub blocked_string_type: Arc<Type>,
    pub missing_type: Arc<Type>,
    pub unique_literal_type: Arc<Type>,
    pub unreachable_never_type: Arc<Type>,
    pub regular_false_type: Arc<Type>,
    pub regular_true_type: Arc<Type>,
    pub undefined_widening_type: Arc<Type>,
    pub null_widening_type: Arc<Type>,
    pub number_or_big_int_type: Arc<Type>,
    pub string_number_symbol_type: Arc<Type>,
    pub marker_sub_type_for_check: Option<Arc<Type>>,
    pub marker_super_type_for_check: Option<Arc<Type>>,
    pub variance_type_parameter: Option<Arc<Type>>,
    pub packages_map: Option<HashMap<String, bool>>,
    pub mu: Mutex<()>,
}

unsafe impl Send for Checker {}
unsafe impl Sync for Checker {}

pub struct CachedSignatureKey {
    pub sig: Arc<Signature>,
    pub key: CacheHashKey,
}

impl PartialEq for CachedSignatureKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.sig, &other.sig) && self.key == other.key
    }
}
impl Eq for CachedSignatureKey {}
impl std::hash::Hash for CachedSignatureKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.sig).hash(state);
        self.key.hash(state);
    }
}

#[allow(non_snake_case)]
pub mod SignatureKey {
    use crate::checker::types_cached_type_kind::CacheHashKey;

    pub const Erased: CacheHashKey = CacheHashKey { hi: u64::MAX, lo: 1 };
    pub const Canonical: CacheHashKey = CacheHashKey { hi: u64::MAX, lo: 2 };
    pub const Base: CacheHashKey = CacheHashKey { hi: u64::MAX, lo: 3 };
    pub const Inner: CacheHashKey = CacheHashKey { hi: u64::MAX, lo: 4 };
    pub const Outer: CacheHashKey = CacheHashKey { hi: u64::MAX, lo: 5 };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubstitutionTypeKey {
    pub base_id: u32,
    pub constraint_id: u32,
}

pub struct FlowLoopKey {
    pub flow_node: Arc<tsox_frontend::ast::FlowNode>,
    pub ref_key: CacheHashKey,
}

impl PartialEq for FlowLoopKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.flow_node, &other.flow_node) && self.ref_key == other.ref_key
    }
}
impl Eq for FlowLoopKey {}
impl std::hash::Hash for FlowLoopKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.flow_node).hash(state);
        self.ref_key.hash(state);
    }
}

pub struct FlowLoopInfo {
    pub key: FlowLoopKey,
    pub types: Vec<Arc<Type>>,
}

pub enum TypeSystemEntity {
    Type(Arc<Type>),
    Symbol(Arc<Symbol>),
    Signature(Arc<Signature>),
}

impl PartialEq for TypeSystemEntity {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeSystemEntity::Type(a), TypeSystemEntity::Type(b)) => Arc::ptr_eq(a, b),
            (TypeSystemEntity::Symbol(a), TypeSystemEntity::Symbol(b)) => Arc::ptr_eq(a, b),
            (TypeSystemEntity::Signature(a), TypeSystemEntity::Signature(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for TypeSystemEntity {}

pub struct TypeResolution {
    pub target: TypeSystemEntity,
    pub property_name: crate::checker::types_alias_symbol_links::TypeSystemPropertyName,
    pub result: bool,
}

impl Checker {
    pub(crate) fn unknown_symbol(&self) -> Arc<Symbol> {
        self.unknown_symbol.clone().expect("unknown_symbol not initialized")
    }

    pub(crate) fn require_symbol(&self) -> Option<Arc<Symbol>> {
        self.require_symbol.clone()
    }

    pub(crate) fn current_node(&self) -> Option<Arc<Node>> {
        self.current_node.clone()
    }

    pub(crate) fn any_signature(&self) -> Arc<Signature> {
        self.any_signature.get().cloned().expect("any_signature not initialized")
    }

    pub(crate) fn global_function_type(&self) -> Arc<Type> {
        self.global_function_type.get().cloned().expect("global_function_type not initialized")
    }

    pub(crate) fn missing_type(&self) -> Arc<Type> {
        Arc::clone(&self.missing_type)
    }

    pub(crate) fn null_widening_type(&self) -> Arc<Type> {
        Arc::clone(&self.null_widening_type)
    }

    pub(crate) fn unique_literal_type(&self) -> Arc<Type> {
        Arc::clone(&self.unique_literal_type)
    }

    pub(crate) fn regular_false_type(&self) -> Arc<Type> {
        Arc::clone(&self.regular_false_type)
    }

    pub(crate) fn number_or_big_int_type(&self) -> Arc<Type> {
        Arc::clone(&self.number_or_big_int_type)
    }
}
