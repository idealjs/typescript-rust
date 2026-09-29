#![allow(unused_imports)]

//! wp1: parser.go 余量收尾批（孪生裁决见文件头注释）

use crate::ast::*;
use crate::parser::ParserDiagnostic;
use crate::parser::parsing_context::Parser;
use crate::scanner::Scanner;
use crate::scanner::{token_flags_intersects, TOKEN_FLAGS_UNTERMINATED};
use crate::ast::mig::m3g_2::is_template_literal_kind;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::{self, Message};

// 孪生/架构裁决（Go 名 → Rust 名/位置）：
// GetJSDocCommentRanges → jsdoc_parser_9.rs get_jsdoc_comment_ranges
// canFollowModifier → members_parser_2.rs token_can_follow_modifier
// isJavaScript → Parser.javascript_file 字段（m4b.rs is_javascript）
// isJSDocLikeText → jsdoc_parser_9.rs is_jsdoc_like_text
// isReservedWord → binary_precedence.rs is_reserved_word_kind
// match → m4b.rs match_str
// nextJSDocCommentTextToken → jsdoc_tokens.rs next_jsdoc_comment_text_token
// nextTokenJSDoc → jsdoc_tokens.rs next_token_jsdoc
// nextIsValidHeritageClauseObjectLiteral → m4b.rs is_valid_heritage_clause_object_literal
// parseExpected → impl_chunk_parser_2.rs expect_report
// parseExpectedJSDoc → jsdoc_tokens.rs parse_expected_jsdoc
// parseExpectedTokenJSDoc → jsdoc_tokens.rs parse_expected_token_jsdoc
// parseArrayLiteralExpression → expressions_parser_6.rs parse_array_literal
// parseArgumentOrArrayLiteralElement → expressions_parser_6.rs parse_array_literal_element
// parseAssignmentExpressionOrHigher → expressions_parser.rs parse_assignment_expression
// parseAssignmentExpressionOrHigherWorker → 内联于 parse_assignment_expression
// parseBinaryExpressionOrHigher → expressions_parser_3.rs parse_binary_expression
// parseBindingIdentifier → impl_chunk_parser_5.rs parse_binding_identifier_with_private_diagnostic
// parseClassDeclarationOrExpression → expressions_parser.rs parse_class_expression 与 statements 侧类声明解析拆分
// parseClassElement → members_parser_4.rs parse_class_member
// parseDeclaration → declarations_parser.rs parse_declaration_with_modifiers
// parseDeclarationWorker → 内联于 parse_declaration_with_modifiers
// parseDecoratorExpression → impl_chunk_parser_6.rs parse_decorator
// parseElementAccessExpressionRest → expressions_parser_5.rs parse_element_access_argument
// createMissingIdentifier → impl_chunk_parser_2.rs missing_identifier_expression
// finishNode/finishNodeWithEnd → 架构差异：Node::with_loc 构造时定界，无独立 finish 步骤
// doInContext → 架构差异：上下文以独立布尔字段内联 save/restore
// overrideParentInImmediateChildren → 架构差异：父指针由 ast 层维护
// getParser/putParser/newParser → 架构差异：无对象池，Parser 字段直构
// createJSDocCache → 架构差异：jsdoc 经 with_jsdoc 依附节点
// isMissingNodeList/createMissingList → 架构差异：缺失列表以 Option/missing 标记表达

pub(crate) fn attach_file_to_diagnostics(mut diagnostics: Vec<ParserDiagnostic>) -> Vec<ParserDiagnostic> {
    diagnostics
}

impl Parser {
    pub(crate) fn can_follow_get_or_set_keyword(&self) -> bool {
        self.token == SyntaxKind::OpenBracketToken || self.is_literal_property_name()
    }

    pub(crate) fn in_await_context(&self) -> bool {
        self.await_context
    }

    pub(crate) fn in_decorator_context(&self) -> bool {
        self.decorator_context
    }

    pub(crate) fn in_disallow_in_context(&self) -> bool {
        self.disallow_in_context
    }

    pub(crate) fn in_yield_context(&self) -> bool {
        self.yield_context
    }

    pub(crate) fn is_binding_identifier_or_private_identifier_or_pattern(&self) -> bool {
        self.token == SyntaxKind::OpenBraceToken
            || self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::PrivateIdentifier
            || self.is_binding_identifier()
    }

    pub(crate) fn is_index_signature(&mut self) -> bool {
        self.token == SyntaxKind::OpenBracketToken
            && self.look_ahead(Parser::next_is_unambiguously_index_signature)
    }

    pub(crate) fn next_is_unambiguously_index_signature(&mut self) -> bool {
        self.next_token();
        if self.token == SyntaxKind::DotDotDotToken || self.token == SyntaxKind::CloseBracketToken {
            return true;
        }
        if is_modifier_kind(self.token) {
            self.next_token();
            if self.is_identifier() {
                return true;
            }
        } else if !self.is_identifier() {
            return false;
        } else {
            self.next_token();
        }
        if self.token == SyntaxKind::ColonToken || self.token == SyntaxKind::CommaToken {
            return true;
        }
        if self.token != SyntaxKind::QuestionToken {
            return false;
        }
        self.next_token();
        self.token == SyntaxKind::ColonToken
            || self.token == SyntaxKind::CommaToken
            || self.token == SyntaxKind::QuestionToken
            || self.token == SyntaxKind::CloseBracketToken
    }

