use crate::ast::*;
use crate::parser::reparser_namespace::create_export_modifier;
use crate::parser::reparser_namespace::get_innermost_name_of_jsdoc_namespace;
use crate::parser::reparser_namespace::wrap_in_jsdoc_namespace;
use crate::parser::reparser_signature::reparse_jsdoc_signature;
use crate::parser::reparser_type_literal::reparse_jsdoc_type_literal;
use crate::parser::reparser_type_parameters::gather_type_parameters;
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

use crate::parser::reparser_type_literal::property_tags_to_signatures;

fn js_doc_property_tags_after(js_doc: &Arc<Node>, typedef_tag: &Arc<Node>) -> Vec<Arc<Node>> {
    let tags = match &js_doc.data {
        NodeData::JSDoc(d) => d.tags.as_ref(),
        _ => None,
    };
    let Some(tags) = tags else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut after = false;
    for t in tags.iter() {
        if Arc::ptr_eq(t, typedef_tag) {
            after = true;
            continue;
        }
        if !after {
            continue;
        }
        // 对齐 Go parseJSDocTypeReferenceAndPopularTags 的子标签收集：
        // property 之外的合法子标签（type/template/this）跳过继续，
        // 其余标签（unknown、param 等）终止收集，rewind 后不再回看
        match t.kind {
            SyntaxKind::JSDocTypedefTag | SyntaxKind::JSDocCallbackTag => break,
            SyntaxKind::JSDocPropertyTag => result.push(Arc::clone(t)),
            SyntaxKind::JSDocTypeTag | SyntaxKind::JSDocTemplateTag | SyntaxKind::JSDocThisTag => {}
            _ => break,
        }
    }
    result
}

fn reparse_typedef_tag(tag: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    let (type_expression, full_name) = match &tag.data {
        NodeData::JSDocTypedefTag(d) => {
            let name = d.name.as_ref()?;
            // @typedef 无类型表达式：隐式 Object（@property 标签并入）
            match d.type_expression.as_ref() {
                Some(te) => (Some(te.clone()), name.clone()),
                None => (None, name.clone()),
            }
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

    let mut type_node = match &type_expression {
        Some(te) => match te.kind {
            SyntaxKind::JSDocTypeExpression => match &te.data {
                NodeData::JSDocTypeExpression(d) => d.type_node.clone(),
                _ => return None,
            },
            SyntaxKind::JSDocTypeLiteral => reparse_jsdoc_type_literal(te),
            _ => return None,
        },
        None => Arc::new(Node::with_loc(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "Object".to_string(),
                    }),
                    tag.loc,
                )),
                type_arguments: None,
            }),
            tag.loc,
        )),
    };

    // @typedef {Object} 后跟 @property 标签：属性并入类型字面量（tsc jsdoc 解析器行为）；
    // 无效标签自然跳过，只收集 JSDocPropertyTag
    let property_tags: Vec<Arc<Node>> = js_doc_property_tags_after(js_doc, tag);
    if !property_tags.is_empty()
        && matches!(type_node.data, NodeData::TypeReferenceNode(_))
        && matches!(&type_node.data, NodeData::TypeReferenceNode(tr) if tr.type_name.text() == "Object")
    {
        let members = Arc::new(NodeList::new(property_tags_to_signatures(&property_tags)));
        type_node = Arc::new(Node::with_loc_flags(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData { members }),
            tag.loc,
            NodeFlags::Reparsed,
        ));
    }

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
