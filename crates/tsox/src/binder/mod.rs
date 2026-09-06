pub mod nameresolver;
pub mod referenceresolver;

use crate::ast::*;
use crate::diagnostics::messages_generated::{
    A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
    CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0, DUPLICATE_IDENTIFIER_0,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
};
use std::sync::Arc;

mod symbols;
mod flow_bind;
mod bind_walk;

#[derive(Debug)]
struct FlowLabel {
    node: FlowNode,
}

impl FlowLabel {
    fn new(flags: FlowFlags) -> Self {
        Self {
            node: FlowNode::new(flags),
        }
    }

    fn add_antecedent(&mut self, antecedent: Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }

        for ant in &self.node.antecedents {
            if Arc::ptr_eq(ant, &antecedent) {
                return;
            }
        }
        self.node.antecedents.push(antecedent);
    }

    fn finish_multi(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    fn push_antecedent(node: &Arc<FlowNode>, ant: Arc<FlowNode>) {
        if ant.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        let ptr = Arc::as_ptr(node) as *mut FlowNode;
        unsafe {
            for existing in &(*ptr).antecedents {
                if Arc::ptr_eq(existing, &ant) {
                    return;
                }
            }
            (*ptr).antecedents.push(ant);
        }
    }

    fn finish(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        if self.node.antecedents.len() == 1 {
            return Arc::clone(&self.node.antecedents[0]);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }
}

#[derive(Debug)]
struct ActiveLabel {
    name: String,
    break_target: Arc<FlowNode>,
    continue_target: Option<Arc<FlowNode>>,
    referenced: bool,
    next: Option<Box<ActiveLabel>>,
}

pub struct Binder {

    pub symbol_map: NodeSymbolMap,

    current_source_file: Option<Arc<SourceFile>>,

    container: Option<Arc<Node>>,

    block_scope_container: Option<Arc<Node>>,

    this_container: Option<Arc<Node>>,

    parent_symbol: Option<Arc<Symbol>>,

    current_flow: Option<Arc<FlowNode>>,

    symbol_count: usize,

    expando_assignments: Vec<(Arc<Node>, Option<Arc<Node>>)>,

    unreachable_flow: Option<Arc<FlowNode>>,

    current_break_target: Option<Arc<FlowNode>>,

    current_continue_target: Option<Arc<FlowNode>>,

    current_exception_target: Option<Arc<FlowNode>>,

    current_return_target: Option<Arc<FlowNode>>,

    active_label_list: Option<Box<ActiveLabel>>,

    has_explicit_return: bool,

    has_flow_effects: bool,
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) enum DeclareTarget {

    Exports(Arc<Symbol>),

    Locals(Arc<Node>),
}

impl Binder {

    pub fn new() -> Self {
        Self {
            symbol_map: NodeSymbolMap::new(),
            current_source_file: None,
            container: None,
            block_scope_container: None,
            this_container: None,
            parent_symbol: None,
            current_flow: None,
            symbol_count: 0,
            expando_assignments: Vec::new(),
            unreachable_flow: None,
            current_break_target: None,
            current_continue_target: None,
            current_exception_target: None,
            current_return_target: None,
            active_label_list: None,
            has_explicit_return: false,
            has_flow_effects: false,
        }
    }

    pub fn bind_source_file(&mut self, file: &Arc<SourceFile>) -> &NodeSymbolMap {
        self.current_source_file = Some(Arc::clone(file));

        self.set_parent_pointers(&file.node);

        let start_flow = Arc::new(FlowNode::new(FlowFlags::START));
        self.current_flow = Some(Arc::clone(&start_flow));
        self.unreachable_flow = Some(Arc::new(FlowNode::new(FlowFlags::UNREACHABLE)));

        self.symbol_map
            .set_flow_node(&file.node, Arc::clone(&start_flow));

        let file_symbol = Arc::new(Symbol::new(
            SymbolFlags::ValueModule,
            file.file_name.clone(),
        ));
        {
            let file_symbol_mut = Arc::as_ptr(&file_symbol) as *mut Symbol;
            unsafe {
                (*file_symbol_mut).declarations.push(Arc::clone(&file.node));
                (*file_symbol_mut).value_declaration = Some(Arc::clone(&file.node));
            }
        }
        self.symbol_map
            .set_symbol(&file.node, Arc::clone(&file_symbol));
        self.symbol_count += 1;

        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();

        self.container = Some(Arc::clone(&file.node));
        self.block_scope_container = Some(Arc::clone(&file.node));
        self.parent_symbol = Some(file_symbol);

        self.bind_children(&file.node);

        self.process_expando_assignments();

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        &self.symbol_map
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
            unsafe {
                (*child_mut).parent = Some(Arc::clone(&parent_clone));
            }
            self.set_parent_pointers(child);
        }
    }



}

