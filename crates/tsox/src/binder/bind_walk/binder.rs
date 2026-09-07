#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::Identifier => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
                self.check_contextual_identifier(node);
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            _ => {}
        }

        match node.kind {
            SyntaxKind::VariableDeclaration => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::VariableStatement => {}
            SyntaxKind::FunctionDeclaration => {
                self.declare_symbol(node, SymbolFlags::Function, SymbolFlags::VALUE);
            }
            SyntaxKind::FunctionExpression => {
                let name = match &node.data {
                    NodeData::FunctionExpression(data) => {
                        data.name.as_ref().map(|n| self.node_text(n))
                    }
                    _ => None,
                }
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string());
                self.bind_anonymous_declaration(node, SymbolFlags::Function, &name);
            }
            SyntaxKind::ArrowFunction => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ClassDeclaration => {
                let class_symbol = self.declare_symbol(
                    node,
                    SymbolFlags::Class,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );

                let prototype = Arc::new(Symbol::new(
                    SymbolFlags::Property | SymbolFlags::Prototype,
                    "prototype",
                ));
                let class_mut = Arc::as_ptr(&class_symbol) as *mut Symbol;
                unsafe {
                    (*class_mut)
                        .exports
                        .insert("prototype", Arc::clone(&prototype));
                    let proto_mut = Arc::as_ptr(&prototype) as *mut Symbol;
                    (*proto_mut).parent = Some(Arc::clone(&class_symbol));
                }
            }
            SyntaxKind::ClassExpression => {
                let has_name = matches!(
                    &node.data,
                    NodeData::ClassExpression(data) if data.name.is_some()
                );
                if has_name {
                    self.bind_anonymous_declaration(
                        node,
                        SymbolFlags::Class,
                        INTERNAL_SYMBOL_NAME_CLASS,
                    );
                } else {
                    self.bind_anonymous_declaration(
                        node,
                        SymbolFlags::Class,
                        INTERNAL_SYMBOL_NAME_CLASS,
                    );
                }
            }
            SyntaxKind::InterfaceDeclaration => {
                self.declare_symbol(node, SymbolFlags::Interface, SymbolFlags::TYPE);
            }
            SyntaxKind::TypeAliasDeclaration => {
                self.declare_symbol(node, SymbolFlags::TypeAlias, SymbolFlags::TYPE);
            }
            SyntaxKind::EnumDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::RegularEnum,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ModuleDeclaration => {
                self.bind_module_declaration(node);
            }
            SyntaxKind::Parameter => {
                let report_2371 = |b: &mut Self, loc: crate::core::text::TextRange| {
                    b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        b.current_source_file.clone(),
                        loc,
                        A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
                        vec![],
                    ));
                };
                if let NodeData::ParameterDeclaration(pd) = &node.data
                    && let Some(parent) = node.parent.as_ref()
                    && !fn_like_body_present(parent)
                {
                    if pd.initializer.is_some() {
                        report_2371(self, node.loc);
                    } else {
                        let mut elements: Vec<&Arc<Node>> = Vec::new();
                        collect_binding_elements(&pd.name, &mut elements);
                        for el in elements {
                            if matches!(&el.data, NodeData::BindingElement(be) if be.initializer.is_some())
                            {
                                report_2371(self, el.loc);
                            }
                        }
                    }
                }
                self.declare_symbol(
                    node,
                    SymbolFlags::FunctionScopedVariable,
                    SymbolFlags::VALUE,
                );
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                self.declare_symbol(node, SymbolFlags::Method, SymbolFlags::VALUE);
            }
            SyntaxKind::PropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::EnumMember => {
                self.declare_symbol(
                    node,
                    SymbolFlags::EnumMember,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::GetAccessor => {
                self.declare_symbol(node, SymbolFlags::GetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::SetAccessor => {
                self.declare_symbol(node, SymbolFlags::SetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier => {
                self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
            }

            SyntaxKind::ImportClause => {
                self.bind_import_clause(node);
            }

            SyntaxKind::ExportAssignment => {
                self.bind_export_assignment(node);
            }

            SyntaxKind::ExportDeclaration => {
                self.bind_export_declaration(node);
            }

            SyntaxKind::NamespaceExportDeclaration => {
                self.bind_namespace_export_declaration(node);
            }
            SyntaxKind::BindingElement => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::TypeParameter => {
                if let Some(list) = node.parent.as_ref()
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                {
                    let mut dup = false;
                    crate::ast::node_data_generated::for_each_child(list, |sibling| {
                        if Arc::ptr_eq(sibling, node) {
                            return true;
                        }
                        if sibling.kind == SyntaxKind::TypeParameter
                            && sibling.name().is_some_and(|sn| sn.text() == name.text())
                        {
                            dup = true;
                        }
                        false
                    });
                    if dup {
                        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                            self.current_source_file.clone(),
                            name.loc,
                            DUPLICATE_IDENTIFIER_0,
                            vec![name.text().to_string()],
                        ));
                    }
                }
                self.bind_type_parameter(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ObjectLiteral,
                    INTERNAL_SYMBOL_NAME_OBJECT,
                );
            }
            SyntaxKind::TypeLiteral => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::TypeLiteral,
                    INTERNAL_SYMBOL_NAME_TYPE,
                );
            }
            _ => {}
        }

        if self.bind_statement_kinds(node) {
            return;
        }

        let container_flags = get_container_flags(node.kind);
        if node.kind == SyntaxKind::PropertyDeclaration
            && matches!(&node.data, NodeData::PropertyDeclaration(d) if d.initializer.is_some())
        {
            let prev_flow = self.current_flow.take();
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
            self.bind_children(node);
            self.current_flow = prev_flow;
        } else if container_flags != ContainerFlags::NONE {
            self.bind_container(node, container_flags);
        } else {
            self.bind_children(node);

            if node.kind == SyntaxKind::CallExpression {
                if let Some(current) = self.current_flow.take() {
                    let call_flow = self.create_flow_call(&current, node);
                    self.current_flow = Some(call_flow);
                }
            }
        }
    }
}
