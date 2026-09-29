use std::cmp::Ordering;
use std::sync::Arc;

use crate::ast::node_node::Node;
use crate::ast::node_source_file::SourceFile;
use crate::ast::symbol::Symbol;
use crate::ast::symbol::SymbolTable;
use crate::ast::syntax_kind_generated::SyntaxKind;

use crate::ast::node_data_generated::NodeData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindAncestorResult {
    False,
    True,
    Quit,
}

pub fn to_find_ancestor_result(b: bool) -> FindAncestorResult {
    if b {
        FindAncestorResult::True
    } else {
        FindAncestorResult::False
    }
}

pub fn find_ancestor_or_quit(
    node: Option<&Arc<Node>>,
    callback: impl Fn(&Arc<Node>) -> FindAncestorResult,
) -> Option<Arc<Node>> {
    let mut node = node.cloned();
    while let Some(current) = node {
        match callback(&current) {
            FindAncestorResult::Quit => return None,
            FindAncestorResult::True => return Some(current),
            FindAncestorResult::False => {}
        }
        node = current.parent();
    }
    None
}

pub fn find_many_ancestors(
    node: Option<&Arc<Node>>,
    callbacks: &[&dyn Fn(&Arc<Node>) -> bool],
) -> Vec<Option<Arc<Node>>> {
    let mut ancestors: Vec<Option<Arc<Node>>> = vec![None; callbacks.len()];
    let mut found = 0;
    let mut node = node.cloned();
    while let Some(current) = node {
        for (i, callback) in callbacks.iter().enumerate() {
            if ancestors[i].is_none() && callback(&current) {
                ancestors[i] = Some(current.clone());
                found += 1;
                if found == callbacks.len() {
                    return ancestors;
                }
                break;
            }
        }
        node = current.parent();
    }
    ancestors
}

pub fn get_members(symbol: &Symbol) -> &SymbolTable {
    get_symbol_table(&symbol.members)
}

pub fn get_exports(symbol: &Symbol) -> &SymbolTable {
    get_symbol_table(&symbol.exports)
}

pub fn get_locals<'a>(
    container: &Arc<Node>,
    symbols: &'a crate::ast::symbol_map::NodeSymbolMap,
) -> Option<&'a SymbolTable> {
    symbols.locals_of(container)
}

fn get_symbol_table(table: &SymbolTable) -> &SymbolTable {
    table
}

pub fn find_last_visible_node(nodes: &[Arc<Node>]) -> Option<Arc<Node>> {
    let mut from_end = 1;
    while from_end <= nodes.len()
        && nodes[nodes.len() - from_end]
            .flags
            .contains(crate::ast::node_flags::NodeFlags::Reparsed)
    {
        from_end += 1;
    }
    if from_end <= nodes.len() {
        Some(nodes[nodes.len() - from_end].clone())
    } else {
        None
    }
}

pub fn can_have_illegal_decorators(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
    )
}

pub fn can_have_illegal_modifiers(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::NamespaceExportDeclaration
    )
}

pub fn for_each_return_statement(
    body: &Arc<Node>,
    mut visitor: impl FnMut(&Arc<Node>) -> bool,
) -> bool {
    fn traverse(node: &Arc<Node>, visitor: &mut impl FnMut(&Arc<Node>) -> bool) -> bool {
        match node.kind {
            SyntaxKind::ReturnStatement => visitor(node),
            SyntaxKind::CaseBlock
            | SyntaxKind::Block
            | SyntaxKind::IfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::CaseClause
            | SyntaxKind::DefaultClause
            | SyntaxKind::LabeledStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::CatchClause => {
                crate::ast::node_data_generated::for_each_child(node, |child| {
                    traverse(child, visitor)
                })
            }
            _ => false,
        }
    }
    traverse(body, &mut visitor)
}

pub fn get_jsdoc_deprecated_tag(node: &Arc<Node>) -> Option<Arc<Node>> {
    if !node
        .flags
        .contains(crate::ast::node_flags::NodeFlags::HasJSDoc)
    {
        return None;
    }
    None
}

