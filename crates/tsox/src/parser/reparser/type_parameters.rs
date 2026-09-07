use super::namespace::deep_clone;
use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

pub(super) fn gather_type_parameters(
    js_doc: &Arc<Node>,
    typedef_or_callback: bool,
) -> Option<Arc<NodeList>> {
    let tags = match &js_doc.data {
        NodeData::JSDoc(d) => d.tags.as_ref(),
        _ => return None,
    };
    let Some(tags) = tags else {
        return None;
    };

    let mut type_parameters: Vec<Arc<Node>> = Vec::new();
    let mut pos = 0usize;
    let mut end_pos = 0usize;
    let mut first_template = true;

    for tag in &tags.nodes {
        if !typedef_or_callback
            && (tag.kind == SyntaxKind::JSDocTypedefTag || tag.kind == SyntaxKind::JSDocCallbackTag)
        {
            return None;
        }
        if tag.kind != SyntaxKind::JSDocTemplateTag {
            continue;
        }

        if first_template {
            pos = tag.pos();
            first_template = false;
        }
        end_pos = tag.end();

        let (constraint, template_type_params) = match &tag.data {
            NodeData::JSDocTemplateTag(d) => (&d.constraint, &d.type_parameters),
            _ => continue,
        };

        let mut first_type_parameter = true;
        for tp in &template_type_params.nodes {
            let reparse = if constraint.kind != SyntaxKind::Unknown && first_type_parameter {
                let (tp_modifiers, tp_name, tp_default) = match &tp.data {
                    NodeData::TypeParameterDeclaration(d) => {
                        (d.modifiers.clone(), d.name.clone(), d.default_type.clone())
                    }
                    _ => continue,
                };
                let constraint_type = match &constraint.data {
                    NodeData::JSDocTypeExpression(d) => d.type_node.clone(),
                    _ => constraint.clone(),
                };
                Arc::new(Node::with_loc_flags(
                    SyntaxKind::TypeParameter,
                    NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                        modifiers: tp_modifiers,
                        name: deep_clone(&tp_name),
                        constraint: Some(constraint_type),
                        expression: None,
                        default_type: tp_default,
                    }),
                    tp.loc,
                    NodeFlags::Reparsed,
                ))
            } else {
                deep_clone(tp)
            };
            type_parameters.push(reparse);
            first_type_parameter = false;
        }
    }

    if type_parameters.is_empty() {
        None
    } else {
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end_pos),
            nodes: type_parameters,
        }))
    }
}
