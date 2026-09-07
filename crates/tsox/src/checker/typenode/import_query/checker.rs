#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_query(node);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn resolve_import_alias_target_symbol(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let (member_name, import_decl): (String, Arc<Node>) = {
            let decl = alias
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ImportClause | SyntaxKind::ImportSpecifier
                    )
                })?
                .clone();
            match &decl.data {
                NodeData::ImportClause(_) => ("default".to_string(), decl),
                NodeData::ImportSpecifier(d) => (
                    d.property_name
                        .as_ref()
                        .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                    decl,
                ),
                _ => return None,
            }
        };
        let mut import_decl = import_decl.parent.as_ref()?;
        while !matches!(import_decl.data, NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_sym = self.resolve_module_file_symbol(&module_spec).or_else(|| {
            let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
            let cur = self.current_file.clone()?;
            let path = self.program.resolve_external_module_path(
                &trimmed,
                &cur.file_name,
                crate::core::compiler_options::ModuleKind::None,
            )?;
            let sf = self.program.get_source_file(&path)?;
            self.program.symbol_map().symbol_of(&sf.node).cloned()
        })?;
        let resolved = self
            .resolve_module_member_symbol(&module_sym, &member_name, 8)
            .or_else(|| self.file_module_exported_member(&module_sym, &member_name));
        let resolved = match resolved {
            Some(t)
                if !t.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                ) =>
            {
                let mut cur = Arc::clone(&t);
                for _ in 0..4 {
                    if cur.flags != crate::ast::SymbolFlags::Alias {
                        break;
                    }

                    let next = cur
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .and_then(|d| match &d.data {
                            NodeData::ExportAssignment(ea)
                                if matches!(
                                    ea.expression.kind,
                                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                ) =>
                            {
                                Some(ea.expression.text().to_string())
                            }
                            _ => None,
                        })
                        .and_then(|n| {
                            module_sym
                                .members
                                .get(&n)
                                .cloned()
                                .or_else(|| module_sym.exports.get(&n).cloned())
                        });
                    match next {
                        Some(n) => cur = n,
                        None => break,
                    }
                }
                let has_type_meaning = cur.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                );
                if has_type_meaning { Some(cur) } else { Some(t) }
            }
            other => other,
        };
        resolved
    }

    pub(crate) fn file_module_exported_member(
        &self,
        module_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !module_sym
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
        {
            return None;
        }
        if let Some(s) = module_sym.exports.get(name) {
            return Some(Arc::clone(s));
        }
        let sym_map = self.program.symbol_map();
        let mut found: Option<Arc<Symbol>> = None;
        self.for_each_module_statement(module_sym, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(ea) => {
                    if !ea.is_export_equals && name == "default" && found.is_none() {
                        let by_name = match &ea.expression.kind {
                            SyntaxKind::Identifier => module_sym
                                .members
                                .get(ea.expression.text())
                                .cloned()
                                .or_else(|| module_sym.exports.get(ea.expression.text()).cloned()),
                            _ => None,
                        };
                        found = by_name.or_else(|| {
                            sym_map.symbol_of(stmt).cloned().or_else(|| {
                                stmt.expression()
                                    .and_then(|e| sym_map.symbol_of(e).cloned())
                            })
                        });
                    }
                }
                NodeData::VariableStatement(vs) => {
                    if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                        for decl in vdl.declarations.iter() {
                            if decl.name().is_some_and(|n| n.text() == name) {
                                found = sym_map.symbol_of(decl).cloned();
                            }
                        }
                    }
                }
                _ => {
                    if stmt.name().is_some_and(|n| n.text() == name)
                        && (stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                            || stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Default))
                    {
                        found = sym_map.symbol_of(stmt).cloned();
                    }
                }
            }
            false
        });
        found
    }
}
