#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn resolve_identifier_scope_symbol(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let result = self.resolve_identifier_scope_symbol_in_scopes(node, meaning);
        if result.is_some()
            && self.is_invalid_property_initializer_reference(node, meaning, result.as_ref())
        {
            return None;
        }
        result
    }

    // Go nameresolver.go Resolve 的 KindPropertyDeclaration 分支：上溯途经非静态
    // 属性声明时，若其所在类构造器（有 body）的 locals 含同名值符号，则本次
    // 解析无效（resolveEntity 经 OnPropertyWithInvalidInitializer 返回 nil，
    // 引用类型落 errorType 渲染 any；GetEmitStandardClassFields 下不生效）。
    // 解析结果落在该属性初始化器子树内（嵌套函数/绑定模式自身的同名绑定）时
    // Go 上溯在更内层已 break loop，不记 propertyWithInvalidInitializer，不作废
    pub(crate) fn is_invalid_property_initializer_reference(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
        result: Option<&Arc<Symbol>>,
    ) -> bool {
        let Some(property) = self.find_property_with_invalid_initializer(node, meaning) else {
            return false;
        };
        match result {
            Some(sym) if sym.declarations.iter().any(|d| {
                tsox_frontend::ast::find_ancestor(d, |a| std::ptr::eq(a, property.as_ref()))
                    .is_some()
            }) => false,
            _ => true,
        }
    }

    fn find_property_with_invalid_initializer(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_property_with_invalid_initializer");
        if self.emit_standard_class_fields {
            return None;
        }
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let value_meaning = meaning & SymbolFlags::VALUE;
        let symbol_map = self.program.symbol_map();
        let mut found: Option<Arc<Node>> = None;
        let mut current = node.parent();
        while let Some(a) = current {
            if a.kind == SyntaxKind::PropertyDeclaration
                && !a.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static)
                && let Some(class) = a.parent()
                && let Some(ctor) =
                    tsox_frontend::ast::mig::m3e_4::find_constructor_declaration(&class)
                && let Some(sym) = symbol_map
                    .locals
                    .get(&ctor.id())
                    .and_then(|locals| locals.get(name))
                && sym.flags.intersects(value_meaning)
            {
                found = Some(Arc::clone(&a));
            }
            current = a.parent();
        }
        found
    }

    fn resolve_identifier_scope_symbol_in_scopes(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_identifier_scope_symbol");
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        if tsox_frontend::ast::node_is_missing(Some(node)) {
            return None;
        }
        let symbol_map = self.program.symbol_map();

        let mut chain: std::collections::HashMap<u64, (Arc<Node>, Arc<Node>)> =
            std::collections::HashMap::new();
        {
            let mut child = Arc::clone(node);
            let mut ancestor = node.parent();
            while let Some(a) = ancestor {
                chain.insert(a.id(), (Arc::clone(&a), Arc::clone(&child)));
                child = Arc::clone(&a);
                ancestor = a.parent();
            }
        }
        let module_meaning = meaning & SymbolFlags::MODULE_MEMBER;
        let enum_meaning = meaning & SymbolFlags::EnumMember;
        let type_meaning = meaning & SymbolFlags::TYPE;
        let decorator_skip =
            crate::checker::checker_resolve_checker::decorator_skip_container_ids(node);
        if let Some(sym) = self.scope_stack_lookup(
            &chain,
            name,
            meaning,
            module_meaning,
            enum_meaning,
            type_meaning,
            decorator_skip.as_ref(),
        ) {
            return Some(sym);
        }
        if let Some(sym) = self.ancestry_lookup(
            node,
            &chain,
            name,
            meaning,
            module_meaning,
            enum_meaning,
            type_meaning,
            decorator_skip.as_ref(),
        ) {
            return Some(sym);
        }

        if self.function_scope_count > 0
            && name == "arguments"
            && meaning.intersects(SymbolFlags::VARIABLE)
        {
            if let Some(ref sym) = self.arguments_symbol {
                return Some(Arc::clone(sym));
            }
        }

        if let Some(sym) = self.globals.get(name) {
            if sym
                .flags
                .intersects(meaning.union(SymbolFlags::GlobalLookup))
            {
                return Some(Arc::clone(sym));
            }
        }

        // Go NameResolver：JS 文件中 require 调用的 callee 解析失败时回退
        // 到 requireSymbol（类型 any），避免报 cannot-find-name
        if name == "require" && node_parent_is_require_call(node) {
            if let Some(ref sym) = self.require_symbol {
                return Some(Arc::clone(sym));
            }
        }

        None
    }

    // Go nameresolver.go KindExpressionWithTypeArguments 分支：extends 基类表达式
    // 内的名字若落在所在类的成员（类型参数）上，解析失败（TS2562 由调用侧报告）
    pub(crate) fn base_expression_type_parameters_hit(
        &self,
        node: &Arc<Node>,
        name: &str,
        type_meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("base_expression_type_parameters_hit"); 
        let symbol_map = self.program.symbol_map();
        let mut child = Arc::clone(node);
        let mut ancestor = node.parent();
        while let Some(a) = ancestor {
            if a.kind == SyntaxKind::ExpressionWithTypeArguments {
                if let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(d) = &a.data
                    && Arc::ptr_eq(&child, &d.expression)
                    && let Some(clause) = a.parent()
                    && matches!(
                        &clause.data,
                        tsox_frontend::ast::NodeData::HeritageClause(h)
                            if h.token == SyntaxKind::ExtendsKeyword
                    )
                    && let Some(container) = clause.parent()
                    && tsox_frontend::ast::is_class_like(&container)
                    && let Some(container_sym) = symbol_map.symbols.get(&container.id())
                    && let Some(member) = container_sym.members.get(name)
                    && member.flags.intersects(type_meaning)
                {
                    return true;
                }
            }
            if let Some(locals) = symbol_map.locals.get(&a.id())
                && !Self::is_global_source_file(&a)
                && let Some(sym) = locals.get(name)
                && sym.flags.intersects(type_meaning)
            {
                return false;
            }
            if matches!(
                a.kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration
            ) {
                if let Some(sym) = symbol_map.symbols.get(&a.id())
                    && let Some(member) = sym.members.get(name)
                    && member.flags.intersects(type_meaning)
                    && crate::binder::nameresolver_get_local_symbol_for_export_default::is_type_parameter_symbol_declared_in_container(
                        member, &a,
                    )
                {
                    return false;
                }
                if a.kind == SyntaxKind::ClassExpression
                    && a.name().is_some_and(|n| n.text() == name)
                {
                    return false;
                }
            }
            child = Arc::clone(&a);
            ancestor = a.parent();
        }
        false
    }
}
