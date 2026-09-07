#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_module_export_names(&mut self, node: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;

        let mut names: Vec<(Arc<Node>, bool)> = Vec::new();
        match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else {
                    return;
                };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                let Some(named) = &ic.named_bindings else {
                    return;
                };
                let NodeData::NamedImports(ni) = &named.data else {
                    return;
                };
                for el in ni.elements.iter() {
                    if let NodeData::ImportSpecifier(spec) = &el.data {
                        if let Some(pn) = &spec.property_name {
                            names.push((Arc::clone(pn), true));
                        }
                    }
                }
            }
            NodeData::ExportDeclaration(d) => {
                let has_module_specifier = d.module_specifier.is_some();
                match &d.export_clause {
                    Some(clause) => match &clause.data {
                        NodeData::NamedExports(ne) => {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    if let Some(pn) = &spec.property_name {
                                        names.push((Arc::clone(pn), has_module_specifier));
                                    }
                                    names.push((Arc::clone(&spec.name), true));
                                }
                            }
                        }
                        NodeData::NamespaceExport(ne) => {
                            names.push((Arc::clone(&ne.name), true));
                        }
                        _ => {}
                    },
                    None => {}
                }
            }
            _ => return,
        }
        if names.is_empty() {
            return;
        }
        let declaration_file = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        for (name, string_allowed) in names {
            if name.kind != SyntaxKind::StringLiteral {
                continue;
            }
            if !string_allowed {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::IDENTIFIER_EXPECTED,
                    vec![],
                ));
            } else if matches!(self.module_kind, ModuleKind::ES2015 | ModuleKind::ES2020)
                && !declaration_file
            {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::
                        STRING_LITERAL_IMPORT_AND_EXPORT_NAMES_ARE_NOT_SUPPORTED_WHEN_THE_MODULE_FLAG_IS_SET_TO_ES2015_OR_ES2020,
                    vec![],
                ));
            }
        }
    }

    pub(crate) fn check_module_specifier_members(&mut self, node: &Arc<Node>) {
        use crate::ast::NodeData;

        let (spec_node, attrs, exclusively_type_only, elements): (
            Arc<Node>,
            Option<Arc<Node>>,
            bool,
            Arc<crate::ast::NodeList>,
        ) = match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else {
                    return;
                };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                let Some(named) = &ic.named_bindings else {
                    return;
                };
                let NodeData::NamedImports(ni) = &named.data else {
                    return;
                };
                (
                    Arc::clone(&d.module_specifier),
                    d.attributes.clone(),
                    ic.phase_modifier == Some(SyntaxKind::TypeKeyword),
                    Arc::clone(&ni.elements),
                )
            }
            NodeData::ExportDeclaration(d) => {
                let Some(spec) = &d.module_specifier else {
                    return;
                };
                let Some(clause) = &d.export_clause else {
                    return;
                };
                let NodeData::NamedExports(ne) = &clause.data else {
                    return;
                };
                (
                    Arc::clone(spec),
                    d.attributes.clone(),
                    d.is_type_only,
                    Arc::clone(&ne.elements),
                )
            }
            _ => return,
        };
        if elements.is_empty() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();

        let mode = match (&attrs, exclusively_type_only) {
            (Some(attrs), true) => self
                .get_resolution_mode_override(attrs, false)
                .unwrap_or(crate::core::compiler_options::ModuleKind::None),
            _ => crate::core::compiler_options::ModuleKind::None,
        };

        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(&spec_text, &file.file_name, mode)
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        let module_symbol = if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        };
        let Some(module_symbol) = module_symbol else {
            return;
        };

        let shorthand_ambient = module_symbol.value_declaration.as_ref().is_some_and(
            |d| matches!(&d.data, NodeData::ModuleDeclaration(md) if md.body.is_none()),
        );
        if shorthand_ambient {
            return;
        }
        for element in elements.iter() {
            let (property_name, name) = match &element.data {
                NodeData::ImportSpecifier(d) => (d.property_name.clone(), d.name.clone()),
                NodeData::ExportSpecifier(d) => (d.property_name.clone(), d.name.clone()),
                _ => continue,
            };
            let member_name = property_name
                .as_ref()
                .unwrap_or(&name)
                .text()
                .trim_matches(['"', '\'', '`'])
                .to_string();
            let error_node = property_name.clone().unwrap_or_else(|| Arc::clone(&name));
            match self.module_member_lookup(&module_symbol, &member_name) {
                ModuleMemberLookup::Found => {}

                ModuleMemberLookup::LocalNotExported => {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        crate::diagnostics::messages_generated::
                            MODULE_0_DECLARES_1_LOCALLY_BUT_IT_IS_NOT_EXPORTED,
                        vec![format!("\"{spec_text}\""), member_name],
                    ));
                }
                ModuleMemberLookup::Missing => {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        crate::diagnostics::messages_generated::MODULE_0_HAS_NO_EXPORTED_MEMBER_1,
                        vec![format!("\"{spec_text}\""), member_name],
                    ));
                }
            }
        }
    }
}
