//! Symbol and flow types for semantic analysis.
//!
//! Ported from `internal/ast/symbol.go`, `internal/ast/symbolflags.go`,
//! `internal/ast/checkflags.go`, and `internal/ast/flow.go`.
//!
//! In the Go implementation, symbols and flow nodes are stored directly on
//! AST nodes via the `nodeData` interface (`DeclarationBase`, `FlowNodeBase`,
//! `LocalsContainerBase`). In Rust, we use side tables keyed by node ID
//! because our `Node` struct is immutable (`Arc<Node>`).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::node::Node;

// ────────────────────────────────────────────────────────────────────────────
// SymbolFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Flags describing what kind of entity a `Symbol` represents.
    ///
    /// Mirrors `ast.SymbolFlags` in Go.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    #[allow(non_camel_case_types)]
    pub struct SymbolFlags: u32 {
        const None                   = 0;
        const FunctionScopedVariable = 1 << 0;
        const BlockScopedVariable    = 1 << 1;
        const Property               = 1 << 2;
        const EnumMember             = 1 << 3;
        const Function               = 1 << 4;
        const Class                  = 1 << 5;
        const Interface              = 1 << 6;
        const ConstEnum              = 1 << 7;
        const RegularEnum            = 1 << 8;
        const ValueModule            = 1 << 9;
        const NamespaceModule        = 1 << 10;
        const TypeLiteral            = 1 << 11;
        const ObjectLiteral          = 1 << 12;
        const Method                 = 1 << 13;
        const Constructor            = 1 << 14;
        const GetAccessor            = 1 << 15;
        const SetAccessor            = 1 << 16;
        const Signature              = 1 << 17;
        const TypeParameter          = 1 << 18;
        const TypeAlias              = 1 << 19;
        const ExportValue            = 1 << 20;
        const Alias                  = 1 << 21;
        const Prototype              = 1 << 22;
        const ExportStar             = 1 << 23;
        const Optional               = 1 << 24;
        const Transient              = 1 << 25;
        const Assignment             = 1 << 26;
        const ModuleExports          = 1 << 27;
        const ConstEnumOnlyModule    = 1 << 28;
        const ReplaceableByMethod    = 1 << 29;
        const GlobalLookup           = 1 << 30;
    }
}

#[allow(non_upper_case_globals)]
impl SymbolFlags {
    // Composite flag groups (matching Go constants)
    pub const ENUM: Self = Self::RegularEnum.union(Self::ConstEnum);
    pub const VARIABLE: Self = Self::FunctionScopedVariable.union(Self::BlockScopedVariable);
    pub const VALUE: Self = Self::VARIABLE
        .union(Self::Property)
        .union(Self::EnumMember)
        .union(Self::ObjectLiteral)
        .union(Self::Function)
        .union(Self::Class)
        .union(Self::ENUM)
        .union(Self::ValueModule)
        .union(Self::Method)
        .union(Self::GetAccessor)
        .union(Self::SetAccessor);
    pub const TYPE: Self = Self::Class
        .union(Self::Interface)
        .union(Self::ENUM)
        .union(Self::EnumMember)
        .union(Self::TypeLiteral)
        .union(Self::TypeParameter)
        .union(Self::TypeAlias);
    pub const NAMESPACE: Self = Self::ValueModule
        .union(Self::NamespaceModule)
        .union(Self::ENUM);
    pub const MODULE: Self = Self::ValueModule.union(Self::NamespaceModule);
    pub const ACCESSOR: Self = Self::GetAccessor.union(Self::SetAccessor);
    pub const BLOCK_SCOPED: Self = Self::BlockScopedVariable
        .union(Self::Class)
        .union(Self::ENUM);
    pub const PROPERTY_OR_ACCESSOR: Self = Self::Property.union(Self::ACCESSOR);
    pub const CLASS_MEMBER: Self = Self::Method.union(Self::ACCESSOR).union(Self::Property);
    pub const MODULE_MEMBER: Self = Self::VARIABLE
        .union(Self::Function)
        .union(Self::Class)
        .union(Self::Interface)
        .union(Self::ENUM)
        .union(Self::MODULE)
        .union(Self::TypeAlias)
        .union(Self::Alias);
    pub const EXPORT_HAS_LOCAL: Self = Self::Function
        .union(Self::Class)
        .union(Self::ENUM)
        .union(Self::ValueModule);

