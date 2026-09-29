use crate::ast::node_data_generated::{
    for_each_child, is_array_literal_expression, is_decorator, is_object_literal_expression,
    node_text, NodeData,
};
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::syntax_kind_generated::SyntaxKind;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

pub fn kind_string(node: &Node) -> String {
    format!("{:?}", node.kind)
}

pub fn kind_value(node: &Node) -> i16 {
    node.kind as i16
}

pub fn decorators(node: &Node) -> Vec<Arc<Node>> {
    node.modifier_nodes()
        .iter()
        .filter(|m| is_decorator(m))
        .cloned()
        .collect()
}

pub fn iter_children(node: &Node) -> Vec<Arc<Node>> {
    let mut children = Vec::new();
    for_each_child(node, |child| {
        children.push(child.clone());
        false
    });
    children
}

pub fn member_list(node: &Node) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => Some(&d.members),
        NodeData::ClassExpression(d) => Some(&d.members),
        NodeData::InterfaceDeclaration(d) => Some(&d.members),
        NodeData::EnumDeclaration(d) => Some(&d.members),
        NodeData::TypeLiteralNode(d) => Some(&d.members),
        NodeData::MappedTypeNode(d) => d.members.as_ref(),
        _ => None,
    }
}

pub fn members(node: &Node) -> &[Arc<Node>] {
    match member_list(node) {
        Some(list) => &list.nodes,
        None => &[],
    }
}

pub fn initializer(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
        NodeData::ParameterDeclaration(d) => d.initializer.as_ref(),
        NodeData::BindingElement(d) => d.initializer.as_ref(),
        NodeData::PropertyDeclaration(d) => d.initializer.as_ref(),
        NodeData::PropertySignatureDeclaration(d) => Some(&d.initializer),
        NodeData::PropertyAssignment(d) => Some(&d.initializer),
        NodeData::EnumMember(d) => d.initializer.as_ref(),
        NodeData::ForStatement(d) => d.initializer.as_ref(),
        NodeData::ForInOrOfStatement(d) => Some(&d.initializer),
        NodeData::JsxAttribute(d) => d.initializer.as_ref(),
        _ => None,
    }
}

pub fn property_name(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ImportSpecifier(d) => d.property_name.as_ref(),
        NodeData::ExportSpecifier(d) => d.property_name.as_ref(),
        NodeData::BindingElement(d) => d.property_name.as_ref(),
        _ => None,
    }
}

pub fn property_name_or_name(node: &Node) -> Option<&Arc<Node>> {
    property_name(node).or_else(|| node.name())
}

pub fn is_type_only(node: &Node) -> bool {
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
        NodeData::ImportSpecifier(d) => d.is_type_only,
        NodeData::ImportClause(d) => d.phase_modifier == Some(SyntaxKind::TypeKeyword),
        NodeData::ExportDeclaration(d) => d.is_type_only,
        NodeData::ExportSpecifier(d) => d.is_type_only,
        _ => false,
    }
}

pub fn label(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::LabeledStatement(d) => Some(&d.label),
        NodeData::BreakStatement(d) => d.label.as_ref(),
        NodeData::ContinueStatement(d) => d.label.as_ref(),
        _ => None,
    }
}

pub fn module_specifier(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => Some(&d.module_specifier),
        NodeData::ExportDeclaration(d) => d.module_specifier.as_ref(),
        NodeData::JSDocImportTag(d) => Some(&d.module_specifier),
        _ => None,
    }
}

pub fn import_clause(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => d.import_clause.as_ref(),
        NodeData::JSDocImportTag(d) => d.import_clause.as_ref(),
        _ => None,
    }
}

pub fn property_list(node: &Node) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::ObjectLiteralExpression(d) => Some(&d.properties),
        NodeData::JsxAttributes(d) => Some(&d.properties),
        _ => None,
    }
}

pub fn properties(node: &Node) -> &[Arc<Node>] {
    match property_list(node) {
        Some(list) => &list.nodes,
        None => &[],
    }
}

pub fn element_list(node: &Node) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::NamedImports(d) => Some(&d.elements),
        NodeData::NamedExports(d) => Some(&d.elements),
        NodeData::BindingPattern(d) => Some(&d.elements),
        NodeData::ArrayLiteralExpression(d) => Some(&d.elements),
        NodeData::TupleTypeNode(d) => Some(&d.elements),
        _ => None,
    }
}

