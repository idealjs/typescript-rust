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

    /// Go parseImportTag：`@import [clause] from "specifier" [with {…}]`，
    /// jsdoc token 模式下解析子句/说明符/属性（字符串字面量由 jsdoc
    /// 扫描器产出）
    /// Go jsdoc import 解析用 skip-asterisks 主扫描器（trivia 自动跳过）；
    /// 本仓 jsdoc 扫描器产出空白 token，各解析步间显式跳 trivia
    /// （换行后的 * 行装饰一并跳过）
    fn skip_jsdoc_import_trivia(&mut self) {
        let mut after_newline = false;
        loop {
            match self.token {
                SyntaxKind::WhitespaceTrivia => {
                    self.next_token_jsdoc();
                }
                SyntaxKind::NewLineTrivia => {
                    after_newline = true;
                    self.next_token_jsdoc();
                }
                SyntaxKind::AsteriskToken if after_newline => {
                    self.next_token_jsdoc();
                }
                _ => break,
            }
        }
    }

    pub(crate) fn parse_import_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let after_import_pos = self.token_pos();

        let mut identifier: Option<Arc<Node>> = None;
        if self.token == SyntaxKind::Identifier {
            identifier = Some(self.parse_jsdoc_identifier_name(None));
            self.skip_jsdoc_import_trivia();
        }

        let import_clause: Option<Arc<Node>> =
            if identifier.is_some() || self.token == SyntaxKind::AsteriskToken || self.token == SyntaxKind::OpenBraceToken
            {
                let named_bindings: Option<Arc<Node>> =
                    if identifier.is_none() || {
                        self.skip_jsdoc_import_trivia();
                        self.parse_optional_jsdoc(SyntaxKind::CommaToken)
                    } {
                        self.skip_jsdoc_import_trivia();
                        if self.token == SyntaxKind::AsteriskToken {
                            Some(self.parse_jsdoc_namespace_import())
                        } else {
                            Some(self.parse_jsdoc_named_imports())
                        }
                    } else {
                        None
                    };
                let clause_end = named_bindings
                    .as_ref()
                    .map(|b| b.end())
                    .or_else(|| identifier.as_ref().map(|i| i.end()))
                    .unwrap_or(after_import_pos);
                self.skip_jsdoc_import_trivia();
                self.parse_expected_jsdoc(SyntaxKind::FromKeyword);
                self.skip_jsdoc_import_trivia();
                Some(Arc::new(Node::with_loc(
                    SyntaxKind::ImportClause,
                    NodeData::ImportClause(ImportClauseData {
                        phase_modifier: None,
                        name: identifier.clone(),
                        named_bindings,
                    }),
                    TextRange::new(after_import_pos, clause_end),
                )))
            } else {
                None
            };

        self.skip_jsdoc_import_trivia();
        let module_specifier = if self.token == SyntaxKind::StringLiteral {
            let lit = self.create_token_node_jsdoc();
            self.next_token_jsdoc();
            lit
        } else {
            // Go parseModuleSpecifier 回退 parseExpression：错误恢复消费到
            // with/@/EOF 为止（`from () with {…}` 的 () 被吃掉）
            let miss_pos = self.token_pos();
            let mut guard = 0;
            while !matches!(
                self.token,
                SyntaxKind::WithKeyword
                    | SyntaxKind::AssertKeyword
                    | SyntaxKind::AtToken
                    | SyntaxKind::EndOfFile
            ) && guard < 8
            {
                self.next_token_jsdoc();
                guard += 1;
            }
            self.create_missing_node(SyntaxKind::StringLiteral, miss_pos, miss_pos)
        };

        self.skip_jsdoc_import_trivia();
        let attributes: Option<Arc<Node>> =
            if self.token == SyntaxKind::WithKeyword || self.token == SyntaxKind::AssertKeyword {
                Some(self.parse_jsdoc_import_attributes())
            } else {
                None
            };

        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let content_end = attributes
            .as_ref()
            .map(|a| a.end())
            .unwrap_or_else(|| module_specifier.end())
            .max(comment.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocImportTag,
            NodeData::JSDocImportTag(JSDocImportTagData {
                tag_name,
                import_clause,
                module_specifier,
                attributes,
                comment: Some(comment),
            }),
            TextRange::new(start, content_end.max(start)),
        ))
    }

    /// `* as name`
    fn parse_jsdoc_namespace_import(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.parse_expected_jsdoc(SyntaxKind::AsteriskToken);
        self.skip_jsdoc_import_trivia();
        self.parse_expected_jsdoc(SyntaxKind::AsKeyword);
        self.skip_jsdoc_import_trivia();
        let name = self.parse_jsdoc_identifier_name(None);
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::NamespaceImport,
            NodeData::NamespaceImport(NamespaceImportData { name }),
            TextRange::new(pos, end),
        ))
    }

    /// `{ A, B as C }`
    fn parse_jsdoc_named_imports(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.parse_expected_jsdoc(SyntaxKind::OpenBraceToken);
        let mut elements: Vec<Arc<Node>> = Vec::new();
        loop {
            self.skip_jsdoc_import_trivia();
            if self.token == SyntaxKind::CloseBraceToken
                || self.token == SyntaxKind::EndOfFile
                || self.token == SyntaxKind::AtToken
            {
                break;
            }
            let spec_pos = self.token_pos();
            let mut property_name: Option<Arc<Node>> = None;
            let first = self.parse_jsdoc_identifier_name(None);
            self.skip_jsdoc_import_trivia();
            let name = if self.parse_optional_jsdoc(SyntaxKind::AsKeyword) {
                property_name = Some(first);
                self.skip_jsdoc_import_trivia();
                self.parse_jsdoc_identifier_name(None)
            } else {
                first
            };
            let end = name.end();
            elements.push(Arc::new(Node::with_loc(
                SyntaxKind::ImportSpecifier,
                NodeData::ImportSpecifier(ImportSpecifierData {
                    is_type_only: false,
                    property_name,
                    name,
                }),
                TextRange::new(spec_pos, end),
            )));
            self.skip_jsdoc_import_trivia();
            if !self.parse_optional_jsdoc(SyntaxKind::CommaToken) {
                break;
            }
        }
        self.skip_jsdoc_import_trivia();
        self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
        let end = self.token_end();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedImports,
            NodeData::NamedImports(NamedImportsData {
                elements: Arc::new(NodeList::new(elements)),
            }),
            TextRange::new(pos, end),
        ))
    }

    /// `with { key: "value", … }`
    fn parse_jsdoc_import_attributes(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let token = self.token;
        self.parse_expected_jsdoc(token);
        self.skip_jsdoc_import_trivia();
        self.parse_expected_jsdoc(SyntaxKind::OpenBraceToken);
        let mut attributes: Vec<Arc<Node>> = Vec::new();
        loop {
            self.skip_jsdoc_import_trivia();
            if self.token == SyntaxKind::CloseBraceToken
                || self.token == SyntaxKind::EndOfFile
                || self.token == SyntaxKind::AtToken
            {
                break;
            }
            let attr_pos = self.token_pos();
            let name = if self.token == SyntaxKind::StringLiteral {
                let lit = self.create_token_node_jsdoc();
                self.next_token_jsdoc();
                lit
            } else {
                self.parse_jsdoc_identifier_name(None)
            };
            self.skip_jsdoc_import_trivia();
            self.parse_expected_jsdoc(SyntaxKind::ColonToken);
            self.skip_jsdoc_import_trivia();
            let value = if self.token == SyntaxKind::StringLiteral {
                let lit = self.create_token_node_jsdoc();
                self.next_token_jsdoc();
                lit
            } else {
                self.create_missing_node(
                    SyntaxKind::StringLiteral,
                    self.token_pos(),
                    self.token_pos(),
                )
            };
            let end = value.end();
            attributes.push(Arc::new(Node::with_loc(
                SyntaxKind::ImportAttribute,
                NodeData::ImportAttribute(ImportAttributeData { name, value }),
                TextRange::new(attr_pos, end),
            )));
            self.skip_jsdoc_import_trivia();
            if !self.parse_optional_jsdoc(SyntaxKind::CommaToken) {
                break;
            }
        }
        self.skip_jsdoc_import_trivia();
        self.parse_expected_jsdoc(SyntaxKind::CloseBraceToken);
        let end = self.token_end();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportAttributes,
            NodeData::ImportAttributes(ImportAttributesData {
                token,
                attributes: Arc::new(NodeList::new(attributes)),
                multi_line: false,
            }),
            TextRange::new(pos, end),
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