    // Excludes flags — which flags cannot merge with a symbol of a given kind.
    // Ported from `internal/ast/symbolflags.go`.
    pub const FunctionScopedVariableExcludes: Self =
        Self::VALUE.difference(Self::FunctionScopedVariable);
    pub const BlockScopedVariableExcludes: Self = Self::VALUE;
    pub const ParameterExcludes: Self = Self::VALUE;
    pub const PropertyExcludes: Self = Self::VALUE.difference(Self::Property.union(Self::ACCESSOR));
    pub const EnumMemberExcludes: Self = Self::VALUE.union(Self::TYPE);
    pub const FunctionExcludes: Self =
        Self::VALUE.difference(Self::Function.union(Self::ValueModule).union(Self::Class));
    pub const ClassExcludes: Self = (Self::VALUE.union(Self::TYPE)).difference(
        Self::ValueModule
            .union(Self::Interface)
            .union(Self::Function),
    );
    pub const InterfaceExcludes: Self = Self::TYPE.difference(Self::Interface.union(Self::Class));
    pub const RegularEnumExcludes: Self =
        (Self::VALUE.union(Self::TYPE)).difference(Self::RegularEnum.union(Self::ValueModule));
    pub const ConstEnumExcludes: Self = (Self::VALUE.union(Self::TYPE)).difference(Self::ConstEnum);
    pub const ValueModuleExcludes: Self = Self::VALUE.difference(
        Self::Function
            .union(Self::Class)
            .union(Self::RegularEnum)
            .union(Self::ValueModule),
    );
    pub const NamespaceModuleExcludes: Self = Self::None;
    pub const MethodExcludes: Self = Self::VALUE.difference(Self::Method);
    pub const GetAccessorExcludes: Self =
        Self::VALUE.difference(Self::SetAccessor.union(Self::Property));
    pub const SetAccessorExcludes: Self =
        Self::VALUE.difference(Self::GetAccessor.union(Self::Property));
    pub const AccessorExcludes: Self = Self::VALUE.difference(Self::Property);
    pub const TypeParameterExcludes: Self = Self::TYPE.difference(Self::TypeParameter);
    pub const TypeAliasExcludes: Self = Self::TYPE;
    pub const AliasExcludes: Self = Self::Alias;
}

// ────────────────────────────────────────────────────────────────────────────
// CheckFlags
// ────────────────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    /// Flags set by the checker on transient symbols.
    ///
    /// Mirrors `ast.CheckFlags` in Go.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct CheckFlags: u32 {
        const None                   = 0;
        const Instantiated           = 1 << 0;
        const SyntheticProperty      = 1 << 1;
        const SyntheticMethod        = 1 << 2;
        const Readonly               = 1 << 3;
        const ReadPartial            = 1 << 4;
        const WritePartial           = 1 << 5;
        const HasNonUniformType      = 1 << 6;
        const HasLiteralType         = 1 << 7;
        const ContainsPublic         = 1 << 8;
        const ContainsProtected      = 1 << 9;
        const ContainsPrivate        = 1 << 10;
        const ContainsStatic         = 1 << 11;
        const Late                   = 1 << 12;
        const ReverseMapped          = 1 << 13;
        const OptionalParameter      = 1 << 14;
        const RestParameter          = 1 << 15;
        const DeferredType           = 1 << 16;
        const HasNeverType           = 1 << 17;
        const Mapped                 = 1 << 18;
        const StripOptional          = 1 << 19;
        const Unresolved             = 1 << 20;
        const IsDiscriminantComputed = 1 << 21;
        const IsDiscriminant         = 1 << 22;
        const IndexSymbol            = 1 << 23;
    }
}

impl CheckFlags {
    pub const SYNTHETIC: Self = Self::SyntheticProperty.union(Self::SyntheticMethod);
}

// ────────────────────────────────────────────────────────────────────────────
// Symbol
// ────────────────────────────────────────────────────────────────────────────

/// A symbol represents a named entity in the program: a variable, function,
/// class, interface, etc.
///
/// Mirrors `ast.Symbol` in Go.
#[derive(Debug)]
pub struct Symbol {
    pub flags: SymbolFlags,
    pub check_flags: CheckFlags,
    pub name: String,
    pub declarations: Vec<Arc<Node>>,
    pub value_declaration: Option<Arc<Node>>,
    pub members: SymbolTable,
    pub exports: SymbolTable,
    pub parent: Option<Arc<Symbol>>,
    pub export_symbol: Option<Arc<Symbol>>,
    id: AtomicU64,
}