pub fn elements(node: &Node) -> &[Arc<Node>] {
    match element_list(node) {
        Some(list) => &list.nodes,
        None => &[],
    }
}

pub fn postfix_token(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::MethodDeclaration(d) => d.postfix_token.as_ref(),
        NodeData::ShorthandPropertyAssignment(d) => d.postfix_token.as_ref(),
        NodeData::MethodSignatureDeclaration(d) => d.postfix_token.as_ref(),
        NodeData::PropertySignatureDeclaration(d) => d.postfix_token.as_ref(),
        NodeData::PropertyAssignment(d) => d.postfix_token.as_ref(),
        NodeData::PropertyDeclaration(d) => d.postfix_token.as_ref(),
        _ => None,
    }
}

pub fn question_dot_token(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ElementAccessExpression(d) => d.question_dot_token.as_ref(),
        NodeData::PropertyAccessExpression(d) => d.question_dot_token.as_ref(),
        NodeData::CallExpression(d) => d.question_dot_token.as_ref(),
        NodeData::TaggedTemplateExpression(d) => d.question_dot_token.as_ref(),
        _ => None,
    }
}

pub fn parameter_list(node: &Node) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => Some(&d.parameters),
        NodeData::FunctionExpression(d) => Some(&d.parameters),
        NodeData::ArrowFunction(d) => Some(&d.parameters),
        NodeData::MethodDeclaration(d) => Some(&d.parameters),
        NodeData::MethodSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::ConstructorDeclaration(d) => Some(&d.parameters),
        NodeData::GetAccessorDeclaration(d) => Some(&d.parameters),
        NodeData::SetAccessorDeclaration(d) => Some(&d.parameters),
        NodeData::CallSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::ConstructSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::FunctionTypeNode(d) => Some(&d.parameters),
        NodeData::ConstructorTypeNode(d) => Some(&d.parameters),
        NodeData::JSDocSignature(d) => Some(&d.parameters),
        _ => None,
    }
}

pub fn parameters(node: &Node) -> &[Arc<Node>] {
    match parameter_list(node) {
        Some(list) => &list.nodes,
        None => &[],
    }
}

pub fn is_locals_container(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::SourceFile
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::JSDocSignature
    )
}

pub fn is_type_or_js_type_alias_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::TypeAliasDeclaration
        || node.kind == SyntaxKind::JSTypeAliasDeclaration
}

