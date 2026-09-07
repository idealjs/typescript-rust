#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind_module_declaration(&mut self, node: &Arc<Node>) {
        let dotted_name = match &node.data {
            crate::ast::NodeData::ModuleDeclaration(md) => match md.name.kind {
                SyntaxKind::Identifier => md.name.text().to_string(),
                SyntaxKind::QualifiedName => {
                    fn qualified_text(n: &Arc<Node>) -> String {
                        match &n.data {
                            crate::ast::NodeData::QualifiedName(q) => {
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
            let symbol = Arc::new(Symbol::new(SymbolFlags::ValueModule, last.to_string()));
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
            self.declare_symbol(node, SymbolFlags::ValueModule, SymbolFlags::MODULE);
        }
    }
}
