//! Shared utilities (1:1 port of Go's `internal/ls/utilities.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};
use crate::checker::Checker;
use crate::core::text::TextRange;

/// Check if a position is inside a string literal.
///
/// Mirrors `IsInString`.
pub fn is_in_string(
    _source_file: &Arc<SourceFile>,
    _position: usize,
    _previous_token: Option<&Arc<Node>>,
) -> bool {
    // TODO: requires astnav.GetStartOfNode + ast.IsStringTextContainingNode
    false
}

/// Check if a node is a module specifier-like string.
///
/// Mirrors `isModuleSpecifierLike`.
pub fn is_module_specifier_like(_node: &Arc<Node>) -> bool {
    // TODO: requires AST parent-kind checks
    false
}

/// Get the non-module symbol of a merged module symbol.
///
/// Mirrors `getNonModuleSymbolOfMergedModuleSymbol`.
pub fn get_non_module_symbol_of_merged_module_symbol(_symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
    // TODO: requires symbol.Declarations traversal
    None
}

/// Check if a position belongs to a node.
///
/// Mirrors `positionBelongsToNode`.
pub fn position_belongs_to_node(
    _candidate: &Arc<Node>,
    _position: usize,
    _file: &Arc<SourceFile>,
) -> bool {
    // TODO: requires lsutil.PositionBelongsToNode
    false
}

/// Check if a position is inside a comment.
///
/// Mirrors `isInComment`.
pub fn is_in_comment(
    _file: &Arc<SourceFile>,
    _position: usize,
    _token_at_position: Option<&Arc<Node>>,
) -> Option<crate::scanner::CommentRange> {
    // TODO: requires getRangeOfEnclosingComment
    None
}

/// Get the container node for a given node (used by hover/quickInfo).
///
/// Mirrors `getContainerNode`.
pub fn get_container_node(_node: &Arc<Node>) -> Option<Arc<Node>> {
    // TODO: requires AST ancestor traversal
    None
}

/// Get the meaning (value/type/namespace) from a node's location.
///
/// Mirrors `getMeaningFromLocation`.
pub fn get_meaning_from_location(_node: &Arc<Node>) -> u32 {
    // TODO: requires AST context analysis
    0
}

/// Get the containing object-literal element for a node.
///
/// Mirrors `getContainingObjectLiteralElement`.
pub fn get_containing_object_literal_element(_node: &Arc<Node>) -> Option<Arc<Node>> {
    // TODO: requires AST traversal
    None
}

/// Create a text range from a node.
///
/// Mirrors `createRangeFromNode`.
pub fn create_range_from_node(_node: &Arc<Node>, _file: &Arc<SourceFile>) -> TextRange {
    // TODO: requires scanner.GetTokenPosOfNode
    TextRange::default()
}

/// Get children from a non-JSDoc node.
///
/// Mirrors `getChildrenFromNonJSDocNode`.
pub fn get_children_from_non_jsdoc_node(
    _node: &Arc<Node>,
    _file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    // TODO: requires scanner child enumeration
    Vec::new()
}

/// Get line end position.
pub fn get_line_end_of_position(_file: &Arc<SourceFile>, _position: usize) -> usize {
    // TODO: requires scanner line-start computation
    0
}

/// Get leading comment ranges of a node.
pub fn get_leading_comment_ranges_of_node(
    _node: &Arc<Node>,
    _file: &Arc<SourceFile>,
) -> Vec<crate::scanner::CommentRange> {
    // TODO: requires scanner.GetLeadingCommentRanges
    Vec::new()
}

/// Get declarations from a location (identifier node) using a checker.
///
/// Mirrors `getDeclarationsFromLocation`.
pub fn get_declarations_from_location(_checker: &Checker, _node: &Arc<Node>) -> Vec<Arc<Node>> {
    Vec::new()
}
