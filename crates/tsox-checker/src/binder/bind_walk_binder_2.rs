#![allow(unused_imports)]

use crate::binder::bind_walk::*;

impl Binder {
    pub(crate) fn bind_anonymous_declaration(
        &mut self,
        node: &Arc<Node>,
        flags: SymbolFlags,
        name: &str,
    ) {
        let symbol = self.new_symbol(flags, name.to_string());
        let symbol_ptr = Arc::as_ptr(&symbol) as *mut Symbol;
        unsafe {
            (*symbol_ptr).declarations.push(Arc::clone(node));
            (*symbol_ptr).value_declaration = Some(Arc::clone(node));
        }
        self.symbol_map.set_symbol(node, symbol);
    }

    pub(crate) fn bind_import_clause(&mut self, node: &Arc<Node>) {
        let has_name = matches!(&node.data, NodeData::ImportClause(data) if data.name.is_some());
        if !has_name {
            return;
        }
        if let Some(container) = &self.container {
            self.declare_symbol_into(
                node,
                SymbolFlags::Alias,
                SymbolFlags::AliasExcludes,
                DeclareTarget::Locals(Arc::clone(container)),
            );
        }
    }

    pub(crate) fn bind_export_assignment(&mut self, node: &Arc<Node>) {
        let (is_export_equals, expression) = match &node.data {
            NodeData::ExportAssignment(data) => (data.is_export_equals, &data.expression),
            _ => return,
        };
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::VALUE,
                    &self.get_declaration_name(node),
                );
                return;
            }
        };

        // Go ExpressionIsAlias：entity name 表达式（含 Foo.Bar 属性访问链）或类表达式
        let is_alias = Self::expression_is_alias(expression);
        let flags = if is_alias {
            SymbolFlags::Alias
        } else {
            SymbolFlags::Property
        };
        let symbol = self.declare_symbol_into(
            node,
            flags,
            SymbolFlags::all(),
            DeclareTarget::Exports(parent_sym),
        );
        if is_export_equals {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).value_declaration = Some(Arc::clone(node));
            }
        }
    }

    pub(crate) fn bind_export_declaration(&mut self, node: &Arc<Node>) {
        let export_clause: Option<Arc<Node>> = match &node.data {
            NodeData::ExportDeclaration(data) => data.export_clause.clone(),
            _ => return,
        };
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ExportStar,
                    &self.get_declaration_name(node),
                );
                return;
            }
        };
        match &export_clause {
            None => {
                self.declare_symbol_into(
                    node,
                    SymbolFlags::ExportStar,
                    SymbolFlags::None,
                    DeclareTarget::Exports(parent_sym),
                );
            }
            Some(clause) if clause.kind == SyntaxKind::NamespaceExport => {
                let name = self.get_declaration_name(clause);
                let merged_with_members = parent_sym
                    .members
                    .get(&name)
                    .cloned()
                    .filter(|existing| self.can_merge_symbols(existing.flags, SymbolFlags::Alias))
                    .map(|existing| {
                        let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                        unsafe {
                            (*existing_mut).declarations.push(Arc::clone(clause));
                            (*existing_mut).flags |= SymbolFlags::Alias;
                        }
                        existing
                    });
                if let Some(merged) = merged_with_members {
                    let parent_mut = Arc::as_ptr(&parent_sym) as *mut Symbol;
                    unsafe {
                        (*parent_mut).exports.insert(name, merged.clone());
                    }
                    self.symbol_map.set_symbol(clause, merged.clone());
                    return;
                }
                self.declare_symbol_into(
                    clause,
                    SymbolFlags::Alias,
                    SymbolFlags::AliasExcludes,
                    DeclareTarget::Exports(parent_sym),
                );
            }
            Some(clause) if clause.kind == SyntaxKind::NamedExports => {
                // Go bindExportSpecifier：`export { foo }` / `export { foo as bar }`
                // 逐 element 在文件模块 exports 建别名符号。无 from 的目标是
                // 文件 locals 绑定（export_symbol 直连，follow_alias 可达）；
                // 带 from 的建纯 alias，由 checker 按模块说明符解析
                let has_module_specifier = match &node.data {
                    NodeData::ExportDeclaration(d) => d.module_specifier.is_some(),
                    _ => false,
                };
                if let NodeData::NamedExports(ne) = &clause.data {
                    // 目标查找的容器链：文件 + 祖先 declare module（ambient 模块
                    // 内 `export { O as P }` 的 O 在模块 locals）
                    let mut scope_nodes: Vec<Arc<Node>> = Vec::new();
                    {
                        let mut cur = Some(Arc::clone(&node));
                        while let Some(n) = cur {
                            if matches!(
                                n.kind,
                                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
                            ) {
                                scope_nodes.push(Arc::clone(&n));
                                if n.kind == SyntaxKind::SourceFile {
                                    break;
                                }
                            }
                            cur = n.parent();
                        }
                    }
                    let _file_node = scope_nodes
                        .iter()
                        .find(|n| n.kind == SyntaxKind::SourceFile)
                        .cloned()
                        .unwrap_or_else(|| Arc::clone(&node));
                        for el in ne.elements.iter() {
                            let NodeData::ExportSpecifier(spec) = &el.data else { continue };
                            // `export { O as P }`：name=P 是导出名，
                            // property_name=O 是本地原始名
                            let exported = spec
                                .name
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let local_name = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .to_string();
                        if has_module_specifier {
                            // re-export：建纯 alias，checker 按模块说明符解析
                            if let Some(existing) = parent_sym.exports.get(&exported)
                                && existing
                                    .declarations
                                    .iter()
                                    .any(|d| d.kind == SyntaxKind::ExportSpecifier)
                            {
                                // Go declareSymbol(AliasExcludes=Alias)：同名
                                // export specifier 二次声明冲突，报两处 2300
                                for d in existing.declarations.iter().filter(|d| {
                                    d.kind == SyntaxKind::ExportSpecifier && !Arc::ptr_eq(d, el)
                                }) {
                                    if let Some(n) = d.name() {
                                        self.symbol_map.binder_diagnostics.push(
                                            Diagnostic::new(
                                                self.current_source_file.clone(),
                                                n.loc,
                                                DUPLICATE_IDENTIFIER_0,
                                                vec![exported.clone()],
                                            ),
                                        );
                                    }
                                }
                                if let Some(n) = el.name() {
                                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                                        self.current_source_file.clone(),
                                        n.loc,
                                        DUPLICATE_IDENTIFIER_0,
                                        vec![exported.clone()],
                                    ));
                                }
                            }
                            if parent_sym.exports.get(&exported).is_none() {
                                let sym = self.new_symbol(SymbolFlags::Alias, exported.clone());
                                let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                                unsafe {
                                    (*sym_mut).declarations.push(Arc::clone(el));
                                }
                                let parent_mut = Arc::as_ptr(&parent_sym) as *mut Symbol;
                                unsafe {
                                    (*parent_mut)
                                        .exports
                                        .insert(exported.clone(), Arc::clone(&sym));
                                }
                                self.symbol_map.set_symbol(el, sym);
                            }
                            continue;
                        }
                        // 无 from：目标是本容器链绑定——直接把绑定符号放入
                        // exports（不建新符号，避免 Duplicate identifier）。
                        // Go declareSymbol(AliasExcludes)：既有 exports 条目含
                        // export specifier 声明时，同名 specifier 二次声明冲突
                        if let Some(existing) = parent_sym.exports.get(&exported)
                            && existing
                                .declarations
                                .iter()
                                .any(|d| d.kind == SyntaxKind::ExportSpecifier)
                        {
                            let prior: Vec<Arc<Node>> = existing
                                .declarations
                                .iter()
                                .filter(|d| {
                                    d.kind == SyntaxKind::ExportSpecifier && !Arc::ptr_eq(d, el)
                                })
                                .cloned()
                                .collect();
                            if !prior.is_empty() {
                                for d in &prior {
                                    if let Some(n) = d.name() {
                                        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                                            self.current_source_file.clone(),
                                            n.loc,
                                            DUPLICATE_IDENTIFIER_0,
                                            vec![exported.clone()],
                                        ));
                                    }
                                }
                                if let Some(n) = el.name() {
                                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                                        self.current_source_file.clone(),
                                        n.loc,
                                        DUPLICATE_IDENTIFIER_0,
                                        vec![exported.clone()],
                                    ));
                                }
                            }
                        }
                        let target = scope_nodes.iter().find_map(|scope| {
                            self.symbol_map
                                .locals
                                .get(&scope.id())
                                .and_then(|l| l.get(&local_name).cloned())
                                .or_else(|| {
                                    self.symbol_map
                                        .symbol_of(scope)
                                        .and_then(|sf| sf.members.get(&local_name).cloned())
                                })
                        });
                        let Some(target) = target else { continue };
                        let target_mut = Arc::as_ptr(&target) as *mut Symbol;
                        unsafe {
                            (*target_mut).declarations.push(Arc::clone(el));
                        }
                        if parent_sym.exports.get(&exported).is_none() {
                            let parent_mut = Arc::as_ptr(&parent_sym) as *mut Symbol;
                            unsafe {
                                (*parent_mut).exports.insert(exported, Arc::clone(&target));
                            }
                        }
                        self.symbol_map.set_symbol(el, Arc::clone(&target));
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn bind_namespace_export_declaration(&mut self, node: &Arc<Node>) {
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => return,
        };
        self.declare_symbol_into(
            node,
            SymbolFlags::Alias,
            SymbolFlags::AliasExcludes,
            DeclareTarget::Exports(parent_sym),
        );
    }

    pub(crate) fn bind_container(&mut self, node: &Arc<Node>, flags: ContainerFlags) {
        let prev_container = self.container.clone();
        let prev_block = self.block_scope_container.take();

        let prev_this_container = self.this_container.take();

        let prev_parent_symbol = self.parent_symbol.take();

        let block_only = is_block_only_container(node.kind);
        if flags.contains(ContainerFlags::IS_CONTAINER) && !block_only {
            self.container = Some(Arc::clone(node));
            self.block_scope_container = Some(Arc::clone(node));
        } else {
            self.block_scope_container = Some(Arc::clone(node));
        }

        if flags.contains(ContainerFlags::IS_THIS_CONTAINER) {
            self.this_container = Some(Arc::clone(node));
        }

        if has_locals(node.kind) {
            self.symbol_map.locals.insert(node.id(), SymbolTable::new());

            if node.kind == SyntaxKind::ClassExpression
                && let NodeData::ClassExpression(data) = &node.data
                && let Some(name_node) = data.name.as_ref()
            {
                let name = name_node.text().to_string();
                let sym = self.new_symbol(SymbolFlags::Class, name.clone());
                let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                unsafe {
                    (*sym_mut).declarations.push(Arc::clone(node));
                    (*sym_mut).value_declaration = Some(Arc::clone(node));
                }
                self.symbol_map
                    .locals
                    .entry(node.id())
                    .or_insert_with(SymbolTable::new)
                    .insert(name, Arc::clone(&sym));
            }
        }

        if let Some(sym) = self.symbol_map.symbol_of(node) {
            self.parent_symbol = Some(Arc::clone(sym));
        }

        let is_static_block = node.kind == SyntaxKind::ClassStaticBlockDeclaration;
        let is_function_like =
            flags.contains(ContainerFlags::IS_FUNCTION_LIKE) && !is_static_block;
        let prev_flow = if is_function_like {
            self.current_flow.take()
        } else {
            None
        };
        if is_function_like {
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
        }
        // Go bindWorker 控制流容器分支：进入时清空跳转目标/活跃标签/返回目标，
        // 静态块按 IIFE 处理（不重置 currentFlow，挂 return 分支标签）
        let static_block_return = if is_static_block {
            Some(Self::new_flow_accumulator())
        } else {
            None
        };
        let save_jump_reset = is_control_flow_jump_reset_container(node.kind);
        let save_break = if save_jump_reset {
            self.current_break_target.take()
        } else {
            None
        };
        let save_continue = if save_jump_reset {
            self.current_continue_target.take()
        } else {
            None
        };
        let save_labels = if save_jump_reset {
            self.active_label_list.take()
        } else {
            None
        };
        let save_return = if save_jump_reset {
            self.current_return_target.take()
        } else {
            None
        };
        let save_exception = if save_jump_reset {
            self.current_exception_target.take()
        } else {
            None
        };
        if let Some(rl) = &static_block_return {
            self.current_return_target = Some(Arc::clone(rl));
        }

        if node.kind == SyntaxKind::FunctionExpression {
            let sym_and_name = self
                .symbol_map
                .symbol_of(node)
                .map(|sym| (Arc::clone(&sym), sym.name.clone()));
            if let Some((sym, sym_name)) = sym_and_name {
                if sym_name != INTERNAL_SYMBOL_NAME_FUNCTION {
                    if let Some(locals) = self.symbol_map.locals.get_mut(&node.id()) {
                        locals.insert(sym_name, sym);
                    }
                }
            }
        }

        self.bind_children(node);

        if let Some(rl) = &static_block_return {
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(rl, current);
            }
            let finished = Self::finish_flow_node(rl, &self.unreachable_flow());
            self.symbol_map.set_flow_node(node, Arc::clone(&finished));
            self.current_flow = Some(finished);
        }
        if is_function_like {
            self.current_flow = prev_flow;
        }
        if save_jump_reset {
            self.current_break_target = save_break;
            self.current_continue_target = save_continue;
            self.active_label_list = save_labels;
            self.current_return_target = save_return;
            self.current_exception_target = save_exception;
        }

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.this_container = prev_this_container;
        self.parent_symbol = prev_parent_symbol;
    }

    pub(crate) fn bind_children(&mut self, node: &Arc<Node>) {
        let this = self as *mut Self;
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            unsafe {
                (*this).bind(child);
            }
            false
        });
    }

    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }

    pub(crate) fn bind_type_parameter(&mut self, node: &Arc<Node>) {
        let parent_is_infer = node
            .parent()
            .as_ref()
            .map_or(false, |p| p.kind == SyntaxKind::InferType);
        if parent_is_infer {
            if let Some(container) = node
                .parent()
                .as_ref()
                .and_then(|infer| self.get_infer_type_container(infer))
            {
                self.declare_local_symbol(
                    &container,
                    node,
                    SymbolFlags::TypeParameter,
                    SymbolFlags::TYPE,
                );
                return;
            }

            let name = self.get_declaration_name(node);
            self.bind_anonymous_declaration(node, SymbolFlags::TypeParameter, &name);
            return;
        }
        self.declare_symbol(node, SymbolFlags::TypeParameter, SymbolFlags::TYPE);
    }

    pub(crate) fn get_infer_type_container(&self, infer_node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(infer_node);
        loop {
            let parent = match current.parent() {
                Some(p) => Arc::clone(&p),
                None => return None,
            };
            if parent.kind == SyntaxKind::ConditionalType {
                let is_extends = match &parent.data {
                    NodeData::ConditionalTypeNode(data) => {
                        Arc::ptr_eq(&data.extends_type, &current)
                    }
                    _ => false,
                };
                if is_extends {
                    return Some(parent);
                }
                return None;
            }
            current = parent;
        }
    }

    fn expression_is_alias(expression: &Arc<Node>) -> bool {
        let mut cur = expression;
        loop {
            match cur.kind {
                SyntaxKind::Identifier => return true,
                SyntaxKind::QualifiedName => match &cur.data {
                    NodeData::QualifiedName(d) => cur = &d.left,
                    _ => return false,
                },
                SyntaxKind::PropertyAccessExpression => match &cur.data {
                    NodeData::PropertyAccessExpression(d) => cur = &d.expression,
                    _ => return false,
                },
                SyntaxKind::ParenthesizedExpression => match &cur.data {
                    NodeData::ParenthesizedExpression(d) => cur = &d.expression,
                    _ => return false,
                },
                SyntaxKind::ClassExpression => return true,
                _ => return false,
            }
        }
    }
}
