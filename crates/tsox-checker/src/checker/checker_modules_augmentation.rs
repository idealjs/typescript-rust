#![allow(unused_imports)]

use crate::checker::checker::*;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::{Node, SourceFile, SyntaxKind};
use std::sync::Arc;

impl Checker {
    /// Go mergeModuleAugmentation 前置遍历：在任何文件检查之前，把本文件
    /// 顶层的外部模块增强（`declare module "spec"`）合并进目标模块符号，
    /// 使后续各文件的类型解析能看到增强成员
    pub fn merge_module_augmentations_in_file(&mut self, file: &Arc<SourceFile>) {
        let statements: Vec<Arc<Node>> = match &file.node.data {
            NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
            _ => return,
        };
        for stmt in &statements {
            if stmt.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            if tsox_frontend::ast::is_global_scope_augmentation(stmt) {
                self.merge_module_augmentation(&stmt, stmt);
                continue;
            }
            let name_node = match &stmt.data {
                NodeData::ModuleDeclaration(md) => Arc::clone(&md.name),
                _ => continue,
            };
            if name_node.kind != SyntaxKind::StringLiteral {
                continue;
            }
            if !Self::is_external_module_augmentation(stmt) {
                continue;
            }
            self.merge_module_augmentation(&name_node, stmt);
        }
    }

    fn merge_module_augmentation(&mut self, name_node: &Arc<Node>, module_node: &Arc<Node>) {
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
            // Go mergeSymbolTable(c.globals, aug.Exports)：增强条目并入全局表
            // （同名列合并 declarations，缺失插入）
            if !self.globals_populated {
                self.populate_globals();
                self.globals_populated = true;
            }
            for (key, aug_export) in aug_sym.exports.entries.iter() {
                match self.globals.entries.get(key).cloned() {
                    Some(existing) => {
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
                            s.declarations = declarations.clone();
                            s
                        });
                        self.globals.entries.insert(key.clone(), merged);
                    }
                    None => {
                        self.globals
                            .entries
                            .insert(key.clone(), Arc::clone(aug_export));
                    }
                }
            }
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
        self.merge_augmentation_exports(&main_module, &aug_sym);
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
