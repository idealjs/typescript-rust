#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_alias_for_symbol_in_container(
        &mut self,
        container: &Arc<Symbol>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        if let Some(parent) = self.get_parent_of_symbol(symbol) {
            if parent.id() == container.id() {
                return Some(Arc::clone(symbol));
            }
        }

        if let Some(export_equals) = container.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS) {
            if self
                .get_symbol_if_same_reference(export_equals, symbol)
                .is_some()
            {
                return Some(Arc::clone(container));
            }
        }

        let exports = self.get_exports_of_symbol(container);
        if let Some(quick) = exports.get(&symbol.name) {
            if self.get_symbol_if_same_reference(quick, symbol).is_some() {
                return Some(Arc::clone(quick));
            }
        }

        let mut candidates: Vec<Arc<Symbol>> = Vec::new();
        for exported in exports.entries.values() {
            if self
                .get_symbol_if_same_reference(exported, symbol)
                .is_some()
            {
                candidates.push(Arc::clone(exported));
            }
        }
        if !candidates.is_empty() {
            self.sort_symbols(&mut candidates);
            return candidates.into_iter().next();
        }
        None
    }

    pub(crate) fn get_accessible_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        use_only_external_aliasing: bool,
    ) -> Vec<Arc<Symbol>> {
        let ctx = AccessibleSymbolChainContext {
            symbol: Arc::clone(symbol),
            enclosing_declaration: enclosing_declaration.cloned(),
            meaning,
            use_only_external_aliasing,
            visited_symbol_tables_map: RefCell::new(HashMap::new()),
        };
        self.get_accessible_symbol_chain_ex(ctx)
    }

    pub fn get_accessible_symbol_chain_public(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        use_only_external_aliasing: bool,
    ) -> Vec<Arc<Symbol>> {
        self.get_accessible_symbol_chain(
            symbol,
            enclosing_declaration,
            meaning,
            use_only_external_aliasing,
        )
    }

    pub(crate) fn get_accessible_symbol_chain_ex(
        &mut self,
        ctx: AccessibleSymbolChainContext,
    ) -> Vec<Arc<Symbol>> {
        if is_property_or_method_declaration_symbol(&ctx.symbol) {
            return Vec::new();
        }

        let tables = self.collect_symbol_tables_in_scope(ctx.enclosing_declaration.as_ref());
        let first_relevant_location = tables.first().and_then(|t| t.scope_node.clone());

        let link_key = AccessibleChainCacheKey {
            use_only_external_aliasing: ctx.use_only_external_aliasing,
            location: first_relevant_location,
            meaning: ctx.meaning,
        };

        if let Some(links) = self.symbol_container_links.get(&ctx.symbol) {
            if let Some(existing) = links.accessible_chain_cache.get(&link_key) {
                return existing.clone();
            }
        }

        let mut result: Vec<Arc<Symbol>> = Vec::new();

        for info in &tables {
            let res = self.get_accessible_symbol_chain_from_symbol_table(
                &ctx,
                &info.table,
                info.table_id,
                info.ignore_qualification,
                info.is_local_name_lookup,
            );
            if !res.is_empty() {
                result = res;
                break;
            }
        }

        self.symbol_container_links
            .get_or_default(&ctx.symbol)
            .accessible_chain_cache
            .insert(link_key, result.clone());
        result
    }

    pub(crate) fn get_accessible_symbol_chain_from_symbol_table(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        t: &SymbolTable,
        table_id: SymbolTableId,
        ignore_qualification: bool,
        is_local_name_lookup: bool,
    ) -> Vec<Arc<Symbol>> {
        let sym_id = ctx.symbol.id();
        {
            let mut visited_map = ctx.visited_symbol_tables_map.borrow_mut();
            let visited_symbol_tables = visited_map.entry(sym_id).or_default();

            if visited_symbol_tables.contains_key(&table_id) {
                return Vec::new();
            }
            visited_symbol_tables.insert(table_id, ());
        }

        let res =
            self.try_symbol_table(ctx, t, table_id, ignore_qualification, is_local_name_lookup);

        {
            let mut visited_map = ctx.visited_symbol_tables_map.borrow_mut();
            if let Some(visited_symbol_tables) = visited_map.get_mut(&sym_id) {
                visited_symbol_tables.remove(&table_id);
            }
        }
        res
    }

    pub(crate) fn get_symbol_table_aliases(
        &mut self,
        symbols: &SymbolTable,
        table_id: SymbolTableId,
    ) -> Vec<Arc<Symbol>> {
        let kind = table_id & ST_KIND_MASK;

        if kind == ST_KIND_MEMBERS {
            return Vec::new();
        }

        if kind == ST_KIND_GLOBALS || kind == ST_KIND_EXPORTS || kind == ST_KIND_RESOLVED_EXPORTS {
            if let Some(aliases) = self.symbol_table_alias_cache.get(&table_id) {
                return aliases.clone();
            }
        }
        let mut aliases: Vec<Arc<Symbol>> = Vec::new();
        for sym in symbols.entries.values() {
            if sym.flags.intersects(SymbolFlags::Alias) {
                aliases.push(Arc::clone(sym));
            }
        }
        if kind == ST_KIND_GLOBALS || kind == ST_KIND_EXPORTS || kind == ST_KIND_RESOLVED_EXPORTS {
            self.symbol_table_alias_cache
                .insert(table_id, aliases.clone());
        }
        aliases
    }

    pub(crate) fn try_symbol_table(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbols: &SymbolTable,
        table_id: SymbolTableId,
        ignore_qualification: bool,
        is_local_name_lookup: bool,
    ) -> Vec<Arc<Symbol>> {
        let is_globals = table_id == ST_KIND_GLOBALS;

        if let Some(res) = symbols.get(&ctx.symbol.name) {
            if self.is_accessible(ctx, res, None, ignore_qualification) {
                return vec![Arc::clone(&ctx.symbol)];
            }

            if let Some(ref export_sym) = res.export_symbol {
                let merged = self.get_merged_symbol(export_sym);
                if self.is_accessible(ctx, &merged, None, ignore_qualification) {
                    return vec![Arc::clone(&ctx.symbol)];
                }
            }
        }

        let mut candidate_chains: Vec<Vec<Arc<Symbol>>> = Vec::new();

        let aliases = self.get_symbol_table_aliases(symbols, table_id);
        for symbol_from_symbol_table in &aliases {
            let enclosing_is_external_module = ctx
                .enclosing_declaration
                .as_ref()
                .map(|n| false)
                .unwrap_or(false);

            if symbol_from_symbol_table.name != INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                && symbol_from_symbol_table.name != INTERNAL_SYMBOL_NAME_DEFAULT
                && !(is_umd_export_symbol(symbol_from_symbol_table)
                    && ctx.enclosing_declaration.is_some()
                    && enclosing_is_external_module)
                && (!ctx.use_only_external_aliasing
                    || symbol_from_symbol_table
                        .declarations
                        .iter()
                        .any(|d| d.kind == SyntaxKind::ExternalModuleReference))
                && (!is_local_name_lookup
                    || !symbol_from_symbol_table
                        .declarations
                        .iter()
                        .any(is_namespace_reexport_declaration))
                && (ignore_qualification
                    || !symbol_from_symbol_table
                        .declarations
                        .iter()
                        .any(|d| d.kind == SyntaxKind::ExportSpecifier))
            {
                let resolved_imported_symbol = self.resolve_alias(symbol_from_symbol_table);
                let candidate = self.get_candidate_list_for_symbol(
                    ctx,
                    symbol_from_symbol_table,
                    &resolved_imported_symbol,
                    ignore_qualification,
                );
                if !candidate.is_empty() {
                    candidate_chains.push(candidate);
                }
            }
        }

        if !candidate_chains.is_empty() {
            candidate_chains.sort_by(|a, b| self.compare_symbol_chains(a, b));
            return candidate_chains.into_iter().next().unwrap_or_default();
        }

        if is_globals {
            if let Some(global_this) = self.global_this_symbol.clone() {
                return self.get_candidate_list_for_symbol(
                    ctx,
                    &global_this,
                    &global_this,
                    ignore_qualification,
                );
            }
        }
        Vec::new()
    }

    pub(crate) fn compare_symbol_chains(
        &self,
        a: &[Arc<Symbol>],
        b: &[Arc<Symbol>],
    ) -> std::cmp::Ordering {
        let chain_len = a.len().cmp(&b.len());
        if chain_len != std::cmp::Ordering::Equal {
            return chain_len;
        }

        for idx in 0..a.len() {
            let cmp = self.compare_symbols(&a[idx], &b[idx]);
            let comparison = match cmp {
                x if x < 0 => std::cmp::Ordering::Less,
                0 => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Greater,
            };
            if comparison != std::cmp::Ordering::Equal {
                return comparison;
            }
        }
        std::cmp::Ordering::Equal
    }
}
