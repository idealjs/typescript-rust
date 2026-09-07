#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn parse_throw_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ThrowKeyword);
        let expression = if !self.has_preceding_line_break() {
            self.parse_expression()
        } else {
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(self.token_pos(), self.token_pos()),
            ))
        };

        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(&expression);
        }
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ThrowStatement,
            NodeData::ThrowStatement(ThrowStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_try_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::TryKeyword);
        let try_block = self.parse_block();
        let catch_clause = if self.token == SyntaxKind::CatchKeyword {
            Some(self.parse_catch_clause())
        } else {
            None
        };
        let finally_block = if catch_clause.is_none() || self.token == SyntaxKind::FinallyKeyword {
            self.expect(SyntaxKind::FinallyKeyword);
            Some(self.parse_block())
        } else {
            None
        };
        let end = finally_block.as_ref().map_or_else(
            || catch_clause.as_ref().map_or(try_block.end(), |c| c.end()),
            |f| f.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::TryStatement,
            NodeData::TryStatement(TryStatementData {
                try_block,
                catch_clause,
                finally_block,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_catch_clause(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::CatchKeyword);
        let variable_declaration = if self.parse_optional(SyntaxKind::OpenParenToken) {
            let name = self.parse_identifier_or_pattern();
            let type_node = self.parse_optional_type_annotation();
            self.expect(SyntaxKind::CloseParenToken);
            Some(Arc::new(Node::with_loc(
                SyntaxKind::VariableDeclaration,
                NodeData::VariableDeclaration(VariableDeclarationData {
                    name,
                    exclamation_token: None,
                    type_node,
                    initializer: None,
                }),
                TextRange::new(pos, self.token_pos()),
            )))
        } else {
            None
        };
        let block = self.parse_block();
        let end = block.end();
        Arc::new(Node::with_loc(
            SyntaxKind::CatchClause,
            NodeData::CatchClause(CatchClauseData {
                variable_declaration,
                block,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_debugger_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::DebuggerKeyword);
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::DebuggerStatement,
            NodeData::DebuggerStatement,
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let expression = self.parse_expression();

        if self.token == SyntaxKind::ColonToken && expression.kind == SyntaxKind::Identifier {
            self.next_token();
            let statement = self.parse_statement();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::LabeledStatement,
                NodeData::LabeledStatement(LabeledStatementData {
                    label: expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }

        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(&expression);
        }
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ExpressionStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    pub fn parse_statement(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            SyntaxKind::OpenBraceToken => self.parse_block(),
            SyntaxKind::VarKeyword => self.parse_variable_statement(),
            SyntaxKind::LetKeyword if self.is_let_declaration() => self.parse_variable_statement(),
            SyntaxKind::UsingKeyword if self.is_using_declaration() => {
                self.parse_variable_statement()
            }
            SyntaxKind::AwaitKeyword if self.is_await_using_declaration() => {
                self.parse_variable_statement()
            }
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::DoKeyword => self.parse_do_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::ContinueKeyword => self.parse_continue_statement(),
            SyntaxKind::BreakKeyword => self.parse_break_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::SwitchKeyword => self.parse_switch_statement(),
            SyntaxKind::ThrowKeyword => self.parse_throw_statement(),
            SyntaxKind::TryKeyword => self.parse_try_statement(),
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),

            SyntaxKind::InterfaceKeyword if self.is_start_of_declaration() => {
                self.parse_interface_declaration()
            }
            SyntaxKind::TypeKeyword if self.is_start_of_declaration() => {
                self.parse_type_alias_declaration()
            }
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword
                if self.is_start_of_declaration() =>
            {
                self.parse_namespace_declaration()
            }
            SyntaxKind::DeclareKeyword if self.is_start_of_declaration() => {
                self.parse_declaration_with_modifiers(Vec::new())
            }
            SyntaxKind::AtToken => self.parse_declaration_with_modifiers(Vec::new()),
            SyntaxKind::ImportKeyword if self.is_start_of_declaration() => {
                self.parse_import_declaration()
            }
            SyntaxKind::ExportKeyword => self.parse_export_declaration(),
            SyntaxKind::DebuggerKeyword => self.parse_debugger_statement(),

            SyntaxKind::AsyncKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::AbstractKeyword
            | SyntaxKind::AccessorKeyword
            | SyntaxKind::StaticKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::GlobalKeyword
                if self.is_start_of_declaration() =>
            {
                self.parse_declaration_with_modifiers(Vec::new())
            }
            _ => self.parse_expression_statement(),
        }
    }

    pub(crate) fn parse_empty_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::EmptyStatement,
            NodeData::EmptyStatement,
            TextRange::new(pos, self.token_pos()),
        ))
    }

    pub(crate) fn parse_block(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let multi_line = self.has_preceding_line_break();
        let statements = self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::Block,
            NodeData::Block(BlockData {
                statements: Arc::new(statements),
                multi_line,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_variable_statement(&mut self) -> Arc<Node> {
        self.parse_variable_statement_with_modifiers(None)
    }

    pub(crate) fn parse_variable_statement_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        let declaration_list = self.parse_variable_declaration_list(false);
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_variable_declaration_list(&mut self, _in_for: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let flags = match self.token {
            SyntaxKind::VarKeyword => NodeFlags::empty(),
            SyntaxKind::LetKeyword => NodeFlags::Let,
            SyntaxKind::ConstKeyword => NodeFlags::Const,
            SyntaxKind::UsingKeyword => NodeFlags::Using,
            SyntaxKind::AwaitKeyword => NodeFlags::AwaitUsing,
            _ => NodeFlags::empty(),
        };
        if self.token == SyntaxKind::AwaitKeyword {
            self.next_token();
        }
        if self.token == SyntaxKind::UsingKeyword {
            self.next_token();
        } else {
            self.next_token();
        }
        let declarations = self.parse_delimited_list(
            ParsingContext::VariableDeclarations,
            if _in_for {
                Parser::parse_variable_declaration
            } else {
                Parser::parse_variable_declaration_allow_exclamation
            },
        );
        let end = self.token_pos();
        let mut node = Node::with_loc(
            SyntaxKind::VariableDeclarationList,
            NodeData::VariableDeclarationList(VariableDeclarationListData {
                declarations: Arc::new(declarations),
            }),
            TextRange::new(pos, end),
        );
        node.flags = flags;
        Arc::new(node)
    }

    pub(crate) fn parse_variable_declaration(&mut self) -> Arc<Node> {
        self.parse_variable_declaration_worker(false)
    }

    pub(crate) fn parse_variable_declaration_allow_exclamation(&mut self) -> Arc<Node> {
        self.parse_variable_declaration_worker(true)
    }
}
