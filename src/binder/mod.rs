//! Symbol binding, ported from `internal/binder/binder.go`.
//!
//! The binder walks the AST and creates symbols for declarations, builds
//! scopes (symbol tables), and associates identifiers with their declarations.
//! It also builds the control flow graph for use by the checker.
//!
//! In Go, symbols and flow nodes are stored directly on AST nodes. In Rust,
//! we use side tables (`NodeSymbolMap`) keyed by node ID.

use crate::ast::*;
use std::sync::Arc;

/// The binder.
///
/// Mirrors `binder.Binder` in Go.
pub struct Binder {
    /// Side table mapping nodes to symbols, locals, and flow nodes.
    pub symbol_map: NodeSymbolMap,
    /// The current container node (where members/exports go).
    container: Option<Arc<Node>>,
    /// The current block-scoped container (where block-scoped locals go).
    block_scope_container: Option<Arc<Node>>,
    /// The current container's parent symbol.
    parent_symbol: Option<Arc<Symbol>>,
    /// The current flow node.
    current_flow: Option<Arc<FlowNode>>,
    /// Symbol count (for diagnostics/stats).
    symbol_count: usize,
    /// Unreachable flow node.
    unreachable_flow: Option<Arc<FlowNode>>,
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

impl Binder {
    /// Create a new binder.
    pub fn new() -> Self {
        Self {
            symbol_map: NodeSymbolMap::new(),
            container: None,
            block_scope_container: None,
            parent_symbol: None,
            current_flow: None,
            symbol_count: 0,
            unreachable_flow: None,
        }
    }

    /// Bind a source file: walk the AST and create symbols.
    ///
    /// Mirrors `binder.BindSourceFile` in Go.
    pub fn bind_source_file(&mut self, file: &SourceFile) -> &NodeSymbolMap {
        let start_flow = Arc::new(FlowNode::new(FlowFlags::START));
        self.current_flow = Some(Arc::clone(&start_flow));
        self.unreachable_flow = Some(Arc::new(FlowNode::new(FlowFlags::UNREACHABLE)));

        // Create a symbol for the source file itself
        let file_symbol = Arc::new(Symbol::new(
            SymbolFlags::ValueModule,
            file.file_name.clone(),
        ));
        self.symbol_map
            .set_symbol(&file.node, Arc::clone(&file_symbol));
        self.symbol_count += 1;

        // Set up container context
        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();

        self.container = Some(Arc::clone(&file.node));
        self.block_scope_container = Some(Arc::clone(&file.node));
        self.parent_symbol = Some(file_symbol);

        // Bind children
        self.bind_children(&file.node);

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        &self.symbol_map
    }

