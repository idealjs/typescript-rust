#![allow(dead_code)]

use crate::ast::*;
use std::sync::Arc;

use super::*;

pub fn get_local_symbol_for_export_default(symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
    if !is_export_default_symbol(symbol) || symbol.declarations.is_empty() {
        return None;
    }
    for decl in &symbol.declarations {
        if let Some(local) = node_local_symbol(decl) {
            return Some(local);
        }
    }
    None
}

pub fn is_export_default_symbol(symbol: &Arc<Symbol>) -> bool {
    !symbol.declarations.is_empty()
        && has_syntactic_modifier(&symbol.declarations[0], ModifierFlags::Default)
}

pub fn get_is_deferred_context(location: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
    if location.kind != SyntaxKind::ArrowFunction && location.kind != SyntaxKind::FunctionExpression
    {
        return is_type_query_node(location)
            || ((is_function_like_declaration(location)
                || (location.kind == SyntaxKind::PropertyDeclaration && !is_static(location)))
                && last_location
                    .map(|l| !ptr_eq_name(l, location.name()))
                    .unwrap_or(true));
    }
    if let Some(last) = last_location {
        if ptr_eq_name(last, location.name()) {
            return false;
        }
    }

    false
}

pub fn is_type_parameter_symbol_declared_in_container(
    symbol: &Arc<Symbol>,
    container: &Arc<Node>,
) -> bool {
    for decl in &symbol.declarations {
        if decl.kind == SyntaxKind::TypeParameter {
            if let Some(parent) = &decl.parent {
                if Arc::ptr_eq(parent, container) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn is_self_reference_location(node: &Arc<Node>, last_location: Option<&Arc<Node>>) -> bool {
    match node.kind {
        SyntaxKind::Parameter => last_location
            .map(|l| ptr_eq_name(l, node.name()))
            .unwrap_or(false),
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::ModuleDeclaration => true,
        _ => false,
    }
}

pub(crate) fn is_const_assertion(_node: &Arc<Node>) -> bool {
    false
}

pub(crate) fn is_global_source_file(_node: &Arc<Node>) -> bool {
    false
}

pub(crate) fn is_type_query_node(_node: &Arc<Node>) -> bool {
    false
}

pub(crate) fn is_require_call(
    _node: &Arc<Node>,
    _require_string_literal_like_argument: bool,
) -> bool {
    false
}

pub(crate) fn node_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

pub(crate) fn node_local_symbol(_node: &Arc<Node>) -> Option<Arc<Symbol>> {
    None
}

pub(crate) fn ptr_eq_name(node: &Arc<Node>, name: Option<&Arc<Node>>) -> bool {
    name.map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
}
