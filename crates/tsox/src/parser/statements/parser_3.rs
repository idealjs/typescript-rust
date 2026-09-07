#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_variable_declaration_worker(
        &mut self,
        allow_exclamation: bool,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            &diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_IN_VARIABLE_DECLARATIONS,
        ));

        let exclamation_token = if allow_exclamation
            && name.kind == SyntaxKind::Identifier
            && self.token == SyntaxKind::ExclamationToken
            && !self.has_preceding_line_break()
        {
            let token = self.create_token_node();
            self.next_token();
            Some(token)
        } else {
            None
        };
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let mut end = name.end();
        if let Some(t) = &exclamation_token {
            end = end.max(t.end());
        }
        if let Some(t) = &type_node {
            end = end.max(t.end());
        }
        if let Some(n) = &initializer {
            end = end.max(n.end());
        }
        Arc::new(Node::with_loc(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(VariableDeclarationData {
                name,
                exclamation_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_identifier_or_pattern(&mut self) -> Arc<Node> {
        self.parse_identifier_or_pattern_with_diagnostic(None)
    }

    pub(crate) fn parse_identifier_or_pattern_with_diagnostic(
        &mut self,
        private_msg: Option<&'static crate::diagnostics::Message>,
    ) -> Arc<Node> {
        if self.token == SyntaxKind::OpenBracketToken {
            self.parse_array_binding_pattern()
        } else if self.token == SyntaxKind::OpenBraceToken {
            self.parse_object_binding_pattern()
        } else {
            self.parse_identifier_with_private_diagnostic(private_msg)
        }
    }

    pub(crate) fn parse_array_binding_pattern(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ArrayBindingElements,
            Parser::parse_array_binding_element,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrayBindingPattern,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_array_binding_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = if self.token != SyntaxKind::CommaToken {
            Some(self.parse_identifier_or_pattern())
        } else {
            None
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.as_ref().map_or(pos, |n| n.end()), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name: None,
                name,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_object_binding_pattern(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ObjectBindingElements,
            Parser::parse_object_binding_element,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ObjectBindingPattern,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_object_binding_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let is_identifier = self.is_identifier();
        let property_name = self.parse_property_name();
        let (property_name, name) = if is_identifier && self.token != SyntaxKind::ColonToken {
            (None, Some(property_name))
        } else {
            self.expect(SyntaxKind::ColonToken);
            (
                Some(property_name),
                Some(self.parse_identifier_or_pattern()),
            )
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.as_ref().map_or(pos, |n| n.end()), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name,
                name,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_optional_type_annotation(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type())
        } else {
            None
        }
    }

    pub(crate) fn parse_optional_return_type(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        }
    }
}
