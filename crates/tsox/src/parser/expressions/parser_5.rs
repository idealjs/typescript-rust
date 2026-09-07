#![allow(unused_imports)]

use super::*;

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
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }
}
