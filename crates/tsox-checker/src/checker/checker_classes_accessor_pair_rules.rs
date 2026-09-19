#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    /// 成员键：标识符/字面量按文本，well-known 计算名按内部名
    fn member_key_of(&self, node: &Arc<Node>) -> Option<String> {
        let name = node.name()?;
        match name.kind {
            SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                Some(name.text().to_string())
            }
            SyntaxKind::ComputedPropertyName => {
                let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &name.data else {
                    return None;
                };
                crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)
            }
            _ => None,
        }
    }

    pub(crate) fn check_accessor_pair_rules(
        &mut self,
        node: &Arc<Node>,
        body: &Option<Arc<Node>>,
        parameters: &Option<Arc<NodeList>>,
    ) {
        if matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor) {
            let ambient = self
                .enclosing_class_stack
                .last()
                .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Ambient))
                || self.ambient_context_depth > 0
                || self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file);
            let is_abstract = node.has_syntactic_modifier(ModifierFlags::Abstract);
            if node.kind == SyntaxKind::SetAccessor
                && let Some(params) = &parameters
                && let Some(first) = params.iter().next()
                && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &first.data
            {
                if let Some(rest) = &pd.dot_dot_dot_token {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            rest.loc,
                            tsox_core::diagnostics::messages_generated::
                                A_SET_ACCESSOR_CANNOT_HAVE_REST_PARAMETER,
                            vec![],
                        ));
                }
                if let Some(question) = &pd.question_token {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            question.loc,
                            tsox_core::diagnostics::messages_generated::
                                A_SET_ACCESSOR_CANNOT_HAVE_AN_OPTIONAL_PARAMETER,
                            vec![],
                        ));
                }
                if pd.initializer.is_some() {
                    let name_loc = Self::class_member_name_node(node)
                        .map(|n| n.loc)
                        .unwrap_or(node.loc);
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::
                                A_SET_ACCESSOR_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
                            vec![],
                        ));
                }
            }
            if body.is_none() && !ambient && !is_abstract && node.loc.end() > 0 {
                let file = self.current_file.clone();
                let mut p = node.loc.end();
                if let Some(f) = file.as_ref() {
                    while p > node.loc.pos()
                        && matches!(f.text.as_bytes()[p - 1], b'\r' | b'\n' | b' ' | b'\t')
                    {
                        p -= 1;
                    }
                }
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    tsox_core::core::text::TextRange::new(p - 1, p),
                    tsox_core::diagnostics::messages_generated::X_0_EXPECTED,
                    vec!["{".to_string()],
                ));
            }

            if body.is_some() && is_abstract {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        node.loc,
                        tsox_core::diagnostics::messages_generated::
                            AN_ABSTRACT_ACCESSOR_CANNOT_HAVE_AN_IMPLEMENTATION,
                        vec![],
                    ));
            }

            if node.kind == SyntaxKind::GetAccessor
                && let Some(class) = self.enclosing_class_stack.last().cloned()
                && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &node.data
            {
                let setter = Self::class_members_of(&class).iter().find_map(|m| {
                    if let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &m.data
                        && self.member_key_of(m).is_some_and(|k| {
                            k == self.member_key_of(node).unwrap_or_default()
                        })
                    {
                        Some((Arc::clone(m), sd.name.loc))
                    } else {
                        None
                    }
                });
                if let Some((setter_node, setter_name_loc)) = setter {
                    let getter_abstract = is_abstract;
                    let setter_abstract =
                        setter_node.has_syntactic_modifier(ModifierFlags::Abstract);
                    if getter_abstract != setter_abstract {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file.clone(),
                                gd.name.loc,
                                tsox_core::diagnostics::messages_generated::
                                    ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                vec![],
                            ));
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                setter_name_loc,
                                tsox_core::diagnostics::messages_generated::
                                    ACCESSORS_MUST_BOTH_BE_ABSTRACT_OR_NON_ABSTRACT,
                                vec![],
                            ));
                    }

                    let setter_param_type_node =
                        if let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) =
                            &setter_node.data
                        {
                            sd.parameters.iter().next().and_then(|p| {
                                if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) =
                                    &p.data
                                {
                                    pd.type_node.clone()
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        };
                    if gd.type_node.is_none()
                        && let Some(setter_tn) = setter_param_type_node
                    {
                        self.accessor_pair_return_hint =
                            Some(self.get_type_from_type_node(&setter_tn));
                    }
                }
            }

            if node.kind == SyntaxKind::SetAccessor
                && let Some(class) = self.enclosing_class_stack.last().cloned()
                && let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &node.data
                && sd.name.kind == SyntaxKind::Identifier
                && let Some(param) = sd.parameters.iter().next()
                && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
                && pd.type_node.is_none()
                && let Some(param_name) = (if pd.name.kind == SyntaxKind::Identifier {
                    Some(pd.name.text().to_string())
                } else {
                    None
                })
            {
                let getter_type = Self::class_members_of(&class).iter().find_map(|m| {
                    if let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &m.data
                        && gd.name.kind == SyntaxKind::Identifier
                        && gd.name.text() == sd.name.text()
                        && let Some(tn) = &gd.type_node
                    {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        None
                    }
                });
                if let (Some(expected), Some(body)) = (getter_type, &sd.body) {
                    for (lhs_loc, rhs) in Self::assignments_to_name(body, &param_name) {
                        let actual = self.get_type_of_node(&rhs);
                        if !actual.flags.contains(TypeFlags::Any)
                            && !self.is_type_assignable_to(&actual, &expected)
                        {
                            let display_type = if crate::checker::is_literal_type(&actual) {
                                self.get_base_type_of_literal_type(&actual)
                            } else {
                                actual.clone()
                            };
                            let actual_str = self.type_to_string(&display_type);
                            let expected_str = self.type_to_string(&expected);
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                lhs_loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![actual_str, expected_str],
                            ));
                        }
                    }
                }
            }

            self.check_accessor_signature_rules(node, body, ambient);
        }
    }
}
