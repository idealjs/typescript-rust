#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_class_accessor_member(&mut self, node: &Arc<Node>) {
        if node.kind != SyntaxKind::Constructor
            && let Some(name) = Self::member_name_node(node)
        {
            self.check_computed_property_name(&name);
        }

        let (body, type_node, parameters): (
            Option<Arc<Node>>,
            Option<Arc<Node>>,
            Option<Arc<NodeList>>,
        ) = match &node.data {
            crate::ast::NodeData::MethodDeclaration(d) => (
                d.body.clone(),
                d.type_node.clone(),
                Some(Arc::clone(&d.parameters)),
            ),
            crate::ast::NodeData::ConstructorDeclaration(d) => (
                d.body.clone(),
                d.type_node.clone(),
                Some(Arc::clone(&d.parameters)),
            ),
            crate::ast::NodeData::GetAccessorDeclaration(d) => (
                d.body.clone(),
                d.type_node.clone(),
                Some(Arc::clone(&d.parameters)),
            ),
            crate::ast::NodeData::SetAccessorDeclaration(d) => (
                d.body.clone(),
                d.type_node.clone(),
                Some(Arc::clone(&d.parameters)),
            ),
            _ => (None, None, None),
        };

        if body.is_some()
            && (self
                .enclosing_class_stack
                .last()
                .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Ambient))
                || self.ambient_context_depth > 0
                || self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file))
            && let Some(body) = &body
        {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    crate::core::text::TextRange::new(body.loc.pos(), body.loc.pos() + 1),
                    crate::diagnostics::messages_generated::
                        AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                    vec![],
                ));
        }

        if matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor) {
            self.check_accessor_pair_rules(node, &body, &parameters);
        }

        if let Some(params) = &parameters {
            let is_ctor_impl = matches!(node.kind, SyntaxKind::Constructor) && body.is_some();
            self.check_parameter_property_modifiers(params, is_ctor_impl);

            if matches!(
                node.kind,
                SyntaxKind::MethodDeclaration | SyntaxKind::Constructor
            ) {
                self.check_parameter_implicit_any(node, params, 0);
            }
            for p in params.iter() {
                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                    && let Some(pt) = &pd.type_node
                {
                    self.check_type_annotation(pt);

                    if matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor) {
                        let _ = self.get_type_from_type_node(pt);
                    }
                }
            }
        }
        if let Some(tn) = &type_node {
            self.check_type_annotation(tn);
        }

        if self.no_implicit_any
            && matches!(node.kind, SyntaxKind::MethodDeclaration)
            && type_node.is_none()
            && body.is_none()
        {
            if let Some(name) = Self::class_member_name_node(node) {
                if name.kind == SyntaxKind::Identifier {
                    let file = self.current_file.clone();
                    let diagnostic = crate::ast::Diagnostic::new(
                            file,
                            name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            vec![name.text().to_string(), "any".to_string()],
                        );
                    self.diagnostics.add(diagnostic);
                }
            }
        }
        if let Some(body) = body {
            if node.kind == SyntaxKind::Constructor
                && self
                    .enclosing_class_stack
                    .last()
                    .is_some_and(|c| self.extends_base_of(c).is_some())
            {
                self.check_super_before_this(&body);
            }

            let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
            self.this_container_stack.push(if is_static {
                ThisContainerKind::StaticMember
            } else {
                ThisContainerKind::InstanceMember
            });
            self.push_function_scope(node);

            self.in_ctor_body_stack
                .push(node.kind == SyntaxKind::Constructor);

            let declared_return = if node.kind == SyntaxKind::GetAccessor
                && type_node.is_none()
                && let Some(hint) = self.accessor_pair_return_hint.take()
            {
                Some(hint)
            } else {
                let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                type_node
                    .as_ref()
                    .map(|tn| self.get_type_from_type_node(tn))
                    .map(|t| self.unwrap_async_return_type(t, is_async))
            };
            self.return_type_stack.push(declared_return.clone());
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => self.check_expression(&body),
            }
            self.return_type_stack.pop();
            self.in_ctor_body_stack.pop();
            self.pop_function_scope();
            self.this_container_stack.pop();

            if let Some(ret_type) = &declared_return
                && !ret_type.flags.contains(TypeFlags::Void)
                && !ret_type.flags.contains(TypeFlags::Undefined)
                && !ret_type.flags.contains(TypeFlags::Any)
                && body.kind == SyntaxKind::Block
                && !self.function_body_definitely_returns(&body)
            {
                let loc = type_node.as_ref().map_or(node.loc, |tn| tn.loc);
                if matches!(node.kind, SyntaxKind::MethodDeclaration) {
                    if Self::function_body_has_explicit_return(&body) {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                loc,
                                FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                                vec![],
                            ));
                    } else {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                loc,
                                A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                vec![],
                            ));
                    }
                } else if node.kind == SyntaxKind::GetAccessor {
                    let tgt = self.type_to_string(ret_type);
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        loc,
                        TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                        vec!["undefined".to_string(), tgt],
                    ));
                }
            }
        }
    }
}
