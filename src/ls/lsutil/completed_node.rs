use std::sync::Arc;

use crate::ast::{Node, SourceFile, SyntaxKind};

pub fn position_belongs_to_node(candidate: &Arc<Node>, position: usize, file: &SourceFile) -> bool {
    assert!(
        candidate.pos() <= position,
        "Expected candidate.pos <= position"
    );
    position < candidate.end() || !is_completed_node(candidate, file)
}

pub fn is_completed_node(_node: &Arc<Node>, _source_file: &SourceFile) -> bool {

    true
}

pub fn node_ends_with(
    _n: &Arc<Node>,
    _expected_last_token: SyntaxKind,
    _source_file: &SourceFile,
) -> bool {

    false
}

pub fn has_child_of_kind(
    _containing_node: &Arc<Node>,
    _kind: SyntaxKind,
    _source_file: &SourceFile,
) -> bool {

    false
}
