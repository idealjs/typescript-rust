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
    CheckFlags, DiagnosticsCollection, FlowFlags, FlowNode, ModifierFlags, Node, NodeData,
    NodeFlags, NodeList, NodeSymbolMap, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::core::text::TextRange;
use crate::diagnostics::messages_generated::{
    A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
    A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER,
    ARGUMENT_EXPRESSION_EXPECTED, ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1,
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
    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1, UNREACHABLE_CODE_DETECTED,
    VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED, X_0_IS_POSSIBLY_UNDEFINED,
};
use crate::evaluator::{EvalResult, EvalValue};
use crate::jsnum;

use super::tracer::Tracer;
use super::types::*;

use super::inference::{InferenceContext, InferenceInfo};

// ────────────────────────────────────────────────────────────────────────────
// Type resolution cycle detection (mirrors Go's typeResolutions)
// ────────────────────────────────────────────────────────────────────────────

/// Which property of a symbol/type/signature is being resolved.
/// Mirrors Go's `TypeSystemPropertyName` enum.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TypeResolutionProperty {
    /// `SymbolLinks.resolvedType` — the type of a value symbol.
    Type,
    /// `TypeAliasLinks.declaredType` — the declared type of a type alias.
    DeclaredType,
    /// Resolving base types of an interface.
    ResolvedBaseTypes,
    /// Resolving base constructor type.
    ResolvedBaseConstructorType,
    /// Resolving resolved return type of a signature.
    ResolvedReturnType,
    /// Resolving resolved type arguments of a type reference.
    ResolvedTypeArguments,
    /// Resolving resolved base constraint of a constrained type.
    ResolvedBaseConstraint,
}

/// A single entry on the type resolution stack. Mirrors Go's `TypeResolution`.
#[derive(Clone, Copy)]
pub struct TypeResolutionEntry {
    /// Raw pointer identity of the symbol being resolved.
    pub target: *const Symbol,
    /// Which property is being resolved.
    pub property: TypeResolutionProperty,
    /// False if a cycle was detected passing through this entry.
    pub result: bool,
}

// SAFETY: The raw pointer is only used for identity comparison within
// a single-threaded checker context.
unsafe impl Send for TypeResolutionEntry {}
unsafe impl Sync for TypeResolutionEntry {}

// ────────────────────────────────────────────────────────────────────────────
// Program trait (simplified)
// ────────────────────────────────────────────────────────────────────────────

/// A simplified version of the Go `checker.Program` interface.
///
/// The Go interface embeds `modulespecifiers.Host` and exposes ~24 methods
/// spanning module resolution, emit-format inference, and project-reference
/// redirect. This trait provides the subset currently needed by the Rust
/// checker; methods are added as the checker grows. Stubs return defaults
/// and are marked with `/// STUB:` for future wiring.
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

    // ── Host methods (embedded `modulespecifiers.Host` in Go) ──

    /// The current working directory of the host, used for path normalization
    /// during module resolution. Mirrors Go's `Host.GetCurrentDirectory()`.
    fn current_directory(&self) -> &str;

    /// Whether the host file system is case-sensitive, used for path key
    /// normalization. Mirrors Go's `Host.UseCaseSensitiveFileNames()`.
    fn use_case_sensitive_file_names(&self) -> bool;

    // ── Source directory ──

    /// The common source directory of all input files, with a trailing
    /// separator. Used for redirect root-dir normalization in composite
    /// project scenarios. Mirrors Go's `Program.CommonSourceDirectory()`.
    fn common_source_directory(&self) -> String;

    // ── Module resolution cluster (stubs — return defaults until module
    //    resolution state is wired into Program) ──

    /// STUB: Returns `None`. Go's `GetResolvedModule` maps an import specifier
    /// to a resolved module file. Needed for cross-file import type resolution.
    fn get_resolved_module(&self, _file_name: &str, _module_name: &str) -> Option<String> {
        None
    }

    /// STUB: Returns `None`. Go's `GetSourceFileForResolvedModule` fetches the
    /// parsed `SourceFile` for a resolved module path.
    fn get_source_file_for_resolved_module(&self, _resolved_path: &str) -> Option<Arc<SourceFile>> {
        None
    }

    // ── Emit format cluster (stubs) ──

    /// STUB: Returns `ModuleKind::None`. Go's `GetEmitModuleFormatOfFile`
    /// determines CJS vs ESM for a file, driving import/export elision and
    /// `VerbatimModuleSyntax` decisions.
    fn get_emit_module_format_of_file(
        &self,
        _file_name: &str,
    ) -> crate::core::compiler_options::ModuleKind {
        crate::core::compiler_options::ModuleKind::None
    }

    // ── Project reference cluster (stubs) ──

    /// STUB: Returns `false`. Go's `SourceFileMayBeEmitted` determines whether
    /// a source file will be emitted (affects module resolution extension
    /// rewriting).
    fn source_file_may_be_emitted(&self, _file_name: &str) -> bool {
        true
    }
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

