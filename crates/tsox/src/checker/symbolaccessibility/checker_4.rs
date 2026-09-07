#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_candidate_list_for_symbol(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        resolved_imported_symbol: &Arc<Symbol>,
        ignore_qualification: bool,
    ) -> Vec<Arc<Symbol>> {
        if self.is_accessible(
            ctx,
            symbol_from_symbol_table,
            Some(resolved_imported_symbol),
            ignore_qualification,
        ) {
            return vec![Arc::clone(symbol_from_symbol_table)];
        }

        let candidate_table = self.get_exports_of_symbol(resolved_imported_symbol);
        let candidate_table_id = symbol_table_id_from_resolved_exports(resolved_imported_symbol);
        let accessible_symbols_from_exports = self.get_accessible_symbol_chain_from_symbol_table(
            ctx,
            &candidate_table,
            candidate_table_id,
            true,
            false,
        );
        if accessible_symbols_from_exports.is_empty() {
            return Vec::new();
        }
        if !self.can_qualify_symbol(
            ctx,
            symbol_from_symbol_table,
            get_qualified_left_meaning(ctx.meaning),
        ) {
            return Vec::new();
        }
        let mut result = vec![Arc::clone(symbol_from_symbol_table)];
        result.extend(accessible_symbols_from_exports);
        result
    }

    pub(crate) fn is_accessible(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        resolved_alias_symbol: Option<&Arc<Symbol>>,
        ignore_qualification: bool,
    ) -> bool {
        let mut like_symbols = false;
        if let Some(ref resolved) = resolved_alias_symbol {
            if ctx.symbol.id() == resolved.id() {
                like_symbols = true;
            }
        }
        if ctx.symbol.id() == symbol_from_symbol_table.id() {
            like_symbols = true;
        }
        let symbol = self.get_merged_symbol(&ctx.symbol);
        if let Some(ref resolved) = resolved_alias_symbol {
            let merged_resolved = self.get_merged_symbol(resolved);
            if symbol.id() == merged_resolved.id() {
                like_symbols = true;
            }
        }
        let merged_from_table = self.get_merged_symbol(symbol_from_symbol_table);
        if symbol.id() == merged_from_table.id() {
            like_symbols = true;
        }
        if !like_symbols {
            return false;
        }

        !symbol_from_symbol_table
            .declarations
            .iter()
            .any(has_non_global_augmentation_external_module_symbol)
            && (ignore_qualification
                || self.can_qualify_symbol(
                    ctx,
                    &self.get_merged_symbol(symbol_from_symbol_table),
                    ctx.meaning,
                ))
    }

    pub(crate) fn can_qualify_symbol(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> bool {
        if !self.needs_qualification(
            symbol_from_symbol_table,
            ctx.enclosing_declaration.as_ref(),
            meaning,
        ) {
            return true;
        }

        if let Some(ref parent) = symbol_from_symbol_table.parent {
            let parent_ctx = AccessibleSymbolChainContext {
                symbol: Arc::clone(parent),
                enclosing_declaration: ctx.enclosing_declaration.clone(),
                meaning: get_qualified_left_meaning(meaning),
                use_only_external_aliasing: ctx.use_only_external_aliasing,
                visited_symbol_tables_map: RefCell::new(
                    ctx.visited_symbol_tables_map.borrow().clone(),
                ),
            };
            !self.get_accessible_symbol_chain_ex(parent_ctx).is_empty()
        } else {
            false
        }
    }

    pub(crate) fn needs_qualification(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool {
        let mut qualify = false;
        let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
        for info in &tables {
            let res = match info.table.get(&symbol.name) {
                Some(r) => r,
                None => continue,
            };
            let mut symbol_from_symbol_table = self.get_merged_symbol(res);

            if symbol_from_symbol_table.id() == symbol.id() {
                return false;
            }

            let should_resolve_alias = symbol_from_symbol_table
                .flags
                .intersects(SymbolFlags::Alias)
                && symbol_from_symbol_table
                    .declarations
                    .iter()
                    .all(|d| d.kind != SyntaxKind::ExportSpecifier);
            if should_resolve_alias {
                symbol_from_symbol_table = self.resolve_alias(&symbol_from_symbol_table);
            }
            let mut flags = symbol_from_symbol_table.flags;
            if should_resolve_alias {
                flags = self.get_symbol_flags(&symbol_from_symbol_table);
            }
            if flags.intersects(meaning) {
                qualify = true;
                break;
            }
        }
        qualify
    }

    pub(crate) fn collect_symbol_tables_in_scope(
        &mut self,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> Vec<SymbolTableInScope> {
        let mut result: Vec<SymbolTableInScope> = Vec::new();
        let mut location = enclosing_declaration.cloned();

        while let Some(loc) = location {
            if can_have_locals(loc.kind) {
                if let Some(locals) = self.program.symbol_map().locals_of(&loc) {
                    let is_global_source_file = loc.kind == SyntaxKind::SourceFile
                        && !Checker::is_external_or_common_js_module(&loc);
                    if !is_global_source_file && !locals.is_empty() {
                        result.push(SymbolTableInScope {
                            table: locals.clone(),
                            table_id: symbol_table_id_from_locals(&loc),
                            ignore_qualification: false,
                            is_local_name_lookup: true,
                            scope_node: Some(Arc::clone(&loc)),
                        });
                    }
                }
            }

            match loc.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                    if loc.kind == SyntaxKind::SourceFile
                        && !Checker::is_external_or_common_js_module(&loc)
                    {
                    } else {
                        if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                            if !sym.exports.is_empty() {
                                result.push(SymbolTableInScope {
                                    table: sym.exports.clone(),
                                    table_id: symbol_table_id_from_exports(&sym),
                                    ignore_qualification: false,
                                    is_local_name_lookup: true,
                                    scope_node: Some(Arc::clone(&loc)),
                                });
                            }
                        }
                    }
                }
                SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression
                | SyntaxKind::InterfaceDeclaration => {
                    if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                        let mut table = SymbolTable::new();
                        for (key, member_symbol) in sym.members.entries.iter() {
                            if member_symbol
                                .flags
                                .intersects(SymbolFlags::TYPE.difference(SymbolFlags::Assignment))
                            {
                                table.insert(key.clone(), Arc::clone(member_symbol));
                            }
                        }
                        if !table.is_empty() {
                            result.push(SymbolTableInScope {
                                table,
                                table_id: symbol_table_id_from_members(&sym),
                                ignore_qualification: false,
                                is_local_name_lookup: false,
                                scope_node: Some(Arc::clone(&loc)),
                            });
                        }

                        if loc.kind == SyntaxKind::ClassExpression {
                            if let Some(name_table) = self.get_class_expression_name_table(&loc) {
                                if !name_table.is_empty() {
                                    result.push(SymbolTableInScope {
                                        table: name_table,
                                        table_id: symbol_table_id_from_locals(&loc),
                                        ignore_qualification: false,
                                        is_local_name_lookup: true,
                                        scope_node: Some(Arc::clone(&loc)),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            location = loc.parent.clone();
        }

        if !self.globals.is_empty() {
            result.push(SymbolTableInScope {
                table: self.globals.clone(),
                table_id: symbol_table_id_from_globals(),
                ignore_qualification: false,
                is_local_name_lookup: true,
                scope_node: None,
            });
        }

        result
    }

    pub(crate) fn get_class_expression_name_table(
        &mut self,
        location: &Arc<Node>,
    ) -> Option<SymbolTable> {
        let node_id = location.id();

        if let Some(table) = self.class_expression_name_tables.get(&node_id) {
            return Some(table.clone());
        }

        let class_symbol = self.get_symbol_of_declaration(location)?;

        let name_text = class_symbol.name.clone();
        if name_text.is_empty() {
            return None;
        }
        let mut table = SymbolTable::new();
        table.insert(name_text, class_symbol);
        self.class_expression_name_tables
            .insert(node_id, table.clone());
        Some(table)
    }

    pub(crate) fn has_visible_declarations_with_aliases(
        &mut self,
        symbol: &Arc<Symbol>,
        _should_compute_aliases_to_make_visible: bool,
    ) -> Option<SymbolAccessibilityResult> {
        self.has_visible_declarations(symbol)
    }
}
