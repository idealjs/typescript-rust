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
        decorator_skip: Option<&std::collections::HashSet<u64>>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("ancestry_lookup");
        let symbol_map = self.program.symbol_map();

            let mut child = Arc::clone(node);
            let mut ancestor = node.parent();
            while let Some(a) = ancestor {
            let child_below = Arc::clone(&child);
            child = Arc::clone(&a);
            let next = a.parent();
            // Go nameresolver.go:207-223 KindExpressionWithTypeArguments 特例：
            // 解析自 extends 子句 EWT 的表达式子树（lastLocation ==
            // EWT.Expression()）上溯、名字以 Type meaning 命中宿主类
            // members（类自身型参）时 Resolve 整体返回 nil——Base class
            // expressions cannot reference class type parameters。
            // typeParameterShadowsOtherTypeParameterInScope
            //（nodebuilderimpl.go:1519-1526）经同一 Resolve，heritage 位
            // 不解出容器类型参，函数型参名不改写（baseExpressionTypeParameters
            // 的 `>base : <T>() => typeof Base` 不呈 T_1）
            if a.kind == SyntaxKind::ExpressionWithTypeArguments
                && let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ewt) = &a.data
                && Arc::ptr_eq(&child_below, &ewt.expression)
                && a.parent().is_some_and(|h| {
                    h.kind == SyntaxKind::HeritageClause
                        && matches!(
                            &h.data,
                            tsox_frontend::ast::NodeData::HeritageClause(hd)
                                if hd.token == SyntaxKind::ExtendsKeyword
                        )
                })
            {
                let in_extends_class = a
                    .parent()
                    .and_then(|h| h.parent())
                    .is_some_and(|g| {
                        tsox_frontend::ast::is_class_like(&g)
                            && symbol_map
                                .symbols
                                .get(&g.id())
                                .and_then(|class_sym| class_sym.members.get(name))
                                .is_some_and(|sym| self.meaning_hit(sym, type_meaning))
                    });
                if in_extends_class {
                    return None;
                }
            }
            // Go nameresolver.go KindDecorator 分支：装饰器与下一查找容器
            // 之间的容器整体跳过（类装饰器跳过类声明，成员/参数装饰器跳
            // 到类声明）
            if decorator_skip.is_some_and(|s| s.contains(&a.id())) {
                ancestor = next;
                continue;
            }
                if ANCESTRY_CONTAINERS.contains(&a.kind) {
                    let aid = a.id();
                    // Go NameResolver：全局脚本文件的 locals 不参与上溯
                    //（已并入 globals，由末端回退按 meaning 解析）
                    if let Some(locals) = symbol_map.locals.get(&aid)
                        && !Self::is_global_source_file(&a)
                    {
                        if let Some(sym) = locals.get(name)
                            && self.locals_symbol_visible_at(&a, &child_below, sym, meaning)
                            && let Some(hit) = self.locals_lookup_hit(sym, meaning)
                        {
                            return Some(hit);
                        }
                    }
                    // Go nameresolver.go:238-247（switch 于 locals 查找之后）：
                    // 上溯命中函数型容器时 `arguments` 结构化解析为
                    // argumentsSymbol（resolvedType 已置 IArguments），不依赖
                    // 检查期 function_scope_count，.types 发射期同样生效；
                    // 容器局部 `arguments`（参数/变量）先行命中并遮蔽
                    if meaning.intersects(SymbolFlags::VARIABLE)
                        && name == "arguments"
                        && matches!(
                            a.kind,
                            SyntaxKind::MethodDeclaration
                                | SyntaxKind::Constructor
                                | SyntaxKind::GetAccessor
                                | SyntaxKind::SetAccessor
                                | SyntaxKind::FunctionDeclaration
                                | SyntaxKind::FunctionExpression
                        )
                        && let Some(ref sym) = self.arguments_symbol
                    {
                        return Some(Arc::clone(sym));
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
                                {
                                    // Go nameresolver.go:122-130：外部模块文件容器
                                    // 先按 export default 的本地名匹配——
                                    // exports["default"] 默认符号满足 meaning 且其
                                    // 本地符号（declareModuleMember binder.go:427-431
                                    // 所建、名字为声明名）与查找名相同时，名字解析到
                                    // 默认导出符号本身（GetLocalSymbolForExportDefault
                                    // nameresolver.go:459-471 遍历声明取 LocalSymbol；
                                    // 本仓节点不挂 localSymbol，以直属容器 locals 表
                                    // 按声明名等价取）
                                    if name
                                        != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT
                                        && let Some(default_sym) = a_sym
                                            .exports
                                            .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT)
                                        && default_sym.flags.intersects(meaning)
                                        && Self::default_export_local_name(
                                            default_sym,
                                            &a,
                                            &symbol_map,
                                        )
                                        .is_some_and(|local_name| local_name == name)
                                    {
                                        return Some(Arc::clone(default_sym));
                                    }
                                    if let Some(sym) = a_sym.members.get(name)
                                        && self.meaning_hit(sym, meaning)
                                    {
                                        return Some(Arc::clone(sym));
                                    }
                                }
                                // Go nameresolver.go:147-158：模块容器 exports
                                // 查找无 class 合并符号排除条件（class+namespace
                                // 合并符号的 exports 即命名空间成员表）
                                if a_sym.flags.intersects(SymbolFlags::MODULE) {
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
                                // Go nameresolver.go:199-205：命名类表达式容器——
                                // 成员查找落空后，名字与类表达式名相同且含义含
                                // Class 的引用取类表达式自身符号
                                if a.kind == SyntaxKind::ClassExpression
                                    && meaning.intersects(SymbolFlags::Class)
                                    && a.name().is_some_and(|n| n.text() == name)
                                {
                                    return Some(Arc::clone(a_sym));
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

    // Go GetLocalSymbolForExportDefault（nameresolver.go:459-471）+ 节点
    // LocalSymbol（declareModuleMember binder.go:430）的 checker 侧等价：
    // default 符号首声明带 Default 修饰符时取其本地符号名（声明名）。本仓
    // 节点不挂 localSymbol，本地符号以直属容器 locals 表按声明名定位
    // （binder local+export 对已插入该表）
    pub(crate) fn default_export_local_name(
        default_sym: &Arc<Symbol>,
        container: &Arc<Node>,
        symbol_map: &tsox_frontend::ast::NodeSymbolMap,
    ) -> Option<String> {
        let decl = default_sym.declarations.first()?;
        if !decl.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Default) {
            return None;
        }
        let decl_name = decl.name()?.text().to_string();
        if decl_name.is_empty() {
            return None;
        }
        symbol_map
            .locals
            .get(&container.id())?
            .get(&decl_name)?;
        Some(decl_name)
    }
}
