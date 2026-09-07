use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

pub(super) fn get_innermost_name_of_jsdoc_namespace(full_name: &Arc<Node>) -> Arc<Node> {
    let mut current = full_name.clone();
    while current.kind == SyntaxKind::ModuleDeclaration {
        let body = match &current.data {
            NodeData::ModuleDeclaration(d) => d.body.clone(),
            _ => break,
        };
        match body {
            Some(b) => current = b,
            None => {
                return current.name().map(|n| deep_clone(n)).unwrap_or_else(|| {
                    Arc::new(Node::with_loc(
                        SyntaxKind::Identifier,
                        NodeData::Identifier(IdentifierData {
                            text: String::new(),
                        }),
                        current.loc,
                    ))
                });
            }
        }
    }
    current
}

pub(super) fn wrap_in_jsdoc_namespace(
    full_name: &Arc<Node>,
    statement: &Arc<Node>,
    nested: bool,
) -> Arc<Node> {
    if full_name.kind != SyntaxKind::ModuleDeclaration {
        return statement.clone();
    }

    let (body, name) = match &full_name.data {
        NodeData::ModuleDeclaration(d) => (d.body.clone(), d.name.clone()),
        _ => return statement.clone(),
    };
    let loc = full_name.loc;

    let wrapped = match &body {
        Some(b) => wrap_in_jsdoc_namespace(b, statement, true),
        None => statement.clone(),
    };

    let block = Arc::new(Node::with_loc_flags(
        SyntaxKind::ModuleBlock,
        NodeData::ModuleBlock(ModuleBlockData {
            statements: Arc::new(NodeList::new(vec![wrapped])),
        }),
        loc,
        NodeFlags::Reparsed,
    ));

    let modifiers = if nested {
        Some(create_export_modifier(&full_name))
    } else {
        None
    };

    Arc::new(Node::with_loc_flags(
        SyntaxKind::ModuleDeclaration,
        NodeData::ModuleDeclaration(ModuleDeclarationData {
            modifiers,
            keyword: SyntaxKind::NamespaceKeyword,
            name: deep_clone(&name),
            body: Some(block),
        }),
        loc,
        NodeFlags::Reparsed,
    ))
}

pub(super) fn create_export_modifier(location_node: &Arc<Node>) -> Arc<ModifierList> {
    let export_modifier = Arc::new(Node::with_loc_flags(
        SyntaxKind::ExportKeyword,
        NodeData::Token,
        location_node.loc,
        NodeFlags::Reparsed,
    ));
    Arc::new(ModifierList::new(
        vec![export_modifier],
        crate::ast::ModifierFlags::Export,
    ))
}

pub(super) fn make_question_if_optional(
    is_bracketed: bool,
    type_expression: &Option<Arc<Node>>,
    location_node: &Arc<Node>,
) -> Option<Arc<Node>> {
    let is_optional_type = type_expression.as_ref().is_some_and(|te| {
        te.kind == SyntaxKind::JSDocTypeExpression
            && matches!(&te.data, NodeData::JSDocTypeExpression(d) if d.type_node.kind == SyntaxKind::JSDocOptionalType)
    });
    if is_bracketed || is_optional_type {
        Some(Arc::new(Node::with_loc_flags(
            SyntaxKind::QuestionToken,
            NodeData::Token,
            location_node.loc,
            NodeFlags::Reparsed,
        )))
    } else {
        None
    }
}

pub(super) fn deep_clone(node: &Arc<Node>) -> Arc<Node> {
    Arc::clone(node)
}

pub(super) fn name_is_qualified_name(name: &Arc<Node>) -> bool {
    name.kind == SyntaxKind::QualifiedName
}

pub(super) fn tag_name_loc(tag: &Arc<Node>) -> Option<TextRange> {
    match &tag.data {
        NodeData::JSDocOverloadTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocTypedefTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocCallbackTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocImportTag(d) => Some(d.tag_name.loc),
        _ => None,
    }
}
