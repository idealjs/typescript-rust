//! The type checker.
//!
//! Ported from `internal/checker/checker.go`. This is the largest and most
//! complex module in the compiler (~32K lines in Go). This file provides
//! the `Checker` struct, its initialization, and the core entry points.
//! Full type-checking logic is added incrementally.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::ast::{
    DiagnosticsCollection, ModifierFlags, Node, NodeFlags, NodeSymbolMap, SourceFile, Symbol,
    SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
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

    // Flow analysis
    pub flow_analysis_disabled: bool,
    pub flow_invocation_count: i32,
    pub flow_type_cache: HashMap<u64, Arc<Type>>,
    pub flow_node_reachable: HashMap<u64, bool>,

    // Tracer
    pub tracer: Arc<Tracer>,

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

        Self {
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
            undefined_symbol: None,
            arguments_symbol: None,
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

            flow_analysis_disabled: false,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            flow_node_reachable: HashMap::new(),

            tracer,
            mu: Mutex::new(()),
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

    // Type conversion
    pub fn type_to_string(&self, t: &Arc<Type>) -> String {
        super::utilities::type_to_string(t)
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
        let file_node = Arc::clone(&file.node);
        let file_id = file_node.id();
        let source_file_symbol = self.program.symbol_map().symbol_of(&file_node).cloned();

        // Save file context for diagnostics.
        let file_arc = Arc::clone(file);
        self.current_file = Some(Arc::clone(&file_arc));
        self.current_file_id = file_id;
        self.current_file_symbol = source_file_symbol;

        // Walk top-level statements.
        let statements: Vec<Arc<Node>> = match &file_node.data {
            crate::ast::NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
            _ => Vec::new(),
        };
        for stmt in &statements {
            self.check_statement(stmt);
        }

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
                    self.check_variable_declaration_list(&data.declaration_list);
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
                    self.check_statement(&data.statement);
                }
            }
            SyntaxKind::DoStatement => {
                if let crate::ast::NodeData::DoStatement(data) = &node.data {
                    self.check_statement(&data.statement);
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ForStatement => {
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
                    self.check_statement(&data.statement);
                }
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                if let crate::ast::NodeData::ForInOrOfStatement(data) = &node.data {
                    self.check_for_initializer(&data.initializer);
                    self.check_expression(&data.expression);
                    self.check_statement(&data.statement);
                }
            }
            SyntaxKind::ReturnStatement => {
                if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            SyntaxKind::Block => {
                if let crate::ast::NodeData::Block(data) = &node.data {
                    for stmt in data.statements.iter() {
                        self.check_statement(stmt);
                    }
                }
            }
            SyntaxKind::ThrowStatement => {
                if let crate::ast::NodeData::ThrowStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    for case in data.clauses.iter() {
                        self.check_case_clause(case);
                    }
                }
            }
            // Declarations: a full checker would compute types and check
            // initializers. For now, only check initializer expressions so
            // that identifier references are visited.
            SyntaxKind::VariableDeclaration => {
                self.check_variable_declaration(node);
            }
            SyntaxKind::FunctionDeclaration
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
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier => {
                // Walk children to find expression-position identifiers.
                // Type-position identifiers are skipped by check_expression.
                self.walk_children_for_expressions(node);
            }
            SyntaxKind::EmptyStatement | SyntaxKind::BreakStatement | SyntaxKind::ContinueStatement => {
                // No expressions to check.
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
            // The name is a declaration, not a reference — skip it.
            // The type annotation is in type position — skip it.
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }
        }
    }

    fn check_case_clause(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::CaseClause(data) = &node.data {
            if let Some(expr) = &data.expression {
                self.check_expression(expr);
            }
            for stmt in data.statements.iter() {
                self.check_statement(stmt);
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
                    if let Some(arg) = &data.argument_expression {
                        self.check_expression(arg);
                    }
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
                        if let Some(expr) = span.expression.as_ref() {
                            self.check_expression(expr);
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
                    if let Some(template) = &data.template {
                        self.check_expression(template);
                    }
                }
            }
            SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
                // JSX expressions contain child identifiers in property names
                // and tag names that are not references. Walk only the
                // expression-container children.
                self.walk_children_for_expressions(node);
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
                    if let Some(init) = &data.initializer {
                        self.check_expression(init);
                    }
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
            crate::ast::NodeData::FunctionExpression(data) => data.body.clone(),
            crate::ast::NodeData::ArrowFunction(data) => data.body.clone(),
            _ => None,
        };
        if let Some(body) = body {
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => self.check_expression(&body),
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

    /// Resolve an identifier reference by walking the parent chain and
    /// checking each container's locals (and the file symbol's members at the
    /// top level).
    ///
    /// Returns the resolved symbol, or `None` if not found.
    ///
    /// Go: `Checker.resolveName` (reduced form — does not yet handle globals,
    /// namespaces, or alias chasing).
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        // Walk up the parent chain, checking locals at each container.
        let mut current = node.parent.clone();
        while let Some(parent) = current {
            // Check if this parent is a container with locals.
            if let Some(locals) = symbol_map.locals_of(&parent) {
                if let Some(sym) = locals.get(name) {
                    return Some(Arc::clone(sym));
                }
            }
            // At the source file, also check the file symbol's members
            // (top-level declarations land there).
            if parent.kind == SyntaxKind::SourceFile {
                if let Some(file_sym) = symbol_map.symbol_of(&parent) {
                    if let Some(sym) = file_sym.members.get(name) {
                        return Some(Arc::clone(sym));
                    }
                    // Also check the file's locals (block-scoped top-level
                    // declarations may be there).
                    if let Some(locals) = symbol_map.locals_of(&parent) {
                        if let Some(sym) = locals.get(name) {
                            return Some(Arc::clone(sym));
                        }
                    }
                }
                break;
            }
            current = parent.parent.clone();
        }

        // TODO: check globals (lib.d.ts symbols) once they are populated.
        None
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
