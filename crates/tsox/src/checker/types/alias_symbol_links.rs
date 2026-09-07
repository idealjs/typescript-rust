#![allow(unused_imports)]

use super::*;

#[derive(Debug, Default)]
pub struct AliasSymbolLinks {
    pub immediate_target: Option<Arc<Symbol>>,
    pub alias_target: Option<Arc<Symbol>>,
    pub referenced: bool,
    pub type_only_declaration: Option<Arc<Node>>,
}

#[derive(Debug, Default)]
pub struct ModuleSymbolLinks {
    pub resolved_exports: SymbolTable,
    pub type_only_export_star_map: HashMap<String, Arc<Node>>,
    pub exports_checked: bool,
}

#[derive(Debug, Default)]
pub struct ReverseMappedSymbolLinks {
    pub property_type: Option<Arc<Type>>,
    pub mapped_type: Option<Arc<Type>>,
    pub constraint_type: Option<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct LateBoundLinks {
    pub late_symbol: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct ExportTypeLinks {
    pub target: Option<Arc<Symbol>>,
    pub originating_import: Option<Arc<Node>>,
}

#[derive(Debug, Default)]
pub struct TypeAliasLinks {
    pub declared_type: Option<Arc<Type>>,
    pub type_parameters: Vec<Arc<Type>>,
    pub is_constructor_declared_property: bool,
}

#[derive(Debug, Default)]
pub struct DeclaredTypeLinks {
    pub declared_type: Option<Arc<Type>>,
    pub interface_checked: bool,
    pub index_signatures_checked: bool,
    pub type_parameters_checked: bool,
    pub enum_checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExhaustiveState {
    #[default]
    Unknown,
    Computing,
    False,
    True,
}

#[derive(Debug, Default)]
pub struct SwitchStatementLinks {
    pub exhaustive_state: ExhaustiveState,
    pub switch_types_computed: bool,
    pub witnesses_computed: bool,
    pub switch_types: Vec<Arc<Type>>,
    pub witnesses: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ArrayLiteralLinks {
    pub indices_computed: bool,
    pub first_spread_index: i32,
    pub last_spread_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MembersOrExportsResolutionKind {
    #[default]
    ResolvedExports,
    ResolvedMembers,
}

#[derive(Debug, Default)]
pub struct MembersAndExportsLinks {
    pub resolved_exports: SymbolTable,
    pub resolved_members: SymbolTable,
}

#[derive(Debug, Default)]
pub struct SpreadLinks {
    pub left_spread: Option<Arc<Symbol>>,
    pub right_spread: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct VarianceLinks {
    pub variances: Vec<VarianceFlags>,
}

#[derive(Debug, Default)]
pub struct MarkedAssignmentSymbolLinks {
    pub last_assignment_pos: i32,
    pub has_definite_assignment: bool,
}

#[derive(Debug, Default)]
pub struct NodeLinks {
    pub flags: NodeCheckFlags,
    pub declaration_requires_scope_change: Tristate,
    pub has_reported_statement_in_ambient_context: bool,
}

#[derive(Debug, Default)]
pub struct SymbolNodeLinks {
    pub resolved_symbol: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct TypeNodeLinks {
    pub resolved_type: Option<Arc<Type>>,
    pub outer_type_parameters: Vec<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct EnumMemberLinks {
    pub value: evaluator::EvalResult,
}

#[derive(Debug, Default)]
pub struct AssertionLinks {
    pub expr_type: Option<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct SourceFileLinks {
    pub type_checked: bool,
    pub unused_checked: bool,
    pub external_helpers_module: Option<Arc<Symbol>>,
    pub requested_external_emit_helpers: ExternalEmitHelpers,
    pub local_jsx_namespace: String,
    pub local_jsx_fragment_namespace: String,
    pub local_jsx_factory: Option<Arc<Node>>,
    pub local_jsx_fragment_factory: Option<Arc<Node>>,
    pub jsx_fragment_type: Option<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct SignatureLinks {
    pub resolved_signature: Option<Arc<Signature>>,
    pub effects_signature: Option<Arc<Signature>>,
    pub decorator_signature: Option<Arc<Signature>>,
}

#[derive(Debug, Default)]
pub struct JsxElementLinks {
    pub attributes_type: Option<Arc<Type>>,
    pub tag_name: Option<Arc<Symbol>>,
}

#[derive(Debug, Clone)]
pub struct AccessibleChainCacheKey {
    pub use_only_external_aliasing: bool,
    pub location: Option<Arc<Node>>,
    pub meaning: SymbolFlags,
}

impl PartialEq for AccessibleChainCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.use_only_external_aliasing == other.use_only_external_aliasing
            && self.meaning == other.meaning
            && match (&self.location, &other.location) {
                (None, None) => true,
                (Some(a), Some(b)) => a.id() == b.id(),
                _ => false,
            }
    }
}

impl Eq for AccessibleChainCacheKey {}

impl std::hash::Hash for AccessibleChainCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.use_only_external_aliasing.hash(state);
        self.meaning.bits().hash(state);
        match &self.location {
            Some(n) => n.id().hash(state),
            None => 0u64.hash(state),
        }
    }
}

#[derive(Debug, Default)]
pub struct ContainingSymbolLinks {
    pub extended_containers_by_file: HashMap<u64, Vec<Arc<Symbol>>>,

    pub extended_containers: Option<Vec<Arc<Symbol>>>,

    pub accessible_chain_cache: HashMap<AccessibleChainCacheKey, Vec<Arc<Symbol>>>,
}

#[derive(Debug, Default)]
pub struct DeclarationLinks {
    pub is_visible: Tristate,
}

#[derive(Debug, Default)]
pub struct DeclarationFileLinks {
    pub aliases_marked: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolAccessibilityResult {
    pub accessibility: SymbolAccessibility,

    pub aliases_to_make_visible: Vec<Arc<crate::ast::Node>>,
    pub error_symbol_name: String,
    pub error_module_name: String,
    pub error_node: Option<Arc<crate::ast::Node>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymbolAccessibility {
    #[default]
    Accessible,
    NotAccessible,
    CannotBeNamed,
    NotResolved,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CheckMode: u32 {
        const Normal               = 0;
        const Contextual           = 1 << 0;
        const Inferential          = 1 << 1;
        const SkipContextSensitive = 1 << 2;
        const SkipGenericFunctions = 1 << 3;
        const IsForSignatureHelp   = 1 << 4;
        const RestBindingElement   = 1 << 5;
        const TypeOnly             = 1 << 6;
        const ForceTuple           = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WideningKind {
    #[default]
    Normal,
    FunctionReturn,
    GeneratorNext,
    GeneratorYield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum TypeSystemPropertyName {
    #[default]
    Type,
    ResolvedBaseConstructorType,
    DeclaredType,
    ResolvedReturnType,
    ResolvedBaseConstraint,
    ResolvedTypeArguments,
    ResolvedBaseTypes,
    WriteType,
    InitializerIsUndefined,
    AliasTarget,
}
