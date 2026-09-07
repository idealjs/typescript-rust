#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_variable_declaration(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::VariableDeclaration(data) = &node.data {
            if data.initializer.is_none() {
                let is_const = node
                    .parent
                    .as_ref()
                    .is_some_and(|list| list.flags.contains(NodeFlags::Const));
                let in_for_in_of = node
                    .parent
                    .as_ref()
                    .and_then(|l| l.parent.as_ref())
                    .is_some_and(|g| {
                        matches!(
                            g.kind,
                            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                        )
                    });
                let is_ambient = self.ambient_context_depth > 0
                    || node.flags.contains(NodeFlags::Ambient)
                    || node
                        .parent
                        .as_ref()
                        .and_then(|p| p.parent.as_ref())
                        .is_some_and(|stmt| stmt.has_syntactic_modifier(ModifierFlags::Ambient))
                    || {
                        let mut anc = node.parent.as_ref();
                        let mut found = false;
                        while let Some(a) = anc {
                            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                                found = true;
                                break;
                            }
                            anc = a.parent.as_ref();
                        }
                        found
                    }
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);
                if is_const && !in_for_in_of && !is_ambient {
                    let file = self.current_file.clone();
                    let name_loc = data.name.loc;
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        name_loc,
                        crate::diagnostics::messages_generated::X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                        vec!["const".to_string()],
                    ));
                }
            }

            if data.initializer.is_some() && data.name.kind == SyntaxKind::Identifier {
                let list_is_var = node.parent.as_ref().is_none_or(|l| {
                    !(l.flags.contains(NodeFlags::Let) || l.flags.contains(NodeFlags::Const))
                });
                let is_param = node
                    .parent
                    .as_ref()
                    .is_some_and(|l| l.kind == SyntaxKind::Parameter);
                if list_is_var && !is_param {
                    let own = self.program.symbol_map().symbol_of(node).cloned();
                    if let Some(local) = self.resolve_identifier(&data.name)
                        && own.as_ref().is_none_or(|o| !Arc::ptr_eq(o, &local))
                        && local.flags.contains(SymbolFlags::BlockScopedVariable)
                        && let Some(vd) = local.value_declaration.clone()
                        && vd.kind == SyntaxKind::VariableDeclaration
                        && let Some(list) = vd.parent.as_ref()
                        && list.kind == SyntaxKind::VariableDeclarationList
                    {
                        let container = list.parent.as_ref().and_then(|s| s.parent.as_ref());
                        let names_share_scope = container.is_some_and(|c| {
                            c.kind == SyntaxKind::ModuleBlock
                                || c.kind == SyntaxKind::ModuleDeclaration
                                || c.kind == SyntaxKind::SourceFile
                                || (c.kind == SyntaxKind::Block
                                    && c.parent.as_ref().is_some_and(|p| {
                                        matches!(
                                            p.kind,
                                            SyntaxKind::FunctionDeclaration
                                                | SyntaxKind::FunctionExpression
                                                | SyntaxKind::ArrowFunction
                                                | SyntaxKind::MethodDeclaration
                                                | SyntaxKind::Constructor
                                                | SyntaxKind::GetAccessor
                                                | SyntaxKind::SetAccessor
                                        )
                                    }))
                        });
                        if !names_share_scope {
                            let name_text = data.name.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_INITIALIZE_OUTER_SCOPED_VARIABLE_0_IN_THE_SAME_SCOPE_AS_BLOCK_SCOPED_DECLARATION_1,
                                vec![name_text.clone(), name_text],
                            ));
                        }
                    }
                }
            }

            self.check_binding_pattern_computed_names(&data.name);

            if data.name.kind == SyntaxKind::ObjectBindingPattern
                && self.in_ctor_body_stack.last() == Some(&true)
                && let Some(init) = &data.initializer
                && init.kind == SyntaxKind::ThisKeyword
            {
                let this_type = self.get_type_of_node(init);
                self.check_this_destructuring_abstract_properties(&data.name, &this_type);
            }
            if let Some(init) = &data.initializer {
                self.check_expression(init);
            }

            let resolved_type = match (&data.type_node, &data.initializer) {
                (Some(type_node), Some(init)) => {
                    let annotation_type = self.get_type_from_type_node(type_node);

                    if init.kind == SyntaxKind::ArrayLiteralExpression {
                        let at = Arc::clone(&annotation_type);
                        self.check_contextual_elements(init, &at, init.loc);
                    }
                    let init_type = self.get_type_of_node(init);
                    let assignable = self.is_type_assignable_to(&init_type, &annotation_type);
                    let mut reported_error = false;

                    if let Some(excess_name) =
                        self.get_excess_property_name(&init_type, &annotation_type)
                    {
                        let file = self.current_file.clone();
                        let annot_str = self.type_to_string(&annotation_type);

                        let loc = self
                            .find_object_literal_property_name_node(init, &excess_name)
                            .unwrap_or(node.loc);
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            loc,
                            OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                            vec![excess_name, annot_str],
                        ));
                        reported_error = true;
                    }

                    if !assignable && !reported_error {
                        self.check_type_assignable_to_and_optionally_elaborate(
                            &init_type,
                            &annotation_type,
                            Some(node),
                            Some(init),
                            None,
                            None,
                        );
                    }
                    annotation_type
                }
                (Some(type_node), None) => self.get_type_from_type_node(type_node),
                (None, Some(init)) => {
                    if data.name.kind == SyntaxKind::ArrayBindingPattern {
                        let init_type = if init.kind == SyntaxKind::Identifier
                            && let Some(sym) = self.resolve_identifier(init)
                        {
                            let flow = self.program.symbol_map().flow_node_of(init).map(Arc::clone);
                            self.get_narrowed_type_of_symbol(&sym, flow.as_ref())
                        } else {
                            self.get_type_of_node(init)
                        };
                        if init_type.flags.contains(TypeFlags::Never) {
                            let type_str = self.type_to_string(&init_type);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                data.name.loc,
                                crate::diagnostics::messages_generated::
                                    TYPE_0_MUST_HAVE_A_SYMBOL_ITERATOR_METHOD_THAT_RETURNS_AN_ITERATOR,
                                vec![type_str],
                            ));
                        }
                    }

                    let is_const_decl = self
                        .get_combined_node_flags(node)
                        .intersects(NodeFlags::Constant);
                    if !is_const_decl
                        && matches!(
                            init.kind,
                            SyntaxKind::NullKeyword | SyntaxKind::UndefinedKeyword
                        )
                    {
                        self.auto_type()
                    } else if self.is_empty_array_literal(init) {
                        self.auto_array_type()
                    } else {
                        let init_type = self.get_type_of_node(init);
                        let widened_literal =
                            self.get_widened_literal_type_for_initializer(node, &init_type);
                        let regularized = self.get_regular_type_of_literal_type(&widened_literal);
                        self.widen_initializer_type(&regularized)
                    }
                }
                (None, None) => match self.initial_type_of_declaration(node) {
                    Some(t) => t,
                    None => self.auto_type(),
                },
            };

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                let primary = symbol.value_declaration.clone();
                if let Some(primary) = primary
                    && !Arc::ptr_eq(&primary, node)
                    && symbol.declarations.len() > 1
                    && primary.kind == SyntaxKind::VariableDeclaration
                    && symbol.flags.intersects(
                        SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable,
                    )
                {
                    let auto_to_any = |t: &Arc<Type>| -> Arc<Type> {
                        if t.intrinsic_name() == Some("auto") {
                            self.get_any_type()
                        } else {
                            Arc::clone(t)
                        }
                    };
                    let primary_type = self
                        .type_node_links
                        .get(&primary)
                        .and_then(|l| l.resolved_type.clone())
                        .map(|t| auto_to_any(&t));
                    let this_type = auto_to_any(&resolved_type);
                    if let Some(primary_type) = primary_type
                        && !matches!(primary_type.intrinsic_name(), Some("error"))
                        && !matches!(this_type.intrinsic_name(), Some("error"))
                        && !self
                            .compare_types_identical(&primary_type, &this_type)
                            .is_true()
                    {
                        let name_text = data.name.text().to_string();
                        let first_str = self.type_to_string(&primary_type);
                        let next_str = self.type_to_string(&this_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                SUBSEQUENT_VARIABLE_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_VARIABLE_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                            vec![name_text, first_str, next_str],
                        ));
                    }
                }
            }

            self.type_node_links.get_or_default(node).resolved_type = Some(resolved_type.clone());

            self.type_node_links
                .get_or_default(&data.name)
                .resolved_type = Some(resolved_type.clone());

            if let Some(symbol) = self.resolve_identifier(&data.name) {
                self.value_symbol_links
                    .get_or_default(&symbol)
                    .resolved_type = Some(resolved_type);
            }
        }
    }

    pub(crate) fn check_case_clause(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::CaseOrDefaultClause(data) = &node.data {
            if data.expression.kind != SyntaxKind::UnknownKeyword {
                self.check_expression(&data.expression);
            }
            for stmt in data.statements.iter() {
                self.check_statement(stmt);
            }
        }
    }
}
