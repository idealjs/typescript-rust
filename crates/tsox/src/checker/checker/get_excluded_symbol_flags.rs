#![allow(unused_imports)]

use super::*;

pub(crate) fn get_excluded_symbol_flags(flags: SymbolFlags) -> SymbolFlags {
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

pub(crate) fn is_module_or_enum_name(_node: &Node) -> bool {
    false
}

pub(crate) fn is_non_local_alias(symbol: &Arc<Symbol>, excludes: SymbolFlags) -> bool {
    if symbol.flags == SymbolFlags::Alias
        || (symbol.flags.intersects(SymbolFlags::Alias)
            && symbol.flags.intersects(SymbolFlags::Assignment))
    {
        !symbol.flags.intersects(excludes)
    } else {
        false
    }
}

pub(crate) fn is_type_only_node(node: &Node) -> bool {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::ImportSpecifier(data) => data.is_type_only,
        NodeData::ExportSpecifier(data) => data.is_type_only,
        NodeData::ExportDeclaration(data) => data.is_type_only,
        NodeData::ImportEqualsDeclaration(data) => data.is_type_only,
        _ => false,
    }
}

pub(crate) fn is_alias_symbol_declaration(node: &Node) -> bool {
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

pub(crate) fn node_name(node: &Node) -> Option<&str> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::Identifier(data) => Some(&data.text),
        NodeData::StringLiteral(data) => Some(&data.text),
        NodeData::NumericLiteral(data) => Some(&data.text),
        _ => None,
    }
}

pub(crate) fn is_expression_position_kind(kind: SyntaxKind) -> bool {
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

pub(crate) fn is_statement_kind(kind: SyntaxKind) -> bool {
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

pub(crate) fn is_declaration_name(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };
    let parent_kind = parent.kind;

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
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::FunctionExpression
            );
        }
    }
    false
}

pub(crate) fn is_property_access_name(node: &Arc<Node>) -> bool {
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

pub(crate) fn is_valid_identifier_text(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}

        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

pub(crate) enum ImportEntityError {
    None,

    NamespaceNotFound(Arc<Node>),

    TypeAsNamespace(Arc<Node>),

    HiddenByLocal(Arc<Node>),

    MissingMember((Arc<Node>, String, String)),
}

pub(crate) fn base_identifier_of(name: &Arc<Node>) -> Arc<Node> {
    let mut cur = Arc::clone(name);
    loop {
        let next = match &cur.data {
            crate::ast::NodeData::QualifiedName(q) => Arc::clone(&q.left),
            _ => return cur,
        };
        cur = next;
    }
}
