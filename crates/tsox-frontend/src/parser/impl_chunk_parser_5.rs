#![allow(unused_imports)]

use crate::parser::impl_chunk::*;

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
                SyntaxKind::UsingKeyword => return self.is_using_declaration(),
                SyntaxKind::AwaitKeyword => return self.is_await_using_declaration(),
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
        let token = s.token();
        !s.has_preceding_line_break()
            && !(token == SyntaxKind::YieldKeyword && self.yield_context)
            && !(token == SyntaxKind::AwaitKeyword && self.await_context)
            && !is_reserved_word_kind(token)
            && (token == SyntaxKind::Identifier || is_keyword(token))
    }

    pub(crate) fn next_token_is_identifier_or_string_literal_on_same_line(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        !s.has_preceding_line_break()
            && (is_identifier_or_keyword(s.token()) || s.token() == SyntaxKind::StringLiteral)
    }

    pub(crate) fn is_identifier(&self) -> bool {
        // Go isIdentifier：yield 在 [Yield] 上下文、await 在 [Await] 上下文
        // 视为关键字不可作标识符；绑定位走 isBindingIdentifier（binder 再拒绝）
        if self.token == SyntaxKind::YieldKeyword && self.yield_context {
            return false;
        }
        if self.token == SyntaxKind::AwaitKeyword && self.await_context {
            return false;
        }
        if is_reserved_word_kind(self.token) {
            return false;
        }
        self.token == SyntaxKind::Identifier || is_keyword(self.token)
    }

    pub(crate) fn is_binding_identifier(&self) -> bool {
        if is_reserved_word_kind(self.token) {
            return false;
        }
        self.token == SyntaxKind::Identifier || is_keyword(self.token)
    }

    pub(crate) fn next_is_identifier_and_close_paren(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        let token = s.token();
        let token_is_identifier = !(token == SyntaxKind::YieldKeyword && self.yield_context)
            && !(token == SyntaxKind::AwaitKeyword && self.await_context)
            && !is_reserved_word_kind(token)
            && (token == SyntaxKind::Identifier || is_keyword(token));
        if !token_is_identifier {
            return false;
        }
        s.scan();
        s.token() == SyntaxKind::CloseParenToken
    }

    pub(crate) fn is_binding_identifier_or_pattern(&self) -> bool {
        self.is_binding_identifier()
            || self.token == SyntaxKind::PrivateIdentifier
            || self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::OpenBraceToken
    }

    /// Go isStartOfParameter
    pub(crate) fn is_start_of_parameter(&self, is_jsdoc_parameter: bool) -> bool {
        self.token == SyntaxKind::DotDotDotToken
            || self.is_binding_identifier_or_pattern()
            || crate::ast::node_data_generated::is_modifier_kind(self.token)
            || self.token == SyntaxKind::AtToken
            || self.is_start_of_type_ex(!is_jsdoc_parameter)
    }

    /// Go isStartOfType(inStartOfParameter)
    pub(crate) fn is_start_of_type_ex(&self, in_start_of_parameter: bool) -> bool {
        use SyntaxKind::*;
        match self.token {
            AnyKeyword | UnknownKeyword | StringKeyword | NumberKeyword | BigIntKeyword
            | BooleanKeyword | ReadonlyKeyword | SymbolKeyword | UniqueKeyword | VoidKeyword
            | UndefinedKeyword | NullKeyword | ThisKeyword | TypeOfKeyword | NeverKeyword
            | OpenBraceToken | OpenBracketToken | LessThanToken | BarToken | AmpersandToken
            | NewKeyword | StringLiteral | NumericLiteral | BigIntLiteral | TrueKeyword
            | FalseKeyword | ObjectKeyword | AsteriskToken | QuestionToken | ExclamationToken
            | DotDotDotToken | InferKeyword | ImportKeyword | AssertsKeyword
            | NoSubstitutionTemplateLiteral | TemplateHead => true,
            FunctionKeyword => !in_start_of_parameter,
            MinusToken => !in_start_of_parameter && self.next_token_is_numeric_or_big_int_literal(),
            OpenParenToken => {
                !in_start_of_parameter && self.next_is_parenthesized_or_function_type()
            }
            _ => self.is_identifier(),
        }
    }

    pub(crate) fn next_token_is_numeric_or_big_int_literal(&self) -> bool {
        let mut s = self.scanner.clone();
        let t = s.scan();
        t == SyntaxKind::NumericLiteral || t == SyntaxKind::BigIntLiteral
    }

    pub(crate) fn next_is_parenthesized_or_function_type(&self) -> bool {
        let mut s = self.clone_state();
        s.next_token();
        s.token == SyntaxKind::CloseParenToken
            || s.is_start_of_parameter(false)
            || s.is_start_of_type_ex(false)
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


    /// Go parser contextFlags：节点创建时的 await/yield 上下文
    pub(crate) fn context_flags_now(&self) -> crate::ast::node_flags::NodeFlags {
        let mut flags = crate::ast::node_flags::NodeFlags::empty();
        if self.await_context {
            flags |= crate::ast::node_flags::NodeFlags::AwaitContext;
        }
        if self.yield_context {
            flags |= crate::ast::node_flags::NodeFlags::YieldContext;
        }
        flags
    }

    pub(crate) fn parse_identifier(&mut self) -> Arc<Node> {
        self.parse_identifier_with_private_diagnostic(None)
    }

    /// Go parseBindingIdentifier：绑定位的 await/yield 一律放行（binder 报 TS1359）
    pub(crate) fn parse_binding_identifier_with_private_diagnostic(
        &mut self,
        private_msg: Option<&'static tsox_core::diagnostics::Message>,
    ) -> Arc<Node> {
        if self.is_binding_identifier() {
            let text = self.scanner.token_value();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc_flags(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
                self.context_flags_now(),
            ));
        }
        self.parse_identifier_with_private_diagnostic(private_msg)
    }

    pub(crate) fn parse_identifier_with_private_diagnostic(
        &mut self,
        private_msg: Option<&'static tsox_core::diagnostics::Message>,
    ) -> Arc<Node> {
        if !self.is_identifier() {
            if self.token == SyntaxKind::PrivateIdentifier {
                let msg = private_msg.unwrap_or(
                    &tsox_core::diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                );
                self.parse_error_at_current_token(*msg, &[]);
                // Go createIdentifierWithDiagnostic：报错后返回零宽 missing，
                // 不消费当前 token（由外层 expect/列表恢复推进）
                let pos = self.token_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: String::new(),
                    }),
                    TextRange::new(pos, pos),
                ));
            }
            return self.identifier_expected_error_and_missing();
        }
        let text = self.scanner.token_value();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc_flags(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData { text }),
            TextRange::new(pos, end),
            self.context_flags_now(),
        ))
    }

    /// Go parseRightSideOfDot：`. 后换行 + 标识符/关键字 + 同行再一个
    /// 标识符/关键字` 视为 ASI 断点，返回零宽 missing 且不消费
    /// （`this.\nclass Baz {}` 的 class 属此类）
    pub(crate) fn parse_right_side_of_dot(&mut self) -> Arc<Node> {
        if self.has_preceding_line_break()
            && (self.token == SyntaxKind::Identifier || is_keyword(self.token))
            && self.next_token_is_identifier_or_keyword_on_same_line()
        {
            let pos = self.scanner.full_start_pos();
            self.parse_error_at(
                pos,
                pos,
                tsox_core::diagnostics::IDENTIFIER_EXPECTED,
                &[],
            );
            return self.missing_identifier_at_current();
        }
        if self.token == SyntaxKind::PrivateIdentifier
            || self.token == SyntaxKind::Identifier
            || is_keyword(self.token)
        {
            return self.parse_property_name();
        }
        self.identifier_expected_error_and_missing()
    }

    pub(crate) fn identifier_expected_error_and_missing(&mut self) -> Arc<Node> {
        if is_reserved_word_kind(self.token) {
            let word = self.scanner.token_text().to_string();
            self.parse_error_at_current_token(
                tsox_core::diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                &[&word],
            );
        } else {
            self.parse_error_at_current_token(tsox_core::diagnostics::IDENTIFIER_EXPECTED, &[]);
        }
        self.missing_identifier_at_current()
    }

    fn missing_identifier_at_current(&self) -> Arc<Node> {
        let pos = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: String::new(),
            }),
            TextRange::new(pos, pos),
        ))
    }

    pub(crate) fn next_token_is_identifier_or_keyword_on_same_line(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let kind = scanner.scan();
        is_identifier_or_keyword(kind) && !scanner.has_preceding_line_break()
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
                let expression = self.allow_in(|p| p.parse_assignment_expression());
                self.expect(SyntaxKind::CloseBracketToken);
                let end = self.node_pos();
                Arc::new(Node::with_loc(
                    SyntaxKind::ComputedPropertyName,
                    NodeData::ComputedPropertyName(ComputedPropertyNameData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_identifier_name_or_keyword(),

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
            SyntaxKind::OverrideKeyword => ModifierFlags::Override,
            _ => ModifierFlags::empty(),
        }
    }
}
