#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn scope_stack_lookup(
        &self,
        chain: &std::collections::HashMap<u64, (Arc<Node>, Arc<Node>)>,
        name: &str,
        meaning: SymbolFlags,
        module_meaning: SymbolFlags,
        enum_meaning: SymbolFlags,
        type_meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name)
                    && chain.get(&container_id).is_none_or(|(c, child)| {
                        self.locals_symbol_visible_at(c, child, sym, meaning)
                    })
                    && self.meaning_hit(sym, meaning)
                {
                    return Some(Arc::clone(sym));
                }
            }

            let Some(container_sym) = symbol_map.symbols.get(&container_id) else {
                continue;
            };
            if let Some((c, _)) = chain.get(&container_id) {
                match c.kind {
                    SyntaxKind::EnumDeclaration => {
                        if let Some(sym) = container_sym
                            .members
                            .get(name)
                            .or_else(|| container_sym.exports.get(name))
                            && self.meaning_hit(sym, enum_meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                        continue;
                    }
                    SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
                        if c.kind == SyntaxKind::SourceFile
                            && let Some(sym) = container_sym.members.get(name)
                            && self.meaning_hit(sym, meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                        if container_sym.flags.intersects(SymbolFlags::MODULE)
                            && !container_sym.flags.intersects(SymbolFlags::Class)
                        {
                            if let Some(sym) = container_sym.exports.get(name) {
                                let is_export_specifier = sym.flags == SymbolFlags::Alias
                                    && sym.declarations.iter().any(|d| {
                                        d.kind == SyntaxKind::ExportSpecifier
                                            || d.kind == SyntaxKind::NamespaceExport
                                    });
                                if !is_export_specifier && self.meaning_hit(sym, module_meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
                            }
                            if let Some(sym) =
                                self.augmentation_target_member(container_sym, name)
                                && self.meaning_hit(&sym, meaning)
                            {
                                return Some(sym);
                            }
                            if let Some(merged) = self.globals.get(container_sym.name.as_str())
                                && !Arc::ptr_eq(merged, container_sym)
                                && merged.flags.intersects(SymbolFlags::MODULE)
                            {
                                if let Some(sym) = merged.exports.get(name)
                                    && self.meaning_hit(sym, module_meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
                                if let Some(sym) = self.ambient_namespace_local(merged, name)
                                    && self.meaning_hit(&sym, meaning)
                                {
                                    return Some(sym);
                                }
                            }
                        }
                        continue;
                    }
                    SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration => {
                        if let Some(sym) = container_sym.members.get(name)
                            && self.meaning_hit(sym, type_meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                        continue;
                    }
                    _ => {}
                }
            }

            if !container_sym.flags.intersects(SymbolFlags::Class)
                || container_sym.flags.intersects(SymbolFlags::Function)
            {
                if let Some(sym) = container_sym.members.get(name) {
                    // Go nameresolver Resolve 的 InterfaceDeclaration 容器分支：
                    // 成员查找以 meaning&Type 限定且命中须为声明于本容器的类型参数，
                    // 同名属性成员不可见（interface B3 { Date: Date } 的注解位
                    // 须继续向外解析到全局 interface Date）
                    let interface_member_visible = !container_sym
                        .flags
                        .intersects(SymbolFlags::Interface)
                        || sym.flags.contains(SymbolFlags::TypeParameter);
                    if interface_member_visible && self.meaning_hit(sym, meaning) {
                        return Some(Arc::clone(sym));
                    }
                }
            }

            if container_sym.flags.intersects(SymbolFlags::MODULE)
                && !container_sym.flags.intersects(SymbolFlags::Class)
                && !chain.contains_key(&container_id)
            {
                if let Some(sym) = container_sym.exports.get(name) {
                    let is_export_specifier = sym.flags == SymbolFlags::Alias
                        && sym
                            .declarations
                            .iter()
                            .any(|d| {
                                d.kind == SyntaxKind::ExportSpecifier
                                    || d.kind == SyntaxKind::NamespaceExport
                            });
                    if !is_export_specifier && self.meaning_hit(sym, meaning) {
                        return Some(Arc::clone(sym));
                    }
                }

                // Go resolver：declare module "foo" 增强块内的名字可见性
                // 延伸到被增强模块的 exports
                if let Some(sym) = self.augmentation_target_member(container_sym, name) {
                    if self.meaning_hit(&sym, meaning) {
                        return Some(sym);
                    }
                }

                if let Some(merged) = self.globals.get(container_sym.name.as_str()) {
                    if !Arc::ptr_eq(merged, container_sym)
                        && merged.flags.intersects(SymbolFlags::MODULE)
                    {
                        if let Some(sym) = merged.exports.get(name)
                            && self.meaning_hit(sym, meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                        if let Some(sym) = self.ambient_namespace_local(merged, name)
                            && self.meaning_hit(&sym, meaning)
                        {
                            return Some(sym);
                        }
                    }
                }
            }

            if container_sym.flags.intersects(SymbolFlags::ENUM)
                && !chain.contains_key(&container_id)
            {
                if let Some(sym) = container_sym.exports.get(name)
                    && self.meaning_hit(sym, meaning)
                {
                    return Some(Arc::clone(sym));
                }
            }

            if let Some(sym) = container_sym.members.get(name) {
                // 同上：Go nameresolver 的 interface/class 容器成员查找仅
                // 类型参数可见（meaning&Type + isTypeParameterSymbolDeclaredInContainer）
                let interface_member_visible = !container_sym
                    .flags
                    .intersects(SymbolFlags::Interface)
                    || sym.flags.contains(SymbolFlags::TypeParameter);
                if interface_member_visible && self.meaning_hit(sym, meaning & SymbolFlags::TYPE) {
                    return Some(Arc::clone(sym));
                }
            }
        }

        None
    }

    pub(crate) fn meaning_hit(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool {
        sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(sym, meaning)
    }
}
