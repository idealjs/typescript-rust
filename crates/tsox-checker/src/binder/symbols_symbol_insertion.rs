#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    pub(crate) fn module_member_is_exported(&self, node: &Arc<Node>) -> bool {
        node.kind == SyntaxKind::ExportSpecifier
            || self
                .get_combined_modifier_flags(node)
                .contains(ModifierFlags::Export)
            // Go declareModuleMember：ambient 容器的 ExportContext 内隐式导出
            || self
                .container
                .as_ref()
                .is_some_and(|c| c.flags.contains(NodeFlags::ExportContext))
    }

    pub(crate) fn insert_symbol_into_container(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
        var_hoist_container: &Option<Arc<Node>>,
    ) {
        if let Some(container) = &self.container {
            // Go bindBlockScopedDeclaration：块作用域声明的目标由
            // blockScopeContainer 决定（catch/嵌套块内 let/const 入块容器
            // locals），仅 blockScopeContainer 即模块/文件容器时走
            // declareModuleMember
            let reroute_block_scoped = var_hoist_container.is_none()
                && symbol.flags.contains(SymbolFlags::BlockScopedVariable)
                && self
                    .block_scope_container
                    .as_ref()
                    .is_some_and(|b| b.id() != container.id());
            if container.kind == SyntaxKind::ModuleDeclaration && !reroute_block_scoped {
                let has_export = self.module_member_is_exported(node);

                let alias_no_local = has_export
                    && matches!(
                        node.kind,
                        SyntaxKind::ExportSpecifier | SyntaxKind::ImportEqualsDeclaration
                    );
                if has_export {
                    if let Some(parent_sym) = &self.parent_symbol {
                        let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                        let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
                        unsafe {
                            (*parent_sym_mut)
                                .exports
                                .insert(name.to_string(), Arc::clone(&symbol));
                            // 命名空间成员挂父链（显示限定名用，对齐 tsc symbol.parent）
                            if (*symbol_mut).parent().is_none() {
                                (*symbol_mut).set_parent(parent_sym);
                            }
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
                    if let Some(parent_sym) = &self.parent_symbol {
                        let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
                        unsafe {
                            if (*symbol_mut).parent().is_none() {
                                (*symbol_mut).set_parent(parent_sym);
                            }
                        }
                    }
                }
            } else if var_hoist_container.is_none()
                && let Some(block_container) = &self.block_scope_container
                && self
                    .container
                    .as_ref()
                    .is_none_or(|c| c.id() != block_container.id())
            {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(block_container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(symbol));
            } else if is_function_like_locals_container(container.kind) {
                // Go declareSymbolAndAddToSymbolTable 的函数类容器分支：
                // 进容器节点 locals、parent 不挂。插进容器符号 members 会让
                // 重载声明的同名类型参数相互覆盖（符号表是按名字索引的）
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(&symbol));
            } else if let Some(parent_sym) = &self.parent_symbol {
                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
                unsafe {
                    // Go declareClassMember：类的 static 成员进 parentSymbol.Exports
                    //（与实例成员分表，同名 static/实例为两个符号）
                    let parent_is_class = parent_sym.declarations.iter().any(|d| {
                        matches!(
                            d.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                        )
                    });
                    let is_static_member = node.has_syntactic_modifier(ModifierFlags::Static)
                        && matches!(
                            node.kind,
                            SyntaxKind::PropertyDeclaration
                                | SyntaxKind::MethodDeclaration
                                | SyntaxKind::GetAccessor
                                | SyntaxKind::SetAccessor
                        );
                    // export 语境的别名进 exports 表（对齐 Go declareModuleMember），
                    // 其余本地声明进 members
                    if matches!(
                        node.kind,
                        SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceExportDeclaration
                    ) || (parent_is_class && is_static_member)
                    {
                        (*parent_sym_mut)
                            .exports
                            .insert(name.to_string(), Arc::clone(symbol));
                    } else {
                        (*parent_sym_mut)
                            .members
                            .insert(name.to_string(), Arc::clone(symbol));
                        // Go declareSourceFileMember：外部模块文件的顶层导出成员
                        // 同时进 exports（default 导出走各自专用路径，此处排除）
                        let file_is_external_module = self
                            .current_source_file
                            .as_ref()
                            .is_some_and(|f| f.external_module_indicator.is_some());
                        if file_is_external_module
                            && self
                                .get_combined_modifier_flags(node)
                                .contains(ModifierFlags::Export)
                            && !self
                                .get_combined_modifier_flags(node)
                                .contains(ModifierFlags::Default)
                        {
                            (*parent_sym_mut)
                                .exports
                                .insert(name.to_string(), Arc::clone(symbol));
                        }
                    }
                    (*symbol_mut).set_parent(parent_sym);
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
