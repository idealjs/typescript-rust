#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn is_yield_expression(&self) -> bool {
        if self.token != SyntaxKind::YieldKeyword {
            return false;
        }
        if self.yield_context {
            return true;
        }

        let mut p = self.clone_state();
        p.next_token();
        if p.token == SyntaxKind::AsteriskToken {
            return true;
        }
        !p.has_preceding_line_break() && p.is_start_of_expression()
    }

    pub(crate) fn is_await_expression(&self) -> bool {
        if self.token != SyntaxKind::AwaitKeyword {
            return false;
        }
        if self.await_context {
            return true;
        }

        let mut p = self.clone_state();
        p.next_token();
        !p.has_preceding_line_break() && p.is_start_of_expression()
    }

    pub(crate) fn parse_yield_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let (asterisk_token, expression) = if !self.has_preceding_line_break()
            && (self.token == SyntaxKind::AsteriskToken || self.is_start_of_expression())
        {
            let asterisk = if self.token == SyntaxKind::AsteriskToken {
                let node = self.create_token_node();
                self.next_token();
                Some(node)
            } else {
                None
            };
            let expr = self.parse_assignment_expression();
            (asterisk, Some(expr))
        } else {
            (None, None)
        };
        let end = expression.as_ref().map_or(self.token_pos(), |e| e.end());
        Arc::new(Node::with_loc(
            SyntaxKind::YieldExpression,
            NodeData::YieldExpression(YieldExpressionData {
                asterisk_token,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn is_parenthesized_arrow_function(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let mut depth = 1usize;
        let mut scanned_tokens = 0usize;
        loop {
            let token = scanner.scan();
            scanned_tokens += 1;
            match token {
                SyntaxKind::EndOfFile => return false,
                SyntaxKind::OpenParenToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenBraceToken => depth += 1,
                SyntaxKind::CloseParenToken => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let next = scanner.scan();
                        if next == SyntaxKind::EqualsGreaterThanToken {
                            return true;
                        }
                        if next == SyntaxKind::ColonToken {
                            return Self::scanner_reaches_arrow_before_line_end(&mut scanner);
                        }

                        if next == SyntaxKind::OpenBraceToken && scanned_tokens == 1 {
                            return true;
                        }
                        return false;
                    }
                }
                SyntaxKind::CloseBracketToken | SyntaxKind::CloseBraceToken => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn scanner_reaches_arrow_before_line_end(scanner: &mut Scanner) -> bool {
        let mut depth = 0usize;
        loop {
            let token = scanner.scan();
            if scanner.has_preceding_line_break() {
                return false;
            }
            match token {
                SyntaxKind::EqualsGreaterThanToken if depth == 0 => return true,
                SyntaxKind::OpenBraceToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenParenToken => depth += 1,
                SyntaxKind::CloseBraceToken
                | SyntaxKind::CloseBracketToken
                | SyntaxKind::CloseParenToken => {
                    depth = depth.saturating_sub(1);
                }
                SyntaxKind::EndOfFile | SyntaxKind::SemicolonToken | SyntaxKind::CommaToken
                    if depth == 0 =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }

    pub(crate) fn is_async_arrow_function(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let next = scanner.scan();
        if scanner.has_preceding_line_break() {
            return false;
        }
        if next == SyntaxKind::OpenParenToken {
            let mut depth = 1usize;
            loop {
                let token = scanner.scan();
                match token {
                    SyntaxKind::EndOfFile => return false,
                    SyntaxKind::OpenParenToken
                    | SyntaxKind::OpenBracketToken
                    | SyntaxKind::OpenBraceToken => depth += 1,
                    SyntaxKind::CloseParenToken => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            let next = scanner.scan();
                            if next == SyntaxKind::EqualsGreaterThanToken {
                                return true;
                            }
                            if next == SyntaxKind::ColonToken {
                                loop {
                                    match scanner.scan() {
                                        SyntaxKind::EqualsGreaterThanToken => return true,
                                        SyntaxKind::EndOfFile | SyntaxKind::SemicolonToken => {
                                            return false;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            return false;
                        }
                    }
                    SyntaxKind::CloseBracketToken | SyntaxKind::CloseBraceToken => {
                        depth = depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
        if is_identifier_or_keyword(next) {
            return scanner.scan() == SyntaxKind::EqualsGreaterThanToken;
        }
        false
    }

    pub(crate) fn make_async_modifier_list(
        &self,
        async_modifier: Arc<Node>,
    ) -> Option<Arc<ModifierList>> {
        Some(Arc::new(ModifierList::new(
            vec![async_modifier],
            ModifierFlags::Async,
        )))
    }

    pub(crate) fn parse_parenthesized_arrow_function_with_async(
        &mut self,
        async_modifier: Arc<Node>,
    ) -> Arc<Node> {
        let modifiers = self.make_async_modifier_list(async_modifier);
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let saved_await = self.await_context;
        self.await_context = true;
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        self.await_context = saved_await;
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers,
                type_parameters: None,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }
}