pub fn get_jsdoc_deprecated_tag_with_file(
    node: &Arc<Node>,
    file: &SourceFile,
) -> Option<Arc<Node>> {
    for jsdoc in node.jsdoc(file) {
        if let NodeData::JSDoc(d) = &jsdoc.data {
            if let Some(tags) = &d.tags {
                for tag in &tags.nodes {
                    if crate::ast::node_data_generated::is_jsdoc_deprecated_tag(tag) {
                        return Some(tag.clone());
                    }
                }
            }
        }
    }
    None
}

pub fn get_element_or_property_access_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::PropertyAccessExpression => {
            let name = node.name().expect("PropertyAccessExpression has name");
            if name.kind == SyntaxKind::Identifier {
                Some(name.clone())
            } else {
                None
            }
        }
        SyntaxKind::ElementAccessExpression => {
            let arg = match &node.data {
                NodeData::ElementAccessExpression(d) => {
                    crate::ast::mig::m3g_3::skip_parentheses(&d.argument_expression)
                }
                _ => unreachable!(),
            };
            if crate::ast::utilities_expressions::is_string_or_numeric_literal_like(&arg) {
                Some(arg)
            } else {
                None
            }
        }
        _ => panic!("Unhandled case in GetElementOrPropertyAccessName"),
    }
}