    /// Create a new symbol.
    fn new_symbol(&mut self, flags: SymbolFlags, name: impl Into<String>) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(flags, name))
    }

    /// Declare a symbol for a node, adding it to the appropriate symbol table.
    ///
    /// Mirrors `binder.declareSymbol` in Go.
    fn declare_symbol(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);
        let symbol = self.new_symbol(includes, name.clone());

        // Add the declaration to the symbol
        // (symbol.declarations is not mutable through Arc, so we'd need interior
        // mutability — for now, the symbol_map tracks the primary symbol)

        // Add to appropriate symbol table
        // 1) container's exports (if in a module/namespace)
        // 2) container's members (if in a class/interface/object)
        // 3) block-scope container's locals
        if let Some(_container) = &self.container {
            if let Some(parent_sym) = &self.parent_symbol {
                // For now, add to parent symbol's members
                // In a full implementation, this would distinguish between
                // members, exports, and locals based on container flags
                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_sym_mut)
                        .members
                        .insert(name.clone(), Arc::clone(&symbol));
                }
            } else if let Some(block_container) = &self.block_scope_container {
                // Add to locals of the block-scoped container
                let container_id = block_container.id();
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container_id)
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        // Associate the symbol with the node
        self.symbol_map.set_symbol(node, Arc::clone(&symbol));

        // Set the value declaration if this is a value declaration
        // (in the full Go implementation, this is more nuanced)

        symbol
    }

    /// Get the name of a declaration node.
    fn get_declaration_name(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::VariableDeclaration(data) => self.node_text(&data.name),
            NodeData::VariableStatement(_) => String::new(),
            NodeData::FunctionDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::FunctionExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string()),
            NodeData::ArrowFunction(_) => INTERNAL_SYMBOL_NAME_FUNCTION.to_string(),
            NodeData::ClassDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::ClassExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_CLASS.to_string()),
            NodeData::InterfaceDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeAliasDeclaration(data) => self.node_text(&data.name),
            NodeData::EnumDeclaration(data) => self.node_text(&data.name),
            NodeData::ModuleDeclaration(data) => self.node_text(&data.name),
            NodeData::ParameterDeclaration(data) => self.node_text(&data.name),
            NodeData::BindingElement(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::ImportSpecifier(data) => data
                .property_name
                .as_ref()
                .map_or_else(|| self.node_text(&data.name), |n| self.node_text(n)),
            NodeData::ImportClause(data) => data.name.as_ref().map_or_else(
                || {
                    data.named_bindings
                        .as_ref()
                        .map_or_else(|| String::new(), |n| self.node_text(n))
                },
                |n| self.node_text(n),
            ),
            NodeData::PropertyDeclaration(data) => self.node_text(&data.name),
            NodeData::MethodDeclaration(data) => self.node_text(&data.name),
            NodeData::PropertyAssignment(data) => self.node_text(&data.name),
            NodeData::ShorthandPropertyAssignment(data) => self.node_text(&data.name),
            NodeData::EnumMember(data) => self.node_text(&data.name),
            NodeData::GetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::SetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::Identifier(data) => data.text.clone(),
            _ => String::new(),
        }
    }

    /// Get the text of a node (for name extraction).
    fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(data) => data.text.clone(),
            NodeData::StringLiteral(data) => data.text.clone(),
            NodeData::NumericLiteral(data) => data.text.clone(),
            NodeData::NoSubstitutionTemplateLiteral(data) => data.text.clone(),
            NodeData::BigIntLiteral(data) => data.text.clone(),
            _ => String::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Binding dispatch
    // ─────────────────────────────────────────────────────────────────────

    /// Bind a single node: create symbols, set flow nodes, then recurse.
    fn bind(&mut self, node: &Arc<Node>) {
        // Set flow node for expressions
        match node.kind {
            SyntaxKind::Identifier => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            _ => {}
        }

        // Create symbols for declarations
        match node.kind {
            SyntaxKind::VariableDeclaration => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::VariableStatement => {
                // The statement itself doesn't get a symbol; its declarations do
            }
            SyntaxKind::FunctionDeclaration => {
                self.declare_symbol(node, SymbolFlags::Function, SymbolFlags::VALUE);
            }
            SyntaxKind::FunctionExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ArrowFunction => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ClassDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::Class,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ClassExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Class,
                    INTERNAL_SYMBOL_NAME_CLASS,
                );
            }
            SyntaxKind::InterfaceDeclaration => {
                self.declare_symbol(node, SymbolFlags::Interface, SymbolFlags::TYPE);
            }
            SyntaxKind::TypeAliasDeclaration => {
                self.declare_symbol(node, SymbolFlags::TypeAlias, SymbolFlags::TYPE);
            }
            SyntaxKind::EnumDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::RegularEnum,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ModuleDeclaration => {
                self.declare_symbol(node, SymbolFlags::ValueModule, SymbolFlags::MODULE);
            }
            SyntaxKind::Parameter => {
                self.declare_symbol(
                    node,
                    SymbolFlags::FunctionScopedVariable,
                    SymbolFlags::VALUE,
                );
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                self.declare_symbol(node, SymbolFlags::Method, SymbolFlags::VALUE);
            }
            SyntaxKind::PropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::EnumMember => {
                self.declare_symbol(
                    node,
                    SymbolFlags::EnumMember,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::GetAccessor => {
                self.declare_symbol(node, SymbolFlags::GetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::SetAccessor => {
                self.declare_symbol(node, SymbolFlags::SetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier => {
                self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
            }
            SyntaxKind::BindingElement => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ObjectLiteral,
                    INTERNAL_SYMBOL_NAME_OBJECT,
                );
            }
            SyntaxKind::TypeLiteral => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::TypeLiteral,
                    INTERNAL_SYMBOL_NAME_TYPE,
                );
            }
            _ => {}
        }

        // Recurse into children
        let container_flags = get_container_flags(node.kind);
        if container_flags != ContainerFlags::NONE {
            self.bind_container(node, container_flags);
        } else {
            self.bind_children(node);
        }
    }

    /// Create an anonymous symbol (for function expressions, class expressions,
    /// object literals, type literals).
    fn bind_anonymous_declaration(&mut self, node: &Arc<Node>, flags: SymbolFlags, name: &str) {
        let symbol = self.new_symbol(flags, name.to_string());
        self.symbol_map.set_symbol(node, symbol);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Container binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind a container node: save/restore container context, then bind children.
    fn bind_container(&mut self, node: &Arc<Node>, _flags: ContainerFlags) {
        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        // Save the current parent_symbol. For container nodes that have a
        // symbol (e.g. FunctionDeclaration), we'll replace it with the
        // container's symbol so children are added to its members. For
        // block-scoped containers without a symbol (e.g. Block), we clear
        // it so children go into the block's locals.
        let prev_parent_symbol = self.parent_symbol.take();

        self.container = Some(Arc::clone(node));

        // Block-scoped containers get a new locals scope
        if is_block_scoped_container(node.kind) {
            self.block_scope_container = Some(Arc::clone(node));
        }

        // Create locals for this container if it has them
        if has_locals(node.kind) {
            self.symbol_map.locals.insert(node.id(), SymbolTable::new());
        }

        // Set parent_symbol to the container's symbol (if it has one).
        // This ensures children (parameters, class members, etc.) are added
        // to the container's symbol members rather than the outer scope.
        if let Some(sym) = self.symbol_map.symbol_of(node) {
            self.parent_symbol = Some(Arc::clone(sym));
        }
        // If the node has no symbol (e.g. Block), parent_symbol remains None,
        // so declare_symbol falls through to the block_scope_container.locals.

        self.bind_children(node);

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent_symbol;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Child binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind all children of a node.
    fn bind_children(&mut self, node: &Arc<Node>) {
        // Use a raw pointer to work around the borrow checker: `bind` needs
        // `&mut self` but `for_each_child` gives us shared references to children.
        // This is safe because we don't alias the node itself.
        let this = self as *mut Self;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            unsafe {
                (*this).bind(child);
            }
            false
        });
    }

    /// Get the number of symbols created.
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }
}

