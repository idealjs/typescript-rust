//! Child-node iteration helpers.
//!
//! Ported from `internal/ls/lsutil/children.go`. These helpers locate the
//! first/last child or token of a node by scanning trivia, which requires the
//! scanner and the node-visitor infrastructure that are not yet ported. The
//! signatures are ported 1:1; bodies are stubbed.

use std::sync::Arc;

use crate::ast::{Node, SourceFile};

/// Replaces `last(node.getChildren(sourceFile))`.
///
/// Mirrors `GetLastChild` in Go.
pub fn get_last_child(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    // TODO: requires GetLastVisitedChild + scanner.GetScannerForSourceFile +
    // SourceFile.GetOrCreateToken.
    None
}

/// Returns the last token of `node`, descending into children as needed.
///
/// Mirrors `GetLastToken` in Go.
pub fn get_last_token(node: Option<&Arc<Node>>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    // TODO: requires get_last_child + AssertHasRealPosition.
    node.cloned()
}

/// Gets the last visited child of the given node (not including unvisited tokens).
///
/// Mirrors `GetLastVisitedChild` in Go.
pub fn get_last_visited_child(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    // TODO: requires astnav.VisitEachChildAndJSDoc.
    None
}

/// Returns the first token of `node`.
///
/// Mirrors `GetFirstToken` in Go.
pub fn get_first_token(_node: &Arc<Node>, _source_file: &SourceFile) -> Option<Arc<Node>> {
    // TODO: requires ForEachChild + scanner.GetScannerForSourceFile +
    // SourceFile.GetOrCreateToken.
    None
}

/// Asserts that `node` has a real (non-synthesized) source position.
///
/// Mirrors `AssertHasRealPosition` in Go. A position is synthesized when it is
/// negative (the AST uses `-1` for the "undefined" range).
pub fn assert_has_real_position(node: &Arc<Node>) {
    if node.loc.pos < 0 || node.loc.end < 0 {
        panic!("Node must have a real position for this operation.");
    }
}