impl Symbol {
    pub fn new(flags: SymbolFlags, name: impl Into<String>) -> Self {
        Self {
            flags,
            check_flags: CheckFlags::None,
            name: name.into(),
            declarations: Vec::new(),
            value_declaration: None,
            members: SymbolTable::default(),
            exports: SymbolTable::default(),
            parent: None,
            export_symbol: None,
            id: AtomicU64::new(0),
        }
    }

    /// A unique numeric ID for this symbol (lazily assigned).
    pub fn id(&self) -> u64 {
        let mut id = self.id.load(Ordering::Relaxed);
        if id == 0 {
            id = NEXT_SYMBOL_ID.fetch_add(1, Ordering::Relaxed);
            self.id.store(id, Ordering::Relaxed);
        }
        id
    }

    /// Whether this symbol represents an external module.
    pub fn is_external_module(&self) -> bool {
        self.flags.contains(SymbolFlags::ValueModule) && self.name.starts_with('"')
    }

    /// Whether this symbol's value declaration has the `static` modifier.
    pub fn is_static(&self) -> bool {
        // TODO: implement modifier flags extraction from the node
        false
    }

    /// Combined flags of this symbol and its export symbol.
    pub fn combined_local_and_export_symbol_flags(&self) -> SymbolFlags {
        if let Some(export) = &self.export_symbol {
            self.flags | export.flags
        } else {
            self.flags
        }
    }
}

static NEXT_SYMBOL_ID: AtomicU64 = AtomicU64::new(1);

// ────────────────────────────────────────────────────────────────────────────
// SymbolTable
// ────────────────────────────────────────────────────────────────────────────

/// A map from name to symbol.
///
/// Mirrors `ast.SymbolTable` in Go.
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    pub entries: HashMap<String, Arc<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&Arc<Symbol>> {
        self.entries.get(name)
    }

    pub fn insert(&mut self, name: impl Into<String>, symbol: Arc<Symbol>) {
        self.entries.insert(name.into(), symbol);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Arc<Symbol>> {
        self.entries.iter()
    }
}

/// Internal symbol name prefix (invalid UTF-8 sentinel in Go, we use a
/// unlikely-to-collide prefix).
pub const INTERNAL_SYMBOL_NAME_PREFIX: &str = "\u{FE}";

pub const INTERNAL_SYMBOL_NAME_CALL: &str = "\u{FE}call";
pub const INTERNAL_SYMBOL_NAME_CONSTRUCTOR: &str = "\u{FE}constructor";
pub const INTERNAL_SYMBOL_NAME_NEW: &str = "\u{FE}new";
pub const INTERNAL_SYMBOL_NAME_INDEX: &str = "\u{FE}index";
pub const INTERNAL_SYMBOL_NAME_EXPORT_STAR: &str = "\u{FE}export";
pub const INTERNAL_SYMBOL_NAME_GLOBAL: &str = "\u{FE}global";
pub const INTERNAL_SYMBOL_NAME_MISSING: &str = "\u{FE}missing";
pub const INTERNAL_SYMBOL_NAME_TYPE: &str = "\u{FE}type";
pub const INTERNAL_SYMBOL_NAME_OBJECT: &str = "\u{FE}object";
pub const INTERNAL_SYMBOL_NAME_JSX_ATTRIBUTES: &str = "\u{FE}jsxAttributes";
pub const INTERNAL_SYMBOL_NAME_CLASS: &str = "\u{FE}class";
pub const INTERNAL_SYMBOL_NAME_FUNCTION: &str = "\u{FE}function";
pub const INTERNAL_SYMBOL_NAME_COMPUTED: &str = "\u{FE}computed";
pub const INTERNAL_SYMBOL_NAME_ASSIGNMENT: &str = "\u{FE}assignment";
pub const INTERNAL_SYMBOL_NAME_INSTANTIATION_EXPRESSION: &str = "\u{FE}instantiationExpression";
pub const INTERNAL_SYMBOL_NAME_IMPORT_ATTRIBUTES: &str = "\u{FE}importAttributes";
pub const INTERNAL_SYMBOL_NAME_EXPORT_EQUALS: &str = "export=";
pub const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default";
pub const INTERNAL_SYMBOL_NAME_THIS: &str = "this";
pub const INTERNAL_SYMBOL_NAME_MODULE_EXPORTS: &str = "module.exports";

