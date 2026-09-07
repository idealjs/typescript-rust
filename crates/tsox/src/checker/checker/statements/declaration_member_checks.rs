#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_class_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);

        if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                self.check_reserved_type_name(
                    name,
                    &crate::diagnostics::messages_generated::CLASS_NAME_CANNOT_BE_0,
                );

                self.check_cjs_reserved_top_level_name(node, name);
            }
        }

        self.push_scope(node);

        let this_type = self.build_class_instance_type_with_base(node);
        self.this_type_stack.push(this_type);

        self.enclosing_class_stack.push(Arc::clone(node));

        if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(heritage) = &data.heritage_clauses {
                for clause in heritage.iter() {
                    self.check_heritage_clause(clause);
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
            }
            self.check_class_heritage_members(node);

            self.check_property_initialization(node);
        }
        self.pop_scope();
        self.this_type_stack.pop();
        self.enclosing_class_stack.pop();

        let class_type = self.get_type_of_class_declaration(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(class_type.clone());
        if let crate::ast::NodeData::ClassDeclaration(data) = &node.data {
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

        if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
            self.check_reserved_type_name(
                &data.name,
                &crate::diagnostics::messages_generated::ENUM_NAME_CANNOT_BE_0,
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
                                matches!(&m.data, crate::ast::NodeData::EnumMember(em) if em.initializer.is_none())
                                    .then_some(())
                            })
                        }) == Some(());
                    if !is_first_decl && first_decl_starts_uninit {
                        let first_member = data.members.iter().next();
                        let uninit = first_member.is_some_and(|m| {
                            matches!(
                                &m.data,
                                crate::ast::NodeData::EnumMember(em)
                                    if em.initializer.is_none()
                            )
                        });
                        if uninit {
                            let loc = first_member
                                .and_then(|m| m.name())
                                .map(|n| n.loc)
                                .unwrap_or(node.loc);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    crate::diagnostics::messages_generated::
                                        IN_AN_ENUM_WITH_MULTIPLE_DECLARATIONS_ONLY_ONE_DECLARATION_CAN_OMIT_AN_INITIALIZER_FOR_ITS_FIRST_ENUM_ELEMENT,
                                    Vec::new(),
                                ));
                        }
                    }
                }
            }
        }

        self.push_scope(node);
        if let crate::ast::NodeData::EnumDeclaration(data) = &node.data {
            for member in data.members.iter() {
                self.check_enum_member(member);
            }
        }
        self.pop_scope();
    }

    pub fn check_return_statement(&mut self, node: &Arc<Node>) {
        if self.function_scope_count == 0 && self.arrow_function_scope_count == 0 {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::
                        A_RETURN_STATEMENT_CAN_ONLY_BE_USED_WITHIN_A_FUNCTION_BODY,
                    Vec::new(),
                ));
        }
        if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
            if let Some(expr) = &data.expression {
                self.check_expression(expr);

                let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                if let Some(expected) = expected {
                    let actual = self.get_type_of_node(expr);

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
            } else {
                let expected = self.return_type_stack.last().and_then(|opt| opt.clone());
                if let Some(expected) = expected {
                    if !expected.flags.contains(TypeFlags::Void)
                        && !expected.flags.contains(TypeFlags::Undefined)
                        && !expected.flags.contains(TypeFlags::Any)
                    {
                        let expected_str = self.type_to_string(&expected);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
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
