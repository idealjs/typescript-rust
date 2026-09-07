#![allow(unused_imports)]

use super::*;

pub fn is_reserved_member_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '\u{FE}' => match chars.next() {
            Some('@') | Some('#') => false,
            Some(_) => true,
            None => false,
        },
        _ => false,
    }
}

pub fn symbols_to_array(symbols: &SymbolTable) -> Vec<Arc<Symbol>> {
    symbols
        .entries
        .values()
        .filter(|s| !is_reserved_member_name(&s.name))
        .cloned()
        .collect()
}

pub fn introduces_arguments_exotic_object(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
    )
}

pub const KNOWN_GENERIC_TYPE_NAMES: &[&str] = &[
    "Array",
    "ArrayLike",
    "ReadonlyArray",
    "Promise",
    "PromiseLike",
    "Iterable",
    "IterableIterator",
    "AsyncIterable",
    "Set",
    "WeakSet",
    "ReadonlySet",
    "Map",
    "WeakMap",
    "ReadonlyMap",
    "Partial",
    "Required",
    "Readonly",
    "Pick",
    "Omit",
    "NonNullable",
];

pub(crate) fn is_known_generic_name(name: &str) -> bool {
    KNOWN_GENERIC_TYPE_NAMES.contains(&name)
}
