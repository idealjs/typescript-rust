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

        if options.module_resolution == ModuleResolutionKind::Node10 {
            let mut deprecation = Diagnostic::new(
                None,
                TextRange::default(),
                crate::diagnostics::messages_generated::
                    OPTION_0_1_IS_DEPRECATED_AND_WILL_STOP_FUNCTIONING_IN_TYPESCRIPT_2_SPECIFY_COMPILEROPTION_IGNOREDEPRECATIONS_COLON_3_TO_SILENCE_THIS_ERROR,
                vec![
                    "moduleResolution".to_string(),
                    "node10".to_string(),
                    "7.0".to_string(),
                    "6.0".to_string(),
                ],
            );
            deprecation.message_chain = vec![Diagnostic::new(
                None,
                TextRange::default(),
                crate::diagnostics::messages_generated::
                    VISIT_HTTPS_COLON_SLASH_SLASHAKA_MS_SLASHTS6_FOR_MIGRATION_INFORMATION,
                Vec::new(),
            )];
            diagnostics.push(Arc::new(deprecation));
        }

        if !options.lib.is_empty() && options.no_lib.is_true() {
            diagnostics.push(Arc::new(Diagnostic::new(
                None,
                TextRange::default(),
                crate::diagnostics::messages_generated::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                vec!["lib".to_string(), "noLib".to_string()],
            )));
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
            let resolution_host: Arc<dyn module::ResolutionHost + Send + Sync> =
                Arc::new(ResolutionHostAdapter::new(host.as_ref()));
            let resolver = module::Resolver::new(
                resolution_host,
                Arc::new(options.clone()),
                String::new(),
                String::new(),
            );

            let mut visited: std::collections::HashSet<String> = by_name.keys().cloned().collect();
            let mut stack: Vec<Arc<SourceFile>> = Vec::new();

            let expanded_types: Vec<String> = if options.types.iter().any(|t| t == "*") {
                let (type_roots, _from_config) =
                    module::resolver::get_effective_type_roots(&options, host.current_directory());
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
                    crate::core::compiler_options::ModuleKind::None,
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
                    let mut mode = crate::core::compiler_options::ModuleKind::None;
                    let mut bad_mode_value = false;
                    match type_ref.mode_value.as_deref() {
                        Some("import") => mode = ModuleKind::ESNext,
                        Some("require") => mode = ModuleKind::CommonJS,
                        Some(_) => bad_mode_value = true,
                        None => {}
                    }
                    if bad_mode_value {
                        diagnostics.push(Arc::new(crate::ast::Diagnostic::new(
                            Some(Arc::clone(&file)),

                            TextRange::new(
                                type_ref.types_value_range.0,
                                type_ref.types_value_range.1,
                            ),
                            crate::diagnostics::messages_generated::
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

                for import_node in &file.imports {
                    let module_spec = import_node.text();
                    if module_spec.is_empty() {
                        continue;
                    }
                    let (resolved, _traces) = resolver.resolve_module_name(
                        module_spec,
                        &file.file_name,
                        import_resolution_mode_override(import_node),
                        None,
                    );
                    let is_resolved = resolved.as_ref().map(|m| m.is_resolved()).unwrap_or(false);
                    if is_resolved {
                        let resolved_module = resolved.unwrap();
                        let resolved_path = resolved_module.resolved_file_name.as_str();
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
                    } else if module_spec.starts_with('.')
                        || !ambient_module_exists(&source_files, module_spec)
                    {
                        let mut module_not_found = Diagnostic::new(
                            Some(file.clone()),
                            import_node.loc,
                            crate::diagnostics::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
                            vec![module_spec.to_string()],
                        );

                        if let Some(alt) =
                            resolved.as_ref().and_then(|m| m.alternate_result.clone())
                        {
                            module_not_found.message_chain = vec![Diagnostic::new(
                                Some(file.clone()),
                                import_node.loc,
                                crate::diagnostics::messages_generated::
                                    THERE_ARE_TYPES_AT_0_BUT_THIS_RESULT_COULD_NOT_BE_RESOLVED_UNDER_YOUR_CURRENT_MODULERESOLUTION_SETTING_CONSIDER_UPDATING_TO_NODE16_NODENEXT_OR_BUNDLER,
                                vec![alt],
                            )];
                        }
                        diagnostics.push(Arc::new(module_not_found));
                    }
                }

                use crate::core::compiler_options::JsxEmit;
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
                    let mode =
                        implied_node_format_of_file(&file.file_name, &|p| host.fs().read_file(p));
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
