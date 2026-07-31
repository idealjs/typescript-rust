//! The type checker.
//!
//! Ported from `internal/checker/checker.go`. This is the largest and most
//! complex module in the compiler (~32K lines in Go). This file provides
//! the `Checker` struct, its initialization, and the core entry points.
//! Full type-checking logic is added incrementally.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::ast::{
    CheckFlags, DiagnosticsCollection, ModifierFlags, Node, NodeData, NodeFlags, NodeList,
    NodeSymbolMap, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::core::text::TextRange;
use crate::diagnostics::messages_generated::{
    A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER,
    ARGUMENT_EXPRESSION_EXPECTED, ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1,
    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY, CANNOT_FIND_NAME_0,
    EXPECTED_0_ARGUMENTS_BUT_GOT_1, EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1,
    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
    THIS_COMPARISON_APPEARS_TO_BE_UNINTENTIONAL_BECAUSE_THE_TYPES_0_AND_1_HAVE_NO_OVERLAP,
    THIS_EXPRESSION_IS_NOT_CALLABLE, THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE,
    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
};
use crate::jsnum;

use super::tracer::Tracer;
use super::types::*;

// ────────────────────────────────────────────────────────────────────────────
// Program trait (simplified)
// ────────────────────────────────────────────────────────────────────────────

/// A simplified version of the Go `checker.Program` interface.
///
/// The full interface has many more methods; this provides the minimum
/// needed by the checker.
pub trait Program: Send + Sync {
    fn options(&self) -> &CompilerOptions;
    fn source_files(&self) -> &[Arc<SourceFile>];
    fn bind_source_files(&self);
    fn file_exists(&self, file_name: &str) -> bool;
    fn get_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>>;
    fn is_source_file_default_library(&self, path: &str) -> bool;
    /// Side table from the binder (symbols, locals, flow nodes), shared
    /// across all source files in the program.
    fn symbol_map(&self) -> &NodeSymbolMap;
}

// ────────────────────────────────────────────────────────────────────────────
// LinkStore (side table for node/symbol links)
// ────────────────────────────────────────────────────────────────────────────

/// A side table mapping from `Arc<T>` to associated data `V`.
///
/// In Go, the checker uses `core.LinkStore` with `*ast.Node` and `*ast.Symbol`
/// keys (raw pointers). In Rust, we use the object's ID as the key.
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

    /// Get the links for a key, creating default if not present.
    pub fn get_or_default(&mut self, key: &K) -> &mut V {
        self.data.entry(key.id()).or_default()
    }

    /// Get the links for a key, if present.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(&key.id())
    }

    /// Get the links for a key mutably, if present.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(&key.id())
    }

    /// Set the links for a key.
    pub fn insert(&mut self, key: &K, value: V) {
        self.data.insert(key.id(), value);
    }
}

/// Trait for objects that have a unique ID.
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

// ────────────────────────────────────────────────────────────────────────────
// Checker
// ────────────────────────────────────────────────────────────────────────────

static NEXT_CHECKER_ID: AtomicU32 = AtomicU32::new(1);

/// Kind of break/continue control-flow context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakContinueContextKind {
    /// `for`, `while`, `do-while`, `for-in`, `for-of`.
    Loop,
    /// `switch` statement.
    Switch,
    /// Function-like boundary (break/continue cannot cross).
    Function,
    /// Labeled statement (stores the label text).
    Labeled,
}

/// A break/continue context entry on the checker's context stack.
#[derive(Debug, Clone)]
pub struct BreakContinueContext {
    pub kind: BreakContinueContextKind,
    /// Label text for `Labeled` entries.
    pub label: Option<String>,
    /// Whether the labeled statement's body is an iteration (for `continue`).
    pub is_iteration: bool,
}

/// The type checker. This is the core of the TypeScript compiler.
///
/// In Go, this struct has ~320 fields. The Rust port organizes them into
/// logical groups while preserving the same semantics.
pub struct Checker {
    // Identity
    pub id: u32,

    // Program and options
    pub program: Arc<dyn Program>,
    pub compiler_options: Arc<CompilerOptions>,
    pub files: Vec<Arc<SourceFile>>,
    pub file_index_map: HashMap<u64, usize>,

    // Counters
    pub type_count: u32,
    pub symbol_count: u32,
    pub signature_count: u32,
    pub total_instantiation_count: u32,
    pub instantiation_count: u32,
    pub instantiation_depth: u32,

    // Configuration flags (derived from compiler options)
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

    // Global symbols
    pub globals: SymbolTable,
    pub undefined_symbol: Option<Arc<Symbol>>,
    pub arguments_symbol: Option<Arc<Symbol>>,
    pub require_symbol: Option<Arc<Symbol>>,
    pub unknown_symbol: Option<Arc<Symbol>>,
    pub global_this_symbol: Option<Arc<Symbol>>,

    // Literal type caches
    pub string_literal_types: HashMap<String, Arc<Type>>,
    pub number_literal_types: HashMap<jsnum::Number, Arc<Type>>,
    pub bigint_literal_types: HashMap<String, Arc<Type>>,
    pub unique_es_symbol_types: HashMap<u64, Arc<Type>>,
    pub nan_type: Option<Arc<Type>>,

    // Type caches
    pub indexed_access_types: HashMap<CacheHashKey, Arc<Type>>,
    pub template_literal_types: HashMap<CacheHashKey, Arc<Type>>,
    pub string_mapping_types: HashMap<u64, Arc<Type>>,
    pub cached_types: HashMap<CachedTypeKey, Arc<Type>>,
    pub union_types: HashMap<CacheHashKey, Arc<Type>>,
    pub intersection_types: HashMap<CacheHashKey, Arc<Type>>,
    pub tuple_types: HashMap<CacheHashKey, Arc<Type>>,
    pub error_types: HashMap<CacheHashKey, Arc<Type>>,

    // Diagnostics
    pub diagnostics: DiagnosticsCollection,
    pub suggestion_diagnostics: DiagnosticsCollection,

    // Link stores (side tables for nodes and symbols)
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
    /// Symbols (by raw pointer identity) whose declared type is currently
    /// being resolved, to break cycles in recursive type aliases
    /// (e.g. `type A = B; type B = A`).
    pub resolving_type_aliases: HashSet<*const Symbol>,
    /// Stack of type-parameter → type-argument substitutions, used when
    /// instantiating a generic type alias (e.g. `T<number[]>` where
    /// `type T<U> = ...`). Each frame maps type-parameter symbol pointers
    /// to their concrete type arguments. When `resolve_type_reference`
    /// encounters a type parameter that's in the current frame, it returns
    /// the mapped type argument instead of the `TypeParameter` type.
    pub type_argument_stack: Vec<HashMap<*const crate::ast::Symbol, Arc<Type>>>,
    /// Recursion depth of `is_type_related_to`. Capped at
    /// `RELATER_MAX_DEPTH` to prevent stack overflow on recursive
    /// structural types such as `type Box<T> = { next: Box<T> | null }`.
    /// Mirrors `Checker.relationStackDepth` in Go (relater.go).
    pub relater_depth: u32,
    /// Per-call relation comparison cache. Stores the final boolean
    /// result of `is_type_related_to` for a `(source, target, relation)`
    /// triple so that repeated sub-comparisons within a single top-level
    /// call don't recompute. Cleared at the start of each top-level call
    /// (when `relater_depth` transitions from 0 to 1) to avoid caching
    /// optimistic cycle-broken results across calls.
    /// Mirrors Go's `Relation.results` (relater.go).
    pub relation_cache: HashMap<crate::checker::relater::RelationCacheKey, bool>,
    /// Set of `(source, target, relation)` triples currently being
    /// computed higher up the relater call stack. When a triple is
    /// already in this set, we've hit a recursive cycle (e.g.
    /// `type Box<T> = { next: Box<T> | null }` comparing `Box<X>` to
    /// `Box<Y>` recursively reaches `Box<X>` vs `Box<Y>` again) and we
    /// optimistically return `true` to break the cycle.
    /// Mirrors Go's `visited` set pattern in `structuredTypeRelatedTo`.
    pub relation_in_progress: std::collections::HashSet<crate::checker::relater::RelationCacheKey>,
    pub spread_links: LinkStore<Symbol, SpreadLinks>,
    pub variance_links: LinkStore<Symbol, VarianceLinks>,
    pub reverse_mapped_symbol_links: LinkStore<Symbol, ReverseMappedSymbolLinks>,
    pub marked_assignment_symbol_links: LinkStore<Symbol, MarkedAssignmentSymbolLinks>,
    pub symbol_container_links: LinkStore<Symbol, ContainingSymbolLinks>,
    pub source_file_links: LinkStore<SourceFile, SourceFileLinks>,
    /// Per-declaration emit-resolver links (caches `isDeclarationVisible`).
    pub declaration_links: LinkStore<Node, DeclarationLinks>,
    /// Per-source-file emit-resolver links (tracks `aliasesMarked`).
    pub declaration_file_links: LinkStore<SourceFile, DeclarationFileLinks>,
    /// Single-entry cache for `get_combined_modifier_flags`, mirroring Go's
    /// `lastGetCombinedModifierFlagsNode`/`Result`.
    last_combined_modifier_flags_node: Option<Arc<Node>>,
    last_combined_modifier_flags_result: ModifierFlags,

    // Built-in types
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

    // Special "auto" types used for evolving array flow analysis.
    // `auto_type` is a special `any` used as a marker for uninitialized
    // variables (mirrors Go's `autoType`). `auto_array_type` is `any[]`
    // used as a marker for empty-array initializers (mirrors Go's
    // `autoArrayType`). Both are replaced by concrete types during flow
    // analysis.
    pub auto_type: OnceLock<Arc<Type>>,

    // Object types
    pub empty_object_type: OnceLock<Arc<Type>>,
    pub empty_generic_type: OnceLock<Arc<Type>>,
    pub any_function_type: OnceLock<Arc<Type>>,
    pub no_constraint_type: OnceLock<Arc<Type>>,
    pub circular_constraint_type: OnceLock<Arc<Type>>,

    // Array types
    pub any_array_type: OnceLock<Arc<Type>>,
    pub auto_array_type: OnceLock<Arc<Type>>,
    pub any_readonly_array_type: OnceLock<Arc<Type>>,

    // Global types (resolved lazily from lib.d.ts)
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

    // Signatures
    pub any_signature: OnceLock<Arc<Signature>>,
    pub unknown_signature: OnceLock<Arc<Signature>>,
    pub resolving_signature: OnceLock<Arc<Signature>>,

    // Current state
    pub current_node: Option<Arc<Node>>,
    pub inline_level: i32,
    pub serialization_level: i32,
    /// The source file currently being checked.
    pub current_file: Option<Arc<SourceFile>>,
    /// The node ID of the source file currently being checked.
    pub current_file_id: u64,
    /// The symbol of the source file currently being checked (top-level
    /// declarations land in its `members`).
    pub current_file_symbol: Option<Arc<Symbol>>,
    /// Stack of container node IDs (functions, blocks, etc.) used for
    /// identifier resolution when parent pointers are not available.
    pub scope_stack: Vec<u64>,
    /// Number of nested function scopes (not arrow functions) currently being checked.
    /// Used to determine whether `arguments` is in scope.
    pub function_scope_count: usize,
    /// Number of nested arrow function scopes. Arrow functions do not have
    /// their own `arguments` object.
    pub arrow_function_scope_count: usize,
    /// Whether globals have been populated from source file symbols.
    pub globals_populated: bool,
    /// Stack of break/continue contexts (loops, switches, functions, labels).
    /// Used by `check_grammar_break_or_continue_statement` since parent
    /// pointers are not set on nodes.
    pub break_continue_context_stack: Vec<BreakContinueContext>,

    /// Stack of `this` types for class member checking. When checking a
    /// class declaration's members, the class's instance type (including
    /// inherited members from `extends`) is pushed here so that `this.prop`
    /// inside a method body resolves correctly. Mirrors Go's
    /// `getThisTypeOfObjectLiteral`/`getThisType` infrastructure.
    pub this_type_stack: Vec<Arc<Type>>,

    /// Stack of declared return types for the enclosing function. When
    /// checking a `return expr;` statement, `expr`'s type is compared
    /// against the top of this stack (the function's declared return
    /// type). `None` entries mean the function has no explicit return-type
    /// annotation (inferred). Mirrors Go's `expectedReturn` tracking in
    /// `checkReturnStatement`/`checkFunctionExpressionBody`.
    pub return_type_stack: Vec<Option<Arc<Type>>>,

    // Flow analysis
    pub flow_analysis_disabled: bool,
    pub flow_invocation_count: i32,
    pub flow_type_cache: HashMap<u64, Arc<Type>>,
    pub flow_node_reachable: HashMap<u64, bool>,
    /// Inlining depth for const-variable alias narrowing. Mirrors Go's
    /// `Checker.inlineLevel` (capped at 5). Incremented while narrowing
    /// through a `const` alias's initializer to prevent infinite recursion.
    pub flow_inline_level: u32,

    // Set while resolving the type annotation of a static class member.
    // When true, resolving a class type parameter reports TS2322, mirroring
    // Go's NameResolver check `ast.IsStatic(lastLocation)` →
    // `Static_members_cannot_reference_class_type_parameters`.
    pub in_static_member_type: bool,

    // Tracer
    pub tracer: Arc<Tracer>,

    // Merged symbols tracking (declaration merging)
    pub merged_symbols: HashMap<u64, u64>,

    // Mutex for thread safety
    pub mu: Mutex<()>,
}

impl Checker {
    /// Create a new checker.
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
            resolving_type_aliases: HashSet::new(),
            type_argument_stack: Vec::new(),
            relater_depth: 0,
            relation_cache: HashMap::new(),
            relation_in_progress: std::collections::HashSet::new(),
            spread_links: LinkStore::new(),
            variance_links: LinkStore::new(),
            reverse_mapped_symbol_links: LinkStore::new(),
            marked_assignment_symbol_links: LinkStore::new(),
            symbol_container_links: LinkStore::new(),
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

            any_signature: OnceLock::new(),
            unknown_signature: OnceLock::new(),
            resolving_signature: OnceLock::new(),

            current_node: None,
            inline_level: 0,
            serialization_level: 0,
            current_file: None,
            current_file_id: 0,
            current_file_symbol: None,
            scope_stack: Vec::new(),
            function_scope_count: 0,
            arrow_function_scope_count: 0,
            globals_populated: false,
            break_continue_context_stack: Vec::new(),
            this_type_stack: Vec::new(),
            return_type_stack: Vec::new(),

            flow_analysis_disabled: false,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            flow_node_reachable: HashMap::new(),
            flow_inline_level: 0,
            in_static_member_type: false,

            merged_symbols: HashMap::new(),

