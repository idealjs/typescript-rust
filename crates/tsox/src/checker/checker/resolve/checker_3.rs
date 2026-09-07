#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn merge_symbol_table(
        &mut self,
        target: &mut SymbolTable,
        source: &SymbolTable,
        unidirectional: bool,
        merged_parent: Option<u64>,
    ) {
        let entries: Vec<(String, Arc<Symbol>)> = source
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        for (name, source_symbol) in entries {
            if let Some(target_symbol) = target.entries.get_mut(&name) {
                let merged = self.merge_symbol(target_symbol, &source_symbol, unidirectional);
                let is_transient = merged.flags.intersects(SymbolFlags::Transient);
                *target_symbol = merged;
                if let Some(_parent_id) = merged_parent {
                    if is_transient {}
                }
            } else {
                let merged = self.get_merged_symbol(&source_symbol);
                target.insert(name, merged);
            }
        }
    }

    pub fn merge_symbol(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
        unidirectional: bool,
    ) -> Arc<Symbol> {
        let excluded = get_excluded_symbol_flags(source.flags);
        if target.flags.intersects(excluded) == false
            || (source.flags | target.flags).intersects(SymbolFlags::Assignment)
        {
            if Arc::ptr_eq(target, source) {
                return Arc::clone(target);
            }

            let effective_target = if !target.flags.intersects(SymbolFlags::Transient) {
                let resolved_target = self.resolve_symbol(target);
                if resolved_target
                    .flags
                    .intersects(get_excluded_symbol_flags(source.flags))
                    == false
                    || (source.flags | resolved_target.flags).intersects(SymbolFlags::Assignment)
                {
                    if let Some(cloned) = self.clone_symbol(&resolved_target) {
                        cloned
                    } else {
                        return Arc::clone(source);
                    }
                } else {
                    return Arc::clone(source);
                }
            } else {
                Arc::clone(target)
            };

            let mut source_flags = source.flags;
            if !effective_target
                .flags
                .intersects(SymbolFlags::ConstEnumOnlyModule)
            {
                source_flags.remove(SymbolFlags::ConstEnumOnlyModule);
            }
            let merged_flags = effective_target.flags | source_flags;

            let mut merged = Symbol::new(merged_flags, &effective_target.name);

            merged.value_declaration = source
                .value_declaration
                .clone()
                .or_else(|| effective_target.value_declaration.clone());

            merged.declarations = effective_target.declarations.clone();
            merged
                .declarations
                .extend(source.declarations.iter().cloned());

            merged.parent = effective_target.parent.clone();

            merged.members = SymbolTable {
                entries: effective_target.members.entries.clone(),
            };
            merged.exports = SymbolTable {
                entries: effective_target.exports.entries.clone(),
            };

            let result = Arc::new(merged);

            let mut result_mut = Symbol::new(result.flags, &result.name);
            result_mut.value_declaration = result.value_declaration.clone();
            result_mut.declarations = result.declarations.clone();
            result_mut.parent = result.parent.clone();
            result_mut.members = SymbolTable {
                entries: result.members.entries.clone(),
            };
            result_mut.exports = SymbolTable {
                entries: result.exports.entries.clone(),
            };

            let source_members: Vec<(String, Arc<Symbol>)> = source
                .members
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_members {
                if let Some(target_sym) = result_mut.members.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.members.insert(name, merged);
                }
            }

            let source_exports: Vec<(String, Arc<Symbol>)> = source
                .exports
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_exports {
                if let Some(target_sym) = result_mut.exports.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.exports.insert(name, merged);
                }
            }

            let final_result = Arc::new(result_mut);

            if !unidirectional {
                self.record_merged_symbol(&final_result, source);
            }

            final_result
        } else {
            self.report_merge_symbol_error(target, source);
            Arc::clone(target)
        }
    }

    pub(crate) fn report_merge_symbol_error(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        let is_either_enum =
            target.flags.contains(SymbolFlags::ENUM) || source.flags.contains(SymbolFlags::ENUM);
        let is_either_block_scoped = target.flags.intersects(SymbolFlags::BlockScopedVariable)
            || source.flags.intersects(SymbolFlags::BlockScopedVariable);
        let message = if is_either_enum {
            crate::diagnostics::messages_generated::
                ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS
        } else if is_either_block_scoped {
            crate::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0
        };
        let name = source.name.clone();
        let mut locs: Vec<crate::core::text::TextRange> = Vec::new();
        for sym in [target, source] {
            for d in &sym.declarations {
                let name_node = crate::ast::utilities::get_name_of_declaration(d)
                    .unwrap_or_else(|| Arc::clone(d));
                locs.push(name_node.loc);
            }
        }
        for loc in locs {
            if self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc == loc && d.code == message.code)
            {
                continue;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                vec![name.clone()],
            ));
        }
    }

    pub fn record_merged_symbol(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        self.merged_symbols.insert(source.id(), target.id());
    }

    pub fn clone_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let mut cloned = Symbol::new(symbol.flags | SymbolFlags::Transient, &symbol.name);
        cloned.declarations = symbol.declarations.clone();
        cloned.parent = symbol.parent.clone();
        cloned.value_declaration = symbol.value_declaration.clone();
        cloned.members = SymbolTable {
            entries: symbol.members.entries.clone(),
        };
        cloned.exports = SymbolTable {
            entries: symbol.exports.entries.clone(),
        };
        let result = Arc::new(cloned);
        Some(result)
    }

    pub fn resolve_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(result) = self.follow_alias(symbol) {
            result
        } else {
            Arc::clone(symbol)
        }
    }

    pub(crate) fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        if node.kind == crate::ast::SyntaxKind::Identifier {
            let mut current = node.parent.as_ref();
            while let Some(parent) = current {
                if let Some(sym) = self.program.symbol_map().symbol_of(parent) {
                    return Some(Arc::clone(sym));
                }
                current = parent.parent.as_ref();
            }
        }

        if node.kind == crate::ast::SyntaxKind::PropertyAccessExpression {
            if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                if let Some(links) = self.type_node_links.get(&data.expression) {
                    if let Some(ref t) = links.resolved_type {
                        return self.get_property_of_type_cached(t, data.name.text());
                    }
                }
            }
        }

        None
    }
}
