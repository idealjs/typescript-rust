#![allow(unused_imports)]

use crate::parser::*;
use crate::scanner::is_line_break;
use crate::scanner::TOKEN_FLAGS_SINGLE_QUOTE;
use tsox_core::core::tristate::Tristate;

pub(crate) fn is_declare_modifier(modifier: &Node) -> bool {
    modifier.kind == SyntaxKind::DeclareKeyword
}

pub(crate) fn is_export_modifier(modifier: &Node) -> bool {
    modifier.kind == SyntaxKind::ExportKeyword
}

pub(crate) fn is_async_modifier(modifier: &Node) -> bool {
    modifier.kind == SyntaxKind::AsyncKeyword
}

pub(crate) fn is_double_quoted_string(node: &Node) -> bool {
    if !is_string_literal(node) {
        return false;
    }
    match &node.data {
        NodeData::StringLiteral(d) => !token_flags_intersects(d.token_flags, TOKEN_FLAGS_SINGLE_QUOTE),
        _ => false,
    }
}

pub(crate) fn is_type_heritage_clause(is_interface: bool, token: SyntaxKind) -> bool {
    is_interface && token == SyntaxKind::ExtendsKeyword
        || !is_interface && token == SyntaxKind::ImplementsKeyword
}

pub(crate) fn modifier_list_has_async(modifiers: &ModifierList) -> bool {
    modifiers.list.nodes.iter().any(|n| is_async_modifier(n))
}

pub(crate) fn is_valid_heritage_type_reference_expression(node: &Node) -> bool {
    if is_identifier(node) {
        return !(node.pos() == node.end() && (node.pos() as i32) >= 0 && node.kind != SyntaxKind::EndOfFile);
    }
    if !is_property_access_expression(node) || is_optional_chain(node) {
        return false;
    }
    match &node.data {
        NodeData::PropertyAccessExpression(pa) => {
            node_is_present(Some(&pa.name)) && is_valid_heritage_type_reference_expression(&pa.expression)
        }
        _ => false,
    }
}

pub(crate) fn match_str(text: &str, pos: usize, s: &str) -> bool {
    text[pos..].starts_with(s)
}

pub(crate) fn line_end_pos(text: &str, pos: usize) -> usize {
    let mut pos = pos;
    for ch in text[pos..].chars() {
        if is_line_break(ch) {
            return pos;
        }
        pos += ch.len_utf8();
    }
    pos
}

pub(crate) struct JSDocScannerInfo(u8);

impl JSDocScannerInfo {
    pub(crate) const NONE: JSDocScannerInfo = JSDocScannerInfo(0);
    pub(crate) const HAS_JSDOC: JSDocScannerInfo = JSDocScannerInfo(1);
    pub(crate) const HAS_DEPRECATED_TAG: JSDocScannerInfo = JSDocScannerInfo(2);
    pub(crate) const HAS_SEE_OR_LINK: JSDocScannerInfo = JSDocScannerInfo(4);

    pub(crate) fn has_jsdoc(self) -> bool {
        self.0 & Self::HAS_JSDOC.0 != 0
    }
}

#[derive(Clone)]
pub(crate) struct ParserState {
    scanner: Scanner,
    token: SyntaxKind,
    language_variant: LanguageVariant,
    javascript_file: bool,
    last_template_literal_was_middle: bool,
    yield_context: bool,
    await_context: bool,
    decorator_context: bool,
    disallow_in_context: bool,
    parsing_contexts: u32,
    diagnostics_len: usize,
}

impl Parser {
    pub(crate) fn mark(&self) -> ParserState {
        ParserState {
            scanner: self.scanner.clone(),
            token: self.token,
            language_variant: self.language_variant,
            javascript_file: self.javascript_file,
            last_template_literal_was_middle: self.last_template_literal_was_middle,
            yield_context: self.yield_context,
            await_context: self.await_context,
            decorator_context: self.decorator_context,
            disallow_in_context: self.disallow_in_context,
            parsing_contexts: self.parsing_contexts,
            diagnostics_len: self.diagnostics.len(),
        }
    }

    pub(crate) fn rewind(&mut self, state: ParserState) {
        self.token = state.token;
        self.language_variant = state.language_variant;
        self.javascript_file = state.javascript_file;
        self.last_template_literal_was_middle = state.last_template_literal_was_middle;
        self.yield_context = state.yield_context;
        self.await_context = state.await_context;
        self.decorator_context = state.decorator_context;
        self.disallow_in_context = state.disallow_in_context;
        self.parsing_contexts = state.parsing_contexts;
        self.diagnostics.truncate(state.diagnostics_len);
        self.scanner = state.scanner;
    }

    pub(crate) fn look_ahead(&mut self, callback: impl FnOnce(&mut Parser) -> bool) -> bool {
        let state = self.mark();
        let result = callback(self);
        self.rewind(state);
        result
    }

    pub(crate) fn next_token_without_check(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan();
        self.token
    }

