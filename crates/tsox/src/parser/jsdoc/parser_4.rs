#![allow(unused_imports)]

use super::*;

impl crate::parser::Parser {
    pub(crate) fn parse_throws_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment
            .end()
            .max(type_expression.as_ref().map(|t| t.end()).unwrap_or(0));
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocThrowsTag,
            NodeData::JSDocThrowsTag(JSDocThrowsTagData {
                tag_name,
                type_expression,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_see_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let name_expression = if self.is_identifier()
            || (self.token == SyntaxKind::OpenBraceToken && {
                let mut sc = self.scanner.clone();
                sc.scan_jsdoc_token();
                let is_id = matches!(
                    sc.token(),
                    SyntaxKind::Identifier
                        | SyntaxKind::ThisKeyword
                        | SyntaxKind::TrueKeyword
                        | SyntaxKind::FalseKeyword
                );
                is_id
            }) {
            Some(self.parse_jsdoc_name_reference())
        } else {
            None
        };
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment
            .end()
            .max(name_expression.as_ref().map(|n| n.end()).unwrap_or(0));
        let name = name_expression.unwrap_or_else(|| {
            self.create_missing_node(SyntaxKind::Identifier, self.token_pos(), self.token_pos())
        });
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocSeeTag,
            NodeData::JSDocSeeTag(JSDocSeeTagData {
                tag_name,
                name_expression: name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_implements_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let class_name = self.parse_expression_with_type_arguments_for_augments();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(class_name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocImplementsTag,
            NodeData::JSDocImplementsTag(JSDocImplementsTagData {
                tag_name,
                class_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_augments_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let class_name = self.parse_expression_with_type_arguments_for_augments();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(class_name.end());
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocAugmentsTag,
            NodeData::JSDocAugmentsTag(JSDocAugmentsTagData {
                tag_name,
                class_name,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_parameter_or_property_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        target: PropertyLikeParse,
        indent: usize,
    ) -> Arc<Node> {
        let type_expression = self.try_parse_type_expression();
        let is_name_first = type_expression.is_none();
        self.skip_whitespace_or_asterisk();
        let (name, is_bracketed) = self.parse_bracket_name_in_property_and_param_tag(target);
        let indent_text = self.skip_whitespace_or_asterisk();

        let type_expression = if is_name_first && type_expression.is_none() {
            let _ = self.parse_jsdoc_link_prefix();
            self.try_parse_type_expression()
        } else {
            type_expression
        };

        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            indent,
            &indent_text,
        );

        let end = comment
            .end()
            .max(type_expression.as_ref().map(|t| t.end()).unwrap_or(0))
            .max(name.end());

        let kind = if target.contains(PropertyLikeParse::PARAMETER) {
            SyntaxKind::JSDocParameterTag
        } else {
            SyntaxKind::JSDocPropertyTag
        };

        Arc::new(Node::with_loc(
            kind,
            NodeData::JSDocParameterOrPropertyTag(JSDocParameterOrPropertyTagData {
                tag_name,
                name,
                is_bracketed,
                type_expression,
                is_name_first,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_template_tag(
        &mut self,
        start: usize,
        tag_name: Arc<Node>,
        margin: usize,
        indent_text: &str,
    ) -> Arc<Node> {
        let constraint = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_jsdoc_type_expression(false))
        } else {
            None
        };
        let type_parameters = self.parse_template_tag_type_parameters();
        let comment = self.parse_trailing_tag_comments(
            self.token_pos(),
            self.token_end(),
            margin,
            indent_text,
        );
        let end = comment.end().max(type_parameters.end());
        let constraint_node = constraint.unwrap_or_else(|| {
            self.create_missing_node(SyntaxKind::MissingDeclaration, start, start)
        });
        Arc::new(Node::with_loc(
            SyntaxKind::JSDocTemplateTag,
            NodeData::JSDocTemplateTag(JSDocTemplateTagData {
                tag_name,
                constraint: constraint_node,
                type_parameters,
                comment: Some(comment),
            }),
            TextRange::new(start, end),
        ))
    }

    pub(crate) fn parse_template_tag_type_parameters(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut params = Vec::new();
        loop {
            params.push(self.parse_template_tag_type_parameter());
            self.skip_whitespace_or_asterisk();
            if !self.parse_optional_jsdoc(SyntaxKind::CommaToken) {
                break;
            }
        }
        let end = params.last().map(|p| p.end()).unwrap_or(pos);
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params,
        })
    }

    pub(crate) fn parse_template_tag_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let modifiers = if self.token == SyntaxKind::ConstKeyword {
            let mod_node = self.create_token_node_jsdoc();
            self.next_token_jsdoc();
            Some(Arc::new(ModifierList::new(
                vec![mod_node],
                crate::ast::ModifierFlags::Const,
            )))
        } else {
            None
        };

        let is_bracketed = self.parse_optional_jsdoc(SyntaxKind::OpenBracketToken);
        let name = self.parse_jsdoc_identifier_name(Some(diagnostics::IDENTIFIER_EXPECTED));
        let default_type = if is_bracketed {
            self.skip_whitespace();
            let default = if self.parse_optional_jsdoc(SyntaxKind::EqualsToken) {
                Some(self.parse_type())
            } else {
                None
            };
            self.parse_expected_token_jsdoc(SyntaxKind::CloseBracketToken);
            default
        } else {
            None
        };

        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers,
                name,
                constraint: None,
                expression: None,
                default_type,
            }),
            TextRange::new(pos, end),
        ))
    }
}
