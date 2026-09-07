use super::namespace::{deep_clone, make_question_if_optional};
use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

pub(super) fn reparse_jsdoc_type_literal(t: &Arc<Node>) -> Arc<Node> {
    if t.kind != SyntaxKind::JSDocTypeLiteral {
        return t.clone();
    }

    let (is_array_type, property_tags) = match &t.data {
        NodeData::JSDocTypeLiteral(d) => (d.is_array_type, d.jsdoc_property_tags.as_ref()),
        _ => return t.clone(),
    };

    let mut properties: Vec<Arc<Node>> = Vec::new();
    if let Some(tags) = property_tags {
        for prop in tags {
            if prop.kind != SyntaxKind::JSDocPropertyTag
                && prop.kind != SyntaxKind::JSDocParameterTag
            {
                continue;
            }
            let (name, is_bracketed, type_expression) = match &prop.data {
                NodeData::JSDocParameterOrPropertyTag(d) => {
                    (&d.name, d.is_bracketed, &d.type_expression)
                }
                _ => continue,
            };

            let prop_name = if name.kind == SyntaxKind::QualifiedName {
                match &name.data {
                    NodeData::QualifiedName(d) => d.right.clone(),
                    _ => name.clone(),
                }
            } else {
                deep_clone(name)
            };

            let prop_type = type_expression.as_ref().and_then(|te| match &te.data {
                NodeData::JSDocTypeExpression(ted) => {
                    Some(reparse_jsdoc_type_literal(&ted.type_node))
                }
                _ => None,
            });

            let question_token = make_question_if_optional(is_bracketed, type_expression, prop);

            let property = Arc::new(Node::with_loc_flags(
                SyntaxKind::PropertySignature,
                NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                    modifiers: None,
                    name: prop_name,
                    postfix_token: question_token,
                    type_node: prop_type.unwrap_or_else(|| {
                        Arc::new(Node::with_loc(
                            SyntaxKind::AnyKeyword,
                            NodeData::KeywordTypeNode,
                            TextRange::new(prop.pos(), prop.pos()),
                        ))
                    }),

                    initializer: Arc::new(Node::with_loc(
                        SyntaxKind::MissingDeclaration,
                        NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
                        TextRange::new(prop.pos(), prop.pos()),
                    )),
                }),
                prop.loc,
                NodeFlags::Reparsed,
            ));
            properties.push(property);
        }
    }

    let members = Arc::new(NodeList::new(properties));
    let mut result = Arc::new(Node::with_loc_flags(
        SyntaxKind::TypeLiteral,
        NodeData::TypeLiteralNode(TypeLiteralNodeData { members }),
        t.loc,
        NodeFlags::Reparsed,
    ));

    if is_array_type {
        result = Arc::new(Node::with_loc_flags(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ArrayTypeNodeData {
                element_type: result,
            }),
            t.loc,
            NodeFlags::Reparsed,
        ));
    }

    result
}
