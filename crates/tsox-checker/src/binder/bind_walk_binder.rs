#![allow(unused_imports)]

use crate::binder::bind_walk::*;

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
                // Go bindVariableDeclarationOrBindingElement：JS 里
                // `var x = require("...")` 绑为别名（目标是模块符号）
                if self.current_source_file.as_ref().is_some_and(|f| {
                    f.node
                        .flags
                        .contains(tsox_frontend::ast::NodeFlags::JavaScriptFile)
                }) && is_variable_declaration_initialized_to_require(node)
                {
                    self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
                } else {
                    self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
                }
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
                    (*proto_mut).set_parent(&class_symbol);
                }
            }
            SyntaxKind::ClassExpression => {
                // TS 命名类表达式：真实名字（不入容器表），匿名类保持内部名
                let name = match &node.data {
                    NodeData::ClassExpression(data) => data
                        .name
                        .as_ref()
                        .map(|n| self.node_text(n))
                        .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_CLASS.to_string()),
                    _ => INTERNAL_SYMBOL_NAME_CLASS.to_string(),
                };
                self.bind_anonymous_declaration(node, SymbolFlags::Class, &name);
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
                let report_2371 = |b: &mut Self, loc: tsox_core::core::text::TextRange| {
                    b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        b.current_source_file.clone(),
                        loc,
                        A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
                        vec![],
                    ));
                };
                if let NodeData::ParameterDeclaration(pd) = &node.data
                    && let Some(parent) = node.parent().as_ref()
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
            // Go bindJsxAttribute：JSX 属性是 Property 符号（JsxAttributes 为容器）
            SyntaxKind::JsxAttribute => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
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
                if let Some(list) = node.parent().as_ref()
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                {
                    let mut dup = false;
                    tsox_frontend::ast::node_data_generated::for_each_child(list, |sibling| {
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

/// Go IsVariableDeclarationInitializedToRequire：JS 文件、无类型注解、
/// 非 export、初始化式为 require(string-like) 调用
fn is_variable_declaration_initialized_to_require(node: &Arc<Node>) -> bool {
    let NodeData::VariableDeclaration(d) = &node.data else {
        return false;
    };
    if d.type_node.is_some() {
        return false;
    }
    if let Some(gp) = node.parent().and_then(|p| p.parent())
        && gp.kind == SyntaxKind::VariableStatement
        && gp.syntactic_modifier_flags().contains(ModifierFlags::Export)
    {
        return false;
    }
    let Some(init) = &d.initializer else {
        return false;
    };
    let NodeData::CallExpression(call) = &init.data else {
        return false;
    };
    matches!(&call.expression.data, NodeData::Identifier(i) if i.text == "require")
        && call.arguments.nodes.first().is_some_and(|a| {
            matches!(
                a.kind,
                SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
            )
        })
}
