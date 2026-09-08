#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    // Go binder 各声明类别的 excludes（值成员互斥表）
    fn excludes_for_declaration(kind: SyntaxKind) -> SymbolFlags {
        let value = SymbolFlags::VALUE;
        match kind {
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                value & !SymbolFlags::Method
            }
            SyntaxKind::GetAccessor => value & !(SymbolFlags::SetAccessor | SymbolFlags::Property),
            SyntaxKind::SetAccessor => value & !(SymbolFlags::GetAccessor | SymbolFlags::Property),
            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyAssignment => value & !SymbolFlags::Property,
            _ => SymbolFlags::empty(),
        }
    }

    pub(crate) fn declare_symbol(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        let var_hoist_container: Option<Arc<Node>> =
            if Self::declaration_is_var(node) && self.parent_symbol.is_none() {
                self.container
                    .as_ref()
                    .filter(|c| is_var_container_kind(c.kind))
                    .cloned()
            } else {
                None
            };

        let is_module_member_container = self
            .container
            .as_ref()
            .is_some_and(|c| c.kind == SyntaxKind::ModuleDeclaration);

        let existing: Option<Arc<Symbol>> =
            if is_module_member_container && let Some(parent_sym) = &self.parent_symbol {
                let has_export = self.module_member_is_exported(node);
                let container_id = self.container.as_ref().unwrap().id();
                let locals_hit = || {
                    self.symbol_map
                        .locals
                        .get(&container_id)
                        .and_then(|l| l.get(&name).cloned())
                };
                if includes.contains(SymbolFlags::Alias) {
                    if has_export {
                        parent_sym.exports.get(&name).cloned()
                    } else {
                        locals_hit()
                    }
                } else if has_export {
                    parent_sym.exports.get(&name).cloned().or_else(locals_hit)
                } else {
                    locals_hit()
                }
            } else if let Some(parent_sym) = &self.parent_symbol {
                parent_sym
                    .members
                    .get(&name)
                    .cloned()
                    .or_else(|| parent_sym.exports.get(&name).cloned())
            } else if let Some(hoist) = &var_hoist_container {
                match hoist.kind {
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => self
                        .symbol_map
                        .symbol_of(hoist)
                        .and_then(|sym| sym.members.get(&name).cloned()),

                    _ => {
                        let container_id = hoist.id();
                        self.symbol_map
                            .locals
                            .get(&container_id)
                            .and_then(|locals| locals.get(&name).cloned())
                    }
                }
            } else if let Some(block_container) = &self.block_scope_container {
                let container_id = block_container.id();
                self.symbol_map
                    .locals
                    .get(&container_id)
                    .and_then(|locals| locals.get(&name).cloned())
            } else {
                None
            };

        let mut conflicted = false;

        if let Some(existing) = existing {
            // Go binder declareSymbol 冲突表：按声明类别 excludes 判定冲突
            // （method 与 accessor 互斥、property 与 accessor 互斥等），
            // 冲突时报所有既有声明 + 当前声明的 Duplicate identifier，且不合并符号
            let excludes = Self::excludes_for_declaration(node.kind);
            let assignment_merge_exception = (includes.contains(SymbolFlags::FunctionScopedVariable)
                && existing.flags.contains(SymbolFlags::Assignment))
                || (includes.contains(SymbolFlags::Assignment)
                    && existing
                        .flags
                        .contains(SymbolFlags::FunctionScopedVariable));
            if !excludes.is_empty()
                && !name.is_empty()
                && existing.flags.intersects(excludes)
                && !assignment_merge_exception
            {
                self.report_duplicate_identifier_all(node, &existing, &name);
                if existing.flags.intersects(SymbolFlags::ACCESSOR)
                    && (existing.flags & SymbolFlags::ACCESSOR) != (includes & SymbolFlags::ACCESSOR)
                {
                    let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                    unsafe {
                        (*existing_mut).flags |= SymbolFlags::ACCESSOR;
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
                self.insert_symbol_into_container(node, &symbol, &name, &var_hoist_container);
                self.symbol_map.set_symbol(node, Arc::clone(&symbol));
                return symbol;
            }
            if let Some(merged) = self.merge_into_existing_symbol(node, &existing, includes) {
                return merged;
            }

            conflicted = self.report_symbol_conflict(node, &existing, &name, includes);
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

        if !conflicted {
            self.insert_symbol_into_container(node, &symbol, &name, &var_hoist_container);
        }

        if let Some(container) = &self.container {
            let is_module_container = container.kind == SyntaxKind::SourceFile
                || container.kind == SyntaxKind::ModuleDeclaration;
            if is_module_container
                && self
                    .get_combined_modifier_flags(node)
                    .contains(ModifierFlags::Export)
            {
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*symbol_mut).export_symbol = Some(Arc::clone(&symbol));
                }
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));

        symbol
    }
}
