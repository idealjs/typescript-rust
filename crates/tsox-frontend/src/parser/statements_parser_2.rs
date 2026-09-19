#![allow(unused_imports)]

use crate::parser::statements::*;

impl Parser {
    pub(crate) fn parse_throw_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ThrowKeyword);
        let expression = if !self.has_preceding_line_break() {
            self.parse_expression()
        } else {
            let pos = self.node_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(pos, pos),
            ))
        };

        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(&expression);
        }
        let end = self.node_pos();
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
            let var_pos = self.token_pos();
            let name = self.parse_identifier_or_pattern();
            let type_node = self.parse_optional_type_annotation();
            let var_end = self.token_pos();
            self.expect(SyntaxKind::CloseParenToken);
            Some(Arc::new(Node::with_loc(
                SyntaxKind::VariableDeclaration,
                NodeData::VariableDeclaration(VariableDeclarationData {
                    name,
                    exclamation_token: None,
                    type_node,
                    initializer: None,
                }),
                TextRange::new(var_pos, var_end),
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
        let end = self.node_pos();
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
            let end = self.node_pos();
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
        let end = self.node_pos();
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
            SyntaxKind::WithKeyword => self.parse_with_statement(),
            SyntaxKind::DoKeyword => self.parse_do_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::ContinueKeyword => self.parse_continue_statement(),
            SyntaxKind::BreakKeyword => self.parse_break_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::SwitchKeyword => self.parse_switch_statement(),
            SyntaxKind::ThrowKeyword => self.parse_throw_statement(),
            SyntaxKind::TryKeyword | SyntaxKind::CatchKeyword | SyntaxKind::FinallyKeyword => {
                self.parse_try_statement()
            }
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

    /// Go parseBlock：`{` 缺失时（shouldAdvance）报错推进并返回空块，
    /// 不解析语句列表（stray catch/finally 的递归防护依赖这一点）
    pub(crate) fn parse_block(&mut self) -> Arc<Node> {
        self.parse_block_ex(false)
    }

    /// Go parseBlock(ignoreMissingOpenBrace=true)：函数体等允许无 `{` 继续解析
    pub(crate) fn parse_block_ex(&mut self, ignore_missing_open_brace: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let open_brace_parsed = self.expect_with_advance(SyntaxKind::OpenBraceToken);
        if !open_brace_parsed && !ignore_missing_open_brace {
            return Arc::new(Node::with_loc(
                SyntaxKind::Block,
                NodeData::Block(BlockData {
                    statements: Arc::new(NodeList::default()),
                    multi_line: false,
                }),
                TextRange::new(pos, pos),
            ));
        }
        let multi_line = self.has_preceding_line_break();
        let statements = self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.node_pos();
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
        let pos = Self::declaration_start(&modifiers, self.token_pos());
        let declaration_list = self.parse_variable_declaration_list(false);
        self.parse_semicolon();
        let end = self.node_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_variable_declaration_list(&mut self, in_for: bool) -> Arc<Node> {
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
        let empty_declarations = self.token == SyntaxKind::OfKeyword
            && self.next_is_identifier_and_close_paren();
        let outer_disallow_in = self.disallow_in_context;
        if in_for {
            self.disallow_in_context = true;
        }
        let declarations = if empty_declarations {
            let of_pos = self.scanner.full_start_pos();
            NodeList {
                loc: TextRange::new(of_pos, of_pos),
                nodes: Vec::new(),
            }
        } else {
            self.parse_delimited_list(
                ParsingContext::VariableDeclarations,
                if in_for {
                    Parser::parse_variable_declaration
                } else {
                    Parser::parse_variable_declaration_allow_exclamation
                },
            )
        };
        self.disallow_in_context = outer_disallow_in;
        let end = self.node_pos();
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