// ────────────────────────────────────────────────────────────────────────────
// FlowNode and FlowFlags
// ────────────────────────────────────────────────────────────────────────────

/// Flags describing the kind of control flow node.
///
/// Mirrors `ast.FlowFlags` in Go.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FlowFlags(u32);

impl FlowFlags {
    pub const UNREACHABLE: Self = Self(1 << 0);
    pub const START: Self = Self(1 << 1);
    pub const BRANCH_LABEL: Self = Self(1 << 2);
    pub const LOOP_LABEL: Self = Self(1 << 3);
    pub const ASSIGNMENT: Self = Self(1 << 4);
    pub const TRUE_CONDITION: Self = Self(1 << 5);
    pub const FALSE_CONDITION: Self = Self(1 << 6);
    pub const SWITCH_CLAUSE: Self = Self(1 << 7);
    pub const ARRAY_MUTATION: Self = Self(1 << 8);
    pub const CALL: Self = Self(1 << 9);
    pub const REDUCE_LABEL: Self = Self(1 << 10);
    pub const REFERENCED: Self = Self(1 << 11);
    pub const SHARED: Self = Self(1 << 12);

    pub const LABEL: Self = Self(Self::BRANCH_LABEL.0 | Self::LOOP_LABEL.0);
    pub const CONDITION: Self = Self(Self::TRUE_CONDITION.0 | Self::FALSE_CONDITION.0);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns true if any bit of `other` is set in `self`.
    /// Mirrors Go's `flags&mask != 0` idiom.
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for FlowFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// A node in the control flow graph.
///
/// Mirrors `ast.FlowNode` in Go.
#[derive(Debug)]
pub struct FlowNode {
    pub flags: FlowFlags,
    pub node: Option<Arc<Node>>,
    pub antecedent: Option<Arc<FlowNode>>,
    pub antecedents: Vec<Arc<FlowNode>>,
    /// Auxiliary node for SWITCH_CLAUSE flows: stores the enclosing
    /// `SwitchStatement` so the checker can resolve the discriminant
    /// expression. `None` for all other flow kinds. Mirrors the
    /// `FlowSwitchClauseData.SwitchStatement` field in Go.
    pub switch_statement: Option<Arc<Node>>,
    /// For SWITCH_CLAUSE flows: the half-open clause-group range
    /// `[start, end)` this flow node narrows by (Go
    /// `FlowSwitchClauseData.ClauseStart/ClauseEnd`). A group is a run of
    /// statement-less clauses followed by the clause that owns the
    /// statements; `[0, 0)` marks the implicit bypass branch of a
    /// default-less switch (no case matched). `None` for all other flow
    /// kinds.
    pub clause_range: Option<(usize, usize)>,
    /// For REDUCE_LABEL flows: the branch label whose antecedent set is
    /// reduced to `antecedents` while the walk is inside this reduce label
    /// (Go `FlowReduceLabelData.Target`). `None` for all other flow kinds.
    pub reduce_target: Option<Arc<FlowNode>>,
}

impl FlowNode {
    pub fn new(flags: FlowFlags) -> Self {
        Self {
            flags,
            node: None,
            antecedent: None,
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        }
    }
}

/// A flow label is a flow node that serves as a junction point.
pub type FlowLabel = FlowNode;

// ────────────────────────────────────────────────────────────────────────────
// NodeSymbolMap — side table mapping nodes to symbols and flow data
// ────────────────────────────────────────────────────────────────────────────

/// Side table that maps node IDs to symbols, locals, and flow nodes.
///
/// In Go, these are stored directly on AST nodes via the `nodeData` interface.
/// In Rust, we use side tables because `Node` is immutable (`Arc<Node>`).
#[derive(Debug, Default)]
pub struct NodeSymbolMap {
    /// Maps declaration nodes to their symbols.
    pub symbols: HashMap<u64, Arc<Symbol>>,
    /// Maps container nodes to their local symbol tables.
    pub locals: HashMap<u64, SymbolTable>,
    /// Maps expression nodes to their flow nodes.
    pub flow_nodes: HashMap<u64, Arc<FlowNode>>,
    /// Diagnostics recorded by the binder (e.g. TS2451 block-scoped
    /// redeclarations). Surfaced through the program's semantic
    /// diagnostics.
    pub binder_diagnostics: Vec<super::diagnostic::Diagnostic>,
}

impl NodeSymbolMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the symbol for a node, if any.
    pub fn symbol_of(&self, node: &Node) -> Option<&Arc<Symbol>> {
        self.symbols.get(&node.id())
    }

