#![allow(unused_imports)]

use super::*;

impl Program {
    pub fn new(opts: ProgramOptions) -> Self {
        let host = opts.host;
        let mut options = opts.config.compiler_options.clone();
        let config_file_name = opts.config.config_file_name.clone();

        if !config_file_name.is_empty() && options.config_file_path.is_empty() {
            options.config_file_path = config_file_name.clone();
        }

        let mut source_files: Vec<Arc<SourceFile>> = Vec::new();
        let mut by_name: HashMap<String, Arc<SourceFile>> = HashMap::new();
        let mut default_lib_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut diagnostics: Vec<Arc<Diagnostic>> = Vec::new();

        if !options.lib.is_empty() && options.no_lib.is_true() {
            diagnostics.push(Arc::new(Diagnostic::new(
                None,
                TextRange::default(),
                tsox_core::diagnostics::messages_generated::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                vec!["lib".to_string(), "noLib".to_string()],
            )));
        }

        if options.emit_declaration_only.is_true() && !options.get_emit_declarations() {
            diagnostics.push(Arc::new(Diagnostic::new(
                None,
                TextRange::default(),
                tsox_core::diagnostics::messages_generated::
                    OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                vec![
                    "emitDeclarationOnly".to_string(),
                    "declaration".to_string(),
                    "composite".to_string(),
                ],
            )));
        }

        if options.declaration_map.is_true() && !options.get_emit_declarations() {
            diagnostics.push(Arc::new(Diagnostic::new(
                None,
                TextRange::default(),
                tsox_core::diagnostics::messages_generated::
                    OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                vec![
                    "declarationMap".to_string(),
                    "declaration".to_string(),
                    "composite".to_string(),
                ],
            )));
        }

        if options.isolated_declarations.is_true() {
            if !options.get_emit_declarations() {
                diagnostics.push(Arc::new(Diagnostic::new(
                    None,
                    TextRange::default(),
                    tsox_core::diagnostics::messages_generated::
                        OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                    vec![
                        "isolatedDeclarations".to_string(),
                        "declaration".to_string(),
                        "composite".to_string(),
                    ],
                )));
            }
        }

        if !opts.config.file_names.is_empty() && !options.no_lib.is_true() {
            let lib_names = default_lib_file_names(&options);
            let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
            for lib_name in &lib_names {
                load_lib_recursive(
                    lib_name,
                    host.as_ref(),
                    &mut source_files,
                    &mut by_name,
                    &mut default_lib_names,
                    &mut visited,
                    &mut diagnostics,
                );
            }
        }

        let allow_js = options.get_allow_js();
        for file_name in &opts.config.file_names {
            load_source_file_with_references(
                file_name,
                host.as_ref(),
                &mut source_files,
                &mut by_name,
                &mut diagnostics,
                allow_js,
            );
        }

        {
            let resolution_host: Arc<dyn tsox_tsoptions::module::ResolutionHost + Send + Sync> =
                Arc::new(ResolutionHostAdapter::new(host.as_ref()));
            let resolver = tsox_tsoptions::module::Resolver::new(
                resolution_host,
                Arc::new(options.clone()),
                String::new(),
                String::new(),
            );

            let mut visited: std::collections::HashSet<String> = by_name.keys().cloned().collect();
            let mut stack: Vec<Arc<SourceFile>> = Vec::new();

            let expanded_types: Vec<String> = if options.types.iter().any(|t| t == "*") {
                let (type_roots, _from_config) =
                    tsox_tsoptions::module::resolver::get_effective_type_roots(
                        &options,
                        host.current_directory(),
                    );
                let mut names: Vec<String> = Vec::new();
                for root in &type_roots {
                    for entry in host.fs().get_accessible_entries(root).directories {
                        names.push(entry);
                    }
                }
                names
            } else {
                options.types.clone()
            };
            for type_name in &expanded_types {
                let (resolved, _traces) = resolver.resolve_type_reference_directive(
                    type_name,
                    &config_file_name,
                    tsox_core::core::compiler_options::ModuleKind::None,
                    None,
                );
                if let Some(resolved_tr) = resolved {
                    if resolved_tr.is_resolved() {
                        let resolved_path = resolved_tr.resolved_file_name.as_str();
                        if visited.insert(resolved_path.to_string()) {
                            let pre = source_files.len();
                            load_source_file_with_references(
                                resolved_path,
                                host.as_ref(),
                                &mut source_files,
                                &mut by_name,
                                &mut diagnostics,
                                allow_js,
                            );
                            stack.extend(source_files[pre..].iter().cloned());
                        }
                    }
                }
            }

            stack.extend(source_files.iter().cloned());
            while let Some(file) = stack.pop() {
                let type_refs = extract_reference_types_directives(&file.text);
                for type_ref in &type_refs {
                    let mut mode = tsox_core::core::compiler_options::ModuleKind::None;
                    let mut bad_mode_value = false;
                    match type_ref.mode_value.as_deref() {
                        Some("import") => mode = ModuleKind::ESNext,
                        Some("require") => mode = ModuleKind::CommonJS,
                        Some(_) => bad_mode_value = true,
                        None => {}
                    }
                    if bad_mode_value {
                        diagnostics.push(Arc::new(tsox_frontend::ast::Diagnostic::new(
                            Some(Arc::clone(&file)),

                            TextRange::new(
                                type_ref.types_value_range.0,
                                type_ref.types_value_range.1,
                            ),
                            tsox_core::diagnostics::messages_generated::
                                X_RESOLUTION_MODE_SHOULD_BE_EITHER_REQUIRE_OR_IMPORT,
                            Vec::new(),
                        )));
                    }
                    let (resolved, _traces) = resolver.resolve_type_reference_directive(
                        &type_ref.name,
                        &file.file_name,
                        mode,
                        None,
                    );
                    if let Some(resolved_tr) = resolved {
                        if resolved_tr.is_resolved() {
                            let resolved_path = resolved_tr.resolved_file_name.as_str();
                            if visited.insert(resolved_path.to_string()) {
                                let pre = source_files.len();
                                load_source_file_with_references(
                                    resolved_path,
                                    host.as_ref(),
                                    &mut source_files,
                                    &mut by_name,
                                    &mut diagnostics,
                                    allow_js,
                                );
                                stack.extend(source_files[pre..].iter().cloned());
                            }
                        }
                    }
                }

                // side-effect import（无 import clause）按语句扫描收集
                // specifier 节点 id
                let side_effect_spec_ids: std::collections::HashSet<u64> = {
                    let mut ids = std::collections::HashSet::new();
                    if let tsox_frontend::ast::NodeData::SourceFile(sf) = &file.node.data {
                        for stmt in sf.statements.iter() {
                            if let tsox_frontend::ast::NodeData::ImportDeclaration(d) = &stmt.data {
                                if d.import_clause.is_none() {
                                    ids.insert(d.module_specifier.id());
                                }
                            }
                        }
                    }
                    ids
                };
                for import_node in &file.imports {
                    let module_spec = import_node.text();
                    if module_spec.is_empty() {
                        continue;
                    }
                    // Go processImport 经 getModeForUsageLocation 取模式：
                    // 显式 resolution-mode 覆盖优先，否则用文件默认解析模式
                    let override_mode = import_resolution_mode_override(import_node);
                    let resolution_mode = if matches!(
                        override_mode,
                        tsox_core::core::compiler_options::ModuleKind::None
                    ) {
                        tsox_tsoptions::tsoptions::implied_node_format_of_file(
                            &file.file_name,
                            &|p| host.fs().read_file(p),
                        )
                    } else {
                        override_mode
                    };
                    let (resolved, resolution_diags) = resolver.resolve_module_name(
                        module_spec,
                        &file.file_name,
                        resolution_mode,
                        None,
                    );
                    for d in resolution_diags {
                        diagnostics.push(Arc::new(Diagnostic::new(
                            None,
                            tsox_core::core::text::TextRange::new(0, 0),
                            *d.message,
                            d.args,
                        )));
                    }
                    let is_resolved = resolved.as_ref().map(|m| m.is_resolved()).unwrap_or(false);
                    if is_resolved {
                        let resolved_module = resolved.unwrap();
                        // Go tsc 默认 preserveSymlinks=false：解析结果经 realpath
                        // 规范化回真实路径，符号链接目标与原文件合一
                        let resolved_path = host
                            .fs()
                            .realpath(resolved_module.resolved_file_name.as_str());
                        if visited.insert(resolved_path.clone()) {
                            let pre = source_files.len();
                            load_source_file_with_references(
                                &resolved_path,
                                host.as_ref(),
                                &mut source_files,
                                &mut by_name,
                                &mut diagnostics,
                                allow_js,
                            );
                            stack.extend(source_files[pre..].iter().cloned());
                        }
                    } else if (module_spec.starts_with('.')
                        && !pattern_ambient_module_exists(&source_files, module_spec)
                        && !node_next_needs_extension(
                            &options,
                            &file.file_name,
                            module_spec,
                            &|p| host.fs().read_file(p),
                        ))
                        || (!module_spec.starts_with('.')
                            && !ambient_module_exists(&source_files, module_spec))
                    {
                        // TS2307 报告位：ImportType（含动态 import() 类型位）由
                        // checker 报，这里跳过避免双报
                        let from_import_type = {
                            let mut cur = import_node.parent();
                            let mut hit = false;
                            for _ in 0..4 {
                                match cur {
                                    Some(p) if p.kind == tsox_frontend::ast::SyntaxKind::ImportType => {
                                        hit = true;
                                        break;
                                    }
                                    Some(p) => cur = p.parent(),
                                    None => break,
                                }
                            }
                            hit
                        };
                        if !from_import_type {
                            // Go checkImportDeclaration：side-effect import（无 import
                            // clause）且未显式 noUncheckedSideEffectImports=false 时用
                            // TS2882 专用消息；常规导入经 node 核心模块专用文案选择
                            let is_side_effect = side_effect_spec_ids.contains(&import_node.id());
                            let (message, args): (_, Vec<String>) = if is_side_effect
                                && !options.no_unchecked_side_effect_imports.is_false()
                            {
                                (
                                    tsox_core::diagnostics::messages_generated::
                                        CANNOT_FIND_MODULE_OR_TYPE_DECLARATIONS_FOR_SIDE_EFFECT_IMPORT_OF_0,
                                    vec![module_spec.to_string()],
                                )
                            } else {
                                let (msg, args) =
                                    tsox_frontend::parser::cannot_resolve_module_error(
                                        &options,
                                        &module_spec,
                                    );
                                (*msg, args)
                            };
                            let mut module_not_found = Diagnostic::new(
                                Some(file.clone()),
                                import_node.loc,
                                message,
                                args,
                            );

                            if let Some(alt) =
                                resolved.as_ref().and_then(|m| m.alternate_result.clone())
                            {
                                module_not_found.message_chain = vec![Diagnostic::new(
                                    Some(file.clone()),
                                    import_node.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        THERE_ARE_TYPES_AT_0_BUT_THIS_RESULT_COULD_NOT_BE_RESOLVED_UNDER_YOUR_CURRENT_MODULERESOLUTION_SETTING_CONSIDER_UPDATING_TO_NODE16_NODENEXT_OR_BUNDLER,
                                    vec![alt],
                                )];
                            }
                            diagnostics.push(Arc::new(module_not_found));
                        }
                    }
                }

                use tsox_core::core::compiler_options::JsxEmit;
                if matches!(options.jsx, JsxEmit::ReactJSX | JsxEmit::ReactJSXDev)
                    && (file.file_name.ends_with(".tsx") || file.file_name.ends_with(".jsx"))
                {
                    let source = if options.jsx_import_source.is_empty() {
                        "react"
                    } else {
                        options.jsx_import_source.as_str()
                    };
                    let module_ref = if options.jsx == JsxEmit::ReactJSXDev {
                        format!("{source}/jsx-dev-runtime")
                    } else {
                        format!("{source}/jsx-runtime")
                    };
                    let mode = tsox_tsoptions::tsoptions::implied_node_format_of_file(
                        &file.file_name,
                        &|p| host.fs().read_file(p),
                    );
                    let (resolved, _traces) =
                        resolver.resolve_module_name(&module_ref, &file.file_name, mode, None);
                    if resolved.as_ref().is_some_and(|m| m.is_resolved()) {
                        let resolved_path = resolved.as_ref().unwrap().resolved_file_name.as_str();
                        if visited.insert(resolved_path.to_string()) {
                            load_source_file_with_references(
                                resolved_path,
                                host.as_ref(),
                                &mut source_files,
                                &mut by_name,
                                &mut diagnostics,
                                allow_js,
                            );
                        }
                    }
                }
            }
        }

        for err in &opts.config.errors {
            diagnostics.push(Arc::new(err.clone()));
        }

        let mut binder = Binder::new();
        for file in &source_files {
            binder.bind_source_file(file);
        }
        let symbol_map = std::mem::take(&mut binder.symbol_map);

        Program {
            options,
            source_files,
            source_files_by_name: by_name,
            default_library_file_names: default_lib_names,
            diagnostics,
            host,
            config_file_name,
            symbol_map,
        }
    }

    pub fn options(&self) -> &CompilerOptions {
        &self.options
    }
}