    pub(crate) fn next_token_jsdoc(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsdoc_token();
        self.token
    }

    pub(crate) fn next_jsdoc_comment_text_token(&mut self, in_backticks: bool) -> SyntaxKind {
        self.token = self.scanner.scan_jsdoc_comment_text_token(in_backticks);
        self.token
    }

    pub(crate) fn jsdoc_scanner_info(&self) -> JSDocScannerInfo {
        if !self.scanner.has_preceding_jsdoc_comment() {
            return JSDocScannerInfo::NONE;
        }
        let mut info = JSDocScannerInfo::HAS_JSDOC;
        if self.scanner.has_preceding_jsdoc_with_deprecated_tag() {
            info = JSDocScannerInfo(info.0 | JSDocScannerInfo::HAS_DEPRECATED_TAG.0);
        }
        if self.scanner.has_preceding_jsdoc_with_see_or_link() {
            info = JSDocScannerInfo(info.0 | JSDocScannerInfo::HAS_SEE_OR_LINK.0);
        }
        info
    }

    pub(crate) fn is_implements_clause(&mut self) -> bool {
        self.token == SyntaxKind::ImplementsKeyword
            && self.look_ahead(|p| is_identifier_or_keyword(p.next_token()))
    }

    pub(crate) fn is_heritage_clause(&self) -> bool {
        self.token == SyntaxKind::ExtendsKeyword || self.token == SyntaxKind::ImplementsKeyword
    }

    pub(crate) fn is_heritage_clause_extends_or_implements_keyword(&mut self) -> bool {
        self.is_heritage_clause()
            && self.look_ahead(|p| {
                p.next_token();
                p.is_start_of_expression()
            })
    }

    pub(crate) fn is_import_attribute_name(&self) -> bool {
        is_identifier_or_keyword(self.token) || self.token == SyntaxKind::StringLiteral
    }

    pub(crate) fn is_parameter_name_start(&self) -> bool {
        self.is_binding_identifier()
            || self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::OpenBraceToken
    }

    pub(crate) fn is_start_of_expression_statement(&self) -> bool {
        self.token != SyntaxKind::OpenBraceToken
            && self.token != SyntaxKind::FunctionKeyword
            && self.token != SyntaxKind::ClassKeyword
            && self.token != SyntaxKind::AtToken
            && self.is_start_of_expression()
    }

    pub(crate) fn is_start_of_function_type_or_constructor_type(&mut self) -> bool {
        self.token == SyntaxKind::LessThanToken
            || self.token == SyntaxKind::OpenParenToken
                && self.look_ahead(|p| p.next_is_unambiguously_start_of_function_type())
            || self.token == SyntaxKind::NewKeyword
            || self.token == SyntaxKind::AbstractKeyword
                && self.look_ahead(|p| p.next_token() == SyntaxKind::NewKeyword)
    }

    pub(crate) fn is_template_start_of_tagged_template(&self) -> bool {
        self.token == SyntaxKind::NoSubstitutionTemplateLiteral
            || self.token == SyntaxKind::TemplateHead
    }