            tracer,
            mu: Mutex::new(()),
        };

        // Initialize global_this_symbol and add built-in symbols to globals
        {
            // Create globalThis symbol
            let mut global_this = Symbol::new(SymbolFlags::ValueModule, "globalThis");
            global_this.check_flags = CheckFlags::Readonly;
            let global_this = Arc::new(global_this);
            checker
                .globals
                .insert("globalThis".to_string(), Arc::clone(&global_this));
            checker.global_this_symbol = Some(global_this);

            // Add undefined to globals
            if let Some(ref undef) = checker.undefined_symbol {
                checker
                    .globals
                    .insert("undefined".to_string(), Arc::clone(undef));
            }
        }

        checker
    }

    /// Populate globals from source file symbols.
    fn populate_globals(&mut self) {
        for file in &self.files {
            // Look up the source file's symbol from the symbol map
            let symbol_map = self.program.symbol_map();
            if let Some(file_sym) = symbol_map.symbol_of(&file.node) {
                // Merge the source file's members into globals
                for (name, sym) in file_sym.members.iter() {
                    self.globals.insert(name.clone(), Arc::clone(sym));
                }
                // Also merge the source file's locals
                if let Some(locals) = symbol_map.locals_of(&file.node) {
                    for (name, sym) in locals.iter() {
                        self.globals.insert(name.clone(), Arc::clone(sym));
                    }
                }
            }
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Built-in type accessors
    // ────────────────────────────────────────────────────────────────────────

    /// Get the `any` type.
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

    /// Get the `unknown` type.
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

    /// Get the `undefined` type.
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

    /// Get the `null` type.
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

    /// Get the `string` type.
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

    /// Get the `number` type.
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

    /// Get the `bigint` type.
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

    /// Get the `boolean` type.
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

    /// Get the `symbol` type.
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

    /// Get the `void` type.
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

    /// Get the `never` type.
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

    /// Get the `auto` type — a special `any` with `NonInferrableType` flag,
    /// used as a marker for evolving-array analysis. Mirrors Go's `autoType`
    /// (`c.autoType = c.newIntrinsicTypeEx(TypeFlagsAny, "any", ObjectFlagsNonInferrableType)`).
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

    /// Get the `auto array type` — `any[]` used as a marker for empty array
    /// literal initializers (`let x = []`). Mirrors Go's `autoArrayType`
    /// (`c.createArrayType(c.autoType)`).
    pub fn auto_array_type(&mut self) -> Arc<Type> {
        if let Some(t) = self.auto_array_type.get() {
            return Arc::clone(t);
        }
        let auto = self.auto_type();
        let arr = self.create_array_type(auto);
        // Race-safe set: if another thread won, use its value.
        self.auto_array_type
            .set(arr.clone())
            .ok()
            .map(|()| arr.clone())
            .unwrap_or_else(|| self.auto_array_type.get().cloned().unwrap_or(arr))
    }

    /// Create an evolving array type with the given element type.
    ///
    /// Mirrors Go's `getEvolvingArrayType` (flow.go ~L1488). Evolving array
    /// types are used in flow analysis to track arrays whose element type
    /// is being inferred from `push`/`unshift` calls (e.g.
    /// `let x = []; x.push(1)` → `x: number[]`).
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

    /// Evolve an evolving array type by adding a new element type.
    ///
    /// Mirrors Go's `addEvolvingArrayElementType` (flow.go ~L1536). If the
    /// new element type is already a subset of the current element type,
    /// returns the original type unchanged. Otherwise, unions the new
    /// element type with the existing one.
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
                // If the new type is a subset of the current type, no change.
                if self.is_type_subset_of(&new_element_type, &current) {
                    return Arc::clone(evolving_type);
                }
                let union = self.get_union_type(vec![current, new_element_type]);
                self.get_evolving_array_type(union)
            }
            None => self.get_evolving_array_type(new_element_type),
        }
    }

    /// Finalize an evolving array type into a concrete array type.
    ///
    /// Mirrors Go's `finalizeEvolvingArrayType` (flow.go ~L1545). If the
    /// type is not an evolving array, returns it unchanged. If it is,
    /// converts it to `T[]` where `T` is the element type (or `any[]` if
    /// the element type is `never`).
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
                    // For union element types, use subtype reduction.
                    self.create_array_type(element)
                } else {
                    self.create_array_type(element)
                };
                // Cache on the evolving type via interior mutability.
                if let TypeData::EvolvingArray(ea) = &t.data {
                    let _ = ea.final_array_type.set(Arc::clone(&result));
                }
                result
            }
            _ => Arc::clone(t),
        }
    }

    /// Check if `a` is a subset of `b` (all members of `a` are assignable to
    /// `b`). Mirrors Go's `isTypeSubsetOf`. Used by evolving-array element
    /// evolution to avoid adding redundant types.
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
        // Simple subset check: a is assignable to b.
        self.is_type_assignable_to(a, b)
    }

    /// Get the `(...args: any) => any` "top function" wildcard type.
    ///
    /// Mirrors Go's `anyFunctionType`. In the relater, a function-type
    /// wildcard source is assignable to every other function type; a
    /// non-wildcard source is *not* assignable to a wildcard target.
    /// The type is represented as an empty anonymous object — the relater
    /// short-circuits on `Arc::ptr_eq` before ever consulting its
    /// signatures, so we don't need to populate them.
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

    /// Get the `object` type (non-primitive).
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

    /// Get the `true` literal type.
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

    /// Get the `false` literal type.
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

    /// Get the `error` type.
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

    // ────────────────────────────────────────────────────────────────────────
    // String literal type creation
    // ────────────────────────────────────────────────────────────────────────

    /// Get or create a string literal type.
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

    /// Get or create a number literal type.
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

    // ────────────────────────────────────────────────────────────────────────
    // Fresh literal types
    // ────────────────────────────────────────────────────────────────────────

    /// Get the fresh variant of a literal type. Mirrors Go's
    /// `getFreshTypeOfLiteralType` (checker.go:25195).
    ///
    /// For freshable literals (`TYPE_FLAGS_FRESHABLE = Enum | Literal`),
    /// lazily creates and caches a fresh variant on the regular type:
    ///
    /// - Regular literal type: `fresh_type` = Some(fresh variant),
    ///   `regular_type` = None.
    /// - Fresh literal type: `fresh_type` = None (empty),
    ///   `regular_type` = Some(regular type).
    ///
    /// This inverted representation avoids the need for a self-referential
    /// `Arc` on the fresh variant. `is_fresh_literal_type` checks
    /// `regular_type.is_some()`.
    ///
    /// For non-freshable types, returns the input unchanged.
    pub fn get_fresh_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        // Only freshable types (literals and enums) get fresh variants.
        if !t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            return Arc::clone(t);
        }
        let lit = match &t.data {
            TypeData::Literal(lit) => lit,
            _ => {
                // Enums and other freshable non-literal types are not
                // handled in Phase 1.
                return Arc::clone(t);
            }
        };
        // If `regular_type` is set, this IS already a fresh variant.
        if lit.regular_type.get().is_some() {
            return Arc::clone(t);
        }
        // Extract the value up front so the closure can be `move` and
        // avoid borrowing `lit` (which would conflict with the
        // `&lit.fresh_type` borrow held by `get_or_init`).
        let value = lit.value.clone();
        let flags = t.flags;
        let regular = Arc::clone(t);
        let fresh = lit.fresh_type.get_or_init(move || {
            Arc::new(Type::new(
                flags,
                TypeData::Literal(LiteralTypeData {
                    value,
                    // Fresh variant has no fresh_type of its own.
                    fresh_type: OnceLock::new(),
                    // Points back to the regular type.
                    regular_type: OnceLock::from(regular),
                }),
            ))
        });
        Arc::clone(fresh)
    }

    /// Get the widened type of a fresh literal type. Mirrors Go's
    /// `getWidenedLiteralType` (checker.go:25395).
    ///
    /// Widens ONLY fresh literals to their primitive base:
    /// - StringLiteral + fresh → `string`
    /// - NumberLiteral + fresh → `number`
    /// - BigIntLiteral + fresh → `bigint`
    /// - BooleanLiteral + fresh → `boolean`
    /// - Enum + fresh → enum base type (skipped in Phase 1)
    /// - Union → widen each constituent
    /// - Otherwise → return t unchanged
    pub fn get_widened_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        // Fresh literals widen to their primitive base.
        if crate::checker::is_fresh_literal_type(t) {
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
            // Enum widening is skipped for Phase 1. Fall through.
        }
        // Unions: widen each constituent recursively.
        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_widened_literal_type(member))
                .collect();
            // Avoid allocating a new union if nothing changed.
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

    /// Get the regular (non-fresh) type of a literal type. Mirrors Go's
    /// `getRegularTypeOfLiteralType` (checker.go:25181).
    ///
    /// For freshable literals, returns the `regular_type` field if set
    /// (i.e. if `t` is a fresh variant). For unions, maps over
    /// constituents. Otherwise returns `t` unchanged.
    pub fn get_regular_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TYPE_FLAGS_FRESHABLE) {
            if let TypeData::Literal(lit) = &t.data {
                if let Some(regular) = lit.regular_type.get() {
                    return Arc::clone(regular);
                }
            }
        }
        // Unions: map over constituents.
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

    /// Get the widened literal type for a variable initializer, gated on
    /// whether the declaration is `const`. Mirrors Go's
    /// `getWidenedLiteralTypeForInitializer` (checker.go:16810).
    ///
    /// For `const` declarations (NodeFlags::Constant), fresh literals are
    /// preserved (no widening). For `let`/`var`, fresh literals widen to
    /// their primitive base via `get_widened_literal_type`.
    pub fn get_widened_literal_type_for_initializer(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
    ) -> Arc<Type> {
        // `NodeFlags::Constant = Const | Using` is a composite flag, so use
        // `intersects` (any bit set) rather than `contains` (all bits set) —
        // a plain `const` declaration only carries `Const`, not `Using`.
        // Mirrors Go's `flags & NodeFlagsConstant != 0`.
        if self
            .get_combined_node_flags(declaration)
            .intersects(NodeFlags::Constant)
        {
            return Arc::clone(t);
        }
        self.get_widened_literal_type(t)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Diagnostics
    // ────────────────────────────────────────────────────────────────────────

    /// Get the diagnostics collected by the checker.
    pub fn get_diagnostics(&self) -> &DiagnosticsCollection {
        &self.diagnostics
    }

    /// Get suggestion diagnostics.
    pub fn get_suggestion_diagnostics(&self) -> &DiagnosticsCollection {
        &self.suggestion_diagnostics
    }

    // ────────────────────────────────────────────────────────────────────────
    // Node flags helpers
    // ────────────────────────────────────────────────────────────────────────

    /// Get combined node flags (caching the result).
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

    /// Get combined modifier flags (caching the result).
    ///
    /// Mirrors Go's `Checker.getCombinedModifierFlagsCached` →
    /// `ast.GetCombinedModifierFlags`. Walks from the root declaration up
    /// through `VariableDeclaration` → `VariableDeclarationList` →
    /// `VariableStatement`, OR-ing the syntactic modifier flags of each.
    pub fn get_combined_modifier_flags(&mut self, node: &Arc<Node>) -> ModifierFlags {
        // Single-entry cache mirroring Go's `lastGetCombinedModifierFlagsNode`.
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

    // ────────────────────────────────────────────────────────────────────────
    // Declaration-container / module helpers (used by the emit resolver)
    // ────────────────────────────────────────────────────────────────────────

    /// Walk up `BindingElement` chains to the root declaration.
    /// Mirrors Go's `ast.GetRootDeclaration`.
    pub fn get_root_declaration(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        while current.kind == SyntaxKind::BindingElement {
            // BindingElement → BindingPattern (parent) → VariableDeclaration (grandparent)
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

    /// The declaration container (SourceFile/ModuleDeclaration/EnumDeclaration)
    /// that holds `node`. Mirrors Go's `ast.GetDeclarationContainer`.
    pub fn get_declaration_container(node: &Arc<Node>) -> Option<Arc<Node>> {
        let root = Self::get_root_declaration(node);
        // FindAncestor: walk up from root, returning the first node whose kind
        // is NOT in the skip set; return that node's parent.
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

    /// Whether `node` is a SourceFile that is a global (non-module) script.
    /// Mirrors Go's `ast.IsGlobalSourceFile`.
    pub fn is_global_source_file(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        !Self::is_external_or_common_js_module(node)
    }

    /// Whether `node` (a SourceFile) is an external or CommonJS module.
    /// Mirrors Go's `ast.IsExternalOrCommonJSModule`: a file is a module if
    /// it has a top-level `import`/`export` declaration (ES module) or a
    /// CommonJS indicator. The Rust SourceFile does not yet track CommonJS
    /// indicators, so only the ES-module heuristic is used.
    pub fn is_external_or_common_js_module(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::SourceFile {
            return false;
        }
        let NodeData::SourceFile(data) = &node.data else {
            return false;
        };
        for stmt in data.statements.nodes.iter() {
            // Mirrors Go's `IsExternalModuleIndicator`: any import/re-export,
            // export assignment, or statement with the `export` modifier marks
            // the file as an ES module.
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

    /// Whether `node` is an ambient module that augments an external module.
    /// Mirrors Go's `ast.IsExternalModuleAugmentation`. A `declare module
    /// "foo"` augmentation is external when its parent is a SourceFile that
    /// is itself an external module.
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
            // `declare module "foo"` (StringLiteral name) or `declare global`
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

    /// Whether `node` is a top-level statement kind whose visibility is
    /// "painted" late via the alias marking visitor.
    /// Mirrors Go's `ast.IsLateVisibilityPaintedStatement`.
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

    /// The enclosing import syntax node for an alias declaration, if any.
    /// Mirrors Go's `getAnyImportSyntax`.
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

/// Free function mirroring Go's `ast.GetCombinedModifierFlags`.
///
/// Walks from the root declaration up through the variable-declaration chain
/// (VariableDeclaration → VariableDeclarationList → VariableStatement),
/// OR-ing the syntactic modifier flags of each level. Non-variable
/// declarations return their own modifier flags.
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

// ────────────────────────────────────────────────────────────────────────────
// Public API methods (from exports.go)
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    // Type accessors
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

    // Symbol accessors
    pub fn get_unknown_symbol(&self) -> Option<Arc<Symbol>> {
        self.unknown_symbol.clone()
    }
    pub fn get_undefined_symbol(&self) -> Option<Arc<Symbol>> {
        self.undefined_symbol.clone()
    }
    pub fn get_arguments_symbol(&self) -> Option<Arc<Symbol>> {
        self.arguments_symbol.clone()
    }

    // Properties
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
        // An array-like type has a numeric index signature or is a tuple
        if t.flags.contains(TypeFlags::Object) {
            if let Some(structured) = t.as_structured() {
                // Check for numeric index info
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
                // Tuple types are array-like
                return t.object_flags.contains(ObjectFlags::Tuple);
            }
        }
        false
    }

    pub fn is_array_type(&self, t: &Arc<Type>) -> bool {
        // A type is an array type if it's a reference to Array<T>
        // This is a simplified check; the full implementation resolves the target
        t.flags.contains(TypeFlags::Object) && t.object_flags.contains(ObjectFlags::Reference)
    }

    pub fn is_tuple_type(&self, t: &Arc<Type>) -> bool {
        super::utilities::is_tuple_type(t)
    }

    // Base type of literal
    pub fn get_base_type_of_literal_type(&self, t: &Arc<Type>) -> Arc<Type> {
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
        Arc::clone(t)
    }

    /// Widen a type by replacing fresh literal types with their primitive
    /// base types. Mirrors Go's `getWidenedType` (checker.go ~L18268) for
    /// the function-return-type use case.
    ///
    /// - Fresh literal types (string/number/bigint/boolean) → their
    ///   primitive base. Regular (non-fresh) literals are preserved.
    /// - Unique `symbol` literals → `symbol`.
    /// - Unions → a new union with each constituent widened (nullable
    ///   constituents are preserved as-is, matching Go).
    /// - All other types are returned unchanged.
    ///
    /// Only fresh literals are widened: a fresh literal is one produced by
    /// a literal expression (e.g. `42`, `"hi"`) that has not yet been
    /// "decided" by a declaration context. Regular literals (e.g. the
    /// preserved type of `const x = "hello"`) are not widened here.
    pub fn get_widened_type(&self, t: &Arc<Type>) -> Arc<Type> {
        // Nullable types are not widened (Go skips them in union widening).
        if t.flags.intersects(TYPE_FLAGS_NULLABLE) {
            return Arc::clone(t);
        }
        // Fresh literal types → primitive base. Regular literals are
        // preserved (Go's `getWidenedType` only widens fresh literals).
        if t.flags.intersects(TYPE_FLAGS_LITERAL) {
            if crate::checker::is_fresh_literal_type(t) {
                return self.get_base_type_of_literal_type(t);
            }
            return Arc::clone(t);
        }
        // Unique symbol → symbol.
        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            return self.es_symbol_type();
        }
        // Unions: widen each non-nullable constituent.
        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.get_widened_type(member))
                .collect();
            // Avoid allocating a new union if nothing changed.
            if widened
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            // Rebuild via get_union_type to deduplicate/flatten. We need
            // &mut self for that, but get_union_type only mutates caches;
            // since this method takes &self we work around it by constructing
            // a minimal union without caching. For the return-type use case
            // this is acceptable.
            return self.build_union_from_types(widened);
        }
        Arc::clone(t)
    }

    /// Widen the type of a variable's initializer when no type annotation
    /// is present. Mirrors Go's `getWidenTypeOfLiteralType` +
    /// `getWidenedTypeOfFreshObjectLiteralType` plumbing that runs at the
    /// variable-declaration site.
    ///
    /// Unlike `get_widened_type` (which only widens literal/union types),
    /// this also handles fresh object literal types by widening each
    /// property type, and is recursive for nested object literals.
    ///
    /// Without this, `let x = { a: 1 }` would infer `{ a: 1 }` (literal)
    /// instead of `{ a: number }` (widened), making `x = { a: 2 }` a
    /// false-positive TS2322.
    pub fn widen_initializer_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        // Fresh object literal: widen each property's type recursively.
        if crate::checker::is_object_literal_type(t) {
            return self.widen_object_literal_type(t);
        }
        // Evolving array literal: element type is already widened at
        // creation; nothing more to do here.
        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return Arc::clone(t);
        }
        // Auto array marker (from `let x = []`): convert to an evolving
        // array type with element `never`. Flow analysis will evolve the
        // element type from subsequent `push`/`unshift` calls. Mirrors
        // Go's flow.go L232-234 (`getEvolvingArrayType(c.neverType)`).
        if self.is_auto_array_type(t) {
            return self.get_evolving_array_type(self.never_type());
        }
        // All other types: defer to the standard widening (handles
        // literals, unique symbols, and unions of literals).
        self.get_widened_type(t)
    }

    /// Check if a type is the auto-array marker (`any[]` whose element
    /// type is the `auto` type with `NonInferrableType` flag, used for
    /// `let x = []`).
    pub fn is_auto_array_type(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::Object) || !t.object_flags.contains(ObjectFlags::Reference)
        {
            return false;
        }
        // The auto-array marker is `Array<autoType>` where `autoType` has
        // the `NonInferrableType` flag. Check the element type.
        match &t.data {
            TypeData::Object(obj) => obj
                .type_arguments
                .first()
                .map(|elem| elem.object_flags.contains(ObjectFlags::NonInferrableType))
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Build a widened copy of a fresh object literal type: each
    /// property's literal type is widened to its primitive base
    /// (`1` → `number`, `'hi'` → `string`, `true` → `boolean`).
    /// Nested object literals are widened recursively.
    fn widen_object_literal_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let structured = match t.as_structured() {
            Some(s) => s,
            None => return Arc::clone(t),
        };
        // Collect (name, widened_type) pairs first to avoid re-borrowing
        // `&mut self` while iterating over the structured type's members.
        let mut widened_pairs: Vec<(String, Arc<Type>)> = Vec::new();
        for prop in &structured.properties {
            let prop_type = self.get_type_of_symbol(prop);
            let widened = self.widen_initializer_type(&prop_type);
            widened_pairs.push((prop.name.clone(), widened));
        }
        // Build the widened object type with fresh property symbols.
        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(widened_pairs.len());
        for (name, t) in widened_pairs {
            let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
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

    /// Build a union type from a vec of constituents without the full
    /// `get_union_type` caching machinery (used by `get_widened_type` which
    /// runs under an immutable borrow).
    fn build_union_from_types(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        // Deduplicate by pointer identity and flatten nested unions.
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

    // Get the constraint of a type parameter
    pub fn get_constraint_of_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.constraint.clone();
        }
        None
    }

    // Get the default type of a type parameter
    pub fn get_default_from_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.resolved_default_type.get().cloned();
        }
        None
    }

    // Get the resolved (true) type of a conditional type
    pub fn get_resolved_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            // Try resolved true type first, then resolved false type
            if let Some(rt) = ct.resolved_true_type.get() {
                return Some(rt.clone());
            }
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    // Get the constraint of a mapped type
    pub fn get_constraint_of_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(mt) = &t.data {
            return mt.constraint_type.clone();
        }
        if let TypeData::ReverseMapped(rm) = &t.data {
            return rm.constraint_type.clone();
        }
        None
    }

    // Get the true type of a conditional type
    pub fn get_true_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_true_type.get().cloned();
        }
        None
    }

    // Get the false type of a conditional type
    pub fn get_false_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_false_type.get().cloned();
        }
        None
    }

    // Get the return type of a signature
    pub fn get_return_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        sig.resolved_return_type.get().cloned()
    }

    // Get the type predicate of a signature
    pub fn get_type_predicate_of_signature<'a>(
        &self,
        sig: &'a Arc<Signature>,
    ) -> Option<&'a TypePredicate> {
        sig.resolved_type_predicate.as_deref()
    }

    /// Compute the type predicate of a signature on demand.
    ///
    /// Mirrors Go's `getTypePredicateOfSignature` (relater.go ~L2016) but
    /// without caching (since `Signature` is behind `Arc`). Checks the
    /// signature's declaration for a `TypePredicateNode` return type
    /// annotation (e.g. `x is string`) and creates a `TypePredicate` from it.
    /// Returns `None` if the signature has no type predicate.
    pub fn compute_type_predicate_of_signature(
        &mut self,
        sig: &Arc<Signature>,
    ) -> Option<TypePredicate> {
        // Check cache first.
        if let Some(pred) = sig.resolved_type_predicate.as_deref() {
            // `<<unresolved>>` is the sentinel for "no type predicate".
            if pred.parameter_name == "<<unresolved>>" {
                return None;
            }
            return Some(pred.clone());
        }
        // Compute from declaration.
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
            || pred_data.parameter_name.kind == SyntaxKind::ThisType;
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

    // Get the base constraint of a type
    pub fn get_base_constraint_of_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::TypeParameter(tp) => tp.constrained.resolved_base_constraint.get().cloned(),
            TypeData::Conditional(ct) => ct.constrained.resolved_base_constraint.get().cloned(),
            TypeData::IndexedAccess(ia) => ia.constrained.resolved_base_constraint.get().cloned(),
            TypeData::Index(it) => it.constrained.resolved_base_constraint.get().cloned(),
            _ => None,
        }
    }

    // Get type arguments of a type reference
    pub fn get_type_arguments(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        if let TypeData::Object(obj) = &t.data {
            return obj.type_arguments.clone();
        }
        Vec::new()
    }

    // Get the type of a unique symbol
    pub fn get_unique_symbol_type(&self, _name: &str) -> Option<Arc<Type>> {
        // Unique symbol types are cached by symbol ID, not by name
        // This is a simplified version
        None
    }

    // Was canceled
    pub fn was_canceled(&self) -> bool {
        false
    }

    // ────────────────────────────────────────────────────────────────────────
    // Entry points (stubs — full implementation in P3.6)
    // ────────────────────────────────────────────────────────────────────────

    /// Type-check a single source file.
    ///
    /// Go: `Checker.checkSourceFile`. This is the main entry point invoked
    /// by `Program.GetSemanticDiagnostics` for each source file.
    ///
    /// Currently implements a minimal subset of type checking:
    /// - Walks statements and expressions.
    /// - For identifiers in expression position, attempts to resolve them
    ///   against the binder's symbol map (locals + file symbol members).
    /// - Emits `TS2304 Cannot find name '{0}'.` for unresolvable identifiers.
    ///
    /// Full type-checking logic (type inference, relation checking, flow
    /// narrowing, etc.) is added incrementally.
    pub fn check_source_file(&mut self, file: &Arc<SourceFile>) {
        // Populate globals from source file symbols on first use.
        if !self.globals_populated {
            self.populate_globals();
            self.globals_populated = true;
        }

        let file_node = Arc::clone(&file.node);
        let file_id = file_node.id();
        let source_file_symbol = self.program.symbol_map().symbol_of(&file_node).cloned();

        // Populate `parent` pointers on every node in this file's AST. The
        // parser builds nodes bottom-up into `Arc`s without setting parent
        // pointers, but contextual typing (`get_contextual_type`) and several
        // grammar checks walk `node.parent` to find the enclosing context.
        // This is safe because the checker runs single-threaded and the AST
        // is a tree (each node has exactly one parent).
        self.set_parent_pointers(&file_node);

        // Save file context for diagnostics.
        let file_arc = Arc::clone(file);
        self.current_file = Some(Arc::clone(&file_arc));
        self.current_file_id = file_id;
        self.current_file_symbol = source_file_symbol;

        // Push the source file scope.
        self.push_scope(&file_node);

        // Walk top-level statements.
        let statements: Vec<Arc<Node>> = match &file_node.data {
            crate::ast::NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
            _ => Vec::new(),
        };
        for stmt in &statements {
            self.check_statement(stmt);
        }

        self.pop_scope();
        self.current_file = None;
        self.current_file_id = 0;
        self.current_file_symbol = None;
    }

    /// Recursively populate `parent` pointers on every node in the AST
    /// rooted at `node`. The parser builds nodes bottom-up into `Arc`s
    /// without setting parent pointers; this walk fixes them up so that
    /// `get_contextual_type` (and grammar checks that inspect `node.parent`)
    /// can locate the enclosing context. Children are collected first and
    /// the parent pointer is set via the same `Arc::as_ptr` mutation pattern
    /// the binder uses for `Symbol` mutation — safe because the checker
    /// runs single-threaded and the AST is a tree (one parent per node).
    fn set_parent_pointers(&mut self, node: &Arc<Node>) {
        use crate::ast::node_data_generated::for_each_child;
        // Collect direct children first so the recursive call below doesn't
        // hold a borrow on `self` through the `for_each_child` closure.
        let mut children: Vec<Arc<Node>> = Vec::new();
        for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        let parent_clone = Arc::clone(node);
        for child in &children {
            let child_mut = Arc::as_ptr(child) as *mut Node;
            // Skip if already pointing at this parent (idempotent on
            // re-checks — avoids churn and redundant Arc clones).
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

    /// Return the semantic diagnostics collected during type checking.
    ///
    /// Go: `Checker.getDiagnostics`.
    pub fn get_semantic_diagnostics(&self) -> Vec<crate::ast::Diagnostic> {
        self.diagnostics.get_all()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Statement and expression checking (P3.6 minimal implementation)
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    // ─────────────────────────────────────────────────────────────────────
    // Type inference
    // ─────────────────────────────────────────────────────────────────────

    /// Get the type of a node, computing and caching it if necessary.
    ///
    /// Go: `Checker.getTypeOfSymbolAtLocation` / `getTypeOfNode`.
    pub fn get_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        // Check cache first (type_node_links stores resolved types for
        // both type nodes and expression nodes).
        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }
        let result = self.compute_type_of_node(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }

    /// Compute the type of a node (without caching).
    fn compute_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {
            // Literal types — wrap with `get_fresh_type_of_literal_type`
            // so that literal expressions produce a *fresh* literal type.
            // Fresh literals widen to their primitive base only at
            // `let`/`var` declaration sites (not `const`), mirroring Go's
            // freshness mechanism.
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
            SyntaxKind::TrueKeyword => {
                self.get_fresh_type_of_literal_type(&self.true_type())
            }
            SyntaxKind::FalseKeyword => {
                self.get_fresh_type_of_literal_type(&self.false_type())
            }
            SyntaxKind::NullKeyword => self.null_type(),
            SyntaxKind::UndefinedKeyword => self.undefined_type(),
            SyntaxKind::BigIntLiteral => {
                self.get_fresh_type_of_literal_type(&self.bigint_type())
            }
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
            // Binary expressions
            SyntaxKind::BinaryExpression => self.get_type_of_binary_expression(node),
            SyntaxKind::PrefixUnaryExpression => {
                if let crate::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    // The parser currently represents `delete x` and
                    // `void x` as PrefixUnaryExpression with DeleteKeyword /
                    // VoidKeyword as the operator (Go creates separate
                    // DeleteExpression / VoidExpression nodes). Handle both
                    // representations here.
                    match data.operator {
                        // `!x` → boolean.
                        SyntaxKind::ExclamationToken => return self.boolean_type(),
                        // `delete x` → boolean.
                        SyntaxKind::DeleteKeyword => return self.boolean_type(),
                        // `void x` → undefined.
                        SyntaxKind::VoidKeyword => return self.undefined_type(),
                        // `+x`/`-x`/`~x`/`++x`/`--x` → number.
                        _ => return self.number_type(),
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::PostfixUnaryExpression => {
                // `x++` / `x--` → number.
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
                // `x as T` has the type of the type annotation `T`.
                if let crate::ast::NodeData::AsExpression(data) = &node.data {
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::SatisfiesExpression => {
                // `x satisfies T` keeps the type of `x` (the assertion does
                // not change the expression's type, only validates it).
                if let crate::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::TypeAssertionExpression => {
                // `<T>x` has the type of the type annotation `T`.
                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::NonNullExpression => {
                // `x!` asserts that `x` is non-null: the type of `x` with
                // `null` and `undefined` removed. Mirrors Go's
                // `getNonNullableType`.
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
                // `cond ? a : b` → widened union of `a` and `b` types.
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
                // `` `a${x}b` `` → string.
                self.string_type()
            }
            SyntaxKind::TaggedTemplateExpression => {
                // `` tag`...` `` → result of calling the tag function.
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
                // `delete x` → boolean.
                self.boolean_type()
            }
            SyntaxKind::VoidExpression => {
                // `void x` → undefined.
                self.undefined_type()
            }
            SyntaxKind::AwaitExpression => {
                // `await x` → unwrapped type of the awaited expression.
                // Simplified: return the expression's type.
                if let crate::ast::NodeData::AwaitExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                // `this` / `super` → the enclosing class's instance type.
                // Mirrors Go's `getThisType` / `getThisTypeOfObjectLiteral`.
                // When not inside a class (e.g. top-level `this` in a
                // module), falls back to `any` (or `globalThis` in
                // script-mode — simplified to `any` here).
                self.this_type_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.get_any_type())
            }
            _ => self.get_any_type(),
        }
    }

    /// Get the type of an identifier reference.
    ///
    /// If the identifier has an associated flow node (set by the binder),
    /// the declared type is narrowed based on control-flow constraints
    /// (e.g. `if (x !== null)` narrows `x` in the then-branch).
    fn get_type_of_identifier(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(symbol) = self.resolve_identifier(node) {
            let flow = self.program.symbol_map().flow_node_of(node).map(Arc::clone);
            let narrowed = self.get_narrowed_type_of_symbol(&symbol, flow.as_ref());
            // When the reference is `x` in `x.length`, `x.push(value)`,
            // `x.unshift(value)` or `x[n] = value`, we give the type
            // `autoArrayType` (instead of finalizing the evolving array)
            // so that operations on empty arrays are possible without
            // implicit any errors and new element types can be inferred
            // without type mismatch errors. Mirrors Go's
            // `getFlowTypeOfReference` (flow.go ~L106-110).
            if narrowed.object_flags.contains(ObjectFlags::EvolvingArray)
                && self.is_evolving_array_operation_target(node)
            {
                return self.auto_array_type();
            }
            // Finalize evolving array types when they are read as a value
            // (i.e. they "escape" the flow context). Mirrors Go's
            // `finalizeEvolvingArrayType` call in `getFlowTypeOfReference`.
            self.finalize_evolving_array_type(&narrowed)
        } else {
            self.get_any_type()
        }
    }

    /// Check if `node` is the receiver of an evolving-array operation:
    /// `node.length`, `node.push(value)`, `node.unshift(value)`, or
    /// `node[i] = value`. Mirrors Go's `isEvolvingArrayOperationTarget`
    /// (flow.go ~L1521).
    fn is_evolving_array_operation_target(&self, node: &Arc<Node>) -> bool {
        let root = self.get_reference_root(node);
        let Some(parent) = &root.parent else {
            return false;
        };
        // `root.length` or `root.push(...)` / `root.unshift(...)`.
        if let NodeData::PropertyAccessExpression(pa) = &parent.data {
            if Arc::ptr_eq(&pa.expression, root) {
                let name = pa.name.text();
                if name == "length" {
                    return true;
                }
                if name == "push" || name == "unshift" {
                    // parent.parent must be a CallExpression for this to be a
                    // mutation (e.g. `arr.push(1)`). Bare `arr.push` (without
                    // a call) is not a mutation.
                    if let Some(grandparent) = &parent.parent {
                        if matches!(grandparent.kind, SyntaxKind::CallExpression) {
                            return true;
                        }
                    }
                }
            }
        }
        // `root[i] = value` — element access assignment. Mirrors Go's
        // `isElementAssignment` branch.
        if let NodeData::ElementAccessExpression(ea) = &parent.data {
            if Arc::ptr_eq(&ea.expression, root) {
                if let Some(grandparent) = &parent.parent {
                    if let NodeData::BinaryExpression(bin) = &grandparent.data {
                        if bin.operator_token.kind == SyntaxKind::EqualsToken
                            && Arc::ptr_eq(&bin.left, parent)
                        {
                            // Go additionally requires the index type to be
                            // number-like. We approximate by always allowing
                            // (avoiding a recursive type computation here).
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Find the root of a reference chain, unwrapping parenthesized
    /// expressions, `=` assignment LHS, and comma-operator RHS. Mirrors
    /// Go's `getReferenceRoot` (flow.go ~L1855).
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

    /// Get the type of a symbol.
    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Declaration merging: when a symbol has both the `ValueModule` flag
        // (a `namespace N` declaration) and a value-side flag (`Function`,
        // `Class`, `RegularEnum`, or `ConstEnum`), the resolved type must
        // combine the value type's call/construct signatures with the
        // namespace's exported members. Mirrors Go's
        // `getDeclaredTypeOfSymbol` namespace + value merge.
        if symbol.flags.contains(SymbolFlags::ValueModule)
            && (symbol.flags.contains(SymbolFlags::Function)
                || symbol.flags.contains(SymbolFlags::Class)
                || symbol.flags.contains(SymbolFlags::RegularEnum)
                || symbol.flags.contains(SymbolFlags::ConstEnum))
        {
            return self.get_type_of_merged_namespace_symbol(symbol);
        }
        // For now, return any for most symbols
        // TODO: implement proper symbol type resolution
        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            || symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            || symbol.flags.contains(SymbolFlags::Function)
            || symbol.flags.contains(SymbolFlags::Class)
            || symbol.flags.contains(SymbolFlags::Property)
            || symbol.flags.contains(SymbolFlags::EnumMember)
        {
            // 1. Symbol-level cache (`value_symbol_links[symbol].resolved_type`),
            //    mirrors Go's `symbol.links.type`.
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
            // 2. Node-level cache on the value declaration — this is where
            //    `check_variable_declaration` writes the resolved type.
            if let Some(decl) = &symbol.value_declaration {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        return Arc::clone(t);
                    }
                }
            }
            // 3. Fallback: any of the symbol's declarations might carry a
            //    cached type (e.g. parameter declarations).
            for decl in &symbol.declarations {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        return Arc::clone(t);
                    }
                }
            }
            self.get_any_type()
        } else if symbol.flags.contains(SymbolFlags::ValueModule) {
            // Namespace: build an anonymous object type from the namespace's
            // exported members. `resolve_namespace_type` caches the result.
            self.resolve_namespace_type(symbol)
        } else if symbol.flags.intersects(SymbolFlags::ENUM) {
            // Enum used as a value (`Color.Red`): build an anonymous object
            // type whose members are the enum's members, each carrying its
            // literal type (populated by `resolve_enum_type`). Mirrors Go's
            // `getDeclaredTypeOfSymbol` enum value type.
            self.resolve_enum_value_type(symbol)
        } else {
            self.get_any_type()
        }
    }

    /// Resolve the type of a symbol that has both a namespace declaration
    /// (`namespace N { ... }`) and a value declaration (`function N`, `class
    /// N`, `enum N`). The result combines the value type's call/construct
    /// signatures with the namespace's exported members as properties, so
    /// both `N()`/`new N()` and `N.exportedMember` type-check.
    ///
    /// Mirrors the namespace slice of Go's `getDeclaredTypeOfSymbol`, which
    /// folds the value-side type into the namespace object type.
    fn get_type_of_merged_namespace_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Cached merged type on `type_alias_links[symbol].declared_type`.
        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        // 1. Resolve the value type (function/class/enum) via the value-side
        //    resolution path (which looks up `value_symbol_links` and
        //    `type_node_links` caches populated during statement checking).
        let value_type = self.get_value_type_of_symbol(symbol);

        // 2. Resolve the namespace members. `resolve_namespace_type` caches
        //    its result on `type_alias_links[symbol].declared_type`; we
        //    overwrite that cache with the merged type below so subsequent
        //    lookups see the combined type.
        let ns_type = self.resolve_namespace_type(symbol);

        // 3. Build the merged type: take the namespace object type (which
        //    carries members + properties) and copy in the value type's
        //    call/construct signatures. The resulting anonymous object type
        //    supports both `N()` / `new N()` and `N.member`.
        let (call_sigs, construct_sigs) = match &value_type.data {
            TypeData::Object(obj) => {
                let cs = obj.structured.call_signatures().to_vec();
                let xs = obj.structured.construct_signatures().to_vec();
                (cs, xs)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let merged = if call_sigs.is_empty() && construct_sigs.is_empty() {
            // No signatures to merge (e.g. the value type wasn't resolved
            // yet); just return the namespace type so members are visible.
            ns_type
        } else {
            // Build a fresh structured type that carries the namespace's
            // members/properties/index_infos plus the value type's
            // call/construct signatures. `StructuredTypeData.signatures`
            // stores call signatures first (count = call_signature_count),
            // then construct signatures.
            let ns_obj = match &ns_type.data {
                TypeData::Object(obj) => obj,
                _ => {
                    // Namespace type isn't an object (unexpected); nothing
                    // to merge into, so return the value type which carries
                    // the signatures.
                    self.type_alias_links.get_or_default(symbol).declared_type =
                        Some(Arc::clone(&value_type));
                    return value_type;
                }
            };
            let ns_structured = &ns_obj.structured;
            let mut structured = StructuredTypeData::default();
            structured.members = ns_structured.members.clone();
            structured.properties = ns_structured.properties.clone();
            structured.index_infos = ns_structured.index_infos.clone();
            // Existing namespace signatures (e.g. from a nested namespace
            // with its own call signatures) come after the value type's
            // call signatures but before construct signatures.
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

        // Cache the merged type so future lookups hit the cache above.
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&merged));
        merged
    }

    /// Resolve the value-side type of a symbol (function, class, variable,
    /// property) by consulting the symbol-level and node-level caches
    /// populated during statement/expression checking. This is the value-
    /// only subset of `get_type_of_symbol` — it does NOT consider the
    /// `ValueModule` flag, so it can be used to recover the function/class
    /// type of a merged namespace+value symbol without recursing back into
    /// the namespace resolution path.
    fn get_value_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // 1. Symbol-level cache (`value_symbol_links[symbol].resolved_type`),
        //    mirrors Go's `symbol.links.type`.
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }
        // 2. Node-level cache on the value declaration — this is where
        //    `check_variable_declaration` / `check_function_declaration`
        //    write the resolved type.
        if let Some(decl) = &symbol.value_declaration {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }
        // 3. Fallback: any of the symbol's declarations might carry a
        //    cached type (e.g. parameter declarations).
        for decl in &symbol.declarations {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }
        self.get_any_type()
    }

    /// Build the value-side type of an enum symbol — an anonymous object
    /// type whose members are the enum's members, each carrying its literal
    /// type. This is what makes `Color.Red` (as a value expression) resolve
    /// to the literal type `0` rather than `any`.
    ///
    /// `resolve_enum_type` (the type-side resolver) populates each member
    /// symbol's `value_symbol_links.resolved_type` with its literal type;
    /// this method first calls `resolve_enum_type` to ensure those are set,
    /// then builds an object type wrapping the enum symbol's members table.
    fn resolve_enum_value_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Reuse a cached value type on `value_symbol_links[symbol]`.
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }
        // Ensure member literal types are populated by resolving the enum's
        // type-side union (this writes to each member symbol's links).
        let _ = self.resolve_enum_type(symbol);
        // Build an anonymous object type from the enum's members table.
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
            // Ensure the member's type is resolvable; the property symbol
            // carries the literal type via value_symbol_links.
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

    /// Get the type of a function-like expression (FunctionExpression /
    /// ArrowFunction / FunctionDeclaration). Returns an anonymous object
    /// type whose single call signature carries the inferred (or annotated)
    /// return type *and* the parameter types resolved from each parameter's
    /// type annotation.
    ///
    /// Parameters without an annotation inherit the corresponding parameter
    /// type from the contextual function type (the annotation on the
    /// variable/parameter/property this function is assigned to). This
    /// contextual typing flows into the function body too: because parameter
    /// types are stored on the binder's actual parameter symbols (shared
    /// with the body's scope resolution), `infer_function_return_type` sees
    /// the contextual types when it walks `return` expressions. This is what
    /// makes `let f: (x: string) => number = (x) => x;` report TS2322 (the
    /// body returns `string`, not the expected `number`).
    ///
    /// To make contextual typing available to return-type inference, the
    /// signature is built in two passes: first with a placeholder return
    /// type to prime parameter-symbol types, then with the inferred return
    /// type for the final signature. The first pass's signature is discarded.
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
        // Fetch the contextual function type (e.g. the annotation on the
        // variable this function expression initializes). When present,
        // unannotated parameters inherit the corresponding parameter type
        // from its first call signature.
        let contextual_type = self.get_contextual_type(node, ContextFlags::None);
        let contextual_signature: Option<&Arc<Signature>> = contextual_type
            .as_ref()
            .and_then(|t| t.as_structured())
            .and_then(|s| s.call_signatures().first());
        // Push the function/arrow scope BEFORE priming parameter types so
        // that type-parameter references in parameter annotations (e.g.
        // `x: T` inside `function f<T>(x: T)`) can be resolved via the scope
        // stack. The scope stays pushed through return-type inference so
        // `get_type_of_node` on body expressions finds the parameters and
        // type parameters.
        let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
        if is_arrow {
            self.push_arrow_function_scope(node);
        } else {
            self.push_function_scope(node);
        }
        // Pass 1: prime parameter-symbol types (annotated/contextual/any)
        // so that return-type inference below can resolve parameter
        // references inside the body. The placeholder return type is
        // discarded along with this signature.
        let placeholder = self.get_any_type();
        let _primed = self.build_signature_from_function_like_type_node(
            parameters,
            placeholder,
            /* is_construct */ false,
            contextual_signature,
            /* declaration */ None, // primed signature is discarded
        );
        // Now infer the return type — the body sees the contextual param
        // types set above. An explicit return-type annotation always wins.
        let return_type = self.infer_function_return_type(body, type_node);
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }
        // Pass 2: build the final signature with the inferred return type.
        // Parameter types are re-resolved (idempotent overwrite of the
        // symbols' resolved types set in pass 1).
        let sig = self.build_signature_from_function_like_type_node(
            parameters,
            return_type,
            /* is_construct */ false,
            contextual_signature,
            /* declaration */ Some(Arc::clone(node)),
        );
        self.create_function_or_constructor_type(vec![sig], false)
    }

    /// Build a multi-signature function type from a symbol's overload
    /// declarations. In TypeScript, overloaded functions have multiple
    /// `FunctionDeclaration` nodes for the same symbol: the overload
    /// signatures (without bodies) come first, followed by a single
    /// implementation signature (with a body). Only the overload signatures
    /// are visible to callers — the implementation is internal.
    ///
    /// This method collects all overload declarations (those without a body)
    /// and builds a function type with one signature per overload. If there
    /// is only one declaration (no overloads), returns `None` (the caller
    /// should use the single-signature type from `get_type_of_function_like`).
    ///
    /// Mirrors Go's `getSignaturesOfType` for function types, which returns
    /// all call signatures (populated during `createSignatureForFunction`).
    fn build_overload_function_type(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        // Collect all FunctionDeclaration nodes for this symbol.
        let fn_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::FunctionDeclaration)
            .cloned()
            .collect();
        if fn_decls.len() <= 1 {
            return None; // Not an overload set.
        }
        // Build a signature for each overload (declarations without a body).
        // The implementation (with a body) is excluded from the visible
        // signature list — callers only see the overloads.
        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        for decl in &fn_decls {
            let has_body = match &decl.data {
                crate::ast::NodeData::FunctionDeclaration(data) => data.body.is_some(),
                _ => false,
            };
            if has_body {
                continue; // Skip the implementation.
            }
            let (parameters, type_node) = match &decl.data {
                crate::ast::NodeData::FunctionDeclaration(data) => {
                    (&data.parameters, data.type_node.as_ref())
                }
                _ => continue,
            };
            let return_type = match type_node {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                parameters,
                return_type,
                /* is_construct */ false,
                /* contextual_signature */ None,
                /* declaration */ Some(Arc::clone(decl)),
            );
            signatures.push(sig);
        }
        if signatures.is_empty() {
            return None;
        }
        Some(self.create_function_or_constructor_type(signatures, false))
    }

    /// Build the type of a class declaration: an anonymous object type
    /// carrying construct signatures derived from the class's `constructor`
    /// member. The construct signature's parameters are resolved (and
    /// cached on the parameter symbols) so `check_call_arguments` can verify
    /// `new Foo(arg)` calls (TS2345).
    fn get_type_of_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let members = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.get_any_type(),
        };
        // Push the class scope before building the instance type so that
        // type-parameter references in property annotations resolve.
        self.push_scope(node);
        // Build the class's instance type (including inherited members from
        // `extends`) to use as the construct signature's return type. This
        // makes `new Foo()` return the instance type, so `instance.prop`
        // is properly checked. Mirrors Go's `createClassType` →
        // `getInstanceTypeFromClassType`.
        let instance_type = self.build_class_instance_type_with_base(node);
        let mut construct_sigs: Vec<Arc<Signature>> = Vec::new();
        for member in members.0.iter() {
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
                /* is_construct */ true,
                /* contextual_signature */ None,
                /* declaration */ Some(Arc::clone(member)),
            );
            construct_sigs.push(sig);
        }
        self.pop_scope();
        if construct_sigs.is_empty() {
            // No explicit constructor: `new Foo()` with zero args is valid.
            // Synthesize a no-arg construct signature.
            let sig = self.build_signature_from_function_like_type_node(
                &Arc::new(NodeList::default()),
                Arc::clone(&instance_type),
                /* is_construct */ true,
                None,
                /* declaration */ None,
            );
            construct_sigs.push(sig);
        }
        self.create_function_or_constructor_type(construct_sigs, /* is_construct */ true)
    }

    /// Get the return type of a `CallExpression`. Resolves the called
    /// expression's type; if it's a function type with at least one call
    /// signature, return that signature's resolved return type. Otherwise
    /// fall back to `any`.
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
            // Overload resolution: find the first signature that accepts
            // the call's arguments. If multiple signatures exist (overloads),
            // try each in order; otherwise use the first (only) signature.
            let matching_idx = if signatures.len() == 1 {
                0
            } else {
                self.find_matching_signature(signatures, &callee.1)
            };
            let sig = &signatures[matching_idx];
            if let Some(rt) = self.get_return_type_of_signature(sig) {
                return rt;
            }
            // Signature without a resolved return type — fall back to
            // any so callers don't blow up.
            return self.get_any_type();
        }
        self.get_any_type()
    }

    /// Get the return type of a `NewExpression` (`new Foo()`).
    ///
    /// Resolves the constructor expression's type, looks for construct
    /// signatures, and returns the first signature's return type (the
    /// instance type). Falls back to `any`.
    fn get_return_type_of_new_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let callee = match &node.data {
            crate::ast::NodeData::NewExpression(data) => &data.expression,
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(callee);
        if let Some(structured) = callee_type.as_structured() {
            for sig in structured.construct_signatures() {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    return rt;
                }
                return self.get_any_type();
            }
        }
        self.get_any_type()
    }

    /// Get the type of a binary expression.
    fn get_type_of_binary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        use crate::ast::SyntaxKind::*;
        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            match data.operator_token.kind {
                // Arithmetic operators return number
                PlusToken
                | MinusToken
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
                // Comparison operators return boolean
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
                // Logical operators return union of operands (simplified)
                AmpersandAmpersandToken | BarBarToken | QuestionQuestionToken => {
                    self.get_type_of_node(&data.left)
                }
                // Assignment operators return the right-hand side type
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

    /// Get the type of a `PropertyAccessExpression` (`x.prop`).
    ///
    /// Resolves the type of `x`, looks up `prop` as a property on that
    /// type, and returns the property's type. Falls back to `any` when
    /// the property is not found or the object type is unknown.
    fn get_type_of_property_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, name) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => (&data.expression, &data.name),
            _ => return self.get_any_type(),
        };
        let obj_type = self.get_type_of_node(obj_expr);
        let name_text = name.text();
        if let Some(t) = self.get_property_type_of_type(&obj_type, name_text) {
            return t;
        }
        // For array types, common properties like `length` are numbers;
        // methods like `push`/`pop`/etc. fall back to `any` for now.
        if name_text == "length" && self.is_array_type(&obj_type) {
            return self.number_type();
        }
        self.get_any_type()
    }

    /// Whether a property named `name` on type `t` is declared `readonly`.
    ///
    /// Walks the structured type's `members` symbol table to find the
    /// property's symbol, then inspects each declaration node for the
    /// `ReadonlyKeyword` modifier. Mirrors Go's
    /// `isReadonlySymbol`/`getDeclarationModifierFlagsFromDeclarations`
    /// read-only check used by `checkAssignmentStatement`.
    ///
    /// Returns `false` if the type is not structured, the property doesn't
    /// exist, or the symbol has no declaration with `readonly`. Conservative
    /// (returns `false`) to avoid false positives.
    fn is_property_readonly(&self, t: &Arc<Type>, name: &str) -> bool {
        let Some(structured) = t.as_structured() else {
            return false;
        };
        let Some(symbol) = structured.members.get(name) else {
            return false;
        };
        // Check if any of the symbol's declarations carry the `readonly`
        // modifier (e.g. `readonly x: number` in a class body or
        // `readonly` parameter property).
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
        // Also honor `CheckFlags::Readonly` set elsewhere (e.g. on
        // synthetic property symbols created from index signatures).
        if symbol.check_flags.contains(CheckFlags::Readonly) {
            return true;
        }
        false
    }

    /// Whether a property named `name` exists on `t`, handling unions,
    /// intersections, type parameters, arrays, and primitives.
    ///
    /// Used by `check_property_access` to decide whether to emit TS2339.
    /// Conservative in the face of unknown type kinds (returns `true` to
    /// avoid false positives).
    pub(super) fn has_property_of_type(&mut self, t: &Arc<Type>, name: &str) -> bool {
        // `any`/`unknown`/`never`/`undefined`/`null` allow any property; never
        // emit TS2339 for them.
        if t.flags.intersects(
            TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::Never
                | TypeFlags::Undefined
                | TypeFlags::Null,
        ) {
            return true;
        }

        // Direct hit on structured members, or applicable index signature.
        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }
            if !structured.index_infos.is_empty() {
                return true;
            }
            // Structured object type with no matching member and no index
            // signature: fall through to `false` for plain object types.
            // However, Object flag without structured members (e.g. a
            // reference like `Array<T>`) is handled below.
            if t.flags.contains(TypeFlags::Object)
                && !t.object_flags.contains(ObjectFlags::Reference)
            {
                return false;
            }
        }

        // Union: property must exist on every non-nullable constituent.
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

        // Intersection: property exists if any constituent has it.
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

        // Type parameter: defer to constraint.
        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.has_property_of_type(&constraint, name);
            }
            // No constraint: allow any property (matches tsc behavior for
            // unconstrained `T`).
            return true;
        }

        // Array types (`Array<T>` reference). Without lib.d.ts, tsc would
        // report TS2339 for any property (including `push`/`unshift`) on a
        // fully-typed array. We allow `length` intrinsically. For the special
        // `autoArrayType` marker (the inferred type of `let x = []`), we
        // also allow array mutation methods (`push`/`unshift`) so that
        // evolving-array flow analysis can run without lib.d.ts — matching
        // the form of `let x = []; x.push(1)` that the ARRAY_MUTATION flow
        // node is designed to handle.
        if self.is_array_type(t) {
            if name == "length" {
                return true;
            }
            if self.is_auto_array_type(t) && self.is_array_mutation_method(name) {
                return true;
            }
            return false;
        }

        // Evolving array types (created by flow analysis from `autoArrayType`):
        // these are not `Reference`-flagged, so `is_array_type` doesn't match.
        // Allow `length` and the array mutation methods so that subsequent
        // `x.push(2)` after `x.push(1)` continues to evolve the element type.
        if t.object_flags.contains(ObjectFlags::EvolvingArray) {
            return name == "length" || self.is_array_mutation_method(name);
        }

        // Tuple types: `length` is intrinsically available.
        if self.is_tuple_type(t) {
            return name == "length";
        }

        // Primitive types and their literals: without lib.d.ts, no
        // properties are available — any access is TS2339.
        if t.flags.intersects(
            TypeFlags::Number
                | TypeFlags::String
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::Void
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral
                | TypeFlags::UniqueESSymbol,
        ) {
            return false;
        }

        // Enum types and other object types without structured data: be
        // conservative and don't error.
        if t.flags.contains(TypeFlags::Object | TypeFlags::Enum) {
            return true;
        }

        // Default: don't emit error for unknown type kinds.
        true
    }

    /// Check if a method name is an array mutation method (push, unshift,
    /// etc.) that the binder creates ARRAY_MUTATION flow nodes for.
    pub fn is_array_mutation_method(&self, name: &str) -> bool {
        matches!(name, "push" | "unshift")
    }

    /// Check a `PropertyAccessExpression` (`x.prop`) and emit TS2339 when
    /// `prop` does not exist on the type of `x`.
    ///
    /// Mirrors tsc behavior: when the object type is known and structured
    /// (object literal, type reference with members, intersection, union of
    /// compatible constituents, type parameter with a constraint, etc.), a
    /// missing property is reported. `any`/`unknown`/`never` and other
    /// permissive types skip the check.
    fn check_property_access(&mut self, node: &Arc<Node>) {
        let (obj_expr, name) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => (&data.expression, &data.name),
            _ => return,
        };
        let obj_type = self.get_type_of_node(obj_expr);
        let name_text = name.text();
        if self.has_property_of_type(&obj_type, name_text) {
            return;
        }
        let file = self.current_file.clone();
        let type_str = self.type_to_string(&obj_type);
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            name.loc,
            PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
            vec![name_text.to_string(), type_str],
        ));
    }

    /// Check that call/new-expression arguments are assignable to the
    /// corresponding parameter types of the callee's signature. Emits
    /// TS2345 for mismatched arguments.
    ///
    /// Mirrors the argument-checking portion of Go's `checkCallExpression`
    /// / `checkNewExpression`. We resolve the callee type, find the call
    /// (or construct) signatures, and compare each argument against the
    /// corresponding parameter type. Rest parameters are handled by
    /// matching all trailing arguments against the rest element type.
    /// `any` callee / missing signature → skip (no false positives).
    ///
    /// When the callee has multiple signatures (function overloads),
    /// overload resolution is performed: each signature is tried in order,
    /// and the first one whose parameters accept all arguments is used.
    /// If no signature matches, the error is reported against the first
    /// signature (matching TypeScript's behavior).
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
        let callee_type = self.get_type_of_node(callee_expr);
        // `any` callee → skip (no false positives without a signature).
        if callee_type.flags.contains(TypeFlags::Any) {
            return;
        }
        let structured = match callee_type.as_structured() {
            Some(s) => s,
            None => {
                // Non-structured callee (primitive like `number`, `string`,
                // etc.) is never callable/constructable. Mirrors Go's
                // `invocationError` for types with no signatures.
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    callee_expr.loc,
                    if is_new {
                        THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE
                    } else {
                        THIS_EXPRESSION_IS_NOT_CALLABLE
                    },
                    vec![],
                ));
                return;
            }
        };
        let signatures = if is_new {
            structured.construct_signatures()
        } else {
            structured.call_signatures()
        };
        if signatures.is_empty() {
            // Structured type but no call/construct signatures — e.g.
            // calling a plain object literal or a number. Mirrors Go's
            // `invocationError` head message ("This expression is not
            // callable" / "This expression is not constructable").
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                callee_expr.loc,
                if is_new {
                    THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE
                } else {
                    THIS_EXPRESSION_IS_NOT_CALLABLE
                },
                vec![],
            ));
            return;
        }
        // Overload resolution: if multiple signatures exist, find the first
        // that accepts all arguments. If none matches, report errors
        // against the first signature.
        let matching_idx = if signatures.len() == 1 {
            0
        } else {
            self.find_matching_signature(signatures, &arguments)
        };
        let sig = Arc::clone(&signatures[matching_idx]);
        // Arity check: mirror Go's `getArgumentArityError`. If the argument
        // count doesn't match the signature's parameter range, emit TS2554
        // ("Expected N arguments, but got M") or TS2555 ("Expected at least N
        // arguments, but got M") for rest-parameter signatures, and skip the
        // per-argument type checks (matching tsc, which reports the arity
        // error as the primary call error).
        if !self.check_call_arity(node, &sig, &arguments, callee_expr, is_new) {
            return;
        }
        let file = self.current_file.clone();
        // When the signature has a rest parameter (always the last one),
        // arguments at/after the rest position are checked against the rest
        // element type, not the rest array type. Mirrors Go's
        // `getTypeAtPosition` rest handling.
        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {
            let rest_param_type = self.get_type_of_symbol(&sig.parameters[rest_index]);
            Some(self.get_array_element_type(&rest_param_type))
        } else {
            None
        };
        for (i, arg) in arguments.iter().enumerate() {
            // Determine the parameter type to check against.
            let param_type = if has_rest && i >= rest_index {
                // Rest position: check against the array element type.
                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {
                self.get_type_of_symbol(&sig.parameters[i])
            } else {
                // Beyond declared params with no rest — should have been
                // caught by the arity check; skip to avoid false positives.
                continue;
            };
            // `any` parameter → always assignable, skip.
            if param_type.flags.contains(TypeFlags::Any) {
                continue;
            }
            let arg_type = self.get_type_of_node(arg);
            if !self.is_type_assignable_to(&arg_type, &param_type) {
                let arg_str = self.type_to_string(&arg_type);
                let param_str = self.type_to_string(&param_type);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file.clone(),
                    arg.loc,
                    ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1,
                    vec![arg_str, param_str],
                ));
            }
        }
    }

    /// Check call/new-expression argument arity against a signature.
    ///
    /// Mirrors the common cases of Go's `getArgumentArityError`: detects
    /// spread arguments (TS2556), too-few arguments (TS2554/TS2555), and
    /// too-many arguments on non-rest signatures (TS2554). Returns `true`
    /// when arity is acceptable (no diagnostic emitted), `false` otherwise.
    ///
    /// For too-few errors the diagnostic spans the callee expression
    /// (matching Go's `getErrorNodeForCallNode`); for too-many errors it
    /// spans the extra arguments, from `args[maxCount]` to the last arg.
    /// The `parameterRange` string follows tsc: `min` for rest signatures,
    /// `min-max` when min < max, otherwise `min`.
    fn check_call_arity(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) -> bool {
        let arg_count = arguments.len();

        // Spread arguments require a rest parameter or a tuple-typed rest.
        // Mirror Go's `getSpreadArgumentIndex` + early return.
        if let Some(spread_idx) = arguments
            .nodes
            .iter()
            .position(|a| matches!(a.data, crate::ast::NodeData::SpreadElement(_)))
        {
            // A spread is allowed only when it falls at/after the minimum
            // argument count and the signature has an effective rest
            // parameter or enough declared parameters to cover it.
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
            // Otherwise the spread is structurally acceptable; defer the
            // per-element type check.
            return true;
        }

        let min_count = self.get_min_argument_count(sig);
        let max_count = self.get_parameter_count(sig);
        let has_rest = self.has_effective_rest_parameter(sig);

        // Too many arguments: only an error when there's no effective rest
        // parameter. The error span covers the trailing extra arguments.
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

        // Too few arguments.
        if arg_count < min_count {
            let file = self.current_file.clone();
            // Error node: for CallExpression, the callee (optionally the
            // property-access name); for NewExpression, the whole node.
            let error_loc = if is_new { node.loc } else { callee_expr.loc };
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

    /// Compute the source range for "too many arguments" errors: from the
    /// first extra argument (`args[maxCount]`) to the end of the last
    /// argument. Mirrors Go's `getArgumentArityError` trailing-args span.
    fn extra_arguments_range(&self, arguments: &Arc<NodeList>, max_count: usize) -> TextRange {
        if max_count >= arguments.nodes.len() {
            // Defensive: fall back to the whole call's argument range.
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

    /// Check whether a signature accepts all given arguments (used for
    /// overload resolution). Returns `true` if every argument is
    /// assignable to the corresponding parameter type.
    ///
    /// Mirrors the "is applicable signature" test in Go's
    /// `checkCallArguments`/`signatureIsAssignable`.
    fn signature_accepts_arguments(
        &mut self,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> bool {
        for (i, arg) in arguments.iter().enumerate() {
            if i < sig.parameters.len() {
                let param_type = self.get_type_of_symbol(&sig.parameters[i]);
                // `any` parameter → always assignable.
                if param_type.flags.contains(TypeFlags::Any) {
                    continue;
                }
                let arg_type = self.get_type_of_node(arg);
                if !self.is_type_assignable_to(&arg_type, &param_type) {
                    return false;
                }
            } else {
                // More arguments than parameters (without rest) — not
                // applicable. (Rest-parameter handling is deferred.)
                return false;
            }
        }
        true
    }

    /// Find the index of the first signature that accepts the given
    /// arguments (overload resolution). If no signature matches, returns
    /// 0 (the first signature is used for error reporting).
    ///
    /// Mirrors Go's overload resolution loop in `checkCallArguments`.
    fn find_matching_signature(
        &mut self,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> usize {
        for (idx, sig) in signatures.iter().enumerate() {
            if self.signature_accepts_arguments(sig, arguments) {
                return idx;
            }
        }
        0
    }

    /// Get the type of an `ElementAccessExpression` (`x[key]`).
    ///
    /// For array types (`T[]`), returns the element type `T`. For tuple
    /// types, returns the element at the given index (or a union of all
    /// element types for non-constant indices). Falls back to `any`.
    fn get_type_of_element_access(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (obj_expr, arg_expr) = match &node.data {
            crate::ast::NodeData::ElementAccessExpression(data) => {
                (&data.expression, &data.argument_expression)
            }
            _ => return self.get_any_type(),
        };
        let obj_type = self.get_type_of_node(obj_expr);

        // Tuple element access with a numeric literal index.
        if self.is_tuple_type(&obj_type) {
            if let Some(index) = self.get_constant_numeric_value(arg_expr) {
                if let Some(t) = self.get_tuple_element_type(&obj_type, index as usize) {
                    return t;
                }
            }
            // Non-constant index on a tuple → union of all element types.
            // For now, fall back to `any`.
            return self.get_any_type();
        }

        // Array element access → element type.
        if self.is_array_type(&obj_type) {
            return self.get_array_element_type(&obj_type);
        }

        // Object with a string/number index signature → index signature
        // value type.
        if let Some(structured) = obj_type.as_structured() {
            for info in &structured.index_infos {
                if let Some(key_type) = &info.key_type {
                    if key_type.flags.contains(crate::checker::TypeFlags::String)
                        || key_type.flags.contains(crate::checker::TypeFlags::Number)
                    {
                        if let Some(val_type) = &info.value_type {
                            return Arc::clone(val_type);
                        }
                    }
                }
            }
        }

        self.get_any_type()
    }

    /// Get the element type of an array type (`Array<T>` → `T`).
    pub(crate) fn get_array_element_type(&self, t: &Arc<Type>) -> Arc<Type> {
        match &t.data {
            crate::checker::TypeData::Object(obj) => {
                // `Array<T>` is a reference type with one type argument.
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

    /// Try to extract a constant numeric value from a literal expression.
    fn get_constant_numeric_value(&self, node: &Arc<Node>) -> Option<f64> {
        match &node.data {
            crate::ast::NodeData::NumericLiteral(data) => data.text.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Get the type of an array literal expression `[e1, e2, ...]`.
    ///
    /// Infers `Array<T>` where `T` is the widened type of the elements.
    /// If all elements have the same widened type (e.g. all numbers), the
    /// array type is `number[]`. If elements have mixed types, the array
    /// type is `any[]` for now (a proper union would be `Array<A | B>`).
    /// Empty arrays infer `any[]`.
    fn get_type_of_array_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let elements = match &node.data {
            crate::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return self.get_any_type(),
        };
        if elements.is_empty() {
            // Empty array literal `[]`: return the auto-array marker.
            // `widen_initializer_type` converts this to an evolving array
            // type with element `never`, which flow analysis evolves from
            // subsequent `push`/`unshift` calls. Mirrors Go's
            // `getWidenedType` returning `autoArrayType` for `[]`.
            return self.auto_array_type();
        }
        // Get the widened type of each element.
        let mut element_types: Vec<Arc<Type>> = Vec::new();
        for elem in elements.iter() {
            // Skip spread elements for now.
            if elem.kind == SyntaxKind::SpreadElement {
                return self.create_array_type(self.get_any_type());
            }
            let t = self.get_type_of_node(elem);
            element_types.push(self.get_widened_type_of_literal(&t));
        }
        // If all elements have the same type, use it.
        let first = &element_types[0];
        let all_same = element_types[1..]
            .iter()
            .all(|t| Arc::ptr_eq(t, first) || self.types_are_equal(t, first));
        if all_same {
            return self.create_array_type(Arc::clone(first));
        }
        // Mixed types → any[] for now.
        self.create_array_type(self.get_any_type())
    }

    /// Get the type of an object literal expression `{ a: 1, b: "hi" }`.
    ///
    /// Infers an anonymous object type `{ a: number, b: string }` where each
    /// property's type is taken from its initializer. Literal initializers
    /// keep their literal type (e.g. `{ kind: "foo" }` → `{ kind: "foo" }`)
    /// rather than being widened — this mirrors TypeScript's "fresh literal"
    /// behavior so that object literals remain assignable to discriminated
    /// unions like `{ kind: "foo" } | { kind: "bar" }`. Spread elements
    /// (`{ ...x }`) and computed property names currently fall back to
    /// `any` for the whole type (the full Go checker would compute a union
    /// with the spread target's apparent type).
    fn get_type_of_object_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let properties = match &node.data {
            crate::ast::NodeData::ObjectLiteralExpression(data) => &data.properties,
            _ => return self.get_any_type(),
        };
        // Collect (name, type) pairs first so that we can borrow
        // `&mut self.value_symbol_links` below without re-entering the type
        // computation (which also borrows `&mut self`).
        let mut prop_pairs: Vec<(String, Arc<Type>)> = Vec::new();
        let mut fell_back_to_any = false;
        for prop in properties.iter() {
            match &prop.data {
                NodeData::PropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }
                    // Keep literal types (no widening) so object literals
                    // remain assignable to discriminated unions.
                    let t = self.get_type_of_node(&data.initializer);
                    prop_pairs.push((name, t));
                }
                NodeData::ShorthandPropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }
                    // Shorthand `{ a }` — resolve the identifier's type
                    // (which is the bound variable's type, e.g. `number`
                    // for `let a = 42`). The variable's type is already
                    // widened at its declaration, so no widening here.
                    let t = self.get_type_of_node(&data.name);
                    prop_pairs.push((name, t));
                }
                NodeData::SpreadAssignment(_) => {
                    // Spread isn't supported yet — fall back to `any` for
                    // the whole object type.
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
        // Build the anonymous object type with property symbols. Each
        // symbol's resolved type is stored in `value_symbol_links` so
        // `get_type_of_symbol` returns it during relater property checks.
        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(prop_pairs.len());
        for (name, t) in prop_pairs {
            let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
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

    /// Extract the property name string from a name node (identifier,
    /// string literal, numeric literal). Returns an empty string for
    /// computed property names (caller should skip those).
    fn get_property_name_from_node(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(id) => id.text.clone(),
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => n.text.clone(),
            NodeData::ComputedPropertyName(_) => String::new(),
            _ => node.text().to_string(),
        }
    }

    /// Widen a literal type to its base type (e.g. `3` → `number`).
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

    /// Check if two types are equal (by identity or by kind/flags).
    fn types_are_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        if a.flags != b.flags {
            return false;
        }
        // For intrinsic types, compare the intrinsic name.
        match (&a.data, &b.data) {
            (crate::checker::TypeData::Intrinsic(a), crate::checker::TypeData::Intrinsic(b)) => {
                a.intrinsic_name == b.intrinsic_name
            }
            _ => false,
        }
    }

    /// Get a number literal type (inferred).
    fn infer_number_literal_type(&mut self, text: &str) -> Arc<Type> {
        // Parse the numeric literal text and create a literal type.
        let num = crate::jsnum::Number::from_string(text);
        if num.is_nan() {
            return self.number_type();
        }
        self.get_number_literal_type(num)
    }

    /// Get a string literal type (inferred).
    fn infer_string_literal_type(&mut self, text: &str) -> Arc<Type> {
        self.get_string_literal_type(text)
    }

    /// Check a statement node.
    ///
    /// Go: `Checker.checkStatement`. Dispatches by node kind.
    pub fn check_statement(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));
        match node.kind {
            SyntaxKind::ExpressionStatement => {
                if let crate::ast::NodeData::ExpressionStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::VariableStatement => {
                if let crate::ast::NodeData::VariableStatement(data) = &node.data {
                    // Grammar check: validate the variable declaration list.
                    self.check_grammar_variable_declaration_list(&data.declaration_list);
                    self.check_variable_declaration_list(&data.declaration_list);
                    // Grammar check: validate modifiers (export, declare, etc.).
                    self.check_grammar_modifiers(node);
                }
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_statement(&data.then_statement);
                    if let Some(else_stmt) = &data.else_statement {
                        self.check_statement(else_stmt);
                    }
                }
            }
            SyntaxKind::WhileStatement => {
                if let crate::ast::NodeData::WhileStatement(data) = &node.data {
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
                if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                        // Check that the return value's type is assignable
                        // to the enclosing function's declared return type.
                        // Mirrors Go's `checkReturnStatement` (checker.go
                        // ~L11800). When there's no declared return type
                        // (inferred), the stack entry is `None` and we skip.
                        // Clone the `Arc<Type>` out of the stack first so we
                        // don't hold an immutable borrow of `self` while
                        // calling mutable methods below.
                        let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                        if let Some(expected) = expected {
                            let actual = self.get_type_of_node(expr);
                            // `any` return value → always assignable.
                            if !actual.flags.contains(TypeFlags::Any)
                                && !self.is_type_assignable_to(&actual, &expected)
                            {
                                let actual_str = self.type_to_string(&actual);
                                let expected_str = self.type_to_string(&expected);
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    expr.loc,
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                    vec![actual_str, expected_str],
                                ));
                            }
                        }
                    } else {
                        // `return;` with no value — if the function declares
                        // a non-void/non-undefined return type, this is an
                        // error (TS1135). Mirrors Go's `checkReturnStatement`
                        // empty-return branch.
                        let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                        if let Some(expected) = expected {
                            if !expected.flags.contains(TypeFlags::Void)
                                && !expected.flags.contains(TypeFlags::Undefined)
                                && !expected.flags.contains(TypeFlags::Any)
                            {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    node.loc,
                                    ARGUMENT_EXPRESSION_EXPECTED,
                                    vec![],
                                ));
                            }
                        }
                    }
                }
            }
            SyntaxKind::Block => {
                self.push_scope(node);
                if let crate::ast::NodeData::Block(data) = &node.data {
                    for stmt in data.statements.iter() {
                        self.check_statement(stmt);
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
                    // case_block is a CaseBlock node; walk its clauses.
                    if let crate::ast::NodeData::CaseBlock(case_block) = &data.case_block.data {
                        for case in case_block.clauses.iter() {
                            self.check_case_clause(case);
                        }
                    }
                    self.break_continue_context_stack.pop();
                }
            }
            // Declarations: walk only expression-position children.
            // Without parent pointers, we cannot detect declaration names
            // via `is_declaration_name`, so we must handle each kind
            // explicitly, skipping names and type-position children.
            SyntaxKind::FunctionDeclaration => {
                // Grammar check: validate modifiers and parameter list.
                self.check_grammar_modifiers(node);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(tps) = &data.type_parameters {
                        let _ = tps; // TODO: check_grammar_type_parameter_list
                    }
                    self.check_grammar_parameter_list(&data.parameters);
                }
                // JSDoc check: validate @param tags against actual
                // parameters. No-op until JSDoc parsing (P2.7) lands.
                self.check_unmatched_jsdoc_parameters(node);
                // Compute the function's type (with inferred return type)
                // and cache it on the declaration node + symbol so later
                // references (e.g. `let y = f()`) can recover it. This must
                // run BEFORE checking the body so that parameter-symbol
                // types are primed (including type-parameter annotations
                // like `x: T`) — otherwise `get_type_of_node` on parameter
                // references inside the body returns `any`. Mirrors Go's
                // `getSymbolLinks(symbol).type = getWidenedTypeOfFunction`.
                let fn_type = self.get_type_of_function_like(node);
                self.type_node_links.get_or_default(node).resolved_type = Some(fn_type.clone());
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        if let Some(symbol) = self.resolve_identifier(name) {
                            // For overloaded functions, build a multi-signature
                            // type from all overload declarations (excluding
                            // the implementation). Only the implementation
                            // declaration has a body; when we reach it, all
                            // overload declarations have already been
                            // processed by the binder. The combined type
                            // replaces the single-signature type on the symbol
                            // so callers see all overloads.
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
                // Check the function body with parameter types primed.
                self.push_function_scope(node);
                self.break_continue_context_stack
                    .push(BreakContinueContext {
                        kind: BreakContinueContextKind::Function,
                        label: None,
                        is_iteration: false,
                    });
                // Push the declared return type so `return expr;` statements
                // in the body can be checked against it. `None` means the
                // function has no explicit return-type annotation (return
                // type is inferred) — in that case return-statement checking
                // is skipped. Mirrors Go's `expectedReturn` tracking.
                let declared_return = match &node.data {
                    crate::ast::NodeData::FunctionDeclaration(data) => data
                        .type_node
                        .as_ref()
                        .map(|tn| self.get_type_from_type_node(tn)),
                    _ => None,
                };
                self.return_type_stack.push(declared_return);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.return_type_stack.pop();
                self.break_continue_context_stack.pop();
                self.pop_function_scope();
            }
            SyntaxKind::ClassDeclaration => {
                // Grammar check: validate modifiers.
                self.check_grammar_modifiers(node);
                // Push the class scope before building the instance type so
                // that type-parameter references in property annotations
                // (e.g. `value: T`) resolve correctly.
                self.push_scope(node);
                // Build the instance type (including inherited members from
                // `extends`) and push it as the `this` type so that method
                // bodies can resolve `this.prop` and `super.prop`.
                let this_type = self.build_class_instance_type_with_base(node);
                self.this_type_stack.push(this_type);
                // Check heritage clauses (e.g. `extends Foo`, `implements I`).
                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(heritage) = &data.heritage_clauses {
                        for clause in heritage.iter() {
                            self.check_heritage_clause(clause);
                        }
                    }
                    // Check member initializers / method bodies.
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                }
                self.pop_scope();
                self.this_type_stack.pop();
                // Build the class type (with construct signatures from the
                // constructor) and cache it on the declaration node + symbol
                // so `new Foo(arg)` can resolve the callee and check args.
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
            SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier => {
                // No expression-position children to check — all type-level
                // or import-level.
            }
            SyntaxKind::EnumDeclaration => {
                // Check enum member initializers.
                self.push_scope(node);
                if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
                    for member in data.members.iter() {
                        self.check_enum_member(member);
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::ExportAssignment => {
                // `export default expr` or `export = expr`
                if let crate::ast::NodeData::ExportAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ModuleDeclaration => {
                // Check the module body.
                self.push_scope(node);
                if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::EmptyStatement => {
                // No expressions to check.
            }
            SyntaxKind::LabeledStatement => {
                // Push a Labeled context so that `break label`/`continue label`
                // can resolve to this label, then check the nested statement.
                // The label identifier itself is NOT an expression and must
                // not be resolved (mirrors Go's `checkStatement` dispatching
                // `LabeledStatement` to `bindLabeledStatement` + statement
                // check without visiting the label name as a reference).
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
                // Grammar check: validate break/continue targets.
                self.check_grammar_break_or_continue_statement(node);
            }
            SyntaxKind::VariableDeclaration => {
                self.check_variable_declaration(node);
            }
            _ => {
                // Fallback: walk children to find expressions.
                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }

    fn check_for_initializer(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::VariableDeclarationList => {
                self.check_variable_declaration_list(node);
            }
            _ => self.check_expression(node),
        }
    }

    fn check_variable_declaration_list(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclarationList(data) = &node.data {
            for decl in data.declarations.iter() {
                self.check_variable_declaration(decl);
            }
        }
    }

    fn check_variable_declaration(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclaration(data) = &node.data {
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }
            // Resolve the variable's type and check assignability of the
            // initializer against the (optional) type annotation.
            //
            // This is the first place the relater is wired into the diagnostic
            // flow: when a type annotation is present, the initializer must be
            // assignable to it (TS2322). When no annotation is present, the
            // type is inferred from the initializer.
            let resolved_type = match (&data.type_node, &data.initializer) {
                (Some(type_node), Some(init)) => {
                    let annotation_type = self.get_type_from_type_node(type_node);
                    let init_type = self.get_type_of_node(init);
                    if !self.is_type_assignable_to(&init_type, &annotation_type) {
                        let file = self.current_file.clone();
                        let init_str = self.type_to_string(&init_type);
                        let annot_str = self.type_to_string(&annotation_type);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            init.loc,
                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec![init_str, annot_str],
                        ));
                    }
                    annotation_type
                }
                (Some(type_node), None) => self.get_type_from_type_node(type_node),
                (None, Some(init)) => {
                    // No type annotation: infer from the initializer, then
                    // apply freshness-gated widening. This mirrors Go's
                    // `getWidenedLiteralTypeForInitializer` +
                    // `getWidenTypeOfLiteralType` plumbing at the
                    // variable-declaration site.
                    //
                    // Step 1: `get_widened_literal_type_for_initializer`
                    // decides the const-vs-let fate of fresh literals:
                    //   - `const x = "hello"` preserves the fresh literal
                    //     `"hello"` (no widening).
                    //   - `let x = "hello"` widens to `string`.
                    // Step 2: `get_regular_type_of_literal_type` converts
                    // any preserved fresh literal to its regular form so
                    // that step 3 doesn't re-widen it (a fresh literal
                    // passed to `widen_initializer_type`/`get_widened_type`
                    // would otherwise be widened).
                    // Step 3: `widen_initializer_type` performs structural
                    // widening for object/array literals (e.g. `{ a: 1 }`
                    // → `{ a: number }`), recursing into properties whose
                    // types are still fresh.
                    let init_type = self.get_type_of_node(init);
                    let widened_literal =
                        self.get_widened_literal_type_for_initializer(node, &init_type);
                    let regularized = self.get_regular_type_of_literal_type(&widened_literal);
                    self.widen_initializer_type(&regularized)
                }
                (None, None) => self.get_any_type(),
            };
            // Cache the resolved type on the VariableDeclaration node — this
            // is what `symbol.value_declaration` points to, so
            // `get_type_of_symbol` can recover the type via `type_node_links`.
            // (Previously this was stored on `data.name`, the Identifier
            // child node, which `get_type_of_symbol` never inspects.)
            self.type_node_links.get_or_default(node).resolved_type = Some(resolved_type.clone());
            // Also cache on the Identifier so direct `get_type_of_node(name)`
            // callers (e.g. hover on the name) hit the cache without
            // recursing through the symbol.
            self.type_node_links
                .get_or_default(&data.name)
                .resolved_type = Some(resolved_type.clone());
            // And mirror onto `value_symbol_links[symbol]` so symbol-driven
            // lookups (without going through a node) work as well. Mirrors
            // Go's `symbol.links.type`.
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
        // Heritage clauses contain type references (e.g., `extends Foo`).
        // For `implements` clauses, verify that the class's instance type
        // is assignable to each implemented interface; otherwise emit
        // TS2420. `extends` clauses are type-level and skipped here (the
        // base-class members are merged separately).
        let data = match &node.data {
            crate::ast::NodeData::HeritageClause(d) => d,
            _ => return,
        };
        if data.token != SyntaxKind::ImplementsKeyword {
            return;
        }
        // Locate the enclosing ClassDeclaration to recover the class name
        // and member list.
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
        // Build the class's instance type including inherited members
        // from `extends`, so that base-class members also satisfy the
        // `implements` check. Mirrors Go's `getBaseTypes` integration in
        // `checkTypeImplementsList`.
        let instance_type = self.build_class_instance_type_with_base(class_node);
        // Check each implemented interface.
        for type_ref in data.types.iter() {
            let interface_type = self.get_type_from_heritage_type_reference(type_ref);
            if interface_type.flags.contains(TypeFlags::Any) {
                // Couldn't resolve the interface (e.g. TS2304 already
                // reported). Skip the assignability check.
                continue;
            }
            if !self.is_type_assignable_to(&instance_type, &interface_type) {
                let iface_name = self.type_to_string(&interface_type);
                self.grammar_error_on_node_with_args(
                    class_node,
                    &crate::diagnostics::messages_generated::CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1,
                    &[class_name.clone(), iface_name],
                );
            }
        }
    }

    /// Build an anonymous object type representing the class's instance type
    /// (the public property/method surface used by `implements` checks and
    /// `this` type resolution inside method bodies).
    ///
    /// Mirrors the Go checker's `build_classInstanceType` / instance-type
    /// construction. Constructor bodies and static members are excluded —
    /// only instance `PropertyDeclaration`/`MethodDeclaration`/accessor
    /// members contribute. Reuses the interface-member builder because the
    /// relevant member kinds share the same name/type-node/parameter shape.
    ///
    /// When the class has an `extends` heritage clause, the base class's
    /// instance type is resolved recursively and its properties are merged
    /// in (underneath the derived class's own properties, so overrides win).
    /// Mirrors Go's `getBaseTypes`/property inheritance in
    /// `getPropertiesOfTypeOfObjectLiteral`/`getPropertiesOfObjectType`.
    fn build_class_instance_type(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        self.build_interface_type_from_members(members)
    }

    /// Build the class instance type including inherited members from the
    /// `extends` clause. The derived class's own members take precedence
    /// (override) over the base class's members with the same name.
    ///
    /// Mirrors Go's `getBaseTypeNodeTypes`/property inheritance:
    /// `class D extends B {}` gives D's instance type all of B's properties
    /// plus D's own.
    fn build_class_instance_type_with_base(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (members, heritage_clauses) = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.build_interface_type_from_members(&Arc::new(NodeList::default())),
        };
        // Build the derived class's own instance type.
        let own_type = self.build_interface_type_from_members(members);
        // Find the `extends` clause and resolve the base class.
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

    /// Resolve a base class reference from an `extends` heritage clause to
    /// its instance type (recursively including the base's own base class).
    /// Returns `any` if the base class cannot be resolved (e.g. unknown
    /// identifier — TS2304 is emitted elsewhere).
    fn resolve_base_class_instance_type(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {
        // The heritage-clause entry is an `ExpressionWithTypeArguments`.
        // Get its type, which for a class reference resolves to the class's
        // constructor type (an object with construct signatures). We need
        // the instance type instead.
        // First, try to resolve the class symbol and build its instance
        // type directly (including the base class's own base).
        if let crate::ast::NodeData::ExpressionWithTypeArguments(data) = &type_ref.data {
            if data.expression.kind == SyntaxKind::Identifier {
                if let Some(symbol) = self.resolve_identifier(&data.expression) {
                    if symbol.flags.contains(SymbolFlags::Class) {
                        // Find the ClassDeclaration node and build its
                        // instance type with base.
                        if let Some(class_node) = symbol
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                            .cloned()
                        {
                            // Avoid infinite recursion for self-referential
                            // extends (shouldn't happen, but be safe).
                            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
                            if !self.resolving_type_aliases.insert(key) {
                                return self.get_any_type();
                            }
                            let instance = self.build_class_instance_type_with_base(&class_node);
                            self.resolving_type_aliases.remove(&key);
                            return instance;
                        }
                    }
                }
            }
        }
        // Fallback: resolve the type reference directly. For interfaces
        // or other types, this gives the object type. For classes, it gives
        // the constructor type — extract its properties (construct
        // signatures' return type would be ideal, but we don't track that
        // yet). Fall back to `any` to avoid false positives.
        let t = self.get_type_from_type_node(type_ref);
        if t.flags.contains(TypeFlags::Any) {
            return self.get_any_type();
        }
        // If it's an object type (e.g. from an interface), use it directly.
        if t.flags.contains(TypeFlags::Object) {
            return t;
        }
        self.get_any_type()
    }

    /// Merge two instance types: `derived` properties override `base`
    /// properties with the same name. Returns a new anonymous object type
    /// containing all properties from both, with derived taking precedence.
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
        // Start with base properties, then overlay derived (so derived
        // overrides win). Preserve declaration order: base first, then
        // derived-only.
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        // Base properties first.
        for prop in &base_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        // Derived properties: override if already present, append if new.
        for prop in &derived_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                symbol_table.insert(prop.name.clone(), Arc::clone(prop));
                // Replace in props list (preserve position for base members).
                if let Some(slot) = props.iter_mut().find(|p| p.name == prop.name) {
                    *slot = Arc::clone(prop);
                }
            } else {
                symbol_table.insert(prop.name.clone(), Arc::clone(prop));
                props.push(Arc::clone(prop));
            }
        }
        // Merge index infos (base first, then derived).
        let mut index_infos = base_data.index_infos.clone();
        index_infos.extend(derived_data.index_infos.iter().cloned());
        let signatures = base_data.signatures.clone();
        let call_signature_count = base_data.call_signature_count;
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    /// Resolve a heritage-clause type reference (`implements Foo` /
    /// `extends Bar<T>`) to its underlying `Type`.
    ///
    /// The node kind for heritage-clause entries is
    /// `ExpressionWithTypeArguments`, which `get_type_from_type_node` already
    /// dispatches to `get_type_from_type_reference`. Returns `error_type`
    /// (any) when the reference cannot be resolved — in that case TS2304 is
    /// emitted elsewhere, so the `implements` check is skipped by the caller.
    fn get_type_from_heritage_type_reference(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {
        self.get_type_from_type_node(type_ref)
    }

    fn check_class_member(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                // Only check the body; the name, parameters, and return type
                // are declarations/types.
                let (body, type_node): (Option<Arc<Node>>, Option<Arc<Node>>) = match &node.data {
                    crate::ast::NodeData::MethodDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone())
                    }
                    crate::ast::NodeData::ConstructorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone())
                    }
                    crate::ast::NodeData::GetAccessorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone())
                    }
                    crate::ast::NodeData::SetAccessorDeclaration(d) => {
                        (d.body.clone(), d.type_node.clone())
                    }
                    _ => (None, None),
                };
                if let Some(body) = body {
                    self.push_function_scope(node);
                    // Push the declared return type so `return expr;`
                    // statements in the body can be checked against it.
                    // `None` means no explicit return-type annotation.
                    let declared_return = type_node
                        .as_ref()
                        .map(|tn| self.get_type_from_type_node(tn));
                    self.return_type_stack.push(declared_return);
                    match body.kind {
                        SyntaxKind::Block => self.check_statement(&body),
                        _ => self.check_expression(&body),
                    }
                    self.return_type_stack.pop();
                    self.pop_function_scope();
                }
            }
            SyntaxKind::PropertyDeclaration => {
                // Only check the initializer; the name and type are
                // declarations/types.
                if let crate::ast::NodeData::PropertyDeclaration(data) = &node.data {
                    // Static members cannot reference class type parameters
                    // (TS2322). Force-resolve the type annotation with the
                    // `in_static_member_type` flag set so any TypeParameter
                    // reference in the type tree is reported. Mirrors Go's
                    // NameResolver `ast.IsStatic(lastLocation)` check.
                    if node.has_syntactic_modifier(ModifierFlags::Static) {
                        if let Some(type_node) = &data.type_node {
                            let prev = self.in_static_member_type;
                            self.in_static_member_type = true;
                            let _ = self.get_type_from_type_node(type_node);
                            self.in_static_member_type = prev;
                        }
                    }
                    if let Some(init) = &data.initializer {
                        self.check_expression(init);
                    }
                }
            }
            SyntaxKind::PropertySignature => {
                // All type-level — no expressions to check.
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                if let crate::ast::NodeData::ClassStaticBlockDeclaration(data) = &node.data {
                    self.check_statement(&data.body);
                }
            }
            _ => {
                // SemicolonClassElement etc. — no expressions.
            }
        }
    }

    fn check_enum_member(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::EnumMember(data) = &node.data {
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }
        }
    }

    /// Check an expression node: resolve identifier references and recurse
    /// into sub-expressions.
    ///
    /// Go: `Checker.checkExpression`.
    pub fn check_expression(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));
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
                // Literal expressions: no identifier references to resolve.
            }
            SyntaxKind::BinaryExpression => {
                if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
                    self.check_expression(&data.left);
                    self.check_expression(&data.right);
                    use crate::ast::SyntaxKind::*;
                    // TS2540: `obj.prop = value` where `prop` is declared
                    // `readonly` (a `readonly` modifier on a class property
                    // or parameter property). Mirrors Go's
                    // `checkAssignmentStatement` read-only check.
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
                    // TS2367: For equality/relational comparisons between
                    // types with no overlap, the comparison is always
                    // `false` (or `true` for `!=`/`!==`). Mirrors Go's
                    // `checkBinaryExpression` comparison-overlap check
                    // (checker.go ~L13800). Skipped for `any`/`unknown`/
                    // `never`/`null`/`undefined` operands (per Go's
                    // `isTypeRelatedTo` short-circuits).
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
                        // Skip the overlap check when either operand is
                        // `any`/`unknown`/`never`/`null`/`undefined` — those
                        // types are comparable to everything in TS.
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
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let crate::ast::NodeData::PostfixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);
                }
            }
            SyntaxKind::ParenthesizedExpression => {
                if let crate::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::CallExpression => {
                if let crate::ast::NodeData::CallExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    for arg in data.arguments.iter() {
                        self.check_expression(arg);
                    }
                }
                self.check_call_arguments(node, /* is_new */ false);
            }
            SyntaxKind::NewExpression => {
                if let crate::ast::NodeData::NewExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    if let Some(args) = &data.arguments {
                        for arg in args.iter() {
                            self.check_expression(arg);
                        }
                    }
                }
                self.check_call_arguments(node, /* is_new */ true);
            }
            SyntaxKind::PropertyAccessExpression => {
                // Only check the left side; the right side is a property name,
                // not an identifier reference. Then verify the property exists
                // on the object type (TS2339).
                if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
                self.check_property_access(node);
            }
            SyntaxKind::ElementAccessExpression => {
                if let crate::ast::NodeData::ElementAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_expression(&data.argument_expression);
                }
            }
            SyntaxKind::ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(data) = &node.data {
                    self.check_expression(&data.condition);
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
                    for prop in data.properties.iter() {
                        self.check_object_literal_element(prop);
                    }
                }
            }
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression => {
                // Parameters are declarations; the body contains expressions.
                self.check_function_like_body(node);
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
                // The left side is an expression; the right side is a type.
                if let crate::ast::NodeData::AsExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TypeAssertionExpression => {
                // `<T>x`: the left side is a type; the right is an expression.
                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    self.check_expression(&data.expression);
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
                // JSX tag names, attribute names, and closing-element tag
                // names are not identifier references. Only walk:
                //   - JsxExpression children (and recursively, nested JSX)
                //   - JsxAttribute initializers / JsxSpreadAttribute expressions
                self.check_jsx_element(node);
                // Run JSX-specific type checks on the opening-like element.
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
            }
            SyntaxKind::JsxExpression => {
                if let crate::ast::NodeData::JsxExpression(data) = &node.data {
                    // Grammar check (e.g. comma operator).
                    self.check_grammar_jsx_expression(node);
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            _ => {
                // Fallback: walk children to find expressions.
                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }

    fn check_object_literal_element(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::PropertyAssignment => {
                if let crate::ast::NodeData::PropertyAssignment(data) = &node.data {
                    // The name is not a reference. The initializer is.
                    self.check_expression(&data.initializer);
                }
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                // `x` in `{ x }` is both a declaration and a reference to the
                // outer-scope `x`. Check it as a reference.
                if let crate::ast::NodeData::ShorthandPropertyAssignment(data) = &node.data {
                    self.check_identifier_reference(&data.name);
                }
            }
            SyntaxKind::SpreadAssignment => {
                if let crate::ast::NodeData::SpreadAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
    }

    fn check_function_like_body(&mut self, node: &Arc<Node>) {
        // Compute the function's type first. This triggers contextual
        // typing: when the function is a call argument (e.g.
        // `f((x) => x.toFixed())`), `get_type_of_function_like` calls
        // `get_contextual_type` which resolves the parameter type from
        // the callee's signature. The resolved parameter types are stored
        // in `value_symbol_links` so that `check_function_like_body`'s
        // body walk (below) sees them when resolving parameter references.
        self.get_type_of_node(node);
        // Walk children, but skip parameter names (they are declarations).
        // The simplest correct approach is to dispatch on the body only.
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
            // Arrow functions do not have their own `arguments` object.
            let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
            if is_arrow {
                self.push_arrow_function_scope(node);
            } else {
                self.push_function_scope(node);
            }
            // Push the declared return type so `return expr;` statements
            // in the body can be checked against it. `None` means no
            // explicit return-type annotation (return type inferred).
            let declared_return = type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn));
            self.return_type_stack.push(declared_return);
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => {
                    // Arrow function expression body (`() => expr`): the
                    // expression IS the return value, so check its type
                    // against the declared return type directly (no
                    // `ReturnStatement` node is involved). Mirrors Go's
                    // `checkFunctionExpressionBody` for arrow bodies.
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
            if is_arrow {
                self.pop_arrow_function_scope();
            } else {
                self.pop_function_scope();
            }
        }
    }

    /// Walk a node's children and check any that look like expressions.
    /// Used as a fallback for node kinds we don't handle explicitly.
    fn walk_children_for_expressions(&mut self, node: &Arc<Node>) {
        // Collect children first to avoid borrow-checker issues.
        let children: Vec<Arc<Node>> = {
            let mut collected = Vec::new();
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collected.push(Arc::clone(child));
                false
            });
            collected
        };
        for child in &children {
            // Skip type-position children and declaration names. We do this
            // by checking the child's kind against a known set of expression-
            // position kinds.
            if is_expression_position_kind(child.kind) {
                self.check_expression(child);
            } else if is_statement_kind(child.kind) {
                self.check_statement(child);
            }
            // Otherwise (type nodes, modifier lists, names, etc.) skip.
        }
    }

    /// Check a JSX element/fragment: walk only JsxExpression children,
    /// nested JSX, and attribute initializers. Tag names and attribute names
    /// are not identifier references.
    fn check_jsx_element(&mut self, node: &Arc<Node>) {
        // For JsxElement, walk attributes (of opening_element) and children.
        // For JsxSelfClosingElement, walk attributes and type_arguments.
        // For JsxFragment, walk children.
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

        // Walk attributes (skip tag_name and closing tag_name).
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
            // Also walk type_arguments if present (they are type-position).
        }

        // Walk children.
        for child in &children {
            self.check_jsx_child(child);
        }
    }

    /// Check a single JSX attribute: skip the name, check the initializer.
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

    /// Check a single JSX child (text, expression, or nested element).
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
            // JsxText, JsxTextAllWhiteSpaces: no references.
            _ => {}
        }
    }

    /// Check an identifier in expression position: attempt to resolve it,
    /// and emit TS2304 if it cannot be found.
    fn check_identifier_reference(&mut self, node: &Arc<Node>) {
        // Skip if the identifier's text is empty (parser recovery).
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return,
        };
        if name.is_empty() {
            return;
        }
        // Skip if the identifier is the name of a declaration rather than a
        // reference. We detect this by looking at the parent's kind and the
        // slot the identifier occupies.
        if is_declaration_name(node) {
            return;
        }
        // Skip property access right-hand sides (e.g., `x.foo` — `foo` is a
        // property name, not a reference).
        if is_property_access_name(node) {
            return;
        }

        if self.resolve_identifier(node).is_some() {
            return;
        }

        // Emit TS2304 "Cannot find name '{0}'."
        let file = self.current_file.clone();
        let diagnostic =
            crate::ast::Diagnostic::new(file, node.loc, CANNOT_FIND_NAME_0, vec![name.to_string()]);
        self.diagnostics.add(diagnostic);
    }

    /// Push a container node onto the scope stack, making its symbol members
    /// and locals visible for identifier resolution.
    pub(crate) fn push_scope(&mut self, node: &Arc<Node>) {
        self.scope_stack.push(node.id());
    }

    /// Push a function-like scope and increment the function-like counter.
    fn push_function_scope(&mut self, node: &Arc<Node>) {
        self.function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    /// Pop a function scope.
    fn pop_function_scope(&mut self) {
        self.function_scope_count -= 1;
        self.scope_stack.pop();
    }

    /// Push an arrow function scope. Arrow functions do not have their own
    /// `arguments` object.
    fn push_arrow_function_scope(&mut self, node: &Arc<Node>) {
        self.arrow_function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    /// Pop an arrow function scope.
    fn pop_arrow_function_scope(&mut self) {
        self.arrow_function_scope_count -= 1;
        self.scope_stack.pop();
    }

    /// Pop the innermost scope from the scope stack.
    pub(crate) fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Resolve an identifier reference by walking the scope stack from
    /// innermost to outermost, checking each container's symbol members and
    /// locals.
    ///
    /// Since the parser does not set parent pointers on nodes, we cannot walk
    /// the parent chain. Instead, we use the scope stack maintained by the
    /// checker during AST traversal.
    ///
    /// Returns the resolved symbol, or `None` if not found.
    ///
    /// Go: `Checker.resolveName` (reduced form — does not yet handle globals,
    /// namespaces, or alias chasing).
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    /// Resolve an identifier with a specific meaning (symbol flags filter).
    ///
    /// Only symbols whose flags intersect with `meaning` are considered.
    /// This mirrors the `meaning` parameter in Go's `NameResolver.Resolve`.
    ///
    /// Go: `NameResolver.Resolve` — scope stack walk → module/enum exports →
    /// class/interface type params → globals → `arguments` → `require`.
    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        // 1. Walk the scope stack from innermost to outermost.
        for &container_id in self.scope_stack.iter().rev() {
            // Check the container's locals (block-scoped variables).
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }
            // Check the container's symbol members (function-scoped
            // declarations like parameters, class members, etc.).
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }
                // Module/namespace export lookup.
                if container_sym.flags.intersects(SymbolFlags::MODULE) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        // Skip pure alias export specifiers (e.g. `export { X }`).
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
                // Enum export lookup.
                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }
                // Class/Interface/Function/Constructor type parameter lookup.
                // Type parameters declared by the container (e.g.
                // `<T extends U>` on a class, interface, function, or
                // constructor) are accessible in the container's scope.
                if container_sym.flags.intersects(
                    SymbolFlags::Class
                        | SymbolFlags::Interface
                        | SymbolFlags::Function
                        | SymbolFlags::Constructor
                        | SymbolFlags::ValueModule,
                ) {
                    if let Some(sym) = container_sym.members.get(name) {
                        if sym.flags.intersects(meaning & SymbolFlags::TYPE) {
                            return self.follow_alias(sym);
                        }
                    }
                }
            }
        }

        // 2. Check for special built-in symbols.
        // `arguments` is in scope inside function declarations, but not arrow functions.
        if self.function_scope_count > 0
            && name == "arguments"
            && meaning.intersects(SymbolFlags::VARIABLE)
        {
            if let Some(ref sym) = self.arguments_symbol {
                return Some(Arc::clone(sym));
            }
        }

        // 3. Check globals (lib.d.ts symbols).
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

    /// Follow an alias symbol chain to find the resolved target.
    ///
    /// An alias symbol (created by import/export declarations) has
    /// `SymbolFlags::Alias` and its `export_symbol` points to the target.
    /// This method follows the chain until a non-alias target is found.
    ///
    /// Go: `Checker.resolveAlias` / `Checker.resolveSymbol`
    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(Arc::clone(symbol));
        }
        // Check if it's a pure alias (not combined with other flags).
        // Go: IsNonLocalAlias — pure alias or alias with Assignment flag.
        let is_pure_alias = symbol.flags == SymbolFlags::Alias
            || (symbol.flags.intersects(SymbolFlags::Alias)
                && symbol.flags.intersects(SymbolFlags::Assignment));
        if !is_pure_alias {
            return Some(Arc::clone(symbol));
        }
        // Follow the export_symbol chain.
        // Simple implementation without cycle detection (aliases are not
        // created yet in the current binder).
        let mut current = Arc::clone(symbol);
        loop {
            if let Some(ref target) = current.export_symbol {
                let is_pure = target.flags == SymbolFlags::Alias
                    || (target.flags.intersects(SymbolFlags::Alias)
                        && target.flags.intersects(SymbolFlags::Assignment));
                if is_pure {
                    current = Arc::clone(target);
                    continue;
                }
                return Some(Arc::clone(target));
            } else {
                // No export_symbol, return the alias itself.
                return Some(Arc::clone(&current));
            }
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // ReferenceResolver — ported from `internal/binder/referenceresolver.go`
    // ────────────────────────────────────────────────────────────────────────

    /// Get the resolved symbol for a value reference.
    ///
    /// Go: `referenceResolver.getReferencedValueSymbol`
    fn get_referenced_value_symbol(
        &self,
        node: &Node,
        start_in_declaration_container: bool,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

        // Check if the node already has a resolved symbol.
        if let Some(sym) = symbol_map.symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        let location = if start_in_declaration_container {
            // When the node is the name of a module/enum declaration,
            // start resolution from the declaration's container.
            // We use the scope stack for resolution, so the container ID
            // is already on the scope stack when we enter the module/enum.
            node
        } else {
            node
        };

        // Use the enhanced name resolver to find the symbol.
        let meaning = SymbolFlags::ExportValue
            .union(SymbolFlags::VALUE)
            .union(SymbolFlags::Alias);
        self.resolve_identifier_at_location(location, node_name(node)?, meaning)
    }

    /// Find the parent declaration container for a module/enum name.
    fn find_parent_declaration_container(&self, _node: &Node) -> Option<u64> {
        // We don't have parent pointers, so we check the scope stack.
        // The innermost scope container is the parent declaration container.
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

    /// Get the export container for a referenced value.
    ///
    /// Go: `referenceResolver.GetReferencedExportContainer`
    pub fn get_referenced_export_container(&self, node: &Node, prefix_locals: bool) -> Option<u64> {
        // If the node is the name of a module/enum declaration, start in
        // the declaration container.
        let start_in_declaration_container = is_module_or_enum_name(node);
        if let Some(symbol) = self.get_referenced_value_symbol(node, start_in_declaration_container)
        {
            if symbol.flags.intersects(SymbolFlags::ExportValue) {
                if let Some(ref export_symbol) = symbol.export_symbol {
                    if let Some(merged) = self.get_merged_symbol(export_symbol) {
                        if !prefix_locals
                            && merged.flags.intersects(SymbolFlags::EXPORT_HAS_LOCAL)
                            && !merged.flags.intersects(SymbolFlags::VARIABLE)
                        {
                            return None;
                        }
                        // Find the parent symbol (the container).
                        if let Some(parent) = &merged.parent {
                            if parent.flags.intersects(SymbolFlags::ValueModule)
                                && parent.value_declaration.is_some()
                            {
                                return Some(parent.value_declaration.as_ref().unwrap().id());
                            }
                            // Find the matching container in the scope stack.
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
        }
        None
    }

    /// Get the import declaration for a referenced value.
    ///
    /// Go: `referenceResolver.GetReferencedImportDeclaration`
    pub fn get_referenced_import_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            // Only get the declaration of an alias if there isn't a local value declaration.
            if is_non_local_alias(&symbol, SymbolFlags::VALUE)
                && !self.is_type_only_alias_declaration(&symbol)
            {
                return self.get_declaration_of_alias_symbol(&symbol);
            }
        }
        None
    }

    /// Get the value declaration for a referenced value.
    ///
    /// Go: `referenceResolver.GetReferencedValueDeclaration`
    pub fn get_referenced_value_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
            return export_sym.value_declaration.clone();
        }
        None
    }

    /// Get all value declarations for a referenced value.
    ///
    /// Go: `referenceResolver.GetReferencedValueDeclarations`
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

    /// Get the name from an element access expression.
    ///
    /// Go: `referenceResolver.GetElementAccessExpressionName`
    pub fn get_element_access_expression_name(&self, expression: &Node) -> Option<String> {
        if expression.kind == SyntaxKind::ElementAccessExpression {
            if let crate::ast::NodeData::ElementAccessExpression(data) = &expression.data {
                // Try to get a string literal key.
                if let crate::ast::NodeData::StringLiteral(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }
                // Try to get a numeric literal key.
                if let crate::ast::NodeData::NumericLiteral(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }
                // Try to resolve an identifier key.
                if let crate::ast::NodeData::Identifier(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }
            }
        }
        None
    }

    /// Get the member value declaration for a member reference.
    ///
    /// Go: `referenceResolver.GetReferencedMemberValueDeclaration`
    pub fn get_referenced_member_value_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        // Member references are `this.something` or `this[something]`.
        // They should always have a resolved symbol.
        let symbol_map = self.program.symbol_map();
        let s = symbol_map.symbol_of(node).map(|s| Arc::clone(s));
        if s.is_none() {
            // Might be a declaration instead of a ref, get the merged symbol.
            if let Some(sym) = symbol_map.symbol_of(node) {
                let merged = self.get_merged_symbol(sym);
                if let Some(merged) = merged {
                    let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&merged);
                    return export_sym.value_declaration.clone();
                }
            }
        }
        if let Some(ref s) = s {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(s);
            return export_sym.value_declaration.clone();
        }
        None
    }

    /// Get the merged symbol.
    ///
    /// Go: `referenceResolver.getMergedSymbol`
    fn get_merged_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if let Some(target_id) = self.merged_symbols.get(&symbol.id()) {
            // We need a way to look up symbols by ID. For now, return the symbol itself.
            // In a full implementation, we'd have a symbol_by_id map.
        }
        Some(Arc::clone(symbol))
    }

    /// Get the export symbol of a value symbol if it's exported.
    ///
    /// Go: `referenceResolver.getExportSymbolOfValueSymbolIfExported`
    fn get_export_symbol_of_value_symbol_if_exported(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        let mut result = Arc::clone(symbol);
        if symbol.flags.intersects(SymbolFlags::ExportValue) {
            if let Some(ref export_sym) = symbol.export_symbol {
                result = self
                    .get_merged_symbol(export_sym)
                    .unwrap_or(Arc::clone(export_sym));
            }
        }
        result
    }

    /// Check if a symbol is a type-only alias declaration.
    ///
    /// Go: `referenceResolver.isTypeOnlyAliasDeclaration`
    fn is_type_only_alias_declaration(&self, symbol: &Arc<Symbol>) -> bool {
        if let Some(node) = self.get_declaration_of_alias_symbol(symbol) {
            let mut current = Some(Arc::clone(&node));
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
                        // Continue to parent - we need parent pointers for this.
                        // Without parent pointers, we stop here.
                        break;
                    }
                    _ => break,
                }
            }
        }
        false
    }

    /// Get the declaration of an alias symbol.
    ///
    /// Go: `referenceResolver.getDeclarationOfAliasSymbol`
    fn get_declaration_of_alias_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
        // Find the last alias symbol declaration.
        symbol
            .declarations
            .iter()
            .filter(|d| is_alias_symbol_declaration(d))
            .last()
            .cloned()
    }

    /// Resolve an identifier at a specific location.
    fn resolve_identifier_at_location(
        &self,
        location: &Node,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        // Use the scope stack to resolve the name.
        let symbol_map = self.program.symbol_map();

        // Walk the scope stack from innermost to outermost.
        for &container_id in self.scope_stack.iter().rev() {
            // Check locals.
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }
            // Check members.
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }
                // Module/namespace exports.
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
                // Enum exports.
                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }
            }
        }

        // Check globals.
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

    // ────────────────────────────────────────────────────────────────────────
    // Declaration merging — ported from `internal/checker/checker.go`
    // ────────────────────────────────────────────────────────────────────────

    /// Merge a source symbol table into a target symbol table.
    ///
    /// Go: `Checker.mergeSymbolTable`
    pub fn merge_symbol_table(
        &mut self,
        target: &mut SymbolTable,
        source: &SymbolTable,
        unidirectional: bool,
        merged_parent: Option<u64>,
    ) {
        // Collect entries to merge to avoid borrow issues
        let entries: Vec<(String, Arc<Symbol>)> = source
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        for (name, source_symbol) in entries {
            if let Some(target_symbol) = target.entries.get_mut(&name) {
                // Merge the existing target symbol with the source
                let merged = self.merge_symbol(target_symbol, &source_symbol, unidirectional);
                let is_transient = merged.flags.intersects(SymbolFlags::Transient);
                *target_symbol = merged;
                if let Some(_parent_id) = merged_parent {
                    if is_transient {
                        // Set parent to the merged parent
                        // Simplified: we skip parent tracking for now
                    }
                }
            } else {
                // No existing target symbol, use the merged version of source
                let merged = self.get_merged_symbol(&source_symbol);
                target.insert(name, merged.unwrap_or_else(|| Arc::clone(&source_symbol)));
            }
        }
    }

    /// Merge a source symbol into a target symbol.
    ///
    /// Go: `Checker.mergeSymbol`
    /// Returns a new merged symbol (Arc<Symbol>).
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
            // Determine the effective target (clone if not transient)
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
                    // Cannot merge — return source
                    return Arc::clone(source);
                }
            } else {
                Arc::clone(target)
            };

            // Build the merged symbol by creating a new one
            let mut source_flags = source.flags;
            if !effective_target
                .flags
                .intersects(SymbolFlags::ConstEnumOnlyModule)
            {
                source_flags.remove(SymbolFlags::ConstEnumOnlyModule);
            }
            let merged_flags = effective_target.flags | source_flags;

            let mut merged = Symbol::new(merged_flags, &effective_target.name);
            // Copy value declaration (source takes priority)
            merged.value_declaration = source
                .value_declaration
                .clone()
                .or_else(|| effective_target.value_declaration.clone());
            // Merge declarations
            merged.declarations = effective_target.declarations.clone();
            merged
                .declarations
                .extend(source.declarations.iter().cloned());
            // Copy parent
            merged.parent = effective_target.parent.clone();
            // Copy members and exports
            merged.members = SymbolTable {
                entries: effective_target.members.entries.clone(),
            };
            merged.exports = SymbolTable {
                entries: effective_target.exports.entries.clone(),
            };

            let result = Arc::new(merged);

            // Merge members and exports
            // We need to mutate the result's members and exports
            // Since result is behind Arc, we use a workaround:
            // Create a mutable temporary, merge, then create new Arc
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

            // Merge source members into target members
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
                    result_mut
                        .members
                        .insert(name, merged.unwrap_or_else(|| Arc::clone(&source_sym)));
                }
            }

            // Merge source exports into target exports
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
                    result_mut
                        .exports
                        .insert(name, merged.unwrap_or_else(|| Arc::clone(&source_sym)));
                }
            }

            let final_result = Arc::new(result_mut);

            if !unidirectional {
                self.record_merged_symbol(&final_result, source);
            }

            final_result
        } else {
            // Cannot merge — return target as-is
            Arc::clone(target)
        }
    }

    /// Record that a source symbol was merged into a target symbol.
    ///
    /// Go: `Checker.recordMergedSymbol`
    pub fn record_merged_symbol(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        self.merged_symbols.insert(source.id(), target.id());
    }

    /// Clone a symbol (creates a transient copy).
    ///
    /// Go: `Checker.cloneSymbol`
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

    /// Resolve a symbol (follow alias chains).
    ///
    /// Go: `Checker.resolveSymbol`
    pub fn resolve_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(result) = self.follow_alias(symbol) {
            result
        } else {
            Arc::clone(symbol)
        }
    }
}