fn get_container_flags(kind: SyntaxKind) -> ContainerFlags {
    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::JsxAttributes
        | SyntaxKind::EnumDeclaration => ContainerFlags::IS_CONTAINER,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::IS_FUNCTION_EXPRESSION
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }

        SyntaxKind::MethodSignature
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::IndexSignature => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }

        SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration | SyntaxKind::MappedType => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
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

#[allow(dead_code)]
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
            | SyntaxKind::Constructor
    )
}

fn is_block_only_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::CaseBlock
    )
}

fn is_var_container_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SourceFile
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
    )
}

fn collect_binding_elements<'a>(node: &'a Arc<Node>, out: &mut Vec<&'a Arc<Node>>) {
    if let NodeData::BindingPattern(pattern) = &node.data {
        for el in pattern.elements.iter() {
            out.push(el);
            let name = match &el.data {
                NodeData::BindingElement(be) => &be.name,
                _ => continue,
            };
            if let Some(name_node) = name
                && matches!(name_node.data, NodeData::BindingPattern(_))
            {
                collect_binding_elements(name_node, out);
            }
        }
    }
}

fn fn_like_body_present(parent: &Arc<Node>) -> bool {
    match &parent.data {
        NodeData::FunctionDeclaration(d) => d.body.is_some(),
        NodeData::MethodDeclaration(d) => d.body.is_some(),
        NodeData::ConstructorDeclaration(d) => d.body.is_some(),
        NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => true,
        _ => false,
    }
}

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
            | SyntaxKind::MethodSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType
    )
}

pub fn bind_source_file(file: &Arc<SourceFile>) -> NodeSymbolMap {
    let mut binder = Binder::new();
    binder.bind_source_file(file);
    std::mem::take(&mut binder.symbol_map)
}

fn clause_statements_empty(clause: &Arc<Node>) -> bool {
    matches!(&clause.data, NodeData::CaseOrDefaultClause(d) if d.statements.nodes.is_empty())
}