    pub(crate) fn is_update_expression(&self) -> bool {
        match self.token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::TildeToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::DeleteKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::AwaitKeyword => false,
            SyntaxKind::LessThanToken => self.language_variant == LanguageVariant::Jsx,
            _ => true,
        }
    }

    pub(crate) fn is_valid_heritage_clause_object_literal(&mut self) -> bool {
        self.look_ahead(|p| p.next_is_valid_heritage_clause_object_literal())
    }

    /// Go nextIsValidHeritageClauseObjectLiteral
    pub(crate) fn next_is_valid_heritage_clause_object_literal(&mut self) -> bool {
        if self.next_token() == SyntaxKind::CloseBraceToken {
            let next = self.next_token();
            return next == SyntaxKind::CommaToken
                || next == SyntaxKind::OpenBraceToken
                || next == SyntaxKind::ExtendsKeyword
                || next == SyntaxKind::ImplementsKeyword;
        }
        true
    }

    pub(crate) fn is_parenthesized_arrow_function_expression(&mut self) -> Tristate {
        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || self.token == SyntaxKind::AsyncKeyword
        {
            let state = self.mark();
            let result = self.next_is_parenthesized_arrow_function_expression();
            self.rewind(state);
            return result;
        }
        if self.token == SyntaxKind::EqualsGreaterThanToken {
            return Tristate::True;
        }
        Tristate::False
    }

    pub(crate) fn next_is_parenthesized_arrow_function_expression(&mut self) -> Tristate {
        if self.token == SyntaxKind::AsyncKeyword {
            self.next_token();
            if self.has_preceding_line_break() {
                return Tristate::False;
            }
            if self.token != SyntaxKind::OpenParenToken && self.token != SyntaxKind::LessThanToken {
                return Tristate::False;
            }
        }
        let first = self.token;
        let second = self.next_token();
        if first == SyntaxKind::OpenParenToken {
            if second == SyntaxKind::CloseParenToken {
                let third = self.next_token();
                return match third {
                    SyntaxKind::EqualsGreaterThanToken
                    | SyntaxKind::ColonToken
                    | SyntaxKind::OpenBraceToken => Tristate::True,
                    _ => Tristate::False,
                };
            }
            if second == SyntaxKind::OpenBracketToken || second == SyntaxKind::OpenBraceToken {
                return Tristate::Unknown;
            }
            if second == SyntaxKind::DotDotDotToken {
                return Tristate::True;
            }
            if is_modifier_kind(second) && second != SyntaxKind::AsyncKeyword {
                let is_ident_next = self.look_ahead(|p| {
                    p.next_token();
                    p.is_identifier()
                });
                if is_ident_next {
                    if self.next_token() == SyntaxKind::AsKeyword {
                        return Tristate::False;
                    }
                    return Tristate::True;
                }
                return Tristate::False;
            }
            if !self.is_identifier() && second != SyntaxKind::ThisKeyword {
                return Tristate::False;
            }
            match self.next_token() {
                SyntaxKind::ColonToken => return Tristate::True,
                SyntaxKind::QuestionToken => {
                    self.next_token();
                    if self.token == SyntaxKind::ColonToken
                        || self.token == SyntaxKind::CommaToken
                        || self.token == SyntaxKind::EqualsToken
                        || self.token == SyntaxKind::CloseParenToken
                    {
                        return Tristate::True;
                    }
                    return Tristate::False;
                }
                SyntaxKind::CommaToken | SyntaxKind::EqualsToken | SyntaxKind::CloseParenToken => {
                    return Tristate::Unknown;
                }
                _ => return Tristate::False,
            }
        }
        if !self.is_identifier() && self.token != SyntaxKind::ConstKeyword {
            return Tristate::False;
        }
        if self.language_variant == LanguageVariant::Jsx {
            let is_arrow_function_in_jsx = self.look_ahead(|p| {
                p.parse_optional(SyntaxKind::ConstKeyword);
                let third = p.next_token();
                if third == SyntaxKind::ExtendsKeyword {
                    let fourth = p.next_token();
                    return !matches!(
                        fourth,
                        SyntaxKind::EqualsToken
                            | SyntaxKind::GreaterThanToken
                            | SyntaxKind::SlashToken
                    );
                }
                third == SyntaxKind::CommaToken || third == SyntaxKind::EqualsToken
            });
            if is_arrow_function_in_jsx {
                return Tristate::True;
            }
            return Tristate::False;
        }
        Tristate::Unknown
    }

    pub(crate) fn next_is_un_parenthesized_async_arrow_function(&mut self) -> bool {
        if self.token != SyntaxKind::AsyncKeyword {
            return false;
        }
        self.next_token_without_check();
        if self.has_preceding_line_break() || self.token == SyntaxKind::EqualsGreaterThanToken {
            return false;
        }
        if !self.is_identifier() {
            return false;
        }
        self.next_token_without_check();
        !self.has_preceding_line_break() && self.token == SyntaxKind::EqualsGreaterThanToken
    }

    pub(crate) fn next_is_not_dot(&mut self) -> bool {
        self.next_token() != SyntaxKind::DotToken
    }

    pub(crate) fn next_is_start_of_expression(&mut self) -> bool {
        self.next_token();
        self.is_start_of_expression()
    }

    pub(crate) fn next_is_start_of_type(&mut self) -> bool {
        self.next_token();
        self.is_start_of_type_ex(false)
    }

    pub(crate) fn next_is_start_of_type_of_import_type(&mut self) -> bool {
        self.next_token();
        self.token == SyntaxKind::ImportKeyword
    }

    pub(crate) fn make_satisfies_expression(&mut self, expression: &Arc<Node>, type_node: Arc<Node>) -> Arc<Node> {
        let pos = expression.pos();
        let end = type_node.end();
        Arc::new(Node::with_loc(
            SyntaxKind::SatisfiesExpression,
            NodeData::SatisfiesExpression(SatisfiesExpressionData {
                expression: expression.clone(),
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn make_as_expression(&mut self, left: &Arc<Node>, right: Arc<Node>) -> Arc<Node> {
        let pos = left.pos();
        let end = right.end();
        Arc::new(Node::with_loc(
            SyntaxKind::AsExpression,
            NodeData::AsExpression(AsExpressionData {
                expression: left.clone(),
                type_node: right,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn make_binary_expression(
        &mut self,
        left: &Arc<Node>,
        operator_token: Arc<Node>,
        right: &Arc<Node>,
        pos: usize,
    ) -> Arc<Node> {
        let end = right.end();
        Arc::new(Node::with_loc(
            SyntaxKind::BinaryExpression,
            NodeData::BinaryExpression(BinaryExpressionData {
                modifiers: None,
                left: left.clone(),
                type_node: None,
                operator_token,
                right: right.clone(),
            }),
            TextRange::new(pos, end),
        ))
    }
}
