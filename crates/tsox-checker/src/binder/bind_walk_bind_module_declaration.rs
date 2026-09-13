#![allow(unused_imports)]

use crate::binder::bind_walk::*;

impl Binder {
    pub(crate) fn bind_module_declaration(&mut self, node: &Arc<Node>) {
        self.set_export_context_flag(node);
        self.bind_module_declaration_inner(node)
    }

    /// Go setExportContextFlag：ambient 模块且无 export 声明时是隐式导出语境
    /// （declare namespace 内未加 export 的声明自动入 exports）
    fn set_export_context_flag(&mut self, node: &Arc<Node>) {
        let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
            || node.flags.contains(NodeFlags::Ambient);
        if is_ambient && !Self::has_export_declarations(node) {
            let ptr = Arc::as_ptr(node) as *mut tsox_frontend::ast::Node;
            unsafe {
                (*ptr).flags |= NodeFlags::ExportContext;
            }
        }
    }

    fn bind_module_declaration_inner(&mut self, node: &Arc<Node>) {
        let dotted_name = match &node.data {
            tsox_frontend::ast::NodeData::ModuleDeclaration(md) => match md.name.kind {
                SyntaxKind::Identifier => md.name.text().to_string(),
                SyntaxKind::QualifiedName => {
                    fn qualified_text(n: &Arc<Node>) -> String {
                        match &n.data {
                            tsox_frontend::ast::NodeData::QualifiedName(q) => {
                                format!("{}.{}", qualified_text(&q.left), q.right.text())
                            }
                            _ => n.text().to_string(),
                        }
                    }
                    qualified_text(&md.name)
                }
                _ => String::new(),
            },
            _ => String::new(),
        };
        if dotted_name.contains('.') {
            let parts: Vec<&str> = dotted_name.split('.').collect();

            let container = self.container.clone();
            let parent_sym = self.parent_symbol.clone();
            let mut table: Option<Arc<Symbol>> = None;
            let mut locals_key: Option<u64> = None;
            if let Some(ps) = &parent_sym {
                table = Some(Arc::clone(ps));
            } else if let Some(c) = &container {
                locals_key = Some(c.id());
            }
            let mut current: Option<Arc<Symbol>> = None;
            for part in &parts[..parts.len() - 1] {
                let existing = current.as_ref().map_or_else(
                    || {
                        table
                            .as_ref()
                            .and_then(|t| {
                                t.members
                                    .get(*part)
                                    .cloned()
                                    .or_else(|| t.exports.get(*part).cloned())
                            })
                            .or_else(|| {
                                locals_key
                                    .and_then(|k| self.symbol_map.locals.get(&k))
                                    .and_then(|l| l.get(*part).cloned())
                            })
                    },
                    |cur| cur.exports.get(*part).cloned(),
                );
                let sym = match existing {
                    Some(s) if s.flags.contains(SymbolFlags::ValueModule) => s,
                    _ => {
                        let fresh =
                            Arc::new(Symbol::new(SymbolFlags::ValueModule, part.to_string()));
                        if let Some(cur) = &current {
                            let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                            unsafe {
                                (*cur_mut)
                                    .exports
                                    .insert(part.to_string(), Arc::clone(&fresh));
                            }
                        } else if let Some(t) = &table {
                            let t_mut = Arc::as_ptr(t) as *mut Symbol;
                            unsafe {
                                (*t_mut)
                                    .members
                                    .insert(part.to_string(), Arc::clone(&fresh));
                            }
                        } else if let Some(k) = locals_key {
                            self.symbol_map
                                .locals
                                .entry(k)
                                .or_default()
                                .insert(part.to_string(), Arc::clone(&fresh));
                        }
                        fresh
                    }
                };
                current = Some(sym);
            }

            let last = parts[parts.len() - 1];
            let (includes, excludes) = Self::module_symbol_flags(node);
            let symbol = Arc::new(Symbol::new(includes, last.to_string()));
            {
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*symbol_mut).declarations.push(Arc::clone(node));
                }
            }
            match &current {
                Some(cur) => {
                    let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                    unsafe {
                        (*cur_mut)
                            .exports
                            .insert(last.to_string(), Arc::clone(&symbol));
                    }
                }
                None => {
                    if let Some(t) = &table {
                        let t_mut = Arc::as_ptr(t) as *mut Symbol;
                        unsafe {
                            (*t_mut)
                                .members
                                .insert(last.to_string(), Arc::clone(&symbol));
                        }
                    } else if let Some(k) = locals_key {
                        self.symbol_map
                            .locals
                            .entry(k)
                            .or_default()
                            .insert(last.to_string(), Arc::clone(&symbol));
                    }
                }
            }
            self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        } else {
            let name_is_string_literal = match &node.data {
                tsox_frontend::ast::NodeData::ModuleDeclaration(md) => {
                    md.name.kind == SyntaxKind::StringLiteral
                }
                _ => false,
            };
            if name_is_string_literal {
                // Go bindModuleDeclaration ambient 分支：字符串名 ambient 模块恒 ValueModule
                self.declare_symbol(node, SymbolFlags::ValueModule, SymbolFlags::ValueModuleExcludes);
            } else {
                // Go declareModuleSymbol：按模块实例化状态取 ValueModule/NamespaceModule
                let state = get_module_instance_state(node);
                let (includes, excludes) = Self::module_symbol_flags(node);
                let symbol = self.declare_symbol(node, includes, excludes);
                if state != ModuleInstanceState::NonInstantiated {
                    let const_enum_only = !symbol
                        .flags
                        .intersects(SymbolFlags::Function | SymbolFlags::Class | SymbolFlags::RegularEnum)
                        && state == ModuleInstanceState::ConstEnumOnly
                        && !self.not_const_enum_only_modules.contains(&symbol.id());
                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    unsafe {
                        if const_enum_only {
                            (*symbol_mut).flags |= SymbolFlags::ConstEnumOnlyModule;
                        } else {
                            (*symbol_mut).flags &= !SymbolFlags::ConstEnumOnlyModule;
                        }
                    }
                    if !const_enum_only {
                        self.not_const_enum_only_modules.insert(symbol.id());
                    }
                }
            }
        }
    }

    fn module_symbol_flags(node: &Arc<Node>) -> (SymbolFlags, SymbolFlags) {
        let state = get_module_instance_state(node);
        if state != ModuleInstanceState::NonInstantiated {
            (SymbolFlags::ValueModule, SymbolFlags::ValueModuleExcludes)
        } else {
            (SymbolFlags::NamespaceModule, SymbolFlags::NamespaceModuleExcludes)
        }
    }
}
