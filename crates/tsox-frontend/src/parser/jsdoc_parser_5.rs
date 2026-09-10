#![allow(unused_imports)]

use crate::parser::jsdoc::*;

impl crate::parser::Parser {
    pub(crate) fn parse_typedef_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        _indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        self.skip_whitespace_or_asterisk();
        let full_name = self.parse_jsdoc_type_name_with_namespace(false);
        let name = full_name.unwrap_or_else(|| {
            self.parse_jsdoc_identifier_name(Some(tsox_core::diagnostics::IDENTIFIER_EXPECTED))
        });
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);

        // Go parseTypedefTag 子标签收集：Object/Object[] 形态下预读后续
        // @property/@param 子标签（遇第一个不可解析子标签 rewind 终止），
        // 构造 JSDocTypeLiteral；主循环随后从 rewind 处重新解析这些行
        let mut type_expression = type_expression;
        let mut collected_end: Option<usize> = None;
        let object_like = type_expression
            .as_ref()
            .is_none_or(|te| is_object_or_object_array_type_reference(te));
        if object_like {
            let mut jsdoc_property_tags: Vec<Arc<Node>> = Vec::new();
            let mut has_children = false;
            let mut child_type_tag: Option<Arc<Node>> = None;
            loop {
                let saved_scanner = self.scanner.save_state();
                let saved_token = self.token;
                let saved_diagnostics_len = self.diagnostics.len();
                let child = self.parse_child_parameter_or_property_tag(
                    PropertyLikeParse(PropertyLikeParse::PROPERTY),
                    margin,
                    None,
                );
                let Some(child) = child else {
                    self.scanner.restore_state(saved_scanner);
                    self.token = saved_token;
                    self.diagnostics.truncate(saved_diagnostics_len);
                    break;
                };
                has_children = true;
                match child.kind {
                    SyntaxKind::JSDocTemplateTag => {}
                    SyntaxKind::JSDocTypeTag => {
                        if child_type_tag.is_none() {
                            child_type_tag = Some(child);
                        }
                    }
                    _ => jsdoc_property_tags.push(child),
                }
            }
            if has_children {
                let is_array_type = type_expression
                    .as_ref()
                    .is_some_and(|te| te.kind == SyntaxKind::ArrayType);
                let literal_start = jsdoc_property_tags
                    .first()
                    .map(|t| t.pos())
                    .unwrap_or(start);
                let literal = Arc::new(Node::with_loc(
                    SyntaxKind::JSDocTypeLiteral,
                    NodeData::JSDocTypeLiteral(JSDocTypeLiteralData {
                        jsdoc_property_tags: Some(jsdoc_property_tags),
                        is_array_type,
                    }),
                    TextRange::new(literal_start, self.token_pos()),
                ));
                type_expression = match child_type_tag
                    .as_ref()
                    .and_then(|tt| jsdoc_type_tag_type_expression(tt))
                    .filter(|te| !is_object_or_object_array_type_reference(te))
                {
                    Some(te) => Some(te),
                    None => Some(literal),
                };
                collected_end = Some(type_expression.as_ref().map(|t| t.end()).unwrap_or(start));
            }
        }

        let end = collected_end
            .unwrap_or_else(|| comment.end().max(name.end()));
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTypedefTag,
            NodeData::JSDocTypedefTag(JSDocTypedefTagData {
                tag_name,
                type_expression,
                name: Some(name),
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_callback_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        _indent_text: &str,
    ) -> Arc<Node> {
        let full_name = self.parse_jsdoc_type_name_with_namespace(false);
        let name = full_name.unwrap_or_else(|| {
            self.parse_jsdoc_identifier_name(Some(tsox_core::diagnostics::IDENTIFIER_EXPECTED))
        });
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);
        let type_expression = self.parse_jsdoc_signature(start, margin);
        let end = type_expression.end().max(comment.end()).max(name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocCallbackTag,
            NodeData::JSDocCallbackTag(JSDocCallbackTagData {
                tag_name,
                type_expression,
                name: Some(name),
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_overload_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        _indent_text: &str,
    ) -> Arc<Node> {
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);
        let type_expression = self.parse_jsdoc_signature(start, margin);
        let end = type_expression.end().max(comment.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocOverloadTag,
            NodeData::JSDocOverloadTag(JSDocOverloadTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_jsdoc_signature(&mut self, start: usize, indent: usize) -> Arc<Node> {
        let parameters = self.parse_callback_tag_parameters(indent);
        let return_tag = if self.parse_optional_jsdoc(SyntaxKind::AtToken) {
            let tag = self.parse_tag(indent);
            if tag.kind == SyntaxKind::JSDocReturnTag {
                Some(tag)
            } else {
                None
            }
        } else {
            None
        };
        let end = return_tag
            .as_ref()
            .map(|t| t.end())
            .unwrap_or_else(|| parameters.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocSignature,
            NodeData::JSDocSignature(JSDocSignatureData {
                type_parameters: None,
                parameters,
                type_node: return_tag,
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_callback_tag_parameters(&mut self, indent: usize) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut params = Vec::new();
        loop {
            if self.token == SyntaxKind::EndOfFile {
                break;
            }
            if self.token == SyntaxKind::AtToken {
                if let Some(child) = self.parse_child_parameter_or_property_tag(
                    PropertyLikeParse(PropertyLikeParse::CALLBACK_PARAMETER),
                    indent,
                    None,
                ) {
                    if child.kind == SyntaxKind::JSDocParameterTag {
                        params.push(child);
                    }
                }
            } else {
                self.next_token_jsdoc();
            }
        }
        let end = params.last().map(|p| p.end()).unwrap_or(pos);
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params,
        })
    }

    pub(crate) fn parse_import_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end();
        let module_specifier = self.create_missing_node(
            SyntaxKind::StringLiteral,
            self.token_pos(),
            self.token_pos(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocImportTag,
            NodeData::JSDocImportTag(JSDocImportTagData {
                tag_name,
                import_clause: None,
                module_specifier,
                attributes: None,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }
}

/// Go isObjectOrObjectArrayTypeReference：Object 关键字、Object 引用（无类型
/// 实参）或其数组形态；入参为 JSDocTypeExpression，判断其内部类型节点
pub(crate) fn is_object_or_object_array_type_reference(te: &Arc<Node>) -> bool {
    let type_node = match &te.data {
        NodeData::JSDocTypeExpression(d) => &d.type_node,
        _ => te,
    };
    match &type_node.data {
        NodeData::KeywordTypeNode => true,
        NodeData::ArrayTypeNode(d) => {
            let inner = Arc::clone(&d.element_type);
            is_object_or_object_array_type_reference(&inner)
        }
        NodeData::TypeReferenceNode(d) => {
            d.type_name.kind == SyntaxKind::Identifier
                && d.type_name.text() == "Object"
                && d.type_arguments.is_none()
        }
        _ => false,
    }
}

pub(crate) fn jsdoc_type_tag_type_expression(tag: &Arc<Node>) -> Option<Arc<Node>> {
    match &tag.data {
        NodeData::JSDocTypeTag(d) => Some(Arc::clone(&d.type_expression)),
        _ => None,
    }
}
