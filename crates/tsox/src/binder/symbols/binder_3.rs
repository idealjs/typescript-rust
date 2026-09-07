#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn declare_symbol_into(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
        target: DeclareTarget,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        let existing: Option<Arc<Symbol>> = match &target {
            DeclareTarget::Exports(parent_sym) => parent_sym.exports.get(&name).cloned(),
            DeclareTarget::Locals(container) => self
                .symbol_map
                .locals
                .get(&container.id())
                .and_then(|locals| locals.get(&name).cloned()),
        };

        if let Some(existing) = existing {
            if self.can_merge_symbols(existing.flags, includes) {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;
                    if (*existing_mut).value_declaration.is_none()
                        && includes.intersects(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                return existing;
            }
        }

        let symbol = self.new_symbol(includes, name.clone());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        match &target {
            DeclareTarget::Exports(parent_sym) => {
                let parent_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_mut)
                        .exports
                        .insert(name.clone(), Arc::clone(&symbol));

                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    (*symbol_mut).parent = Some(Arc::clone(parent_sym));
                }
            }
            DeclareTarget::Locals(container) => {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }

    pub(crate) fn ns_is_instantiated_static(ns: &Arc<Node>) -> bool {
        let NodeData::ModuleDeclaration(md) = &ns.data else {
            return false;
        };
        let Some(body) = &md.body else {
            return false;
        };
        let mut found = false;
        crate::ast::node_data_generated::for_each_child(body, |stmt| {
            match stmt.kind {
                SyntaxKind::InterfaceDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::ImportDeclaration
                | SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::ExportDeclaration => {}
                _ => found = true,
            }
            false
        });
        found
    }
    pub(crate) fn can_merge_symbols(
        &self,
        existing_flags: SymbolFlags,
        new_flags: SymbolFlags,
    ) -> bool {
        let existing_alias = existing_flags.contains(SymbolFlags::Alias);
        let new_alias = new_flags.contains(SymbolFlags::Alias);
        if existing_alias || new_alias {
            return !(existing_alias && new_alias);
        }

        if existing_flags.contains(SymbolFlags::Interface)
            && new_flags.contains(SymbolFlags::Interface)
        {
            return true;
        }

        let existing_interface = existing_flags.contains(SymbolFlags::Interface);
        let new_interface = new_flags.contains(SymbolFlags::Interface);
        let existing_type_alias = existing_flags.contains(SymbolFlags::TypeAlias);
        let new_type_alias = new_flags.contains(SymbolFlags::TypeAlias);
        let class_side = SymbolFlags::Class;

        let enum_side = SymbolFlags::ENUM;
        if (existing_flags.intersects(enum_side) && new_interface)
            || (new_flags.intersects(enum_side) && existing_interface)
        {
            return false;
        }
        if (existing_interface && !new_interface && !new_type_alias)
            || (new_interface && !existing_interface && !existing_type_alias)
            || (existing_type_alias
                && !new_type_alias
                && !new_flags.intersects(class_side)
                && !new_interface)
            || (new_type_alias
                && !existing_type_alias
                && !existing_flags.intersects(class_side)
                && !existing_interface)
        {
            return true;
        }

        let existing_class = existing_flags.contains(SymbolFlags::Class);
        let new_class = new_flags.contains(SymbolFlags::Class);
        let existing_fn = existing_flags.contains(SymbolFlags::Function);
        let new_fn = new_flags.contains(SymbolFlags::Function);
        if (existing_class && new_fn) || (existing_fn && new_class) {
            return true;
        }

        let existing_ns = existing_flags.contains(SymbolFlags::ValueModule);
        let new_ns = new_flags.contains(SymbolFlags::ValueModule);
        if existing_ns || new_ns {
            let other_existing = if existing_ns {
                new_flags
            } else {
                existing_flags
            };
            let _other_new = if existing_ns {
                existing_flags
            } else {
                new_flags
            };

            let can_merge_with_ns = other_existing.contains(SymbolFlags::ValueModule)
                || other_existing.contains(SymbolFlags::Function)
                || other_existing.contains(SymbolFlags::Class)
                || other_existing.contains(SymbolFlags::RegularEnum)
                || other_existing.contains(SymbolFlags::ConstEnum)
                || other_existing.contains(SymbolFlags::Interface);
            if can_merge_with_ns {
                return true;
            }
        }

        if existing_flags.contains(SymbolFlags::Function)
            && new_flags.contains(SymbolFlags::Function)
        {
            return true;
        }

        if (existing_flags.contains(SymbolFlags::RegularEnum)
            || existing_flags.contains(SymbolFlags::ConstEnum))
            && (new_flags.contains(SymbolFlags::RegularEnum)
                || new_flags.contains(SymbolFlags::ConstEnum))
        {
            return true;
        }

        let type_param_existing = existing_flags.contains(SymbolFlags::TypeParameter);
        let type_param_new = new_flags.contains(SymbolFlags::TypeParameter);
        if (type_param_existing && !new_flags.intersects(SymbolFlags::TYPE))
            || (type_param_new && !existing_flags.intersects(SymbolFlags::TYPE))
        {
            return true;
        }
        false
    }

    pub(crate) fn is_let_or_const_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        true
    }

    pub(crate) fn has_export_declarations(container: &Arc<Node>) -> bool {
        let statements: &[Arc<Node>] = match &container.data {
            crate::ast::NodeData::SourceFile(sf) => &sf.statements.nodes,
            crate::ast::NodeData::ModuleDeclaration(md) => {
                if let Some(body) = &md.body
                    && body.kind == SyntaxKind::ModuleBlock
                    && let crate::ast::NodeData::ModuleBlock(block) = &body.data
                {
                    &block.statements.nodes
                } else {
                    &[]
                }
            }
            _ => &[],
        };
        statements.iter().any(|s| {
            s.kind == SyntaxKind::ExportDeclaration || s.kind == SyntaxKind::ExportAssignment
        })
    }

    #[allow(dead_code)]
    pub(crate) fn is_var_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        false
    }

    pub(crate) fn declaration_is_var(node: &Arc<Node>) -> bool {
        let mut current = node;
        loop {
            match current.kind {
                SyntaxKind::VariableDeclaration => {
                    return if let Some(parent) = current.parent.as_ref() {
                        parent.kind == SyntaxKind::VariableDeclarationList
                            && !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const)
                    } else {
                        false
                    };
                }
                SyntaxKind::BindingElement
                | SyntaxKind::ObjectBindingPattern
                | SyntaxKind::ArrayBindingPattern => match current.parent.as_ref() {
                    Some(parent) => current = parent,
                    None => return false,
                },
                _ => return false,
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn symbol_is_var_declaration(symbol: &Arc<Symbol>) -> bool {
        let decl: Option<&Arc<Node>> = symbol
            .value_declaration
            .as_ref()
            .or_else(|| symbol.declarations.first());
        match decl {
            Some(node) => Self::is_var_declaration(node),
            None => false,
        }
    }

    pub(crate) fn get_combined_modifier_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut flags = node.syntactic_modifier_flags();
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = &node.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    flags |= parent.syntactic_modifier_flags();
                    if let Some(gp) = &parent.parent {
                        if gp.kind == SyntaxKind::VariableStatement {
                            flags |= gp.syntactic_modifier_flags();
                        }
                    }
                }
            }
        }
        flags
    }
}
