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
    CheckFlags, DiagnosticsCollection, ModifierFlags, Node, NodeData, NodeFlags, NodeSymbolMap,
    SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::diagnostics::messages_generated::{CANNOT_FIND_NAME_0, TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1};
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
    /// Recursion depth of `is_type_related_to`. Capped at
    /// `RELATER_MAX_DEPTH` to prevent stack overflow on recursive
    /// structural types such as `type Box<T> = { next: Box<T> | null }`.
    /// Mirrors `Checker.relationStackDepth` in Go (relater.go).
    pub relater_depth: u32,
    pub spread_links: LinkStore<Symbol, SpreadLinks>,
    pub variance_links: LinkStore<Symbol, VarianceLinks>,
    pub reverse_mapped_symbol_links: LinkStore<Symbol, ReverseMappedSymbolLinks>,
    pub marked_assignment_symbol_links: LinkStore<Symbol, MarkedAssignmentSymbolLinks>,
    pub symbol_container_links: LinkStore<Symbol, ContainingSymbolLinks>,
    pub source_file_links: LinkStore<SourceFile, SourceFileLinks>,

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

    // Flow analysis
    pub flow_analysis_disabled: bool,
    pub flow_invocation_count: i32,
    pub flow_type_cache: HashMap<u64, Arc<Type>>,
    pub flow_node_reachable: HashMap<u64, bool>,

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
            undefined_symbol: Some(Arc::new(Symbol::new(
                SymbolFlags::Property,
                "undefined",
            ))),
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
            relater_depth: 0,
            spread_links: LinkStore::new(),
            variance_links: LinkStore::new(),
            reverse_mapped_symbol_links: LinkStore::new(),
            marked_assignment_symbol_links: LinkStore::new(),
            symbol_container_links: LinkStore::new(),
            source_file_links: LinkStore::new(),

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

            flow_analysis_disabled: false,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            flow_node_reachable: HashMap::new(),

            merged_symbols: HashMap::new(),

            tracer,
            mu: Mutex::new(()),
    };

        // Initialize global_this_symbol and add built-in symbols to globals
        {
            // Create globalThis symbol
            let mut global_this = Symbol::new(
                SymbolFlags::ValueModule,
                "globalThis",
            );
            global_this.check_flags = CheckFlags::Readonly;
            let global_this = Arc::new(global_this);
            checker.globals
                .insert("globalThis".to_string(), Arc::clone(&global_this));
            checker.global_this_symbol = Some(global_this);

            // Add undefined to globals
            if let Some(ref undef) = checker.undefined_symbol {
                checker.globals
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
    pub fn get_combined_modifier_flags(&mut self, node: &Arc<Node>) -> ModifierFlags {
        let flags = ModifierFlags::empty();
        let mut current = Some(Arc::clone(node));
        while let Some(n) = current {
            current = n.parent.clone();
        }
        flags
    }
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
    /// base types. Mirrors Go's `getWidenedType`/`getWidenedLiteralType`
    /// (checker.go ~L18268/L25395) for the function-return-type use case.
    ///
    /// - Literal types (string/number/bigint/boolean) → their primitive base.
    /// - Unique `symbol` literals → `symbol`.
    /// - Unions → a new union with each constituent widened (nullable
    ///   constituents are preserved as-is, matching Go).
    /// - All other types are returned unchanged.
    ///
    /// Freshness tracking (`freshType`) is not yet implemented in the Rust
    /// port, so this widens *all* literal types. This matches TypeScript's
    /// observable behavior for the common cases (e.g. `function f() { return
    /// 42; }` infers `number`).
    pub fn get_widened_type(&self, t: &Arc<Type>) -> Arc<Type> {
        // Nullable types are not widened (Go skips them in union widening).
        if t.flags.intersects(TYPE_FLAGS_NULLABLE) {
            return Arc::clone(t);
        }
        // Literal types → primitive base.
        if t.flags.intersects(TYPE_FLAGS_LITERAL) {
            return self.get_base_type_of_literal_type(t);
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
        let decl = sig.declaration.as_ref()?;
        let type_node = decl.type_node()?;
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
            // Literal types
            SyntaxKind::NumericLiteral => {
                if let crate::ast::NodeData::NumericLiteral(data) = &node.data {
                    return self.infer_number_literal_type(&data.text);
                }
                self.number_type()
            }
            SyntaxKind::StringLiteral => {
                if let crate::ast::NodeData::StringLiteral(data) = &node.data {
                    return self.infer_string_literal_type(&data.text);
                }
                self.string_type()
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                self.string_type()
            }
            SyntaxKind::TrueKeyword => {
                self.true_type()
            }
            SyntaxKind::FalseKeyword => {
                self.false_type()
            }
            SyntaxKind::NullKeyword => {
                self.null_type()
            }
            SyntaxKind::UndefinedKeyword => {
                self.undefined_type()
            }
            SyntaxKind::BigIntLiteral => {
                self.bigint_type()
            }
            SyntaxKind::ArrayLiteralExpression => {
                return self.get_type_of_array_literal(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                // TODO: infer object type from properties
                self.get_any_type()
            }
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                self.get_type_of_function_like(node)
            }
            SyntaxKind::FunctionDeclaration => {
                self.get_type_of_function_like(node)
            }
            SyntaxKind::Identifier => {
                self.get_type_of_identifier(node)
            }
            // Binary expressions
            SyntaxKind::BinaryExpression => {
                self.get_type_of_binary_expression(node)
            }
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
            SyntaxKind::CallExpression => {
                self.get_return_type_of_call_expression(node)
            }
            SyntaxKind::NewExpression => {
                self.get_return_type_of_new_expression(node)
            }
            SyntaxKind::PropertyAccessExpression => {
                self.get_type_of_property_access(node)
            }
            SyntaxKind::ElementAccessExpression => {
                self.get_type_of_element_access(node)
            }
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
                // `x!` has the type of `x` (ideally with null/undefined
                // removed, but for now we return the expression type).
                if let crate::ast::NodeData::NonNullExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
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
            _ => {
                self.get_any_type()
            }
        }
    }

    /// Get the type of an identifier reference.
    ///
    /// If the identifier has an associated flow node (set by the binder),
    /// the declared type is narrowed based on control-flow constraints
    /// (e.g. `if (x !== null)` narrows `x` in the then-branch).
    fn get_type_of_identifier(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(symbol) = self.resolve_identifier(node) {
            let flow = self
                .program
                .symbol_map()
                .flow_node_of(node)
                .map(Arc::clone);
            self.get_narrowed_type_of_symbol(&symbol, flow.as_ref())
        } else {
            self.get_any_type()
        }
    }

    /// Get the type of a symbol.
    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // For now, return any for most symbols
        // TODO: implement proper symbol type resolution
        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            || symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            || symbol.flags.contains(SymbolFlags::Function)
            || symbol.flags.contains(SymbolFlags::Class)
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
        } else {
            self.get_any_type()
        }
    }

    /// Get the type of a function-like expression (FunctionExpression /
    /// ArrowFunction). Returns an anonymous object type whose single call
    /// signature has the inferred (or annotated) return type.
    fn get_type_of_function_like(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (body, type_node) = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => (Some(&data.body), data.type_node.as_ref()),
            crate::ast::NodeData::ArrowFunction(data) => (Some(&data.body), data.type_node.as_ref()),
            crate::ast::NodeData::FunctionDeclaration(data) => (data.body.as_ref(), data.type_node.as_ref()),
            _ => return self.get_any_type(),
        };
        let return_type = self.infer_function_return_type(body, type_node);
        self.create_function_type(return_type)
    }

    /// Get the return type of a `CallExpression`. Resolves the called
    /// expression's type; if it's a function type with at least one call
    /// signature, return that signature's resolved return type. Otherwise
    /// fall back to `any`.
    fn get_return_type_of_call_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let callee = match &node.data {
            crate::ast::NodeData::CallExpression(data) => &data.expression,
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(callee);
        if let Some(structured) = callee_type.as_structured() {
            for sig in structured.call_signatures() {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    return rt;
                }
                // Signature without a resolved return type — fall back to
                // any so callers don't blow up. The full Go checker would
                // run inference here; that's P3.8c.
                return self.get_any_type();
            }
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
                PlusToken | MinusToken | AsteriskToken | SlashToken | PercentToken
                | AsteriskAsteriskToken | LessThanLessThanToken
                | GreaterThanGreaterThanToken | GreaterThanGreaterThanGreaterThanToken
                | AmpersandToken | BarToken | CaretToken => {
                    self.number_type()
                }
                // Comparison operators return boolean
                LessThanToken | GreaterThanToken | LessThanEqualsToken
                | GreaterThanEqualsToken | EqualsEqualsToken
                | ExclamationEqualsToken | EqualsEqualsEqualsToken
                | ExclamationEqualsEqualsToken | InKeyword | InstanceOfKeyword => {
                    self.boolean_type()
                }
                // Logical operators return union of operands (simplified)
                AmpersandAmpersandToken | BarBarToken | QuestionQuestionToken => {
                    self.get_type_of_node(&data.left)
                }
                // Assignment operators return the right-hand side type
                EqualsToken | PlusEqualsToken | MinusEqualsToken
                | AsteriskEqualsToken | SlashEqualsToken | PercentEqualsToken
                | AsteriskAsteriskEqualsToken
                | LessThanLessThanEqualsToken
                | GreaterThanGreaterThanEqualsToken
                | GreaterThanGreaterThanGreaterThanEqualsToken
                | AmpersandEqualsToken | BarEqualsToken | CaretEqualsToken
                | BarBarEqualsToken | AmpersandAmpersandEqualsToken
                | QuestionQuestionEqualsToken => {
                    self.get_type_of_node(&data.right)
                }
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
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                (&data.expression, &data.name)
            }
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
    fn get_array_element_type(&self, t: &Arc<Type>) -> Arc<Type> {
        match &t.data {
            crate::checker::TypeData::Object(obj) => {
                // `Array<T>` is a reference type with one type argument.
                if let Some(elem) = obj.type_arguments.first() {
                    return Arc::clone(elem);
                }
                self.get_any_type()
            }
            crate::checker::TypeData::EvolvingArray(ea) => {
                ea.element_type.clone().unwrap_or_else(|| self.get_any_type())
            }
            _ => self.get_any_type(),
        }
    }

    /// Try to extract a constant numeric value from a literal expression.
    fn get_constant_numeric_value(&self, node: &Arc<Node>) -> Option<f64> {
        match &node.data {
            crate::ast::NodeData::NumericLiteral(data) => {
                data.text.parse::<f64>().ok()
            }
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
            return self.create_array_type(self.get_any_type());
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

    /// Widen a literal type to its base type (e.g. `3` → `number`).
    fn get_widened_type_of_literal(&self, t: &Arc<Type>) -> Arc<Type> {
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
                    self.break_continue_context_stack.push(BreakContinueContext {
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
                    self.break_continue_context_stack.push(BreakContinueContext {
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
                    self.break_continue_context_stack.push(BreakContinueContext {
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
                    self.break_continue_context_stack.push(BreakContinueContext {
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
                    self.break_continue_context_stack.push(BreakContinueContext {
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
                // Only check the function body; the name, parameters,
                // type parameters, and return type are declarations/types.
                self.push_function_scope(node);
                self.break_continue_context_stack.push(BreakContinueContext {
                    kind: BreakContinueContextKind::Function,
                    label: None,
                    is_iteration: false,
                });
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.break_continue_context_stack.pop();
                self.pop_function_scope();
                // Compute the function's type (with inferred return type)
                // and cache it on the declaration node + symbol so later
                // references (e.g. `let y = f()`) can recover it. Mirrors
                // Go's `getSymbolLinks(symbol).type = getWidenedTypeOfFunction`.
                let fn_type = self.get_type_of_function_like(node);
                self.type_node_links
                    .get_or_default(node)
                    .resolved_type = Some(fn_type.clone());
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        if let Some(symbol) = self.resolve_identifier(name) {
                            self.value_symbol_links
                                .get_or_default(&symbol)
                                .resolved_type = Some(fn_type.clone());
                            self.type_node_links
                                .get_or_default(name)
                                .resolved_type = Some(fn_type);
                        }
                    }
                }
            }
            SyntaxKind::ClassDeclaration => {
                // Grammar check: validate modifiers.
                self.check_grammar_modifiers(node);
                // Check heritage clauses (e.g. `extends Foo`).
                self.push_scope(node);
                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(heritage) = &data.heritage_clauses {
                        for clause in heritage.iter() {
                            self.check_heritage_clause(clause);
                        }
                    }
                    // Check member initializers.
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                }
                self.pop_scope();
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
                (None, Some(init)) => self.get_type_of_node(init),
                (None, None) => self.get_any_type(),
            };
            // Cache the resolved type on the VariableDeclaration node — this
            // is what `symbol.value_declaration` points to, so
            // `get_type_of_symbol` can recover the type via `type_node_links`.
            // (Previously this was stored on `data.name`, the Identifier
            // child node, which `get_type_of_symbol` never inspects.)
            self.type_node_links
                .get_or_default(node)
                .resolved_type = Some(resolved_type.clone());
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
        // These are type references, not expression references, so we skip
        // them for now. Full checking will resolve the type later.
    }

    fn check_class_member(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                // Only check the body; the name, parameters, and return type
                // are declarations/types.
                let body: Option<Arc<Node>> = match &node.data {
                    crate::ast::NodeData::MethodDeclaration(d) => d.body.clone(),
                    crate::ast::NodeData::ConstructorDeclaration(d) => d.body.clone(),
                    crate::ast::NodeData::GetAccessorDeclaration(d) => d.body.clone(),
                    crate::ast::NodeData::SetAccessorDeclaration(d) => d.body.clone(),
                    _ => None,
                };
                if let Some(body) = body {
                    self.push_function_scope(node);
                    match body.kind {
                        SyntaxKind::Block => self.check_statement(&body),
                        _ => self.check_expression(&body),
                    }
                    self.pop_function_scope();
                }
            }
            SyntaxKind::PropertyDeclaration => {
                // Only check the initializer; the name and type are
                // declarations/types.
                if let crate::ast::NodeData::PropertyDeclaration(data) = &node.data {
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
            }
            SyntaxKind::PropertyAccessExpression => {
                // Only check the left side; the right side is a property name,
                // not an identifier reference.
                if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
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
            SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
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
                        crate::ast::NodeData::JsxFragment(d) => Some(Arc::clone(&d.opening_fragment)),
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
        // Walk children, but skip parameter names (they are declarations).
        // The simplest correct approach is to dispatch on the body only.
        let body: Option<Arc<Node>> = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => Some(data.body.clone()),
            crate::ast::NodeData::ArrowFunction(data) => Some(data.body.clone()),
            _ => None,
        };
        if let Some(body) = body {
            // Arrow functions do not have their own `arguments` object.
            let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
            if is_arrow {
                self.push_arrow_function_scope(node);
            } else {
                self.push_function_scope(node);
            }
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => self.check_expression(&body),
            }
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
            crate::ast::NodeData::JsxElement(data) => {
                data.children.iter().cloned().collect()
            }
            crate::ast::NodeData::JsxFragment(data) => {
                data.children.iter().cloned().collect()
            }
            _ => Vec::new(),
        };

        // Walk attributes (skip tag_name and closing tag_name).
        if let Some(opening) = opening_element {
            let attributes: Option<Arc<Node>> = match &opening.data {
                crate::ast::NodeData::JsxOpeningElement(data) => {
                    Some(Arc::clone(&data.attributes))
                }
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
        let diagnostic = crate::ast::Diagnostic::new(
            file,
            node.loc,
            CANNOT_FIND_NAME_0,
            vec![name.to_string()],
        );
        self.diagnostics.add(diagnostic);
    }

    /// Push a container node onto the scope stack, making its symbol members
    /// and locals visible for identifier resolution.
    fn push_scope(&mut self, node: &Arc<Node>) {
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
    fn pop_scope(&mut self) {
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
                            && sym.declarations.iter().any(|d| d.kind == SyntaxKind::ExportSpecifier);
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
                // Class/Interface type parameter lookup.
                if container_sym.flags.intersects(SymbolFlags::Class)
                    || container_sym.flags.intersects(SymbolFlags::Interface)
                {
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
            if sym.flags.intersects(meaning.union(SymbolFlags::GlobalLookup)) {
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
                if container_sym.flags.intersects(SymbolFlags::MODULE | SymbolFlags::ENUM) {
                    return Some(container_id);
                }
            }
        }
        None
    }

    /// Get the export container for a referenced value.
    ///
    /// Go: `referenceResolver.GetReferencedExportContainer`
    pub fn get_referenced_export_container(
        &self,
        node: &Node,
        prefix_locals: bool,
    ) -> Option<u64> {
        // If the node is the name of a module/enum declaration, start in
        // the declaration container.
        let start_in_declaration_container = is_module_or_enum_name(node);
        if let Some(symbol) = self.get_referenced_value_symbol(node, start_in_declaration_container) {
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
                result = self.get_merged_symbol(export_sym).unwrap_or(Arc::clone(export_sym));
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
        symbol.declarations.iter()
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
                            && sym.declarations.iter().any(|d| d.kind == SyntaxKind::ExportSpecifier);
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
            if sym.flags.intersects(meaning.union(SymbolFlags::GlobalLookup)) {
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
        let entries: Vec<(String, Arc<Symbol>)> = source.iter().map(|(k, v)| (k.clone(), Arc::clone(v))).collect();
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
                if resolved_target.flags.intersects(get_excluded_symbol_flags(source.flags)) == false
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
            if !effective_target.flags.intersects(SymbolFlags::ConstEnumOnlyModule) {
                source_flags.remove(SymbolFlags::ConstEnumOnlyModule);
            }
            let merged_flags = effective_target.flags | source_flags;

            let mut merged = Symbol::new(merged_flags, &effective_target.name);
            // Copy value declaration (source takes priority)
            merged.value_declaration = source.value_declaration.clone()
                .or_else(|| effective_target.value_declaration.clone());
            // Merge declarations
            merged.declarations = effective_target.declarations.clone();
            merged.declarations.extend(source.declarations.iter().cloned());
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
            let mut result_mut = Symbol::new(
                result.flags,
                &result.name,
            );
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
            let source_members: Vec<(String, Arc<Symbol>)> = source.members.iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_members {
                if let Some(target_sym) = result_mut.members.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.members.insert(name, merged.unwrap_or_else(|| Arc::clone(&source_sym)));
                }
            }

            // Merge source exports into target exports
            let source_exports: Vec<(String, Arc<Symbol>)> = source.exports.iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_exports {
                if let Some(target_sym) = result_mut.exports.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.exports.insert(name, merged.unwrap_or_else(|| Arc::clone(&source_sym)));
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
                    | SyntaxKind::ShorthandPropertyAssignment
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
    std::ptr::eq(name_field.as_ref() as *const Node, node.as_ref() as *const Node)
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
