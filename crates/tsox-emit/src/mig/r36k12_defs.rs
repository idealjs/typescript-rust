#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::{ModifierList, Node, NodeList};
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::format::mig::m4o::ListFormat;

pub fn is_type_only(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_type_only"); 
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
        NodeData::ImportSpecifier(d) => d.is_type_only,
        NodeData::ExportSpecifier(d) => d.is_type_only,
        NodeData::ExportDeclaration(d) => d.is_type_only,
        _ => false,
    }
}

pub fn is_export_equals(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_export_equals"); 
    match &node.data {
        NodeData::ExportAssignment(d) => d.is_export_equals,
        _ => false,
    }
}

pub fn phase_modifier(node: &Arc<Node>) -> Option<SyntaxKind> { ::tsox_core::fntrace::enter("phase_modifier"); 
    match &node.data {
        NodeData::ImportClause(d) => d.phase_modifier,
        _ => None,
    }
}

pub fn named_bindings(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("named_bindings"); 
    match &node.data {
        NodeData::ImportClause(d) => d.named_bindings.as_ref(),
        _ => None,
    }
}

pub fn import_clause(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("import_clause"); 
    match &node.data {
        NodeData::ImportDeclaration(d) => d.import_clause.as_ref(),
        _ => None,
    }
}

pub fn module_specifier(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("module_specifier"); 
    match &node.data {
        NodeData::ImportDeclaration(d) => Some(&d.module_specifier),
        NodeData::ExportDeclaration(d) => d.module_specifier.as_ref(),
        _ => None,
    }
}

pub fn export_clause(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("export_clause"); 
    match &node.data {
        NodeData::ExportDeclaration(d) => d.export_clause.as_ref(),
        _ => None,
    }
}

pub fn attributes(node: &Arc<Node>) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("attributes"); 
    match &node.data {
        NodeData::ImportDeclaration(d) => d.attributes.as_ref(),
        NodeData::ExportDeclaration(d) => d.attributes.as_ref(),
        _ => None,
    }
}

pub fn module_reference(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("module_reference"); 
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) => &d.module_reference,
        _ => panic!("unexpected ImportEqualsDeclaration: {:?}", node.kind),
    }
}

pub fn import_attributes_token(node: &Arc<Node>) -> SyntaxKind { ::tsox_core::fntrace::enter("import_attributes_token"); 
    match &node.data {
        NodeData::ImportAttributes(d) => d.token,
        _ => panic!("unexpected ImportAttributes: {:?}", node.kind),
    }
}

pub fn import_attributes_elements(node: &Arc<Node>) -> &Arc<NodeList> { ::tsox_core::fntrace::enter("import_attributes_elements"); 
    match &node.data {
        NodeData::ImportAttributes(d) => &d.attributes,
        _ => panic!("unexpected ImportAttributes: {:?}", node.kind),
    }
}

pub fn import_attribute_value(node: &Arc<Node>) -> &Arc<Node> { ::tsox_core::fntrace::enter("import_attribute_value"); 
    match &node.data {
        NodeData::ImportAttribute(d) => &d.value,
        _ => panic!("unexpected ImportAttribute: {:?}", node.kind),
    }
}

pub fn greatest_modifier_end(end: usize, modifiers: Option<&Arc<ModifierList>>) -> usize { ::tsox_core::fntrace::enter("greatest_modifier_end"); 
    match modifiers {
        Some(ml) if end < ml.end() => ml.end(),
        _ => end,
    }
}

pub fn greatest_opt_node_end(end: usize, node: Option<&Arc<Node>>) -> usize { ::tsox_core::fntrace::enter("greatest_opt_node_end"); 
    match node {
        Some(n) if end < n.end() => n.end(),
        _ => end,
    }
}

pub const LF_IMPORT_ATTRIBUTES: ListFormat = ListFormat(
    ListFormat::PRESERVE_LINES.0
        | ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::INDENTED.0
        | ListFormat::BRACES.0
        | ListFormat::NO_SPACE_IF_EMPTY.0,
);

pub const LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS: ListFormat = ListFormat(
    ListFormat::COMMA_DELIMITED.0
        | ListFormat::SPACE_BETWEEN_SIBLINGS.0
        | ListFormat::ALLOW_TRAILING_COMMA.0
        | ListFormat::SINGLE_ELEMENT.0
        | ListFormat::SPACE_BETWEEN_BRACES.0
        | ListFormat::NO_SPACE_IF_EMPTY.0,
);
