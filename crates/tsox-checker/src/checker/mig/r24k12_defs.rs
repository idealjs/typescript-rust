use crate::checker::symboltracker::NodeBuilderFlags;
use crate::checker::types_type_id::SymbolFormatFlags;
use std::sync::Arc;
use tsox_frontend::ast::node_data_generated::ClassDeclarationData;
use tsox_frontend::ast::{Node, NodeData};

pub fn symbol_format_flags_to_node_builder_flags(flags: SymbolFormatFlags) -> NodeBuilderFlags { ::tsox_core::fntrace::enter("symbol_format_flags_to_node_builder_flags"); 
    NodeBuilderFlags::from_bits_truncate(flags.bits())
}

pub fn as_class_declaration(node: &Arc<Node>) -> &ClassDeclarationData { ::tsox_core::fntrace::enter("as_class_declaration"); 
    match &node.data {
        NodeData::ClassDeclaration(d) => d,
        _ => panic!("AsClassDeclaration on wrong node kind"),
    }
}
