#![allow(unused_imports)]

use crate::checker::checker_modules::*;

impl Checker {
    pub(crate) fn resolve_import_alias_module(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_import_alias_module"); 
        let decl = symbol
            .declarations
            .iter()
            .find(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::NamespaceImport
                        | SyntaxKind::ImportSpecifier
                        | SyntaxKind::NamespaceExport
                        | SyntaxKind::ExportSpecifier
                )
            })?
            .clone();

        let mut cur = decl;
        for _ in 0..4 {
            let Some(parent) = cur.parent() else {
                return None;
            };
            if parent.kind == SyntaxKind::ExportDeclaration {
                if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &parent.data {
                    let Some(specifier) = &d.module_specifier else {
                        return None;
                    };
                    let spec = specifier.text();
                    if !spec.starts_with('.') {
                        // Go resolveExternalModuleName（checker.go:15942）按
                        // 声明文件的程序期解析——re-export 说明符相对其所在
                        // 文件解析，不随显示/求值上下文平移（/node_modules/a/
                        // index.d.ts 的 "foo" 落 a/node_modules/foo）。快索引
                        // 以当前检查文件为向上起点，跨层 node_modules 命中不
                        // 了时兜底同一程序解析（与下方相对臂及 ImportDeclaration
                        // 非相对臂同构）
                        return self.resolve_module_file_symbol(&spec).or_else(|| {
                            self.resolve_module_file_by_program_resolution(&parent, &spec)
                        });
                    }
                    let dir = self.declaring_dir_of(&parent)?;
                    let decl_file = self.get_source_file_of_node(&parent);
                    if self.node_next_esm_relative_import_unresolved(
                        &parent,
                        decl_file.as_deref(),
                        &spec,
                    ) {
                        return None;
                    }
                    if let Some(sym) = self.resolve_module_file_symbol_in(&dir, &spec) {
                        return Some(sym);
                    }
                    return self.resolve_module_file_by_program_resolution(&parent, &spec);
                }
                return None;
            }
            if parent.kind == SyntaxKind::ImportDeclaration {
                if let tsox_frontend::ast::NodeData::ImportDeclaration(d) = &parent.data {
                    let spec = d.module_specifier.text();
                    if !spec.starts_with('.') {
                        // Go resolveExternalModuleName（checker.go:15942）经程序
                        // 模块解析结果取模块符号；本仓快速 node_modules 索引
                        // 文件命中不了 package.json typings 指向的入口
                        //（@raymondfeng/pkg2 → dist/index.d.ts），未命中按
                        // 同一程序解析兜底（非相对臂与相对臂同规）
                        return self.resolve_module_file_symbol(&spec).or_else(|| {
                            self.resolve_module_file_by_program_resolution(&parent, &spec)
                        });
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
                    let decl_file = self.get_source_file_of_node(&parent);
                    if self.node_next_esm_relative_import_unresolved(
                        &parent,
                        decl_file.as_deref(),
                        &spec,
                    ) {
                        return None;
                    }
                    if let Some(sym) = self.resolve_module_file_symbol_in(&dir, &spec) {
                        return Some(sym);
                    }
                    return self.resolve_module_file_by_program_resolution(&parent, &spec);
                }
                return None;
            }
            cur = parent;
        }
        None
    }
    pub(crate) fn resolve_module_file_by_program_resolution(
        &self,
        declaration: &Arc<Node>,
        module_specifier: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_module_file_by_program_resolution");
        let file = self.get_source_file_of_node(declaration)?;
        let trimmed = module_specifier.trim_matches(['"', '\'', '`']);
        // resolve_external_module_path 返回 realpath 形态；程序 file_name 可能是
        // 相对形态（cwd 基），按绝对/相对两种形态精确匹配，不做逐文件 realpath
        let path = self.program.resolve_external_module_path(
            trimmed,
            &file.file_name,
            tsox_core::core::compiler_options::ModuleKind::None,
        )?;
        let sf = self.program.get_source_file(&path).or_else(|| {
            let cwd = self.program.current_directory();
            let rel = path
                .strip_prefix(cwd)
                .map(|r| r.trim_start_matches('/'))
                .unwrap_or(&path);
            self.program
                .source_files()
                .iter()
                .find(|f| f.file_name == rel || f.file_name == path)
                .cloned()
        })?;
        let symbol = self.program.symbol_map().symbol_of(&sf.node).cloned()?;
        Some(self.get_merged_symbol(&symbol))
    }
    pub(crate) fn check_module_format_mismatch(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_module_format_mismatch"); 
        use tsox_core::core::compiler_options::ModuleKind;
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
        // Go checkImportDeclaration 按 getModeForUsageLocation 取模式；
        // attrs 覆盖已在上方早退，这里用当前文件的默认解析格式
        let resolution_mode =
            tsox_tsoptions::tsoptions::implied_node_format_of_file(&file.file_name, &read);
        let target_path = match self.program.resolve_external_module_path(
            &spec_text,
            &file.file_name,
            resolution_mode,
        ) {
            Some(p) => p,
            None => return,
        };

        if !module_format_is_esm_for_require_check(&target_path, &read) {
            return;
        }
        if is_import_equals {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                tsox_core::diagnostics::messages_generated::
                    MODULE_0_CANNOT_BE_IMPORTED_USING_THIS_CONSTRUCT_THE_SPECIFIER_ONLY_RESOLVES_TO_AN_ES_MODULE_WHICH_CANNOT_BE_IMPORTED_WITH_REQUIRE_USE_AN_ECMASCRIPT_IMPORT_INSTEAD,
                vec![spec_text.clone()],
            ));
        } else if importer_is_cjs_for_require_check(&file.file_name, &read) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                tsox_core::diagnostics::messages_generated::
                    THE_CURRENT_FILE_IS_A_COMMONJS_MODULE_WHOSE_IMPORTS_WILL_PRODUCE_REQUIRE_CALLS_HOWEVER_THE_REFERENCED_FILE_IS_AN_ECMASCRIPT_MODULE_AND_CANNOT_BE_IMPORTED_WITH_REQUIRE_CONSIDER_WRITING_A_DYNAMIC_IMPORT_0_CALL_INSTEAD,
                vec![spec_text],
            ));
        }
    }

    pub(crate) fn check_declaration_nameability(&mut self, stmt: &Arc<Node>) { ::tsox_core::fntrace::enter("check_declaration_nameability"); 
        let Some(file) = self.nameability_file_guards() else {
            return;
        };
        if stmt.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export) {
            let tsox_frontend::ast::NodeData::VariableStatement(data) = &stmt.data else {
                return;
            };
            let tsox_frontend::ast::NodeData::VariableDeclarationList(list) =
                &data.declaration_list.data
            else {
                return;
            };
            let (mut imported_files, spec_names) = self.nameability_import_context(&file);
            let imported_symbol_ids = self.imported_binding_symbol_ids(&file);
            for d in list.declarations.iter() {
                let tsox_frontend::ast::NodeData::VariableDeclaration(vd) = &d.data else {
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
                                tsox_core::core::compiler_options::ModuleKind::ESNext,
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
                self.check_type_nameability(
                    &file,
                    &vd.name,
                    &var_name,
                    &t,
                    &imported_files,
                    &spec_names,
                    &imported_symbol_ids,
                );
            }
        }
    }

    pub(crate) fn check_declaration_nameability_export_default(&mut self, stmt: &Arc<Node>) { ::tsox_core::fntrace::enter("check_declaration_nameability_export_default"); 
        let Some(file) = self.nameability_file_guards() else {
            return;
        };
        let tsox_frontend::ast::NodeData::ExportAssignment(data) = &stmt.data else {
            return;
        };
        if data.is_export_equals {
            return;
        }
        let (imported_files, spec_names) = self.nameability_import_context(&file);
        let imported_symbol_ids = self.imported_binding_symbol_ids(&file);
        // Go 声明发射：`export default <非 identifier 表达式>` 合成
        // `declare const _default: <表达式类型>` 并经 tracker 序列化该类型
        //（fallback 节点 = ExportAssignment）；identifier 形态原样重发标识符，
        // 不写类型，不做可命名性检查
        let t = if data.expression.kind == SyntaxKind::Identifier {
            let Some(sym) = self.program.symbol_map().symbol_of(stmt).cloned() else {
                return;
            };
            self.get_type_of_symbol(&sym)
        } else {
            self.get_type_of_node(&data.expression)
        };
        self.check_type_nameability(
            &file,
            stmt,
            "default",
            &t,
            &imported_files,
            &spec_names,
            &imported_symbol_ids,
        );
    }

    fn nameability_file_guards(&self) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("nameability_file_guards"); 
        if !self.program.options().declaration.is_true() {
            return None;
        }
        let file = self.current_file.clone()?;
        if file.file_name.starts_with("bundled://") || file.is_declaration_file {
            return None;
        }
        if file.file_name.contains("/node_modules/") {
            return None;
        }
        Some(file)
    }

    fn nameability_import_context(
        &self,
        file: &Arc<SourceFile>,
    ) -> (Vec<String>, Vec<String>) { ::tsox_core::fntrace::enter("nameability_import_context"); 
        let mut imported_files: Vec<String> = Vec::new();
        let mut spec_names: Vec<String> = Vec::new();
        let NodeData::SourceFile(sfd) = &file.node.data else {
            return (imported_files, spec_names);
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
                tsox_core::core::compiler_options::ModuleKind::None,
            ) {
                imported_files.push(p);
            }
        }
        (imported_files, spec_names)
    }

    fn imported_binding_symbol_ids(&self, file: &Arc<SourceFile>) -> Vec<u64> { ::tsox_core::fntrace::enter("imported_binding_symbol_ids"); 
        let NodeData::SourceFile(sfd) = &file.node.data else {
            return Vec::new();
        };
        let mut ids = Vec::new();
        for st in sfd.statements.iter() {
            let NodeData::ImportDeclaration(imp) = &st.data else {
                continue;
            };
            let Some(clause) = &imp.import_clause else {
                continue;
            };
            let NodeData::ImportClause(ic) = &clause.data else {
                continue;
            };
            let mut bindings: Vec<Arc<Node>> = Vec::new();
            if let Some(name) = &ic.name {
                bindings.push(Arc::clone(name));
            }
            if let Some(named) = &ic.named_bindings {
                if let NodeData::NamedImports(ni) = &named.data {
                    for el in ni.elements.iter() {
                        bindings.push(Arc::clone(el));
                    }
                }
            }
            for b in bindings {
                if let Some(sym) = self.program.symbol_map().symbol_of(&b).cloned() {
                    ids.push(sym.id());
                }
            }
        }
        ids
    }

    pub(crate) fn symbol_in_ambient_module_named(
        &self,
        symbol: &Arc<Symbol>,
        imported_specs: &[String],
    ) -> bool { ::tsox_core::fntrace::enter("symbol_in_ambient_module_named"); 
        if imported_specs.is_empty() {
            return false;
        }
        for decl in &symbol.declarations {
            let mut cur = decl.parent();
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
                cur = n.parent();
            }
        }
        false
    }
}
