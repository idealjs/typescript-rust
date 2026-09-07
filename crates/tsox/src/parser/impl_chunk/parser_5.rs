#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn scan_start_of_declaration(&mut self) -> bool {
        loop {
            match self.token {
                SyntaxKind::VarKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::ConstKeyword
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::EnumKeyword => return true,
                SyntaxKind::InterfaceKeyword | SyntaxKind::TypeKeyword => {
                    return self.next_token_is_identifier_on_same_line();
                }
                SyntaxKind::ModuleKeyword | SyntaxKind::NamespaceKeyword => {
                    return self.next_token_is_identifier_or_string_literal_on_same_line();
                }
                SyntaxKind::AbstractKeyword
                | SyntaxKind::AccessorKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::ReadonlyKeyword => {
                    let previous_token = self.token;
                    self.next_token();

                    if self.has_preceding_line_break() {
                        return false;
                    }
                    if previous_token == SyntaxKind::DeclareKeyword
                        && self.token == SyntaxKind::TypeKeyword
                    {
                        return true;
                    }
                    continue;
                }
                SyntaxKind::GlobalKeyword => {
                    self.next_token();
                    return self.token == SyntaxKind::OpenBraceToken
                        || self.token == SyntaxKind::Identifier
                        || self.token == SyntaxKind::ExportKeyword;
                }

                SyntaxKind::StaticKeyword => {
                    self.next_token();
                    continue;
                }
                SyntaxKind::ImportKeyword => {
                    self.next_token();
                    return self.token == SyntaxKind::StringLiteral
                        || self.token == SyntaxKind::AsteriskToken
                        || self.token == SyntaxKind::OpenBraceToken
                        || is_identifier_or_keyword(self.token);
                }
                SyntaxKind::ExportKeyword => {
                    self.next_token();
                    if self.token == SyntaxKind::EqualsToken
                        || self.token == SyntaxKind::AsteriskToken
                        || self.token == SyntaxKind::OpenBraceToken
                        || self.token == SyntaxKind::DefaultKeyword
                        || self.token == SyntaxKind::AsKeyword
                        || self.token == SyntaxKind::AtToken
                    {
                        return true;
                    }
                    if self.token == SyntaxKind::TypeKeyword {
                        self.next_token();
                        return self.token == SyntaxKind::AsteriskToken
                            || self.token == SyntaxKind::OpenBraceToken
                            || (self.is_identifier() && !self.has_preceding_line_break());
                    }
                    return self.is_start_of_declaration();
                }
                _ => return false,
            }
        }
    }

    pub(crate) fn next_token_is_identifier_on_same_line(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        !s.has_preceding_line_break() && is_identifier_or_keyword(s.token())
    }

    pub(crate) fn next_token_is_identifier_or_string_literal_on_same_line(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        !s.has_preceding_line_break()
            && (is_identifier_or_keyword(s.token()) || s.token() == SyntaxKind::StringLiteral)
    }

    pub(crate) fn is_identifier(&self) -> bool {
        self.token == SyntaxKind::Identifier || is_keyword(self.token)
    }

    pub(crate) fn is_binding_identifier_or_pattern(&self) -> bool {
        self.is_identifier()
            || self.token == SyntaxKind::PrivateIdentifier
            || self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::OpenBraceToken
    }

    pub(crate) fn is_start_of_parameter(&self) -> bool {
        self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::OpenBraceToken
            || self.token == SyntaxKind::DotDotDotToken
            || self.is_identifier()
            || self.is_literal_property_name()
    }

    pub(crate) fn is_start_of_type(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::AnyKeyword
                | SyntaxKind::UnknownKeyword
                | SyntaxKind::StringKeyword
                | SyntaxKind::NumberKeyword
                | SyntaxKind::BigIntKeyword
                | SyntaxKind::BooleanKeyword
                | SyntaxKind::UndefinedKeyword
                | SyntaxKind::NeverKeyword
                | SyntaxKind::ObjectKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::ThisKeyword
                | SyntaxKind::TypeOfKeyword
                | SyntaxKind::KeyOfKeyword
                | SyntaxKind::UniqueKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::NewKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::Identifier
                | SyntaxKind::OpenBraceToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::BarToken
                | SyntaxKind::AmpersandToken
                | SyntaxKind::AsteriskToken
                | SyntaxKind::QuestionToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::DotDotDotToken
                | SyntaxKind::MinusToken
                | SyntaxKind::TemplateHead
        ) || is_keyword(self.token)
    }

    pub(crate) fn is_literal_property_name(&self) -> bool {
        is_identifier_or_keyword(self.token)
            || self.token == SyntaxKind::StringLiteral
            || self.token == SyntaxKind::NumericLiteral
            || self.token == SyntaxKind::BigIntLiteral
            || self.token == SyntaxKind::PrivateIdentifier
    }

    pub(crate) fn parse_identifier(&mut self) -> Arc<Node> {
        self.parse_identifier_with_private_diagnostic(None)
    }

    pub(crate) fn parse_identifier_with_private_diagnostic(
        &mut self,
        private_msg: Option<&'static crate::diagnostics::Message>,
    ) -> Arc<Node> {
        if !self.is_identifier() {
            if self.token == SyntaxKind::PrivateIdentifier {
                let msg = private_msg.unwrap_or(
                    &diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                );
                self.parse_error_at_current_token(*msg, &[]);
            } else {
                self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
            }
        }
        let text = self.scanner.token_value();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData { text }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_property_name(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::PrivateIdentifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::PrivateIdentifier,
                    NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::StringLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::StringLiteral,
                    NodeData::StringLiteral(StringLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NumericLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NumericLiteral,
                    NodeData::NumericLiteral(NumericLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::BigIntLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::BigIntLiteral,
                    NodeData::BigIntLiteral(BigIntLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::OpenBracketToken => {
                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_assignment_expression();
                self.expect(SyntaxKind::CloseBracketToken);
                let end = self.token_pos();
                Arc::new(Node::with_loc(
                    SyntaxKind::ComputedPropertyName,
                    NodeData::ComputedPropertyName(ComputedPropertyNameData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_identifier(),
        }
    }

    pub(crate) fn modifier_flag(kind: SyntaxKind) -> ModifierFlags {
        match kind {
            SyntaxKind::ExportKeyword => ModifierFlags::Export,
            SyntaxKind::DeclareKeyword => ModifierFlags::Ambient,
            SyntaxKind::DefaultKeyword => ModifierFlags::Default,
            SyntaxKind::AbstractKeyword => ModifierFlags::Abstract,
            SyntaxKind::StaticKeyword => ModifierFlags::Static,
            SyntaxKind::ReadonlyKeyword => ModifierFlags::Readonly,
            SyntaxKind::PublicKeyword => ModifierFlags::Public,
            SyntaxKind::PrivateKeyword => ModifierFlags::Private,
            SyntaxKind::ProtectedKeyword => ModifierFlags::Protected,
            SyntaxKind::AsyncKeyword => ModifierFlags::Async,
            SyntaxKind::ConstKeyword => ModifierFlags::Const,
            SyntaxKind::AccessorKeyword => ModifierFlags::Accessor,
            _ => ModifierFlags::empty(),
        }
    }
}
