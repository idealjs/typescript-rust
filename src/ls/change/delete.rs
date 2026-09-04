#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use crate::ast::{Node, SourceFile};

use super::tracker::{LeadingTriviaOption, Tracker, TrailingTriviaOption};

pub fn delete_declaration(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) {

}

pub fn delete_default_import(
    _t: &mut Tracker,
    _source_file: &SourceFile,
    _import_clause: &Arc<Node>,
) {
    todo!("deleteDefaultImport")
}

pub fn delete_import_binding(_t: &mut Tracker, _source_file: &SourceFile, _node: &Arc<Node>) {
    todo!("deleteImportBinding")
}

pub fn delete_variable_declaration(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) {
    todo!("deleteVariableDeclaration")
}

pub fn delete_node(
    _t: &mut Tracker,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
    _leading_trivia: LeadingTriviaOption,
    _trailing_trivia: TrailingTriviaOption,
) {
    todo!("deleteNode")
}

pub fn delete_node_in_list(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) {
    todo!("deleteNodeInList")
}

pub fn positions_are_on_same_line(_pos1: usize, _pos2: usize, _source_file: &SourceFile) -> bool {

    true
}

pub fn has_jsdoc_nodes(node: &Arc<Node>) -> bool {

    let _ = node;
    false
}
