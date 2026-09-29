#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use super::m5u_3::{is_only_single_line_whitespace, is_only_spaces_or_tabs, skip_single_line_whitespace, trim_right_single_line_whitespace};
use tsox_frontend::ast::{self, *};

pub fn is_expando_property_declaration_for_fix(node: Option<&Node>) -> bool {
    node.is_some_and(|n| {
        is_property_access_expression(n) || is_element_access_expression(n) || is_binary_expression(n)
    })
}

pub fn is_named_declaration_kind(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ArrowFunction
            | SyntaxKind::BindingElement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::EnumMember
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::GetAccessor
            | SyntaxKind::ImportClause
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::JsxAttribute
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::NamespaceExport
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::SetAccessor
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::TypeParameter
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::JSDocTypedefTag
            | SyntaxKind::JSDocCallbackTag
            | SyntaxKind::JSDocPropertyTag
            | SyntaxKind::NamedTupleMember
    )
}

pub fn is_infinity_or_nan_string(text: &str) -> bool {
    text == "Infinity" || text == "NaN"
}

pub fn is_non_empty_jsdoc(jsdoc: Option<&Node>) -> bool {
    let Some(jsdoc) = jsdoc else {
        return false;
    };
    match &jsdoc.data {
        NodeData::JSDoc(data) => {
            !data.comment.nodes.is_empty()
                || data.tags.as_ref().is_some_and(|t| !t.nodes.is_empty())
        }
        _ => false,
    }
}

pub fn is_jsdoc_snippet_prefix(prefix: &str) -> bool {
    let trimmed = trim_right_single_line_whitespace(prefix);
    if trimmed.ends_with("/**") {
        return true;
    }
    let start = skip_single_line_whitespace(prefix, 0);
    if start >= trimmed.len() || !trimmed[start..].starts_with('/') {
        return false;
    }
    if start + 3 > trimmed.len() {
        return false;
    }
    for i in start + 1..trimmed.len() {
        if !trimmed[i..].starts_with('*') {
            return false;
        }
    }
    trimmed.len() - start >= 3
}

pub fn is_jsdoc_snippet_suffix(suffix: &str) -> bool {
    let start = skip_single_line_whitespace(suffix, 0);
    let trimmed = trim_right_single_line_whitespace(&suffix[start..]);
    if trimmed.is_empty() {
        return true;
    }
    if !trimmed.ends_with('/') {
        return false;
    }
    for i in 0..trimmed.len() - 1 {
        if !trimmed[i..].starts_with('*') {
            return false;
        }
    }
    true
}
