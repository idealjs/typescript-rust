#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::Symbol;

pub fn is_in_string(
    _source_file: &Arc<SourceFile>,
    _position: usize,
    _previous_token: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("is_in_string"); 
    false
}

pub fn is_module_specifier_like(_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_module_specifier_like"); 
    false
}

pub fn get_non_module_symbol_of_merged_module_symbol(_symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_non_module_symbol_of_merged_module_symbol"); 
    None
}

pub fn position_belongs_to_node(
    _candidate: &Arc<Node>,
    _position: usize,
    _file: &Arc<SourceFile>,
) -> bool { ::tsox_core::fntrace::enter("position_belongs_to_node"); 
    false
}

pub fn is_in_comment(
    _file: &Arc<SourceFile>,
    _position: usize,
    _token_at_position: Option<&Arc<Node>>,
) -> Option<tsox_frontend::scanner::CommentRange> { ::tsox_core::fntrace::enter("is_in_comment"); 
    None
}

pub fn get_container_node(_node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_container_node"); 
    None
}

pub fn get_meaning_from_location(_node: &Arc<Node>) -> u32 { ::tsox_core::fntrace::enter("get_meaning_from_location"); 
    0
}

pub fn get_containing_object_literal_element(_node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_containing_object_literal_element"); 
    None
}

pub fn create_range_from_node(_node: &Arc<Node>, _file: &Arc<SourceFile>) -> TextRange { ::tsox_core::fntrace::enter("create_range_from_node"); 
    TextRange::default()
}

pub fn get_children_from_non_jsdoc_node(
    _node: &Arc<Node>,
    _file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_children_from_non_jsdoc_node"); 
    Vec::new()
}

pub fn get_line_end_of_position(_file: &Arc<SourceFile>, _position: usize) -> usize { ::tsox_core::fntrace::enter("get_line_end_of_position"); 
    0
}

pub fn get_leading_comment_ranges_of_node(
    _node: &Arc<Node>,
    _file: &Arc<SourceFile>,
) -> Vec<tsox_frontend::scanner::CommentRange> { ::tsox_core::fntrace::enter("get_leading_comment_ranges_of_node"); 
    Vec::new()
}

pub fn get_declarations_from_location(_checker: &Checker, _node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_declarations_from_location"); 
    Vec::new()
}
