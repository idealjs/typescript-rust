//! Completed-node detection.
//!
//! Ported from `internal/ls/lsutil/completednode.go`. Determines whether a node
//! is syntactically complete (e.g. a block is closed, an expression is
//! finished). The full decision tree requires AST child accessors and the
//! scanner; the public entry points are ported with stubbed bodies.

use std::sync::Arc;

use crate::ast::{Node, SourceFile, SyntaxKind};

/// Returns true if the position belongs to the node.
///
/// Assumes `candidate.pos() <= position` holds.
///
/// Mirrors `PositionBelongsToNode` in Go.
pub fn position_belongs_to_node(candidate: &Arc<Node>, position: usize, file: &SourceFile) -> bool {
    assert!(
        candidate.pos() <= position,
        "Expected candidate.pos <= position"
    );
    position < candidate.end() || !is_completed_node(candidate, file)
}

/// Whether `node` is syntactically complete.
///
/// Mirrors `IsCompletedNode` in Go. The full decision tree walks AST children
/// and scans trivia; this is stubbed to a conservative default until the AST
/// child accessors and scanner are ported.
pub fn is_completed_node(_node: &Arc<Node>, _source_file: &SourceFile) -> bool {
    // TODO: port the full switch over node kind (ClassDeclaration,
    // InterfaceDeclaration, CatchClause, NewExpression, function-likes, etc.),
    // which requires nodeEndsWith / hasChildOfKind / scanner.
    true
}

/// Checks if node ends with `expected_last_token`.
///
/// If the child at position `length - 1` is a `SemicolonToken` it is skipped and
/// `expected_last_token` is compared with the child at position `length - 2`.
///
/// Mirrors `nodeEndsWith` in Go. Stubbed until scanner/token creation is ported.
pub fn node_ends_with(
    _n: &Arc<Node>,
    _expected_last_token: SyntaxKind,
    _source_file: &SourceFile,
) -> bool {
    // TODO: requires get_last_visited_child + scanner.GetScannerForSourceFile +
    // SourceFile.GetOrCreateToken.
    false
}

/// Whether `containing_node` has a child of the given `kind`.
///
/// Mirrors `hasChildOfKind` in Go.
pub fn has_child_of_kind(
    _containing_node: &Arc<Node>,
    _kind: SyntaxKind,
    _source_file: &SourceFile,
) -> bool {
    // TODO: requires astnav.FindChildOfKind.
    false
}
