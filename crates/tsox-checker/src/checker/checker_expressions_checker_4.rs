#![allow(unused_imports)]

use crate::checker::checker_expressions::*;

impl Checker {
    pub(crate) fn check_identifier_reference(&mut self, node: &Arc<Node>) {
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
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
                let mut cur = node.parent();
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
                            cur = a.parent();
                            continue;
                        }
                        SyntaxKind::PropertyDeclaration
                        | SyntaxKind::ClassStaticBlockDeclaration => {
                            in_initializer_or_static_block = true;
                            break;
                        }
                        _ => {}
                    }
                    cur = a.parent();
                }
                if in_initializer_or_static_block {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_ARGUMENTS_CANNOT_BE_REFERENCED_IN_PROPERTY_INITIALIZERS_OR_CLASS_STATIC_INITIALIZATION_BLOCKS,
                        Vec::new(),
                    ));
                    return;
                }
            }

            let is_export_assignment_name = node
                .parent()
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::ExportAssignment);

            // Go resolveEntityName 尾段：值位引用 type-only 别名报 TS1362/1363
            //（getTypeOnlyAliasDeclarationEx 沿别名链回溯，export-star 中转亦可命中）
            if symbol.flags.contains(SymbolFlags::Alias)
                && !symbol.flags.contains(SymbolFlags::VALUE)
                && !is_type_position_use_site(node)
            {
                if let Some((type_only_decl, is_export_form)) =
                    self.type_only_alias_value_declaration(&symbol)
                {
                    let message = if is_export_form {
                        tsox_core::diagnostics::messages_generated::
                            X_0_CANNOT_BE_USED_AS_A_VALUE_BECAUSE_IT_WAS_EXPORTED_USING_EXPORT_TYPE
                    } else {
                        tsox_core::diagnostics::messages_generated::
                            X_0_CANNOT_BE_USED_AS_A_VALUE_BECAUSE_IT_WAS_IMPORTED_USING_IMPORT_TYPE
                    };
                    let mut diag = tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        message,
                        vec![name.to_string()],
                    );
                    let related_message = if is_export_form {
                        tsox_core::diagnostics::messages_generated::X_0_WAS_EXPORTED_HERE
                    } else {
                        tsox_core::diagnostics::messages_generated::X_0_WAS_IMPORTED_HERE
                    };
                    let related_file = self
                        .get_source_file_of_node(&type_only_decl)
                        .or_else(|| self.current_file.clone());
                    diag.related_information
                        .push(tsox_frontend::ast::Diagnostic::new(
                            related_file,
                            type_only_decl.loc,
                            related_message,
                            vec![name.to_string()],
                        ));
                    self.diagnostics.add(diag);
                    return;
                }
            }

            let base = self.resolve_alias_base(Arc::clone(&symbol));

            let is_true_namespace = base.declarations.iter().any(|d| {
                d.kind == SyntaxKind::ModuleDeclaration
                    && d.name()
                        .is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
            });
            // Go 值位语义：模块符号不含 Value 含义（未实例化 namespace）即不可作值；
            // 另保留 ValueModule 但按声明推导不可用（实例化状态与 binder 判定分叉）的兜底
            let module_without_value_meaning = base.flags.contains(SymbolFlags::NamespaceModule)
                && !base.flags.contains(SymbolFlags::ValueModule);
            if !is_export_assignment_name
                && is_true_namespace
                && (module_without_value_meaning
                    || (base.flags.contains(SymbolFlags::ValueModule)
                        && !self.namespace_usable_as_value(&base)))
            {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                    vec![name.to_string()],
                ));
                return;
            }

            // Go checkAndReportErrorForUsingTypeAsValue：值位按 Value 含义解析失败
            // 而全含义解析到类型符号（interface/type alias 等）报 TS2693；
            // bundled lib 的同名 var+interface 合并解析有分叉，先不做此检查
            if !base.flags.intersects(SymbolFlags::VALUE)
                && base.flags.intersects(SymbolFlags::TYPE)
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                && self
                    .resolve_identifier_with_meaning(node, SymbolFlags::VALUE)
                    .is_none()
            {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                    vec![name.to_string()],
                ));
                return;
            }

            self.check_block_scoped_variable_used_before_declaration(node, &symbol, name);

            self.check_variable_used_before_assigned(node, &symbol, name);

            let in_bundled_lib = self
                .get_source_file_of_node(node)
                .is_some_and(|f| crate::bundled::is_bundled(&f.file_name));
            if !in_bundled_lib
                && !is_export_assignment_name
                && !base.flags.intersects(SymbolFlags::VALUE)
            {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                    vec![name.to_string()],
                ));
            }
            return;
        }

        let file = self.current_file.clone();

        {
            let is_primitive_type_name = matches!(
                name,
                "any" | "string" | "number" | "boolean" | "never" | "unknown"
            );
            let reported = if is_primitive_type_name {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file.clone(),
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE,
                    vec![name.to_string()],
                ));
                true
            } else {
                let in_bundled_lib = self
                    .get_source_file_of_node(node)
                    .is_some_and(|f| crate::bundled::is_bundled(&f.file_name));
                let type_hit = self
                    .resolve_identifier_with_meaning(node, SymbolFlags::TYPE)
                    .map(|s| self.resolve_alias_base(s));
                if let Some(sym) = type_hit
                    && !in_bundled_lib
                    && !sym.flags.intersects(SymbolFlags::VALUE)
                {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file.clone(),
                        node.loc,
                        tsox_core::diagnostics::messages_generated::
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
                    tsox_frontend::ast::Diagnostic::new(
                        file,
                        node.loc,
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_STATIC_MEMBER_1_0,
                        vec![name.to_string(), class_name],
                    )
                } else if self.this_container_stack.last()
                    == Some(&ThisContainerKind::InstanceMember)
                {
                    tsox_frontend::ast::Diagnostic::new(
                        file,
                        node.loc,
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_INSTANCE_MEMBER_THIS_0,
                        vec![name.to_string()],
                    )
                } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE)
                {
                    tsox_frontend::ast::Diagnostic::new(
                        file,
                        node.loc,
                        tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE)
                {
                    tsox_frontend::ast::Diagnostic::new(
                        file,
                        node.loc,
                        tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                        vec![name.to_string(), suggestion],
                    )
                } else {
                    tsox_frontend::ast::Diagnostic::new(
                        file,
                        node.loc,
                        CANNOT_FIND_NAME_0,
                        vec![name.to_string()],
                    )
                }
            } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE) {
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                    vec![name.to_string(), suggestion],
                )
            } else {
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    CANNOT_FIND_NAME_0,
                    vec![name.to_string()],
                )
            }
        } else if let Some(msg) = Self::cannot_find_name_message_for(name, Some(node)) {
            tsox_frontend::ast::Diagnostic::new(file, node.loc, *msg, vec![name.to_string()])
        } else if let Some(suggestion) = self.find_name_suggestion(name, SymbolFlags::VALUE) {
            tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1,
                vec![name.to_string(), suggestion],
            )
        } else {
            tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                *Self::cannot_find_name_message_for(name, Some(node)).unwrap_or(&CANNOT_FIND_NAME_0),
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
                    c.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        n.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_SUPER_MUST_BE_CALLED_BEFORE_ACCESSING_THIS_IN_THE_CONSTRUCTOR_OF_A_DERIVED_CLASS,
                        vec![],
                    ));
                }
                return;
            }

            if n.kind == SyntaxKind::CallExpression
                && let tsox_frontend::ast::NodeData::CallExpression(call) = &n.data
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
            tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
                visit(c, child, super_seen);
                false
            });
        }
        let mut super_seen = false;
        visit(self, body, &mut super_seen);
    }
}

fn is_type_position_use_site(node: &Arc<Node>) -> bool {
    let mut cur = node.parent();
    while let Some(p) = cur {
        match p.kind {
            SyntaxKind::TypeReference
            | SyntaxKind::TypeQuery
            | SyntaxKind::ImportType
            | SyntaxKind::QualifiedName => return true,
            _ => break,
        }
    }
    false
}