pub fn is_import_declaration_or_js_import_declaration(node: &Node) -> bool {
    node.kind == SyntaxKind::ImportDeclaration
        || node.kind == SyntaxKind::JSImportDeclaration
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind {
    Read,
    Write,
    ReadWrite,
}

pub fn is_write_only_access(node: &Arc<Node>) -> bool {
    access_kind(node) == AccessKind::Write
}

pub fn is_write_access(node: &Arc<Node>) -> bool {
    access_kind(node) != AccessKind::Read
}

pub fn is_write_access_for_reference(
    name: &Arc<Node>,
    symbols: &crate::ast::symbol_map::NodeSymbolMap,
) -> bool {
    let decl_write = get_declaration_from_name(name, symbols)
        .map(|decl| declaration_is_write_access(&decl))
        .unwrap_or(false);
    (decl_write || name.kind == SyntaxKind::DefaultKeyword) || is_write_access(name)
}

pub fn reverse_access_kind(a: AccessKind) -> AccessKind {
    match a {
        AccessKind::Read => AccessKind::Write,
        AccessKind::Write => AccessKind::Read,
        AccessKind::ReadWrite => AccessKind::ReadWrite,
    }
}

pub fn access_kind(node: &Arc<Node>) -> AccessKind {
    let parent = match node.parent() {
        Some(p) => p,
        None => return AccessKind::Read,
    };
    match &parent.data {
        NodeData::ParenthesizedExpression(_) => access_kind(&parent),
        NodeData::PrefixUnaryExpression(d) => {
            if d.operator == SyntaxKind::PlusPlusToken
                || d.operator == SyntaxKind::MinusMinusToken
            {
                AccessKind::ReadWrite
            } else {
                AccessKind::Read
            }
        }
        NodeData::PostfixUnaryExpression(d) => {
            if d.operator == SyntaxKind::PlusPlusToken
                || d.operator == SyntaxKind::MinusMinusToken
            {
                AccessKind::ReadWrite
            } else {
                AccessKind::Read
            }
        }
        NodeData::BinaryExpression(d) => {
            if Arc::ptr_eq(&d.left, node)
                && crate::ast::node_data_generated::is_assignment_operator(d.operator_token.kind)
            {
                if d.operator_token.kind == SyntaxKind::EqualsToken {
                    AccessKind::Write
                } else {
                    AccessKind::ReadWrite
                }
            } else {
                AccessKind::Read
            }
        }
        NodeData::PropertyAccessExpression(_) => {
            let is_name = parent
                .name()
                .map(|n| Arc::ptr_eq(n, node))
                .unwrap_or(false);
            if is_name {
                access_kind(&parent)
            } else {
                AccessKind::Read
            }
        }
        NodeData::PropertyAssignment(d) => {
            let parent_access = parent.parent().map(|p| access_kind(&p));
            if Arc::ptr_eq(&d.name, node) {
                parent_access.map(reverse_access_kind).unwrap_or(AccessKind::Read)
            } else {
                parent_access.unwrap_or(AccessKind::Read)
            }
        }
        NodeData::ShorthandPropertyAssignment(d) => {
            let is_obj_assign_init = d
                .object_assignment_initializer
                .as_ref()
                .map(|n| Arc::ptr_eq(n, node))
                .unwrap_or(false);
            if is_obj_assign_init {
                AccessKind::Read
            } else {
                parent.parent().map(|p| access_kind(&p)).unwrap_or(AccessKind::Read)
            }
        }
        NodeData::ArrayLiteralExpression(_) => access_kind(&parent),
        NodeData::ForInOrOfStatement(d) => {
            if Arc::ptr_eq(&d.initializer, node) {
                AccessKind::Write
            } else {
                AccessKind::Read
            }
        }
        _ => AccessKind::Read,
    }
}

pub fn declaration_is_write_access(decl: &Node) -> bool {
    use crate::ast::node_flags::NodeFlags;
    if decl.flags.contains(NodeFlags::Ambient) {
        return true;
    }
    match decl.kind {
        SyntaxKind::BinaryExpression
        | SyntaxKind::BindingElement
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::DefaultKeyword
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::EnumMember
        | SyntaxKind::ExportSpecifier
        | SyntaxKind::ImportClause
        | SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::ImportSpecifier
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::JSDocCallbackTag
        | SyntaxKind::JSDocTypedefTag
        | SyntaxKind::JsxAttribute
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::NamespaceExportDeclaration
        | SyntaxKind::NamespaceImport
        | SyntaxKind::NamespaceExport
        | SyntaxKind::Parameter
        | SyntaxKind::ShorthandPropertyAssignment
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::TypeParameter => true,
        SyntaxKind::PropertyAssignment => !decl
            .parent()
            .map(|p| is_array_literal_or_object_literal_destructuring_pattern(&p))
            .unwrap_or(false),
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::Constructor
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor => node_body(decl).is_some(),
        SyntaxKind::VariableDeclaration | SyntaxKind::PropertyDeclaration => {
            initializer(decl).is_some()
                || decl
                    .parent()
                    .map(|p| p.kind == SyntaxKind::CatchClause)
                    .unwrap_or(false)
        }
        SyntaxKind::MethodSignature
        | SyntaxKind::PropertySignature
        | SyntaxKind::JSDocPropertyTag
        | SyntaxKind::JSDocParameterTag => false,
        _ => panic!("Unhandled case in declaration_is_write_access"),
    }
}

fn node_body(node: &Node) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(d.body.clone()),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        NodeData::ArrowFunction(d) => Some(d.body.clone()),
        NodeData::ClassStaticBlockDeclaration(d) => Some(d.body.clone()),
        _ => None,
    }
}

pub fn is_array_literal_or_object_literal_destructuring_pattern(node: &Arc<Node>) -> bool {
    if !(is_array_literal_expression(node) || is_object_literal_expression(node)) {
        return false;
    }
    let parent = match node.parent() {
        Some(p) => p,
        None => return false,
    };
    if let NodeData::BinaryExpression(d) = &parent.data {
        if Arc::ptr_eq(&d.left, node) && d.operator_token.kind == SyntaxKind::EqualsToken {
            return true;
        }
    }
    if parent.kind == SyntaxKind::ForOfStatement {
        if let NodeData::ForInOrOfStatement(d) = &parent.data {
            if Arc::ptr_eq(&d.initializer, node) {
                return true;
            }
        }
    }
    if matches!(&parent.data, NodeData::PropertyAssignment(_)) {
        return parent
            .parent()
            .map(|p| is_array_literal_or_object_literal_destructuring_pattern(&p))
            .unwrap_or(false);
    }
    is_array_literal_or_object_literal_destructuring_pattern(&parent)
}