/// Get container flags for a node kind.
fn get_container_flags(kind: SyntaxKind) -> ContainerFlags {
    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::JsxAttributes => ContainerFlags::IS_CONTAINER,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::IS_FUNCTION_EXPRESSION
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::IndexSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::Block | SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_BLOCK_SCOPED_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::CatchClause
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement => {
            ContainerFlags::IS_BLOCK_SCOPED_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        _ => ContainerFlags::NONE,
    }
}

/// Whether a node kind is a block-scoped container.
fn is_block_scoped_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
    )
}

/// Whether a node kind has locals (a local symbol table).
fn has_locals(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
    )
}

/// Bind a source file using a fresh binder.
pub fn bind_source_file(file: &SourceFile) -> NodeSymbolMap {
    let mut binder = Binder::new();
    binder.bind_source_file(file);
    std::mem::take(&mut binder.symbol_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_and_bind(source: &str) -> (SourceFile, NodeSymbolMap) {
        let source_file = Parser::parse_source_file_text("test.ts", source.to_string());
        let symbol_map = bind_source_file(&source_file);
        (source_file, symbol_map)
    }

    #[test]
    fn bind_variable_declaration() {
        let (file, map) = parse_and_bind("var x = 1;");
        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };
        assert!(!statements.nodes.is_empty());
        // The variable statement contains a declaration list with declarations
        let var_stmt = &statements.nodes[0];
        assert_eq!(var_stmt.kind, SyntaxKind::VariableStatement);
        // Symbol count should be > 0 (file symbol + variable symbol)
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
        let _ = map;
    }

    #[test]
    fn bind_function_declaration() {
        let (file, _map) = parse_and_bind("function foo() { return 42; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn bind_class_declaration() {
        let (file, _map) = parse_and_bind("class Foo { bar() {} }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3); // file + class + method
    }

    #[test]
    fn bind_interface_declaration() {
        let (file, _map) = parse_and_bind("interface Foo { bar: number; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3); // file + interface + property
    }

    #[test]
    fn bind_import_declaration() {
        let (file, _map) = parse_and_bind("import { foo } from 'mod';");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        // Import is not yet parsed by our parser, but binding shouldn't crash
        let _ = binder.symbol_count();
    }

    #[test]
    fn bind_multiple_declarations() {
        let (file, _map) = parse_and_bind("let x = 1; let y = 2; let z = 3;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 4); // file + 3 variables
    }

    #[test]
    fn bind_nested_scope() {
        let (file, _map) = parse_and_bind("function foo() { let x = 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        // file + function + variable
        assert!(binder.symbol_count() >= 3);
    }
}