/// Get the excluded symbol flags for a given set of flags.
///
/// Go: `getExcludedSymbolFlags`
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

/// Check if a node is a module or enum declaration name./// Check if a node is a module or enum declaration name.
fn is_module_or_enum_name(node: &Node) -> bool {
    // We can't check parent pointers, so we check if the node's kind
    // suggests it's a name of a module/enum declaration.
    // This is a best-effort check.
    false
}

/// Check if a symbol is a non-local alias (pure alias with exclusions).
///
/// Go: `ast.IsNonLocalAlias`
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

/// Check if a node is a type-only declaration.
///
/// Go: `ast.Node.IsTypeOnly`
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

/// Check if a node is an alias symbol declaration.
///
/// Go: `ast.IsAliasSymbolDeclaration`
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

/// Get the name text of a node.
///
/// Go: `ast.Node.Name`
fn node_name(node: &Node) -> Option<&str> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::Identifier(data) => Some(&data.text),
        NodeData::StringLiteral(data) => Some(&data.text),
        NodeData::NumericLiteral(data) => Some(&data.text),
        _ => None,
    }
}

/// Whether a syntax kind is an expression-position kind.
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

/// Whether a syntax kind is a statement kind.
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

/// Whether an identifier node is the *name* of a declaration rather than a
/// reference. We detect this by inspecting the parent node.
fn is_declaration_name(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    let parent_kind = parent.kind;
    // For declaration nodes whose `name` field is an identifier, the
    // identifier is a declaration name, not a reference.
    // NOTE: `ShorthandPropertyAssignment` is intentionally excluded — its
    // name is *also* a reference to an outer-scope variable (`{ x }`
    // references `x`), so it must be checked as a reference (and emit
    // TS2304 when the name is unresolvable).
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
            );
        }
    }
    false
}

/// Whether an identifier node is the property name on the right of a property
/// access (`x.foo` — `foo` is the name, not a reference).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_store_basic() {
        let store: LinkStore<Node, NodeLinks> = LinkStore::new();
        // We can't easily create a Node for testing, but we can verify the API
        // works with the Default impl.
        assert!(store.data.is_empty());
    }

    #[test]
    fn ternary_and_or() {
        assert_eq!(Ternary::True.and(Ternary::False), Ternary::False);
        assert_eq!(Ternary::True.or(Ternary::False), Ternary::True);
    }
}
