#![allow(unused_imports)]

use crate::binder::bind_walk::*;

impl Binder {
    pub(crate) fn bind(&mut self, node: &Arc<Node>) {
        // Go bindChildren 前奏：flow 已不可达时节点打 Unreachable 旗标
        //（子嗣仍走 bind 声明符号；kind 专属 flow 管理在其后，见各 arm）
        if self.current_flow.as_ref().zip(self.unreachable_flow.as_ref()).is_some_and(
            |(cur, unreach)| Arc::ptr_eq(cur, unreach),
        ) && tsox_frontend::ast::is_potentially_executable_node(node)
        {
            let ptr = Arc::as_ptr(node) as *mut tsox_frontend::ast::Node;
            unsafe {
                (*ptr).flags |= tsox_frontend::ast::NodeFlags::Unreachable;
            }
        }
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
                // Go bindWorker KindCatchClause：catch 变量名 eval/arguments
                // 报 TS1100（tsgo 无条件）
                if node
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::CatchClause)
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                    && matches!(name.text(), "eval" | "arguments")
                {
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            INVALID_USE_OF_0_IN_STRICT_MODE,
                        vec![name.text().to_string()],
                    ));
                }
                // Go bindVariableDeclaration：非 ambient 变量名 eval/arguments
                // 报 TS1100/1210（按类/模块语境选消息）
                if !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
                    && !self
                        .current_source_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file)
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                    && matches!(name.text(), "eval" | "arguments")
                {
                    let msg = strict_mode_eval_or_arguments_message(
                        node,
                        self.current_source_file.as_ref(),
                    );
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        name.loc,
                        msg,
                        vec![name.text().to_string()],
                    ));
                }
                // Go bindVariableDeclarationOrBindingElement：JS 里
                // `var x = require("...")` 绑为别名（目标是模块符号）
                if self.current_source_file.as_ref().is_some_and(|f| {
                    f.node
                        .flags
                        .contains(tsox_frontend::ast::NodeFlags::JavaScriptFile)
                }) && is_variable_declaration_initialized_to_require(node)
                {
                    self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
                } else if Self::is_let_or_const_declaration(node) {
                    self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
                } else {
                    self.declare_symbol(
                        node,
                        SymbolFlags::FunctionScopedVariable,
                        SymbolFlags::VALUE,
                    );
                }
            }
            SyntaxKind::VariableStatement => {}
            SyntaxKind::FunctionDeclaration => {
                // Go checkStrictModeFunctionName：非 ambient 函数名
                // eval/arguments 报 TS1100（tsgo 无条件）
                if !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
                    && !self
                        .current_source_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file)
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                    && matches!(name.text(), "eval" | "arguments")
                {
                    let msg = strict_mode_eval_or_arguments_message(
                        node,
                        self.current_source_file.as_ref(),
                    );
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        name.loc,
                        msg,
                        vec![name.text().to_string()],
                    ));
                }
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
                if !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
                    && !self
                        .current_source_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file)
                    && let Some(name_node) = node.name()
                    && name_node.kind == SyntaxKind::Identifier
                    && matches!(name_node.text(), "eval" | "arguments")
                {
                    let msg = strict_mode_eval_or_arguments_message(
                        node,
                        self.current_source_file.as_ref(),
                    );
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        name_node.loc,
                        msg,
                        vec![name_node.text().to_string()],
                    ));
                }
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
                // Go bindParameter：非 ambient 参数名 eval/arguments 报
                // TS1100（tsgo 无条件）
                if !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
                    && !self
                        .current_source_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file)
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                    && matches!(name.text(), "eval" | "arguments")
                {
                    let msg = strict_mode_eval_or_arguments_message(
                        node,
                        self.current_source_file.as_ref(),
                    );
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        name.loc,
                        msg,
                        vec![name.text().to_string()],
                    ));
                }
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
                // Go bindParameter：构造器参数属性（ParameterPropertyModifier）
                // 同时向所属类 members 表声明 Property 成员（实例表，
                // Property|Optional，PropertyExcludes）
                if let Some(parent) = node.parent().as_ref()
                    && parent.kind == SyntaxKind::Constructor
                    && node
                        .syntactic_modifier_flags()
                        .intersects(tsox_frontend::ast::ModifierFlags::ParameterPropertyModifier)
                    && node.name().is_some_and(|n| {
                        !matches!(
                            n.kind,
                            SyntaxKind::ObjectBindingPattern
                                | SyntaxKind::ArrayBindingPattern
                        )
                    })
                    && let Some(class_declaration) = parent.parent().as_ref()
                    && let Some(class_symbol) = self.symbol_map.symbol_of(class_declaration).cloned()
                {
                    let question = matches!(
                        &node.data,
                        NodeData::ParameterDeclaration(pd) if pd.question_token.is_some()
                    );
                    let mut includes = SymbolFlags::Property;
                    if question {
                        includes |= SymbolFlags::Optional;
                    }
                    self.declare_symbol_into(
                        node,
                        includes,
                        SymbolFlags::PropertyExcludes,
                        crate::binder::binder::DeclareTarget::Members(class_symbol),
                    );
                }
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
            | SyntaxKind::ImportSpecifier => {
                self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
            }
            // ExportSpecifier 由 bind_export_declaration 的 NamedExports
            // 分支统一绑定（否则双重 declare_symbol 报 Duplicate identifier）

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
                let includes = if Self::is_let_or_const_declaration(node) {
                    SymbolFlags::BlockScopedVariable
                } else {
                    SymbolFlags::FunctionScopedVariable
                };
                self.declare_symbol(node, includes, SymbolFlags::VALUE);
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
            SyntaxKind::TypeLiteral | SyntaxKind::MappedType => {
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

        let mut container_flags = get_container_flags(node.kind);
        // Go getContainerFlags 的 Block 特判：函数体/类静态块体不是块作用域容器，
        // 其声明与参数同域（同名 let 与参数冲突）
        if node.kind == SyntaxKind::Block
            && node.parent().is_some_and(|p| {
                matches!(
                    p.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::ClassStaticBlockDeclaration
                )
            })
        {
            container_flags = ContainerFlags::NONE;
        }
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

fn strict_mode_eval_or_arguments_message(
    node: &Arc<Node>,
    file: Option<&Arc<tsox_frontend::ast::SourceFile>>,
) -> tsox_core::diagnostics::Message {
    if tsox_frontend::ast::utilities::get_containing_class(node).is_some() {
        tsox_core::diagnostics::messages_generated::CODE_CONTAINED_IN_A_CLASS_IS_EVALUATED_IN_JAVASCRIPT_S_STRICT_MODE_WHICH_DOES_NOT_ALLOW_THIS_USE_OF_0_FOR_MORE_INFORMATION_SEE_HTTPS_COLON_SLASH_SLASHDEVELOPER_MOZILLA_ORG_SLASHEN_US_SLASHDOCS_SLASHWEB_SLASHJAVASCRIPT_SLASHREFERENCE_SLASHSTRICT_MODE
    } else if file.is_some_and(|f| f.external_module_indicator.is_some()) {
        tsox_core::diagnostics::messages_generated::
            INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
    } else {
        tsox_core::diagnostics::messages_generated::INVALID_USE_OF_0_IN_STRICT_MODE
    }
}