/// No-op entity resolver for `evaluate_expression`. Returns `EvalResult::none()`
/// for every entity reference, which means enum member initializers that
/// reference other enum members or computed names won't resolve to a constant
/// value (they are treated as opaque/numeric in `isEnumTypeRelatedTo`).
/// This is a Phase 1 limitation; a full checker-backed entity resolver is a
/// follow-up.
fn noop_entity_fn(_: &Arc<Node>, _: Option<&Arc<Node>>) -> EvalResult {
    EvalResult::none()
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

/// The nearest non-arrow "this container" enclosing the current check point
/// (Go's `getThisContainer`). Determines which class-member suggestion the
/// checker offers when a bare identifier fails to resolve inside a class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThisContainerKind {
    /// Directly inside a `static` class member.
    StaticMember,
    /// Directly inside an instance member (method/constructor/accessor).
    InstanceMember,
    /// Inside a nested non-arrow function (the class-member chain is broken).
    PlainFunction,
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

    /// Lazily-computed cache of member names declared by global interfaces,
    /// keyed by interface name (e.g. `"Array"`). Built by scanning every
    /// loaded file's top-level `interface` declarations so it is robust to
    /// cross-file declaration-merging gaps in the binder. Used as a fallback
    /// to resolve methods like `find`/`map`/`reduce` on `T[]` array types.
    pub global_interface_members: HashMap<String, Vec<String>>,

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
    /// Stack-based cycle detection for type resolution, mirroring Go's
    /// `typeResolutions` + `pushTypeResolution`/`popTypeResolution`/
    /// `findResolutionCycle` (checker.go:18663). Each entry tracks a
    /// (symbol, property) pair being resolved. When a duplicate pair is
    /// found on the stack, all entries from the cycle start onward are
    /// marked as failed, breaking the cycle.
    pub type_resolution_stack: Vec<TypeResolutionEntry>,
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
    /// Complexity budget for type relation comparisons. Decremented on
    /// each failed sub-comparison. When it reaches zero, the relater
    /// reports overflow and stops. Mirrors Go's `relationCount`
    /// (relater.go:369). Initialized per top-level comparison call.
    pub relation_count: u32,
    /// Set to true when the relater exceeds either the depth limit
    /// (`RELATER_MAX_DEPTH`) or the complexity budget (`relation_count`).
    /// Mirrors Go's `r.overflow` (relater.go:3087).
    pub relater_overflow: bool,
    /// Per-call relation comparison cache. Stores the final boolean
    /// result of `is_type_related_to` for a `(source, target, relation)`
    /// triple so that repeated sub-comparisons within a single top-level
    /// call don't recompute. Cleared at the start of each top-level call
    /// (when `relater_depth` transitions from 0 to 1) to avoid caching
    /// optimistic cycle-broken results across calls.
    /// Mirrors Go's `Relation.results` (relater.go).
    pub relation_cache: HashMap<crate::checker::relater::RelationCacheKey, bool>,
    /// Cache for `is_enum_type_related_to`, mapping a `(source, target)`
    /// enum-symbol pair to a `RelationComparisonResult`. Mirrors Go's
    /// `Checker.enumRelation` (relater.go).
    pub enum_relation: HashMap<EnumRelationKey, crate::checker::relater::RelationComparisonResult>,
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
    /// Cache of alias symbols from globals/exports/resolved-exports symbol
    /// tables, keyed by `symbolTableID`. Mirrors Go's
    /// `symbolTableAliasCache` (checker.go).
    pub symbol_table_alias_cache: HashMap<u64, Vec<Arc<Symbol>>>,
    /// Cached symbol tables for class expression names, keyed by node ID.
    /// Mirrors Go's `classExpressionNameTables` (checker.go).
    pub class_expression_name_tables: HashMap<u64, SymbolTable>,
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

    /// Stack of enclosing class declaration nodes (the `ClassDeclaration`
    /// whose members are currently being checked). Used by the TS2341
    /// private-member check to decide whether a `private` member is accessed
    /// from within its declaring class. Empty outside any class body.
    pub enclosing_class_stack: Vec<Arc<Node>>,

    /// Nearest "this container" context (Go's `getThisContainer` with
    /// `includeArrowFunctions=false`): `StaticMember`/`InstanceMember` while
    /// directly inside a class member, `PlainFunction` inside a nested
    /// non-arrow function declaration/expression. Arrow functions don't push
    /// (they inherit the enclosing context). Used by the TS2662/TS2663
    /// fallback when a bare name fails to resolve inside a class.
    pub this_container_stack: Vec<ThisContainerKind>,

    /// Depth of enclosing ambient contexts (`declare`-modified declarations,
    /// e.g. `declare namespace N { class C { ... } }`). Mirrors Go's
    /// NodeFlagsAmbient propagation: a class inside a declared namespace is
    /// ambient even without its own `declare` modifier — TS2564/TS1005
    /// grammar checks are suppressed there.
    pub ambient_context_depth: usize,
    /// Block node ids that already reported TS1036 (statements in ambient
    /// contexts) — Go reports once per block.
    ambient_ts1036_reported_blocks: std::collections::HashSet<u64>,

    /// Recursion depth guard for `namespace_has_value_side`.
    pub namespace_value_depth: u8,

    /// Expected return type for a getter without its own annotation, taken
    /// from the paired setter's parameter annotation (the accessor pair's
    /// property type). Consumed by the getter's body check.
    pub accessor_pair_return_hint: Option<Arc<Type>>,

    /// Contextual-parameter counts for arrow/function-expression arguments
    /// currently being checked (one entry per enclosing call-argument arrow;
    /// consumed by the ArrowFunction/FunctionExpression check so their
    /// parameters typed through the callee's callback-parameter signature
    /// skip TS7006). Mirrors Go's contextual typing of call arguments.
    pub call_arg_arrow_context: Vec<usize>,

    /// Class symbols currently being resolved through
    /// `resolve_base_class_constructor_type` — guards self-referential
    /// `extends` cycles.
    pub resolving_type_aliases: std::collections::HashSet<*const Symbol>,

    /// Whether each enclosing function-like body is a CONSTRUCTOR body
    /// (entries pushed per function-like; nested functions push `false`).
    /// TS2715's "abstract property accessed in the constructor" check reads
    /// the top — accesses inside nested functions run after construction.
    pub in_ctor_body_stack: Vec<bool>,

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

    // Depth counter set while building interface call/construct signatures.
    // When non-zero, unresolved type-name references in type nodes do not
    // emit TS2304. This is needed because lib.d.ts construct/call signatures
    // declare their own signature-level type parameters (e.g.
    // `new <TArrayBuffer>(buffer: TArrayBuffer)`) that the binder does not
    // always create symbols for; without suppression, processing those
    // signatures would emit false "Cannot find name" errors. The unresolved
    // names degrade to `any`, preserving the signature (which JSX component
    // checks rely on).
    pub suppress_cannot_find_name_in_type_nodes: u32,
    /// The source file (node id) where TS2304 suppression started — the
    /// sticky counter must not silence diagnostics in OTHER files reached
    /// through cross-file resolution (a bundled lib signature resolving a
    /// user global).
    pub suppress_source_file: Option<u64>,

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

            global_interface_members: HashMap::new(),

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
            relater_depth: 0,
            relation_count: 0,
            relater_overflow: false,
            relation_cache: HashMap::new(),
            enum_relation: HashMap::new(),
            relation_in_progress: std::collections::HashSet::new(),
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
            this_container_stack: Vec::new(),
            ambient_context_depth: 0,
            ambient_ts1036_reported_blocks: std::collections::HashSet::new(),
            namespace_value_depth: 0,
            accessor_pair_return_hint: None,
            this_type_stack: Vec::new(),
            enclosing_class_stack: Vec::new(),
            call_arg_arrow_context: Vec::new(),
            resolving_type_aliases: std::collections::HashSet::new(),
            in_ctor_body_stack: Vec::new(),
            return_type_stack: Vec::new(),

            flow_analysis_disabled: false,
            flow_invocation_count: 0,
            flow_type_cache: HashMap::new(),
            flow_node_reachable: HashMap::new(),
            flow_inline_level: 0,
            in_static_member_type: false,
            suppress_cannot_find_name_in_type_nodes: 0,
            suppress_source_file: None,

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

    /// Merge `src` into `dst` (Go's `mergeSymbol`): union flags, extend
    /// declarations, and merge member/export tables recursively (same-name
    /// members merge instead of replacing). Mutates through the raw pointer
    /// — the checker initializes single-threaded.
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

    /// Populate globals from source file symbols.
    fn populate_globals(&mut self) {
        for file in &self.files {
            // Look up the source file's symbol from the symbol map
            let symbol_map = self.program.symbol_map();
            if let Some(file_sym) = symbol_map.symbol_of(&file.node) {
                // Merge the source file's members into globals — same-name
                // declarations across files MERGE (Go's mergeSymbol), so a
                // lib interface augmentation (ReadonlyArray in
                // lib.es2015.core) contributes members to the base symbol
                // instead of replacing it.
                for (name, sym) in file_sym.members.iter() {
                    match self.globals.get(name) {
                        Some(existing) => Self::merge_global_symbols(existing, sym),
                        None => {
                            self.globals.insert(name.clone(), Arc::clone(sym));
                        }
                    }
                }
                // Also merge the source file's locals
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
        // G2: Ensure common DOM/host globals are resolvable even when
        // lib.dom.d.ts isn't loaded (or its `declare var` declarations aren't
        // merged into the global scope yet). Without these, references like
        // `document`, `window`, `console`, `setTimeout` produce false TS2304
        // "Cannot find name" diagnostics.
        self.ensure_host_globals();
        // G1: Ensure a global `JSX` namespace exists when `--jsx` is enabled.
        // Without it (e.g. when `@types/react` isn't loaded), every JSX
        // element triggers TS2602/TS7026 under `noImplicitAny`.
        self.ensure_jsx_namespace();
    }

    /// G2: Insert permissive fallback globals (resolving to `any`) for common
    /// DOM value and type names, but only when no real declaration already
    /// provides them. This avoids false TS2304 "Cannot find name" diagnostics
    /// in real projects that reference `document`, `window`, `console`, DOM
    /// types, etc. without lib.dom.d.ts being fully merged into globals.
    fn ensure_host_globals(&mut self) {
        // Value-position globals (e.g. `document`, `window`, `console`).
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
            // Value constructor also referenced as a type.
            "Function",
            // CommonJS globals.
            "exports",
            "require",
            "module",
            "__dirname",
            "__filename",
            "global",
            "process",
        ];
        // Type-position globals (e.g. `HTMLElement`, `Event`).
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
        // ES2015+ built-in types referenced in type position by ambient `.d.ts`
        // files when the matching `lib.es*.d.ts` is not fully merged into
        // globals.
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
            // ES built-in constructors/types frequently referenced in value
            // and type position when lib.d.ts is not fully merged.
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
        // Built-in utility (mapped) types that ambient declarations depend on.
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
        // Use `Property` as a neutral flag for both value and type
        // fallbacks: it is found by both value and type reference
        // resolution (which query globals with an all-meaning filter), and it
        // resolves to `any` via the non-interface fallback in
        // `resolve_type_reference` (no declaration nodes, so interface member
        // resolution is never attempted). `Property` (unlike
        // `FunctionScopedVariable`) is not on the TS2749 value-as-type flag
        // list, so eager alias-body resolution of `Array<infer U>` under
        // no-lib stays silent instead of suggesting `typeof Array`.
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

    /// G1: Provide a synthetic global `JSX` namespace when JSX is enabled but
    /// no `JSX` namespace is already in scope (e.g. `@types/react` isn't
    /// loaded, or its `declare global { namespace JSX }` isn't merged into
    /// globals). Without it, every JSX element triggers TS2602 (missing
    /// `JSX.Element`) and TS7026 (missing `JSX.IntrinsicElements`) under
    /// `noImplicitAny`. The synthetic namespace is permissive: `Element`
    /// resolves to `any`, and `IntrinsicElements` carries a string index
    /// signature so any tag name is accepted.
    fn ensure_jsx_namespace(&mut self) {
        use super::jsx::JsxNames;
        if !self.is_jsx_enabled() || self.get_jsx_namespace().is_some() {
            return;
        }

        // The JSX namespace itself.
        let mut jsx = Symbol::new(SymbolFlags::NamespaceModule, JsxNames::JSX);

        // `JSX.Element` — its mere presence suppresses TS2602 in
        // `check_jsx_preconditions`. It resolves to `any`.
        let element = Symbol::new(SymbolFlags::TypeLiteral, JsxNames::ELEMENT);
        jsx.members
            .insert(JsxNames::ELEMENT.to_string(), Arc::new(element));

        // `JSX.IntrinsicElements` — model a string index signature
        // (`[elemName: string]: ...`) so any intrinsic tag is accepted and
        // TS7026 is suppressed. The index-signature marker member keeps the
        // interface's members non-empty, matching how a real index signature
        // would be represented internally.
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
// Type resolution cycle detection (mirrors Go checker.go:18663)
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Push a (target, property) pair onto the type resolution stack.
    /// Returns `true` if no cycle was detected (safe to proceed), or
    /// `false` if a cycle was found (all entries from the cycle start
    /// onward are marked as failed). Mirrors Go's `pushTypeResolution`.
    pub fn push_type_resolution(
        &mut self,
        target: *const Symbol,
        property: TypeResolutionProperty,
    ) -> bool {
        // Search the stack from top to bottom for a matching entry.
        let cycle_start = self
            .type_resolution_stack
            .iter()
            .rposition(|entry| entry.target == target && entry.property == property);

        if let Some(idx) = cycle_start {
            // Cycle found: mark all entries from cycle_start as failed.
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

    /// Pop the top entry from the resolution stack and return its result.
    /// `true` means no circularity was detected; `false` means a cycle
    /// was found. Mirrors Go's `popTypeResolution`.
    pub fn pop_type_resolution(&mut self) -> bool {
        self.type_resolution_stack
            .pop()
            .map(|entry| entry.result)
            .unwrap_or(true)
    }

    /// Check if a (target, property) pair is currently being resolved.
    /// Convenience method for simple cycle guards that don't need the
    /// full push/pop protocol.
    pub fn is_resolving(&self, target: *const Symbol, property: TypeResolutionProperty) -> bool {
        self.type_resolution_stack
            .iter()
            .any(|entry| entry.target == target && entry.property == property)
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
    pub(crate) fn build_union_from_types(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
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
        // Function-overload validation first (TS2389/2391), then the regular
        // statement walk.
        self.check_function_overloads_recursive(&statements);
        for stmt in &statements {
            self.check_statement(stmt);
        }

        // TS2309: an `export =` cannot be used in a module with other
        // exported elements (Go: checkExternalModuleExports — the
        // export-equals symbol plus any exported VALUE member is an error
        // on the `export =` declaration). Approximated syntactically: any
        // other top-level statement with an `export` modifier that declares
        // a value (class/function/variable/enum/namespace).
        self.check_export_assignment_conflicts(&statements);

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
            SyntaxKind::TrueKeyword => self.get_fresh_type_of_literal_type(&self.true_type()),
            SyntaxKind::FalseKeyword => self.get_fresh_type_of_literal_type(&self.false_type()),
            SyntaxKind::NullKeyword => self.null_type(),
            SyntaxKind::UndefinedKeyword => self.undefined_type(),
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
                    // `x as const` — narrow the expression's literal type
                    // without widening. Mirrors Go's `getConstAssertionType`.
                    if data.type_node.kind == SyntaxKind::ConstKeyword {
                        return self.get_const_assertion_type(&data.expression);
                    }
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
            // A named import (`import { f } from 'm'`) types as the
            // imported module's exported member (following an `export =`
            // alias one level).
            if symbol.flags == SymbolFlags::Alias {
                if let Some(t) = self.type_of_imported_symbol(&symbol) {
                    return t;
                }
            }
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
        // Cached merged type on `declared_type_links[symbol].declared_type`.
        // This is a VALUE-side type (call/construct signatures + namespace
        // exports) and must NOT share `type_alias_links`'s declared-type
        // slot with the class INSTANCE type that `resolve_type_reference`'s
        // Class arm builds for the same merged symbol — Go keeps them
        // distinct (`getDeclaredTypeOfSymbol` vs the class instance type).
        if let Some(cached) = self
            .declared_type_links
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
        //    its result on `type_alias_links[symbol].declared_type` — for a
        //    symbol merged with a class, resolve_type_reference's Class arm
        //    bypasses that slot, so the pollution is harmless there; the
        //    merged type below is cached separately on declared_type_links.
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
        self.declared_type_links.get_or_default(symbol).declared_type = Some(Arc::clone(&merged));
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
            // Push the overload declaration's scope before resolving its
            // parameter and return types so its own type parameters (e.g.
            // `<S, A>` in `function f<S, A>(s: S, a: A): [S, A]`) are visible
            // via the function symbol's members. Without this, the type
            // parameters of every overload except the first go out of scope
            // and produce false-positive TS2304 "Cannot find name" errors.
            self.push_scope(decl);
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
            self.pop_scope();
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
    pub(crate) fn get_type_of_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> {
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
            // No explicit constructor. A DERIVED class inherits its base's
            // constructor (`class D extends B {}` — `new D(args)` takes B's
            // parameters), so build the signature from the nearest base
            // class's constructor declaration, with the derived instance
            // type as the return. A base-less class gets a synthesized
            // no-arg construct signature (`new Foo()` valid).
            let mut inherited: Option<(Arc<Node>, Arc<Node>)> = None; // (base ctor decl, base class node)
            let mut cursor = Arc::clone(node);
            // Cycle guard: bounded by class-declaration count in the file.
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
                        /* is_construct */ true,
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
                /* is_construct */ true,
                None,
                None,
            );
            construct_sigs.push(sig);
        }
        // An abstract class's construct signatures carry
        // `SignatureFlagsAbstract` (mirrors Go's createClassType) — the flag
        // `new`-expression checking reads for TS2511, including through
        // unions of `typeof` class types.
        if node.has_syntactic_modifier(ModifierFlags::Abstract) {
            construct_sigs = construct_sigs
                .into_iter()
                .map(|sig| {
                    let mut s = crate::checker::types::Signature {
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
        let ctor_type = self.create_function_or_constructor_type(construct_sigs, /* is_construct */ true);
        // Attach the class symbol so the type displays as `typeof C`
        // (Go's TypeToString for class constructor types) — e.g. TS2348.
        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let t_mut = Arc::as_ptr(&ctor_type) as *mut crate::checker::types::Type;
            unsafe {
                (*t_mut).symbol = Some(Arc::clone(class_sym));
            }
        }
        ctor_type
    }

    /// The `extends` base `ClassDeclaration` (and class symbol) of
    /// `class_node`, if any. Resolves the base identifier through the
    /// checker's current scope (callers check classes whose scope chain is
    /// active); returns `None` for non-identifier or non-class bases.
    fn extends_base_of(&self, class_node: &Arc<Node>) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let heritage = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
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
                // For a generic signature, infer the type arguments from the
                // call arguments and substitute them into the return type
                // (so `identity(42)` yields `number`, not the type parameter
                // `T`). Mirrors Go's `getReturnTypeOfSignature` applied to
                // the instantiated signature from `chooseOverload`.
                if !sig.type_parameters.is_empty() {
                    let args: Vec<Arc<Node>> = callee.1.iter().cloned().collect();
                    let inferred = self.infer_call_type_arguments(node, sig, &args);
                    return self.substitute_infer_type_parameters(
                        &rt,
                        &sig.type_parameters,
                        &inferred,
                    );
                }
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
                    // For a generic construct signature, infer the type
                    // arguments from the call arguments and substitute them
                    // into the return type. (Class constructors don't carry
                    // their own type parameters, so this is a no-op for the
                    // common `new Foo()` case.)
                    if !sig.type_parameters.is_empty() {
                        let arg_vec: Vec<Arc<Node>> = args.iter().cloned().collect();
                        let inferred = self.infer_call_type_arguments(node, sig, &arg_vec);
                        return self.substitute_infer_type_parameters(
                            &rt,
                            &sig.type_parameters,
                            &inferred,
                        );
                    }
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
                // `+` follows Go's checkAddition: a string-like operand
                // makes the result string; any dominates otherwise
                // (any + number = any, any + string = string); number-like
                // operands give number.
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
                // Other arithmetic operators return number
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
        // A property of a NAMESPACE used as a value (`NS.f`) — resolve the
        // member through the namespace symbol's tables (exports, members,
        // and the declaration's locals for ambient namespaces) and type it
        // from its declaration.
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
                            _ => {}
                        }
                    }
                }
            }
        }
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

    /// Whether a syntax kind is an assignment operator (`=`, `+=`, `-=`, …),
    /// i.e. one that writes to its left-hand operand. Used by the TS2588
    /// const-reassignment check to distinguish assignments from equality
    /// tests (`==`) and plain binary operators.
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

    /// Whether a statement unconditionally terminates the enclosing block:
    /// `return`, `throw`, `break`, or `continue`. Used by the TS7027
    /// unreachable-code check. A statement that *contains* one of these
    /// (e.g. an `if` with a `return` body, a loop) does not terminate the
    /// block, because control may still flow past it — matching the simple
    /// heuristic used by tsc's reachability analysis.
    fn is_block_terminating_statement(stmt: &Arc<Node>) -> bool {
        matches!(
            stmt.kind,
            SyntaxKind::ReturnStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
        )
    }

    /// Whether a symbol is a class declared `abstract`. Used by the TS2511
    /// check for `new AbstractClass()`. Mirrors Go's
    /// `getDeclarationModifierFlagsFromDeclarations` abstract check in
    /// `checkNewExpression`.
    /// Whether a property's declared type includes `undefined` (exempt from
    /// TS2564).
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

    /// Whether any constructor body in the enclosing class assigns
    /// `this.<name>` — approximates Go's definite-assignment flow analysis
    /// for TS2564.
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

    /// Check one call argument, pushing the contextual-parameter count for
    /// arrow/function-expression arguments (consumed by their check arm to
    /// exempt contextually-typed parameters from TS7006).
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
            self.call_arg_arrow_context.push(ctx);
        }
        self.check_expression(arg);
        if is_function_arg {
            self.call_arg_arrow_context.pop();
        }
    }

    /// Contextual parameter count for an arrow/function-expression argument
    /// at position `arg_index` of a call to `callee_expr`: when the callee's
    /// corresponding parameter is itself a function type, its callback
    /// parameters are contextually typed (exempt from TS7006); an `any` /
    /// unresolvable parameter context leaves them implicit-any.
    /// Approximates Go's contextual typing through generic signatures (e.g.
    /// `[a].map(x => ...)`) without full type-parameter instantiation.
    fn contextual_param_count_for_arg(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
    ) -> usize {
        let t = self.get_type_of_node(callee_expr);
        if t.flags.contains(TypeFlags::Any) {
            // Lib array iteration methods fall back to `any` here (no
            // generic instantiation yet), but Go contextually types their
            // callbacks from the lib signature — exempt up to the real
            // signature's parameter count. Approximation until generic
            // instantiation lands.
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
        let Some(structured) = t.as_structured() else {
            return 0;
        };
        // Pick the first call signature whose parameter list reaches the
        // argument, else the first signature.
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

    /// Whether a callee type — possibly a union of constructor types —
    /// includes an abstract class constructor. Mirrors Go's
    /// `someSignature(constructSignatures, SignatureFlagsAbstract)` plus the
    /// abstract-class-symbol check; our class constructor types carry the
    /// class symbol, so the symbol check covers both.
    fn type_includes_abstract_constructor(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Any) {
            return false;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u.types.iter().any(|m| self.type_includes_abstract_constructor(m));
        }
        // Abstract construct signature: our signatures don't track the
        // Abstract flag, but class constructor types carry the class symbol.
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

    /// The `ClassDeclaration` that declares a given class member symbol, found
    /// by walking the member declaration's parent pointer. Returns `None` for
    /// non-class members (e.g. interface signatures, which carry no parent
    /// class). Used by the TS2341 private-member accessibility check.
    fn declaring_class_of_member(&self, member_symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
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
    }

    /// Whether the checker is currently inside the body of `class_node` (i.e.
    /// `class_node` is on the enclosing-class stack). Used by the TS2341
    /// check to allow private-member access from within the declaring class.
    fn is_within_declaring_class(&self, class_node: &Arc<Node>) -> bool {
        self.enclosing_class_stack
            .iter()
            .any(|c| Arc::ptr_eq(c, class_node))
    }

    /// Whether a function body unconditionally returns (or throws) on every
    /// path, for the TS2366 heuristic. A block returns if its last statement
    /// always returns. This is a conservative approximation (it does not model
    /// loops or short-circuiting) that mirrors the simple last-statement check
    /// described for tsc's reachability analysis.
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

    /// Whether a single statement always completes abruptly via `return` or
    /// `throw` (so control cannot fall through). Recognizes direct
    /// `return`/`throw`, a trailing `return`/`throw` in a nested block, and an
    /// `if/else` where *both* branches always return.
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
            _ => false,
        }
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
            // Fallback: look up the global `Array<T>` interface for methods
            // like find/map/reduce. Array types created by `create_array_type`
            // carry no members of their own, so resolve property existence
            // against the global `Array` interface symbol. The interface
            // symbol's own `members` table may be incomplete in this port, so
            // we resolve its declared type (which walks all declaration AST
            // nodes) for an accurate member set.
            if self.global_interface_has_property("Array", name) {
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

        // String types and their literals: resolve methods/properties via
        // the global `String` interface (lib.d.ts `interface String`),
        // using the same AST-scanning fallback as the Array fix.
        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            return self.global_interface_has_property("String", name);
        }
        // Number types and their literals: resolve via the global `Number`
        // interface.
        if t.flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            return self.global_interface_has_property("Number", name);
        }
        // Boolean types and their literals: resolve via the global `Boolean`
        // interface.
        if t.flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            return self.global_interface_has_property("Boolean", name);
        }
        // BigInt types and their literals: resolve via the global `BigInt`
        // interface.
        if t.flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            return self.global_interface_has_property("BigInt", name);
        }
        // Remaining primitive types (symbol, void, unique symbol) have no
        // resolvable properties — any access is TS2339.
        if t.flags
            .intersects(TypeFlags::ESSymbol | TypeFlags::Void | TypeFlags::UniqueESSymbol)
        {
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

    /// Resolve a named global interface symbol (e.g. `Array`) and check whether
    /// its declared type has a property or method named `prop_name`.
    ///
    /// The global interface symbol may not carry all of its cross-file
    /// declarations in this port (the binder doesn't fully merge `interface`
    /// augmentations spread across the bundled lib files), so we scan every
    /// loaded file's top-level `interface` declarations directly and cache the
    /// resulting member-name set. Used as a fallback to resolve methods like
    /// `find`/`map`/`reduce` on array types whose own members are empty.
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

    /// Scan every loaded file's top-level statements for `interface <name>`
    /// declarations and collect the names of all property/method members
    /// across all matching declarations (declaration merging). Robust to
    /// cross-file merging gaps in the binder.
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

    /// Check a `PropertyAccessExpression` (`x.prop`) and emit TS2339 when
    /// `prop` does not exist on the type of `x`.
    ///
    /// Mirrors tsc behavior: when the object type is known and structured
    /// (object literal, type reference with members, intersection, union of
    /// compatible constituents, type parameter with a constraint, etc.), a
    /// missing property is reported. `any`/`unknown`/`never` and other
    /// permissive types skip the check.
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
        // TS18048: When the object type is possibly `undefined` (or `null`)
        // and the property exists on the non-nullable part of the union,
        // report TS18048 ("'x' is possibly 'undefined'") even though the
        // property itself exists. Optional chaining (`?.`) suppresses this —
        // the `?.` already handles the undefined case. Mirrors Go's error
        // selection in `checkPropertyAccess`.
        if !question_dot
            && self.strict_null_checks
            && type_is_possibly_undefined(&obj_type)
            && self.property_exists_on_non_nullable_part(&obj_type, name_text)
        {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                X_0_IS_POSSIBLY_UNDEFINED,
                vec![obj_expr.text().to_string()],
            ));
            return;
        }
        // TS2341: a `private` class member accessed from outside its
        // declaring class. Mirrors Go's accessibility check in
        // `checkPropertyAccess` (`getSymbolModifierFlags` → `private`).
        if let Some(structured) = obj_type.as_structured() {
            if let Some(member_symbol) = structured.members.get(name_text) {
                // TS2715: an abstract property accessed via `this` inside a
                // constructor body (nested functions exempt — they run after
                // construction). Mirrors Go's `checkPropertyAccessibilityAtLocation`
                // abstract-property branch.
                if obj_expr.kind == SyntaxKind::ThisKeyword
                    && self.in_ctor_body_stack.last() == Some(&true)
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
                if let Some(declaring_class) = self.declaring_class_of_member(member_symbol) {
                    let is_private = member_symbol
                        .declarations
                        .iter()
                        .any(|d| d.has_syntactic_modifier(ModifierFlags::Private));
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
        if self.has_property_of_type(&obj_type, name_text) {
            return;
        }
        // Fallback for static methods on global constructor values (e.g.
        // `Object.values`/`Object.entries`). These are declared on the
        // `ObjectConstructor` interface, whose augmentations are spread across
        // multiple lib files (lib.es5 + lib.es2017) and not fully merged by
        // the binder. Resolve the accessed value to its global symbol and scan
        // the corresponding constructor interface, mirroring the Array fix.
        if self.global_constructor_value_has_property(obj_expr, name_text) {
            return;
        }
        // A NAMESPACE property access (`NS.f`) resolves through the
        // namespace's EXPORTS only — non-exported members live in the
        // namespace's locals and are not visible from outside (Go's
        // resolveEntityName consults `exports`; TS2339 otherwise). The
        // property's TYPE is recovered by `get_type_of_property_access`.
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
        let type_str = self.type_to_string(&obj_type);
        // TS2551: suggest an existing member within one edit (Go funnels
        // this through getSpellingSuggestion over the type's properties).
        let suggestion = obj_type.as_structured().and_then(|st| {
            let members: Vec<&String> = st.members.entries.keys().collect();
            members
                .into_iter()
                .filter(|cand| cand.len() >= 2 && cand.as_str() != name_text)
                .map(|cand| {
                    (
                        edit_distance(&name_text.to_ascii_lowercase(), &cand.to_ascii_lowercase()),
                        cand,
                    )
                })
                .filter(|(d, _)| *d <= 1)
                .min_by_key(|(d, _)| *d)
                .map(|(_, c)| c.as_str().to_string())
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

    /// Check whether `obj_expr` references a global constructor value (such as
    /// `Object`) whose corresponding interface (e.g. `ObjectConstructor`)
    /// declares a property named `name`. Used as a fallback for static methods
    /// whose declarations span multiple lib files and aren't fully merged by
    /// the binder.
    fn global_constructor_value_has_property(&mut self, obj_expr: &Arc<Node>, name: &str) -> bool {
        if obj_expr.kind != SyntaxKind::Identifier {
            return false;
        }
        // Resolve the accessed identifier. Only proceed when it resolves to the
        // actual global symbol (a local shadow must not trigger the fallback).
        let resolved = match self.resolve_identifier(obj_expr) {
            Some(sym) => sym,
            None => return false,
        };
        let interface_name = match resolved.name.as_str() {
            "Object" => {
                // Confirm this is the global `Object` value, not a local
                // variable that happens to be named `Object`.
                match self.globals.get("Object") {
                    Some(global_sym) if Arc::ptr_eq(&resolved, global_sym) => "ObjectConstructor",
                    _ => return false,
                }
            }
            _ => return false,
        };
        self.global_interface_has_property(interface_name, name)
    }

    /// Whether `name` exists as a property on the non-nullable constituents of
    /// a union type. Used to distinguish TS18048 (possibly undefined) from
    /// TS2339 (property doesn't exist). Mirrors Go's check that looks up the
    /// property on the non-undefined part of the union.
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
        // Non-union: check the type itself (excluding the nullable flags).
        self.has_property_of_type(t, name)
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
    ///
    /// For generic signatures, type arguments are first inferred from the
    /// call arguments and substituted into the parameter types before
    /// checking assignability (so `identity(42)` infers `T = number` and
    /// checks `number` against `number`, not against the bare type
    /// parameter `T`). Simplified port of the inference branch of Go's
    /// `chooseOverload`.
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
        // `super(args)` — a super call invokes the base class's constructor.
        // The `super` keyword itself types as the enclosing class's instance
        // type (not callable), so resolve the base class's constructor type
        // here and check the arguments against its construct signatures.
        // Mirrors Go's `checkCallExpression` super-call handling. When the
        // base class can't be resolved, skip rather than reporting a false
        // TS2349 against the constructor.
        if !is_new && callee_expr.kind == SyntaxKind::SuperKeyword {
            let Some(base_ctor_type) = self.resolve_base_class_constructor_type() else {
                return;
            };
            self.check_call_arguments_against(
                node,
                &base_ctor_type,
                &arguments,
                callee_expr,
                /*is_new*/ true,
            );
            return;
        }
        let callee_type = self.get_type_of_node(callee_expr);
        self.check_call_arguments_against(node, &callee_type, &arguments, callee_expr, is_new);
    }

    /// Argument-checking core shared by direct calls/constructs and `super()`
    /// calls: select a signature from `callee_type`, check arity and each
    /// argument's assignability. `callee_expr` is the diagnostic anchor.
    fn check_call_arguments_against(
        &mut self,
        node: &Arc<Node>,
        callee_type: &Arc<Type>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) {
        // `any` callee → skip (no false positives without a signature).
        if callee_type.flags.contains(TypeFlags::Any) {
            return;
        }
        // Union callee (e.g. `typeof A | typeof B`): constructable when every
        // member carries construct signatures; the signatures flatten into
        // one overload set (Go: `getSignaturesOfType` over union members).
        // NOTE: union types also expose `as_structured` (member tables), so
        // the union check must come FIRST for `new`.
        let mut union_signatures: Vec<Arc<Signature>> = Vec::new();
        let signatures: &[Arc<Signature>] =
            if is_new && callee_type.as_union_or_intersection().is_some() {
                // Flatten (possibly nested) union members; every leaf must carry
                // construct signatures for the union to be constructable.
                let mut leaves: Vec<&Arc<Type>> = Vec::new();
                flatten_union_leaves(callee_type, &mut leaves);
                let all_constructable = !leaves.is_empty()
                    && leaves.iter().all(|m| {
                        m.as_structured()
                            .is_some_and(|s| !s.construct_signatures().is_empty())
                    });
                if all_constructable {
                    for m in &leaves {
                        if let Some(s) = m.as_structured() {
                            union_signatures.extend(s.construct_signatures().iter().cloned());
                        }
                    }
                    &union_signatures
                } else {
                    // Some member isn't constructable.
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
            } else if let Some(structured) = callee_type.as_structured() {
                if is_new {
                    structured.construct_signatures()
                } else {
                    structured.call_signatures()
                }
            } else {
                // Non-structured callee (primitive like `number`, `string`,
                // etc.) is never callable/constructable. Mirrors Go's
                // `invocationError` for types with no signatures.
                if !is_new && self.report_get_accessor_call(callee_expr) {
                    return;
                }
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
            };
        if signatures.is_empty() {
            if !is_new {
                // TS2348: a type with construct signatures but no call
                // signatures called without `new` (Go's
                // Value_of_type_0_is_not_callable_Did_you_mean_to_include_new).
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
                // TS 1.0 Spec 4.11 (Go resolveNewExpression): an object type
                // with NO construct signatures but call signatures is
                // processed as a function call — the arguments are checked
                // against the call signatures, and (only when noImplicitAny
                // is off) a non-void return type reports TS2350. Legacy
                // constructor functions (`function P(x) { this.x = x; }`,
                // inferred return void) construct silently with result any.
                if let Some(structured) = callee_type.as_structured() {
                    let call_sigs: &[Arc<Signature>] = structured.call_signatures();
                    if !call_sigs.is_empty() {
                        if !self.no_implicit_any {
                            let matching = self.find_matching_signature(call_sigs, &arguments);
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
                            /*is_new*/ false,
                        );
                        return;
                    }
                }
            }
            // Structured type but no call/construct signatures — e.g.
            // calling a plain object literal or a number. Mirrors Go's
            // `invocationError` head message ("This expression is not
            // callable" / "This expression is not constructable").
            if !is_new && self.report_get_accessor_call(callee_expr) {
                return;
            }
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
        // For a generic signature, infer type arguments from the call
        // arguments and substitute them into each parameter type before the
        // assignability check. Mirrors Go's `getSignatureInstantiation` +
        // `isSignatureApplicable` flow inside `chooseOverload`.
        // TS2558: explicit type-argument count must match the signature's
        // type-parameter count (Go's getArityError for type arguments).
        if !sig.type_parameters.is_empty() || Self::has_explicit_type_arguments(node) {
            let provided = Self::explicit_type_argument_count(node);
            if provided != 0 && provided != sig.type_parameters.len() {
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
                    vec![
                        sig.type_parameters.len().to_string(),
                        provided.to_string(),
                    ],
                ));
            }
        }
        let inferred_types = self.infer_call_type_arguments(node, &sig, &arguments.nodes);
        for (i, arg) in arguments.iter().enumerate() {
            // Determine the parameter type to check against.
            let base_param_type = if has_rest && i >= rest_index {
                // Rest position: check against the array element type.
                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {
                self.get_type_of_symbol(&sig.parameters[i])
            } else {
                // Beyond declared params with no rest — should have been
                // caught by the arity check; skip to avoid false positives.
                continue;
            };
            // Substitute inferred type arguments into the parameter type.
            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &base_param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else {
                base_param_type
            };
            // `any` parameter → always assignable, skip. When the
            // signature is generic but inference produced NO candidates,
            // the parameter type stays an unsubstituted type parameter —
            // Go falls back to the constraint/`unknown` there, which
            // accepts any argument; don't mis-report TS2345.
            let inference_empty =
                !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }
            // Contextual element checks for literal arguments
            // (`f([1, "a"])` with `param: number[]` — per-element TS2322,
            // excess TS2353 on nested object literals).
            if matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) {
                let pt = Arc::clone(&param_type);
                self.check_contextual_elements(arg, &pt, arg.loc);
            }
            let arg_type = self.get_type_of_node(arg);
            if !self.is_type_assignable_to(&arg_type, &param_type) {
                // Widen literal source types for display, like the other
                // relation errors (Go's reportRelationError).
                let display_type = if crate::checker::is_literal_type(&arg_type)
                    && !crate::checker::is_literal_type(&param_type)
                {
                    self.get_base_type_of_literal_type(&arg_type)
                } else {
                    arg_type.clone()
                };
                let arg_str = self.type_to_string(&display_type);
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

    /// Run call-site type-argument inference for `signature` given the
    /// call's argument nodes. Returns one inferred `Type` per type
    /// parameter of `signature`; returns an empty vec when the signature is
    /// non-generic.
    ///
    /// Simplified port of the inference branch of Go's `chooseOverload`:
    /// builds an `InferenceContext` from the signature's type parameters,
    /// invokes `infer_type_arguments`, and returns the resolved inferred
    /// types. Does not yet handle explicit type-argument lists on the call
    /// (`f<T>(...)`), context-sensitive two-pass inference, or the
    /// `CheckModeSkipGenericFunctions` deferred-function heuristic.
    fn infer_call_type_arguments(
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
        // No signature accepted the arguments (Go reports the arity error
        // against the FIRST ARITY-COMPATIBLE overload, not signatures[0] —
        // `Promise.resolve(1)` with overloads [(), (value)] must blame the
        // (value) overload's arity, or match it when only assignability
        // failed).
        let arg_count = arguments.len();
        for (idx, sig) in signatures.iter().enumerate() {
            let max_params = if sig.has_rest_parameter() {
                usize::MAX
            } else {
                sig.parameters.len()
            };
            if arg_count <= max_params && arg_count >= sig.min_argument_count.max(0) as usize {
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

    /// Get the type of a `const` assertion expression (`x as const`).
    ///
    /// For `as const`, TypeScript narrows all literal types to their literal
    /// forms (no widening) and makes object properties readonly and arrays
    /// readonly tuples. Mirrors Go's `getConstAssertionType`.
    ///
    ///   - `[1, 2, 3] as const` → `readonly [1, 2, 3]` (tuple, not widened)
    ///   - `{ a: 1 } as const` → `{ readonly a: 1 }`
    ///   - `"hello" as const` → `"hello"` (already narrow)
    ///
    /// The checker already returns narrow literal types for literals and
    /// keeps literal types in object literals, so the main special case is
    /// array literals (which `get_type_of_array_literal` normally widens to
    /// `T[]`).
    fn get_const_assertion_type(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        match expr.kind {
            SyntaxKind::ArrayLiteralExpression => {
                // Build a tuple with narrow (non-widened) element types.
                let elements = match &expr.data {
                    crate::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
                    _ => return self.get_any_type(),
                };
                let mut element_types: Vec<Arc<Type>> = Vec::new();
                for elem in elements.iter() {
                    if elem.kind == SyntaxKind::SpreadElement {
                        // Spread in `as const` → fall back to array type.
                        let t = self.get_type_of_node(elem);
                        element_types.push(t);
                    } else {
                        element_types.push(self.get_type_of_node(elem));
                    }
                }
                self.create_tuple_type(element_types)
            }
            _ => {
                // Object literals already keep literal types; literals are
                // already narrow. Just return the expression's type.
                self.get_type_of_node(expr)
            }
        }
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

    /// For an object-literal `source` being assigned to `target`, return the
    /// name of the first source property that does not exist on the target
    /// (an "excess" property). Returns `None` when the source isn't an object
    /// literal, or when the target has an index signature (which permits
    /// arbitrary properties). Mirrors the fresh-object-literal branch of Go's
    /// `hasExcessProperties`.
    fn get_excess_property_name(&self, source: &Arc<Type>, target: &Arc<Type>) -> Option<String> {
        // Only object-literal sources undergo excess property checking.
        if !crate::checker::is_object_literal_type(source) {
            return None;
        }
        let source_struct = source.as_structured()?;
        let target_struct = target.as_structured()?;
        // An index signature on the target permits any property name.
        if !target_struct.index_infos.is_empty() {
            return None;
        }
        for prop in &source_struct.properties {
            // `target_has_property` descends into union/intersection
            // constituents: a property is excess only if it exists on NONE
            // of the constituents. (Union and intersection structured
            // `members` tables are not pre-merged in this port.)
            if !self.target_has_property(target, &prop.name) {
                return Some(prop.name.clone());
            }
        }
        None
    }

    /// Read-only check whether a property named `name` exists on `t`,
    /// descending into union and intersection constituents. Unlike
    /// `has_property_of_type`, this does not fall back to the global
    /// `Array<T>` interface (not needed for excess-property checking, which
    /// only runs against object-literal sources assigned to object-like
    /// targets).
    fn target_has_property(&self, t: &Arc<Type>, name: &str) -> bool {
        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }
            // An index signature on any constituent permits the property.
            if !structured.index_infos.is_empty() {
                return true;
            }
        }
        // Union: property exists if present on any constituent.
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }
        // Intersection: property exists if present on any constituent.
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

    /// Return the names of `target` properties that are required (non-optional)
    /// but absent from `source`. Mirrors Go's `getUnmatchedProperties` for the
    /// missing-required-property case.
    fn get_missing_required_properties(
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

    /// Locate the name node of an object-literal property by name, so excess
    /// property errors (TS2353) can be reported at the offending property.
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

    /// Extract the property name string from a name node (identifier,
    /// string literal, numeric literal). Returns an empty string for
    /// computed property names (caller should skip those).
    fn get_property_name_from_node(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(id) => id.text.clone(),
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => n.text.clone(),
            NodeData::ComputedPropertyName(_) => {
                // Use source text including brackets, matching TS behavior.
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
        // TS1036: non-declaration statements are not allowed in ambient
        // contexts (Go's checkGrammarStatementInAmbientContext — reported
        // on the first token, once per directly-enclosing block).
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
                // TS1108: `return` outside a function body (top level or
                // namespace level) — Go's checkGrammarStatementInAmbientContext
                // adjacent rule in checkReturnStatement.
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
                                // Go's checkReturnExpression anchors on the
                                // RETURN STATEMENT (not the expression) for
                                // plain return statements, and widens literal
                                // source types for display.
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
                                    node.loc,
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                    vec![actual_str, expected_str],
                                ));
                            }
                        }
                    } else {
                        // `return;` with no value — if the function declares
                        // a non-void/non-undefined return type, Go reports
                        // TS2322 "Type 'undefined' is not assignable to type
                        // '<expected>'" on the return statement (the same
                        // checkReturnExpression path as valued returns).
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
                    // TS7027: code following an unconditional `return`,
                    // `throw`, `break`, or `continue` is unreachable. Track a
                    // flag while walking the block's statements; once a
                    // terminating statement is seen, every subsequent
                    // statement is reported and (per tsc) still checked.
                    let mut after_terminator = false;
                    for stmt in data.statements.iter() {
                        if after_terminator {
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
                    // case_block is a CaseBlock node; walk its clauses. Push
                    // the case block's scope first so case-clause
                    // declarations (`case 1: let x;`) resolve — the binder
                    // scopes them to the CaseBlock (all clauses share one
                    // scope, mirroring Go's block-scoped CaseBlock).
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
            // Declarations: walk only expression-position children.
            // Without parent pointers, we cannot detect declaration names
            // via `is_declaration_name`, so we must handle each kind
            // explicitly, skipping names and type-position children.
            SyntaxKind::FunctionDeclaration => {
                // Grammar check: validate modifiers and parameter list.
                self.check_grammar_modifiers(node);
                // TS2393: two or more same-name function declarations in the
                // same container have bodies. Go's
                // `checkFunctionOrConstructorSymbol` reports on EVERY
                // function declaration of the name (overload signatures
                // included); reported once, when visiting the first
                // declaration in source order.
                self.check_duplicate_function_implementations(node);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(tps) = &data.type_parameters {
                        let _ = tps; // TODO: check_grammar_type_parameter_list
                    }
                    self.check_grammar_parameter_list(&data.parameters);
                    // TS2369: parameter properties are never allowed in a
                    // function declaration (only constructor implementations).
                    self.check_parameter_property_modifiers(&data.parameters, false);
                    // TS7006/TS7019: implicit-any parameters; plus parameter
                    // type annotations and the return-type annotation may
                    // contain function-type nodes with their own parameters.
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
                    // TS7010: a signature declaration without a body or
                    // return-type annotation implicitly returns `any` under
                    // noImplicitAny. Fires per declaration — even when an
                    // implementation exists elsewhere (oracle-verified).
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
                // A nested function declaration breaks the class-member
                // "this container" chain (Go's getThisContainer treats plain
                // functions as this containers — TS2663 no longer applies).
                self.this_container_stack
                    .push(ThisContainerKind::PlainFunction);
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        self.check_statement(body);
                    }
                }
                self.this_container_stack.pop();
                // TS2355 vs TS2366 (Go `checkFunctionAndBodies`): with a
                // declared return type that isn't `undefined`/`void`/`any`:
                //  - no `return` anywhere in the body → TS2355 on the
                //    return-type annotation (the body never returns a value);
                //  - some `return` but not all paths return → TS2366.
                if let Some(ret_type) = &declared_return {
                    if !ret_type.flags.contains(TypeFlags::Void)
                        && !ret_type.flags.contains(TypeFlags::Undefined)
                        && !ret_type.flags.contains(TypeFlags::Any)
                    {
                        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                            if let Some(body) = &data.body {
                                if !self.function_body_definitely_returns(body) {
                                    if !Self::function_body_has_explicit_return(body) {
                                        // TS2355 — on the annotation, like Go.
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
                                        // Go anchors TS2366 on the return
                                        // type annotation (`fn.Type()`).
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
                // Grammar check: validate modifiers.
                self.check_grammar_modifiers(node);
                // Reserved type names (TS2414): `class any {}` etc.
                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(name) = &data.name {
                        self.check_reserved_type_name(
                            name,
                            &crate::diagnostics::messages_generated::CLASS_NAME_CANNOT_BE_0,
                        );
                    }
                }
                // Push the class scope before building the instance type so
                // that type-parameter references in property annotations
                // (e.g. `value: T`) resolve correctly.
                self.push_scope(node);
                // Build the instance type (including inherited members from
                // `extends`) and push it as the `this` type so that method
                // bodies can resolve `this.prop` and `super.prop`.
                let this_type = self.build_class_instance_type_with_base(node);
                self.this_type_stack.push(this_type);
                // Track the enclosing class declaration so the TS2341
                // private-member check knows when access is within the
                // declaring class.
                self.enclosing_class_stack.push(Arc::clone(node));
                // Check heritage clauses (e.g. `extends Foo`, `implements I`).
                if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
                    if let Some(heritage) = &data.heritage_clauses {
                        for clause in heritage.iter() {
                            self.check_heritage_clause(clause);
                        }
                    }
                    // Overload-consecutiveness check (TS2389/2390/2391).
                    // Ambient (`declare class` OR inside a `declare
                    // module`/namespace) members are exempt.
                    if !node.has_syntactic_modifier(ModifierFlags::Ambient)
                        && self.ambient_context_depth == 0
                        && !self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| f.is_declaration_file)
                    {
                        self.check_class_member_overloads(&data.members);
                    }
                    // Check member initializers / method bodies.
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                    // TS2564: Check property initialization under
                    // strictPropertyInitialization.
                    self.check_property_initialization(node);
                    self.check_class_heritage_members(node);
                }
                self.pop_scope();
                self.this_type_stack.pop();
                self.enclosing_class_stack.pop();
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
            SyntaxKind::InterfaceDeclaration => {
                // TS2427: reserved predefined type keywords can't name an
                // interface (`interface string {}`).
                if let crate::ast::NodeData::InterfaceDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::INTERFACE_NAME_CANNOT_BE_0,
                    );
                    self.check_interface_members(&data.members);
                }
                // Force interface-type resolution so `extends` relation
                // errors (TS2430) and base merging run even when the
                // interface is never referenced (Go resolves declared
                // interface types during checking).
                let iface_sym = self.program.symbol_map().symbol_of(node).cloned();
                if let Some(sym) = iface_sym {
                    let _ = self.resolve_interface_type(&sym, None);
                }
            }
            SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier => {
                // No expression-position children to check — all type-level
                // or import-level. Type-alias RHS still gets grammar checks
                // (TS1183 for accessors with bodies in a type literal).
                if node.kind == SyntaxKind::TypeAliasDeclaration
                    && let crate::ast::NodeData::TypeAliasDeclaration(d) = &node.data
                {
                    self.check_type_annotation(&d.type_node);
                    // Resolve the alias body eagerly (Go checks alias
                    // declarations when encountered — TS7039 and friends
                    // don't wait for a usage). Bundled lib files stay lazy:
                    // the binder has no symbols for signature-scoped type
                    // parameters (e.g. `<TFunction extends Function>` in
                    // lib.decorators.legacy), so eager resolution would
                    // report false TS2304s there.
                    if !self.current_file.as_ref().is_some_and(|f| {
                        f.file_name.starts_with("bundled://")
                    }) {
                        let _ = self.get_type_from_type_node(&d.type_node);
                    }
                }
                // TS2439: an import inside an ambient module declaration
                // can't use a relative module name (both `import ... from`
                // and `import x = require(...)` forms).
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
                            // Go's module resolution never resolves a
                            // relative specifier inside an ambient module —
                            // TS2307 lands on the module specifier itself
                            // and the import degrades to error (member
                            // accesses stay silent).
                            let spec_loc = match &node.data {
                                crate::ast::NodeData::ImportDeclaration(d) => {
                                    d.module_specifier.loc
                                }
                                crate::ast::NodeData::ImportEqualsDeclaration(d) => {
                                    // The string literal itself, not the
                                    // `require` keyword (Go anchors TS2307
                                    // on the specifier).
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
                // `import X = <entity>` — eagerly resolve the entity so a
                // bad namespace path reports at the import (TS2503/TS2694),
                // like Go's checkImportEqualsDeclaration.
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
                    // A non-namespace leftmost segment (`import r =
                    // undefined;`) is TS2503 like an unresolved one.
                    let base_is_namespace = match &d.module_reference.data {
                        crate::ast::NodeData::Identifier(_) => {
                            self.resolve_identifier(&d.module_reference).is_none_or(|s| {
                                let b = self.resolve_alias_base(s);
                                b.flags.contains(SymbolFlags::ValueModule)
                            })
                        }
                        _ => true,
                    };
                    let traced_err = if entity_ok && !base_is_namespace {
                        Some((Arc::clone(&d.module_reference), String::new(), String::new()))
                    } else if entity_ok {
                        match self.resolve_qualified_symbol_traced(&d.module_reference) {
                            Err(e) => Some(e),
                            Ok(_) => None,
                        }
                    } else {
                        None
                    };
                    if let Some((segment, ns_path, member)) = traced_err
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        if ns_path.is_empty() {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                crate::diagnostics::messages_generated::CANNOT_FIND_NAMESPACE_0,
                                vec![segment.text().to_string()],
                            ));
                        } else {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                crate::diagnostics::messages_generated::
                                    NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                vec![ns_path, member],
                            ));
                        }
                    }
                }
            }
            SyntaxKind::EnumDeclaration => {
                // Reserved type names (TS2431): `enum any {}` etc.
                if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::ENUM_NAME_CANNOT_BE_0,
                    );
                }
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
                // TS2436: ambient module declarations can't use RELATIVE
                // module names (`declare module "./foo"`).
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
                    // Inside an ambient module, an import/export through a
                    // relative name is TS2439 (checked at the import sites
                    // instead).
                    if relative && ambient {
                        // Only the DECLARATION itself is TS2436 when it's
                        // top-level-ish; imports inside are TS2439 — handled
                        // in the import walker.
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
                // TS2664: a module AUGMENTATION (`declare module "x"` inside
                // an external-module file) whose target cannot be found.
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
                // Check the module body. A `declare namespace/module` makes
                // everything inside ambient (Go's NodeFlagsAmbient
                // propagation).
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

    /// Check the computed property names inside a variable declaration's
    /// binding pattern (`let {[a]: a} = …`). Those names are ordinary
    /// expression positions — resolving them enables the used-before-
    /// declaration check (TS2448) when a pattern references the symbol it
    /// declares. Nested patterns (`{a: {b}}`) are walked recursively;
    /// non-computed property names are declaration-side and skipped.
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
                            if let crate::ast::NodeData::ComputedPropertyName(cd) = &pn.data {
                                self.check_expression(&cd.expression);
                                // TS2538: a computed property name whose
                                // expression is `any` is not a valid index.
                                // Mirrors tsc's `checkComputedPropertyName`
                                // (`maybeTypeOfKind(type, TypeFlags.Any)`),
                                // which fires for binding-pattern computed
                                // names like `let {[a]: a} = …` where the
                                // name resolves to the (implicit-any) symbol
                                // it declares.
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
                // Ambient initializers (Go's checkGrammarVariableDeclaration):
                // - `var`/`let` with an initializer → TS1039.
                // - `const` (no type annotation) requires a simple literal or
                //   literal enum reference → else the const-initializer
                //   message.
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
                        // Literal enum reference: `Enum.member` and
                        // `Enum["member"]` / Enum[`member`].
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
                // TS1100/TS1215: `var arguments` / `var eval` in strict code
                // (alwaysStrict, modules, or a "use strict" prologue) —
                // Go's binder checkStrictModeEvalOrArguments.
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

    /// Whether the current context is strict: `alwaysStrict`, an external
    /// module, or the file starts with a "use strict" prologue.
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

    /// TS2715 for `let { x, y: y1 } = this;` in a constructor: each bound
    /// element's property name (or shorthand name) that resolves to an
    /// abstract property of the enclosing class errors on that name node.
    /// Mirrors Go's `isThisInitializedObjectBindingExpression` branch.
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
            // The property read is the property name (`{y: y1}` reads `y`);
            // shorthand (`{x}`) reads its own name.
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
            // TS1155: a `const` declaration must be initialized (Go's
            // `checkVariableDeclaration`). Skipped for for-in/for-of
            // declarations (`for (const k in o)`) and ambient declarations.
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
                        // Ambient ancestor (e.g. `declare namespace M { const
                        // x }` — the declare is on the namespace).
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
            // TS2481 (Go `checkVarDeclaredNamesNotShadowed`): a `var` with
            // initializer whose name lexically resolves to a DIFFERENT
            // block-scoped symbol (the block-scoped binding binds tighter,
            // so the initializer would write to it). Only fires when the
            // block-scoped declaration lives in a plain nested block —
            // when its container is the function body / module / source
            // file scope, hoisting makes the names share a scope and the
            // binder's duplicate check covers it instead.
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
            // A binding-pattern name's computed property names are reference
            // positions: `let {[a]: a} = …` reads `a` before its own
            // declaration (TS2448 via the regular identifier check). Plain
            // property names (`{a: b}`) are not references. Mirrors Go's
            // `checkComputedPropertyName` being invoked for binding patterns
            // through the declaration check.
            self.check_binding_pattern_computed_names(&data.name);
            // TS2715 (destructuring form): `let { x, y } = this;` inside a
            // constructor accesses each bound abstract property.
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
                    // Nested literal elements (`var v: {id:number}[] =
                    // [{id:1}, {id:2, name:"x"}]`): the direct
                    // object-literal checks below cover non-array
                    // initializers; array literals need per-element
                    // contextual checks (TS2353/TS2741/TS2322).
                    if init.kind == SyntaxKind::ArrayLiteralExpression {
                        let at = Arc::clone(&annotation_type);
                        self.check_contextual_elements(init, &at, init.loc);
                    }
                    let init_type = self.get_type_of_node(init);
                    let assignable = self.is_type_assignable_to(&init_type, &annotation_type);
                    let mut reported_error = false;

                    // Excess property check for object-literal initializers.
                    // An object literal with properties not present on the
                    // target type is an error even when all required target
                    // properties are present. Mirrors Go's `hasExcessProperties`
                    // performed inside `isRelatedToEx` for fresh literals.
                    if let Some(excess_name) =
                        self.get_excess_property_name(&init_type, &annotation_type)
                    {
                        let file = self.current_file.clone();
                        let annot_str = self.type_to_string(&annotation_type);
                        // Report at the offending property name when locatable.
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
                        let file = self.current_file.clone();
                        // Generalize literal types for error display (mirrors Go's
                        // reportRelationError: when source is a literal type and
                        // target can't have singleton types, widen to base type).
                        // Widen a literal source for display only when the
                        // target can't have singleton types — Go's
                        // reportRelationError keeps both literals
                        // ('"no_dunder"' vs '"__dunder"').
                        let display_type = if crate::checker::is_literal_type(&init_type)
                            && !crate::checker::is_literal_type(&annotation_type)
                        {
                            self.get_base_type_of_literal_type(&init_type)
                        } else {
                            init_type.clone()
                        };
                        let init_str = self.type_to_string(&display_type);
                        let annot_str = self.type_to_string(&annotation_type);
                        // Report at the variable declaration (name), not the
                        // initializer — mirrors Go's checkVariableLikeDeclaration.
                        let missing =
                            self.get_missing_required_properties(&init_type, &annotation_type);
                        let message = if missing.len() == 1 {
                            crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                                vec![missing[0].clone(), init_str, annot_str],
                            )
                        } else if missing.len() > 1 {
                            crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                                vec![init_str, annot_str, missing.join(", ")],
                            )
                        } else {
                            crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![init_str, annot_str],
                            )
                        };
                        self.diagnostics.add(message);
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
        // TS2420.
        let data = match &node.data {
            crate::ast::NodeData::HeritageClause(d) => d,
            _ => return,
        };
        if data.token == SyntaxKind::ExtendsKeyword {
            // TS1174: `extends A, B` — classes extend a single class
            // (Go's checkGrammarClassDeclarationHeritageClause; reported on
            // the second-and-later type reference's first token).
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
            // For `extends` clauses, resolve the base class expression as a
            // type reference (suppressing false TS2304 for global names like
            // `Object` that are resolvable in type position). The base-class
            // instance type is built separately in `build_class_instance_type_with_base`.
            // Here we just try to resolve the expression to suppress TS2304.
            for type_ref in data.types.iter() {
                if let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data {
                    // TS2689: `extends` naming an INTERFACE-only symbol
                    // (Go's checkClassExtends → Cannot_extend_an_interface).
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
                    // Try to resolve the expression as a type reference.
                    // This populates type_node_links without emitting TS2304.
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
    pub(crate) fn build_class_instance_type_with_base(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (members, heritage_clauses) = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.build_interface_type_from_members(&Arc::new(NodeList::default())),
        };
        // Build the derived class's own instance type.
        let own_type = self.build_interface_type_from_members(members);
        // Attach the class symbol so the type printer uses the declared
        // name (Go's `Type.Symbol`). The type was just created and is not
        // yet shared.
        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let own_mut = Arc::as_ptr(&own_type) as *mut crate::checker::types::Type;
            unsafe {
                (*own_mut).symbol = Some(Arc::clone(class_sym));
            }
        }
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

    /// Resolve the base class's constructor (static-side) type for a
    /// `super(...)` call in the currently enclosing class. Returns `None`
    /// when there is no enclosing class, no `extends` clause, or the base
    /// cannot be resolved to a class declaration.
    fn resolve_base_class_constructor_type(&mut self) -> Option<Arc<Type>> {
        let (base_node, symbol) = self.base_class_node_of_enclosing_class()?;
        // Guard against self-referential `extends`.
        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return None;
        }
        let ctor_type = self.get_type_of_class_declaration(&base_node);
        self.resolving_type_aliases.remove(&key);
        Some(ctor_type)
    }

    /// The `ClassDeclaration` node (and class symbol) of the enclosing
    /// class's `extends` base, if resolvable.
    fn base_class_node_of_enclosing_class(&self) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let class_node = self.enclosing_class_stack.last().cloned()?;
        self.extends_base_of(&class_node)
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
                            if !self.push_type_resolution(
                                key,
                                TypeResolutionProperty::ResolvedBaseTypes,
                            ) {
                                return self.get_any_type();
                            }
                            let instance = self.build_class_instance_type_with_base(&class_node);
                            self.pop_type_resolution();
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

    /// TS2564: Check property initialization. Under strictNullChecks and
    /// strictPropertyInitialization, reports an error for non-static instance
    /// properties that have no initializer, no definite-assignment assertion
    /// (`!`), and are not definitely assigned in the constructor. Mirrors
    /// Go's `checkPropertyInitialization`.
    fn check_property_initialization(&mut self, class_node: &Arc<Node>) {
        if !self.strict_null_checks || !self.strict_property_initialization {
            return;
        }
        // Skip ambient classes (`declare class`, or inside a `declare
        // namespace` / .d.ts — Go's NodeFlagsAmbient propagation).
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
        // Find the constructor (if any) for definite-assignment checking.
        let constructor = members.iter().find(|m| m.kind == SyntaxKind::Constructor);
        for member in members.iter() {
            if member.kind != SyntaxKind::PropertyDeclaration {
                continue;
            }
            // Skip ambient and static members.
            let mods = self.get_combined_modifier_flags(member);
            if mods.contains(ModifierFlags::Ambient) || mods.contains(ModifierFlags::Static) {
                continue;
            }
            // Skip abstract properties.
            if mods.contains(ModifierFlags::Abstract) {
                continue;
            }
            let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data else {
                continue;
            };
            // Skip if it has an initializer or a definite-assignment assertion.
            if pd.initializer.is_some() || pd.postfix_token.is_some() {
                continue;
            }
            // Skip if there's no name or the name isn't an identifier/private/computed.
            let name_node = &pd.name;
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier
                    | SyntaxKind::PrivateIdentifier
                    | SyntaxKind::ComputedPropertyName
            ) {
                continue;
            }
            // Get the property's type from the type annotation. If no type
            // annotation, the property type is `any` — skip (matches Go's
            // `TypeFlagsAnyOrUnknown` guard).
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
            // Check if the property is assigned in the constructor.
            if let Some(ctor) = constructor {
                if self.is_property_assigned_in_constructor(name_node, ctor) {
                    continue;
                }
            }
            // Report TS2564.
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

    /// Get the text representation of a property name node.
    fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            crate::ast::NodeData::Identifier(d) => d.text.clone(),
            crate::ast::NodeData::PrivateIdentifier(d) => d.text.clone(),
            crate::ast::NodeData::ComputedPropertyName(_) => {
                // Reported with the source text including the brackets,
                // e.g. `[Symbol.toPrimitive]` or `["a"]`, matching TS.
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

    /// Resolve the symbol for a property declaration's name node.
    fn resolve_property_name(
        &mut self,
        member: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        // Try to resolve via the name node directly.
        self.resolve_identifier(name)
    }

    /// Check whether a property is definitely assigned in the constructor body.
    /// Simplified: looks for `this.propName =` or `this[propName] =` patterns.
    fn is_property_assigned_in_constructor(&self, name_node: &Arc<Node>, ctor: &Arc<Node>) -> bool {
        let name_text = match &name_node.data {
            crate::ast::NodeData::Identifier(d) => d.text.as_str(),
            _ => return false,
        };
        // Walk the constructor body looking for `this.name =` assignments.
        let body = match &ctor.data {
            crate::ast::NodeData::ConstructorDeclaration(d) => &d.body,
            _ => return false,
        };
        let Some(body) = body else {
            return false;
        };
        Self::node_contains_this_assignment(body, name_text)
    }

    /// Recursively check if a node tree contains `this.<name> =` or
    /// `this[<name>] =`.
    fn node_contains_this_assignment(node: &Arc<Node>, name: &str) -> bool {
        // Check for BinaryExpression with `=` operator where the left side is
        // `this.name` or `this[name]`.
        if let crate::ast::NodeData::BinaryExpression(data) = &node.data {
            if data.operator_token.kind == SyntaxKind::EqualsToken {
                if Self::is_this_property_access(&data.left, name) {
                    return true;
                }
            }
        }
        // Recursively check child nodes.
        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if Self::node_contains_this_assignment(child, name) {
                found = true;
                return true; // stop iteration
            }
            false
        });
        found
    }

    /// Check if a node is `this.name` or `this["name"]` or `this['name']`.
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

    /// The `name` node of a class member that has one (`None` for
    /// constructors — their "name" is the `constructor` keyword and errors
    /// anchor on the member node itself, mirroring `name ?? node` in Go).
    fn class_member_name_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.name)),
            _ => None,
        }
    }

    /// Declaration-name text of a class member, rendered like Go's
    /// `DeclarationNameToString`: identifiers verbatim, string literals WITH
    /// their quotes, numeric literals verbatim. Constructors share the fixed
    /// key `"constructor"`. `None` for non-literal names (computed/private).
    fn class_member_name_text(node: &Arc<Node>) -> Option<String> {
        if matches!(node.kind, SyntaxKind::Constructor) {
            return Some("constructor".to_string());
        }
        let name = Self::class_member_name_node(node)?;
        match name.kind {
            // Go's `DeclarationNameToString`: string literals print WITH
            // their quotes; identifiers and numeric literals print verbatim.
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

    /// Whether a method/constructor member has an implementation body.
    fn class_member_has_body(node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::MethodDeclaration(d) if d.body.is_some()
        ) || matches!(
            &node.data,
            crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some()
        )
    }

    /// Class-body overload validation, mirroring Go's
    /// `checkFunctionOrConstructorSymbolWorker`: a run of overload signatures
    /// must be immediately followed by its implementation. Reports
    /// - TS2390 when a constructor overload group has no implementation,
    /// - TS2391 when a method overload group has no implementation (or its
    ///   signatures are not consecutive),
    /// - TS2389 when a differently-named implementation directly follows an
    ///   overload signature.
    ///
    /// Ambient (`declare`) class members are exempt — ambient declarations
    /// can be interleaved. Abstract / optional members don't need bodies.
    fn check_class_member_overloads(&mut self, members: &NodeList) {
        // Group function-like members by name text, preserving order.
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
            // Walk the group like Go's declaration loop: a signature not
            // immediately followed (list-adjacency approximates Go's
            // `previousDeclaration.End() == node.Pos()` trivia-adjacency)
            // reports on the PREVIOUS signature; at the end, a trailing
            // signature with no implementation reports on itself.
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
            }
        }
    }

    /// The `reportImplementationExpectedError` core: `node` is an overload
    /// signature with no implementation. When the member immediately after it
    /// is a same-kind implementation with a DIFFERENT name, that
    /// implementation is really this overload's implementation gone wrong →
    /// TS2389; otherwise the group is simply missing its implementation →
    /// TS2390 (constructor) / TS2391 (method).
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
                // Same name: the overload has its implementation right after
                // (static/instance mismatch is a separate Go check, not
                // ported here).
                if same_name {
                    return;
                }
                if Self::class_member_has_body(sib) {
                    // TS2389 on the implementation's name node, naming the
                    // overload's declaration name.
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
        // Missing implementation: constructor → TS2390 on the member; method
        // → TS2391 on its name (or the member when the name is missing).
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

    /// TS2369: parameter properties (`constructor(private x)`) are only
    /// allowed in a constructor IMPLEMENTATION (one with a body). Mirrors
    /// Go's `checkParameter` modifier check.
    fn check_parameter_property_modifiers(&mut self, params: &NodeList, is_ctor_impl: bool) {
        for param in params.iter() {
            let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            // Grammar-check modifiers on every parameter regardless of
            // context — Go's `checkParameter` (checker.go:2661) calls
            // `checkGrammarModifiers`, reporting TS1090 (`static`/`export`
            // modifier cannot appear on a parameter), TS1028 (accessibility
            // modifier already seen), etc.
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

    /// TS7006/TS7019: parameters without a type annotation or initializer
    /// implicitly have an `any` (rest: `any[]`) type under `noImplicitAny`.
    /// Reported on the parameter node for every function-like context —
    /// implementations, overload signatures, interface members and ambient
    /// `declare`s alike (oracle-verified: `declare function g(y);` in a
    /// source file still reports). Mirrors Go's `reportImplicitAny`.
    ///
    /// Exemptions (parameters that are NOT implicitly any):
    /// - `contextual_param_count`: parameters of contextually-typed function
    ///   expressions / arrows (`let f: (x: number) => number = (x) => ...`)
    ///   inherit their type from the contextual signature.
    /// - parameters typed via a JSDoc `@param {T}` tag.
    /// KNOWN GAP: binding-pattern parameters don't report per-element
    /// implicit-any errors yet.
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
            // Contextually typed: the corresponding contextual-signature
            // parameter supplies the type.
            if i < contextual_param_count {
                continue;
            }
            // JSDoc `@param {T} name` provides the type.
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

    /// Whether the function `node` has a JSDoc `@param {T} <name>` tag with a
    /// type expression for the given parameter name. Resolves the node's
    /// JSDoc comments through the source file's lazy JSDoc cache.
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

    /// Walk a type annotation, checking nested function/constructor TYPE
    /// nodes: their parameters are function-likes for TS2369 (parameter
    /// properties) and TS7006 (implicit any), and nested annotations are
    /// recursed. This is how `(public B) => C` annotations get their
    /// parameter-level diagnostics (Go checks type nodes via
    /// `checkSourceElement` → `checkFunctionTypeNode`).
    fn check_type_annotation(&mut self, tn: &Arc<Node>) {
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
                }
            }
            SyntaxKind::TypeLiteral => {
                // TS1183: accessors with bodies inside a type literal are
                // implementations in a type context (Go's
                // `checkGrammarAccessor` Parent==KindTypeLiteral branch).
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

    /// Whether a function body contains any `return` statement outside
    /// nested function-likes — Go's binder `NodeFlagsHasExplicitReturn`.
    fn function_body_has_explicit_return(body: &Arc<Node>) -> bool {
        fn walk(n: &Arc<Node>) -> bool {
            match n.kind {
                SyntaxKind::ReturnStatement => return true,
                // Nested function-likes track their own return flags.
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

    /// TS6234: `x.prop()` where `prop` is a `get` accessor — the callee's
    /// type (the getter's return type) has no call signatures, but the
    /// intended fix is dropping the `()`, so the message says so. Mirrors
    /// Go's `resolveErrorCall` → get-accessor special case. Returns true when
    /// the diagnostic was emitted.
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
            // On the property NAME (Go anchors at the accessor name, e.g.
            // `x.property()` reports on `property`).
            pa.name.loc,
            crate::diagnostics::messages_generated::
                THIS_EXPRESSION_IS_NOT_CALLABLE_BECAUSE_IT_IS_A_GET_ACCESSOR_DID_YOU_MEAN_TO_USE_IT_WITHOUT,
            vec![],
        ));
        true
    }

    /// Whether a same-named symbol with TYPE meaning (interface/class/
    /// enum/alias/type-param) is visible from the current scope — lib.d.ts
    /// merges `interface X` + `declare var X` as one logical entity (though
    /// our binder may keep separate symbols), and merged names are legal
    /// type references.
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


    /// Whether a namespace/module symbol has a VALUE side: any exported or
    /// local member with value meaning, following an `export =` target one
    /// level (Go's export-assignment alias resolution).
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
                    // Skip mis-routed parameter/signature symbols (a
                    // method's parameters can land in the container's
                    // members table; they don't make the namespace a value).
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
        // A nested namespace with its own value side makes this one a
        // value too (`namespace Outer { export namespace Inner { export
        // class C {} } }` — Outer is a value). Depth-limited recursion.
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
        // Ambient namespaces implicitly export their members — the binder
        // routes un-exported `declare namespace` members to the module
        // node's LOCALS, so scan those too.
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
        // `export = <entity>`: the namespace is value-ful iff the target is.
        if let Some(export_equals) = namespace.exports.get("export=") {
            for decl in &export_equals.declarations {
                if let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {
                    // The export= expression lives inside the namespace's
                    // body — resolve it in that scope.
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


    /// The type of a named-import alias: resolve the import's module,
    /// look the imported name up in its exports/members/locals, following
    /// an `export = <entity>` target one level.
    fn type_of_imported_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportSpecifier)?;
        let name = match &decl.data {
            // `import { default as Foo }` imports the PROPERTY name.
            crate::ast::NodeData::ImportSpecifier(d) => d
                .property_name
                .as_ref()
                .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
            _ => return None,
        };
        // Walk up through NamedImports/ImportClause to the ImportDeclaration.
        let mut import_decl = decl.parent.as_ref()?;
        while !matches!(import_decl.data, crate::ast::NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            crate::ast::NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_sym = self.resolve_module_file_symbol(&module_spec)?;
        let Some(member) = self.namespace_member_recursive(&module_sym, &name) else {
            // allowSyntheticDefaultImports: a missing `default` falls back
            // to the module namespace (any-typed here).
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
        None
    }

    /// Look up `name` in a namespace/module's exports, members, and the
    /// declaration's locals (ambient namespaces), following an `export =`
    /// target one level.
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
        // `export = <entity>`: resolve the target and look there.
        let export_equals = namespace.exports.get("export=")?;
        for d in &export_equals.declarations {
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
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
                let target = scope_decl.and_then(|scope_decl| {
                    self.push_scope(&scope_decl);
                    let t = self.resolve_qualified_symbol(&ea.expression);
                    self.pop_scope();
                    t
                });
                if let Some(mut target) = target {
                    // `import X = NS` entity-name aliases (an `export =`
                    // target commonly is one) follow to the namespace.
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
        None
    }


    /// The dotted full path of a namespace symbol — from its
    /// (nested) ModuleDeclaration chain: `foo.bar.baz`.
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

    /// TS2352: `expr as T` where neither type sufficiently overlaps the
    /// other. Mirrors Go's `checkAssertionDeferred`: literal source types
    /// are widened first, and the types must be comparable (approximated
    /// here as assignable in either direction — Go's
    /// `isTypeComparableTo`); `as const` and error/any/unknown/never types
    /// are exempt.
    fn check_assertion_overlap(&mut self, node: &Arc<Node>, expr: &Arc<Node>, type_node: &Arc<Node>) {
        // `x as const` is a const assertion, not a cast — exempt.
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
        // Go's checkAssertion uses `isTypeComparableTo` in both directions
        // (comparability widens primitive literals: `{n: 1}` overlaps IFoo
        // because number ~ 1); plain assignability over-reports casts of
        // partial object literals.
        let comparable = self.is_type_comparable_to(&expr_base, &target_type)
            || self.is_type_comparable_to(&target_type, &expr_base);
        if !comparable {
            let source_str = self.type_to_string(&expr_base);
            let target_str = self.type_to_string(&target_type);
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST,
                vec![source_str, target_str],
            ));
        }
    }

    /// TS1183: an accessor (or method) with a body inside an interface or a
    /// type literal is an implementation in an ambient/type context. Mirrors
    /// Go's `checkGrammarAccessor` body-present branch. Reported on the body
    /// node, like Go.
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

    /// Interface-member checks: signature parameters (TS2369 parameter
    /// properties, TS7006 implicit any), implicit-any returns (TS7010 for
    /// methods, TS7013 construct / TS7010-analog call signatures — Go's
    /// `reportImplicitAny` switch), and type annotations that hold
    /// function-type nodes. Mirrors Go's `checkInterfaceDeclaration` →
    /// `checkSourceElement` walk over members.
    fn check_interface_members(&mut self, members: &NodeList) {
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
                    // TS7010: no return-type annotation on a method signature.
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
                    // TS7013 (construct) / TS7010-analog (call): signature
                    // without a return-type annotation implicitly returns any.
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
                // TS1183: accessors with bodies are implementations, which an
                // interface (ambient context) cannot contain.
                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                    self.check_accessor_in_type_context(member);
                }
                _ => {}
            }
        }
    }

    /// TS2813/TS2814: a NON-ambient class declaration cannot share a symbol
    /// with function declarations — the class "cannot implement overload
    /// list" (TS2813, on each class name) and functions with bodies can only
    /// merge with ambient classes (TS2814, on each function name). Ambient
    /// classes (`declare class X` + `function X` declarations) merge cleanly.
    /// Go: checkFunctionOrConstructorSymbol's `hasNonAmbientClass && Flags&
    /// Function != 0` arm — reported on every declaration of the symbol.
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

    /// `check_statement_function_overloads` over a statement list AND the
    /// nested statement lists of blocks and namespace bodies — overloads
    /// declared inside `{ ... }` blocks or `namespace N { ... }` follow the
    /// same implementation-must-follow rule (Go's check is symbol-based and
    /// covers declarations wherever they appear). Skipped entirely in
    /// declaration files, where overload groups need no implementation
    /// (Go's `inAmbientContext` covers everything in a .d.ts).
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
                    // Ambient modules (`declare module "..."` / `declare
                    // namespace N`): everything inside is ambient — overload
                    // groups need no implementation.
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

    /// Top-level function overload validation — the statement-list sibling of
    /// `check_class_member_overloads` (Go funnels both through
    /// `checkFunctionOrConstructorSymbolWorker`): same-named overload
    /// signatures must be immediately followed by their implementation.
    /// Ambient (`declare function`) signatures are exempt from the
    /// implementation requirement.
    fn check_statement_function_overloads(&mut self, statements: &[Arc<Node>]) {
        // Ambient function declarations (`declare function f();`, or any
        // function inside a `declare namespace` / .d.ts) are exempt from
        // the implementation-must-follow rule.
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
                // TS2394: an overload signature must be satisfiable by the
                // implementation signature (Go's
                // isImplementationCompatibleWithOverload — simplified to
                // the arity rule: the implementation's required-parameter
                // count must not exceed the overload's parameter count,
                // unless the implementation takes a rest parameter).
                let fn_params = |f: &Arc<Node>| -> (usize, bool) {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data {
                        let mut required = 0;
                        let mut rest = false;
                        for p in d.parameters.iter() {
                            if p.kind == SyntaxKind::Parameter {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    if pd.dot_dot_dot_token.is_some() {
                                        rest = true;
                                        break;
                                    }
                                    if pd.question_token.is_none() {
                                        required += 1;
                                    }
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
                    // recompute required-only count
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
                    // Duplicate overload signatures collapse (Go's
                    // getSignaturesOfSymbol dedupes identical consecutive
                    // signatures — `function f(x: any); function f(x: any);`
                    // is ONE signature for diagnostics).
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
                        if overload_count < impl_required
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

    /// `reportImplementationExpectedError` for statement-level function
    /// declarations: TS2389 when a differently-named implementation follows
    /// the signature immediately, TS2391 otherwise.
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

    /// TS2309 helper: `export =` conflicts with any other exported VALUE
    /// element in the module (statement-level approximation of Go's
    /// `hasExportedMembersOfKind(SymbolFlagsValue)`).
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

    /// TS 1.0 spec 3.6.1: predefined type keywords are reserved and cannot
    /// name user-defined types. Class-likes report TS2414, enums TS2431.
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

    fn check_class_member(&mut self, node: &Arc<Node>) {
        // Grammar check on the member's modifiers (TS1248 `const` member,
        // duplicate modifiers, etc.) — Go's checkPropertyDeclaration /
        // checkMethodDeclaration call checkGrammarModifiers per member.
        self.check_grammar_modifiers(node);
        // TS2392: multiple constructor implementations in one class.
        if node.kind == SyntaxKind::Constructor {
            self.check_multiple_constructor_implementations(node);
        }
        // TS2300: same-name class members that can't merge (two properties,
        // two methods of identical shape, property+method). Getter/setter
        // pairs and method overloads are exempt (Go's
        // checkClassMemberDuplicates through resolveDeclaredMembers).
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
                if !my_name.is_empty() {
                    let dup = cd.members.iter().any(|m| {
                        if Arc::ptr_eq(m, node) || m.loc.pos() >= node.loc.pos() {
                            return false;
                        }
                        match &m.data {
                            crate::ast::NodeData::PropertyDeclaration(d) => {
                                d.name.text() == my_name
                            }
                            // Method pairs: only BOTH-bodied duplicates conflict
                            // (overload + implementation is legal).
                            crate::ast::NodeData::MethodDeclaration(d) => {
                                d.name.text() == my_name
                                    && d.body.is_some()
                                    && matches!(&node.data, crate::ast::NodeData::MethodDeclaration(cur) if cur.body.is_some())
                            }
                            _ => false,
                        }
                    });
                    if dup {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            my_loc,
                            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                            vec![my_name],
                        ));
                    }
                }
            }
        }
        match node.kind {
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                // Only check the body; the name, parameters, and return type
                // are declarations/types.
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
                // Ambient context (declare class / declare namespace / .d.ts):
                // a body here is an implementation in an ambient context —
                // TS1183 on the body's first token (Go's
                // checkGrammarStatementInAmbientContext; the ambient flag
                // propagates from any ambient ancestor, e.g. a declared
                // namespace).
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
                // Accessor grammar (Go `checkGrammarAccessor`): a body-less
                // accessor in a non-ambient class needs `abstract` (TS1005
                // "'{' expected" at the trailing `;`); an abstract accessor
                // cannot have a body (TS1310); setter parameters reject
                // rest/optional/initializer forms (TS1053/TS1090/TS1052).
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
                        // Locate the trailing `;` (Go reports at
                        // accessor.End()-1; our spans may swallow trailing
                        // CRLF, so trim whitespace back first).
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
                    // TS1183 for accessor bodies in ambient contexts is
                    // handled above (shared with methods/constructors).
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
                    // TS2676: a get/set pair must be uniformly abstract or
                    // non-abstract (reported once, from the getter, on both
                    // names — Go's checkAccessor pair check).
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
                            // The pair's property type: getter annotation,
                            // else the setter's parameter annotation. A
                            // getter without its own annotation checks its
                            // return statements against the setter's param
                            // type (Go's getTypeOfAccessors ordering). NB:
                            // DIFFERENTLY annotated get/set types are legal —
                            // no both-annotated conflict error.
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
                        // Setter-side pair typing: an unannotated setter
                        // parameter carries the paired getter's annotated
                        // return type, so `param = expr` assignments in the
                        // setter body are checked against it (TS2322 at the
                        // parameter reference — Go types the parameter symbol
                        // from the pair).
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
                }
                // TS2369: parameter properties are only allowed in a
                // constructor IMPLEMENTATION (one with a body).
                if let Some(params) = &parameters {
                    let is_ctor_impl =
                        matches!(node.kind, SyntaxKind::Constructor) && body.is_some();
                    self.check_parameter_property_modifiers(params, is_ctor_impl);
                    // TS7006/TS7019: implicit-any parameters; parameter and
                    // return type annotations may hold function-type nodes
                    // with their own parameter checks (`(public A) => any`).
                    // Accessor parameters are EXEMPT here: Go suppresses the
                    // setter-param TS7006 when the paired getter carries a
                    // type (and reports TS7032 instead) — that pairing rule
                    // is not yet ported, so accessors stay unchecked rather
                    // than over-reporting.
                    if matches!(node.kind, SyntaxKind::MethodDeclaration | SyntaxKind::Constructor)
                    {
                        self.check_parameter_implicit_any(node, params, 0);
                    }
                    for p in params.iter() {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                            && let Some(pt) = &pd.type_node
                        {
                            self.check_type_annotation(pt);
                            // Accessor parameters don't go through
                            // `get_type_of_function_like` (methods get their
                            // annotations resolved during instance-type
                            // building), so resolve them explicitly — Go's
                            // checkParameter resolves every annotation,
                            // reporting TS2304 for unresolved names.
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
                // TS7010: a method signature without a body or return-type
                // annotation implicitly returns `any` under noImplicitAny
                // (constructors are exempt — Go's reportImplicitAny switch
                // omits KindConstructor).
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
                    // TS17009: in a DERIVED class's constructor, `this` is
                    // not accessible before the `super()` call (including
                    // `super(this.x)` arguments — Go's definite
                    // super-call analysis, approximated with an in-order
                    // body scan).
                    if node.kind == SyntaxKind::Constructor
                        && self
                            .enclosing_class_stack
                            .last()
                            .is_some_and(|c| self.extends_base_of(c).is_some())
                    {
                        self.check_super_before_this(&body);
                    }
                    // TS2662/TS2663 context: while directly inside this
                    // member's body, a bare name failing to resolve checks
                    // the class's static/instance members for a suggestion
                    // (Go's `checkAndReportErrorForMissingPrefix`).
                    let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                    self.this_container_stack.push(if is_static {
                        ThisContainerKind::StaticMember
                    } else {
                        ThisContainerKind::InstanceMember
                    });
                    self.push_function_scope(node);
                    // TS2715 context: constructor bodies count; other
                    // function-likes (and anything nested in them) do not.
                    self.in_ctor_body_stack
                        .push(node.kind == SyntaxKind::Constructor);
                    // Push the declared return type so `return expr;`
                    // statements in the body can be checked against it.
                    // `None` means no explicit return-type annotation.
                    // A getter without an annotation inherits the paired
                    // setter's parameter annotation (the pair's property
                    // type).
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
                    // TS2355 (methods): declared non-`undefined`/`void`/`any`
                    // return type + no `return` anywhere in the body →
                    // "must return a value", on the annotation (Go
                    // `checkFunctionAndBodies`; TS2366-all-paths checking is
                    // currently function-declaration-only).
                    if let Some(ret_type) = &declared_return
                        && !ret_type.flags.contains(TypeFlags::Void)
                        && !ret_type.flags.contains(TypeFlags::Undefined)
                        && !ret_type.flags.contains(TypeFlags::Any)
                        && body.kind == SyntaxKind::Block
                        && !self.function_body_definitely_returns(&body)
                        && !Self::function_body_has_explicit_return(&body)
                    {
                        let loc = type_node
                            .as_ref()
                            .map_or(node.loc, |tn| tn.loc);
                        if matches!(node.kind, SyntaxKind::MethodDeclaration) {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                loc,
                                A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                vec![],
                            ));
                        } else if node.kind == SyntaxKind::GetAccessor {
                            // Getter with an annotation whose body never
                            // returns: Go reports TS2322 "Type 'undefined'
                            // is not assignable to type '<annotation>'" on
                            // the annotation (checkFunctionAndBodies'
                            // accessor branch).
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
            SyntaxKind::PropertyDeclaration => {
                // Only check the initializer; the name and type are
                // declarations/types.
                // (TS2564 is handled class-level by `check_property_initialization`
                // — the upstream implementation with its assignment-scan helpers.)
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
                        // TS2662/TS2663 context for property initializers
                        // (Go's `checkAndReportErrorForMissingPrefix` also
                        // fires here — `a = inst` suggests `this.inst`).
                        let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                        self.this_container_stack.push(if is_static {
                            ThisContainerKind::StaticMember
                        } else {
                            ThisContainerKind::InstanceMember
                        });
                        self.check_expression(init);
                        self.this_container_stack.pop();
                        // Contextual element checks for annotated property
                        // initializers (`public bar:{id:number} =
                        // {id:5, name:"foo"}` — TS2353/TS2741/TS2322).
                        if let Some(tn) = &data.type_node {
                            let target = self.get_type_from_type_node(tn);
                            let anchor = data.name.loc;
                            self.check_contextual_elements(init, &target, anchor);
                        }
                    }
                }
            }
            SyntaxKind::PropertySignature => {
                // All type-level — no expressions to check.
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                if let crate::ast::NodeData::ClassStaticBlockDeclaration(data) = &node.data {
                    // Static blocks are static `this` contexts (TS2662
                    // suggestions apply; instance members aren't suggested).
                    self.this_container_stack
                        .push(ThisContainerKind::StaticMember);
                    self.check_statement(&data.body);
                    self.this_container_stack.pop();
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
                // TS1066: ambient-enum member initializers must be constant
                // expressions (Go's computeConstantValue ambient branch).
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

    /// Whether an enum-member initializer is a compile-time constant
    /// expression: literals, member references, unary ±, and arithmetic /
    /// bitwise combinations thereof.
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

    // ────────────────────────────────────────────────────────────────────
    // Enum member value computation (ported from Go's checker.go:23820-23901)
    // ────────────────────────────────────────────────────────────────────

    /// Get the first declaration of `kind` for a symbol.
    /// Mirrors Go's `ast.GetDeclarationOfKind`.
    pub fn get_declaration_of_kind(
        &self,
        symbol: &Arc<Symbol>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> {
        symbol.declarations.iter().find(|d| d.kind == kind).cloned()
    }

    /// Get the constant value of an enum member node.
    /// Mirrors Go's `Checker.getEnumMemberValue` (checker.go:23820).
    pub fn get_enum_member_value(&mut self, node: &Arc<Node>) -> EvalResult {
        if let Some(parent) = node.parent.as_ref() {
            self.compute_enum_member_values(parent);
        }
        self.enum_member_links
            .get(node)
            .map(|l| l.value.clone())
            .unwrap_or_else(EvalResult::none)
    }

    /// Compute and cache the values of all members of an `EnumDeclaration`.
    /// Idempotent (guarded by `NodeCheckFlags::EnumValuesComputed`).
    /// Mirrors Go's `Checker.computeEnumMemberValues` (checker.go:23846).
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

    /// Compute the value of a single enum member.
    /// Mirrors Go's `Checker.computeEnumMemberValue` (checker.go:23866).
    /// Phase 1: omits the TS1062 "enum member must have initializer" error
    /// and the isolated-modules checks, returning `EvalResult::none()` for
    /// members without a computable value.
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

    /// Evaluate the initializer of a constant enum member.
    /// Mirrors Go's `Checker.computeConstantEnumMemberValue` (checker.go:23903).
    /// Phase 1: skips NaN/Infinity (const enum) and string-syntax
    /// (isolatedModules) error checks.
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

    /// Check an expression node: resolve identifier references and recurse
    /// into sub-expressions.
    ///
    /// Number of explicit type arguments on a Call/New expression (0 when
    /// none or not a call-like node).
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

    /// Whether a Call/New expression carries explicit type arguments.
    fn has_explicit_type_arguments(node: &Arc<Node>) -> bool {
        Self::explicit_type_argument_count(node) > 0
    }

    /// Display string for arithmetic/bitwise operator tokens (the parsed
    /// operator node carries no text).
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

    /// Non-plus arithmetic/bitwise operand checks (Go's
    /// checkBinaryLikeExpression): TS18050 for null/undefined literals,
    /// TS2447 for boolean operand pairs (suggesting the logical operator),
    /// TS2362/TS2363 when an operand isn't assignable to number|bigint.
    /// Runs on declared (pre-flow) types, before the operand expressions'
    /// identifier checks.
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
        let lt = self.get_type_of_node(&data.left);
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
        // Null/undefined literals already reported TS18050 — Go's
        // checkNonNullType short-circuits their operand type check.
        let left_is_literal = matches!(data.left.kind, NullKeyword | UndefinedKeyword);
        let right_is_literal = matches!(data.right.kind, NullKeyword | UndefinedKeyword);
        if !left_is_literal && !ok_number(self, &lt) {
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

    /// `+` operator error (TS2365): neither string-like, number-like,
    /// bigint-like, nor any operands — Go's checkAddition resultType==nil
    /// path. Runs after the operand expressions are checked.
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
        let lt = self.get_type_of_node(&data.left);
        let rt = self.get_type_of_node(&data.right);
        let number_like = |t: &Arc<Type>| {
            t.flags.intersects(
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
                    // Arithmetic/bitwise operand checks on DECLARED types run
                    // before flow analysis (official baseline emission order:
                    // TS2362/TS2363 precede TS2454 at the same position).
                    self.check_binary_arith_pre(node, data);
                    self.check_expression(&data.left);
                    self.check_expression(&data.right);
                    self.check_binary_plus_operator_error(node, data);
                    use crate::ast::SyntaxKind::*;
                    // NOTE: Go emits TS2872/2873 ("always truthy"/"always
                    // falsy") only from `checkTruthinessOfType` at
                    // truthiness-test positions (if/while/ternary/`!`) via
                    // SYNTACTIC predicate semantics — never for the operand
                    // TYPES of `&&`/`||`/`??` (a null-typed identifier like
                    // `let x = null; x ?? 5` is Go-legal). The syntactic
                    // predicate-semantics family (TS2869/2871/2872/2873 over
                    // ??/&&/|| chains, predicateSemantics.ts) is ported with
                    // its test batch.
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
                    // TS2629: assigning to a class/enum/namespace-only
                    // name (Go's checkDeprecatedSymbol/assignment checks:
                    // `f += ''` where `f` is a class reports per operator).
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
                            }
                        }
                    }
                    // TS2588: Assigning to a `const` variable after its
                    // declaration. Mirrors Go's `checkAssignmentStatement`
                    // const-target check. Fires for every assignment operator
                    // (`=`, `+=`, …) whose left-hand side is an identifier
                    // resolving to a `const` binding.
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
                    // Contextual element checks for `x = <literal>` with an
                    // annotated LHS: excess/missing properties on object
                    // literals, per-element checks for array literals
                    // (TS2353/TS2741/TS2322).
                    if data.operator_token.kind == EqualsToken
                        && data.left.kind == SyntaxKind::Identifier
                    {
                        if let Some(target) = self.declared_annotation_type_of(&data.left) {
                            self.check_contextual_elements(
                                &data.right,
                                &target,
                                data.right.loc,
                            );
                        }
                    }
                    // TS2540: `M.x <op>= v` where `M` is a namespace and `x`
                    // a `const` member (all assignment operators, mirroring
                    // Go's checkReferenceExpression const check). Element
                    // accesses with string-literal arguments (`M["x"] = 0`)
                    // behave the same.
                    if Self::is_assignment_operator(data.operator_token.kind)
                        && matches!(
                            data.left.kind,
                            SyntaxKind::PropertyAccessExpression
                                | SyntaxKind::ElementAccessExpression
                        )
                    {
                        self.check_const_property_assignment(&data.left);
                    }
                    // TS2322: Assignment type check is deferred until type
                    // resolution is more precise (currently causes false
                    // positives due to imprecise left-side type inference).
                    // TODO: Implement precise assignment type checking.
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
                    // `++x`/`--x` assign to the operand: a `const` target
                    // reports TS2588 (Go checks ++/-- through the
                    // assignment pipeline's `checkReferenceExpression`).
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
                // Grammar-check members (TS1248 `const` member, duplicate
                // modifiers, …) — class expressions don't go through the
                // statement-level class checker.
                if let crate::ast::NodeData::ClassExpression(data) = &node.data {
                    for member in data.members.iter() {
                        self.check_grammar_modifiers(member);
                    }
                }
            }
            SyntaxKind::CallExpression => {
                if let crate::ast::NodeData::CallExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    for (i, arg) in data.arguments.iter().enumerate() {
                        self.check_call_arg_with_context(&data.expression, i, arg);
                    }
                }
                self.check_call_arguments(node, /* is_new */ false);
            }
            SyntaxKind::NewExpression => {
                if let crate::ast::NodeData::NewExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    if let Some(args) = &data.arguments {
                        for (i, arg) in args.iter().enumerate() {
                            self.check_call_arg_with_context(&data.expression, i, arg);
                        }
                    }
                    // TS2511: `new AbstractClass()` where the resolved class
                    // declaration carries the `abstract` modifier. Mirrors
                    // Go's `checkNewExpression` abstract-class guard —
                    // reported on the NewExpression node (`new`), not the
                    // callee.
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
                    // TS2511 via the callee's TYPE — covers non-identifier
                    // callees and unions like `typeof A | typeof B` where ANY
                    // member (Go: `someSignature` abstract / abstract class
                    // symbol) is an abstract constructor.
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
                // TS2369: parameter properties are never allowed in arrow
                // functions / function expressions.
                // TS7006: unannotated parameters are exempt when the function
                // is contextually typed — its signature supplies the types
                // (Go: contextual parameters aren't implicit-any). Sources:
                // an annotation on the assignee (`let f: F = (x) => ...`) or
                // a function-typed parameter of an enclosing call argument
                // (`.map(x => ...)`); the latter is consumed from
                // `call_arg_arrow_context` here.
                let mut contextual_param_count = self
                    .call_arg_arrow_context
                    .last_mut()
                    .map(|v| std::mem::replace(v, 0))
                    .unwrap_or(0);
                if contextual_param_count == 0 {
                    contextual_param_count = self
                        .get_contextual_type(node, ContextFlags::None)
                        .as_ref()
                        .and_then(|t| t.as_structured())
                        .and_then(|s| s.call_signatures().first())
                        .map_or(0, |sig| sig.parameters.len());
                }
                match &node.data {
                    crate::ast::NodeData::ArrowFunction(d) => {
                        self.check_parameter_property_modifiers(&d.parameters, false);
                        self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);
                    }
                    crate::ast::NodeData::FunctionExpression(d) => {
                        self.check_parameter_property_modifiers(&d.parameters, false);
                        self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);
                    }
                    _ => {}
                }
                // A function expression is its own "this container" (breaks
                // the class-member chain for TS2663); arrow functions are
                // NOT (Go's getThisContainer skips arrows).
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
                    self.check_assertion_overlap(
                        node,
                        &data.expression,
                        &data.type_node,
                    );
                    // TS1355 (Go `checkAssertion`): `x as const` requires a
                    // literal or an enum-member reference on the left.
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
                // `<T>x`: the left side is a type; the right is an expression.
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
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                // Object-literal methods/accessors have their own function
                // scope like class members: locals (e.g. a hoisted `var
                // _this = this`) live on the accessor node itself, so the
                // scope must be pushed before walking the body.
                self.check_class_member(node);
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
        // Nested function-likes run after construction — TS2715 exempt.
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
            // Arrow functions do not have their own `arguments` object.
            let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
            if is_arrow {
                self.push_arrow_function_scope(node);
            } else {
                self.push_function_scope(node);
            }
            // Push the declared return type so `return expr;` statements
            // in the body can be checked against it. `None` means no
            // explicit return-type annotation (return type inferred). For
            // async function-likes a `Promise<X>` annotation unwraps to `X`
            // (return values are promisified).
            let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
            let declared_return = type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn))
                .map(|t| self.unwrap_async_return_type(t, is_async));
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
            self.in_ctor_body_stack.pop();
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
    /// Go's `getCannotFindNameDiagnosticForName` — name-specific
    /// "cannot find name" variants (install-@types suggestions). Returns
    /// the message that REPLACES the plain TS2304. The Map/Set/... ES-name
    /// arm (target-library suggestions) is not ported yet.
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

    fn check_identifier_reference(&mut self, node: &Arc<Node>) {
        // Skip if the identifier's text is empty (parser recovery).
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return,
        };
        if name.is_empty() {
            return;
        }
        // Skip identifiers whose text is not a valid JS/TS identifier (parser
        // recovery artifacts: punctuation like `(`, `{`, `)`, `;` leaked into
        // Identifier nodes). Valid identifiers start with a letter, `_`, or `$`.
        if !is_valid_identifier_text(name) {
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
        // TS2301: an instance member initializer referencing a name declared
        // in the class constructor (parameter or local). Reported even when
        // the name resolves to an outer declaration, because the initializer
        // is emitted into the constructor where that declaration shadows it.
        // Mirrors Go's `checkAndReportErrorForInvalidInitializer`, reached
        // from the name resolver's PropertyDeclaration scope-climb case.
        if self.check_invalid_initializer_reference(node, name) {
            return;
        }

        // If we're inside a type node (e.g., heritage clause expression),
        // suppress TS2304 to avoid false positives for global names.
        if !self.ts2304_reporting_allowed_for(node) {
            return;
        }

        if let Some(symbol) = self.resolve_identifier(node) {
            // TS2708: a types-only namespace used as a VALUE. Follow the
            // alias base (import-equals → module), then require the
            // namespace to have at least one value member (Go's
            // checkAndReportErrorForUsingNamespaceAsTypeOrValue).
            // `export = ns` may legitimately reference a namespace (Go's
            // isExportAssignmentExpressionName exemption).
            let is_export_assignment_name = node
                .parent
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::ExportAssignment);
            let base = self.resolve_alias_base(Arc::clone(&symbol));
            if !is_export_assignment_name
                && base.flags.contains(SymbolFlags::ValueModule)
                && base
                    .declarations
                    .iter()
                    .any(|d| d.kind == SyntaxKind::ModuleDeclaration)
                && !self.namespace_has_value_side(&base)
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
            // TS2448: Block-scoped variable used before its declaration
            // (temporal dead zone). When a `let`/`const`/`class` variable is
            // referenced before its declaration position, report TS2448.
            // Mirrors Go's `checkResolvedBlockScopedVariable`.
            self.check_block_scoped_variable_used_before_declaration(node, &symbol, name);
            // TS2454: Variable used before being assigned. A `let` variable
            // with a non-undefined type annotation, no initializer, and no
            // assignment before the usage point reports TS2454. Only checked
            // under strictNullChecks. Mirrors Go's definite-assignment check
            // in `getFlowTypeOfReference`.
            self.check_variable_used_before_assigned(node, &symbol, name);
            return;
        }

        // Emit TS2662/TS2663 when inside a class and a static/instance member
        // with this name exists (Go's `checkAndReportErrorForMissingPrefix`:
        // a matching static member wins regardless of context; a matching
        // instance member suggests `this.x` only from directly inside an
        // instance member). Otherwise TS2304 "Cannot find name '{0}'."
        let file = self.current_file.clone();
        // Class-member suggestions (TS2662/TS2663) take precedence over
        // spelling suggestions; TS2552 spelling suggestions apply distance-1
        // visible names only (Go's getSpellingSuggestion behavior).
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


    /// TS17009: walk a derived-class constructor body in source order;
    /// every `this` expression seen before the `super()` call reports.
    /// `super(...)` marks super-called only after its arguments are
    /// visited (arguments evaluate first — `super(this.x)` errors on the
    /// `this`).
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
            // `super(args)` — visit arguments first, then mark super called.
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
            // `this` inside nested function-likes belongs to THEM (or is
            // deferred, for arrows) — only DIRECT constructor-body `this`
            // accesses report (Go's checkSuperCallBeforeThisAccessing:
            // nested functions/arrows are exempt; checkSuperCallBefore-
            // ThisAccessing4/6).
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
            // A nested class's constructors are fresh contexts with their
            // own super/this rules (checkSuperCallBeforeThisAccessing3).
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

    /// Find a visible name (scope stack containers + globals) within edit
    /// distance 2 of `name`, case-insensitively — the best (lowest)
    /// distance wins. Mirrors Go's spelling-correction suggestions.
    fn find_name_suggestion(&self, name: &str, meaning: SymbolFlags) -> Option<String> {
        let lower = name.to_ascii_lowercase();
        // Candidate symbols filtered by MEANING like Go's `getCandidateName`
        // (a type-only global such as `IArguments` is never suggested for a
        // VALUE reference like `arguments`).
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
        let mut best: Option<(usize, (usize, usize), &String)> = None;
        for sym in candidates {
            let cand: &String = &sym.name;
            if cand.len() < 2 || cand == name {
                continue;
            }
            let d = edit_distance(&lower, &cand.to_ascii_lowercase());
            if d > 1 {
                continue;
            }
            // Go's tie-break (compareSymbols → compareNodes): earliest
            // (program file index, declaration position) wins — this also
            // makes the result deterministic over HashMap iteration order.
            let key = self.suggestion_order_key(sym);
            let replace = match &best {
                None => true,
                Some((bd, bkey, _)) => d < *bd || (d == *bd && key < *bkey),
            };
            if replace {
                best = Some((d, key, cand));
            }
        }
        best.map(|(_, _, c)| c.clone())
    }

    /// Ordering key for spelling-suggestion ties (Go's compareNodes):
    /// (program file index, first-declaration position). Symbols without
    /// declarations sort last.
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

    /// Whether `node` sits inside a function-like/accessor body (before any
    /// module boundary) — statements there are covered by TS1183 (ambient
    /// implementations), not TS1036 (Go's checkGrammarStatementInAmbientContext
    /// function-like-parent branch).
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

    /// TS2654/TS2416: class-heritage member checks for a derived class —
    /// abstract members of the base chain not implemented by a non-abstract
    /// class, and derived property types not assignable to the same-named
    /// base property (Go's `checkClassHeritage` member relation).
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
        // TS2654: only non-abstract classes must implement abstract members.
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
        // TS2416: each own property or accessor that shadows a base
        // member must have a type assignable to the base member's.
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
                    // Annotated return type, else inferred from the body's
                    // return expressions.
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

    /// The member list of a class-like node (empty for other kinds).
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

    /// Find a class member by (identifier) name.
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

    /// Collect abstract members of the base chain that `class` (and its
    /// bases) don't implement concretely. Walks the `extends` chain.
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
                // Implemented if ANY class in the derived chain (starting
                // at `class`) declares a non-abstract member with the name.
                if !Self::chain_implements(class, name) {
                    out.push(name.to_string());
                }
            } else if out.iter().any(|m| m == name) {
                // A later concrete base member implements an earlier
                // abstract one.
                out.retain(|m| m != name);
            }
        }
    }

    /// The expression of the first `return <expr>;` in a body (used to
    /// infer a getter's return type when unannotated).
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

    /// Whether `class` or any of its bases declares a CONCRETE member named
    /// `name`.
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
        // Recurse into this class's own base (node-linked via heritage
        // identifiers is not available without resolution; single-level
        // check covers direct implementation).
        false
    }

    /// Find `<name> = <rhs>` assignments in a body: returns the LHS
    /// identifier's location and the RHS node for each.
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

    /// Resolve a (possibly qualified) entity name — `A`, `A.B`, `A.B.C` —
    /// to a symbol: the leftmost identifier through the scope stack, then
    /// each subsequent segment through the previous symbol's exports
    /// (modules/namespaces) or members. Follows alias symbols along the way;
    /// an `import X = require("...")` alias resolves to the module file's
    /// symbol (Go: `resolveEntityName` + `resolveExternalModuleName`).
    pub fn resolve_qualified_symbol(&mut self, name: &Arc<Node>) -> Option<Arc<Symbol>> {
        match self.resolve_qualified_symbol_traced(name) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    /// Traced qualified-name resolution: on failure, reports WHERE the
    /// chain broke — (segment node, namespace-path text, member name) —
    /// enough for the caller to emit TS2503 (unresolved namespace) or
    /// TS2694 (namespace has no exported member).
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
                let mut symbol = self.resolve_qualified_symbol_traced(&data.left)?;
                let path_so_far = qualified_name_text(&data.left);
                symbol = self.resolve_alias_base(symbol);
                // `right` is always an Identifier in valid syntax.
                let text = data.right.text();
                let mut next = symbol
                    .exports
                    .get(text)
                    .or_else(|| symbol.members.get(text))
                    .cloned()
                    .or_else(|| self.ambient_namespace_local(&symbol, text));
                // Single-hop `export = <ns>` chase: when the module
                // exports a namespace via export=, members resolve there
                // (Go's resolveEntityName through export assignments). No
                // recursion — the target's tables only.
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
                    // Resolve the export= entity with the module's scope
                    // pushed, then look the member up in its tables.
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
                                .cloned();
                        }
                    }
                }
                // A base that is still a pure alias from an UNRESOLVED
                // `require(...)` degrades to error (Go's error type):
                // member lookups through it stay silent instead of TS2694.
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
                        // An alias member (`export import X = N` inside the
                        // namespace) resolves through its declaration's
                        // reference — Go's resolveAlias → resolveEntityName —
                        // evaluated in the scope of the module declaring it
                        // (the bare target name `N` is not visible from
                        // here). Non-alias members keep the export_symbol
                        // chase.
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
                            Arc::clone(&data.right),
                            Self::namespace_full_path(&symbol),
                            text.to_string(),
                        ))
                    }
                }
            }
            _ => Err((Arc::clone(name), String::new(), String::new())),
        }
    }

    /// Go's implicit export for ambient containers (`setExportContextFlag` +
    /// `declareModuleMember`): every member of an ambient namespace (a
    /// `declare namespace`, or a namespace in a .d.ts) is exported when the
    /// container has no explicit export declarations. The binder keeps such
    /// members in the namespace's LOCALS (routing them into `exports`
    /// perturbs lazily-resolved lib types), so outside visibility of ambient
    /// locals is decided here.
    pub(crate) fn ambient_namespace_locals_visible(&self, ns: &Arc<Symbol>) -> bool {
        if std::env::var_os("TSOX_NO_AMBIENT").is_some() {
            return false;
        }
        ns.declarations.iter().any(|d| {
            d.kind == SyntaxKind::ModuleDeclaration
                && (d.has_syntactic_modifier(ModifierFlags::Ambient)
                    || self
                        .get_source_file_of_node(d)
                        .is_some_and(|f| f.is_declaration_file))
                && !crate::binder::Binder::has_export_declarations(d)
        })
    }

    /// Ambient member lookup: the namespace's locals when the namespace is
    /// an implicit-export ambient container (see
    /// [`Self::ambient_namespace_locals_visible`]).
    fn ambient_namespace_local(&self, ns: &Arc<Symbol>, name: &str) -> Option<Arc<Symbol>> {
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

    /// Resolve an alias symbol to its underlying base symbol: follows the
    /// `export_symbol` chain, and for `import X = require("./m")` aliases
    /// resolves the module file's symbol from the program's loaded files.
    fn resolve_alias_base(&mut self, symbol: Arc<Symbol>) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return symbol;
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            if let crate::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {
                // `import X = require("...")` — the module file/ambient
                // module symbol.
                if let crate::ast::NodeData::ExternalModuleReference(ext) =
                    &data.module_reference.data
                    && ext.expression.kind == SyntaxKind::StringLiteral
                    && let Some(module_sym) =
                        self.resolve_module_file_symbol(&ext.expression.text())
                {
                    return module_sym;
                }
                // `import X = NS.Path` — follow the entity-name target
                // (up to four alias hops to avoid cycles).
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

    /// Find the file symbol of an already-loaded module by specifier
    /// (relative to the current file's directory), trying the common
    /// TypeScript extension/index forms.
    pub(crate) fn resolve_module_file_symbol(&self, specifier: &str) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {
            // Ambient module: a top-level `declare module "name"` in any
            // loaded file (Go's resolveExternalModuleName consults the
            // global module map).
            for file in self.program.source_files() {
                // Augmentations (declare module inside an external-module
                // file) don't make the name resolvable.
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
        let stem = specifier.strip_prefix("./").unwrap_or(specifier);
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

    /// The source text of a class declaration's name ("" when anonymous).
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

    /// Whether the class has a property/method/accessor member named `name`
    /// and, if so, whether that member is `static`. Scans the class's member
    /// list declarationally (Go consults the constructor type's properties
    /// and the instance type — equivalent for direct members; inherited
    /// statics/instance members from base classes are not yet considered).
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

    /// TS2301: `Initializer of instance member variable 'x' cannot reference
    /// identifier 'y' declared in the constructor.` Walks up from the
    /// identifier to the nearest enclosing non-static property declaration
    /// and checks whether the class's constructor (the one with a body)
    /// declares the name as a parameter or local. Mirrors the Go name
    /// resolver's `KindPropertyDeclaration` scope-climb case plus
    /// `checkAndReportErrorForInvalidInitializer`. Returns true when the
    /// diagnostic was reported (resolution treats the name as "handled").
    /// TS2393: `Duplicate function implementation.` Collects same-name
    /// `function` declarations among the node's siblings (same container);
    /// when two or more have bodies, reports on every one of them (overload
    /// signatures included — oracle-verified: `function f(): number;` above
    /// two implementations also reports at the signature). Skipped in
    /// ambient contexts (.d.ts / `declare`), mirroring Go's
    /// `checkFunctionOrConstructorSymbolWorker`.
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
        // Report only when visiting the first declaration of the name, so the
        // per-name error set is emitted exactly once.
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
        // TS2384 (Go `checkFlagAgreementBetweenOverloads`): the canonical
        // declaration is the implementation when one exists in this
        // container, else the first overload; every body-less overload
        // whose ambient-ness deviates reports on its name.
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

    /// TS2392: `Multiple constructor implementations are not allowed.` When
    /// two or more of the class's constructor declarations have bodies,
    /// report on every constructor declaration (overload signatures
    /// included). Mirrors Go's `checkFunctionOrConstructorSymbolWorker` via
    /// the constructor symbol's merged declarations.
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
        if property.has_syntactic_modifier(ModifierFlags::Static) {
            return false;
        }
        let Some(class) = property.parent.as_ref() else {
            return false;
        };
        if !matches!(class.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
            return false;
        }
        // Go's FindConstructorDeclaration: the first constructor with a body.
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

    /// Contextual element check for literal expressions against a target
    /// type (Go's contextual typing of array/object literals): recurses
    /// into array-literal elements against the target's element type,
    /// reports TS2353 excess properties and TS2741 missing properties on
    /// object literals, and TS2322 for primitive-literal elements that
    /// don't match a non-any target. Anchored at the offending property
    /// name / element, like the official baselines.
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
            let elem_t = self.get_array_element_type(target);
            if elem_t.flags.contains(TypeFlags::Any) {
                // A numeric index signature on the target (`interface I {
                // [x: number]: Date }`) also types the elements.
                let indexed = target.as_structured().and_then(|s| {
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
        // Type-assertion elements (`[<foo>({})]`): check the ASSERTED type
        // against the element type (Go checks the contextually-typed
        // assertion result — TS2741 on class types missing properties).
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
        // Primitive literals: report only genuine mismatches (null/undefined
        // excluded — their assignability depends on strictNullChecks).
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
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                expr.loc,
                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                vec![src_str, tgt_str],
            ));
        }
    }

    /// For an async function-like, the effective type that `return expr`
    /// must satisfy: a declared `Promise<X>` unwraps to `X` (async return
    /// values are promisified). Mirrors Go's `getReturnTypeOfSignature`
    /// handling of async functions (`getPromisedTypeOfPromise`).
    fn unwrap_async_return_type(&self, declared: Arc<Type>, is_async: bool) -> Arc<Type> {
        if !is_async {
            return declared;
        }
        // Promise<X> with one type argument → X. Generic references that
        // are not yet instantiated (the type argument was dropped during
        // resolution) degrade to `any` — the promised type is unknowable,
        // so return-value checking is suppressed rather than mis-reported.
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

    /// The annotated type of an identifier's variable binding — used to
    /// contextually check assignment right-hand sides (`foo = {...}` where
    /// `foo: {id:number}`). Only VariableDeclaration annotations are
    /// consulted; parameters and properties are checked at their own sites.
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

    /// Go `isValidConstAssertionArgument`: valid left sides of `as const`.
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
                // An enum-member reference: `E.a` / `E["a"]` resolving to an
                // enum symbol.
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

    /// Go `isConstTypeReference`: the type side of an `as const` assertion
    /// is exactly the keyword `const` (the parser produces a `ConstKeyword`
    /// type node for the asserted type).
    fn is_const_type_node(type_node: &Arc<Node>) -> bool {
        type_node.kind == SyntaxKind::ConstKeyword
    }

    /// Delete-operand checks (Go `checkDeleteExpression`): TS1102
    /// (identifier operand in strict mode) + TS2703 (non-property operand),
    /// both on the operand; TS2704 when the property is read-only, on the
    /// property name.
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
                // Strict-family default is ON (tsgo semantics: Unknown →
                // true), plus modules are always strict.
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
                let (obj_expr, name, name_loc) = match &target.data {
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
                // `any` object → no checks.
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
                // TS2542: read-only index signature
                // ('readonly [k: string]: T' — 'delete b["test"]').
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
                // TS2790 (strictNullChecks): the deleted property must be
                // optional.
                if self.strict_null_checks && self.has_property_of_type(&obj_type, &name) {
                    // Go's rule: the operand is deletable when the property
                    // is optional OR its type includes `undefined`
                    // ('b: number | undefined' deletes fine).
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
                        // A string index signature permits deleting any
                        // property ('[s: string]: number').
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

    /// TS2588/TS2540 for assignment-like targets. `x = 1` / `++x` with an
    /// identifier LHS resolving to a `const` binding reports TS2588 on the
    /// operand; `M.x = 1` / `++M.x` where `M` is a namespace and `x` a
    /// const member reports TS2540 (read-only property) on the property
    /// name (Go: the assignment pipeline's `checkReferenceExpression`
    /// resolving the property symbol and checking its const-ness).
    fn check_const_assignment_target(&mut self, operand: &Arc<Node>) {
        // Unwrap parentheses (`++((x))` — Go's checkReferenceExpression
        // walks through parenthesized expressions).
        let mut target = operand;
        loop {
            target = match &target.data {
                // Parentheses (`++((x))`) and non-null assertions (`x!++`)
                // — Go's checkReferenceExpression walks through both.
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

    /// TS2540: assigning to a namespace's `const` member (`M.x = 1`).
    /// Resolves the member through the namespace symbol's exports/members
    /// (and the module declaration's locals for ambient namespaces) — the
    /// same lookup as `get_type_of_property_access`'s namespace path.
    fn check_const_property_assignment(&mut self, node: &Arc<Node>) {
        // `M.x` or `M["x"]` (a string-literal element access behaves like a
        // property access).
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

    /// TS2448: Check if a block-scoped variable (`let`/`const`/`class`) is
    /// used before its declaration. Mirrors Go's
    /// `checkResolvedBlockScopedVariable` + `isBlockScopedNameDeclaredBeforeUse`.
    ///
    /// Reports TS2448 when the resolved symbol is block-scoped (or a class)
    /// and the reference's source position precedes the declaration's
    /// source position.
    fn check_block_scoped_variable_used_before_declaration(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {
        // Only block-scoped variables and classes are subject to the TDZ
        // check. `var` (function-scoped) and `function` declarations are
        // hoisted.
        if !symbol
            .flags
            .intersects(SymbolFlags::BlockScopedVariable | SymbolFlags::Class)
        {
            return;
        }
        // Skip references inside function/arrow-function bodies — those are
        // deferred (the body runs later, by which point the variable is
        // initialized). Mirrors Go's `isUsedInFunctionOrInstanceProperty`.
        if self.function_scope_count > 0 || self.arrow_function_scope_count > 0 {
            return;
        }
        // Find the relevant declaration node (block-scoped or class-like).
        // `BindingElement` covers destructuring declarations —
        // `let {[a]: a} = …` declares `a` via a binding element, whose
        // computed property name may reference `a` itself (TS2448).
        let declaration = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
            )
        });
        let Some(declaration) = declaration else {
            return;
        };
        // Skip `var` declarations — they are function-scoped and hoisted.
        // The binder assigns BlockScopedVariable to all variable declarations,
        // so we must verify the keyword is actually `let`/`const`.
        if declaration.kind == SyntaxKind::VariableDeclaration
            && !is_let_or_const_declaration(declaration)
        {
            return;
        }
        // Skip ambient declarations (`declare let x`).
        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
        {
            return;
        }
        // If the declaration position is at or before the usage position,
        // the variable is declared before use — no error. The position used
        // is the declaration's NAME node (`: b` in `{[b]: b}`), not the
        // declaration's start: a binding element's computed property name
        // precedes but lies within the element's span.
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
            return;
        }
        // Cross-file references never report TS2448 (Go's
        // `isBlockScopedNameDeclaredBeforeUse`: nodes in different files
        // have no determinable order — e.g. `const x = 0` in file1.ts,
        // `x++` in file2.ts).
        let decl_file = self.get_source_file_of_node(declaration);
        let use_file = self.get_source_file_of_node(node);
        if let (Some(df), Some(uf)) = (&decl_file, &use_file) {
            if df.file_name != uf.file_name {
                return;
            }
        }
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            BLOCK_SCOPED_VARIABLE_0_USED_BEFORE_ITS_DECLARATION,
            vec![name.to_string()],
        ));
    }

    /// TS2454: Check if a `let` variable is used before being assigned a
    /// value. Mirrors (as a simplified heuristic) Go's definite-assignment
    /// check in `getFlowTypeOfReference`.
    ///
    /// Reports TS2454 when, under strictNullChecks, a `let` variable has a
    /// non-undefined type annotation, no initializer, and no assignment to
    /// it in the flow graph between its declaration and the usage point.
    fn check_variable_used_before_assigned(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {
        // Skip assignment targets (e.g., the `v` in `v = 1`). TS2454 applies
        // to reads, not writes.
        if is_assignment_target(node) {
            return;
        }
        // Only check under strictNullChecks (`@strict: false` cases like
        // ambiguousOverloadResolution stay clean; the CLI default resolves
        // it ON). `any`-typed variables are exempt (anyPlusAny1).
        if !self.strict_null_checks {
            return;
        }
        // Only block-scoped variables (let/const). `const` without an
        // initializer is a syntax error, so in practice this is `let`.
        if !symbol.flags.contains(SymbolFlags::BlockScopedVariable) {
            return;
        }
        // Find the variable declaration.
        let declaration = symbol.value_declaration.as_ref().or_else(|| {
            symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::VariableDeclaration)
        });
        let Some(declaration) = declaration else {
            return;
        };
        // `var` is subject to the same definite-assignment check as
        // `let`/`const` under strictNullChecks (Go's flow analysis reports
        // `var x: number; use(x)` too).
        let crate::ast::NodeData::VariableDeclaration(vd) = &declaration.data else {
            return;
        };
        // Must have a type annotation and no initializer. A variable with an
        // initializer is already assigned; a variable without a type
        // annotation has type `any` (or is auto-typed) and is not subject to
        // the definite-assignment check.
        if vd.initializer.is_some() || vd.type_node.is_none() {
            return;
        }
        // Skip ambient declarations (`declare let x`) and definite-assignment
        // assertions (`let x!: number`). Mirrors Go's `assumeInitialized`.
        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
            || vd.exclamation_token.is_some()
        {
            return;
        }
        // The declared type must not include `undefined`, and an `any`
        // declaration is exempt.
        let declared_type = self.get_type_of_symbol(symbol);
        if declared_type.flags.contains(TypeFlags::Any)
            || type_contains_undefined(&declared_type)
        {
            return;
        }
        // Check the flow graph: has the variable been definitely assigned
        // between its declaration and this usage?
        if self.is_symbol_definitely_assigned_at(node, symbol) {
            return;
        }
        let file = self.current_file.clone();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED,
            vec![name.to_string()],
        ));
    }

    /// Whether `symbol` has been definitely assigned a value at the flow
    /// point of `node`. Walks the flow graph backwards from the reference's
    /// flow node, looking for an ASSIGNMENT flow node that targets `symbol`.
    ///
    /// This is a simplified version of Go's definite-assignment analysis.
    /// It follows the linear antecedent chain (and label antecedents) without
    /// handling branch joins — sufficient for the common single-assignment-
    /// before-use pattern.
    ///
    /// Additionally, we use a pre-scan heuristic (mirroring Go's
    /// `markNodeAssignmentsWorker`): if there is ANY assignment to `symbol`
    /// within the same enclosing function (even in a conditional branch), we
    /// suppress TS2454. This avoids false positives from imprecise flow
    /// analysis at the cost of missing some genuine errors.
    fn is_symbol_definitely_assigned_at(&mut self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        // Flow graph analysis.
        let flow = match self.program.symbol_map().flow_node_of(node) {
            Some(f) => Arc::clone(f),
            None => return true, // no flow info → assume assigned (no false positive)
        };
        self.flow_has_assignment_to_symbol(&flow, symbol, &mut HashSet::new(), 0)
    }

    /// Pre-scan heuristic: walk the enclosing function body looking for any
    /// assignment (=, +=, ++, etc.) whose target resolves to `symbol`.
    /// Mirrors Go's `markNodeAssignmentsWorker` (flow.go:2673-2694) which
    /// records `lastAssignmentPos` for each symbol in the same function.
    fn symbol_has_assignment_in_enclosing_function(
        &self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        // Find the enclosing function or source file.
        let func_body = match self.find_enclosing_function_body(node) {
            Some(b) => b,
            None => return false,
        };
        // Walk the function body looking for assignments.
        self.scan_for_assignment(&func_body, symbol, &mut HashSet::new())
    }

    /// Find the body (Block) of the enclosing function or arrow function.
    fn find_enclosing_function_body(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = node.parent.as_ref()?;
        loop {
            match current.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => {
                    // Return the function body if present.
                    return crate::ast::node_data_generated::for_each_child(current, |child| {
                        child.kind != SyntaxKind::Block && child.kind != SyntaxKind::Identifier
                    })
                    .then(|| Arc::clone(current))
                    .or_else(|| {
                        // Find Block child.
                        let mut body = None;
                        crate::ast::node_data_generated::for_each_child(current, |child| {
                            if child.kind == SyntaxKind::Block {
                                body = Some(Arc::clone(child));
                                false // stop
                            } else {
                                true // continue
                            }
                        });
                        body
                    });
                }
                SyntaxKind::SourceFile => return None,
                _ => {
                    current = current.parent.as_ref()?;
                }
            }
        }
    }

    /// Recursively scan a node tree for assignments targeting `symbol`.
    fn scan_for_assignment(
        &self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        visited: &mut HashSet<usize>,
    ) -> bool {
        let key = Arc::as_ptr(node) as *const Node as usize;
        if !visited.insert(key) {
            return false;
        }
        // Check if this node is an assignment targeting the symbol.
        if self.is_assignment_to_symbol(node, symbol) {
            return true;
        }
        // Recurse into children using for_each_child.
        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if self.scan_for_assignment(child, symbol, visited) {
                found = true;
                false // stop
            } else {
                true // continue
            }
        });
        found
    }

    /// Whether `node` is an assignment expression targeting `symbol`.
    fn is_assignment_to_symbol(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData::*;
        match &node.data {
            BinaryExpression(bin) => {
                if is_compound_or_simple_assignment(bin.operator_token.kind) {
                    return self.identifier_is_symbol(&bin.left, symbol);
                }
                false
            }
            PostfixUnaryExpression(unary) => {
                (unary.operator == SyntaxKind::PlusPlusToken
                    || unary.operator == SyntaxKind::MinusMinusToken)
                    && self.identifier_is_symbol(&unary.operand, symbol)
            }
            PrefixUnaryExpression(unary) => {
                (unary.operator == SyntaxKind::PlusPlusToken
                    || unary.operator == SyntaxKind::MinusMinusToken)
                    && self.identifier_is_symbol(&unary.operand, symbol)
            }
            // ForOfStatement / ForInStatement: the loop variable is assigned.
            ForInOrOfStatement(for_loop) => {
                self.for_initializer_targets_symbol(&for_loop.initializer, symbol)
            }
            _ => false,
        }
    }

    /// Check if a for-loop initializer (`var x`, `let x`, or `x`) targets
    /// `symbol`. Used for for-of/for-in definite assignment.
    fn for_initializer_targets_symbol(&self, init: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        match &init.data {
            NodeData::VariableDeclarationList(vdl) => vdl.declarations.iter().any(|decl| {
                if let NodeData::VariableDeclaration(vd) = &decl.data {
                    self.identifier_is_symbol(&vd.name, symbol)
                } else {
                    false
                }
            }),
            _ => self.identifier_is_symbol(init, symbol),
        }
    }

    /// Recursive helper: walk the flow graph backwards looking for an
    /// ASSIGNMENT to `symbol`. Uses a visited set keyed by flow-node pointer
    /// to avoid infinite loops on cyclic flow graphs (loops, labels).
    fn flow_has_assignment_to_symbol(
        &mut self,
        flow: &Arc<FlowNode>,
        symbol: &Arc<Symbol>,
        visited: &mut HashSet<usize>,
        depth: u32,
    ) -> bool {
        if depth >= 64 {
            return true; // depth guard → assume assigned (conservative)
        }
        let key = Arc::as_ptr(flow) as *const FlowNode as usize;
        if !visited.insert(key) {
            return true; // already visited → assume assigned (break cycle)
        }
        // START / UNREACHABLE → end of the flow graph; no assignment found.
        if flow
            .flags
            .intersects(FlowFlags::START | FlowFlags::UNREACHABLE)
        {
            return false;
        }
        // ASSIGNMENT → check if the assignment targets our symbol.
        if flow.flags.contains(FlowFlags::ASSIGNMENT) {
            if let Some(expr) = &flow.node {
                if self.assignment_targets_symbol(expr, symbol) {
                    return true;
                }
            }
        }
        // Recurse into the antecedent.
        if let Some(antecedent) = &flow.antecedent {
            return self.flow_has_assignment_to_symbol(antecedent, symbol, visited, depth + 1);
        }
        // Branch joins: all antecedents must have an assignment for it to be
        // "definite". For the heuristic, we return true if any antecedent has
        // one (conservative — avoids false TS2454 positives).
        if !flow.antecedents.is_empty() {
            return flow
                .antecedents
                .iter()
                .any(|a| self.flow_has_assignment_to_symbol(a, symbol, visited, depth + 1));
        }
        false
    }

    /// Whether the flow-node expression `expr` is an assignment (including
    /// compound assignment and ++/--) whose left-hand side resolves to
    /// `symbol`.
    fn assignment_targets_symbol(&self, expr: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData::*;
        match &expr.data {
            BinaryExpression(bin) => {
                if is_compound_or_simple_assignment(bin.operator_token.kind) {
                    return self.identifier_is_symbol(&bin.left, symbol);
                }
                false
            }
            PostfixUnaryExpression(unary) => {
                (unary.operator == SyntaxKind::PlusPlusToken
                    || unary.operator == SyntaxKind::MinusMinusToken)
                    && self.identifier_is_symbol(&unary.operand, symbol)
            }
            PrefixUnaryExpression(unary) => {
                (unary.operator == SyntaxKind::PlusPlusToken
                    || unary.operator == SyntaxKind::MinusMinusToken)
                    && self.identifier_is_symbol(&unary.operand, symbol)
            }
            _ => false,
        }
    }

    /// Whether `node` is an identifier that resolves to `symbol`. A name-based
    /// fallback is used when the binder hasn't set a symbol on the node.
    fn identifier_is_symbol(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        if node.kind != SyntaxKind::Identifier {
            return false;
        }
        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            return Arc::ptr_eq(sym, symbol);
        }
        match &node.data {
            NodeData::Identifier(data) => data.text == symbol.name,
            _ => false,
        }
    }

    /// Push a container node onto the scope stack, making its symbol members
    /// and locals visible for identifier resolution.
    /// Enter a TS2304-suppression scope, remembering the originating file
    /// (see [`Self::ts2304_reporting_allowed`]).
    pub(crate) fn push_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes += 1;
        if self.suppress_source_file.is_none() {
            self.suppress_source_file = self.current_file.as_ref().map(|f| f.node.id());
        }
    }

    /// Leave a TS2304-suppression scope (clears the origin at the outermost pop).
    pub(crate) fn pop_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes = self
            .suppress_cannot_find_name_in_type_nodes
            .saturating_sub(1);
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            self.suppress_source_file = None;
        }
    }

    /// Whether TS2304-family diagnostics may be reported in the CURRENT
    /// file: allowed when no suppression is active, or when the active
    /// suppression originates in a DIFFERENT file (cross-file resolution
    /// must not inherit the origin file's suppression).
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
                    // Same file the suppression started in — intended.
                    false
                } else {
                    // Cross-file resolution: USER files always report
                    // (bundled-lib signature type parameters stay silenced
                    // wherever they're reached from).
                    !f.file_name.starts_with("bundled://")
                }
            }
            _ => false,
        }
    }

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
            // declarations like parameters, enum members, etc.).
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                // Class members are NOT lexically visible: `foo` inside a
                // method must be written `this.foo` / `C.foo` (Go's
                // `resolveName` never consults class `Members`; the checker
                // instead reports TS2662/TS2663 suggestions when a bare name
                // fails to resolve inside a class). Exception: a symbol that
                // merged an ambient class with a function
                // (`declare class P {}` + `function P() {}`) is ALSO the
                // function's declaration symbol — its members include the
                // function's parameters, which ARE in scope in the body
                // (oracle: no TS2304 for `function P(x) { this.x = x; }`).
                if !container_sym.flags.intersects(SymbolFlags::Class)
                    || container_sym.flags.intersects(SymbolFlags::Function)
                {
                    if let Some(sym) = container_sym.members.get(name) {
                        if sym.flags.intersects(meaning) {
                            return self.follow_alias(sym);
                        }
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

        // 1b. Node-location ancestry walk (Go's NameResolver resolves by the
        // reference's lexical location, not the checker's dynamic position).
        // Interface types are built lazily and may first be resolved from a
        // foreign context (another file/statement), where the dynamic scope
        // stack above doesn't contain the declaration's enclosing
        // namespaces — walk the reference node's own parent chain instead.
        // Runs after the dynamic walk so shadowing behavior is unchanged;
        // only names that would otherwise be unresolved are recovered.
        {
            // Only true LEXICAL containers participate — object literals,
            // interfaces, classes, and property-like nodes are not
            // name-resolution scopes in Go (a sibling property `{ a: a }'
            // must not satisfy the value reference).
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
                SyntaxKind::Constructor,
                SyntaxKind::GetAccessor,
                SyntaxKind::SetAccessor,
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
                        && sym.flags.intersects(meaning)
                    {
                        return self.follow_alias(sym);
                    }
                }
                if let Some(a_sym) = symbol_map.symbols.get(&aid)
                    && !a_sym.flags.intersects(SymbolFlags::Class)
                {
                    if let Some(sym) = a_sym.members.get(name)
                        && sym.flags.intersects(meaning)
                    {
                        return self.follow_alias(sym);
                    }
                    if a_sym.flags.intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                        && let Some(sym) = a_sym.exports.get(name)
                        && sym.flags.intersects(meaning)
                    {
                        return self.follow_alias(sym);
                    }
                }
                ancestor = a.parent.as_ref();
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
        // Follow the export_symbol chain, stopping on any cycle. Exported
        // module members carry a self-referential export link (binder's
        // single-symbol export pattern), so a pure alias whose export_symbol
        // points back into the chain must not be chased forever — Go's eager
        // alias resolution (aliasMarker) never re-enters an alias being
        // resolved; on a cycle it yields the alias itself.
        let mut current = Arc::clone(symbol);
        let mut seen: Vec<*const Symbol> = vec![Arc::as_ptr(symbol)];
        loop {
            if let Some(ref target) = current.export_symbol {
                let target_ptr = Arc::as_ptr(target);
                if seen.contains(&target_ptr) {
                    // Cycle (A→A, A→B→A, …) — return the alias itself.
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
                    let merged = self.get_merged_symbol(export_symbol);
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

    /// Get the merged symbol.
    ///
    /// Go: `referenceResolver.getMergedSymbol`
    pub fn get_merged_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(_target_id) = self.merged_symbols.get(&symbol.id()) {
            // We need a way to look up symbols by ID. For now, return the symbol itself.
            // In a full implementation, we'd have a symbol_by_id map.
        }
        Arc::clone(symbol)
    }

    /// Get the export symbol of a value symbol if it's exported.
    ///
    /// Go: `referenceResolver.getExportSymbolOfValueSymbolIfExported`
    fn get_export_symbol_of_value_symbol_if_exported(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        let mut result = Arc::clone(symbol);
        if symbol.flags.intersects(SymbolFlags::ExportValue) {
            if let Some(ref export_sym) = symbol.export_symbol {
                result = self.get_merged_symbol(export_sym);
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
                target.insert(name, merged);
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
                    result_mut.members.insert(name, merged);
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
                    result_mut.exports.insert(name, merged);
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

    /// Get the symbol associated with a given AST node.
    ///
    /// A deliberately "fuzzy" lookup intended for language-service and
    /// external-tool use (not for type-checking internals). Mirrors Go's
    /// `Checker.GetSymbolAtLocation` / `getSymbolAtLocation` (reduced form).
    ///
    /// Handles three cases:
    /// 1. Declaration nodes with a direct symbol in the symbol map.
    /// 2. Declaration-name identifiers — walks the parent chain to find the
    ///    enclosing declaration and returns its symbol.
    /// 3. Property-access expressions — resolves the property symbol from
    ///    the cached type of the left-hand expression.
    pub fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        // 1. Direct symbol lookup (declaration nodes).
        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        // 2. Declaration-name identifiers: walk up the parent chain.
        if node.kind == crate::ast::SyntaxKind::Identifier {
            let mut current = node.parent.as_ref();
            while let Some(parent) = current {
                if let Some(sym) = self.program.symbol_map().symbol_of(parent) {
                    return Some(Arc::clone(sym));
                }
                current = parent.parent.as_ref();
            }
        }

        // 3. Property-access expressions: resolve the property from the
        //    cached type of the left-hand expression.
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
                    // Class/function EXPRESSION names are declaration-side
                    // too (`const W = class Wrapped {}` — `Wrapped` binds
                    // the class itself, scoping references inside the class
                    // body; it is not a reference to an outer variable).
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::FunctionExpression
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

/// Whether a string is valid as a JS/TS identifier name. Valid identifiers
/// must start with a letter, `_`, or `$`, and contain only alphanumeric,
/// `_`, or `$` characters. Used to filter out parser-recovery artifacts
/// (e.g. punctuation like `(`, `{`, `)` that leaked into Identifier nodes).
fn is_valid_identifier_text(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        // Allow Unicode identifiers (e.g. `$`, `café`). Non-ASCII letters are
        // valid identifier starts per ECMAScript spec.
        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Whether an identifier node is the left-hand side of an assignment (e.g.,
/// the `v` in `v = 1` or `v += 1`). Used to skip TS2454 for write targets.
fn is_assignment_target(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    if parent.kind != SyntaxKind::BinaryExpression {
        return false;
    }
    let crate::ast::NodeData::BinaryExpression(bin) = &parent.data else {
        return false;
    };
    if !is_compound_or_simple_assignment(bin.operator_token.kind) {
        return false;
    }
    // The node must be the left operand.
    std::ptr::eq(
        bin.left.as_ref() as *const Node,
        node.as_ref() as *const Node,
    )
}

/// Whether a `VariableDeclaration` node is declared with `let` or `const`
/// (as opposed to `var`). The parser sets `NodeFlags::Let` or `NodeFlags::Const`
/// on the parent `VariableDeclarationList`; `var` has neither.
fn is_let_or_const_declaration(declaration: &Arc<Node>) -> bool {
    if let Some(parent) = declaration.parent.as_ref() {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
        }
    }
    // No parent info — assume block-scoped (conservative).
    true
}

/// Whether a type includes `undefined` (directly or as a union constituent).
/// Mirrors Go's `containsUndefinedType`.
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

/// Whether a type is possibly `undefined` or `null` (directly or as a union
/// constituent). Mirrors Go's `(type.flags & TypeFlagsNullable) != 0` plus
/// union-constituent check.
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

/// Whether a `SyntaxKind` is `=` or any compound-assignment operator
/// (`+=`, `-=`, etc.). Mirrors Go's `isAssignmentOperator`.
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

/// Flatten a (possibly nested) union into its non-union leaf types.
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

/// The declared name text of a class declaration, if any.
fn class_declaration_name(class: &Arc<Node>) -> Option<String> {
    if let crate::ast::NodeData::ClassDeclaration(d) = &class.data {
        return d.name.as_ref().map(|n| n.text().to_string());
    }
    None
}

/// Whether a constructor body assigns `this.<name>` anywhere (outside nested
/// function-likes) — the TS2564 definite-assignment approximation.
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
        // Nested function-likes have their own `this`.
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

    // ── Ports of Go checker tests ──────────────────────────────────────────

    /// Port of Go's `TestGetSymbolAtLocation`.
    ///
    /// Go test sets up a program with an interface, a variable, and a property
    /// access expression, builds a type checker, then asserts that
    /// `checker.GetSymbolAtLocation` returns a non-nil symbol for the interface
    /// name, the variable name, and the property access expression.
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

        // Position 10 = 'F' in "interface Foo" (interface name identifier).
        let interface_name = get_token_at_position(&file.node, 10).expect("interface name");
        let sym = checker.get_symbol_at_location(&interface_name);
        assert!(sym.is_some(), "Expected symbol for interface name 'Foo'");

        // Position 47 = 'f' in "declare const foo" (variable name identifier).
        let var_name = get_token_at_position(&file.node, 47).expect("variable name");
        let sym = checker.get_symbol_at_location(&var_name);
        assert!(sym.is_some(), "Expected symbol for variable name 'foo'");

        // Position 60 = '.' in "foo.bar" (inside the PropertyAccessExpression,
        // between its children, so get_token_at_position returns the PAE itself).
        let prop_access = get_token_at_position(&file.node, 60).expect("property access");
        let sym = checker.get_symbol_at_location(&prop_access);
        assert!(
            sym.is_some(),
            "Expected symbol for property access 'foo.bar'"
        );
    }

    /// Port of Go's `TestTracerPushPreservesEndArgMutations`.
    ///
    /// The Go test verifies that `Tracer.Push` preserves end-arg mutations
    /// (the caller's `args` map can be mutated between push and pop, and the
    /// end event captures the mutation). That shared-mutable-args pattern is
    /// not representable under Rust ownership: `tracing::Tracer::push` takes
    /// `args` by value and snapshots them at push time.
    ///
    /// Rust adaptation: instead of testing args mutation between push/pop, we
    /// exercise the equivalent `tracing::Tracer` push/pop (begin/end) machinery
    /// and verify the behaviour that *is* expressible:
    /// 1. Push records a "B" (begin) phase event; drop (pop) records a matching
    ///    "E" (end) event with the same phase name and args snapshot.
    /// 2. The phase name and args snapshot captured at push time are correct.
    /// 3. Multiple nested pushes/pops maintain correct ordering (LIFO) and each
    ///    begin/end pair shares a thread id.
    #[test]
    fn tracer_push_preserves_end_arg_mutations() {
        use crate::tracing::{Phase, TraceArg, Tracer};

        let tr = Tracer::new();

        // Outer span with a `checkerId` arg.
        let args = vec![
            ("checkerId".to_string(), TraceArg::Int(7)),
            ("id".to_string(), TraceArg::Int(1)),
        ];
        let outer = tr.push(Phase::CheckTypes, "getVariancesWorker", args.clone());
        // The caller's `args` must remain untouched after `push` (it takes a
        // value, not a reference, so there is no leakage into the caller's
        // map — the Go invariant "Push does not leak checkerId").
        assert_eq!(args.len(), 2);

        // Nested span sharing the same checkerId thread.
        let inner_args = vec![("checkerId".to_string(), TraceArg::Int(7))];
        let inner = tr.push(Phase::Check, "checkSourceFile", inner_args);

        // Pop inner then outer (LIFO ordering).
        drop(inner);
        drop(outer);

        let events = tr.take_events();

        // Find the begin/end pair for the outer span.
        let outer_begin = events
            .iter()
            .find(|e| e.ph == "B" && e.name == "getVariancesWorker")
            .expect("outer begin event");
        let outer_end = events
            .iter()
            .find(|e| e.ph == "E" && e.name == "getVariancesWorker")
            .expect("outer end event");

        // 1. The begin event carries the args snapshot captured at push time.
        assert_eq!(outer_begin.cat, "checkTypes");
        assert_eq!(
            outer_begin.args,
            vec![
                ("checkerId".to_string(), TraceArg::Int(7)),
                ("id".to_string(), TraceArg::Int(1)),
            ]
        );

        // 2. The end (pop) event reproduces the same args snapshot as the
        //    begin event (Rust replays the snapshot rather than a live map).
        assert_eq!(outer_end.args, outer_begin.args);

        // 3. Begin and end of the same span share a thread id.
        assert_eq!(outer_begin.tid, outer_end.tid);

        // The inner span must share the checkerId=7 thread with the outer span.
        let inner_begin = events
            .iter()
            .find(|e| e.ph == "B" && e.name == "checkSourceFile")
            .expect("inner begin event");
        assert_eq!(inner_begin.tid, outer_begin.tid);

        // 3 (continued): nested pushes/pops maintain correct LIFO ordering —
        // the inner begin comes after the outer begin, and the inner end comes
        // before the outer end.
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


/// The dotted source text of a (possibly qualified) entity name.
fn qualified_name_text(name: &Arc<Node>) -> String {
    match &name.data {
        crate::ast::NodeData::QualifiedName(d) => {
            format!("{}.{}", qualified_name_text(&d.left), d.right.text())
        }
        _ => name.text().to_string(),
    }
}

/// Levenshtein edit distance (bounded early-exit at 3).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > 2 {
        return 3;
    }
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