pub fn get_initializer_of_binary_expression(expr: &Arc<Node>) -> Option<Arc<Node>> {
    let mut expr = expr.clone();
    loop {
        let right = match &expr.data {
            NodeData::BinaryExpression(d) => d.right.clone(),
            _ => return expr.expression().cloned(),
        };
        if right.kind == SyntaxKind::BinaryExpression {
            expr = right;
        } else {
            return right.expression().cloned();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsDeclarationKind {
    None,
    Property,
    ModuleExports,
    ExportsProperty,
    ThisProperty,
    ObjectDefinePropertyValue,
    ObjectDefinePropertyExports,
}

pub fn get_assignment_declaration_kind(node: &Arc<Node>) -> JsDeclarationKind {
    match &node.data {
        NodeData::BinaryExpression(bin) => {
            if bin.operator_token.kind == SyntaxKind::EqualsToken
                && crate::ast::utilities_expressions::is_access_expression(&bin.left)
            {
                if crate::ast::utilities_sourcefile::is_in_js_file(&bin.left) {
                    if crate::ast::mig::m3g::is_module_exports_access_expression(&bin.left)
                        && !crate::ast::utilities_misc::is_exports_identifier(&bin.right)
                    {
                        return JsDeclarationKind::ModuleExports;
                    }
                    let left_expr = bin.left.expression();
                    if (left_expr.is_some_and(|e| crate::ast::mig::m3g::is_module_exports_access_expression(e))
                        || left_expr.is_some_and(|e| crate::ast::utilities_misc::is_exports_identifier(e)))
                        && get_element_or_property_access_name(&bin.left).is_some()
                    {
                        return JsDeclarationKind::ExportsProperty;
                    }
                    if bin.left.expression().map(|e| e.kind) == Some(SyntaxKind::ThisKeyword) {
                        return JsDeclarationKind::ThisProperty;
                    }
                }
                let left_is_property_access = bin.left.kind == SyntaxKind::PropertyAccessExpression;
                let entity_ok = bin
                    .left
                    .expression()
                    .map(|e| {
                        crate::ast::mig::x6a::is_entity_name_expression_ex(
                            e,
                            crate::ast::utilities_sourcefile::is_in_js_file(&bin.left),
                        )
                    })
                    .unwrap_or(false);
                if (left_is_property_access
                    && entity_ok
                    && bin.left.name().map(|n| n.kind) == Some(SyntaxKind::Identifier))
                    || (bin.left.kind == SyntaxKind::ElementAccessExpression && entity_ok)
                {
                    return JsDeclarationKind::Property;
                }
            }
        }
        NodeData::CallExpression(_) => {
            if crate::ast::utilities_sourcefile::is_in_js_file(node)
                && crate::ast::mig::m3f_3::is_bindable_object_define_property_call(node)
            {
                let entity_name = &crate::ast::mig::x1a::arguments(node)[0];
                if crate::ast::utilities_misc::is_exports_identifier(entity_name)
                    || crate::ast::mig::m3g::is_module_exports_access_expression(entity_name)
                {
                    return JsDeclarationKind::ObjectDefinePropertyExports;
                }
                return JsDeclarationKind::ObjectDefinePropertyValue;
            }
        }
        _ => {}
    }
    JsDeclarationKind::None
}

pub fn get_heritage_clause_element_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    if node.kind == SyntaxKind::TypeReference {
        match &node.data {
            NodeData::TypeReferenceNode(d) => Some(d.type_name.clone()),
            _ => None,
        }
    } else {
        node.expression().cloned()
    }
}

pub fn get_immediately_invoked_function_expression(fn_node: &Arc<Node>) -> Option<Arc<Node>> {
    if crate::ast::utilities_functions::is_function_expression_or_arrow_function(fn_node) {
        let mut prev = fn_node.clone();
        let mut parent = fn_node.parent();
        while let Some(p) = &parent {
            if p.kind != SyntaxKind::ParenthesizedExpression {
                break;
            }
            prev = p.clone();
            parent = p.parent();
        }
        if let Some(p) = &parent {
            if p.kind == SyntaxKind::CallExpression
                && p.expression().map(|e| Arc::ptr_eq(e, &prev)) == Some(true)
            {
                return Some(p.clone());
            }
        }
    }
    None
}

pub fn expression_is_alias(node: &Arc<Node>) -> bool {
    crate::ast::utilities_predicates::is_entity_name_expression(node)
        || node.kind == SyntaxKind::ClassExpression
}

pub fn get_import_attributes(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => d.attributes.clone(),
        NodeData::ExportDeclaration(d) => d.attributes.clone(),
        NodeData::ImportTypeNode(d) => d.attributes.clone(),
        _ => panic!("Unhandled case in GetImportAttributes: {:?}", node.kind),
    }
}

pub fn get_this_container(
    node: &Arc<Node>,
    include_arrow_functions: bool,
    include_class_computed_property_name: bool,
) -> Arc<Node> {
    let mut node = node.clone();
    loop {
        let mut current = node
            .parent()
            .expect("nil parent in getThisContainer");
        match current.kind {
            SyntaxKind::ComputedPropertyName => {
                if include_class_computed_property_name {
                    let grandparent = current.parent().and_then(|p| p.parent());
                    if grandparent
                        .as_ref()
                        .is_some_and(|p| crate::ast::utilities_functions::is_class_like(p))
                    {
                        return current;
                    }
                }
                let grandparent = current.parent().and_then(|p| p.parent());
                node = grandparent.expect("nil parent in getThisContainer");
                continue;
            }
            SyntaxKind::Decorator => {
                let parent = current.parent();
                let parent_kind = parent.as_ref().map(|p| p.kind);
                if parent_kind == Some(SyntaxKind::Parameter) {
                    let grandparent = parent.as_ref().and_then(|p| p.parent());
                    if grandparent
                        .as_ref()
                        .is_some_and(|p| crate::ast::utilities_functions::is_class_element(p))
                    {
                        node = grandparent.expect("nil parent in getThisContainer");
                        continue;
                    }
                }
                if parent
                    .as_ref()
                    .is_some_and(|p| crate::ast::utilities_functions::is_class_element(p))
                {
                    node = parent.expect("nil parent in getThisContainer");
                    continue;
                }
            }
            SyntaxKind::ArrowFunction => {
                if include_arrow_functions {
                    return current;
                }
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::SourceFile => return current,
            _ => {}
        }
        node = current;
    }
}

pub fn get_new_target_container(node: &Arc<Node>) -> Option<Arc<Node>> {
    let container = get_this_container(node, false, false);
    match container.kind {
        SyntaxKind::Constructor
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression => Some(container),
        _ => None,
    }
}

pub fn get_enclosing_block_scope_container(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent()?;
    crate::ast::utilities_navigation::find_ancestor(&parent, |current| {
        current
            .parent()
            .is_some_and(|p| crate::ast::mig::m3f_3::is_block_scope(current, &p))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticMeaning(pub u32);

impl SemanticMeaning {
    pub const NONE: SemanticMeaning = SemanticMeaning(0);
    pub const VALUE: SemanticMeaning = SemanticMeaning(1 << 0);
    pub const TYPE: SemanticMeaning = SemanticMeaning(1 << 1);
    pub const NAMESPACE: SemanticMeaning = SemanticMeaning(1 << 2);
    pub const ALL: SemanticMeaning = SemanticMeaning((1 << 0) | (1 << 1) | (1 << 2));

    pub fn contains(self, other: SemanticMeaning) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn union(self, other: SemanticMeaning) -> SemanticMeaning {
        SemanticMeaning(self.0 | other.0)
    }
}

pub fn get_meaning_from_declaration(node: &Arc<Node>) -> SemanticMeaning {
    match node.kind {
        SyntaxKind::VariableDeclaration => SemanticMeaning::VALUE,
        SyntaxKind::Parameter
        | SyntaxKind::BindingElement
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::ShorthandPropertyAssignment
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::Constructor
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::CatchClause
        | SyntaxKind::JsxAttribute => SemanticMeaning::VALUE,
        SyntaxKind::TypeParameter
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::TypeLiteral => SemanticMeaning::TYPE,
        SyntaxKind::EnumMember | SyntaxKind::ClassDeclaration => {
            SemanticMeaning::VALUE.union(SemanticMeaning::TYPE)
        }
        SyntaxKind::ModuleDeclaration => {
            if crate::ast::utilities_modules::is_ambient_module(node) {
                SemanticMeaning::NAMESPACE.union(SemanticMeaning::VALUE)
            } else if crate::ast::utilities_module_state::get_module_instance_state(node)
                == crate::ast::utilities_module_state::ModuleInstanceState::Instantiated
            {
                SemanticMeaning::NAMESPACE.union(SemanticMeaning::VALUE)
            } else {
                SemanticMeaning::NAMESPACE
            }
        }
        SyntaxKind::EnumDeclaration
        | SyntaxKind::NamedImports
        | SyntaxKind::ImportSpecifier
        | SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::ImportDeclaration
        | SyntaxKind::JSImportDeclaration
        | SyntaxKind::ExportAssignment
        | SyntaxKind::ExportDeclaration => SemanticMeaning::ALL,
        SyntaxKind::SourceFile => SemanticMeaning::NAMESPACE.union(SemanticMeaning::VALUE),
        _ => SemanticMeaning::ALL,
    }
}

pub fn get_declaration_of_kind(symbol: &Symbol, kind: SyntaxKind) -> Option<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .find(|declaration| declaration.kind == kind)
        .cloned()
}

pub fn find_constructor_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
    let members = match &node.data {
        NodeData::ClassDeclaration(d) => &d.members,
        NodeData::ClassExpression(d) => &d.members,
        _ => return None,
    };
    for member in &members.nodes {
        if member.kind == SyntaxKind::Constructor {
            let body = match &member.data {
                NodeData::ConstructorDeclaration(d) => d.body.clone(),
                _ => None,
            };
            if crate::ast::utilities_synthesized::node_is_present(body.as_ref()) {
                return Some(member.clone());
            }
        }
    }
    None
}

pub fn get_first_identifier(node: &Arc<Node>) -> Arc<Node> {
    match node.kind {
        SyntaxKind::Identifier => node.clone(),
        SyntaxKind::QualifiedName => match &node.data {
            NodeData::QualifiedName(d) => get_first_identifier(&d.left),
            _ => unreachable!(),
        },
        SyntaxKind::PropertyAccessExpression => match &node.data {
            NodeData::PropertyAccessExpression(d) => get_first_identifier(&d.expression),
            _ => unreachable!(),
        },
        _ => panic!("Unhandled case in GetFirstIdentifier"),
    }
}

pub fn get_namespace_declaration_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => {
            let import_clause = match &node.data {
                NodeData::ImportDeclaration(d) => d.import_clause.clone(),
                _ => None,
            };
            if let Some(import_clause) = import_clause {
                if let NodeData::ImportClause(d) = &import_clause.data {
                    if let Some(named_bindings) = &d.named_bindings {
                        if named_bindings.kind == SyntaxKind::NamespaceImport {
                            return Some(named_bindings.clone());
                        }
                    }
                }
            }
            None
        }
        SyntaxKind::ImportEqualsDeclaration => Some(node.clone()),
        SyntaxKind::ExportDeclaration => match &node.data {
            NodeData::ExportDeclaration(d) => d.export_clause.as_ref().and_then(|export_clause| {
                if export_clause.kind == SyntaxKind::NamespaceExport {
                    Some(export_clause.clone())
                } else {
                    None
                }
            }),
            _ => unreachable!(),
        },
        _ => panic!("Unhandled case in GetNamespaceDeclarationNode"),
    }
}