pub fn get_declaration_from_name(
    name: &Arc<Node>,
    symbols: &crate::ast::symbol_map::NodeSymbolMap,
) -> Option<Arc<Node>> {
    let parent = name.parent()?;
    match name.kind {
        SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::NumericLiteral => {
            if matches!(&parent.data, NodeData::ComputedPropertyName(_)) {
                return parent.parent();
            }
            declaration_from_identifier_name(name, &parent, symbols)
        }
        SyntaxKind::Identifier => declaration_from_identifier_name(name, &parent, symbols),
        SyntaxKind::PrivateIdentifier => {
            if crate::ast::utilities_declarations::is_declaration(&parent)
                && parent
                    .name()
                    .map(|n| Arc::ptr_eq(n, name))
                    .unwrap_or(false)
            {
                Some(parent)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn declaration_from_identifier_name(
    name: &Arc<Node>,
    parent: &Arc<Node>,
    symbols: &crate::ast::symbol_map::NodeSymbolMap,
) -> Option<Arc<Node>> {
    if crate::ast::utilities_declarations::is_declaration(parent) {
        if parent
            .name()
            .map(|n| Arc::ptr_eq(n, name))
            .unwrap_or(false)
        {
            return Some(parent.clone());
        }
        return None;
    }
    if matches!(&parent.data, NodeData::QualifiedName(_)) {
        let tag = parent.parent()?;
        if tag.kind == SyntaxKind::JSDocParameterTag
            && tag.name().map(|n| Arc::ptr_eq(n, parent)).unwrap_or(false)
        {
            return Some(tag);
        }
        return None;
    }
    let bin_exp = parent.parent()?;
    if let NodeData::BinaryExpression(d) = &bin_exp.data {
        if crate::ast::mig::m3e_4::get_assignment_declaration_kind(&bin_exp)
            != crate::ast::mig::m3e_4::JsDeclarationKind::None
        {
            let left_has_symbol = symbols.symbol_of(&d.left).is_some();
            if left_has_symbol || symbols.symbol_of(&bin_exp).is_some() {
                let decl_name = crate::ast::utilities_navigation::get_name_of_declaration(&bin_exp);
                if decl_name.map(|n| Arc::ptr_eq(&n, name)).unwrap_or(false) {
                    return Some(bin_exp);
                }
            }
        }
    }
    None
}

pub fn get_declaration_name(declaration: &Arc<Node>) -> String {
    let name =
        crate::ast::utilities_navigation::get_non_assigned_name_of_declaration(declaration);
    if let Some(name) = name {
        if let NodeData::ComputedPropertyName(d) = &name.data {
            if crate::ast::utilities_expressions::is_string_or_numeric_literal_like(&d.expression)
            {
                return node_text(&d.expression).to_string();
            }
            if matches!(&d.expression.data, NodeData::PropertyAccessExpression(_)) {
                return d
                    .expression
                    .name()
                    .map(|n| node_text(n).to_string())
                    .unwrap_or_default();
            }
        } else if crate::ast::utilities_predicates::is_property_name_node(&name) {
            return node_text(&name).to_string();
        }
    }
    String::new()
}

pub type PragmaKindFlags = u8;

pub const PRAGMA_KIND_TRIPLE_SLASH_XML: PragmaKindFlags = 1;

#[derive(Debug)]
pub struct PragmaArgumentSpecification {
    pub name: String,
    pub optional: bool,
    pub capture_span: bool,
}

#[derive(Debug)]
pub struct PragmaSpecification {
    pub args: Vec<PragmaArgumentSpecification>,
    pub kind: PragmaKindFlags,
}

pub fn is_triple_slash(spec: &PragmaSpecification) -> bool {
    spec.kind & PRAGMA_KIND_TRIPLE_SLASH_XML > 0
}

pub fn get_resolution_mode_override(
    node: &Node,
    grammar_error_on_node: Option<&dyn Fn(&Arc<Node>) -> bool>,
) -> (
    tsox_core::core::compiler_options::ResolutionMode,
    bool,
) {
    use tsox_core::core::compiler_options::ResolutionMode;
    let attributes = match &node.data {
        NodeData::ImportAttributes(d) => &d.attributes,
        _ => return (ResolutionMode::default(), false),
    };
    let attribute = attributes.nodes.iter().find(|a| {
        a.name()
            .map(|n| node_text(n) == "resolution-mode")
            .unwrap_or(false)
    });
    let attribute = match attribute {
        Some(a) => a,
        None => return (ResolutionMode::default(), false),
    };
    let value = match &attribute.data {
        NodeData::ImportAttribute(d) => &d.value,
        _ => return (ResolutionMode::default(), false),
    };
    if !crate::ast::utilities_expressions::is_string_literal_like(value) {
        return (ResolutionMode::default(), false);
    }
    let text = node_text(value);
    if text != "import" && text != "require" {
        if let Some(report) = grammar_error_on_node {
            report(value);
        }
        return (ResolutionMode::default(), false);
    }
    if text == "import" {
        (ResolutionMode::ESNext, true)
    } else {
        (ResolutionMode::CommonJS, true)
    }
}

pub fn has_identifier(file: &crate::ast::node_source_file::SourceFile, name: &str) -> bool {
    let mut identifiers = HashSet::new();
    collect_identifiers_for_source_file(file, &mut identifiers);
    identifiers.contains(name)
}

fn collect_identifiers_for_source_file(
    source_file: &crate::ast::node_source_file::SourceFile,
    identifiers: &mut HashSet<String>,
) {
    fn collect(node: &Node, identifiers: &mut HashSet<String>) -> bool {
        match node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => {
                identifiers.insert(node_text(node).to_string());
            }
            _ => {}
        }
        for_each_child(node, |child| {
            collect(child, identifiers);
            false
        });
        false
    }
    collect(&source_file.node, identifiers);
}

pub fn get_name_table(
    file: &crate::ast::node_source_file::SourceFile,
) -> HashMap<String, i64> {
    let mut name_table: HashMap<String, i64> = HashMap::new();

    fn walk(node: &Arc<Node>, file: &crate::ast::node_source_file::SourceFile, name_table: &mut HashMap<String, i64>) -> bool {
        let is_name = (crate::ast::node_data_generated::is_identifier(node)
            && !is_tag_name(node)
            && !node_text(node).is_empty())
            || (crate::ast::utilities_expressions::is_string_or_numeric_literal_like(node)
                && literal_is_name(node))
            || crate::ast::node_data_generated::is_private_identifier(node);
        if is_name {
            let text = node_text(node).to_string();
            if name_table.contains_key(&text) {
                name_table.insert(text, -1);
            } else {
                name_table.insert(text, node.pos() as i64);
            }
        }

        for_each_child(node, |child| {
            walk(child, file, name_table);
            false
        });
        let jsdoc_nodes = node.jsdoc(file);
        for jsdoc in jsdoc_nodes {
            for_each_child(&jsdoc, |child| {
                walk(child, file, name_table);
                false
            });
        }
        false
    }

    for_each_child(&file.node, |child| {
        walk(child, file, &mut name_table);
        false
    });
    name_table
}

fn is_tag_name(node: &Arc<Node>) -> bool {
    node.parent()
        .map(|p| {
            crate::ast::utilities_types::is_jsdoc_tag(&p)
                && p.name().map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
        })
        .unwrap_or(false)
}

fn literal_is_name(node: &Arc<Node>) -> bool {
    is_declaration_name(node)
        || node
            .parent()
            .map(|p| p.kind == SyntaxKind::ExternalModuleReference)
            .unwrap_or(false)
        || is_argument_of_element_access_expression(node)
        || is_literal_computed_property_declaration_name(node)
}

fn is_declaration_name(name: &Arc<Node>) -> bool {
    match name.parent() {
        Some(parent) => {
            crate::ast::utilities_declarations::is_declaration(&parent)
                && parent
                    .name()
                    .map(|n| Arc::ptr_eq(n, name))
                    .unwrap_or(false)
        }
        None => false,
    }
}

fn is_argument_of_element_access_expression(node: &Arc<Node>) -> bool {
    match node.parent() {
        Some(parent) => match &parent.data {
            NodeData::ElementAccessExpression(d) => Arc::ptr_eq(&d.argument_expression, node),
            _ => false,
        },
        None => false,
    }
}

fn is_literal_computed_property_declaration_name(node: &Arc<Node>) -> bool {
    let parent = match node.parent() {
        Some(p) => p,
        None => return false,
    };
    if !matches!(&parent.data, NodeData::ComputedPropertyName(_)) {
        return false;
    }
    if !crate::ast::utilities_declarations::is_declaration(&parent) {
        return false;
    }
    crate::ast::utilities_expressions::is_string_or_numeric_literal_like(node)
}
