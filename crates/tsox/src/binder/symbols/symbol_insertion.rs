#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn module_member_is_exported(&self, node: &Arc<Node>) -> bool {
        node.kind == SyntaxKind::ExportSpecifier
            || self
                .get_combined_modifier_flags(node)
                .contains(ModifierFlags::Export)
    }

    pub(crate) fn insert_symbol_into_container(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
        var_hoist_container: &Option<Arc<Node>>,
    ) {
        if let Some(container) = &self.container {
            if container.kind == SyntaxKind::ModuleDeclaration {
                let has_export = self.module_member_is_exported(node);

                let alias_no_local = has_export
                    && matches!(
                        node.kind,
                        SyntaxKind::ExportSpecifier | SyntaxKind::ImportEqualsDeclaration
                    );
                if has_export {
                    if let Some(parent_sym) = &self.parent_symbol {
                        let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                        unsafe {
                            (*parent_sym_mut)
                                .exports
                                .insert(name.to_string(), Arc::clone(&symbol));
                        }
                    }

                    if has_locals(container.kind) && !alias_no_local {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(container.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.to_string(), Arc::clone(&symbol));
                    }
                } else if has_locals(container.kind) {
                    let locals = self
                        .symbol_map
                        .locals
                        .entry(container.id())
                        .or_insert_with(SymbolTable::new);
                    locals.insert(name.to_string(), Arc::clone(&symbol));
                }
            } else if let Some(parent_sym) = &self.parent_symbol {
                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_sym_mut)
                        .members
                        .insert(name.to_string(), Arc::clone(&symbol));
                }
            } else if let Some(hoist) = &var_hoist_container {
                match hoist.kind {
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                        if let Some(sym) = self.symbol_map.symbol_of(hoist) {
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut)
                                    .members
                                    .insert(name.to_string(), Arc::clone(&symbol));
                            }
                        }
                    }
                    _ => {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(hoist.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.to_string(), Arc::clone(&symbol));
                    }
                }
            } else if let Some(block_container) = &self.block_scope_container {
                let container_id = block_container.id();
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container_id)
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(&symbol));
            }
        }
    }
}
