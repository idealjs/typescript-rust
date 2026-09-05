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
            SyntaxKind::AwaitKeyword => {

                NodeFlags::AwaitUsing
            }
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

    pub(crate) fn parse_variable_declaration_worker(&mut self, allow_exclamation: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            &diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_IN_VARIABLE_DECLARATIONS,
        ));

        let exclamation_token = if allow_exclamation
            && name.kind == SyntaxKind::Identifier
            && self.token == SyntaxKind::ExclamationToken
            && !self.has_preceding_line_break()
        {
            let token = self.create_token_node();
            self.next_token();
            Some(token)
        } else {
            None
        };
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let mut end = name.end();
        if let Some(t) = &exclamation_token {
            end = end.max(t.end());
        }
        if let Some(t) = &type_node {
            end = end.max(t.end());
        }
        if let Some(n) = &initializer {
            end = end.max(n.end());
        }
        Arc::new(Node::with_loc(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(VariableDeclarationData {
                name,
                exclamation_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_identifier_or_pattern(&mut self) -> Arc<Node> {
        self.parse_identifier_or_pattern_with_diagnostic(None)
    }

    pub(crate) fn parse_identifier_or_pattern_with_diagnostic(
        &mut self,
        private_msg: Option<&'static crate::diagnostics::Message>,
    ) -> Arc<Node> {
        if self.token == SyntaxKind::OpenBracketToken {
            self.parse_array_binding_pattern()
        } else if self.token == SyntaxKind::OpenBraceToken {
            self.parse_object_binding_pattern()
        } else {
            self.parse_identifier_with_private_diagnostic(private_msg)
        }
    }

    pub(crate) fn parse_array_binding_pattern(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ArrayBindingElements,
            Parser::parse_array_binding_element,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrayBindingPattern,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_array_binding_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = if self.token != SyntaxKind::CommaToken {
            Some(self.parse_identifier_or_pattern())
        } else {
            None
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.as_ref().map_or(pos, |n| n.end()), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name: None,
                name,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_object_binding_pattern(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ObjectBindingElements,
            Parser::parse_object_binding_element,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ObjectBindingPattern,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_object_binding_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let is_identifier = self.is_identifier();
        let property_name = self.parse_property_name();
        let (property_name, name) = if is_identifier && self.token != SyntaxKind::ColonToken {
            (None, Some(property_name))
        } else {
            self.expect(SyntaxKind::ColonToken);
            (
                Some(property_name),
                Some(self.parse_identifier_or_pattern()),
            )
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.as_ref().map_or(pos, |n| n.end()), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name,
                name,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    pub(crate) fn parse_optional_type_annotation(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type())
        } else {
            None
        }
    }

    pub(crate) fn parse_optional_return_type(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        }
    }

}