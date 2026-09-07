#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parsing_context_errors(&mut self, context: ParsingContext) {
        match context {
            ParsingContext::SourceElements => {
                if self.token == SyntaxKind::DefaultKeyword {
                    self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &["export"]);
                } else {
                    self.parse_error_at_current_token(
                        diagnostics::DECLARATION_OR_STATEMENT_EXPECTED,
                        &[],
                    );
                }
            }
            ParsingContext::BlockStatements => {
                self.parse_error_at_current_token(
                    diagnostics::DECLARATION_OR_STATEMENT_EXPECTED,
                    &[],
                );
            }
            ParsingContext::SwitchClauses => {
                self.parse_error_at_current_token(diagnostics::X_CASE_OR_DEFAULT_EXPECTED, &[]);
            }
            ParsingContext::SwitchClauseStatements => {
                self.parse_error_at_current_token(diagnostics::STATEMENT_EXPECTED, &[]);
            }
            ParsingContext::RestProperties | ParsingContext::TypeMembers => {
                self.parse_error_at_current_token(diagnostics::PROPERTY_OR_SIGNATURE_EXPECTED, &[]);
            }
            ParsingContext::ClassMembers => {
                self.parse_error_at_current_token(
                    diagnostics::UNEXPECTED_TOKEN_A_CONSTRUCTOR_METHOD_ACCESSOR_OR_PROPERTY_WAS_EXPECTED,
                    &[],
                );
            }
            ParsingContext::EnumMembers => {
                self.parse_error_at_current_token(diagnostics::ENUM_MEMBER_EXPECTED, &[]);
            }
            ParsingContext::HeritageClauseElement => {
                self.parse_error_at_current_token(diagnostics::EXPRESSION_EXPECTED, &[]);
            }
            ParsingContext::VariableDeclarations => {
                if is_keyword_kind(self.token) {
                    self.parse_error_at_current_token(
                        diagnostics::X_0_IS_NOT_ALLOWED_AS_A_VARIABLE_DECLARATION_NAME,
                        &[token_to_string(self.token)],
                    );
                } else {
                    self.parse_error_at_current_token(
                        diagnostics::VARIABLE_DECLARATION_EXPECTED,
                        &[],
                    );
                }
            }
            ParsingContext::ObjectBindingElements => {
                self.parse_error_at_current_token(
                    diagnostics::PROPERTY_DESTRUCTURING_PATTERN_EXPECTED,
                    &[],
                );
            }
            ParsingContext::ArrayBindingElements => {
                self.parse_error_at_current_token(
                    diagnostics::ARRAY_ELEMENT_DESTRUCTURING_PATTERN_EXPECTED,
                    &[],
                );
            }
            ParsingContext::ArgumentExpressions => {
                self.parse_error_at_current_token(diagnostics::ARGUMENT_EXPRESSION_EXPECTED, &[]);
            }
            ParsingContext::ObjectLiteralMembers => {
                self.parse_error_at_current_token(diagnostics::PROPERTY_ASSIGNMENT_EXPECTED, &[]);
            }
            ParsingContext::ArrayLiteralMembers => {
                self.parse_error_at_current_token(diagnostics::EXPRESSION_OR_COMMA_EXPECTED, &[]);
            }
            ParsingContext::JSDocParameters => {
                self.parse_error_at_current_token(diagnostics::PARAMETER_DECLARATION_EXPECTED, &[]);
            }
            ParsingContext::Parameters => {
                if is_keyword_kind(self.token) {
                    self.parse_error_at_current_token(
                        diagnostics::X_0_IS_NOT_ALLOWED_AS_A_PARAMETER_NAME,
                        &[token_to_string(self.token)],
                    );
                } else {
                    self.parse_error_at_current_token(
                        diagnostics::PARAMETER_DECLARATION_EXPECTED,
                        &[],
                    );
                }
            }
            ParsingContext::TypeParameters => {
                self.parse_error_at_current_token(
                    diagnostics::TYPE_PARAMETER_DECLARATION_EXPECTED,
                    &[],
                );
            }
            ParsingContext::TypeArguments => {
                self.parse_error_at_current_token(diagnostics::TYPE_ARGUMENT_EXPECTED, &[]);
            }
            ParsingContext::TupleElementTypes => {
                self.parse_error_at_current_token(diagnostics::TYPE_EXPECTED, &[]);
            }
            ParsingContext::HeritageClauses => {
                self.parse_error_at_current_token(diagnostics::UNEXPECTED_TOKEN_EXPECTED, &[]);
            }
            ParsingContext::ImportOrExportSpecifiers => {
                if self.token == SyntaxKind::FromKeyword {
                    self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &["}"]);
                } else {
                    self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
                }
            }
            ParsingContext::JsxAttributes
            | ParsingContext::JsxChildren
            | ParsingContext::JSDocComment => {
                self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
            }
            ParsingContext::ImportAttributes => {
                self.parse_error_at_current_token(
                    diagnostics::IDENTIFIER_OR_STRING_LITERAL_EXPECTED,
                    &[],
                );
            }
        }
    }

    pub(crate) fn abort_parsing_list_or_move_to_next_token(
        &mut self,
        context: ParsingContext,
    ) -> bool {
        self.parsing_context_errors(context);
        if self.is_in_some_parsing_context() {
            true
        } else {
            self.next_token();
            false
        }
    }

    pub(crate) fn parse_list(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
    ) -> NodeList {
        let pos = self.token_pos();

        let save_contexts = self.parsing_contexts;
        self.parsing_contexts |= 1 << (context as u32);
        let mut nodes = Vec::new();
        while !self.is_list_terminator(context) {
            if self.is_list_element(context, false) {
                let element = parse_element(self);
                nodes.push(element);
            } else if self.abort_parsing_list_or_move_to_next_token(context) {
                break;
            }
        }
        self.parsing_contexts = save_contexts;
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes,
        }
    }

    pub(crate) fn parse_delimited_list(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
    ) -> NodeList {
        let pos = self.token_pos();
        let save_contexts = self.parsing_contexts;
        self.parsing_contexts |= 1 << (context as u32);
        let mut nodes = Vec::new();
        loop {
            if self.is_list_element(context, false) {
                let element_start = self.token_pos();
                let element = parse_element(self);
                nodes.push(element);
                if self.parse_optional(SyntaxKind::CommaToken) {
                    continue;
                }
                if self.is_list_terminator(context) {
                    break;
                }

                self.expect(SyntaxKind::CommaToken);

                if element_start == self.token_pos() {
                    self.next_token();
                }
                continue;
            }
            if self.is_list_terminator(context) {
                break;
            }

            if self.abort_parsing_list_or_move_to_next_token(context) {
                break;
            }
        }
        self.parsing_contexts = save_contexts;
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes,
        }
    }

    #[allow(non_snake_case)]
    pub(crate) fn parse_bracketedList(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
        opening: SyntaxKind,
        closing: SyntaxKind,
    ) -> NodeList {
        if self.parse_optional(opening) {
            let list = self.parse_delimited_list(context, parse_element);
            self.expect(closing);
            list
        } else {
            NodeList::default()
        }
    }

    pub(crate) fn is_list_terminator(&self, context: ParsingContext) -> bool {
        if self.token == SyntaxKind::EndOfFile {
            return true;
        }
        match context {
            ParsingContext::BlockStatements
            | ParsingContext::SwitchClauses
            | ParsingContext::TypeMembers
            | ParsingContext::ClassMembers
            | ParsingContext::EnumMembers
            | ParsingContext::ObjectLiteralMembers
            | ParsingContext::ObjectBindingElements
            | ParsingContext::ImportOrExportSpecifiers
            | ParsingContext::ImportAttributes => self.token == SyntaxKind::CloseBraceToken,
            ParsingContext::SwitchClauseStatements => {
                self.token == SyntaxKind::CloseBraceToken
                    || self.token == SyntaxKind::CaseKeyword
                    || self.token == SyntaxKind::DefaultKeyword
            }
            ParsingContext::HeritageClauseElement => {
                self.token == SyntaxKind::OpenBraceToken
                    || self.token == SyntaxKind::ExtendsKeyword
                    || self.token == SyntaxKind::ImplementsKeyword
            }
            ParsingContext::VariableDeclarations => {
                self.can_parse_semicolon()
                    || self.token == SyntaxKind::InKeyword
                    || self.token == SyntaxKind::OfKeyword
                    || self.token == SyntaxKind::EqualsGreaterThanToken
            }
            ParsingContext::TypeParameters => {
                self.token == SyntaxKind::GreaterThanToken
                    || self.token == SyntaxKind::OpenParenToken
                    || self.token == SyntaxKind::OpenBraceToken
                    || self.token == SyntaxKind::ExtendsKeyword
                    || self.token == SyntaxKind::ImplementsKeyword
            }

            ParsingContext::TypeArguments => self.token != SyntaxKind::CommaToken,
            ParsingContext::ArgumentExpressions => {
                self.token == SyntaxKind::CloseParenToken
                    || self.token == SyntaxKind::SemicolonToken
            }
            ParsingContext::ArrayLiteralMembers
            | ParsingContext::TupleElementTypes
            | ParsingContext::ArrayBindingElements => self.token == SyntaxKind::CloseBracketToken,
            ParsingContext::JSDocParameters
            | ParsingContext::Parameters
            | ParsingContext::RestProperties => {
                self.token == SyntaxKind::CloseParenToken
                    || self.token == SyntaxKind::CloseBracketToken
            }
            ParsingContext::HeritageClauses => {
                self.token == SyntaxKind::OpenBraceToken
                    || self.token == SyntaxKind::CloseBraceToken
            }
            ParsingContext::JsxAttributes => {
                self.token == SyntaxKind::GreaterThanToken || self.token == SyntaxKind::SlashToken
            }
            ParsingContext::JsxChildren => self.token == SyntaxKind::LessThanToken,
            _ => false,
        }
    }
}
