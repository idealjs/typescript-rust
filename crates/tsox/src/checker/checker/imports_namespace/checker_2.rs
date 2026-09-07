#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn namespace_has_value_side(&mut self, namespace: &Arc<Symbol>) -> bool {
        let value_flags = SymbolFlags::Function
            | SymbolFlags::Class
            | SymbolFlags::FunctionScopedVariable
            | SymbolFlags::BlockScopedVariable
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum
            | SymbolFlags::Method;
        let has_value_member = |table: &crate::ast::SymbolTable| {
            table.iter().any(|(name, s)| {
                name != "export="
                    && s.flags.intersects(value_flags)
                    && s.declarations.iter().any(|d| {
                        !matches!(d.kind, SyntaxKind::Parameter | SyntaxKind::MethodSignature)
                    })
            })
        };
        if has_value_member(&namespace.exports) || has_value_member(&namespace.members) {
            return true;
        }

        for d in &namespace.declarations {
            if d.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            let entries: Vec<(String, Arc<Symbol>)> = self
                .program
                .symbol_map()
                .locals
                .get(&d.id())
                .map(|table| {
                    table
                        .iter()
                        .map(|(k, v)| (k.clone(), Arc::clone(v)))
                        .collect()
                })
                .unwrap_or_default();
            if entries.iter().any(|(name, s)| {
                name != "export="
                    && (s.flags.intersects(value_flags)
                        || (s.flags.contains(SymbolFlags::ValueModule)
                            && self.namespace_has_value_side(s)))
            }) {
                return true;
            }
        }

        if self.namespace_value_depth < 4 {
            self.namespace_value_depth += 1;
            let nested = namespace
                .exports
                .iter()
                .chain(namespace.members.iter())
                .any(|(name, s)| {
                    name != "export="
                        && s.flags.contains(SymbolFlags::ValueModule)
                        && self.namespace_has_value_side(s)
                });
            self.namespace_value_depth -= 1;
            if nested {
                return true;
            }
        }

        for decl in &namespace.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals
                    .iter()
                    .any(|(name, s)| name != "export=" && s.flags.intersects(value_flags))
            {
                return true;
            }
        }

        if let Some(export_equals) = namespace.exports.get("export=") {
            for decl in &export_equals.declarations {
                if let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {
                    let scope_decl = namespace
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(scope_decl) = scope_decl {
                        self.push_scope(&scope_decl);
                        let target = self.resolve_qualified_symbol(&ea.expression);
                        self.pop_scope();
                        if let Some(target) = target {
                            if target.flags.intersects(value_flags) {
                                return true;
                            }
                            if target.flags.contains(SymbolFlags::ValueModule) {
                                return self.namespace_has_value_side(&target);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn resolve_module_member_symbol(
        &mut self,
        module_sym: &Arc<Symbol>,
        name: &str,
        depth: usize,
    ) -> Option<Arc<Symbol>> {
        if depth == 0 {
            return None;
        }
        let sym = self.namespace_member_recursive(module_sym, name);
        if let Some(sym) = &sym {
            if let Some(target) = &sym.export_symbol
                && !Arc::ptr_eq(target, &sym)
            {
                return Some(Arc::clone(target));
            }
        }

        let mut clause_hits: Vec<(String, Option<String>)> = Vec::new();
        self.for_each_module_statement(module_sym, |stmt| {
            if let crate::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let crate::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let crate::ast::NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        let imported = spec
                            .property_name
                            .as_ref()
                            .unwrap_or(&spec.name)
                            .text()
                            .trim_matches(['"', '\'', '`'])
                            .to_string();
                        let module_text = d.module_specifier.as_ref().map(|module_spec| {
                            module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string()
                        });
                        clause_hits.push((imported, module_text));
                        return true;
                    }
                }
            }
            false
        });
        for (imported, module_text) in clause_hits {
            let target_module = match module_text {
                None => Arc::clone(module_sym),
                Some(text) => match self.resolve_module_spec_from(module_sym, &text) {
                    Some(m) => m,
                    None => continue,
                },
            };
            if let Some(target) =
                self.resolve_module_member_symbol(&target_module, &imported, depth - 1)
            {
                return Some(target);
            }
        }
        sym
    }

    pub(crate) fn resolve_module_spec_from(
        &self,
        base_module: &Arc<Symbol>,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {
            return self.resolve_module_file_symbol(specifier);
        }
        let dir = base_module
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|f| {
                f.file_name
                    .rfind('/')
                    .map(|i| f.file_name[..i].to_string())
                    .unwrap_or_default()
            })?;
        self.resolve_module_file_symbol_in(&dir, specifier)
    }

    pub(crate) fn type_of_dynamic_import(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        let spec = self.spec_of_dynamic_import_call(node)?;
        if spec.is_empty() {
            return None;
        }
        let cur = self.current_file.clone()?;

        let module_sym = match self.resolve_module_file_symbol(&spec) {
            Some(s) => s,
            None => {
                let path = self.program.resolve_external_module_path(
                    &spec,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::ESNext,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()?
            }
        };
        Some(self.resolve_namespace_type(&module_sym))
    }

    pub(crate) fn spec_of_dynamic_import_call(&self, node: &Arc<Node>) -> Option<String> {
        if node.kind != SyntaxKind::CallExpression {
            return None;
        }
        let (callee, args) = match &node.data {
            NodeData::CallExpression(d) => (&d.expression, &d.arguments),
            _ => return None,
        };
        if callee.kind != SyntaxKind::ImportKeyword {
            return None;
        }
        let spec_node = args.iter().next()?;
        if spec_node.kind != SyntaxKind::StringLiteral {
            return None;
        }
        Some(spec_node.text().trim_matches(['"', '\'', '`']).to_string())
    }
}
