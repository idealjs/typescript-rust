#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn module_has_export_clause(&self, module_symbol: &Arc<Symbol>, name: &str) -> bool {
        use crate::ast::NodeData;
        let mut found = false;
        self.for_each_module_statement(module_symbol, |stmt| {
            if let NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        found = true;
                        return true;
                    }
                }
            }
            false
        });
        found
    }

    pub(crate) fn module_has_syntactic_default(&self, module_symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData;
        let mut found = false;
        self.for_each_module_statement(module_symbol, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(d) if !d.is_export_equals => found = true,
                _ => {
                    if stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Default) {
                        found = true;
                    }
                }
            }
            found
        });
        found
    }

    pub(crate) fn module_is_ambient_export_context(&self, module_symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData;
        let mut is_ambient = false;
        let mut has_export_declaration = false;
        for decl in &module_symbol.declarations {
            let ambient = match &decl.data {
                NodeData::ModuleDeclaration(_) => {
                    decl.has_syntactic_modifier(crate::ast::ModifierFlags::Ambient)
                        || self
                            .get_source_file_of_node(decl)
                            .is_some_and(|f| f.is_declaration_file)
                }
                NodeData::SourceFile(_) => self
                    .get_source_file_of_node(decl)
                    .is_some_and(|f| f.is_declaration_file),
                _ => false,
            };
            is_ambient |= ambient;
        }
        if !is_ambient {
            return false;
        }
        self.for_each_module_statement(module_symbol, |stmt| match &stmt.data {
            NodeData::ExportDeclaration(_) => {
                has_export_declaration = true;
                true
            }
            NodeData::ExportAssignment(_) => {
                has_export_declaration = true;
                true
            }
            _ => false,
        });
        !has_export_declaration
    }

    pub(crate) fn module_ambient_locals_contain(
        &self,
        module_symbol: &Arc<Symbol>,
        name: &str,
    ) -> bool {
        for decl in &module_symbol.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals.get(name).is_some()
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn module_star_chain_exports(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
    ) -> bool {
        if name == "default" {
            return false;
        }
        let stars = self.module_star_specs(module_symbol);
        let mut visited: Vec<*const Symbol> = vec![Arc::as_ptr(module_symbol)];
        for (spec, file) in &stars {
            if let Some(target) = self.resolve_module_symbol_from(spec, file)
                && self.star_target_exports(&target, name, &mut visited, 0)
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn module_star_specs(
        &self,
        module_symbol: &Arc<Symbol>,
    ) -> Vec<(Arc<Node>, Arc<crate::ast::SourceFile>)> {
        use crate::ast::NodeData;
        let mut stars = Vec::new();
        self.for_each_module_statement(module_symbol, |stmt| {
            if let NodeData::ExportDeclaration(d) = &stmt.data
                && d.export_clause.is_none()
                && let Some(spec) = &d.module_specifier
                && let Some(file) = self.get_source_file_of_node(stmt)
            {
                stars.push((Arc::clone(spec), file));
            }
            false
        });
        stars
    }

    pub(crate) fn star_target_exports(
        &mut self,
        target: &Arc<Symbol>,
        name: &str,
        visited: &mut Vec<*const Symbol>,
        depth: usize,
    ) -> bool {
        if depth >= 8 || visited.contains(&Arc::as_ptr(target)) {
            return false;
        }
        visited.push(Arc::as_ptr(target));

        let face = match target.exports.get("export=") {
            Some(ee) => self.resolve_export_equals_target(ee),
            None => Arc::clone(target),
        };
        if face.exports.get(name).is_some()
            || self.module_has_export_clause(&face, name)
            || face
                .members
                .get(name)
                .is_some_and(|s| s.export_symbol.is_some())
            || (self.module_is_ambient_export_context(&face)
                && self.module_ambient_locals_contain(&face, name))
        {
            return true;
        }
        let stars = self.module_star_specs(&face);
        for (spec, file) in &stars {
            if let Some(next) = self.resolve_module_symbol_from(spec, file)
                && self.star_target_exports(&next, name, visited, depth + 1)
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn resolve_module_symbol_from(
        &mut self,
        spec_node: &Arc<Node>,
        file: &Arc<crate::ast::SourceFile>,
    ) -> Option<Arc<Symbol>> {
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(
                    &spec_text,
                    &file.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                )
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        }
    }

    pub(crate) fn resolve_export_equals_target(
        &mut self,
        export_equals: &Arc<Symbol>,
    ) -> Arc<Symbol> {
        let mut target = self.resolve_alias_base(Arc::clone(export_equals));
        for decl in export_equals.declarations.clone() {
            if let crate::ast::NodeData::ExportAssignment(d) = &decl.data
                && matches!(
                    d.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
            {
                if let Some(t) = self.with_declaring_file_context(&decl, |c| {
                    c.resolve_qualified_symbol(&d.expression)
                }) {
                    target = if t.flags.intersects(SymbolFlags::Alias) {
                        self.resolve_alias_base(t)
                    } else {
                        t
                    };
                }
                break;
            }
        }
        target
    }

    pub(crate) fn module_target_has_member(&self, target: &Arc<Symbol>, name: &str) -> bool {
        use crate::ast::NodeData;
        if target.exports.get(name).is_some() || target.members.get(name).is_some() {
            return true;
        }

        let mut has_export_declaration = false;
        let mut ambient = false;
        let mut locals_hit = false;
        for decl in &target.declarations {
            if decl.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            if decl.has_syntactic_modifier(crate::ast::ModifierFlags::Ambient)
                || self
                    .get_source_file_of_node(decl)
                    .is_some_and(|f| f.is_declaration_file)
            {
                ambient = true;
            }
            let body = match &decl.data {
                NodeData::ModuleDeclaration(md) => md.body.clone(),
                _ => None,
            };
            if let Some(body) = body
                && let NodeData::ModuleBlock(b) = &body.data
            {
                for s in b.statements.iter() {
                    match &s.data {
                        NodeData::ExportDeclaration(_) | NodeData::ExportAssignment(_) => {
                            has_export_declaration = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if self
                .program
                .symbol_map()
                .locals
                .get(&decl.id())
                .is_some_and(|l| l.get(name).is_some())
            {
                locals_hit = true;
            }
        }
        ambient && !has_export_declaration && locals_hit
    }

}