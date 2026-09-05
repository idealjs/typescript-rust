use crate::ast::*;
use crate::core::text::TextRange;
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
        SyntaxKind::JSDocTypeExpression => {

            match &type_expression.data {
                NodeData::JSDocTypeExpression(d) => d.type_node.clone(),
                _ => return None,
            }
        }
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

fn reparse_jsdoc_signature(
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
        SyntaxKind::JSDocCallbackTag => {

            Arc::new(Node::with_loc_flags(
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
            ))
        }
        _ => {

            Arc::new(Node::with_loc_flags(
                SyntaxKind::FunctionType,
                NodeData::FunctionTypeNode(FunctionTypeNodeData {
                    type_parameters,
                    parameters,
                    type_node: return_type,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
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
            let return_type = d.type_node.as_ref().and_then(|tn| {

                match &tn.data {
                    NodeData::JSDocReturnTag(rt) => {
                        rt.type_expression.as_ref().and_then(|te| match &te.data {
                            NodeData::JSDocTypeExpression(ted) => Some(ted.type_node.clone()),
                            _ => None,
                        })
                    }
                    _ => None,
                }
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

fn reparse_jsdoc_type_literal(t: &Arc<Node>) -> Arc<Node> {
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

fn gather_type_parameters(js_doc: &Arc<Node>, typedef_or_callback: bool) -> Option<Arc<NodeList>> {
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

fn get_innermost_name_of_jsdoc_namespace(full_name: &Arc<Node>) -> Arc<Node> {
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

fn wrap_in_jsdoc_namespace(
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

fn create_export_modifier(location_node: &Arc<Node>) -> Arc<ModifierList> {
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

fn make_question_if_optional(
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

fn deep_clone(node: &Arc<Node>) -> Arc<Node> {
    Arc::clone(node)
}

fn name_is_qualified_name(name: &Arc<Node>) -> bool {
    name.kind == SyntaxKind::QualifiedName
}

fn tag_name_loc(tag: &Arc<Node>) -> Option<TextRange> {
    match &tag.data {
        NodeData::JSDocOverloadTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocTypedefTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocCallbackTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocImportTag(d) => Some(d.tag_name.loc),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    pub(crate) fn parse_source(text: &str) -> (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>) {
        let result = Parser::parse_source_file_text_with_diagnostics("test.ts", text.to_string());
        (Arc::new(result.0), result.1)
    }

    pub(crate) fn get_first_statement_jsdoc(file: &SourceFile) -> Vec<Arc<Node>> {
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => return Vec::new(),
        };
        if statements.is_empty() {
            return Vec::new();
        }

        let stmt = statements.last().unwrap();
        file.resolve_jsdoc(stmt)
    }

    #[test]
    pub(crate) fn test_typedef_simple() {
        let text = r#"
/**
 * @typedef {string} MyString
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        assert!(!jsdocs.is_empty(), "should have JSDoc");

        let tags = match &jsdocs[0].data {
            NodeData::JSDoc(d) => d.tags.as_ref(),
            _ => None,
        };
        assert!(tags.is_some(), "should have tags");
        let tags = tags.unwrap();
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypedefTag);

        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

        match &reparsed[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "MyString");
                assert_eq!(d.type_node.kind, SyntaxKind::StringKeyword);
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    pub(crate) fn test_typedef_object_literal() {
        let text = r#"
/**
 * @typedef {Object} Point
 * @property {number} x
 * @property {number} y
 */
let p;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        assert!(!jsdocs.is_empty());

        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

        match &reparsed[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "Point");

                assert_eq!(d.type_node.kind, SyntaxKind::TypeReference);
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    pub(crate) fn test_typedef_namespace() {
        let text = r#"
/**
 * @typedef {string} Foo.Bar
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);

        assert_eq!(reparsed[0].kind, SyntaxKind::ModuleDeclaration);

        match &reparsed[0].data {
            NodeData::ModuleDeclaration(d) => {
                assert_eq!(d.keyword, SyntaxKind::NamespaceKeyword);
                assert_eq!(node_text(&d.name), "Foo");

                let body = d.body.as_ref().expect("should have body");
                assert_eq!(body.kind, SyntaxKind::ModuleBlock);
                if let NodeData::ModuleBlock(mb) = &body.data {
                    assert_eq!(mb.statements.len(), 1);
                    assert_eq!(
                        mb.statements.nodes[0].kind,
                        SyntaxKind::TypeAliasDeclaration
                    );
                }
            }
            _ => panic!("expected ModuleDeclaration"),
        }
    }

    #[test]
    pub(crate) fn test_callback_tag() {
        let text = r#"
/**
 * @callback MyCallback
 * @param {string} x
 * @returns {number}
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

        match &reparsed[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "MyCallback");
                assert_eq!(d.type_node.kind, SyntaxKind::FunctionType);

                if let NodeData::FunctionTypeNode(ft) = &d.type_node.data {

                    assert!(
                        ft.type_node.is_some(),
                        "FunctionType should have a return type"
                    );
                } else {
                    panic!("expected FunctionTypeNode");
                }
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    pub(crate) fn test_import_tag() {

        let text = r#"
/**
 * @import { Foo } from "bar"
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);

        assert_eq!(reparsed.len(), 0);
    }

    #[test]
    pub(crate) fn test_overload_tag_function() {
        let text = r#"
/**
 * @overload
 * @param {string} x
 * @returns {string}
 */
function foo(x) { return x; }
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::FunctionDeclaration);
    }

    #[test]
    pub(crate) fn test_no_unhosted_tags() {
        let text = r#"
/**
 * @param {string} x
 * @returns {number}
 */
function foo(x) { return 42; }
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(
            reparsed.len(),
            0,
            "@param/@returns are hosted tags, no new statements"
        );
    }

    #[test]
    pub(crate) fn test_get_innermost_name_simple() {
        let ident = Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: "Foo".to_string(),
            }),
            TextRange::new(0, 3),
        ));
        let result = get_innermost_name_of_jsdoc_namespace(&ident);
        assert_eq!(result.kind, SyntaxKind::Identifier);
        assert_eq!(node_text(&result), "Foo");
    }

    #[test]
    pub(crate) fn test_get_innermost_name_namespace() {

        let c = Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: "C".to_string(),
            }),
            TextRange::new(0, 1),
        ));
        let b = Arc::new(Node::with_loc(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers: None,
                keyword: SyntaxKind::NamespaceKeyword,
                name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "B".to_string(),
                    }),
                    TextRange::new(0, 1),
                )),
                body: Some(c),
            }),
            TextRange::new(0, 1),
        ));
        let a = Arc::new(Node::with_loc(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers: None,
                keyword: SyntaxKind::NamespaceKeyword,
                name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "A".to_string(),
                    }),
                    TextRange::new(0, 1),
                )),
                body: Some(b),
            }),
            TextRange::new(0, 1),
        ));
        let result = get_innermost_name_of_jsdoc_namespace(&a);
        assert_eq!(result.kind, SyntaxKind::Identifier);
        assert_eq!(node_text(&result), "C");
    }

    #[test]
    pub(crate) fn test_wrap_in_jsdoc_namespace_simple() {
        let statement = Arc::new(Node::with_loc(
            SyntaxKind::TypeAliasDeclaration,
            NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
                modifiers: None,
                name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "T".to_string(),
                    }),
                    TextRange::new(0, 1),
                )),
                type_parameters: None,
                type_node: Arc::new(Node::with_loc(
                    SyntaxKind::StringKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(0, 1),
                )),
            }),
            TextRange::new(0, 1),
        ));

        let result = wrap_in_jsdoc_namespace(&statement, &statement, false);
        assert_eq!(result.kind, SyntaxKind::TypeAliasDeclaration);
    }

    #[test]
    pub(crate) fn test_integration_typedef_prepended_to_statements() {
        let text = r#"
/**
 * @typedef {string} MyString
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => panic!("expected SourceFile"),
        };

        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].kind, SyntaxKind::TypeAliasDeclaration);
        assert_eq!(statements[1].kind, SyntaxKind::VariableStatement);

        match &statements[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "MyString");
                assert_eq!(d.type_node.kind, SyntaxKind::StringKeyword);
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    pub(crate) fn test_integration_typedef_namespace_prepended() {
        let text = r#"
/**
 * @typedef {string} Foo.Bar
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => panic!("expected SourceFile"),
        };

        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].kind, SyntaxKind::ModuleDeclaration);
        assert_eq!(statements[1].kind, SyntaxKind::VariableStatement);
    }

    #[test]
    pub(crate) fn test_integration_overload_prepended_to_function() {
        let text = r#"
/**
 * @overload
 * @param {string} x
 * @returns {string}
 */
function foo(x) { return x; }
"#;
        let (file, _diags) = parse_source(text);
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => panic!("expected SourceFile"),
        };

        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].kind, SyntaxKind::FunctionDeclaration);
        assert_eq!(statements[1].kind, SyntaxKind::FunctionDeclaration);

        assert!(statements[0].flags.contains(NodeFlags::Reparsed));
        match &statements[0].data {
            NodeData::FunctionDeclaration(d) => {
                assert!(d.body.is_none(), "overload signature should have no body");
            }
            _ => panic!("expected FunctionDeclaration"),
        }
    }

    #[test]
    pub(crate) fn test_integration_no_jsdoc_unchanged() {

        let text = "let x = 1;\nlet y = 2;\n";
        let (file, _diags) = parse_source(text);
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => panic!("expected SourceFile"),
        };
        assert_eq!(statements.len(), 2, "no JSDoc, no reparsed nodes");
    }

    #[test]
    pub(crate) fn test_integration_hosted_tags_only_unchanged() {

        let text = r#"
/**
 * @param {string} x
 * @returns {number}
 */
function foo(x) { return 42; }
"#;
        let (file, _diags) = parse_source(text);
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => panic!("expected SourceFile"),
        };
        assert_eq!(statements.len(), 1, "hosted tags only, no new statements");
    }
}
