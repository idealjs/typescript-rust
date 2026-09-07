#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind_anonymous_declaration(
        &mut self,
        node: &Arc<Node>,
        flags: SymbolFlags,
        name: &str,
    ) {
        let symbol = self.new_symbol(flags, name.to_string());
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
        let (is_export_equals, expr_kind) = match &node.data {
            NodeData::ExportAssignment(data) => (data.is_export_equals, data.expression.kind),
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

        let is_alias = matches!(
            expr_kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName | SyntaxKind::ClassExpression
        );
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

        let is_function_like = flags.contains(ContainerFlags::IS_FUNCTION_LIKE);
        let prev_flow = if is_function_like {
            self.current_flow.take()
        } else {
            None
        };
        if is_function_like {
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
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

        if is_function_like {
            self.current_flow = prev_flow;
        }

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.this_container = prev_this_container;
        self.parent_symbol = prev_parent_symbol;
    }

    pub(crate) fn bind_children(&mut self, node: &Arc<Node>) {
        let this = self as *mut Self;
        crate::ast::node_data_generated::for_each_child(node, |child| {
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
            .parent
            .as_ref()
            .map_or(false, |p| p.kind == SyntaxKind::InferType);
        if parent_is_infer {
            if let Some(container) = node
                .parent
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
            let parent = match &current.parent {
                Some(p) => Arc::clone(p),
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
}
