use super::*;

impl Parser {
    pub(crate) fn is_import_meta(&self) -> bool {
        if self.token != SyntaxKind::ImportKeyword {
            return false;
        }
        let mut scanner = self.scanner.clone();
        scanner.scan() == SyntaxKind::DotToken && !scanner.has_preceding_line_break()
    }

    pub(crate) fn parse_import_meta(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        self.next_token();
        let name = self.parse_identifier_name_or_keyword();
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::MetaProperty,
            NodeData::MetaProperty(MetaPropertyData {
                keyword_token: SyntaxKind::ImportKeyword,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn is_async_function_expression(&self) -> bool {
        if self.token != SyntaxKind::AsyncKeyword {
            return false;
        }
        let mut scanner = self.scanner.clone();
        scanner.scan() == SyntaxKind::FunctionKeyword && !scanner.has_preceding_line_break()
    }

    pub(crate) fn parse_async_function_expression(&mut self) -> Arc<Node> {
        let async_modifier = self.create_token_node();
        self.next_token();
        let pos = async_modifier.pos();

        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let is_generator = asterisk_token.is_some();
        let body = self.parse_function_block(is_generator, true);
        let end = body.end();
        let modifiers = self.make_async_modifier_list(async_modifier);
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(FunctionExpressionData {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_function_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = self.parse_block();
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(FunctionExpressionData {
                modifiers: None,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_class_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();

        let name = if self.is_identifier()
            && !matches!(self.token, SyntaxKind::ExtendsKeyword | SyntaxKind::ImplementsKeyword)
        {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        let members = self.parse_class_members();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ClassExpression,
            NodeData::ClassExpression(ClassExpressionData {
                modifiers: None,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub fn parse_expression(&mut self) -> Arc<Node> {
        let expr = self.parse_assignment_expression();

        if self.token == SyntaxKind::CommaToken {
            let pos = expr.pos();
            let mut left = expr;
            loop {
                let comma_pos = self.token_pos();
                let comma_end = self.token_end();
                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
                let right = self.parse_assignment_expression();
                let end = right.end();
                left = Arc::new(Node::with_loc(
                    SyntaxKind::BinaryExpression,
                    NodeData::BinaryExpression(BinaryExpressionData {
                        modifiers: None,
                        left,
                        type_node: None,
                        operator_token: Arc::new(Node::with_loc(
                            SyntaxKind::CommaToken,
                            NodeData::Token,
                            TextRange::new(comma_pos, comma_end),
                        )),
                        right,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            return left;
        }
        expr
    }

    pub(crate) fn parse_assignment_expression(&mut self) -> Arc<Node> {

        if self.is_yield_expression() {
            return self.parse_yield_expression();
        }

        if self.token == SyntaxKind::LessThanToken
            || (self.token == SyntaxKind::AsyncKeyword
                && self.look_ahead_token() == SyntaxKind::LessThanToken)
        {
            if let Some(arrow) = self.try_parse_generic_arrow_function() {
                return arrow;
            }
        }
        if self.token == SyntaxKind::AsyncKeyword && self.is_async_arrow_function() {

            let async_modifier = self.create_token_node();
            self.next_token();
            if self.token == SyntaxKind::OpenParenToken {
                return self.parse_parenthesized_arrow_function_with_async(async_modifier);
            }
            let identifier = self.parse_identifier();
            return self.parse_simple_arrow_function_with_async(identifier, async_modifier);
        }

        if self.token == SyntaxKind::OpenParenToken && self.is_parenthesized_arrow_function() {
            return self.parse_parenthesized_arrow_function();
        }

        let mut expr = self.parse_binary_expression(0);
        if expr.kind == SyntaxKind::Identifier && self.token == SyntaxKind::EqualsGreaterThanToken {
            return self.parse_simple_arrow_function(expr);
        }

        while self.token == SyntaxKind::AsKeyword || self.token == SyntaxKind::SatisfiesKeyword {
            let pos = expr.pos();
            let kind = self.token;
            self.next_token();

            let type_node = if kind == SyntaxKind::AsKeyword
                && self.token == SyntaxKind::ConstKeyword
                && !self.has_preceding_line_break()
            {
                let tp = self.token_pos();
                let te = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::ConstKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(tp, te),
                ))
            } else {
                self.parse_type()
            };
            let end = type_node.end();
            expr = match kind {
                SyntaxKind::AsKeyword => Arc::new(Node::with_loc(
                    SyntaxKind::AsExpression,
                    NodeData::AsExpression(AsExpressionData {
                        expression: expr,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                )),
                _ => Arc::new(Node::with_loc(
                    SyntaxKind::SatisfiesExpression,
                    NodeData::SatisfiesExpression(SatisfiesExpressionData {
                        expression: expr,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                )),
            };
        }

        if self.token == SyntaxKind::QuestionToken {
            let pos = expr.pos();
            let question_token = self.create_token_node();
            self.next_token();
            let when_true = self.parse_expression();
            let colon_token = self.create_token_node();
            self.expect(SyntaxKind::ColonToken);
            let when_false = self.parse_assignment_expression();
            let end = when_false.end();
            expr = Arc::new(Node::with_loc(
                SyntaxKind::ConditionalExpression,
                NodeData::ConditionalExpression(ConditionalExpressionData {
                    condition: expr,
                    question_token,
                    when_true,
                    colon_token,
                    when_false,
                }),
                TextRange::new(pos, end),
            ));
        }

        if is_assignment_operator(self.token) {
            let pos = expr.pos();
            let operator_token = self.create_token_node();
            self.next_token();
            let right = self.parse_assignment_expression();
            let end = right.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::BinaryExpression,
                NodeData::BinaryExpression(BinaryExpressionData {
                    modifiers: None,
                    left: expr,
                    type_node: None,
                    operator_token,
                    right,
                }),
                TextRange::new(pos, end),
            ));
        }
        expr
    }

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
                    return false
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
                                        SyntaxKind::EndOfFile
                                        | SyntaxKind::SemicolonToken => return false,
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

    pub(crate) fn make_async_modifier_list(&self, async_modifier: Arc<Node>) -> Option<Arc<ModifierList>> {
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

    pub(crate) fn try_parse_generic_arrow_function(&mut self) -> Option<Arc<Node>> {
        let starts_with_async = self.token == SyntaxKind::AsyncKeyword;
        if !starts_with_async
            && (self.token != SyntaxKind::LessThanToken
                || self.language_variant == LanguageVariant::Jsx)
        {
            return None;
        }

        {
            let mut s = self.scanner.clone();
            if starts_with_async {
                let after = s.scan();
                if s.has_preceding_line_break() || after != SyntaxKind::LessThanToken {
                    return None;
                }
            }
            let t1 = s.scan();
            if !(t1 == SyntaxKind::Identifier
                || t1 == SyntaxKind::ConstKeyword
                || (t1 as i16) > (SyntaxKind::WithKeyword as i16))
            {
                return None;
            }
        }

        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();

        if starts_with_async {
            self.next_token();
        }

        let type_parameters = self.parse_optional_type_parameters();

        if type_parameters.is_none()
            || self.token != SyntaxKind::OpenParenToken
            || self.diagnostics.len() != diag_len
        {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();

        if self.token != SyntaxKind::EqualsGreaterThanToken || self.diagnostics.len() != diag_len {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }

        let equals_greater_than_token = self.create_token_node();
        self.next_token();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Some(Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        )))
    }

    pub(crate) fn parse_simple_arrow_function_with_async(
        &mut self,
        identifier: Arc<Node>,
        async_modifier: Arc<Node>,
    ) -> Arc<Node> {
        let modifiers = self.make_async_modifier_list(async_modifier);
        let pos = identifier.pos();
        let parameter = Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(pos, self.token_pos()),
        ));
        let parameters = Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: vec![parameter],
        });
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
                type_node: None,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_parenthesized_arrow_function(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
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

    pub(crate) fn parse_simple_arrow_function(&mut self, identifier: Arc<Node>) -> Arc<Node> {
        let pos = identifier.pos();
        let parameter = Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(pos, self.token_pos()),
        ));
        let parameters = Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: vec![parameter],
        });
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters: None,
                parameters,
                type_node: None,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_binary_expression(&mut self, min_precedence: u8) -> Arc<Node> {
        let mut left = self.parse_unary_expression();

        loop {
            let precedence = binary_precedence(self.token);
            if precedence == 0 || precedence < min_precedence {
                break;
            }
            let operator_token = self.create_token_node();
            self.next_token();
            let right = self.parse_binary_expression(precedence + 1);
            let loc = TextRange::new(left.pos(), right.end());
            left = Arc::new(Node::with_loc(
                SyntaxKind::BinaryExpression,
                NodeData::BinaryExpression(BinaryExpressionData {
                    modifiers: None,
                    left,
                    type_node: None,
                    operator_token,
                    right,
                }),
                loc,
            ));
        }

        left
    }

    pub(crate) fn parse_unary_expression(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::TildeToken
            | SyntaxKind::PlusPlusToken
            | SyntaxKind::MinusMinusToken => {
                let operator = self.token;
                let op_pos = self.token_pos();
                self.next_token();
                let operand = self.parse_unary_expression();
                let loc = TextRange::new(op_pos, operand.end());
                Arc::new(Node::with_loc(
                    SyntaxKind::PrefixUnaryExpression,
                    NodeData::PrefixUnaryExpression(PrefixUnaryExpressionData {
                        operator,
                        operand,
                    }),
                    loc,
                ))
            }
            SyntaxKind::TypeOfKeyword => {

                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::TypeOfExpression,
                    NodeData::TypeOfExpression(TypeOfExpressionData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::VoidKeyword | SyntaxKind::DeleteKeyword => {
                let pos = self.token_pos();
                let is_delete = self.token == SyntaxKind::DeleteKeyword;
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                if is_delete {
                    Arc::new(Node::with_loc(
                        SyntaxKind::DeleteExpression,
                        NodeData::DeleteExpression(DeleteExpressionData { expression }),
                        TextRange::new(pos, end),
                    ))
                } else {
                    Arc::new(Node::with_loc(
                        SyntaxKind::VoidExpression,
                        NodeData::VoidExpression(VoidExpressionData { expression }),
                        TextRange::new(pos, end),
                    ))
                }
            }
            SyntaxKind::AwaitKeyword if self.is_await_expression() => {
                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::AwaitExpression,
                    NodeData::AwaitExpression(AwaitExpressionData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::LessThanToken if self.language_variant != LanguageVariant::Jsx => {

                self.parse_type_assertion()
            }
            _ => self.parse_postfix_expression(),
        }
    }

    pub(crate) fn parse_type_assertion(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let type_node = self.parse_type();
        self.expect(SyntaxKind::GreaterThanToken);
        let expression = self.parse_unary_expression();
        let end = expression.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeAssertionExpression,
            NodeData::TypeAssertion(TypeAssertionData {
                type_node,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_postfix_expression(&mut self) -> Arc<Node> {
        let operand = self.parse_left_hand_side_expression();

        if !self.has_preceding_line_break()
            && (self.token == SyntaxKind::PlusPlusToken
                || self.token == SyntaxKind::MinusMinusToken)
        {
            let pos = operand.pos();
            let operator = self.token;
            let op_end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc(
                SyntaxKind::PostfixUnaryExpression,
                NodeData::PostfixUnaryExpression(PostfixUnaryExpressionData { operand, operator }),
                TextRange::new(pos, op_end),
            ));
        }
        operand
    }

    pub(crate) fn parse_left_hand_side_expression(&mut self) -> Arc<Node> {
        let expr = if self.token == SyntaxKind::NewKeyword {
            self.parse_new_expression()
        } else {
            self.parse_primary_expression()
        };
        self.parse_call_and_member_chain(expr, false)
    }

    pub(crate) fn parse_member_chain(&mut self, expr: Arc<Node>) -> Arc<Node> {
        self.parse_call_and_member_chain(expr, true)
    }

    pub(crate) fn parse_new_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let expression = if self.token == SyntaxKind::DotToken {
            self.next_token();
            let name = self.parse_identifier();
            let end = name.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(pos, pos),
                    )),
                    question_dot_token: None,
                    name,
                }),
                TextRange::new(pos, end),
            ))
        } else {

            let primary = if self.token == SyntaxKind::NewKeyword {
                self.parse_new_expression()
            } else {
                self.parse_primary_expression()
            };
            self.parse_member_chain(primary)
        };
        let type_arguments = self.parse_optional_type_arguments();
        let arguments = if self.token == SyntaxKind::OpenParenToken {
            Some(self.parse_argument_list())
        } else {
            None
        };
        let end = arguments.as_ref().map_or(expression.end(), |a| a.end());
        Arc::new(Node::with_loc(
            SyntaxKind::NewExpression,
            NodeData::NewExpression(NewExpressionData {
                expression,
                type_arguments,
                arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_call_and_member_chain(&mut self, expr: Arc<Node>, member_only: bool) -> Arc<Node> {
        let mut expr = expr;
        loop {

            if member_only
                && matches!(
                    self.token,
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                )
            {
                break;
            }
            match self.token {
                SyntaxKind::DotToken => {
                    let pos = expr.pos();
                    self.next_token();
                    let name = self.parse_property_name();
                    let end = name.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::PropertyAccessExpression,
                        NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            name,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::QuestionDotToken => {
                    let pos = expr.pos();
                    let question_dot = self.create_token_node();
                    self.next_token();
                    if self.token == SyntaxKind::OpenParenToken {
                        let arguments = self.parse_argument_list();
                        let end = arguments.end();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::CallExpression,
                            NodeData::CallExpression(CallExpressionData {
                                expression: expr,
                                question_dot_token: Some(question_dot),
                                type_arguments: None,
                                arguments,
                            }),
                            TextRange::new(pos, end),
                        ));
                    } else if self.token == SyntaxKind::OpenBracketToken {

                        self.next_token();
                        let argument = self.parse_expression();
                        self.expect(SyntaxKind::CloseBracketToken);
                        let end = self.token_pos();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::ElementAccessExpression,
                            NodeData::ElementAccessExpression(ElementAccessExpressionData {
                                expression: expr,
                                question_dot_token: Some(question_dot),
                                argument_expression: argument,
                            }),
                            TextRange::new(pos, end),
                        ));
                    } else {
                        let name = self.parse_property_name();
                        let end = name.end();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::PropertyAccessExpression,
                            NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                                expression: expr,
                                question_dot_token: Some(question_dot),
                                name,
                            }),
                            TextRange::new(pos, end),
                        ));
                    }
                }
                SyntaxKind::OpenParenToken => {
                    let pos = expr.pos();
                    let arguments = self.parse_argument_list();
                    let end = arguments.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::CallExpression,
                        NodeData::CallExpression(CallExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            type_arguments: None,
                            arguments,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::OpenBracketToken => {
                    let pos = expr.pos();
                    self.next_token();
                    let argument = self.parse_expression();
                    self.expect(SyntaxKind::CloseBracketToken);
                    let end = self.token_pos();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::ElementAccessExpression,
                        NodeData::ElementAccessExpression(ElementAccessExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            argument_expression: argument,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::LessThanToken => {
                    let pos = expr.pos();

                    let type_arguments = match self.try_parse_type_arguments(true) {
                        Some(ta) => ta,
                        None => break,
                    };
                    let arguments = self.parse_argument_list();
                    let end = arguments.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::CallExpression,
                        NodeData::CallExpression(CallExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            type_arguments: Some(type_arguments),
                            arguments,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::ExclamationToken if !self.has_preceding_line_break() => {

                    let pos = expr.pos();
                    self.next_token();
                    let end = self.token_pos();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::NonNullExpression,
                        NodeData::NonNullExpression(NonNullExpressionData { expression: expr }),
                        TextRange::new(pos, end),
                    ));
                }
                _ => break,
            }
        }
        expr
    }

    pub(crate) fn parse_argument_list(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenParenToken);
        let nodes =
            self.parse_delimited_list(ParsingContext::ArgumentExpressions, Parser::parse_argument);
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: nodes.nodes,
        })
    }

    pub(crate) fn parse_argument(&mut self) -> Arc<Node> {
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }
        self.parse_assignment_expression()
    }

    pub(crate) fn parse_optional_type_arguments(&mut self) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.token_pos();
        self.next_token();
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);

        self.re_scan_greater_than();
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }

    pub(crate) fn try_parse_type_arguments(&mut self, require_following_paren: bool) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();
        self.next_token();
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);
        self.re_scan_greater_than();
        let closed_cleanly =
            self.token == SyntaxKind::GreaterThanToken && self.diagnostics.len() == diag_len;
        if !closed_cleanly {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        self.next_token();
        if require_following_paren && self.token != SyntaxKind::OpenParenToken {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }

    pub(crate) fn parse_primary_expression(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::Identifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData { text }),
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
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NoSubstitutionTemplateLiteral,
                    NodeData::NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteralData {
                        text,
                        template_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NullKeyword => self.parse_keyword_expression(SyntaxKind::NullKeyword),
            SyntaxKind::TrueKeyword => self.parse_keyword_expression(SyntaxKind::TrueKeyword),
            SyntaxKind::FalseKeyword => self.parse_keyword_expression(SyntaxKind::FalseKeyword),
            SyntaxKind::UndefinedKeyword => {
                self.parse_keyword_expression(SyntaxKind::UndefinedKeyword)
            }
            SyntaxKind::ThisKeyword => self.parse_keyword_expression(SyntaxKind::ThisKeyword),
            SyntaxKind::SuperKeyword => self.parse_keyword_expression(SyntaxKind::SuperKeyword),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_arrow(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::LessThanToken if self.language_variant == LanguageVariant::Jsx => {
                self.parse_jsx_element_or_fragment(true)
            }
            SyntaxKind::FunctionKeyword => self.parse_function_expression(),
            SyntaxKind::ClassKeyword => self.parse_class_expression(),

            SyntaxKind::ImportKeyword => {
                if matches!(
                    self.look_ahead_token(),
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                ) {
                    self.parse_keyword_expression(SyntaxKind::ImportKeyword)
                } else if self.is_import_meta() {
                    self.parse_import_meta()
                } else {
                    self.parse_fallback_identifier_or_error()
                }
            }

            SyntaxKind::AsyncKeyword if self.is_async_function_expression() => {
                self.parse_async_function_expression()
            }
            SyntaxKind::TemplateHead => self.parse_template_expression(),

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
            SyntaxKind::SlashToken | SyntaxKind::SlashEqualsToken => {
                self.re_scan_slash_token();
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::RegularExpressionLiteral,
                    NodeData::RegularExpressionLiteral(RegularExpressionLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            _ => {

                self.parse_fallback_identifier_or_error()
            }
        }
    }

    pub(crate) fn parse_fallback_identifier_or_error(&mut self) -> Arc<Node> {
        if is_identifier_or_keyword(self.token)
            && self.token != SyntaxKind::InKeyword
            && self.token != SyntaxKind::InstanceOfKeyword
        {
            let text = self.scanner.token_text().to_string();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
            ))
        } else {
            let pos = self.token_pos();
            let end = self.token_end();
            self.parse_error_at(pos, end, diagnostics::EXPRESSION_EXPECTED, &[]);
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(pos, pos),
            ))
        }
    }

    pub(crate) fn parse_keyword_expression(&mut self, kind: SyntaxKind) -> Arc<Node> {
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            kind,
            NodeData::Token,
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_parenthesized_or_arrow(&mut self) -> Arc<Node> {
        if self.is_parenthesized_arrow_function() {
            return self.parse_parenthesized_arrow_function();
        }

        let pos = self.token_pos();
        self.next_token();

        let expr = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();

        if self.token == SyntaxKind::EqualsGreaterThanToken {
            let arrow_token = self.create_token_node();
            self.next_token();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                self.parse_block()
            } else {
                self.parse_assignment_expression()
            };
            let end = body.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ArrowFunction,
                NodeData::ArrowFunction(ArrowFunctionData {
                    modifiers: None,
                    type_parameters: None,
                    parameters: Arc::new(NodeList::default()),
                    type_node: None,
                    equals_greater_than_token: arrow_token,
                    body,
                    full_signature: None,
                }),
                TextRange::new(pos, end),
            ));
        }

        Arc::new(Node::with_loc(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ParenthesizedExpressionData { expression: expr }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_array_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ArrayLiteralMembers,
            Parser::parse_array_literal_element,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(ArrayLiteralExpressionData {
                elements: Arc::new(elements),
                multi_line: false,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_array_literal_element(&mut self) -> Arc<Node> {
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }
        self.parse_assignment_expression()
    }

    pub(crate) fn parse_object_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_delimited_list(
            ParsingContext::ObjectLiteralMembers,
            Parser::parse_object_literal_element,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ObjectLiteralExpression,
            NodeData::ObjectLiteralExpression(ObjectLiteralExpressionData {
                properties: Arc::new(members),
                multi_line: false,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_object_literal_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        if dot_dot_dot_token.is_some() {
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                self.next_token();
                let name = self.parse_property_name();
                let type_parameters = self.parse_optional_type_parameters();
                let parameters = self.parse_parameter_list();
                let type_node = self.parse_optional_return_type();

                let body = if self.token == SyntaxKind::OpenBraceToken {
                    Some(self.parse_block())
                } else {
                    self.parse_semicolon();
                    None
                };

                let end = body
                    .as_ref()
                    .map_or(self.scanner.full_start_pos(), |b| b.end());
                let range = TextRange::new(pos, end);
                return match accessor_kind {
                    SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                        SyntaxKind::GetAccessor,
                        NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                            modifiers: None,
                            name,
                            type_parameters,
                            parameters,
                            type_node,
                            full_signature: None,
                            body,
                        }),
                        range,
                    )),
                    _ => Arc::new(Node::with_loc(
                        SyntaxKind::SetAccessor,
                        NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                            modifiers: None,
                            name,
                            type_parameters,
                            parameters,
                            type_node,
                            full_signature: None,
                            body,
                        }),
                        range,
                    )),
                };
            }
        }

        let is_async = self.token == SyntaxKind::AsyncKeyword;
        if is_async {
            self.next_token();
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);

        let name = self.parse_property_name();
        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
            || is_async
        {

            let type_parameters = self.parse_optional_type_parameters();
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();

            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.expect(SyntaxKind::OpenBraceToken);
                None
            };
            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers: None,
                    asterisk_token,
                    name,
                    postfix_token: None,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            let initializer = self.parse_assignment_expression();
            let end = initializer.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAssignment,
                NodeData::PropertyAssignment(PropertyAssignmentData {
                    modifiers: None,
                    name,
                    postfix_token: None,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    initializer,
                }),
                TextRange::new(pos, end),
            ))
        } else {

            let end = name.end();
            Arc::new(Node::with_loc(
                SyntaxKind::ShorthandPropertyAssignment,
                NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
                    modifiers: None,
                    name,
                    postfix_token: None,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    equals_token: None,
                    object_assignment_initializer: None,
                }),
                TextRange::new(pos, end),
            ))
        }
    }

    #[allow(dead_code)]
    pub(crate) fn is_get_or_set_accessor(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let next = scanner.scan();

        matches!(
            next,
            SyntaxKind::Identifier
                | SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::PrivateIdentifier
        )
    }

    #[allow(dead_code)]
    pub(crate) fn parse_object_accessor(&mut self, pos: usize, is_get: bool) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let body = self.parse_block();
        let end = body.end();
        let kind = if is_get {
            SyntaxKind::GetAccessor
        } else {
            SyntaxKind::SetAccessor
        };
        let data = if is_get {
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: Arc::new(NodeList::default()),
                type_node: None,
                full_signature: None,
                body: Some(body),
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: Arc::new(NodeList::default()),
                type_node: None,
                full_signature: None,
                body: Some(body),
            })
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(pos, end)))
    }

    #[allow(dead_code)]
    pub(crate) fn parse_class_accessor(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        is_get: bool,
    ) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        let kind = if is_get {
            SyntaxKind::GetAccessor
        } else {
            SyntaxKind::SetAccessor
        };
        let data = if is_get {
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(pos, end)))
    }

}