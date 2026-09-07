#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_namespace_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let mut members: Vec<(String, Arc<Symbol>)> = symbol
            .exports
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();

        if self.ambient_namespace_locals_visible(symbol) {
            let local_members: Vec<(String, Arc<Symbol>)> = symbol
                .declarations
                .iter()
                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                .filter_map(|d| {
                    self.program.symbol_map().locals.get(&d.id()).map(|l| {
                        l.iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v)))
                            .collect::<Vec<(String, Arc<Symbol>)>>()
                    })
                })
                .flatten()
                .collect();
            for (k, v) in local_members {
                if !members.iter().any(|(mk, _)| *mk == k) {
                    members.push((k, v));
                }
            }
        }

        let mut file_exported: Vec<(String, Arc<Symbol>)> = Vec::new();
        if symbol
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
            && members.is_empty()
        {
            let sym_map = self.program.symbol_map();

            let mut wanted: Vec<(String, Option<String>)> = Vec::new();
            let mut default_node: Option<Arc<Node>> = None;
            self.for_each_module_statement(symbol, |stmt| {
                let has_export = stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                match &stmt.data {
                    NodeData::ExportDeclaration(d) => {
                        if let Some(clause) = &d.export_clause
                            && let NodeData::NamedExports(ne) = &clause.data
                        {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    let exported =
                                        spec.name.text().trim_matches(['"', '\'', '`']).to_string();
                                    let local = spec
                                        .property_name
                                        .as_ref()
                                        .unwrap_or(&spec.name)
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    wanted.push((exported, Some(local)));
                                }
                            }
                        }
                    }
                    NodeData::ExportAssignment(ea) => {
                        if !ea.is_export_equals && default_node.is_none() {
                            default_node = Some(Arc::clone(stmt));
                        }
                    }
                    NodeData::VariableStatement(vs) if has_export => {
                        if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                            for decl in vdl.declarations.iter() {
                                if let Some(name) = decl.name() {
                                    wanted.push((name.text().to_string(), None));
                                }
                            }
                        }
                    }
                    _ if has_export => {
                        if let Some(name) = stmt.name() {
                            wanted.push((name.text().to_string(), None));
                        }
                    }
                    _ => {}
                }
                false
            });
            let file_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SourceFile);
            let locals = file_node.and_then(|f| sym_map.locals.get(&f.id()));
            for (exported, clause_local) in wanted.iter() {
                if members.iter().any(|(k, _)| *k == *exported)
                    || file_exported.iter().any(|(k, _)| *k == *exported)
                {
                    continue;
                }
                let lookup = clause_local.as_deref().unwrap_or(exported);
                if let Some(s) = locals.and_then(|l| l.get(lookup).cloned()) {
                    file_exported.push((exported.clone(), s));
                } else if let Some(s) = symbol.members.get(lookup).cloned() {
                    file_exported.push((exported.clone(), s));
                }
            }
            if let Some(node) = default_node
                && !members.iter().any(|(k, _)| k == "default")
                && !file_exported.iter().any(|(k, _)| k == "default")
            {
                let s = sym_map.symbol_of(&node).cloned().or_else(|| {
                    node.expression()
                        .and_then(|e| sym_map.symbol_of(e).cloned())
                });
                if let Some(s) = s {
                    file_exported.push(("default".to_string(), s));
                }
            }
        }

        let mut reexported: Vec<(String, Arc<Symbol>)> = Vec::new();
        {
            let mut clause_specs: Vec<(String, String, String)> = Vec::new();
            self.for_each_module_statement(symbol, |stmt| {
                if let NodeData::ExportDeclaration(d) = &stmt.data
                    && let Some(clause) = &d.export_clause
                    && let NodeData::NamedExports(ne) = &clause.data
                    && let Some(module_spec) = &d.module_specifier
                {
                    for el in ne.elements.iter() {
                        if let NodeData::ExportSpecifier(spec) = &el.data {
                            let exported =
                                spec.name.text().trim_matches(['"', '\'', '`']).to_string();
                            let imported = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let module_text = module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            if !exported.is_empty() && !module_text.is_empty() {
                                clause_specs.push((exported, imported, module_text));
                            }
                        }
                    }
                }
                false
            });
            for (exported, imported, module_text) in clause_specs {
                if members.iter().any(|(k, _)| *k == exported)
                    || reexported.iter().any(|(k, _)| *k == exported)
                {
                    continue;
                }
                let target = self
                    .resolve_module_spec_from(symbol, &module_text)
                    .and_then(|m| self.resolve_module_member_symbol(&m, &imported, 8));
                if let Some(t) = target {
                    reexported.push((exported, t));
                }
            }
        }
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in members
            .iter()
            .chain(file_exported.iter())
            .chain(reexported.iter())
        {
            if name.starts_with(crate::ast::INTERNAL_SYMBOL_NAME_PREFIX)
                || name == crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            {
                continue;
            }
            let member_type = self.get_type_of_symbol(member_sym);

            let prop_sym = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
            self.value_symbol_links.insert(
                &prop_sym,
                ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name.clone(), Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        result
    }
}
