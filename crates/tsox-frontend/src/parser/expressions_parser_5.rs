#![allow(unused_imports)]

use crate::parser::expressions::*;

impl Parser {
    pub(crate) fn parse_call_and_member_chain(
        &mut self,
        expr: Arc<Node>,
        member_only: bool,
    ) -> Arc<Node> {
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
                    let name = self.parse_right_side_of_dot();
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
                        let end = self.node_pos();
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
                        let argument = self.parse_element_access_argument();
                        self.expect(SyntaxKind::CloseBracketToken);
                        let end = self.node_pos();
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
                    let end = self.node_pos();
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
                SyntaxKind::OpenBracketToken if !self.decorator_context => {
                    let pos = expr.pos();
                    self.next_token();
                    let argument = self.parse_element_access_argument();
                    self.expect(SyntaxKind::CloseBracketToken);
                    let end = self.node_pos();
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

                    // Go parseMemberExpressionRest tryParseTypeArgumentsInExpression：
                    // 类型实参须能被表达式语境跟随（`(`/模板/行断/二元符/非表达式起始），
                    // 否则回退为关系运算符解释
                    let Some(type_arguments) = self.try_parse_type_arguments_in_expression()
                    else {
                        break;
                    };
                    if self.token == SyntaxKind::OpenParenToken {
                        let arguments = self.parse_argument_list();
                        let end = self.node_pos();
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
                    } else if self.token == SyntaxKind::NoSubstitutionTemplateLiteral
                        || self.token == SyntaxKind::TemplateHead
                    {
                        // Go parseTaggedTemplateRest 吸收 ExpressionWithTypeArguments 的类型实参
                        let template = self.parse_tagged_template_literal();
                        let end = template.end();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::TaggedTemplateExpression,
                            NodeData::TaggedTemplateExpression(TaggedTemplateExpressionData {
                                tag: expr,
                                question_dot_token: None,
                                type_arguments: Some(type_arguments),
                                template,
                            }),
                            TextRange::new(pos, end),
                        ));
                    } else {
                        // 实例化表达式/装饰器类型实参（Go ExpressionWithTypeArguments）
                        let end = self.node_pos();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::ExpressionWithTypeArguments,
                            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData {
                                expression: expr,
                                type_arguments: Some(type_arguments),
                            }),
                            TextRange::new(pos, end),
                        ));
                    }
                }
                SyntaxKind::NoSubstitutionTemplateLiteral | SyntaxKind::TemplateHead => {
                    // Go parseMemberExpressionRest isTemplateStartOfTaggedTemplate
                    let pos = expr.pos();
                    let template = self.parse_tagged_template_literal();
                    let end = template.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::TaggedTemplateExpression,
                        NodeData::TaggedTemplateExpression(TaggedTemplateExpressionData {
                            tag: expr,
                            question_dot_token: None,
                            type_arguments: None,
                            template,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::ExclamationToken if !self.has_preceding_line_break() => {
                    let pos = expr.pos();
                    self.next_token();
                    let end = self.node_pos();
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

    pub(crate) fn parse_tagged_template_literal(&mut self) -> Arc<Node> {
        // Go parseTaggedTemplateRest：无替换模板为字面量 token，带插值的走模板表达式
        if self.token == SyntaxKind::NoSubstitutionTemplateLiteral {
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
        } else {
            self.parse_template_expression_ex(true)
        }
    }

    // Go canFollowTypeArgumentsInExpression：实参表后可被表达式语境跟随的 token 集
    fn can_follow_type_arguments_in_expression(&self) -> bool {
        match self.token {
            SyntaxKind::OpenParenToken
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead => true,
            SyntaxKind::LessThanToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::PlusToken
            | SyntaxKind::MinusToken => false,
            _ => {
                self.has_preceding_line_break()
                    || crate::ast::node_data_generated::is_binary_operator(self.token)
                    || !self.is_start_of_expression()
            }
        }
    }

    // Go tryParseTypeArgumentsInExpression：JS 文件禁用（与二元 `<` 歧义），失败整体回退
    pub(crate) fn try_parse_type_arguments_in_expression(&mut self) -> Option<Arc<NodeList>> {
        if self.javascript_file || self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();
        self.next_token();
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);
        self.re_scan_greater_than();
        if self.token == SyntaxKind::GreaterThanToken {
            self.next_token();
            if self.can_follow_type_arguments_in_expression() {
                let end = self.node_pos();
                return Some(Arc::new(NodeList {
                    loc: TextRange::new(pos, end),
                    nodes: args.nodes,
                }));
            }
        }
        self.scanner = saved_scanner;
        self.token = saved_token;
        self.diagnostics.truncate(diag_len);
        None
    }

    pub(crate) fn parse_element_access_argument(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::CloseBracketToken {
            let pos = self.scanner.full_start_pos();
            self.parse_error_at(
                pos,
                pos,
                tsox_core::diagnostics::AN_ELEMENT_ACCESS_EXPRESSION_SHOULD_TAKE_AN_ARGUMENT,
                &[],
            );
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(pos, pos),
            ))
        } else {
            self.allow_in(|p| p.parse_expression())
        }
    }

    pub(crate) fn parse_argument_list(&mut self) -> Arc<NodeList> {
        self.expect(SyntaxKind::OpenParenToken);
        let nodes =
            self.parse_delimited_list(ParsingContext::ArgumentExpressions, Parser::parse_argument);
        self.expect(SyntaxKind::CloseParenToken);
        Arc::new(nodes)
    }

    pub(crate) fn parse_argument(&mut self) -> Arc<Node> {
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let expression = self.allow_in(|p| p.parse_assignment_expression());
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }
        self.allow_in(|p| p.parse_assignment_expression())
    }

    pub(crate) fn parse_optional_type_arguments(&mut self) -> Option<Arc<NodeList>> {
        if self.token == SyntaxKind::LessThanLessThanToken {
            self.token = self.scanner.re_scan_less_than();
            self.drain_scanner_errors();
        }
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.token_pos();
        self.next_token();
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);

        self.re_scan_greater_than();
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.node_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }

    #[allow(dead_code)]
    pub(crate) fn try_parse_type_arguments(
        &mut self,
        require_following_paren: bool,
    ) -> Option<Arc<NodeList>> {
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
        let end = self.node_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }
}
