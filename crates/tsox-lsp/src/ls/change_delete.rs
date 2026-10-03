#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;

use crate::ls::change_tracker::LeadingTriviaOption;
use crate::ls::change_tracker::Tracker;
use crate::ls::change_tracker::TrailingTriviaOption;

pub fn delete_declaration(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) { ::tsox_core::fntrace::enter("delete_declaration"); 
}

pub fn delete_default_import(
    _t: &mut Tracker,
    _source_file: &SourceFile,
    _import_clause: &Arc<Node>,
) { ::tsox_core::fntrace::enter("delete_default_import"); 
    todo!("deleteDefaultImport")
}

pub fn delete_import_binding(_t: &mut Tracker, _source_file: &SourceFile, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("delete_import_binding"); 
    todo!("deleteImportBinding")
}

pub fn delete_variable_declaration(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) { ::tsox_core::fntrace::enter("delete_variable_declaration"); 
    todo!("deleteVariableDeclaration")
}

pub fn delete_node(
    _t: &mut Tracker,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
    _leading_trivia: LeadingTriviaOption,
    _trailing_trivia: TrailingTriviaOption,
) { ::tsox_core::fntrace::enter("delete_node"); 
    todo!("deleteNode")
}

pub fn delete_node_in_list(
    _t: &mut Tracker,
    _deleted_nodes_in_lists: &mut HashSet<u64>,
    _source_file: &SourceFile,
    _node: &Arc<Node>,
) { ::tsox_core::fntrace::enter("delete_node_in_list"); 
    todo!("deleteNodeInList")
}

pub fn positions_are_on_same_line(_pos1: usize, _pos2: usize, _source_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("positions_are_on_same_line"); 
    true
}

pub fn has_jsdoc_nodes(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_jsdoc_nodes"); 
    let _ = node;
    false
}
