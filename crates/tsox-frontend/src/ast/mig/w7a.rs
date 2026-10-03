use super::m3g_2::is_optional_chain_root;
use super::m3g_3::module_export_name_is_default;
use super::m3g_3::try_get_class_extending_expression_with_type_arguments;
use super::m3h::has_comment;
use crate::ast::*;
use std::sync::Arc;

fn comment_list(node: &Node) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("comment_list"); 
    match &node.data {
        NodeData::JSDoc(d) => Some(&d.comment),
        NodeData::JSDocUnknownTag(d) => d.comment.as_ref(),
        NodeData::JSDocAugmentsTag(d) => d.comment.as_ref(),
        NodeData::JSDocImplementsTag(d) => d.comment.as_ref(),
        NodeData::JSDocDeprecatedTag(d) => d.comment.as_ref(),
        NodeData::JSDocPublicTag(d) => d.comment.as_ref(),
        NodeData::JSDocPrivateTag(d) => d.comment.as_ref(),
        NodeData::JSDocProtectedTag(d) => d.comment.as_ref(),
        NodeData::JSDocReadonlyTag(d) => d.comment.as_ref(),
        NodeData::JSDocOverrideTag(d) => d.comment.as_ref(),
        NodeData::JSDocSeeTag(d) => d.comment.as_ref(),
        NodeData::JSDocSatisfiesTag(d) => d.comment.as_ref(),
        NodeData::JSDocThrowsTag(d) => d.comment.as_ref(),
        NodeData::JSDocTypeTag(d) => d.comment.as_ref(),
        NodeData::JSDocReturnTag(d) => d.comment.as_ref(),
        NodeData::JSDocThisTag(d) => d.comment.as_ref(),
        NodeData::JSDocTypeExpression(_) => None,
        NodeData::JSDocTemplateTag(d) => d.comment.as_ref(),
        NodeData::JSDocTypedefTag(d) => d.comment.as_ref(),
        NodeData::JSDocCallbackTag(d) => d.comment.as_ref(),
        NodeData::JSDocOverloadTag(d) => d.comment.as_ref(),
        NodeData::JSDocImportTag(d) => d.comment.as_ref(),
        NodeData::JSDocParameterOrPropertyTag(d) => d.comment.as_ref(),
        _ => None,
    }
}

pub fn is_expression_of_optional_chain_root(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_expression_of_optional_chain_root"); 
    node.parent().is_some_and(|parent| {
        is_optional_chain_root(&parent)
            && parent
                .expression()
                .is_some_and(|e| std::ptr::eq(Arc::as_ptr(e), node))
    })
}

pub fn is_jsdoc_type_assertion(node: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_type_assertion"); 
    let Some(node) = node else {
        return false;
    };
    if !is_parenthesized_expression(node) || !is_in_js_file(node) {
        return false;
    }
    let Some(expr) = node.expression() else {
        return false;
    };
    is_as_expression(&expr) && expr.type_node().is_some_and(|t| t.flags.intersects(NodeFlags::Reparsed))
}

pub fn is_function_or_module_block(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_function_or_module_block"); 
    is_source_file(node)
        || is_module_block(node)
        || (is_block(node) && node.parent().is_some_and(|p| is_function_like(&p)))
}

pub fn is_expression_with_type_arguments_in_class_extends_clause(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_expression_with_type_arguments_in_class_extends_clause"); 
    try_get_class_extending_expression_with_type_arguments(node).is_some()
}

pub fn is_external_module_import_equals_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_external_module_import_equals_declaration"); 
    node.kind == SyntaxKind::ImportEqualsDeclaration
        && matches!(
            &node.data,
            NodeData::ImportEqualsDeclaration(d) if d.module_reference.kind == SyntaxKind::ExternalModuleReference
        )
}

pub fn is_external_module_indicator(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_external_module_indicator"); 
    is_any_import_or_re_export(node)
        || is_export_assignment(node)
        || has_syntactic_modifier(node, ModifierFlags::Export)
}

pub fn is_export_namespace_as_default_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_export_namespace_as_default_declaration"); 
    if is_export_declaration(node) {
        if let NodeData::ExportDeclaration(decl) = &node.data {
            if let Some(export_clause) = &decl.export_clause {
                return is_namespace_export(export_clause)
                    && export_clause
                        .name()
                        .is_some_and(|n| module_export_name_is_default(&n));
            }
        }
    }
    false
}

pub fn is_jsdoc_name_reference_context(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_name_reference_context"); 
    node.flags.intersects(NodeFlags::JSDoc)
        && find_ancestor(node, |n| is_jsdoc_name_reference(n) || is_jsdoc_link_like(n)).is_some()
}

pub fn is_jsdoc_single_comment_node_list(node_list: Option<&NodeList>) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_single_comment_node_list"); 
    let Some(node_list) = node_list else {
        return false;
    };
    if node_list.nodes.is_empty() {
        return false;
    }
    let Some(parent) = node_list.nodes[0].parent() else {
        return false;
    };
    is_jsdoc_single_comment_node(&parent)
        && comment_list(&parent)
            .is_some_and(|l| std::ptr::eq(Arc::as_ptr(l), node_list))
}

pub fn is_jsdoc_single_comment_node_comment(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_single_comment_node_comment"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    is_jsdoc_single_comment_node(&parent)
        && comment_list(&parent)
            .and_then(|l| l.nodes.first())
            .is_some_and(|first| std::ptr::eq(Arc::as_ptr(first), node))
}

pub fn is_jsdoc_single_comment_node(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_single_comment_node"); 
    has_comment(node.kind)
        && comment_list(node).is_some_and(|l| l.nodes.len() == 1)
}

pub fn is_import_or_import_equals_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_import_or_import_equals_declaration"); 
    is_import_declaration(node) || is_import_equals_declaration(node)
}

pub fn is_expando_initializer(declaration: &Node, initializer: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("is_expando_initializer"); 
    let Some(initializer) = initializer else {
        return false;
    };
    if is_function_expression_or_arrow_function(initializer) {
        return true;
    }
    if is_in_js_file(initializer) {
        let no_properties = match &initializer.data {
            NodeData::ObjectLiteralExpression(d) => d.properties.nodes.is_empty(),
            _ => true,
        };
        return is_class_expression(initializer)
            || (is_object_literal_expression(initializer)
                && no_properties
                && declaration.type_node().is_none());
    }
    false
}

fn is_js_type_alias_declaration(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_js_type_alias_declaration"); 
    node.kind == SyntaxKind::JSTypeAliasDeclaration
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

pub fn is_expando_property_declaration(node: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("is_expando_property_declaration"); 
    node.is_some_and(|n| is_binary_expression(n))
}
