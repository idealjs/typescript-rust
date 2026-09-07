#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn ambient_namespace_locals_visible(&self, ns: &Arc<Symbol>) -> bool {
        if std::env::var_os("TSOX_NO_AMBIENT").is_some() {
            return false;
        }
        ns.declarations.iter().any(|d| {
            d.kind == SyntaxKind::ModuleDeclaration
                && (d.has_syntactic_modifier(ModifierFlags::Ambient)
                    || self.ambient_ancestor(d)
                    || self
                        .get_source_file_of_node(d)
                        .is_some_and(|f| f.is_declaration_file))
                && !crate::binder::Binder::has_export_declarations(d)
        })
    }

    pub(crate) fn ambient_namespace_local(
        &self,
        ns: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !self.ambient_namespace_locals_visible(ns) {
            return None;
        }
        ns.declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .find_map(|d| {
                self.program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
                    .cloned()
            })
    }

    pub(crate) fn resolve_alias_base(&mut self, symbol: Arc<Symbol>) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return symbol;
        }

        if symbol.declarations.iter().any(|d| {
            matches!(
                d.kind,
                SyntaxKind::NamespaceImport | SyntaxKind::NamespaceExport
            )
        }) && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
        {
            return module_sym;
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            if let crate::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {
                if let crate::ast::NodeData::ExternalModuleReference(ext) =
                    &data.module_reference.data
                    && ext.expression.kind == SyntaxKind::StringLiteral
                    && let Some(module_sym) =
                        self.resolve_module_file_symbol(&ext.expression.text())
                {
                    if let Some(export_eq) = module_sym
                        .exports
                        .get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                    {
                        let entity_decl = export_eq
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ExportAssignment)
                            .cloned();
                        let scope_decl = module_sym
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                            .cloned();
                        if let (Some(export_decl), Some(scope)) = (entity_decl, scope_decl)
                            && let crate::ast::NodeData::ExportAssignment(ea) = &export_decl.data
                            && ea.is_export_equals
                            && matches!(
                                ea.expression.kind,
                                SyntaxKind::Identifier | SyntaxKind::QualifiedName
                            )
                        {
                            self.push_scope(&scope);
                            let target = self.resolve_qualified_symbol(&ea.expression);
                            self.pop_scope();
                            if let Some(target) = target {
                                return target;
                            }
                        }
                    }
                    return module_sym;
                }

                if matches!(
                    data.module_reference.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    let mut current = Arc::clone(&symbol);
                    for _ in 0..4 {
                        let next = current
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                            .and_then(|d| {
                                if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                                    && matches!(
                                        ied.module_reference.kind,
                                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                    )
                                {
                                    Some(self.resolve_qualified_symbol(&ied.module_reference))
                                } else {
                                    None
                                }
                            })
                            .flatten();
                        match next {
                            Some(n) => current = n,
                            None => break,
                        }
                        if !current.flags.intersects(SymbolFlags::Alias) {
                            return current;
                        }
                    }
                    return current;
                }
            }
        }
        symbol
    }

    pub(crate) fn resolve_module_file_symbol(&self, specifier: &str) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {
            for file in self.program.source_files() {
                if file.external_module_indicator.is_some() {
                    continue;
                }
                if let crate::ast::NodeData::SourceFile(sf) = &file.node.data {
                    for stmt in sf.statements.iter() {
                        if let crate::ast::NodeData::ModuleDeclaration(md) = &stmt.data
                            && md.name.kind == SyntaxKind::StringLiteral
                            && md.name.text().trim_matches(['"', '\'']) == specifier
                        {
                            return self.program.symbol_map().symbol_of(stmt).cloned();
                        }
                    }
                }
            }
            return None;
        }
        let current = self.current_file.as_ref()?;
        let dir = match current.file_name.rfind('/') {
            Some(i) => &current.file_name[..i],
            None => "",
        };
        self.resolve_module_file_symbol_in(dir, specifier)
    }

    pub(crate) fn resolve_module_file_symbol_in(
        &self,
        dir: &str,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        let stem = specifier.strip_prefix("./").unwrap_or(specifier);

        let stem = stem
            .strip_suffix(".js")
            .or_else(|| stem.strip_suffix(".jsx"))
            .unwrap_or(stem);
        let symbol_map = self.program.symbol_map();
        for cand in [
            format!("{dir}/{stem}.ts"),
            format!("{dir}/{stem}.tsx"),
            format!("{dir}/{stem}.d.ts"),
            format!("{dir}/{stem}/index.ts"),
            format!("{dir}/{stem}/index.d.ts"),
        ] {
            if let Some(sf) = self
                .program
                .source_files()
                .iter()
                .find(|f| f.file_name == cand)
            {
                if let Some(sym) = symbol_map.symbol_of(&sf.node) {
                    return Some(Arc::clone(sym));
                }
            }
        }
        None
    }

    pub(crate) fn for_each_module_statement(
        &self,
        module_symbol: &Arc<Symbol>,
        mut f: impl FnMut(&Arc<Node>) -> bool,
    ) {
        use crate::ast::NodeData;
        for decl in &module_symbol.declarations {
            let statements: Option<&Arc<crate::ast::NodeList>> = match &decl.data {
                NodeData::SourceFile(sf) => Some(&sf.statements),
                NodeData::ModuleDeclaration(md) => match &md.body {
                    Some(body) => match &body.data {
                        NodeData::ModuleBlock(b) => Some(&b.statements),
                        _ => None,
                    },
                    None => None,
                },
                _ => None,
            };
            if let Some(list) = statements {
                for s in list.iter() {
                    if f(s) {
                        return;
                    }
                }
            }
        }
    }

    pub(crate) fn class_name_text(class: &Arc<Node>) -> String {
        match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => d
                .name
                .as_ref()
                .map(|n| n.text().to_string())
                .unwrap_or_default(),
            crate::ast::NodeData::ClassExpression(d) => d
                .name
                .as_ref()
                .map(|n| n.text().to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    pub(crate) fn class_member_static_by_name(
        &self,
        class: &Arc<Node>,
        name: &str,
    ) -> Option<bool> {
        let members = match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            crate::ast::NodeData::ClassExpression(d) => &d.members,
            _ => return None,
        };
        for member in members.iter() {
            let member_name = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                _ => continue,
            };
            if member_name.kind == SyntaxKind::Identifier && member_name.text() == name {
                return Some(member.has_syntactic_modifier(ModifierFlags::Static));
            }
        }
        None
    }
}