    pub(crate) fn next_is_unambiguously_start_of_function_type(&mut self) -> bool {
        self.next_token();
        if self.token == SyntaxKind::CloseParenToken || self.token == SyntaxKind::DotDotDotToken {
            return true;
        }
        if self.skip_parameter_start() {
            if self.token == SyntaxKind::ColonToken
                || self.token == SyntaxKind::CommaToken
                || self.token == SyntaxKind::QuestionToken
                || self.token == SyntaxKind::EqualsToken
            {
                return true;
            }
            if self.token == SyntaxKind::CloseParenToken && self.next_token() == SyntaxKind::EqualsGreaterThanToken {
                return true;
            }
        }
        false
    }

    pub(crate) fn next_is_using_keyword_then_binding_identifier_or_start_of_object_destructuring_on_same_line(
        &mut self,
    ) -> bool {
        self.next_token() == SyntaxKind::UsingKeyword
            && self.next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(false)
    }

    pub(crate) fn next_token_is_binding_identifier_or_start_of_destructuring(&mut self) -> bool {
        self.next_token();
        self.is_binding_identifier()
            || self.token == SyntaxKind::OpenBraceToken
            || self.token == SyntaxKind::OpenBracketToken
    }

    pub(crate) fn next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(
        &mut self,
        disallow_of: bool,
    ) -> bool {
        self.next_token();
        if disallow_of && self.token == SyntaxKind::OfKeyword {
            return false;
        }
        (self.is_binding_identifier()
            || self.token == SyntaxKind::OpenBraceToken
            || self.token == SyntaxKind::OpenBracketToken)
            && !self.has_preceding_line_break()
    }

    pub(crate) fn next_token_is_binding_identifier_or_start_of_destructuring_on_same_line_disallow_of(
        &mut self,
    ) -> bool {
        self.next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(true)
    }

    pub(crate) fn next_token_is_class_keyword_on_same_line(&mut self) -> bool {
        self.next_token() == SyntaxKind::ClassKeyword && !self.has_preceding_line_break()
    }

    pub(crate) fn next_token_is_colon_or_question_colon(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::ColonToken
    }

    pub(crate) fn next_token_is_dot(&mut self) -> bool {
        self.next_token() == SyntaxKind::DotToken
    }

    pub(crate) fn next_token_is_equals_or_semicolon_or_colon_token(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::EqualsToken
            || self.token == SyntaxKind::SemicolonToken
            || self.token == SyntaxKind::ColonToken
    }

    pub(crate) fn next_token_is_from_keyword_or_equals_token(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::FromKeyword || self.token == SyntaxKind::EqualsToken
    }

    pub(crate) fn next_token_is_function_keyword_on_same_line(&mut self) -> bool {
        self.next_token() == SyntaxKind::FunctionKeyword && !self.has_preceding_line_break()
    }

    pub(crate) fn next_token_is_identifier_or_keyword(&mut self) -> bool {
        self.next_token() == SyntaxKind::Identifier
            || (self.token as u32) > (SyntaxKind::WithKeyword as u32)
    }

    pub(crate) fn next_token_is_identifier_or_keyword_or_greater_than(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::Identifier
            || (self.token as u32) > (SyntaxKind::WithKeyword as u32)
            || self.token == SyntaxKind::GreaterThanToken
    }

    pub(crate) fn next_token_is_identifier_or_keyword_or_literal_on_same_line(&mut self) -> bool {
        self.next_token();
        (self.token == SyntaxKind::StringLiteral
            || self.token == SyntaxKind::NumericLiteral
            || token_is_identifier_or_keyword(self.token))
            && !self.has_preceding_line_break()
    }

    pub(crate) fn next_token_is_identifier_or_keyword_or_open_bracket_or_template(&mut self) -> bool {
        self.next_token();
        token_is_identifier_or_keyword(self.token)
            || self.token == SyntaxKind::OpenBracketToken
            || is_template_literal_kind(self.token)
    }

    pub(crate) fn next_token_is_new_keyword(&mut self) -> bool {
        self.next_token() == SyntaxKind::NewKeyword
    }

    pub(crate) fn next_token_is_on_same_line_and_can_follow_modifier(&mut self) -> bool {
        self.next_token();
        if self.has_preceding_line_break() {
            return false;
        }
        Parser::token_can_follow_modifier(self.token)
    }

    pub(crate) fn next_token_is_open_brace(&mut self) -> bool {
        self.next_token() == SyntaxKind::OpenBraceToken
    }

    pub(crate) fn next_token_is_open_paren(&mut self) -> bool {
        self.next_token() == SyntaxKind::OpenParenToken
    }

    pub(crate) fn next_token_is_open_paren_or_less_than(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::OpenParenToken || self.token == SyntaxKind::LessThanToken
    }

    pub(crate) fn next_token_is_open_paren_or_less_than_or_dot(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || self.token == SyntaxKind::DotToken
    }

    pub(crate) fn next_token_is_slash(&mut self) -> bool {
        self.next_token() == SyntaxKind::SlashToken
    }

    pub(crate) fn next_token_is_token_string_literal(&mut self) -> bool {
        self.next_token() == SyntaxKind::StringLiteral
    }

    pub(crate) fn get_template_literal_raw_text(&self, end_length: usize) -> String {
        let token_text = self.scanner.token_text().to_string();
        let end_length = if token_flags_intersects(self.scanner.token_flags(), TOKEN_FLAGS_UNTERMINATED) {
            0
        } else {
            end_length
        };
        token_text[1..token_text.len() - end_length].to_string()
    }
}

pub(crate) fn token_is_identifier_or_keyword(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || (token as u32) > (SyntaxKind::WithKeyword as u32)
}
