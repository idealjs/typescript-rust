#![allow(unused_imports)]

use crate::parser::impl_chunk::*;

impl Parser {
    pub(crate) fn parse_json_text(&mut self) -> (NodeList, Arc<Node>) {
        let pos = self.node_pos();
        if self.token == SyntaxKind::EndOfFile {
            let end_of_file = self.create_token_node();
            return (
                NodeList {
                    loc: TextRange::new(pos, self.node_pos()),
                    nodes: Vec::new(),
                },
                end_of_file,
            );
        }

        let mut collected: Vec<Arc<Node>> = Vec::new();
        let mut single: Option<Arc<Node>> = None;
        while self.token != SyntaxKind::EndOfFile {
            let expression = match self.token {
                SyntaxKind::OpenBracketToken => self.parse_array_literal(),
                SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::NullKeyword => self.parse_keyword_expression(self.token),
                SyntaxKind::MinusToken => {
                    let (next1, next2) = self.look_ahead_2_tokens_after_current();
                    if next1 == SyntaxKind::NumericLiteral
                        && next2 != SyntaxKind::ColonToken
                    {
                        self.parse_unary_expression()
                    } else {
                        self.parse_object_literal()
                    }
                }
                SyntaxKind::NumericLiteral | SyntaxKind::StringLiteral => {
                    if self.look_ahead_token() != SyntaxKind::ColonToken {
                        self.parse_primary_expression()
                    } else {
                        self.parse_object_literal()
                    }
                }
                _ => self.parse_object_literal(),
            };

            if collected.is_empty() && single.is_none() {
                single = Some(expression);
                if self.token != SyntaxKind::EndOfFile {
                    self.parse_error_at_current_token(diagnostics::UNEXPECTED_TOKEN, &[]);
                }
            } else {
                if let Some(first) = single.take() {
                    collected.push(first);
                }
                collected.push(expression);
            }
        }

        let expression = if collected.is_empty() {
            single.expect("json body expression")
        } else {
            let loc = TextRange::new(pos, self.node_pos());
            Arc::new(Node::with_loc(
                SyntaxKind::ArrayLiteralExpression,
                NodeData::ArrayLiteralExpression(ArrayLiteralExpressionData {
                    elements: Arc::new(NodeList {
                        loc,
                        nodes: collected,
                    }),
                    multi_line: false,
                }),
                loc,
            ))
        };
        let statement = Arc::new(Node::with_loc(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ExpressionStatementData { expression }),
            TextRange::new(pos, self.node_pos()),
        ));
        let statements = NodeList {
            loc: TextRange::new(pos, self.node_pos()),
            nodes: vec![statement],
        };
        let end_of_file = self.create_token_node();
        (statements, end_of_file)
    }

    fn look_ahead_2_tokens_after_current(&self) -> (SyntaxKind, SyntaxKind) {
        let mut scanner = self.scanner.clone();
        let next1 = scanner.scan();
        let next2 = scanner.scan();
        (next1, next2)
    }
}