    /// Get the locals (symbol table) for a container node, if any.
    pub fn locals_of(&self, node: &Node) -> Option<&SymbolTable> {
        self.locals.get(&node.id())
    }

    /// Get the flow node for an expression node, if any.
    pub fn flow_node_of(&self, node: &Node) -> Option<&Arc<FlowNode>> {
        self.flow_nodes.get(&node.id())
    }

    /// Set the symbol for a node.
    pub fn set_symbol(&mut self, node: &Node, symbol: Arc<Symbol>) {
        self.symbols.insert(node.id(), symbol);
    }

    /// Set the locals for a container node.
    pub fn set_locals(&mut self, node: &Node, locals: SymbolTable) {
        self.locals.insert(node.id(), locals);
    }

    /// Set the flow node for an expression node.
    pub fn set_flow_node(&mut self, node: &Node, flow: Arc<FlowNode>) {
        self.flow_nodes.insert(node.id(), flow);
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ContainerFlags
// ────────────────────────────────────────────────────────────────────────────

/// Flags describing the container properties of a node.
///
/// Mirrors `binder.ContainerFlags` in Go.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ContainerFlags(u32);

impl ContainerFlags {
    pub const NONE: Self = Self(0);
    pub const IS_CONTAINER: Self = Self(1 << 0);
    pub const IS_BLOCK_SCOPED_CONTAINER: Self = Self(1 << 1);
    pub const IS_CONTROL_FLOW_CONTAINER: Self = Self(1 << 2);
    pub const IS_FUNCTION_LIKE: Self = Self(1 << 3);
    pub const IS_FUNCTION_EXPRESSION: Self = Self(1 << 4);
    pub const HAS_LOCALS: Self = Self(1 << 5);
    pub const IS_INTERFACE: Self = Self(1 << 6);
    pub const IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR: Self = Self(1 << 7);
    pub const IS_THIS_CONTAINER: Self = Self(1 << 8);
    pub const PROPAGATES_THIS_KEYWORD: Self = Self(1 << 9);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for ContainerFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_flags_composites() {
        let flags = SymbolFlags::Function.union(SymbolFlags::Class);
        assert!(flags.contains(SymbolFlags::Function));
        assert!(flags.contains(SymbolFlags::Class));
        assert!(!flags.contains(SymbolFlags::Interface));
    }

    #[test]
    fn symbol_creation() {
        let sym = Symbol::new(SymbolFlags::Function, "foo");
        assert_eq!(sym.name, "foo");
        assert!(sym.flags.contains(SymbolFlags::Function));
        assert_eq!(sym.id(), sym.id()); // ID is stable
    }

    #[test]
    fn symbol_table_operations() {
        let mut table = SymbolTable::new();
        let sym = Arc::new(Symbol::new(SymbolFlags::VARIABLE, "x"));
        table.insert("x", sym);
        assert!(table.get("x").is_some());
        assert!(table.get("y").is_none());
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn flow_flags() {
        let flags = FlowFlags::START | FlowFlags::ASSIGNMENT;
        assert!(flags.contains(FlowFlags::START));
        assert!(flags.contains(FlowFlags::ASSIGNMENT));
        assert!(!flags.contains(FlowFlags::CALL));
    }

    #[test]
    fn node_symbol_map() {
        let node = Arc::new(Node::new(
            crate::ast::SyntaxKind::Identifier,
            crate::ast::NodeData::Identifier(crate::ast::IdentifierData {
                text: "x".to_string(),
            }),
        ));
        let mut map = NodeSymbolMap::new();
        let sym = Arc::new(Symbol::new(SymbolFlags::VARIABLE, "x"));
        map.set_symbol(&node, Arc::clone(&sym));
        assert!(map.symbol_of(&node).is_some());
        assert_eq!(map.symbol_of(&node).unwrap().name, "x");
    }

    #[test]
    fn container_flags() {
        let flags = ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS;
        assert!(flags.contains(ContainerFlags::IS_CONTAINER));
        assert!(flags.contains(ContainerFlags::HAS_LOCALS));
        assert!(!flags.contains(ContainerFlags::IS_INTERFACE));
    }
}
