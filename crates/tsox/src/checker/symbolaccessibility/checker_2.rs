#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_with_alternative_containers(
        &mut self,
        container: &Arc<Symbol>,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> {
        let additional_containers: Vec<Arc<Symbol>> = container
            .declarations
            .iter()
            .filter_map(|d| {
                self.get_file_symbol_if_file_symbol_export_equals_container(d, container)
            })
            .collect();

        let reexport_containers = if enclosing_declaration.is_some() {
            self.get_alternative_containing_modules(symbol, enclosing_declaration)
        } else {
            Vec::new()
        };

        let object_literal_container =
            self.get_variable_declaration_of_object_literal(container, meaning);
        let left_meaning = get_qualified_left_meaning(meaning);

        if enclosing_declaration.is_some()
            && container.flags.intersects(left_meaning)
            && !self
                .get_accessible_symbol_chain(
                    container,
                    enclosing_declaration,
                    SymbolFlags::NAMESPACE,
                    false,
                )
                .is_empty()
        {
            let mut res = vec![Arc::clone(container)];
            res.extend(additional_containers.iter().cloned());
            res.extend(reexport_containers.iter().cloned());
            if let Some(olc) = object_literal_container {
                res.push(olc);
            }
            return res;
        }

        let mut variable_matches: Vec<Arc<Symbol>> = Vec::new();
        if meaning == SymbolFlags::VALUE
            && !container.flags.intersects(left_meaning)
            && container.flags.intersects(SymbolFlags::TYPE)
            && self
                .get_declared_type_of_symbol(container)
                .flags
                .intersects(TypeFlags::Object)
        {
            let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
            for info in &tables {
                let mut found = false;
                for s in info.table.entries.values() {
                    if s.flags.intersects(left_meaning)
                        && Arc::ptr_eq(
                            &self.get_type_of_symbol(s),
                            &self.get_declared_type_of_symbol(container),
                        )
                    {
                        variable_matches.push(Arc::clone(s));
                        found = true;
                    }
                }
                if found {
                    break;
                }
            }
            self.sort_symbols(&mut variable_matches);
        }

        let mut res: Vec<Arc<Symbol>> = Vec::new();
        res.extend(variable_matches);
        res.extend(additional_containers);
        res.push(Arc::clone(container));
        if let Some(olc) = object_literal_container {
            res.push(olc);
        }
        res.extend(reexport_containers);
        res
    }

    pub(crate) fn get_alternative_containing_modules(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> Vec<Arc<Symbol>> {
        let enclosing_declaration = match enclosing_declaration {
            Some(enc) => enc,
            None => return Vec::new(),
        };

        let containing_file = self.get_source_file_of_node(enclosing_declaration);
        let id = containing_file.as_ref().map(|f| f.id()).unwrap_or(0);

        if let Some(links) = self.symbol_container_links.get(symbol) {
            if let Some(existing) = links.extended_containers_by_file.get(&id) {
                return existing.clone();
            }
        }

        let mut results: Vec<Arc<Symbol>> = Vec::new();

        if let Some(links) = self.symbol_container_links.get(symbol) {
            if let Some(ref extended) = links.extended_containers {
                return extended.clone();
            }
        }

        let other_files: Vec<Arc<SourceFile>> = self.files.clone();
        for file in &other_files {
            let sym = self.get_symbol_of_declaration(&file.node);
            if let Some(ref sym) = sym {
                let ref_sym = self.get_alias_for_symbol_in_container(sym, symbol);
                if ref_sym.is_some() {
                    results.push(Arc::clone(sym));
                }
            }
        }

        self.symbol_container_links
            .get_or_default(symbol)
            .extended_containers = Some(results.clone());
        self.symbol_container_links
            .get_or_default(symbol)
            .extended_containers_by_file
            .insert(id, results.clone());
        results
    }

    pub(crate) fn get_variable_declaration_of_object_literal(
        &self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        if !meaning.intersects(SymbolFlags::VALUE) {
            return None;
        }
        if symbol.declarations.is_empty() {
            return None;
        }
        let first_decl = &symbol.declarations[0];
        let parent = first_decl.parent.as_ref()?;

        None
    }

    pub(crate) fn get_external_module_container(
        &self,
        declaration: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        if has_external_module_symbol(declaration) {
            return self.get_symbol_of_declaration(declaration);
        }

        let mut node = declaration.parent.as_ref();
        while let Some(n) = node {
            if has_external_module_symbol(n) {
                return self.get_symbol_of_declaration(n);
            }
            node = n.parent.as_ref();
        }
        None
    }

    pub(crate) fn get_external_module_container_of_symbol(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        for d in &symbol.declarations {
            if let Some(sym) = self.get_external_module_container(d) {
                return Some(sym);
            }
        }
        None
    }

    pub(crate) fn get_file_symbol_if_file_symbol_export_equals_container(
        &self,
        d: &Arc<Node>,
        container: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let file_symbol = self.get_external_module_container(d)?;
        let exported = file_symbol
            .exports
            .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)?;
        if self
            .get_symbol_if_same_reference(exported, container)
            .is_some()
        {
            Some(file_symbol)
        } else {
            None
        }
    }

    pub(crate) fn get_containers_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> {
        let container = self.get_parent_of_symbol(symbol);

        if let Some(ref container) = container {
            if !symbol.flags.intersects(SymbolFlags::TypeParameter) {
                return self.get_with_alternative_containers(
                    container,
                    symbol,
                    enclosing_declaration,
                    meaning,
                );
            }
        }

        let mut candidates: Vec<Arc<Symbol>> = Vec::new();
        for d in &symbol.declarations {
            if let Some(ref parent) = d.parent {
                if has_non_global_augmentation_external_module_symbol(parent) {
                    if let Some(sym) = self.get_symbol_of_declaration(parent) {
                        if !candidates.iter().any(|c| c.id() == sym.id()) {
                            candidates.push(sym);
                        }
                    }
                    continue;
                }
            }
        }

        if candidates.is_empty() {
            return Vec::new();
        }

        let mut best_containers: Vec<Arc<Symbol>> = Vec::new();
        let mut alternative_containers: Vec<Arc<Symbol>> = Vec::new();
        for container in &candidates {
            if self
                .get_alias_for_symbol_in_container(container, symbol)
                .is_none()
            {
                continue;
            }
            let all_alts = self.get_with_alternative_containers(
                container,
                symbol,
                enclosing_declaration,
                meaning,
            );
            if all_alts.is_empty() {
                continue;
            }
            best_containers.push(Arc::clone(&all_alts[0]));
            if all_alts.len() > 1 {
                alternative_containers.extend(all_alts[1..].iter().cloned());
            }
        }
        best_containers.extend(alternative_containers);
        best_containers
    }
}
