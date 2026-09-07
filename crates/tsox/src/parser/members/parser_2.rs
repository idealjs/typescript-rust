#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_heritage_clauses(&mut self) -> Option<Arc<NodeList>> {
        let mut clauses = Vec::new();
        let mut pos = self.token_pos();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let types = self.parse_delimited_list(
                ParsingContext::HeritageClauseElement,
                Parser::parse_heritage_clause_element,
            );
            let end = self.token_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: SyntaxKind::ExtendsKeyword,
                    types: Arc::new(types),
                }),
                TextRange::new(pos, end),
            )));
        }
        if self.token == SyntaxKind::ImplementsKeyword {
            pos = self.token_pos();
            self.next_token();
            let types = self.parse_delimited_list(
                ParsingContext::HeritageClauseElement,
                Parser::parse_heritage_clause_element,
            );
            let end = self.token_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: SyntaxKind::ImplementsKeyword,
                    types: Arc::new(types),
                }),
                TextRange::new(pos, end),
            )));
        }
        if clauses.is_empty() {
            None
        } else {
            let end = clauses.last().unwrap().end();
            Some(Arc::new(NodeList {
                loc: TextRange::new(clauses[0].pos(), end),
                nodes: clauses,
            }))
        }
    }

    pub(crate) fn parse_heritage_clause_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let expression = self.parse_left_hand_side_expression();
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments
            .as_ref()
            .map_or(expression.end(), |ta| ta.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionWithTypeArguments,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData {
                expression,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_member(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::OpenParenToken || self.token == SyntaxKind::LessThanToken {
            return self.parse_signature_member(SyntaxKind::CallSignature);
        }

        if self.token == SyntaxKind::NewKeyword && {
            let mut s = self.scanner.clone();
            let t = s.scan();
            t == SyntaxKind::OpenParenToken || t == SyntaxKind::LessThanToken
        } {
            return self.parse_signature_member(SyntaxKind::ConstructSignature);
        }

        let pos = self.token_pos();
        let modifiers = self.parse_type_member_modifiers();

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                return self.parse_accessor_declaration(pos, modifiers, accessor_kind);
            }
        }
        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        let name = self.parse_property_name();
        let postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_parameters = self.parse_optional_type_parameters();
        if self.token == SyntaxKind::OpenParenToken {
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            self.parse_type_member_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodSignature,
                NodeData::MethodSignatureDeclaration(MethodSignatureDeclarationData {
                    modifiers,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ));
        }

        let type_node = self
            .parse_optional_type_annotation()
            .unwrap_or_else(|| self.missing_node(self.token_pos()));
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_type()
        } else {
            self.missing_node(self.token_pos())
        };
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_type_member_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        let mut modifiers = Vec::new();

        while matches!(
            self.token,
            SyntaxKind::ReadonlyKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
        ) {
            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() || !Self::token_can_follow_modifier(s.token()) {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }
        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_modifier_list(modifiers))
        }
    }

    pub(crate) fn token_can_follow_modifier(token: SyntaxKind) -> bool {
        token == SyntaxKind::OpenBracketToken
            || token == SyntaxKind::OpenBraceToken
            || token == SyntaxKind::AsteriskToken
            || token == SyntaxKind::DotDotDotToken
            || is_identifier_or_keyword(token)
            || token == SyntaxKind::StringLiteral
            || token == SyntaxKind::NumericLiteral
            || token == SyntaxKind::BigIntLiteral
    }

    pub(crate) fn token_can_follow_get_or_set(token: SyntaxKind) -> bool {
        token == SyntaxKind::OpenBracketToken
            || is_identifier_or_keyword(token)
            || token == SyntaxKind::StringLiteral
            || token == SyntaxKind::NumericLiteral
            || token == SyntaxKind::BigIntLiteral
    }

    pub(crate) fn can_follow_export_modifier(token: SyntaxKind) -> bool {
        token == SyntaxKind::AtToken
            || (token != SyntaxKind::AsteriskToken
                && token != SyntaxKind::AsKeyword
                && token != SyntaxKind::OpenBraceToken
                && Self::token_can_follow_modifier(token))
    }

    pub(crate) fn token_can_follow_default_keyword(
        t: SyntaxKind,
        s: &mut crate::scanner::Scanner,
    ) -> bool {
        match t {
            SyntaxKind::ClassKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::AtToken => true,
            SyntaxKind::AbstractKeyword => {
                s.scan() == SyntaxKind::ClassKeyword && !s.has_preceding_line_break()
            }
            SyntaxKind::AsyncKeyword => {
                s.scan() == SyntaxKind::FunctionKeyword && !s.has_preceding_line_break()
            }
            _ => false,
        }
    }

    pub(crate) fn is_class_member_modifier(token: SyntaxKind) -> bool {
        matches!(
            token,
            SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::AccessorKeyword
        )
    }

    pub(crate) fn look_ahead_class_member_start(&self) -> bool {
        if self.token == SyntaxKind::AtToken {
            return true;
        }
        let mut id_token = SyntaxKind::Unknown;
        let mut s = self.scanner.clone();
        let mut t = self.token;

        while is_modifier_kind(t) {
            id_token = t;
            if Self::is_class_member_modifier(id_token) {
                return true;
            }
            t = s.scan();
        }
        if t == SyntaxKind::AsteriskToken {
            return true;
        }

        if is_identifier_or_keyword(t)
            || t == SyntaxKind::PrivateIdentifier
            || t == SyntaxKind::StringLiteral
            || t == SyntaxKind::NumericLiteral
            || t == SyntaxKind::BigIntLiteral
        {
            id_token = t;
            t = s.scan();
        }

        if t == SyntaxKind::OpenBracketToken {
            return true;
        }

        if id_token != SyntaxKind::Unknown {
            if !is_keyword_kind(id_token)
                || id_token == SyntaxKind::SetKeyword
                || id_token == SyntaxKind::GetKeyword
            {
                return true;
            }

            match t {
                SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::ColonToken
                | SyntaxKind::EqualsToken
                | SyntaxKind::QuestionToken => return true,
                _ => {}
            }

            return t == SyntaxKind::SemicolonToken
                || t == SyntaxKind::CloseBraceToken
                || t == SyntaxKind::EndOfFile
                || s.has_preceding_line_break();
        }
        false
    }
}
