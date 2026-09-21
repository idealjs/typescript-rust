#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub fn check_class_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);
        self.check_grammar_class_declaration_heritage_clauses(node);
        self.check_exports_on_merged_declarations(node);
        self.check_type_parameters_on_node(node);

        if node.name().is_none() && !node.has_syntactic_modifier(ModifierFlags::Default) {
            self.grammar_error_on_first_token(
                node,
                &tsox_core::diagnostics::messages_generated::
                    A_CLASS_DECLARATION_WITHOUT_THE_DEFAULT_MODIFIER_MUST_HAVE_A_NAME,
            );
        }

        if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                self.check_reserved_type_name(
                    name,
                    &tsox_core::diagnostics::messages_generated::CLASS_NAME_CANNOT_BE_0,
                );

                self.check_cjs_reserved_top_level_name(node, name);
            }
        }

        self.push_scope(node);
        self.check_node_decorators(node);

        let this_type = self.build_class_instance_type_with_base(node);
        self.this_type_stack.push(this_type);

        self.enclosing_class_stack.push(Arc::clone(node));

        self.check_class_type_for_duplicate_declarations(node);

        if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(heritage) = &data.heritage_clauses {
                let mut seen_extends = false;
                let mut seen_implements = false;
                for clause in heritage.iter() {
                    let duplicate_kind = match &clause.data {
                        tsox_frontend::ast::NodeData::HeritageClause(h) => match h.token {
                            SyntaxKind::ExtendsKeyword if seen_extends => true,
                            SyntaxKind::ImplementsKeyword if seen_implements => true,
                            SyntaxKind::ExtendsKeyword => {
                                seen_extends = true;
                                false
                            }
                            SyntaxKind::ImplementsKeyword => {
                                seen_implements = true;
                                false
                            }
                            _ => false,
                        },
                        _ => false,
                    };
                    if !duplicate_kind {
                        self.check_heritage_clause(clause);
                    }
                }
            }

            if !node.has_syntactic_modifier(ModifierFlags::Ambient)
                && self.ambient_context_depth == 0
                && !self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file)
            {
                self.check_class_member_overloads(&data.members);
            }

            for member in data.members.iter() {
                self.check_class_member(member);
            }

            if let Some(this_type) = self.this_type_stack.last().cloned() {
                self.check_index_constraints(&this_type, node);
                // Go checkClassDeclaration：static 索引签名同样过 number⊑string
                // 约束（构造侧类型）
                let static_type = self.get_type_of_class_declaration(node);
                self.check_index_constraints(&static_type, node);
            }
            self.check_class_heritage_members(node);

            self.check_property_accessor_override_kinds(node);

            self.check_members_for_override_modifier(node);

            self.check_property_initialization(node);
        }
        self.pop_scope();
        self.this_type_stack.pop();
        self.enclosing_class_stack.pop();

        let class_type = self.get_type_of_class_declaration(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(class_type.clone());
        if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                if let Some(symbol) = self.resolve_identifier(name) {
                    self.value_symbol_links
                        .get_or_default(&symbol)
                        .resolved_type = Some(class_type);
                }
            }
        }
    }

    pub fn check_enum_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);
        self.check_exports_on_merged_declarations(node);
        self.compute_enum_member_values(node);

        if let tsox_frontend::ast::NodeData::EnumDeclaration(data) = &node.data {
            self.check_cjs_reserved_top_level_name(node, &data.name);
            self.check_reserved_type_name(
                &data.name,
                &tsox_core::diagnostics::messages_generated::ENUM_NAME_CANNOT_BE_0,
            );

            if let Some(sym) = self.program.symbol_map().symbol_of(node) {
                let enum_decls: Vec<&Arc<Node>> = sym
                    .declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::EnumDeclaration)
                    .collect();
                if enum_decls.len() > 1 {
                    let is_first_decl = enum_decls.first().is_some_and(|d| Arc::ptr_eq(d, &node));

                    let first_decl_starts_uninit = enum_decls.first().and_then(|d| {
                            let NodeData::EnumDeclaration(ed) = &d.data else {
                                return None;
                            };
                            ed.members.iter().next().and_then(|m| {
                                matches!(&m.data, tsox_frontend::ast::NodeData::EnumMember(em) if em.initializer.is_none())
                                    .then_some(())
                            })
                        }) == Some(());
                    if !is_first_decl && first_decl_starts_uninit {
                        let first_member = data.members.iter().next();
                        let uninit = first_member.is_some_and(|m| {
                            matches!(
                                &m.data,
                                tsox_frontend::ast::NodeData::EnumMember(em)
                                    if em.initializer.is_none()
                            )
                        });
                        if uninit {
                            let loc = first_member
                                .and_then(|m| m.name())
                                .map(|n| n.loc)
                                .unwrap_or(node.loc);
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    tsox_core::diagnostics::messages_generated::
                                        IN_AN_ENUM_WITH_MULTIPLE_DECLARATIONS_ONLY_ONE_DECLARATION_CAN_OMIT_AN_INITIALIZER_FOR_ITS_FIRST_ENUM_ELEMENT,
                                    Vec::new(),
                                ));
                        }
                    }
                }
            }
        }

        self.push_scope(node);
        if let tsox_frontend::ast::NodeData::EnumDeclaration(data) = &node.data {
            for member in data.members.iter() {
                self.check_enum_member(member);
            }
        }
        self.pop_scope();
    }

    pub fn check_return_statement(&mut self, node: &Arc<Node>) {
        let container = crate::checker::utilities_get_assignment_target::
            get_containing_function_or_class_static_block(node);
        if container
            .as_ref()
            .is_some_and(|c| c.kind == SyntaxKind::ClassStaticBlockDeclaration)
        {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    A_RETURN_STATEMENT_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
                Vec::new(),
            ));
            return;
        }
        if self.function_scope_count == 0 && self.arrow_function_scope_count == 0 {
            if self
                .current_file
                .as_ref()
                .is_some_and(|f| f.has_parse_diagnostics)
            {
                return;
            }
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        A_RETURN_STATEMENT_CAN_ONLY_BE_USED_WITHIN_A_FUNCTION_BODY,
                    Vec::new(),
                ));
        }
        if let tsox_frontend::ast::NodeData::ReturnStatement(data) = &node.data {
            // Go unwrapReturnType：生成器 return 比对注解返回型的 TReturn
            //（async 生成器再 awaited），而非整个注解类型；容器缺失
            //（accessor/构造器等非生成器容器）时透传期望型
            let unwrap_if_generator = |c: &mut Self, expected: Option<Arc<Type>>| -> Option<Arc<Type>> {
                let expected = expected?;
                match c.yield_container_of(node) {
                    Some(container) if container.is_generator => {
                        Some(c.unwrap_generator_return_type(&expected, container.is_async))
                    }
                    _ => Some(expected),
                }
            };
            if let Some(expr) = &data.expression {
                self.check_expression(expr);

                let expected = unwrap_if_generator(
                    self,
                    self.return_type_stack.last().and_then(|opt| opt.clone()),
                );
                if let Some(expected) = expected {
                    // Go checkReturnExpression：返回表达式（剥括号）为条件表达式时
                    // 按分支逐个对返回型比较，错误锚定分支节点，不再整体比较
                    let unwrapped = Checker::skip_parentheses(expr);
                    if matches!(
                        &unwrapped.data,
                        tsox_frontend::ast::NodeData::ConditionalExpression(_)
                    ) {
                        self.check_return_expression_against_type(&expected, node, expr, false, true);
                    } else {
                        // Go checkReturnExpression：async 容器返回表达式先取
                        // awaited 型，thenable 报 TS1058 并按 errorType 参与比对
                        let raw_actual = self.get_type_of_node(expr);
                        let container_async = container
                            .as_ref()
                            .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Async));
                        let actual = if container_async {
                            match self.check_awaited_type_no_alias(
                                &raw_actual,
                                Some(node),
                                tsox_core::diagnostics::messages_generated::
                                    THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER,
                            ) {
                                Some(t) => t,
                                None => self.get_error_type(),
                            }
                        } else {
                            raw_actual
                        };

                        if !actual.flags.contains(TypeFlags::Any)
                            && !self.is_type_assignable_to(&actual, &expected)
                        {
                            let display_type = if crate::checker::is_literal_type(&actual) {
                                self.get_base_type_of_literal_type(&actual)
                            } else {
                                actual.clone()
                            };
                            let ok = self.check_type_related_to_and_optionally_elaborate(
                                &display_type,
                                &expected,
                                crate::checker::relater::RelationKind::Assignable,
                                Some(node),
                                Some(expr),
                                None,
                                None,
                            );
                            if ok {}
                        }
                    }
                }
            } else {
                let expected = unwrap_if_generator(
                    self,
                    self.return_type_stack.last().and_then(|opt| opt.clone()),
                );
                if let Some(expected) = expected {
                    if !expected.flags.contains(TypeFlags::Void)
                        && !expected.flags.contains(TypeFlags::Undefined)
                        && !expected.flags.contains(TypeFlags::Any)
                    {
                        let expected_str = self.type_to_string(&expected);
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec!["undefined".to_string(), expected_str],
                        ));
                    }
                }
            }
        }
    }
}
