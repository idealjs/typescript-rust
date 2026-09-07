#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_identifier_reference(&mut self, node: &Arc<Node>) {
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return,
        };

        if name.is_empty() {
            return;
        }

        if !is_valid_identifier_text(name) {
            return;
        }

        if is_declaration_name(node) {
            return;
        }

        if is_property_access_name(node) {
            return;
        }

        if self.check_invalid_initializer_reference(node, name) {
            return;
        }

        if !self.ts2304_reporting_allowed_for(node) {
            return;
        }

        if let Some(symbol) = self.resolve_identifier(node) {
            if name == "arguments"
                && self.arguments_symbol.is_some()
                && Arc::ptr_eq(&symbol, self.arguments_symbol.as_ref().unwrap())
            {
                let mut cur = node.parent.as_ref();
                let mut in_initializer_or_static_block = false;
                while let Some(a) = cur {
                    match a.kind {
                        SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor => break,
                        SyntaxKind::ArrowFunction => {
                            cur = a.parent.as_ref();
                            continue;
                        }
                        SyntaxKind::PropertyDeclaration
                        | SyntaxKind::ClassStaticBlockDeclaration => {
                            in_initializer_or_static_block = true;
                            break;
                        }
                        _ => {}
                    }
                    cur = a.parent.as_ref();
                }
                if in_initializer_or_static_block {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        crate::diagnostics::messages_generated::
                            X_ARGUMENTS_CANNOT_BE_REFERENCED_IN_PROPERTY_INITIALIZERS_OR_CLASS_STATIC_INITIALIZATION_BLOCKS,
                        Vec::new(),
                    ));
                    return;
                }
            }

            let is_export_assignment_name = node
                .parent
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::ExportAssignment);
            let base = self.resolve_alias_base(Arc::clone(&symbol));

            let is_true_namespace = base.declarations.iter().any(|d| {
                d.kind == SyntaxKind::ModuleDeclaration
                    && d.name()
                        .is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
            });
            if !is_export_assignment_name
                && base.flags.contains(SymbolFlags::ValueModule)
                && is_true_namespace
                && !self.namespace_usable_as_value(&base)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                    vec![name.to_string()],
                ));
                return;
            }

            self.check_block_scoped_variable_used_before_declaration(node, &symbol, name);

            self.check_variable_used_before_assigned(node, &symbol, name);
            return;
        }

        let file = self.current_file.clone();

        {
            let is_primitive_type_name = matches!(
                name,
                "any" | "string" | "number" | "boolean" | "never" | "unknown"
            );
            let reported = if is_primitive_type_name {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file.clone(),
                    node.loc,
                    crate::diagnostics::messages_generated::
                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                    vec![name.to_string()],
                ));
                true
            } else {
                let type_hit = self
                    .resolve_identifier_with_meaning(node, SymbolFlags::TYPE)
                    .map(|s| self.resolve_alias_base(s));
                if let Some(sym) = type_hit
                    && !sym.flags.intersects(SymbolFlags::VALUE)
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file.clone(),
                        node.loc,
                        crate::diagnostics::messages_generated::
                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                        vec![name.to_string()],
                    ));
                    true
                } else {
                    false
                }
            };
            if reported {
                return;
            }
        }

        let diagnostic = if let Some(class) = self.enclosing_class_stack.last().cloned() {
            let class_name = Self::class_name_text(&class);
            if let Some(is_member_static) = self.class_member_static_by_name(&class, name) {
                if is_member_static {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_STATIC_MEMBER_1_0,
                        vec![name.to_string(), class_name],
                    )
                } else if self.this_container_stack.last()
                    == Some(&ThisContainerKind::InstanceMember)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_INSTANCE_MEMBER_THIS_0,
                        vec![name.to_string()],
                    )
                } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE)
                {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else {
                    crate::ast::Diagnostic::new(
                        file,
                        node.loc,
                        CANNOT_FIND_NAME_0,
                        vec![name.to_string()],
                    )
                }
            } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE) {
                crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                    vec![name.to_string(), suggestion],
                )
            } else {
                crate::ast::Diagnostic::new(
                    file,
                    node.loc,
                    CANNOT_FIND_NAME_0,
                    vec![name.to_string()],
                )
            }
        } else if let Some(msg) = Self::cannot_find_name_message_for(name) {
            crate::ast::Diagnostic::new(file, node.loc, *msg, vec![name.to_string()])
        } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE) {
            crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                vec![name.to_string(), suggestion],
            )
        } else {
            crate::ast::Diagnostic::new(
                file,
                node.loc,
                *Self::cannot_find_name_message_for(name).unwrap_or(&CANNOT_FIND_NAME_0),
                vec![name.to_string()],
            )
        };
        self.diagnostics.add(diagnostic);
    }

    pub(crate) fn check_super_before_this(&mut self, body: &Arc<Node>) {
        fn visit(c: &mut Checker, n: &Arc<Node>, super_seen: &mut bool) {
            if n.kind == SyntaxKind::ThisKeyword {
                if !*super_seen {
                    let file = c.current_file.clone();
                    c.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        n.loc,
                        crate::diagnostics::messages_generated::
                            X_SUPER_MUST_BE_CALLED_BEFORE_ACCESSING_THIS_IN_THE_CONSTRUCTOR_OF_A_DERIVED_CLASS,
                        vec![],
                    ));
                }
                return;
            }

            if n.kind == SyntaxKind::CallExpression
                && let crate::ast::NodeData::CallExpression(call) = &n.data
                && call.expression.kind == SyntaxKind::SuperKeyword
            {
                for arg in call.arguments.iter() {
                    visit(c, arg, super_seen);
                }
                *super_seen = true;
                return;
            }

            if matches!(
                n.kind,
                SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return;
            }

            if matches!(
                n.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                return;
            }
            crate::ast::node_data_generated::for_each_child(n, |child| {
                visit(c, child, super_seen);
                false
            });
        }
        let mut super_seen = false;
        visit(self, body, &mut super_seen);
    }
}
