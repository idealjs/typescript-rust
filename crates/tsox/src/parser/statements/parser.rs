#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_if_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::IfKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let then_statement = self.parse_statement();
        let else_statement = if self.parse_optional(SyntaxKind::ElseKeyword) {
            Some(self.parse_statement())
        } else {
            None
        };
        let end = else_statement
            .as_ref()
            .map_or(then_statement.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::IfStatement,
            NodeData::IfStatement(IfStatementData {
                expression,
                then_statement,
                else_statement,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_do_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::DoKeyword);
        let statement = self.parse_statement();
        self.expect(SyntaxKind::WhileKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        self.parse_optional(SyntaxKind::SemicolonToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::DoStatement,
            NodeData::DoStatement(DoStatementData {
                statement,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_while_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::WhileKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();
        let end = statement.end();
        Arc::new(Node::with_loc(
            SyntaxKind::WhileStatement,
            NodeData::WhileStatement(WhileStatementData {
                expression,
                statement,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_for_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ForKeyword);

        let await_modifier = if self.token == SyntaxKind::AwaitKeyword {
            let node = self.create_token_node();
            self.next_token();
            Some(node)
        } else {
            None
        };
        self.expect(SyntaxKind::OpenParenToken);
        let initializer = if self.token != SyntaxKind::SemicolonToken {
            if matches!(
                self.token,
                SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword
            ) {
                Some(self.parse_variable_declaration_list(true))
            } else {
                Some(self.parse_expression())
            }
        } else {
            None
        };

        if self.token == SyntaxKind::InKeyword {
            self.next_token();
            let expression = self.parse_expression();
            self.expect(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            let end = statement.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ForInStatement,
                NodeData::ForInOrOfStatement(ForInOrOfStatementData {
                    await_modifier: None,
                    initializer: initializer.unwrap(),
                    expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }
        if self.token == SyntaxKind::OfKeyword {
            self.next_token();
            let expression = self.parse_expression();
            self.expect(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            let end = statement.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ForOfStatement,
                NodeData::ForInOrOfStatement(ForInOrOfStatementData {
                    await_modifier,
                    initializer: initializer.unwrap(),
                    expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }

        self.expect(SyntaxKind::SemicolonToken);
        let condition = if self.token != SyntaxKind::SemicolonToken
            && self.token != SyntaxKind::CloseParenToken
        {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(SyntaxKind::SemicolonToken);
        let incrementor = if self.token != SyntaxKind::CloseParenToken {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();
        let end = statement.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ForStatement,
            NodeData::ForStatement(ForStatementData {
                initializer,
                condition,
                incrementor,
                statement,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_break_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::BreakKeyword);
        let label = self.parse_identifier_if_not_semicolon();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::BreakStatement,
            NodeData::BreakStatement(BreakStatementData { label }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_continue_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ContinueKeyword);
        let label = self.parse_identifier_if_not_semicolon();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ContinueStatement,
            NodeData::ContinueStatement(ContinueStatementData { label }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_identifier_if_not_semicolon(&mut self) -> Option<Arc<Node>> {
        if !self.can_parse_semicolon() {
            Some(self.parse_identifier())
        } else {
            None
        }
    }

    pub(crate) fn parse_return_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ReturnKeyword);
        let expression = if !self.can_parse_semicolon() {
            Some(self.parse_expression())
        } else {
            None
        };
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ReturnStatement,
            NodeData::ReturnStatement(ReturnStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_switch_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::SwitchKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let case_block = self.parse_case_block();
        let end = case_block.end();
        Arc::new(Node::with_loc(
            SyntaxKind::SwitchStatement,
            NodeData::SwitchStatement(SwitchStatementData {
                expression,
                case_block,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_case_block(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let clauses = self.parse_list(
            ParsingContext::SwitchClauses,
            Parser::parse_case_or_default_clause,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::CaseBlock,
            NodeData::CaseBlock(CaseBlockData {
                clauses: Arc::new(clauses),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_case_or_default_clause(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::CaseKeyword {
            let pos = self.token_pos();
            self.next_token();
            let expression = self.parse_expression();
            self.expect(SyntaxKind::ColonToken);
            let statements = self.parse_list(
                ParsingContext::SwitchClauseStatements,
                Parser::parse_statement,
            );
            let end = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::CaseClause,
                NodeData::CaseOrDefaultClause(CaseOrDefaultClauseData {
                    expression,
                    statements: Arc::new(statements),
                }),
                TextRange::new(pos, end),
            ))
        } else {
            let pos = self.token_pos();
            self.expect(SyntaxKind::DefaultKeyword);
            self.expect(SyntaxKind::ColonToken);
            let statements = self.parse_list(
                ParsingContext::SwitchClauseStatements,
                Parser::parse_statement,
            );
            let end = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::DefaultClause,
                NodeData::CaseOrDefaultClause(CaseOrDefaultClauseData {
                    expression: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(pos, pos),
                    )),
                    statements: Arc::new(statements),
                }),
                TextRange::new(pos, end),
            ))
        }
    }
}
