//! Deletion edit utilities.
//!
//! Ported from `internal/ls/change/delete.go`. These free functions operate on
//! a [`Tracker`] and handle special deletion cases (import specifiers in lists,
//! parameters, default imports, variable declarations, etc.). They depend on
//! AST child accessors and the scanner; bodies are stubbed (`todo!()`).

#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use crate::ast::{Node, SourceFile};

use super::tracker::{LeadingTriviaOption, Tracker, TrailingTriviaOption};

/// Deletes a node with smart handling for different node types.
///
/// Mirrors `deleteDeclaration` in Go.
pub fn delete_declaration(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) {
    // TODO: port the full switch over node kind.
}

/// Deletes a default import, preserving any namespace/named bindings.
///
/// Mirrors `deleteDefaultImport` in Go.
pub fn delete_default_import(
    _t: &mut Tracker,
    _source_file: &SourceFile,
    _import_clause: &Arc<Node>,
) {
    todo!("deleteDefaultImport")
}

/// Deletes an entire import binding (namespace or named imports).
///
/// Mirrors `deleteImportBinding` in Go.
pub fn delete_import_binding(_t: &mut Tracker, _source_file: &SourceFile, _node: &Arc<Node>) {
    todo!("deleteImportBinding")
}

/// Deletes a variable declaration, handling single/multi-declaration lists.
///
/// Mirrors `deleteVariableDeclaration` in Go.
pub fn delete_variable_declaration(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) {
    todo!("deleteVariableDeclaration")
}

/// Deletes a node with the specified trivia options.
///
/// Mirrors `deleteNode` in Go.
pub fn delete_node(
    _t: &mut Tracker,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
    _leading_trivia: LeadingTriviaOption,
    _trailing_trivia: TrailingTriviaOption,
) {
    todo!("deleteNode")
}

/// Deletes a node that is an element of a delimited list.
///
/// Mirrors `deleteNodeInList` in Go.
pub fn delete_node_in_list(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) {
    todo!("deleteNodeInList")
}

/// Whether two positions are on the same line in `source_file`.
///
/// Mirrors `positionsAreOnSameLine` in Go.
pub fn positions_are_on_same_line(_pos1: usize, _pos2: usize, _source_file: &SourceFile) -> bool {
    // TODO: requires format.GetLineStartPositionForPosition.
    true
}

/// Whether `node` has JSDoc comments.
///
/// Mirrors `hasJSDocNodes` in Go.
pub fn has_jsdoc_nodes(node: &Arc<Node>) -> bool {
    // TODO: requires Node.JSDoc(file) which needs a SourceFile; the Go version
    // passes nil and checks len(jsdocs) > 0.
    let _ = node;
    false
}