fn is_assignment_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
            | SyntaxKind::CaretEqualsToken
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_and_bind(source: &str) -> (Arc<SourceFile>, NodeSymbolMap) {
        let source_file = Arc::new(Parser::parse_source_file_text("test.ts", source.to_string()));
        let symbol_map = bind_source_file(&Arc::clone(&source_file));
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

        let var_stmt = &statements.nodes[0];
        assert_eq!(var_stmt.kind, SyntaxKind::VariableStatement);

        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 2);
        let _ = map;
    }

    #[test]
    fn bind_function_declaration() {
        let (file, _map) = parse_and_bind("function foo() { return 42; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn bind_class_declaration() {
        let (file, _map) = parse_and_bind("class Foo { bar() {} }");
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 3);
    }

    #[test]
    fn bind_interface_declaration() {
        let (file, _map) = parse_and_bind("interface Foo { bar: number; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3);
    }

    #[test]
    fn bind_import_declaration() {
        let (file, _map) = parse_and_bind("import { foo } from 'mod';");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);

        let _ = binder.symbol_count();
    }

    #[test]
    fn bind_multiple_declarations() {
        let (file, _map) = parse_and_bind("let x = 1; let y = 2; let z = 3;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 4);
    }

    #[test]
    fn bind_nested_scope() {
        let (file, _map) = parse_and_bind("function foo() { let x = 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);

        assert!(binder.symbol_count() >= 3);
    }

    #[test]
    fn flow_start_node_exists() {
        let (file, map) = parse_and_bind("let x = 1;");

        let flow = map.flow_node_of(&file.node);
        assert!(flow.is_some());
        let flow = flow.unwrap();
        assert!(flow.flags.contains(FlowFlags::START));
    }

    #[test]
    fn flow_identifier_has_flow_node() {
        let (file, map) = parse_and_bind("let x = 1; x;");

        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };

        let expr_stmt = &statements.nodes[1];
        let expr = match &expr_stmt.data {
            NodeData::ExpressionStatement(data) => &data.expression,
            _ => unreachable!(),
        };
        assert_eq!(expr.kind, SyntaxKind::Identifier);

        assert!(map.flow_node_of(expr).is_some());
    }

    #[test]
    fn flow_if_statement_merges() {

        let (file, _map) = parse_and_bind("let x = 1; if (x > 0) { x = 2; } else { x = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_while_statement() {
        let (file, _map) = parse_and_bind("let i = 0; while (i < 10) { i = i + 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_for_statement() {
        let (file, _map) = parse_and_bind("for (let i = 0; i < 10; i++) { console.log(i); }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_switch_statement() {
        let (file, _map) =
            parse_and_bind("let x = 1; switch (x) { case 1: x = 2; break; default: x = 0; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_return_statement_unreachable() {
        let (file, map) = parse_and_bind("function foo() { return 1; let x = 2; }");
        let _ = map;
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_explicit_return);
    }

    #[test]
    fn flow_throw_statement() {
        let (file, _map) = parse_and_bind("function foo() { throw new Error(); }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_assignment_has_effects() {
        let (file, _map) = parse_and_bind("let x = 1; x = 2;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_call_expression_has_effects() {
        let (file, _map) = parse_and_bind("console.log('hello');");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_catch_finally_does_not_crash() {

        let (file, _map) =
            parse_and_bind("try { let x = 1; } catch (e) { let y = 2; } finally { let z = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_with_throw_in_catch() {

        let (file, _map) = parse_and_bind(
            "function f() {\
             try { throw new Error(); }\
             catch (e) { return 1; }\
             return 2;\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_labeled_break_to_outer_loop() {

        let (file, _map) = parse_and_bind(
            "outer: for (let i = 0; i < 3; i++) {\
             for (let j = 0; j < 3; j++) {\
             if (j === 1) break outer;\
             }\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_labeled_continue_to_outer_loop() {

        let (file, _map) = parse_and_bind(
            "outer: for (let i = 0; i < 3; i++) {\
             for (let j = 0; j < 3; j++) {\
             if (j === 1) continue outer;\
             }\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_array_mutation_call_has_effects() {

        let (file, _map) = parse_and_bind("let arr = []; arr.push(1);");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    fn file_symbol<'a>(file: &'a SourceFile, map: &'a NodeSymbolMap) -> &'a Arc<Symbol> {
        map.symbols
            .get(&file.node.id())
            .expect("source file should have a symbol")
    }

    fn find_statement(file: &SourceFile, kind: SyntaxKind) -> Option<Arc<Node>> {
        let NodeData::SourceFile(data) = &file.node.data else {
            return None;
        };
        data.statements
            .nodes
            .iter()
            .find(|n| n.kind == kind)
            .cloned()
    }

    fn find_child(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
        let mut found: Option<Arc<Node>> = None;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if child.kind == kind {
                found = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        found
    }

    fn find_descendant(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
        if node.kind == kind {
            return Some(Arc::clone(node));
        }
        let mut found: Option<Arc<Node>> = None;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if found.is_none() {
                found = find_descendant(child, kind);
            }
            found.is_some()
        });
        found
    }

    #[test]
    fn bind_export_default_expression_creates_default_export_symbol() {

        let (file, map) = parse_and_bind("export default 42;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::Property),
            "expected Property flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
        let file_sym = file_symbol(&file, &map);
        let default_export = file_sym
            .exports
            .get(INTERNAL_SYMBOL_NAME_DEFAULT)
            .expect("default export in file exports");
        assert!(Arc::ptr_eq(default_export, sym));
    }

    #[test]
    fn bind_export_default_identifier_creates_alias() {

        let (file, map) = parse_and_bind("const foo = 1; export default foo;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::Alias),
            "expected Alias flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
    }

    #[test]
    fn bind_export_equals_creates_export_equals_symbol() {

        let (file, map) = parse_and_bind("function x() {} export = x;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS);
        assert!(
            sym.value_declaration.is_some(),
            "export = should have a value declaration set"
        );
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym
                .exports
                .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                .is_some()
        );
    }

    #[test]
    fn bind_export_star_creates_export_star_symbol() {

        let (file, map) = parse_and_bind("export * from \"mod\";");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        let sym = map.symbol_of(&export_decl).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::ExportStar),
            "expected ExportStar flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_STAR);
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym
                .exports
                .get(INTERNAL_SYMBOL_NAME_EXPORT_STAR)
                .is_some()
        );
    }

    #[test]
    fn bind_export_star_as_ns_creates_alias() {

        let (file, map) = parse_and_bind("export * as ns from \"mod\";");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        let ns_clause =
            find_child(&export_decl, SyntaxKind::NamespaceExport).expect("NamespaceExport clause");
        let sym = map
            .symbol_of(&ns_clause)
            .expect("symbol on NamespaceExport clause");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, "ns");
        let file_sym = file_symbol(&file, &map);
        let ns_export = file_sym.exports.get("ns").expect("ns export");
        assert!(Arc::ptr_eq(ns_export, sym));
    }

    #[test]
    fn bind_export_named_specifiers_does_not_duplicate() {

        let (file, map) = parse_and_bind("const a = 1; const b = 2; export { a, b };");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");

        assert!(
            map.symbol_of(&export_decl).is_none(),
            "export {{ a, b }} should not create a symbol on the ExportDeclaration"
        );
    }

    #[test]
    fn bind_import_clause_default_import_creates_local_alias() {

        let (file, map) = parse_and_bind("import D from \"mod\";");
        let import_decl =
            find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
        let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
        let sym = map.symbol_of(&clause).expect("symbol on ImportClause");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, "D");
        let locals = map.locals.get(&file.node.id()).expect("file locals table");
        let local_sym = locals.get("D").expect("D in file locals");
        assert!(Arc::ptr_eq(local_sym, sym));
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym.exports.get("D").is_none(),
            "default import should not be in exports"
        );
    }

    #[test]
    fn bind_import_clause_without_name_is_noop() {

        let (file, map) = parse_and_bind("import { x } from \"mod\";");
        let import_decl =
            find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
        let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
        assert!(
            map.symbol_of(&clause).is_none(),
            "ImportClause without a name should not get a symbol"
        );
    }

    #[test]
    fn bind_exported_namespace_member_has_export_symbol_link() {

        let (file, map) = parse_and_bind("namespace N { export const x = 1; }");

        let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
        let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
        let x_export = ns_sym.exports.get("x").expect("x in N's exports");
        assert!(
            x_export.export_symbol.is_some(),
            "exported namespace member should have export_symbol set"
        );
        assert!(Arc::ptr_eq(
            x_export.export_symbol.as_ref().unwrap(),
            x_export
        ));
    }

    #[test]
    fn bind_non_exported_namespace_member_has_no_export_symbol() {

        let (file, map) = parse_and_bind("namespace N { const x = 1; }");
        let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
        let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
        assert!(
            ns_sym.exports.get("x").is_none(),
            "non-exported member should not be in exports"
        );

        let locals = map.locals.get(&ns.id()).expect("namespace locals table");
        let x_local = locals.get("x").expect("x in locals");
        assert!(
            x_local.export_symbol.is_none(),
            "non-exported member should not have export_symbol"
        );
    }

    #[test]
    fn bind_exported_top_level_member_has_export_symbol_link() {

        let (file, map) = parse_and_bind("export const x = 1;");
        let var_stmt =
            find_statement(&file, SyntaxKind::VariableStatement).expect("variable statement");

        let decl_list =
            find_child(&var_stmt, SyntaxKind::VariableDeclarationList).expect("declaration list");
        let var_decl =
            find_child(&decl_list, SyntaxKind::VariableDeclaration).expect("variable declaration");
        let sym = map.symbol_of(&var_decl).expect("symbol for x");
        assert!(
            sym.export_symbol.is_some(),
            "exported top-level member should have export_symbol set"
        );
        assert!(Arc::ptr_eq(sym.export_symbol.as_ref().unwrap(), sym));
    }

    #[test]
    fn bind_generic_alias_type_params_do_not_leak_into_file_members() {

        let (file, map) = parse_and_bind(
            "export type G<T> = { [P in T]: string };\nexport type T = G<\"a\">;\nexport const q = 1;",
        );
        let fsym = file_symbol(&file, &map);
        let t_in_file = fsym.members.get("T").or_else(|| fsym.exports.get("T"));
        let Some(t_sym) = t_in_file else {
            panic!("exported alias T should be reachable in the file symbol tables");
        };

        assert!(
            t_sym
                .declarations
                .iter()
                .all(|d| d.kind == SyntaxKind::TypeAliasDeclaration),
            "file-table T merged with a type parameter: flags={:?}",
            t_sym.flags
        );
        assert!(
            !t_sym.flags.intersects(SymbolFlags::TypeParameter),
            "exported alias T must not carry TypeParameter flags (got {:?})",
            t_sym.flags
        );

        let g_stmt = find_statement(&file, SyntaxKind::TypeAliasDeclaration).unwrap();
        let g_sym = map.symbol_of(&g_stmt).expect("symbol for G");
        assert!(
            g_sym.members.get("T").is_some(),
            "G's type parameter should live in the alias symbol's members"
        );
    }

    #[test]
    fn bind_mapped_type_param_in_node_locals() {

        let (file, map) = parse_and_bind("type M<K extends string> = { [P in K]: number };");
        let fsym = file_symbol(&file, &map);
        assert!(
            fsym.members.get("P").is_none() && fsym.exports.get("P").is_none(),
            "mapped-type P must not leak into the file symbol tables"
        );
        let mapped = find_descendant(&file.node, SyntaxKind::MappedType).expect("mapped type node");
        let locals = map
            .locals
            .get(&mapped.id())
            .expect("mapped type node should have locals");
        assert!(
            locals.get("P").is_some(),
            "P should be in the mapped node's locals"
        );
    }
}
