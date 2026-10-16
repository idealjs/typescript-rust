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
        decorator_skip: Option<&std::collections::HashSet<u64>>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("scope_stack_lookup");
        let symbol_map = self.program.symbol_map();
        for &container_id in self.scope_stack.iter().rev() {
            // Go nameresolver.go KindDecorator 分支：装饰器与下一查找容器之间
            // 的容器整体跳过（类装饰器跳过类声明，成员/参数装饰器跳到类声明）
            if decorator_skip.is_some_and(|s| s.contains(&container_id)) {
                continue;
            }
            // Go NameResolver：全局脚本文件的 locals 不参与作用域查找
            //（已并入 globals，由末端回退按 meaning 解析）
            let container_is_global_source_file = chain
                .get(&container_id)
                .is_some_and(|(c, _)| Self::is_global_source_file(c));
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && !container_is_global_source_file
            {
                if let Some(sym) = locals.get(name)
                    && chain.get(&container_id).is_none_or(|(c, child)| {
                        self.locals_symbol_visible_at(c, child, sym, meaning)
                    })
                    && let Some(hit) = self.locals_lookup_hit(sym, meaning)
                {
                    return Some(hit);
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
                        // Go NameResolver SourceFile 分支：全局脚本文件跳过
                        // 符号表查找（break），仅外部模块继续；全局名由末端
                        // globals 回退解析
                        if c.kind == SyntaxKind::SourceFile
                            && !Self::is_global_source_file(c)
                        {
                            // Go nameresolver.go:122-129：外部模块文件容器先按
                            // export default 的本地名匹配——exports["default"]
                            // 默认符号满足 meaning 且其本地符号（declareModuleMember
                            // binder.go:427-431 所建、名字为声明名）与查找名相同
                            // 时，名字解析到默认导出符号本身
                            //（GetLocalSymbolForExportDefault nameresolver.go:459-471；
                            // 与 ancestry_lookup 同臂——本仓解析分 scope_stack 与
                            // ancestry 两走查，Go 单一 SourceFile case 的该臂两侧
                            // 皆须在位，default 导出 local（local+export 对）与
                            // import 合流出的纯别名符号类型位由此供接口符号）
                            if name != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT
                                && let Some(default_sym) = container_sym
                                    .exports
                                    .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT)
                                && default_sym.flags.intersects(meaning)
                                && Self::default_export_local_name(
                                    default_sym,
                                    c,
                                    &symbol_map,
                                )
                                .is_some_and(|local_name| local_name == name)
                            {
                                return Some(Arc::clone(default_sym));
                            }
                            if let Some(sym) = container_sym.members.get(name)
                                && self.meaning_hit(sym, meaning)
                            {
                                return Some(Arc::clone(sym));
                            }
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
                        // Go nameresolver.go:199-205：命名类表达式容器——
                        // 成员查找落空后，名字与类表达式名相同且含义含
                        // Class 的引用取类表达式自身符号
                        if c.kind == SyntaxKind::ClassExpression
                            && meaning.intersects(SymbolFlags::Class)
                            && c.name().is_some_and(|n| n.text() == name)
                        {
                            return Some(Arc::clone(container_sym));
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

    pub(crate) fn meaning_hit(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool { ::tsox_core::fntrace::enter("meaning_hit"); 
        sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(sym, meaning)
    }

    // Go resolveName 的 getSymbol（checker.go:2275-2293）对 locals 命中先按
    // 查询含义过滤 local 本身：declareModuleMember（binder.go:406-416）local
    // 侧占位被后续非导出声明合并后（interface __proto__ 并入 export var
    // __proto__ 的占位，declareSymbol binder.go:232-291 无冲突合并臂）带
    // Type/Value 位，Type 含义查询必须命中 local 本身（intrinsics.ts 的
    // { __proto__: __proto__ }），映射 export 符号会以 Variable 位落空跌到
    // exports 臂取错符号报 TS2749。纯 ExportValue 占位（ExportValue 不入
    // Value/Type/Namespace 任何 meaning，nameresolver.go:60 必落空）与全含义
    // 查询（resolve_identifier_use 以 ALL 解析、下游消费方未全部经
    // checkIdentifier checker.go:11669 映射）保留基线捷径：映射 export 符号，
    // 结局与 Go 落容器 exports 臂（nameresolver.go:147-158）一致
    pub(crate) fn locals_lookup_hit(
        &self,
        sym: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("locals_lookup_hit");
        if meaning != SymbolFlags::all() {
            let merged = self.get_merged_symbol(sym);
            if self.meaning_hit(&merged, meaning) {
                return Some(merged);
            }
        }
        let effective = self.get_export_symbol_of_value_symbol_if_exported(sym);
        if self.meaning_hit(&effective, meaning) {
            return Some(effective);
        }
        None
    }
}
