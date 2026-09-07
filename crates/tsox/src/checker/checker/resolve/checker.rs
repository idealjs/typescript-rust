#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let result = self.resolve_identifier_with_meaning_inner(node, meaning);

        if let Some(sym) = &result {
            let mut bits = meaning;
            if meaning.intersects(SymbolFlags::VALUE) {
                bits |= SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable;
            }
            self.record_symbol_reference(sym, bits);
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

    pub(crate) fn resolve_identifier_with_meaning_inner(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning)
                    {
                        return self.follow_alias(sym);
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
                            return self.follow_alias(sym);
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
                                .any(|d| d.kind == SyntaxKind::ExportSpecifier);
                        if !is_export_specifier {
                            return self.follow_alias(sym);
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
                                    return self.follow_alias(sym);
                                }
                            }
                            if let Some(sym) = self.ambient_namespace_local(merged, name) {
                                if sym.flags.intersects(meaning)
                                    || self.alias_chain_hits_meaning(&sym, meaning)
                                {
                                    return self.follow_alias(&sym);
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
                            return self.follow_alias(sym);
                        }
                    }
                }

                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning & SymbolFlags::TYPE)
                        || self.alias_chain_hits_meaning(&sym, meaning)
                    {
                        return self.follow_alias(sym);
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
            let mut ancestor = node.parent.as_ref();
            while let Some(a) = ancestor {
                if !ANCESTRY_CONTAINERS.contains(&a.kind) {
                    ancestor = a.parent.as_ref();
                    continue;
                }
                let aid = a.id();
                if let Some(locals) = symbol_map.locals.get(&aid) {
                    if let Some(sym) = locals.get(name)
                        && (sym.flags.intersects(meaning)
                            || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return self.follow_alias(sym);
                    }
                }
                if let Some(a_sym) = symbol_map.symbols.get(&aid) {
                    if !a_sym.flags.intersects(SymbolFlags::Class) {
                        if let Some(sym) = a_sym.members.get(name)
                            && (sym.flags.intersects(meaning)
                                || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return self.follow_alias(sym);
                        }
                        if a_sym
                            .flags
                            .intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                            && let Some(sym) = a_sym.exports.get(name)
                            && (sym.flags.intersects(meaning)
                                || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return self.follow_alias(sym);
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
                                        return self.follow_alias(sym);
                                    }
                                    if let Some(sym) = self.ambient_namespace_local(merged, name)
                                        && (sym.flags.intersects(meaning)
                                            || self.alias_chain_hits_meaning(&sym, meaning))
                                    {
                                        return self.follow_alias(&sym);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(sym) = a_sym.members.get(name)
                        && (sym.flags.intersects(meaning & SymbolFlags::TYPE)
                            || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return self.follow_alias(sym);
                    }
                }
                ancestor = a.parent.as_ref();
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

        None
    }

    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if symbol.flags.intersects(SymbolFlags::Alias) {
            self.record_symbol_reference(
                symbol,
                SymbolFlags::VALUE
                    | SymbolFlags::TYPE
                    | SymbolFlags::NAMESPACE
                    | SymbolFlags::FunctionScopedVariable
                    | SymbolFlags::BlockScopedVariable,
            );
        }
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
