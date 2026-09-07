use super::namespace::{
    create_export_modifier, get_innermost_name_of_jsdoc_namespace, wrap_in_jsdoc_namespace,
};
use super::signature::reparse_jsdoc_signature;
use super::type_literal::reparse_jsdoc_type_literal;
use super::type_parameters::gather_type_parameters;
use crate::ast::*;
use std::sync::Arc;

pub fn reparse_tags(parent: &Arc<Node>, js_docs: &[Arc<Node>]) -> Vec<Arc<Node>> {
    let mut reparse_list: Vec<Arc<Node>> = Vec::new();

    for (i, js_doc) in js_docs.iter().enumerate() {
        let is_last = i == js_docs.len() - 1;
        let tags = match &js_doc.data {
            NodeData::JSDoc(d) => d.tags.as_ref(),
            _ => continue,
        };
        let Some(tags) = tags else {
            continue;
        };

        for tag in &tags.nodes {
            if let Some(stmt) = reparse_unhosted(tag, parent, js_doc) {
                reparse_list.push(stmt);
            }

            let _ = is_last;
        }
    }

    reparse_list
}

fn reparse_unhosted(tag: &Arc<Node>, parent: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    match tag.kind {
        SyntaxKind::JSDocTypedefTag => reparse_typedef_tag(tag, js_doc),
        SyntaxKind::JSDocCallbackTag => reparse_callback_tag(tag, js_doc),
        SyntaxKind::JSDocImportTag => reparse_import_tag(tag),
        SyntaxKind::JSDocOverloadTag => reparse_overload_tag(tag, parent, js_doc),
        _ => None,
    }
}

fn reparse_typedef_tag(tag: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    let (type_expression, full_name) = match &tag.data {
        NodeData::JSDocTypedefTag(d) => {
            let te = d.type_expression.as_ref()?;
            let name = d.name.as_ref()?;
            (te.clone(), name.clone())
        }
        _ => return None,
    };

    let is_namespace = full_name.kind == SyntaxKind::ModuleDeclaration;
    let modifiers = if is_namespace {
        Some(create_export_modifier(&full_name))
    } else {
        None
    };

    let inner_name = get_innermost_name_of_jsdoc_namespace(&full_name);
    let type_parameters = gather_type_parameters(js_doc, true);

    let type_node = match type_expression.kind {
        SyntaxKind::JSDocTypeExpression => match &type_expression.data {
            NodeData::JSDocTypeExpression(d) => d.type_node.clone(),
            _ => return None,
        },
        SyntaxKind::JSDocTypeLiteral => reparse_jsdoc_type_literal(&type_expression),
        _ => return None,
    };

    let type_alias = Arc::new(Node::with_loc_flags(
        SyntaxKind::TypeAliasDeclaration,
        NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
            modifiers,
            name: inner_name,
            type_parameters,
            type_node,
        }),
        tag.loc,
        NodeFlags::Reparsed | NodeFlags::HasJSDoc,
    ));

    let result = wrap_in_jsdoc_namespace(&full_name, &type_alias, false);
    Some(result)
}

fn reparse_callback_tag(tag: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    let (type_expression, full_name) = match &tag.data {
        NodeData::JSDocCallbackTag(d) => {
            let name = d.name.as_ref()?;
            (d.type_expression.clone(), name.clone())
        }
        _ => return None,
    };

    let is_namespace = full_name.kind == SyntaxKind::ModuleDeclaration;
    let modifiers = if is_namespace {
        Some(create_export_modifier(&full_name))
    } else {
        None
    };

    let inner_name = get_innermost_name_of_jsdoc_namespace(&full_name);
    let type_parameters = gather_type_parameters(js_doc, true);

    let function_type = reparse_jsdoc_signature(&type_expression, tag, js_doc, tag, None);

    let type_alias = Arc::new(Node::with_loc_flags(
        SyntaxKind::TypeAliasDeclaration,
        NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
            modifiers,
            name: inner_name,
            type_parameters,
            type_node: function_type,
        }),
        tag.loc,
        NodeFlags::Reparsed | NodeFlags::HasJSDoc,
    ));

    let result = wrap_in_jsdoc_namespace(&full_name, &type_alias, false);
    Some(result)
}

fn reparse_import_tag(tag: &Arc<Node>) -> Option<Arc<Node>> {
    let (import_clause, module_specifier, attributes) = match &tag.data {
        NodeData::JSDocImportTag(d) => {
            let clause = d.import_clause.as_ref()?;
            (
                clause.clone(),
                d.module_specifier.clone(),
                d.attributes.clone(),
            )
        }
        _ => return None,
    };

    let import_clause = match &import_clause.data {
        NodeData::ImportClause(d) => Arc::new(Node::with_loc_flags(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier: Some(SyntaxKind::TypeKeyword),
                name: d.name.clone(),
                named_bindings: d.named_bindings.clone(),
            }),
            import_clause.loc,
            NodeFlags::Reparsed,
        )),
        _ => import_clause.clone(),
    };

    let import_declaration = Arc::new(Node::with_loc_flags(
        SyntaxKind::ImportDeclaration,
        NodeData::ImportDeclaration(ImportDeclarationData {
            modifiers: None,
            import_clause: Some(import_clause),
            module_specifier,
            attributes,
        }),
        tag.loc,
        NodeFlags::Reparsed,
    ));

    Some(import_declaration)
}

fn reparse_overload_tag(
    tag: &Arc<Node>,
    parent: &Arc<Node>,
    js_doc: &Arc<Node>,
) -> Option<Arc<Node>> {
    let is_valid_parent = matches!(
        parent.kind,
        SyntaxKind::FunctionDeclaration | SyntaxKind::MethodDeclaration | SyntaxKind::Constructor
    );
    if !is_valid_parent {
        return None;
    }

    let type_expression = match &tag.data {
        NodeData::JSDocOverloadTag(d) => &d.type_expression,
        _ => return None,
    };

    let modifiers = parent.modifiers().cloned();
    let signature = reparse_jsdoc_signature(type_expression, parent, js_doc, tag, modifiers);
    Some(signature)
}
