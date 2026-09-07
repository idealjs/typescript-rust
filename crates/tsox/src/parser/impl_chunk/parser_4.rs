#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn is_list_element(&self, context: ParsingContext, in_error_recovery: bool) -> bool {
        match context {
            ParsingContext::SourceElements
            | ParsingContext::BlockStatements
            | ParsingContext::SwitchClauseStatements => {
                !(self.token == SyntaxKind::SemicolonToken && in_error_recovery)
                    && self.is_start_of_statement()
            }
            ParsingContext::SwitchClauses => {
                self.token == SyntaxKind::CaseKeyword || self.token == SyntaxKind::DefaultKeyword
            }
            ParsingContext::TypeMembers => !self.is_list_terminator(context),
            ParsingContext::ClassMembers => {
                self.look_ahead_class_member_start()
                    || (self.token == SyntaxKind::SemicolonToken && !in_error_recovery)
            }
            ParsingContext::EnumMembers | ParsingContext::ObjectLiteralMembers => {
                !self.is_list_terminator(context)
            }
            ParsingContext::RestProperties => self.is_literal_property_name(),
            ParsingContext::ObjectBindingElements => {
                self.token == SyntaxKind::OpenBracketToken
                    || self.token == SyntaxKind::DotDotDotToken
                    || self.is_literal_property_name()
            }
            ParsingContext::VariableDeclarations => self.is_binding_identifier_or_pattern(),
            ParsingContext::ArrayBindingElements => {
                self.token == SyntaxKind::CommaToken
                    || self.token == SyntaxKind::DotDotDotToken
                    || self.is_binding_identifier_or_pattern()
            }
            ParsingContext::TypeParameters => {
                is_identifier_or_keyword(self.token)
                    || self.token == SyntaxKind::InKeyword
                    || self.token == SyntaxKind::ConstKeyword
            }
            ParsingContext::ArgumentExpressions => {
                self.token == SyntaxKind::DotDotDotToken || self.is_start_of_expression()
            }
            ParsingContext::ArrayLiteralMembers => {
                if self.token == SyntaxKind::CommaToken || self.token == SyntaxKind::DotToken {
                    return true;
                }
                self.token == SyntaxKind::DotDotDotToken || self.is_start_of_expression()
            }
            ParsingContext::Parameters => self.is_start_of_parameter(),
            ParsingContext::JSDocParameters => self.is_start_of_parameter(),
            ParsingContext::TypeArguments | ParsingContext::TupleElementTypes => {
                self.token == SyntaxKind::CommaToken || self.is_start_of_type()
            }
            ParsingContext::HeritageClauseElement => self.is_start_of_left_hand_side_expression(),
            ParsingContext::HeritageClauses => {
                self.token == SyntaxKind::ExtendsKeyword
                    || self.token == SyntaxKind::ImplementsKeyword
            }
            ParsingContext::ImportOrExportSpecifiers => {
                if self.token == SyntaxKind::FromKeyword
                    && self.look_ahead_token() == SyntaxKind::StringLiteral
                {
                    return false;
                }
                if self.token == SyntaxKind::StringLiteral {
                    return true;
                }
                is_identifier_or_keyword(self.token)
            }
            ParsingContext::ImportAttributes => {
                is_identifier_or_keyword(self.token) || self.token == SyntaxKind::StringLiteral
            }
            ParsingContext::JsxAttributes => {
                is_identifier_or_keyword(self.token) || self.token == SyntaxKind::OpenBraceToken
            }
            ParsingContext::JsxChildren => true,
            ParsingContext::JSDocComment => true,
        }
    }

    pub(crate) fn is_in_some_parsing_context(&self) -> bool {
        const CONTEXTS: [ParsingContext; 26] = [
            ParsingContext::SourceElements,
            ParsingContext::BlockStatements,
            ParsingContext::SwitchClauses,
            ParsingContext::SwitchClauseStatements,
            ParsingContext::TypeMembers,
            ParsingContext::ClassMembers,
            ParsingContext::EnumMembers,
            ParsingContext::HeritageClauseElement,
            ParsingContext::VariableDeclarations,
            ParsingContext::ObjectBindingElements,
            ParsingContext::ArrayBindingElements,
            ParsingContext::ArgumentExpressions,
            ParsingContext::ObjectLiteralMembers,
            ParsingContext::JsxAttributes,
            ParsingContext::JsxChildren,
            ParsingContext::ArrayLiteralMembers,
            ParsingContext::Parameters,
            ParsingContext::JSDocParameters,
            ParsingContext::RestProperties,
            ParsingContext::TypeParameters,
            ParsingContext::TypeArguments,
            ParsingContext::TupleElementTypes,
            ParsingContext::HeritageClauses,
            ParsingContext::ImportOrExportSpecifiers,
            ParsingContext::ImportAttributes,
            ParsingContext::JSDocComment,
        ];
        for kind in CONTEXTS {
            if self.parsing_contexts & (1 << (kind as u32)) != 0
                && (self.is_list_element(kind, true) || self.is_list_terminator(kind))
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn is_start_of_statement(&self) -> bool {
        match self.token {
            SyntaxKind::AtToken
            | SyntaxKind::SemicolonToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::VarKeyword
            | SyntaxKind::LetKeyword
            | SyntaxKind::UsingKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::IfKeyword
            | SyntaxKind::DoKeyword
            | SyntaxKind::WhileKeyword
            | SyntaxKind::ForKeyword
            | SyntaxKind::ContinueKeyword
            | SyntaxKind::BreakKeyword
            | SyntaxKind::ReturnKeyword
            | SyntaxKind::WithKeyword
            | SyntaxKind::SwitchKeyword
            | SyntaxKind::ThrowKeyword
            | SyntaxKind::TryKeyword
            | SyntaxKind::DebuggerKeyword
            | SyntaxKind::CatchKeyword
            | SyntaxKind::FinallyKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::ExportKeyword
            | SyntaxKind::ImportKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::TypeKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::AsyncKeyword
            | SyntaxKind::GlobalKeyword
            | SyntaxKind::DeferKeyword
            | SyntaxKind::AccessorKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::StaticKeyword
            | SyntaxKind::ReadonlyKeyword => true,
            _ => self.is_start_of_expression(),
        }
    }

    pub(crate) fn is_start_of_expression(&self) -> bool {
        if self.is_start_of_left_hand_side_expression() {
            return true;
        }
        matches!(
            self.token,
            SyntaxKind::PlusToken
                | SyntaxKind::MinusToken
                | SyntaxKind::TildeToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::DeleteKeyword
                | SyntaxKind::TypeOfKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::PlusPlusToken
                | SyntaxKind::MinusMinusToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::AwaitKeyword
                | SyntaxKind::YieldKeyword
                | SyntaxKind::PrivateIdentifier
                | SyntaxKind::AtToken
        ) || self.is_identifier()
    }

    pub(crate) fn is_start_of_left_hand_side_expression(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::ThisKeyword
                | SyntaxKind::SuperKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateHead
                | SyntaxKind::OpenParenToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenBraceToken
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::NewKeyword
                | SyntaxKind::SlashToken
                | SyntaxKind::SlashEqualsToken
                | SyntaxKind::Identifier
        ) || self.token == SyntaxKind::ImportKeyword
    }

    #[allow(dead_code)]
    pub(crate) fn is_literal(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::NumericLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateHead
                | SyntaxKind::BigIntLiteral
        )
    }

    pub(crate) fn is_let_declaration(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        let t = s.token();
        Self::is_binding_identifier_token(t)
            || t == SyntaxKind::OpenBraceToken
            || t == SyntaxKind::OpenBracketToken
    }

    pub(crate) fn is_using_declaration(&self) -> bool {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        let next = scanner.token();
        let no_line_break = !scanner.has_preceding_line_break();
        (Self::is_binding_identifier_token(next) || next == SyntaxKind::OpenBraceToken)
            && no_line_break
    }

    pub(crate) fn is_await_using_declaration(&self) -> bool {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        if scanner.token() != SyntaxKind::UsingKeyword {
            return false;
        }
        let no_line_break = !scanner.has_preceding_line_break();
        scanner.scan();
        let next = scanner.token();
        let no_line_break_2 = !scanner.has_preceding_line_break();
        (Self::is_binding_identifier_token(next) || next == SyntaxKind::OpenBraceToken)
            && no_line_break
            && no_line_break_2
    }

    pub(crate) fn is_binding_identifier_token(token: SyntaxKind) -> bool {
        token == SyntaxKind::Identifier || (token as i16) > (SyntaxKind::WithKeyword as i16)
    }

    pub(crate) fn clone_state(&self) -> Parser {
        Parser {
            scanner: self.scanner.clone(),
            token: self.token,
            diagnostics: Vec::new(),
            language_variant: self.language_variant,
            last_template_literal_was_middle: self.last_template_literal_was_middle,
            yield_context: self.yield_context,
            await_context: self.await_context,
            parsing_contexts: self.parsing_contexts,
        }
    }

    pub(crate) fn is_start_of_declaration(&self) -> bool {
        let mut p = self.clone_state();
        p.scan_start_of_declaration()
    }
}
