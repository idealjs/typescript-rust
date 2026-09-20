#![allow(unused_imports)]

use crate::checker::checker::*;

enum Provider {
    Local,
    Star(String, Option<Arc<Symbol>>),
}

impl Checker {
    pub(crate) fn check_export_star_ambiguity(&mut self, file: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::SourceFile(sf) = &file.data else {
            return;
        };
        let Some(file_sym) = self.program.symbol_map().symbol_of(file).cloned() else {
            return;
        };

        let mut provided: std::collections::HashMap<String, Provider> =
            std::collections::HashMap::new();
        let mut star_queue: Vec<(Arc<Node>, String)> = Vec::new();

        for stmt in sf.statements.iter() {
            let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data else {
                if stmt
                    .has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export)
                    && let Some(name) = stmt.name()
                {
                    provided
                        .entry(name.text().to_string())
                        .or_insert(Provider::Local);
                }
                continue;
            };
            if let Some(clause) = &d.export_clause
                && let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) = &el.data {
                        provided
                            .entry(spec.name.text().to_string())
                            .or_insert(Provider::Local);
                    }
                }
                continue;
            }
            if let Some(spec) = &d.module_specifier {
                let text = spec.text().trim_matches(['"', '\'', '`']).to_string();
                star_queue.push((Arc::clone(stmt), text));
            }
        }

        for (stmt, spec) in star_queue {
            let Some(target) = self
                .resolve_module_spec_from(&file_sym, &spec)
                .or_else(|| self.resolve_module_file_symbol(&spec))
            else {
                continue;
            };
            let entries: Vec<(String, Arc<Symbol>)> = self
                .get_exports_of_module_table(&target)
                .entries
                .iter()
                .filter(|(name, _)| {
                    !name.starts_with(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX)
                        && name.as_str() != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                })
                .map(|(name, symbol)| (name.clone(), Arc::clone(symbol)))
                .collect();
            for (name, symbol) in entries {
                let resolved = Some(self.resolve_alias_base(Arc::clone(&symbol)));
                match provided.get(&name) {
                    None => {
                        provided.insert(name, Provider::Star(spec.clone(), resolved));
                    }
                    Some(Provider::Local) => {}
                    Some(Provider::Star(first_spec, first_sym)) => {
                        if first_spec == &spec {
                            continue;
                        }
                        let same_symbol = first_sym
                            .as_ref()
                            .is_some_and(|first| Arc::ptr_eq(first, resolved.as_ref().unwrap()));
                        if same_symbol {
                            continue;
                        }
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            stmt.loc,
                            tsox_core::diagnostics::messages_generated::
                                MODULE_0_HAS_ALREADY_EXPORTED_A_MEMBER_NAMED_1_CONSIDER_EXPLICITLY_RE_EXPORTING_TO_RESOLVE_THE_AMBIGUITY,
                            vec![format!("\"{}\"", first_spec.clone()), name],
                        ));
                        break;
                    }
                }
            }
        }
    }
}
