use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    IndexedAccessTypeNodeData, LiteralTypeNodeData, NamedTupleMemberData, TypeReferenceNodeData,
};
use tsox_frontend::ast::{ModifierFlags, Node, NodeData, SyntaxKind};

use crate::checker::types_type_id::{TypeFlags, TYPE_FLAGS_BIG_INT_LIKE};

use tsox_frontend::ast::NodeFlags;

pub const SYMBOL_FLAGS_ALL: tsox_frontend::ast::SymbolFlags = tsox_frontend::ast::SymbolFlags::VALUE
    .union(tsox_frontend::ast::SymbolFlags::TYPE)
    .union(tsox_frontend::ast::SymbolFlags::NAMESPACE)
    .union(tsox_frontend::ast::SymbolFlags::ACCESSOR)
    .union(tsox_frontend::ast::SymbolFlags::Alias)
    .union(tsox_frontend::ast::SymbolFlags::TypeParameter)
    .union(tsox_frontend::ast::SymbolFlags::Method)
    .union(tsox_frontend::ast::SymbolFlags::Constructor)
    .union(tsox_frontend::ast::SymbolFlags::Signature);

impl TypeFlags {
    pub const BIGINT_LIKE: Self = TYPE_FLAGS_BIG_INT_LIKE;
    pub const VOID_LIKE: Self = Self::Void;
    pub const TYPE_PARAMETER: Self = Self::TypeParameter;
}

pub trait R20k2NodeExt {
    fn as_literal_type_node(&self) -> &LiteralTypeNodeData;
    fn as_type_reference_node(&self) -> &TypeReferenceNodeData;
    fn as_indexed_access_type_node(&self) -> &IndexedAccessTypeNodeData;
    fn as_named_tuple_member(&self) -> &NamedTupleMemberData;
    fn is_static(&self) -> bool;
    fn is_private_identifier_class_element_declaration(&self) -> bool;
    fn is_initialized_property(&self) -> bool;
    fn members(&self) -> Option<&tsox_frontend::ast::NodeList>;
    fn decorators(&self) -> Vec<&Arc<Node>>;
    fn property_name_or_name(&self) -> &Node;
    fn is_in_js_file(&self) -> bool;
    fn is_import_or_export_specifier(&self) -> bool;
}

pub fn is_in_js_file(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_in_js_file"); 
    node.flags.contains(NodeFlags::JavaScriptFile)
}

pub fn is_static(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_static"); 
    node.has_syntactic_modifier(ModifierFlags::Static)
}

pub fn has_syntactic_modifier(node: &Node, flags: ModifierFlags) -> bool { ::tsox_core::fntrace::enter("has_syntactic_modifier"); 
    node.has_syntactic_modifier(flags)
}

pub fn get_source_file_of_node(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_source_file_of_node"); 
    let mut current = Arc::clone(node);
    while let Some(parent) = current.parent() {
        current = parent;
    }
    Some(current)
}

pub fn node_is_missing(node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("node_is_missing"); 
    match node {
        None => true,
        Some(n) => n.pos() == n.end() && n.kind == SyntaxKind::EndOfFile,
    }
}

macro_rules! r20k2_as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl R20k2NodeExt for Node {
    r20k2_as_data!(as_literal_type_node, LiteralTypeNode, LiteralTypeNodeData);
    r20k2_as_data!(
        as_type_reference_node,
        TypeReferenceNode,
        TypeReferenceNodeData
    );
    r20k2_as_data!(
        as_indexed_access_type_node,
        IndexedAccessTypeNode,
        IndexedAccessTypeNodeData
    );
    r20k2_as_data!(as_named_tuple_member, NamedTupleMember, NamedTupleMemberData);

    fn is_static(&self) -> bool { ::tsox_core::fntrace::enter("is_static"); 
        self.has_syntactic_modifier(ModifierFlags::Static)
    }

    fn is_in_js_file(&self) -> bool { ::tsox_core::fntrace::enter("is_in_js_file"); 
        self.flags.contains(NodeFlags::JavaScriptFile)
    }

    fn is_import_or_export_specifier(&self) -> bool { ::tsox_core::fntrace::enter("is_import_or_export_specifier"); 
        matches!(self.kind, SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier)
    }

    fn is_private_identifier_class_element_declaration(&self) -> bool { ::tsox_core::fntrace::enter("is_private_identifier_class_element_declaration"); 
        self.name().is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
    }

    fn is_initialized_property(&self) -> bool { ::tsox_core::fntrace::enter("is_initialized_property"); 
        match &self.data {
            NodeData::PropertyDeclaration(d) => d.initializer.is_some(),
            _ => false,
        }
    }

    fn members(&self) -> Option<&tsox_frontend::ast::NodeList> { ::tsox_core::fntrace::enter("members"); 
        match &self.data {
            NodeData::ClassDeclaration(d) => Some(&d.members),
            NodeData::ClassExpression(d) => Some(&d.members),
            NodeData::InterfaceDeclaration(d) => Some(&d.members),
            _ => None,
        }
    }

    fn decorators(&self) -> Vec<&Arc<Node>> { ::tsox_core::fntrace::enter("decorators"); 
        self.modifier_nodes()
            .iter()
            .filter(|m| m.kind == SyntaxKind::Decorator)
            .collect()
    }

    fn property_name_or_name(&self) -> &Node { ::tsox_core::fntrace::enter("property_name_or_name"); 
        match &self.data {
            NodeData::ImportSpecifier(d) => d.property_name.as_deref().unwrap_or(&d.name),
            NodeData::ExportSpecifier(d) => d.property_name.as_deref().unwrap_or(&d.name),
            NodeData::ShorthandPropertyAssignment(d) => &d.name,
            _ => self,
        }
    }
}
