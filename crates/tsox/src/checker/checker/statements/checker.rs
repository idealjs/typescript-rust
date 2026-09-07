#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_statement(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));

        self.type_instantiation_count = 0;

        if self.ambient_context_depth > 0
            && !matches!(
                node.kind,
                SyntaxKind::VariableStatement
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::ModuleDeclaration
                    | SyntaxKind::ImportDeclaration
                    | SyntaxKind::ImportEqualsDeclaration
                    | SyntaxKind::ExportDeclaration
                    | SyntaxKind::ExportAssignment
                    | SyntaxKind::NamespaceExportDeclaration
            )
            && node.parent.as_ref().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::Block | SyntaxKind::ModuleBlock | SyntaxKind::SourceFile
                )
            })
            && !Self::inside_function_body(node)
        {
            let block_id = node.parent.as_ref().unwrap().id();
            if !self.ambient_ts1036_reported_blocks.contains(&block_id) {
                self.ambient_ts1036_reported_blocks.insert(block_id);
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::
                        STATEMENTS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                    Vec::new(),
                ));
            }
        }
        match node.kind {
            SyntaxKind::ExpressionStatement => {
                if let crate::ast::NodeData::ExpressionStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::VariableStatement => {
                if let crate::ast::NodeData::VariableStatement(data) = &node.data {
                    self.check_grammar_variable_declaration_list(&data.declaration_list);
                    self.check_variable_declaration_list(&data.declaration_list);

                    self.check_grammar_modifiers(node);

                    if let crate::ast::NodeData::VariableDeclarationList(list) =
                        &data.declaration_list.data
                    {
                        let decls = list.declarations.clone();
                        for d in decls.iter() {
                            if let crate::ast::NodeData::VariableDeclaration(vd) = &d.data {
                                self.check_cjs_reserved_top_level_name(d, &vd.name);
                            }
                        }
                    }

                    self.check_declaration_nameability(node);
                }
            }
            SyntaxKind::IfStatement => {
                if let crate::ast::NodeData::IfStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.check_statement(&data.then_statement);
                    if let Some(else_stmt) = &data.else_statement {
                        self.check_statement(else_stmt);
                    }
                }
            }
            SyntaxKind::WhileStatement => {
                if let crate::ast::NodeData::WhileStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
            }
            SyntaxKind::DoStatement => {
                if let crate::ast::NodeData::DoStatement(data) = &node.data {
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                    self.check_expression(&data.expression);
                    self.check_truthiness_of_type(&data.expression);
                }
            }
            SyntaxKind::ForStatement => {
                self.push_scope(node);
                if let crate::ast::NodeData::ForStatement(data) = &node.data {
                    if let Some(init) = &data.initializer {
                        self.check_for_initializer(init);
                    }
                    if let Some(cond) = &data.condition {
                        self.check_expression(cond);
                        self.check_truthiness_of_type(cond);
                    }
                    if let Some(incr) = &data.incrementor {
                        self.check_expression(incr);
                    }
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
                self.pop_scope();
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.push_scope(node);
                if let crate::ast::NodeData::ForInOrOfStatement(data) = &node.data {
                    if node.kind == SyntaxKind::ForOfStatement && data.await_modifier.is_none() {
                        self.check_for_of_iterated_type(node, &data.expression);
                    }
                    self.check_for_initializer(&data.initializer);
                    self.check_expression(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Loop,
                            label: None,
                            is_iteration: true,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
                self.pop_scope();
            }
            SyntaxKind::ReturnStatement => {
                self.check_return_statement(node);
            }
            SyntaxKind::Block => {
                self.push_scope(node);
                if let crate::ast::NodeData::Block(data) = &node.data {
                    let mut after_terminator = false;
                    for stmt in data.statements.iter() {
                        let is_hoistable_decl = matches!(
                            stmt.kind,
                            SyntaxKind::EnumDeclaration
                                | SyntaxKind::FunctionDeclaration
                                | SyntaxKind::ClassDeclaration
                        );
                        if after_terminator && !is_hoistable_decl {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                stmt.loc,
                                UNREACHABLE_CODE_DETECTED,
                                vec![],
                            ));
                        }
                        self.check_statement(stmt);
                        if Self::is_block_terminating_statement(stmt) {
                            after_terminator = true;
                        }
                    }
                }
                self.pop_scope();
            }
            SyntaxKind::ThrowStatement => {
                if let crate::ast::NodeData::ThrowStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SwitchStatement => {
                if let crate::ast::NodeData::SwitchStatement(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Switch,
                            label: None,
                            is_iteration: false,
                        });

                    if let crate::ast::NodeData::CaseBlock(case_block) = &data.case_block.data {
                        self.push_scope(&data.case_block);
                        for case in case_block.clauses.iter() {
                            self.check_case_clause(case);
                        }
                        self.pop_scope();
                    }
                    self.break_continue_context_stack.pop();
                }
            }

            SyntaxKind::FunctionDeclaration => {
                self.check_function_declaration(node);
            }
            SyntaxKind::ClassDeclaration => {
                self.check_class_declaration(node);
            }
            SyntaxKind::InterfaceDeclaration => {
                self.check_grammar_modifiers(node);

                if let crate::ast::NodeData::InterfaceDeclaration(data) = &node.data {
                    self.check_reserved_type_name(
                        &data.name,
                        &crate::diagnostics::messages_generated::INTERFACE_NAME_CANNOT_BE_0,
                    );
                    self.check_interface_members(&data.members);
                }

                let iface_sym = self.program.symbol_map().symbol_of(node).cloned();
                if let Some(sym) = iface_sym {
                    let iface_type = self.resolve_interface_type(&sym, None);

                    self.check_index_constraints(&iface_type, node);
                }
            }
            SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier => {
                self.check_type_alias_and_specifiers(node);
                self.check_import_ambient_rules(node);
                self.check_import_equals_conflicts(node);
            }
            SyntaxKind::EnumDeclaration => {
                self.check_enum_declaration(node);
            }
            SyntaxKind::ExportAssignment => {
                if let crate::ast::NodeData::ExportAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ModuleDeclaration => {
                self.check_module_declaration(node);
            }
            SyntaxKind::EmptyStatement => {}
            SyntaxKind::LabeledStatement => {
                if let crate::ast::NodeData::LabeledStatement(data) = &node.data {
                    let label_text = data.label.text().to_string();
                    let is_iteration = matches!(
                        data.statement.kind,
                        SyntaxKind::WhileStatement
                            | SyntaxKind::DoStatement
                            | SyntaxKind::ForStatement
                            | SyntaxKind::ForInStatement
                            | SyntaxKind::ForOfStatement
                    );
                    self.break_continue_context_stack
                        .push(BreakContinueContext {
                            kind: BreakContinueContextKind::Labeled,
                            label: Some(label_text),
                            is_iteration,
                        });
                    self.check_statement(&data.statement);
                    self.break_continue_context_stack.pop();
                }
            }
            SyntaxKind::BreakStatement | SyntaxKind::ContinueStatement => {
                self.check_grammar_break_or_continue_statement(node);
            }
            SyntaxKind::VariableDeclaration => {
                self.check_variable_declaration(node);
            }

            SyntaxKind::ModuleBlock => {
                if let crate::ast::NodeData::ModuleBlock(data) = &node.data {
                    for stmt in data.statements.iter() {
                        self.check_statement(stmt);
                    }
                }
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }
}
