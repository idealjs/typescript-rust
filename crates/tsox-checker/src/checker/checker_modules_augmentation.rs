#![allow(unused_imports)]

use crate::checker::checker::*;
use tsox_frontend::ast::Node;
use std::sync::Arc;

impl Checker {
    pub(crate) fn merge_module_augmentation(&mut self, name_node: &Arc<Node>, module_node: &Arc<Node>) {
        // 同文件多处增强共享合并符号：仅处理首个声明（Go combined-symbol guard）
        let Some(aug_sym) = self.program.symbol_map().symbol_of(module_node).cloned() else {
            return;
        };
        if aug_sym
            .declarations
            .first()
            .is_some_and(|d| !Arc::ptr_eq(d, module_node))
        {
            return;
        }

        if tsox_frontend::ast::is_global_scope_augmentation(module_node) {
            // Go mergeSymbolTable(c.globals, aug.Exports)：增强条目并入全局表，
            // 嵌套 members/exports 递归合并，冲突走 reportMergeSymbolError
            if !self.globals_populated {
                self.populate_globals();
                self.globals_populated = true;
            }
            let mut globals = std::mem::take(&mut self.globals);
            let source = aug_sym.exports.clone();
            self.merge_symbol_table(&mut globals, &source, false, None);
            self.globals = globals;
            return;
        }

        let spec = name_node
            .text()
            .trim_matches(['"', '\'', '`'])
            .to_string();
        let Some(file_node) =
            tsox_frontend::ast::utilities::get_source_file_of_node(module_node)
        else {
            return;
        };
        let Some(file_sym) = self.program.symbol_map().symbol_of(&file_node).cloned() else {
            return;
        };
        let Some(main_module) = self.resolve_module_spec_from(&file_sym, &spec) else {
            return;
        };
        let main_module = self.resolve_external_module_symbol_go(&main_module);
        if !main_module
            .flags
            .intersects(tsox_frontend::ast::SymbolFlags::NAMESPACE)
        {
            let file = self.get_source_file_of_node(module_node);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                name_node.loc,
                tsox_core::diagnostics::messages_generated::
                    CANNOT_AUGMENT_MODULE_0_BECAUSE_IT_RESOLVES_TO_A_NON_MODULE_ENTITY,
                vec![spec],
            ));
            return;
        }
        if self.module_augmentation_merge_conflict(&main_module, &aug_sym) {
            return;
        }
        self.merge_augmentation_exports(&main_module, &aug_sym);
    }

    /// Go mergeSymbol(mainModule, augmentation) 冲突闸门（checker.go:14388-14412）：
    /// 冲突且目标含 NamespaceModule → TS2649（globalThis 豁免），其余冲突 →
    /// reportMergeSymbolError；两种冲突都返回 true，调用方不得再执行合并
    pub(crate) fn module_augmentation_merge_conflict(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
    ) -> bool {
        let conflict = target
            .flags
            .intersects(get_excluded_symbol_flags(source.flags))
            && !(source.flags | target.flags)
                .intersects(tsox_frontend::ast::SymbolFlags::Assignment);
        if !conflict {
            return false;
        }
        if target
            .flags
            .intersects(tsox_frontend::ast::SymbolFlags::NamespaceModule)
        {
            self.report_cannot_augment_with_value_exports(target, source);
        } else {
            self.report_merge_symbol_error(target, source);
        }
        true
    }

    pub(crate) fn report_cannot_augment_with_value_exports(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
    ) {
        if self
            .global_this_symbol
            .as_ref()
            .is_some_and(|g| Arc::ptr_eq(g, target))
        {
            return;
        }
        let Some(first) = source.declarations.first() else {
            return;
        };
        let name_node = tsox_frontend::ast::utilities::get_name_of_declaration(first)
            .unwrap_or_else(|| Arc::clone(first));
        let file = self.get_source_file_of_node(&name_node);
        let target_name = self.symbol_to_string(target);
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            name_node.loc,
            tsox_core::diagnostics::messages_generated::
                CANNOT_AUGMENT_MODULE_0_WITH_VALUE_EXPORTS_BECAUSE_IT_RESOLVES_TO_A_NON_MODULE_ENTITY,
            vec![target_name],
        ));
    }

    /// 把增强符号的 exports 并入目标模块符号：同名条目合并声明（class+interface
    /// 等跨声明形态合并），缺失条目直接插入
    fn merge_augmentation_exports(
        &mut self,
        target: &Arc<Symbol>,
        augmentation: &Arc<Symbol>,
    ) {
        for (key, aug_export) in augmentation.exports.entries.iter() {
            if let Some(existing) = target.exports.entries.get(key).cloned() {
                if Arc::ptr_eq(&existing, aug_export) {
                    continue;
                }
                let mut declarations = existing.declarations.clone();
                for d in &aug_export.declarations {
                    if !declarations.iter().any(|e| Arc::ptr_eq(e, d)) {
                        declarations.push(Arc::clone(d));
                    }
                }
                let merged = Arc::new({
                    let mut s = tsox_frontend::ast::Symbol::new(
                        existing.flags | aug_export.flags,
                        existing.name.clone(),
                    );
                    s.declarations = declarations;
                    s
                });
                let target_mut = Arc::as_ptr(target) as *mut Symbol;
                unsafe {
                    (*target_mut).exports.entries.insert(key.clone(), merged);
                }
            } else {
                let target_mut = Arc::as_ptr(target) as *mut Symbol;
                unsafe {
                    (*target_mut)
                        .exports
                        .entries
                        .insert(key.clone(), Arc::clone(aug_export));
                }
            }
        }
    }
}
