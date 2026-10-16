#![allow(unused_imports)]

use crate::binder::mig::r19k5_ext::{set_export_symbol, set_symbol_flags_or};
use crate::binder::symbols::*;
use crate::binder::symbols_binder_3::set_value_declaration;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SymbolTable};

enum SideTarget {
    Locals(Arc<Node>),
    Members(Arc<Symbol>),
    Exports(Arc<Symbol>),
}

impl Binder {
    // Go declareModuleMember（binder.go:406-416）：模块容器内带 export 的非别名
    // 成员拆 local/export 双符号——local 侧以 exportKind 元位（值成员 ExportValue）
    // 并入容器 locals 表，export 侧以完整 includes 建/并入容器符号 exports 表
    // （parent 挂容器符号）；local.ExportSymbol 指向 export 侧，node.symbol 经
    // addDeclarationToSymbol（binder.go:2690）声明序覆写最终持 export 符号。
    // 外部模块 SourceFile 容器（declareSourceFileMember binder.go:458-462 同走
    // declareModuleMember）的顶层符号表为文件符号 members（本仓 insert_symbol_
    // into_container 的 SourceFile 臂），local 侧同表才与同位 import alias 合并
    pub(crate) fn declare_export_module_member_pair(
        &mut self,
        node: &Arc<Node>,
        name: &str,
        includes: SymbolFlags,
        parent_sym: &Arc<Symbol>,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("declare_export_module_member_pair");
        let excludes = Self::excludes_for_declaration(node, includes);
        let container = self.container.clone().unwrap();
        let export_kind = if includes.intersects(SymbolFlags::VALUE) {
            SymbolFlags::ExportValue
        } else {
            SymbolFlags::empty()
        };
        let locals_target = if container.kind == SyntaxKind::SourceFile {
            SideTarget::Members(Arc::clone(parent_sym))
        } else {
            SideTarget::Locals(Arc::clone(&container))
        };
        let local = self.declare_symbol_side_go(
            node,
            name,
            export_kind,
            excludes,
            locals_target,
        );
        let export_symbol = self.declare_symbol_side_go(
            node,
            name,
            includes,
            excludes,
            SideTarget::Exports(Arc::clone(parent_sym)),
        );
        set_export_symbol(&local, Arc::clone(&export_symbol));
        self.symbol_map.set_symbol(node, Arc::clone(&export_symbol));
        local
    }

    // Go declareSymbolEx（binder.go:159-301）单表侧核心：既有符号 excludes 相交
    // 即报所有声明冲突并另立不入表的新符号（binder.go:279-281），无冲突按
    // addDeclarationToSymbol（binder.go:2689-2703）无条件并入
    fn declare_symbol_side_go(
        &mut self,
        node: &Arc<Node>,
        name: &str,
        includes: SymbolFlags,
        excludes: SymbolFlags,
        target: SideTarget,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("declare_symbol_side_go");
        let existing = match &target {
            SideTarget::Exports(parent_sym) => parent_sym.exports.get(name).cloned(),
            SideTarget::Locals(container) => self
                .symbol_map
                .locals
                .get(&container.id())
                .and_then(|l| l.get(name).cloned()),
            SideTarget::Members(parent_sym) => parent_sym.members.get(name).cloned(),
        };
        if let Some(existing) = existing {
            // Go binder.go:231-233：Assignment 声明可与变量合并
            let assignment_merge_exception = (includes.contains(SymbolFlags::VARIABLE)
                && existing.flags.contains(SymbolFlags::Assignment))
                || (includes.contains(SymbolFlags::Assignment)
                    && existing.flags.contains(SymbolFlags::VARIABLE));
            if !excludes.is_empty()
                && existing.flags.intersects(excludes)
                && !assignment_merge_exception
            {
                // Go binder.go:236-243：消息按既有符号位选择（enum > block-scoped
                // > duplicate identifier），报既有与当前全部声明
                if existing.flags.intersects(SymbolFlags::ENUM)
                    || includes.intersects(SymbolFlags::ENUM)
                {
                    self.report_declaration_conflict_all(
                        node,
                        &existing,
                        None,
                        &tsox_core::diagnostics::messages_generated::
                            ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS,
                    );
                } else if existing.flags.contains(SymbolFlags::BlockScopedVariable) {
                    self.report_declaration_conflict_all(
                        node,
                        &existing,
                        Some(name),
                        &CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0,
                    );
                } else {
                    self.report_duplicate_identifier_all(node, &existing, name);
                }
                // Go binder.go:266-270：访问器与非访问器冲突后整符号标记 Accessor
                if existing.flags.intersects(SymbolFlags::ACCESSOR)
                    && (existing.flags & SymbolFlags::ACCESSOR)
                        != (includes & SymbolFlags::ACCESSOR)
                {
                    set_symbol_flags_or(&existing, SymbolFlags::ACCESSOR);
                }
                let symbol = self.new_symbol(includes, name.to_string());
                {
                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    unsafe {
                        (*symbol_mut).declarations.push(Arc::clone(node));
                    }
                }
                if includes.intersects(SymbolFlags::VALUE) {
                    set_value_declaration(&symbol, node);
                }
                self.symbol_map.set_symbol(node, Arc::clone(&symbol));
                return symbol;
            }
            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
            unsafe {
                (*existing_mut).declarations.push(Arc::clone(node));
                (*existing_mut).flags |= includes;
            }
            if includes.intersects(SymbolFlags::VALUE) {
                set_value_declaration(&existing, node);
            }
            self.symbol_map.set_symbol(node, Arc::clone(&existing));
            return existing;
        }
        let symbol = self.new_symbol(includes, name.to_string());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
            }
        }
        if includes.intersects(SymbolFlags::VALUE) {
            set_value_declaration(&symbol, node);
        }
        match &target {
            SideTarget::Exports(parent_sym) => {
                let parent_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*parent_mut)
                        .exports
                        .insert(name.to_string(), Arc::clone(&symbol));
                    // binder.go:299-301：export 侧 parent 挂容器符号（显示限定名用）
                    if (*symbol_mut).parent().is_none() {
                        (*symbol_mut).set_parent(parent_sym);
                    }
                }
            }
            SideTarget::Locals(container) => {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(&symbol));
            }
            SideTarget::Members(parent_sym) => {
                let parent_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*parent_mut)
                        .members
                        .insert(name.to_string(), Arc::clone(&symbol));
                }
            }
        }
        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }
}
