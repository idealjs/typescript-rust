#![allow(unused_imports)]

use crate::checker::checker_resolve::*;

impl Checker {
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        self.resolve_identifier_scope_symbol(node, meaning)
            .and_then(|s| self.follow_alias(&s))
    }

    pub fn resolve_identifier_use(
        &self,
        node: &Arc<Node>,
        record_meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let scope = self.resolve_identifier_scope_symbol(node, SymbolFlags::all());
        let result = scope.as_ref().and_then(|s| self.follow_alias(s));
        if let Some(sym) = &scope
            && access_kind(node) != AccessKind::Write
        {
            self.record_symbol_reference(sym, record_meaning);
        }
        result
    }

    pub(crate) fn record_symbol_reference(&self, symbol: &Arc<Symbol>, bits: SymbolFlags) {
        self.symbol_reference_kinds
            .entry(symbol.id())
            .and_modify(|f| *f |= bits)
            .or_insert(bits);
    }

    pub(crate) fn alias_chain_hits_meaning(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool {
        if !sym.flags.intersects(SymbolFlags::Alias) {
            return false;
        }
        match self.follow_alias(sym) {
            Some(target) if !Arc::ptr_eq(&target, sym) => target.flags.intersects(meaning),
            _ => true,
        }
    }

    pub(crate) fn resolve_identifier_scope_symbol(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning)
                    {
                        return Some(Arc::clone(sym));
                    }
                }
            }

            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if !container_sym.flags.intersects(SymbolFlags::Class)
                    || container_sym.flags.intersects(SymbolFlags::Function)
                {
                    if let Some(sym) = container_sym.members.get(name) {
                        if sym.flags.intersects(meaning)
                            || self.alias_chain_hits_meaning(&sym, meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::MODULE)
                    && !container_sym.flags.intersects(SymbolFlags::Class)
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
                        if !is_export_specifier {
                            return Some(Arc::clone(sym));
                        }
                    }

                    // Go resolver：declare module "foo" 增强块内的名字可见性
                    // 延伸到被增强模块的 exports
                    if let Some(sym) = self.augmentation_target_member(container_sym, name) {
                        if sym.flags.intersects(meaning)
                            || self.alias_chain_hits_meaning(&sym, meaning)
                        {
                            return Some(sym);
                        }
                    }

                    if let Some(merged) = self.globals.get(container_sym.name.as_str()) {
                        if !Arc::ptr_eq(merged, container_sym)
                            && merged.flags.intersects(SymbolFlags::MODULE)
                        {
                            if let Some(sym) = merged.exports.get(name) {
                                if sym.flags.intersects(meaning)
                                    || self.alias_chain_hits_meaning(&sym, meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
                            }
                            if let Some(sym) = self.ambient_namespace_local(merged, name) {
                                if sym.flags.intersects(meaning)
                                    || self.alias_chain_hits_meaning(&sym, meaning)
                                {
                                    return Some(sym);
                                }
                            }
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning)
                            || self.alias_chain_hits_meaning(&sym, meaning)
                        {
                            return Some(Arc::clone(sym));
                        }
                    }
                }

                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning & SymbolFlags::TYPE)
                        || self.alias_chain_hits_meaning(&sym, meaning)
                    {
                        return Some(Arc::clone(sym));
                    }
                }
            }
        }

        {
            const ANCESTRY_CONTAINERS: &[SyntaxKind] = &[
                SyntaxKind::SourceFile,
                SyntaxKind::ModuleDeclaration,
                SyntaxKind::Block,
                SyntaxKind::CatchClause,
                SyntaxKind::ForStatement,
                SyntaxKind::ForInStatement,
                SyntaxKind::ForOfStatement,
                SyntaxKind::FunctionDeclaration,
                SyntaxKind::FunctionExpression,
                SyntaxKind::ArrowFunction,
                SyntaxKind::MethodDeclaration,
                SyntaxKind::MethodSignature,
                SyntaxKind::CallSignature,
                SyntaxKind::ConstructSignature,
                SyntaxKind::FunctionType,
                SyntaxKind::ConstructorType,
                SyntaxKind::MappedType,
                SyntaxKind::Constructor,
                SyntaxKind::GetAccessor,
                SyntaxKind::SetAccessor,
                SyntaxKind::InterfaceDeclaration,
                SyntaxKind::ClassDeclaration,
                SyntaxKind::ClassExpression,
                SyntaxKind::TypeAliasDeclaration,
                SyntaxKind::EnumDeclaration,
            ];
            let mut ancestor = node.parent();
            while let Some(a) = ancestor {
                if !ANCESTRY_CONTAINERS.contains(&a.kind) {
                    ancestor = a.parent();
                    continue;
                }
                let aid = a.id();
                if let Some(locals) = symbol_map.locals.get(&aid) {
                    if let Some(sym) = locals.get(name)
                        && (sym.flags.intersects(meaning)
                            || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return Some(Arc::clone(sym));
                    }
                }
                if let Some(a_sym) = symbol_map.symbols.get(&aid) {
                    if !a_sym.flags.intersects(SymbolFlags::Class) {
                        if let Some(sym) = a_sym.members.get(name)
                            && (sym.flags.intersects(meaning)
                                || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return Some(Arc::clone(sym));
                        }
                        if a_sym
                            .flags
                            .intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                            && let Some(sym) = a_sym.exports.get(name)
                        {
                            // Go resolver：模块导出含纯 alias 的 export specifier
                            //（export * as ns 亦同）不视为作用域内名字
                            let is_export_specifier = sym.flags == SymbolFlags::Alias
                                && sym
                                    .declarations
                                    .iter()
                                    .any(|d| {
                                        d.kind == SyntaxKind::ExportSpecifier
                                            || d.kind == SyntaxKind::NamespaceExport
                                    });
                            if !is_export_specifier
                                && (sym.flags.intersects(meaning)
                                    || self.alias_chain_hits_meaning(&sym, meaning))
                            {
                                return Some(Arc::clone(sym));
                            }
                        }

                        if a_sym.flags.intersects(SymbolFlags::MODULE) {
                            if let Some(merged) = self.globals.get(a_sym.name.as_str()) {
                                if !Arc::ptr_eq(merged, a_sym)
                                    && merged.flags.intersects(SymbolFlags::MODULE)
                                {
                                    if let Some(sym) = merged.exports.get(name)
                                        && (sym.flags.intersects(meaning)
                                            || self.alias_chain_hits_meaning(&sym, meaning))
                                    {
                                        return Some(Arc::clone(sym));
                                    }
                                    if let Some(sym) = self.ambient_namespace_local(merged, name)
                                        && (sym.flags.intersects(meaning)
                                            || self.alias_chain_hits_meaning(&sym, meaning))
                                    {
                                        return Some(sym);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(sym) = a_sym.members.get(name)
                        && (sym.flags.intersects(meaning & SymbolFlags::TYPE)
                            || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return Some(Arc::clone(sym));
                    }
                }
                ancestor = a.parent();
            }
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

    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(Arc::clone(symbol));
        }

        let is_pure_alias = symbol.flags == SymbolFlags::Alias
            || (symbol.flags.intersects(SymbolFlags::Alias)
                && symbol.flags.intersects(SymbolFlags::Assignment));
        if !is_pure_alias {
            return Some(Arc::clone(symbol));
        }

        let mut current = Arc::clone(symbol);
        let mut seen: Vec<*const Symbol> = vec![Arc::as_ptr(symbol)];
        loop {
            if let Some(ref target) = current.export_symbol {
                let target_ptr = Arc::as_ptr(target);
                if seen.contains(&target_ptr) {
                    return Some(Arc::clone(&current));
                }
                let is_pure = target.flags == SymbolFlags::Alias
                    || (target.flags.intersects(SymbolFlags::Alias)
                        && target.flags.intersects(SymbolFlags::Assignment));
                if is_pure {
                    seen.push(target_ptr);
                    current = Arc::clone(target);
                    continue;
                }
                return Some(Arc::clone(target));
            } else {
                return Some(Arc::clone(&current));
            }
        }
    }
}

impl Checker {
    /// Go getVisibleSymbolInAugmentationScope：declare module "foo"（augmentation）
    /// 块内的裸名查找延伸到被增强模块的 exports
    pub(crate) fn augmentation_target_member(
        &self,
        container_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let decl = container_sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ModuleDeclaration)?;
        let tsox_frontend::ast::NodeData::ModuleDeclaration(md) = &decl.data else {
            return None;
        };
        if md.name.kind != SyntaxKind::StringLiteral {
            return None;
        }
        let spec = md.name.text().trim_matches(['"', '\'']).to_string();
        let resolved = self.resolve_module_file_symbol(&spec)?;
        let target = if Arc::ptr_eq(&resolved, container_sym) {
            return None;
        } else {
            resolved
        };
        target
            .exports
            .get(name)
            .cloned()
            .or_else(|| target.members.get(name).cloned())
    }
}

fn node_parent_is_require_call(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if let tsox_frontend::ast::NodeData::CallExpression(call) = &parent.data
        && call.expression.kind == SyntaxKind::Identifier
        && call.expression.text() == "require"
        && call.arguments.len() == 1
    {} else {
        return false;
    }
    let mut ancestor = Some(Arc::clone(node));
    while let Some(a) = ancestor {
        if a.kind == SyntaxKind::SourceFile {
            return a
                .flags
                .contains(tsox_frontend::ast::NodeFlags::JavaScriptFile);
        }
        ancestor = a.parent();
    }
    false
}
