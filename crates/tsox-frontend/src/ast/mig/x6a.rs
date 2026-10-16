#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::node_flags::NodeFlags;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities::*;
use crate::ast::*;
use super::m3g_2::{
    is_property_access_entity_name_expression, is_type_only_import_or_export_declaration,
};
use super::m3h::{is_common_js_containing_module_kind, is_element_access_entity_name_expression};
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options::ModuleKind;

fn is_js_type_alias_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_js_type_alias_declaration"); 
    node.kind == SyntaxKind::JSTypeAliasDeclaration
}

pub fn is_entity_name_expression_ex(node: &Node, allow_js: bool) -> bool { ::tsox_core::fntrace::enter("is_entity_name_expression_ex"); 
    is_identifier(node)
        || is_property_access_entity_name_expression(node, allow_js)
        || (allow_js
            && (node.kind == SyntaxKind::ThisKeyword
                || is_element_access_entity_name_expression(node, allow_js)))
}

pub fn is_any_export_assignment(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_any_export_assignment"); 
    node.kind == SyntaxKind::ExportAssignment
}

pub fn is_accessor_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_accessor_declaration"); 
    matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
}

pub fn is_effective_external_module(file: &crate::ast::node_source_file::SourceFile, compiler_options: &CompilerOptions) -> bool { ::tsox_core::fntrace::enter("is_effective_external_module"); 
    is_external_module(file)
        || (is_common_js_containing_module_kind(compiler_options.get_emit_module_kind())
            && file.common_js_module_indicator.is_some())
}

pub fn is_enum_const(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_enum_const"); 
    let mut flags = node.syntactic_modifier_flags();
    let mut current = node.parent();
    while let Some(p) = current {
        flags |= p.syntactic_modifier_flags();
        current = p.parent();
    }
    flags.intersects(crate::ast::node_flags::ModifierFlags::Const)
}

pub fn is_exclusively_type_only_import_or_export(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_exclusively_type_only_import_or_export"); 
    match node.kind {
        SyntaxKind::ExportDeclaration => match &node.data {
            crate::ast::node_data_generated::NodeData::ExportDeclaration(d) => d.is_type_only,
            _ => false,
        },
        SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => match &node.data {
            crate::ast::node_data_generated::NodeData::ImportDeclaration(d) => d
                .import_clause
                .as_ref()
                .is_some_and(|clause| is_type_only_import_or_export_declaration(clause)),
            _ => false,
        },
        SyntaxKind::JSDocImportTag => match &node.data {
            crate::ast::node_data_generated::NodeData::JSDocImportTag(d) => d
                .import_clause
                .as_ref()
                .is_some_and(|clause| is_type_only_import_or_export_declaration(clause)),
            _ => false,
        },
        _ => false,
    }
}

pub fn is_implicitly_exported_jsdoc_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_implicitly_exported_jsdoc_declaration"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    if !is_source_file(&parent) {
        return false;
    }
    if is_js_type_alias_declaration(node) {
        return true;
    }
    is_module_declaration(node) && node.flags.intersects(NodeFlags::Reparsed)
}
