#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use crate::checker::checker_resolve_checker::ANCESTRY_CONTAINERS;
use std::sync::Arc;

impl Checker {
    #[allow(unused_variables)]
    pub(crate) fn ancestry_lookup(
        &self,
        node: &Arc<Node>,
        chain: &std::collections::HashMap<u64, (Arc<Node>, Arc<Node>)>,
        name: &str,
        meaning: SymbolFlags,
        module_meaning: SymbolFlags,
        enum_meaning: SymbolFlags,
        type_meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

            let mut child = Arc::clone(node);
            let mut ancestor = node.parent();
            while let Some(a) = ancestor {
                let child_below = Arc::clone(&child);
                child = Arc::clone(&a);
                let next = a.parent();
                if ANCESTRY_CONTAINERS.contains(&a.kind) {
                    let aid = a.id();
                    // Go NameResolver：全局脚本文件的 locals 不参与上溯
                    //（已并入 globals，由末端回退按 meaning 解析）
                    if let Some(locals) = symbol_map.locals.get(&aid)
                        && !Self::is_global_source_file(&a)
                    {
                        if let Some(sym) = locals.get(name)
                            && self.locals_symbol_visible_at(&a, &child_below, sym, meaning)
                            && self.meaning_hit(sym, meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                    }
                    if let Some(a_sym) = symbol_map.symbols.get(&aid) {
                        match a.kind {
                            SyntaxKind::EnumDeclaration => {
                                if let Some(sym) = a_sym
                                    .members
                                    .get(name)
                                    .or_else(|| a_sym.exports.get(name))
                                    && self.meaning_hit(sym, enum_meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
                            }
                            SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
                                // Go NameResolver SourceFile 分支：全局脚本文件
                                // 跳过符号表查找（break），仅外部模块继续查
                                // members/exports；全局名由末端 globals 回退解析
                                if a.kind == SyntaxKind::SourceFile
                                    && !Self::is_global_source_file(&a)
                                    && let Some(sym) = a_sym.members.get(name)
                                    && self.meaning_hit(sym, meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
                                if a_sym.flags.intersects(SymbolFlags::MODULE)
                                    && !a_sym.flags.intersects(SymbolFlags::Class)
                                {
                                    if let Some(sym) = a_sym.exports.get(name) {
                                        // Go resolver：模块导出含纯 alias 的 export specifier
                                        //（export * as ns 亦同）不视为作用域内名字
                                        let is_export_specifier = sym.flags == SymbolFlags::Alias
                                            && sym.declarations.iter().any(|d| {
                                                d.kind == SyntaxKind::ExportSpecifier
                                                    || d.kind == SyntaxKind::NamespaceExport
                                            });
                                        if !is_export_specifier
                                            && self.meaning_hit(sym, module_meaning)
                                        {
                                            return Some(Arc::clone(sym));
                                        }
                                    }

                                    if let Some(merged) = self.globals.get(a_sym.name.as_str())
                                        && !Arc::ptr_eq(merged, a_sym)
                                        && merged.flags.intersects(SymbolFlags::MODULE)
                                    {
                                        if let Some(sym) = merged.exports.get(name)
                                            && self.meaning_hit(sym, module_meaning)
                                        {
                                            return Some(Arc::clone(sym));
                                        }
                                        if let Some(sym) =
                                            self.ambient_namespace_local(merged, name)
                                            && self.meaning_hit(&sym, meaning)
                                        {
                                            return Some(sym);
                                        }
                                    }
                                }
                            }
                            SyntaxKind::ClassDeclaration
                            | SyntaxKind::ClassExpression
                            | SyntaxKind::InterfaceDeclaration => {
                                if let Some(sym) = a_sym.members.get(name)
                                    && self.meaning_hit(sym, type_meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
                            }
                            _ => {
                                if let Some(sym) = self.ancestry_legacy_lookup(
                                    a_sym,
                                    name,
                                    meaning,
                                    type_meaning,
                                ) {
                                    return Some(sym);
                                }
                            }
                        }
                    }
                }
                ancestor = next;
            }
        None
    }
}
