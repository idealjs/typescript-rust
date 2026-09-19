#![allow(unused_imports)]

use crate::checker::checker_modules::*;

impl Checker {
    pub(crate) fn check_module_export_names(&mut self, node: &Arc<Node>) {
        use tsox_core::core::compiler_options::ModuleKind;

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
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED,
                    vec![],
                ));
            } else if matches!(self.module_kind, ModuleKind::ES2015 | ModuleKind::ES2020)
                && !declaration_file
            {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    tsox_core::diagnostics::messages_generated::
                        STRING_LITERAL_IMPORT_AND_EXPORT_NAMES_ARE_NOT_SUPPORTED_WHEN_THE_MODULE_FLAG_IS_SET_TO_ES2015_OR_ES2020,
                    vec![],
                ));
            }
        }
    }

    pub(crate) fn check_import_untyped_module(&mut self, node: &Arc<Node>) {
        use tsox_core::core::compiler_options::ModuleKind;

        let NodeData::ImportDeclaration(d) = &node.data else {
            return;
        };
        if d.import_clause.is_none() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec = d
            .module_specifier
            .text()
            .trim_matches(['"', '\'', '`'])
            .to_string();
        if self.resolve_module_file_symbol(&spec).is_some() {
            return;
        }
        let Some(path) =
            self.program
                .resolve_external_module_path(&spec, &file.file_name, ModuleKind::None)
        else {
            return;
        };
        let lower = path.to_ascii_lowercase();
        let ts_or_json = lower.ends_with(".ts")
            || lower.ends_with(".tsx")
            || lower.ends_with(".mts")
            || lower.ends_with(".cts")
            || lower.ends_with(".json");
        if ts_or_json {
            return;
        }
        if !self.no_implicit_any {
            return;
        }
        let mut diag = tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            d.module_specifier.loc,
            tsox_core::diagnostics::messages_generated::
                COULD_NOT_FIND_A_DECLARATION_FILE_FOR_MODULE_0_1_IMPLICITLY_HAS_AN_ANY_TYPE,
            vec![spec.clone(), path],
        );
        let types_pkg = if let Some(rest) = spec.strip_prefix('@')
            && let Some((scope, name)) = rest.split_once('/')
        {
            format!("{scope}__{name}")
        } else {
            spec.clone()
        };
        diag.related_information.push(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            d.module_specifier.loc,
            tsox_core::diagnostics::messages_generated::
                TRY_NPM_I_SAVE_DEV_TYPES_SLASH_1_IF_IT_EXISTS_OR_ADD_A_NEW_DECLARATION_D_TS_FILE_CONTAINING_DECLARE_MODULE_0,
            vec![spec, types_pkg],
        ));
        self.diagnostics.add(diag);
    }

    pub(crate) fn check_module_specifier_members(&mut self, node: &Arc<Node>) {
        use tsox_frontend::ast::NodeData;

        let (spec_node, attrs, exclusively_type_only, elements): (
            Arc<Node>,
            Option<Arc<Node>>,
            bool,
            Arc<tsox_frontend::ast::NodeList>,
        ) = match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else {
                    return;
                };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                self.check_default_import_binding(
                    clause,
                    ic,
                    &d.module_specifier,
                    d.attributes.as_ref(),
                );
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
                .unwrap_or(tsox_core::core::compiler_options::ModuleKind::None),
            _ => tsox_core::core::compiler_options::ModuleKind::None,
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
            // Go DeclarationNameToString：成员名按源文本展示（字符串字面量带引号）
            let member_display = self
                .node_source_text(&error_node)
                .unwrap_or_else(|| member_name.clone());
            match self.module_member_lookup(&module_symbol, &member_name) {
                ModuleMemberLookup::Found => {}

                ModuleMemberLookup::LocalNotExported | ModuleMemberLookup::Missing => {
                    // Go errorNoModuleMemberSymbol：模块带 default 导出时
                    // 建议 import x from（优先于本地未导出 TS2459）
                    if module_symbol.exports.get("default").is_some()
                        && member_name != "default"
                    {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            Some(file.clone()),
                            error_node.loc,
                            tsox_core::diagnostics::messages_generated::
                                MODULE_0_HAS_NO_EXPORTED_MEMBER_1_DID_YOU_MEAN_TO_USE_IMPORT_1_FROM_0_INSTEAD,
                            vec![format!("\"{spec_text}\""), member_display],
                        ));
                        continue;
                    }
                    let message = if matches!(
                        self.module_member_lookup(&module_symbol, &member_name),
                        ModuleMemberLookup::LocalNotExported
                    ) {
                        &tsox_core::diagnostics::messages_generated::
                            MODULE_0_DECLARES_1_LOCALLY_BUT_IT_IS_NOT_EXPORTED
                    } else {
                        &tsox_core::diagnostics::messages_generated::MODULE_0_HAS_NO_EXPORTED_MEMBER_1
                    };
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        message.clone(),
                        vec![format!("\"{spec_text}\""), member_display],
                    ));
                }
            }
        }
    }

    // Go getTargetOfModuleDefault/reportNonDefaultExport：默认导入绑定
    // 解析不到 default 导出时报 TS2613/TS1192
    fn check_default_import_binding(
        &mut self,
        clause: &Arc<Node>,
        ic: &tsox_frontend::ast::ImportClauseData,
        spec_node: &Arc<Node>,
        attrs: Option<&Arc<Node>>,
    ) {
        use tsox_core::diagnostics::messages_generated as msgs;

        let Some(binding) = &ic.name else {
            return;
        };
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        let mode = match attrs {
            Some(attrs) if ic.phase_modifier == Some(SyntaxKind::TypeKeyword) => self
                .get_resolution_mode_override(attrs, false)
                .unwrap_or(tsox_core::core::compiler_options::ModuleKind::None),
            _ => tsox_core::core::compiler_options::ModuleKind::None,
        };
        let Some(path) = self
            .program
            .resolve_external_module_path(&spec_text, &file.file_name, mode)
        else {
            return;
        };
        let Some(sf) = self.program.get_source_file(&path) else {
            return;
        };
        let Some(module_symbol) = self.program.symbol_map().symbol_of(&sf.node).cloned() else {
            return;
        };
        if module_symbol
            .value_declaration
            .as_ref()
            .is_some_and(|d| {
                matches!(&d.data, NodeData::ModuleDeclaration(md) if md.body.is_none())
            })
        {
            return;
        }
        let mut default_found = matches!(
            self.module_member_lookup(&module_symbol, "default"),
            ModuleMemberLookup::Found
        );
        // Go canHaveSyntheticDefault：usage 与目标 emit 格式都确定为 ESM 时
        // 不给模块合成 default 导出
        if default_found
            && module_symbol.exports.get("default").is_none()
            && !self.module_has_syntactic_default(&module_symbol)
        {
            let usage_mode = self
                .current_file
                .as_ref()
                .map(|f| self.usage_emit_mode_for_import(&f.file_name))
                .unwrap_or(tsox_core::core::compiler_options::ModuleKind::None);
            let target_mode = self.emit_implied_node_format_for_file(&sf.file_name);
            if usage_mode == tsox_core::core::compiler_options::ModuleKind::ESNext
                && target_mode == tsox_core::core::compiler_options::ModuleKind::ESNext
            {
                default_found = false;
            }
        }
        if default_found {
            return;
        }
        // Go symbolToString：模块符号名 = 文件路径去扩展名（声明文件再去 .d）
        let mut display = sf.file_name.trim_end_matches('/').to_string();
        let lowered = display.to_ascii_lowercase();
        for decl_ext in [
            ".d.ts", ".d.tsx", ".d.mts", ".d.mjs", ".d.cts", ".d.cjs", ".d.jsx",
        ] {
            if lowered.ends_with(decl_ext) {
                display.truncate(display.len() - decl_ext.len());
                break;
            }
        }
        if display == sf.file_name {
            if let Some((idx, _)) = display.char_indices().rev().find(|(_, c)| *c == '.') {
                display.truncate(idx);
            }
        }
        if module_symbol.exports.get(binding.text()).is_some() {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                clause.loc,
                msgs::MODULE_0_HAS_NO_DEFAULT_EXPORT_DID_YOU_MEAN_TO_USE_IMPORT_1_FROM_0_INSTEAD,
                vec![format!("\"{display}\""), binding.text().to_string()],
            ));
        } else {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                binding.loc,
                msgs::MODULE_0_HAS_NO_DEFAULT_EXPORT,
                vec![format!("\"{display}\"")],
            ));
        }
    }
}
