use super::namespace::{
    deep_clone, make_question_if_optional, name_is_qualified_name, tag_name_loc,
};
use super::type_literal::reparse_jsdoc_type_literal;
use super::type_parameters::gather_type_parameters;
use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

pub(super) fn reparse_jsdoc_signature(
    js_signature: &Arc<Node>,
    fun: &Arc<Node>,
    js_doc: &Arc<Node>,
    tag: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
) -> Arc<Node> {
    let loc = if tag.kind == SyntaxKind::JSDocOverloadTag {
        tag_name_loc(tag).unwrap_or(tag.loc)
    } else {
        js_signature.loc
    };

    let type_parameters = if tag.kind != SyntaxKind::JSDocCallbackTag {
        gather_type_parameters(js_doc, false)
    } else {
        None
    };

    let (params_list, return_type) = extract_jsdoc_signature_data(js_signature);

    let parameters = Arc::new(NodeList::new(params_list));
    match fun.kind {
        SyntaxKind::FunctionDeclaration => {
            let name = fun.name().map(|n| deep_clone(n));
            Arc::new(Node::with_loc_flags(
                SyntaxKind::FunctionDeclaration,
                NodeData::FunctionDeclaration(FunctionDeclarationData {
                    modifiers,
                    asterisk_token: None,
                    name,
                    type_parameters,
                    parameters,
                    type_node: return_type,
                    full_signature: None,
                    body: None,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
        SyntaxKind::MethodDeclaration => {
            let name = fun.name().map(|n| deep_clone(n)).unwrap_or_else(|| {
                Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: String::new(),
                    }),
                    TextRange::new(fun.pos(), fun.pos()),
                ))
            });
            Arc::new(Node::with_loc_flags(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers,
                    asterisk_token: None,
                    name,
                    postfix_token: None,
                    type_parameters,
                    parameters,
                    type_node: return_type,
                    full_signature: None,
                    body: None,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
        SyntaxKind::Constructor => Arc::new(Node::with_loc_flags(
            SyntaxKind::Constructor,
            NodeData::ConstructorDeclaration(ConstructorDeclarationData {
                modifiers,
                type_parameters,
                parameters,
                type_node: return_type,
                full_signature: None,
                body: None,
            }),
            loc,
            NodeFlags::Reparsed,
        )),
        SyntaxKind::JSDocCallbackTag => Arc::new(Node::with_loc_flags(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters: None,
                parameters,
                type_node: return_type.or_else(|| {
                    Some(Arc::new(Node::with_loc(
                        SyntaxKind::AnyKeyword,
                        NodeData::KeywordTypeNode,
                        TextRange::new(fun.pos(), fun.pos()),
                    )))
                }),
            }),
            loc,
            NodeFlags::Reparsed,
        )),
        _ => Arc::new(Node::with_loc_flags(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters,
                parameters,
                type_node: return_type,
            }),
            loc,
            NodeFlags::Reparsed,
        )),
    }
}

fn extract_jsdoc_signature_data(js_signature: &Arc<Node>) -> (Vec<Arc<Node>>, Option<Arc<Node>>) {
    match &js_signature.data {
        NodeData::JSDocSignature(d) => {
            let params: Vec<Arc<Node>> = d
                .parameters
                .nodes
                .iter()
                .filter_map(|param| reparse_parameter_from_jsdoc(param))
                .collect();
            let return_type = d.type_node.as_ref().and_then(|tn| match &tn.data {
                NodeData::JSDocReturnTag(rt) => {
                    rt.type_expression.as_ref().and_then(|te| match &te.data {
                        NodeData::JSDocTypeExpression(ted) => Some(ted.type_node.clone()),
                        _ => None,
                    })
                }
                _ => None,
            });
            (params, return_type)
        }
        _ => (Vec::new(), None),
    }
}

fn reparse_parameter_from_jsdoc(param: &Arc<Node>) -> Option<Arc<Node>> {
    match param.kind {
        SyntaxKind::JSDocThisTag => {
            let (tag_name, type_expression) = match &param.data {
                NodeData::JSDocThisTag(d) => (&d.tag_name, &d.type_expression),
                _ => return None,
            };
            let this_ident = Arc::new(Node::with_loc_flags(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: "this".to_string(),
                }),
                tag_name.loc,
                NodeFlags::Reparsed,
            ));

            let param_type = match &type_expression.data {
                NodeData::JSDocTypeExpression(d) => Some(d.type_node.clone()),
                _ => None,
            };
            Some(Arc::new(Node::with_loc_flags(
                SyntaxKind::Parameter,
                NodeData::ParameterDeclaration(ParameterDeclarationData {
                    modifiers: None,
                    dot_dot_dot_token: None,
                    name: this_ident,
                    question_token: None,
                    type_node: param_type,
                    initializer: None,
                }),
                param.loc,
                NodeFlags::Reparsed,
            )))
        }
        SyntaxKind::JSDocParameterTag | SyntaxKind::JSDocPropertyTag => {
            let (name, is_bracketed, type_expression) = match &param.data {
                NodeData::JSDocParameterOrPropertyTag(d) => {
                    if name_is_qualified_name(&d.name) {
                        return None;
                    }
                    (&d.name, d.is_bracketed, &d.type_expression)
                }
                _ => return None,
            };

            let mut dot_dot_dot_token = None;
            let mut param_type = None;

            if let Some(te) = type_expression {
                if let NodeData::JSDocTypeExpression(ted) = &te.data {
                    if ted.type_node.kind == SyntaxKind::JSDocVariadicType {
                        dot_dot_dot_token = Some(Arc::new(Node::with_loc(
                            SyntaxKind::DotDotDotToken,
                            NodeData::Token,
                            param.loc,
                        )));
                        if let NodeData::JSDocVariadicType(vd) = &ted.type_node.data {
                            param_type = Some(reparse_jsdoc_type_literal(&vd.type_node));
                        }
                    } else {
                        param_type = Some(reparse_jsdoc_type_literal(&ted.type_node));
                    }
                }
            }

            let question_token = make_question_if_optional(is_bracketed, type_expression, param);

            Some(Arc::new(Node::with_loc_flags(
                SyntaxKind::Parameter,
                NodeData::ParameterDeclaration(ParameterDeclarationData {
                    modifiers: None,
                    dot_dot_dot_token,
                    name: deep_clone(name),
                    question_token,
                    type_node: param_type,
                    initializer: None,
                }),
                param.loc,
                NodeFlags::Reparsed,
            )))
        }
        _ => None,
    }
}
