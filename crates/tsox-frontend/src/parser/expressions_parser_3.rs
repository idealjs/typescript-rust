#![allow(unused_imports)]

use crate::parser::expressions::*;

impl Parser {
    pub(crate) fn try_parse_generic_arrow_function(&mut self) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_parse_generic_arrow_function"); 
        let starts_with_async = self.token == SyntaxKind::AsyncKeyword;
        if !starts_with_async && self.token != SyntaxKind::LessThanToken {
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
            // Go nextIsParenthesizedArrowFunctionExpression 的 JSX 消歧：
            // `<T extends X`（X 非 =/>//）、`<T,`、`<T=` 视为泛型箭头，其余归 JSX
            if self.language_variant == LanguageVariant::Jsx {
                let t2 = s.scan();
                let is_arrow = if t2 == SyntaxKind::ExtendsKeyword {
                    let t3 = s.scan();
                    !matches!(
                        t3,
                        SyntaxKind::EqualsToken
                            | SyntaxKind::GreaterThanToken
                            | SyntaxKind::SlashToken
                    )
                } else {
                    t2 == SyntaxKind::CommaToken || t2 == SyntaxKind::EqualsToken
                };
                if !is_arrow {
                    return None;
                }
            }
        }

        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();

        let async_modifier = if starts_with_async {
            let token_node = self.create_token_node();
            self.next_token();
            Some(token_node)
        } else {
            None
        };

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
        let saved_yield = self.yield_context;
        let saved_await = self.await_context;
        self.yield_context = false;
        self.await_context = starts_with_async;
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block_ex(true)
        } else {
            self.parse_assignment_expression()
        };
        self.yield_context = saved_yield;
        self.await_context = saved_await;
        let end = self.node_pos();
        let modifiers = async_modifier.and_then(|m| self.make_async_modifier_list(m));
        Some(Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers,
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
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_simple_arrow_function_with_async");
        let modifiers = self.make_async_modifier_list(async_modifier.clone());
        // Go tryParseAsyncSimpleArrowFunctionExpression（parser.go:4845-4848）：
        // pos 取于 parseModifiersForArrowFunction 消费 async 之前，节点范围
        // 含 async 修饰符
        let pos = async_modifier.pos();
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
        let saved_yield = self.yield_context;
        self.await_context = true;
        self.yield_context = false;
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block_ex(true)
        } else {
            self.parse_assignment_expression()
        };
        self.await_context = saved_await;
        self.yield_context = saved_yield;
        let end = self.node_pos();
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

    pub(crate) fn parse_parenthesized_arrow_function(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_parenthesized_arrow_function"); 
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let last_token = self.token;
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let saved_yield = self.yield_context;
        let saved_await = self.await_context;
        self.yield_context = false;
        self.await_context = false;
        // Go parseParenthesizedArrowFunctionExpression：'=>' 缺失且当前非
        // '{' 时 body 取单个标识符（不走赋值表达式，避免 '.' 等被当成员访问）
        let body = if last_token == SyntaxKind::EqualsGreaterThanToken
            || last_token == SyntaxKind::OpenBraceToken
        {
            self.parse_arrow_function_expression_body(false, false)
        } else {
            self.parse_identifier()
        };
        self.yield_context = saved_yield;
        self.await_context = saved_await;
        let end = self.node_pos();
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

    pub(crate) fn parse_simple_arrow_function(&mut self, identifier: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_simple_arrow_function"); 
        let pos = identifier.pos();
        let outer_yield = self.yield_context;
        let outer_await = self.await_context;
        self.yield_context = false;
        self.await_context = false;
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
            self.parse_block_ex(true)
        } else {
            self.parse_assignment_expression()
        };
        self.yield_context = outer_yield;
        self.await_context = outer_await;
        let end = self.node_pos();
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

    pub(crate) fn parse_binary_expression(&mut self, min_precedence: u8) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_binary_expression");
        let mut left = self.parse_unary_expression();
        let mut last_operand = Arc::clone(&left);

        loop {
            if self.token == SyntaxKind::InKeyword && self.disallow_in_context {
                break;
            }
            let precedence = binary_precedence(self.token);
            if precedence == 0 || precedence < min_precedence {
                break;
            }
            // Go parseBinaryExpressionRest（parser.go:4974-4985）：as/satisfies
            // 位处关系优先级档（precedence.go:349-352），在二元循环内消费；
            // 行首 ASI 中断（var x = foo\nas(Bar) 拆两条语句）；消费后下一
            // 运算符若将绑定到断点左侧最后二元运算符同级（**）或更紧则停
            //（GH#63527 断言擦除保序守卫）
            if self.token == SyntaxKind::AsKeyword || self.token == SyntaxKind::SatisfiesKeyword {
                if self.has_preceding_line_break() {
                    break;
                }
                let kind = self.token;
                self.next_token();
                // Go parseBinaryExpressionRest（parser.go:4987/4989）：断言类型一律
                // parseType()，无 as-const 特判——`const` 经 parseNonArrayType
                // default（parser.go:3041-3042）落 parseTypeReference，产出
                // TypeReference(Identifier("const"))，IsConstTypeReference
                //（ast/utilities.go:2773-2776）按该形态判定
                let type_node = self.parse_type();
                let pos = left.pos();
                let end = type_node.end();
                left = if kind == SyntaxKind::AsKeyword {
                    Arc::new(Node::with_loc(
                        SyntaxKind::AsExpression,
                        NodeData::AsExpression(AsExpressionData {
                            expression: Arc::clone(&left),
                            type_node,
                        }),
                        TextRange::new(pos, end),
                    ))
                } else {
                    Arc::new(Node::with_loc(
                        SyntaxKind::SatisfiesExpression,
                        NodeData::SatisfiesExpression(SatisfiesExpressionData {
                            expression: Arc::clone(&left),
                            type_node,
                        }),
                        TextRange::new(pos, end),
                    ))
                };
                let last_precedence = match &last_operand.data {
                    NodeData::BinaryExpression(d) => binary_precedence(d.operator_token.kind),
                    _ => u8::MAX,
                };
                let next_precedence = binary_precedence(self.token);
                if next_precedence > last_precedence
                    || (next_precedence == last_precedence
                        && self.token == SyntaxKind::AsteriskAsteriskToken)
                {
                    break;
                }
                continue;
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
            last_operand = Arc::clone(&left);
        }

        left
    }
}
