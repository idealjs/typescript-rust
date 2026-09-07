#![allow(unused_imports)]

use super::*;

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
            self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED))
        });
        self.skip_whitespace();
        let comment = self.parse_tag_comments(margin, None);
        let end = comment.end().max(name.end());
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
            self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED))
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
