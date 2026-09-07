use crate::ast::*;

pub fn is_type_node_kind(kind: SyntaxKind) -> bool {
    if matches!(
        kind,
        SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::IntrinsicKeyword
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::JSDocAllType
            | SyntaxKind::JSDocNullableType
            | SyntaxKind::JSDocNonNullableType
            | SyntaxKind::JSDocOptionalType
            | SyntaxKind::JSDocVariadicType
    ) {
        return true;
    }

    (kind as i16) >= (SyntaxKind::TypePredicate as i16)
        && (kind as i16) <= (SyntaxKind::ImportType as i16)
}

pub fn is_type_node(node: &Node) -> bool {
    is_type_node_kind(node.kind)
}

pub fn is_jsdoc_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::JSDocTypeExpression as i16)
        && (kind as i16) <= (SyntaxKind::JSDocImportTag as i16)
}

pub fn is_jsdoc_node(node: &Node) -> bool {
    is_jsdoc_kind(node.kind)
}

pub fn is_jsdoc_tag(node: &Node) -> bool {
    (node.kind as i16) >= (SyntaxKind::JSDocUnknownTag as i16)
        && (node.kind as i16) <= (SyntaxKind::JSDocImportTag as i16)
}

pub fn is_jsdoc_link_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkCode | SyntaxKind::JSDocLinkPlain
    )
}

pub fn is_non_whitespace_token(node: &Node) -> bool {
    is_token_kind(node.kind) && !is_whitespace_only_jsx_text(node)
}

pub fn is_whitespace_only_jsx_text(node: &Node) -> bool {
    if let NodeData::JsxText(d) = &node.data {
        return d.contains_only_trivia_white_spaces;
    }
    false
}
