#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_import_alias_module(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::NamespaceImport
                        | SyntaxKind::ImportSpecifier
                        | SyntaxKind::NamespaceExport
                )
            })?
            .clone();

        let mut cur = decl;
        for _ in 0..4 {
            let Some(parent) = cur.parent.clone() else {
                return None;
            };
            if parent.kind == SyntaxKind::ExportDeclaration {
                if let crate::ast::NodeData::ExportDeclaration(d) = &parent.data {
                    let Some(specifier) = &d.module_specifier else {
                        return None;
                    };
                    let spec = specifier.text();
                    if !spec.starts_with('.') {
                        return self.resolve_module_file_symbol(&spec);
                    }
                    let dir = self.declaring_dir_of(&parent)?;
                    return self.resolve_module_file_symbol_in(&dir, &spec);
                }
                return None;
            }
            if parent.kind == SyntaxKind::ImportDeclaration {
                if let crate::ast::NodeData::ImportDeclaration(d) = &parent.data {
                    let spec = d.module_specifier.text();
                    if !spec.starts_with('.') {
                        return self.resolve_module_file_symbol(&spec);
                    }

                    let dir = self
                        .get_source_file_of_node(&parent)
                        .map(|f| match f.file_name.rfind('/') {
                            Some(i) => f.file_name[..i].to_string(),
                            None => String::new(),
                        })
                        .or_else(|| {
                            self.current_file
                                .as_ref()
                                .map(|f| match f.file_name.rfind('/') {
                                    Some(i) => f.file_name[..i].to_string(),
                                    None => String::new(),
                                })
                        })?;
                    return self.resolve_module_file_symbol_in(&dir, &spec);
                }
                return None;
            }
            cur = parent;
        }
        None
    }
    pub(crate) fn check_module_format_mismatch(&mut self, node: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;
        if !matches!(self.module_kind, ModuleKind::Node16 | ModuleKind::Node18) {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        if file.file_name.starts_with("bundled://") {
            return;
        }
        let (spec_node, attrs, is_import_equals): (Arc<Node>, Option<Arc<Node>>, bool) = match &node
            .data
        {
            NodeData::ImportDeclaration(d) => {
                (Arc::clone(&d.module_specifier), d.attributes.clone(), false)
            }
            NodeData::ExportDeclaration(d) => match &d.module_specifier {
                Some(spec) => (Arc::clone(spec), d.attributes.clone(), false),
                None => return,
            },
            NodeData::ImportEqualsDeclaration(d) => match &d.module_reference.data {
                NodeData::ExternalModuleReference(ext) => (Arc::clone(&ext.expression), None, true),
                _ => return,
            },
            _ => return,
        };

        if let Some(attrs) = &attrs
            && self.get_resolution_mode_override(attrs, false).is_some()
        {
            return;
        }
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        if spec_text.is_empty() {
            return;
        }
        let read = |p: &str| self.program.read_file(p);
        let target_path = match self.program.resolve_external_module_path(
            &spec_text,
            &file.file_name,
            ModuleKind::None,
        ) {
            Some(p) => p,
            None => return,
        };

        if !module_format_is_esm_for_require_check(&target_path, &read) {
            return;
        }
        if is_import_equals {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                crate::diagnostics::messages_generated::
                    MODULE_0_CANNOT_BE_IMPORTED_USING_THIS_CONSTRUCT_THE_SPECIFIER_ONLY_RESOLVES_TO_AN_ES_MODULE_WHICH_CANNOT_BE_IMPORTED_WITH_REQUIRE_USE_AN_ECMASCRIPT_IMPORT_INSTEAD,
                vec![spec_text.clone()],
            ));
        } else if importer_is_cjs_for_require_check(&file.file_name, &read) {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                crate::diagnostics::messages_generated::
                    THE_CURRENT_FILE_IS_A_COMMONJS_MODULE_WHOSE_IMPORTS_WILL_PRODUCE_REQUIRE_CALLS_HOWEVER_THE_REFERENCED_FILE_IS_AN_ECMASCRIPT_MODULE_AND_CANNOT_BE_IMPORTED_WITH_REQUIRE_CONSIDER_WRITING_A_DYNAMIC_IMPORT_0_CALL_INSTEAD,
                vec![spec_text],
            ));
        }
    }

    pub(crate) fn check_declaration_nameability(&mut self, stmt: &Arc<Node>) {
        if !self.program.options().declaration.is_true() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        if file.file_name.starts_with("bundled://") || file.is_declaration_file {
            return;
        }

        if file.file_name.contains("/node_modules/") {
            return;
        }
        let crate::ast::NodeData::VariableStatement(data) = &stmt.data else {
            return;
        };
        let has_export = stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
        if !has_export {
            return;
        }

        let mut imported_files: Vec<String> = Vec::new();
        let mut spec_names: Vec<String> = Vec::new();
        let NodeData::SourceFile(sfd) = &file.node.data else {
            return;
        };
        for st in sfd.statements.iter() {
            let spec = match &st.data {
                NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
                NodeData::ExportDeclaration(d) => match &d.module_specifier {
                    Some(s) => s.text().to_string(),
                    None => continue,
                },
                _ => continue,
            };
            let text = spec.trim_matches(['"', '\'', '`']).to_string();
            if text.is_empty() {
                continue;
            }
            spec_names.push(text.clone());
            if let Some(p) = self.program.resolve_external_module_path(
                &text,
                &file.file_name,
                crate::core::compiler_options::ModuleKind::None,
            ) {
                imported_files.push(p);
            }
        }
        let crate::ast::NodeData::VariableDeclarationList(list) = &data.declaration_list.data
        else {
            return;
        };
        for d in list.declarations.iter() {
            let crate::ast::NodeData::VariableDeclaration(vd) = &d.data else {
                continue;
            };

            if let Some(init) = &vd.initializer {
                let mut import_expr = Some(Arc::clone(init));
                if let Some(inner) = import_expr.take() {
                    let unwrapped = match &inner.data {
                        NodeData::AwaitExpression(a) => Some(Arc::clone(&a.expression)),
                        _ => Some(inner),
                    };
                    if let Some(call) = unwrapped
                        && call.kind == SyntaxKind::CallExpression
                        && let Some(spec) = self.spec_of_dynamic_import_call(&call)
                        && let Some(path) = self.program.resolve_external_module_path(
                            &spec,
                            &file.file_name,
                            crate::core::compiler_options::ModuleKind::ESNext,
                        )
                        && !imported_files.contains(&path)
                    {
                        imported_files.push(path);
                    }
                }
            }

            if vd.type_node.is_some() {
                continue;
            }
            let Some(sym) = self.program.symbol_map().symbol_of(d).cloned() else {
                continue;
            };
            let var_name = vd.name.text().to_string();
            let t = self.get_type_of_symbol(&sym);
            let Some(target) = t.symbol.clone() else {
                continue;
            };
            let Some(target_file) = target
                .declarations
                .first()
                .and_then(|dn| self.get_source_file_of_node(dn))
            else {
                continue;
            };
            if target_file.file_name == file.file_name
                || !target_file.file_name.contains("/node_modules/")
                || imported_files.contains(&target_file.file_name)
            {
                continue;
            }

            if self.symbol_in_ambient_module_named(&target, &spec_names) {
                continue;
            }

            let spec = relative_emit_specifier(&file.file_name, &target_file.file_name);
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file.clone()),
                vd.name.loc,
                crate::diagnostics::messages_generated::
                    THE_INFERRED_TYPE_OF_0_CANNOT_BE_NAMED_WITHOUT_A_REFERENCE_TO_2_FROM_1_THIS_IS_LIKELY_NOT_PORTABLE_A_TYPE_ANNOTATION_IS_NECESSARY,
                vec![var_name, spec, target.name.clone()],
            ));
        }
    }

    pub(crate) fn symbol_in_ambient_module_named(
        &self,
        symbol: &Arc<Symbol>,
        imported_specs: &[String],
    ) -> bool {
        if imported_specs.is_empty() {
            return false;
        }
        for decl in &symbol.declarations {
            let mut cur = decl.parent.as_ref();
            while let Some(n) = cur {
                if let NodeData::ModuleDeclaration(md) = &n.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    let module_name = md.name.text().trim_matches(['"', '\'']).to_string();
                    return imported_specs.iter().any(|s| *s == module_name);
                }
                if n.kind == SyntaxKind::SourceFile {
                    break;
                }
                cur = n.parent.as_ref();
            }
        }
        false
    }
}
