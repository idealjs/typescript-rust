#![allow(unused_imports)]

use super::*;

impl Program {
    pub fn new(opts: ProgramOptions) -> Arc<Self> { ::tsox_core::fntrace::enter("new"); 
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

        let resolution_host: Arc<dyn tsox_tsoptions::module::ResolutionHost + Send + Sync> =
            Arc::new(ResolutionHostAdapter::new(host.as_ref()));
        let resolver = tsox_tsoptions::module::Resolver::new(
            resolution_host,
            Arc::new(options.clone()),
            String::new(),
            String::new(),
        );

        if !opts.config.file_names.is_empty() && !options.no_lib.is_true() {
            let lib_names = default_lib_file_names(&options);
            let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
            for lib_name in &lib_names {
                load_lib_recursive(
                    lib_name,
                    host.as_ref(),
                    &options,
                    &resolver,
                    &mut source_files,
                    &mut by_name,
                    &mut default_lib_names,
                    &mut visited,
                    &mut diagnostics,
                );
            }
        }

        let allow_js = options.get_allow_js();
        let mut resolved_modules: HashMap<
            String,
            Vec<(String, Option<tsox_tsoptions::module::ResolvedModule>)>,
        > = HashMap::new();
        for file_name in &opts.config.file_names {
            if root_file_unsupported_extension_diagnostic(
                file_name,
                &options,
                allow_js,
                host.use_case_sensitive_file_names(),
                &mut diagnostics,
            ) {
                continue;
            }
            load_source_file_with_references(
                file_name,
                host.as_ref(),
                &mut source_files,
                &mut by_name,
                &mut diagnostics,
                allow_js,
            );
        }

        let mut source_files_found_searching_node_modules: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        {
            let resolver = &resolver;

            let mut visited: std::collections::HashSet<String> = by_name.keys().cloned().collect();
            let mut pid_first_path: HashMap<tsox_tsoptions::module::PackageId, String> =
                HashMap::new();
            package_dedupe::prewalk_package_first_paths(
                &resolver,
                host.as_ref(),
                &source_files,
                &mut pid_first_path,
            );
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
            let containing_directory = if !options.config_file_path.is_empty() {
                tsox_core::tspath::get_directory_path(&options.config_file_path)
            } else {
                host.current_directory().to_string()
            };
            let inferred_types_containing_file = tsox_core::tspath::combine_paths(
                &containing_directory,
                &[tsox_tsoptions::module::INFERRED_TYPES_CONTAINING_FILE],
            );
            for type_name in &expanded_types {
                let (resolved, _traces) = resolver.resolve_type_reference_directive(
                    type_name,
                    &inferred_types_containing_file,
                    tsox_core::core::compiler_options::ModuleKind::None,
                    None,
                );
                let type_entry_resolved = resolved
                    .as_ref()
                    .is_some_and(|tr| tr.is_resolved());
                if !type_entry_resolved {
                    let mut diag = tsox_frontend::ast::Diagnostic::new(
                        None,
                        TextRange::default(),
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_FIND_TYPE_DEFINITION_FILE_FOR_0,
                        vec![type_name.clone()],
                    );
                    let mut reason = tsox_frontend::ast::Diagnostic::new(
                        None,
                        TextRange::default(),
                        tsox_core::diagnostics::messages_generated::
                            THE_FILE_IS_IN_THE_PROGRAM_BECAUSE_COLON,
                        Vec::new(),
                    );
                    reason.message_chain = vec![tsox_frontend::ast::Diagnostic::new(
                        None,
                        TextRange::default(),
                        tsox_core::diagnostics::messages_generated::
                            ENTRY_POINT_OF_TYPE_LIBRARY_0_SPECIFIED_IN_COMPILEROPTIONS,
                        vec![type_name.clone()],
                    )];
                    diag.message_chain = vec![reason];
                    diagnostics.push(Arc::new(diag));
                }
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
                    let type_ref_resolved = resolved
                        .as_ref()
                        .is_some_and(|tr| tr.is_resolved());
                    if !type_ref_resolved {
                        diagnostics.push(Arc::new(tsox_frontend::ast::Diagnostic::new(
                            Some(Arc::clone(&file)),
                            TextRange::new(
                                type_ref.types_value_range.0,
                                type_ref.types_value_range.1,
                            ),
                            tsox_core::diagnostics::messages_generated::
                                CANNOT_FIND_TYPE_DEFINITION_FILE_FOR_0,
                            vec![type_ref.name.clone()],
                        )));
                    }
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
                let mut resolution_diag_keys: std::collections::HashSet<
                    (String, tsox_core::core::compiler_options::ModuleKind),
                > = std::collections::HashSet::new();
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
                    resolved_modules
                        .entry(file.file_name.clone())
                        .or_default()
                        .push((module_spec.to_string(), resolved.clone()));
                    if resolution_diag_keys.insert((module_spec.to_string(), resolution_mode)) {
                        for d in resolution_diags {
                            diagnostics.push(Arc::new(Diagnostic::new(
                                None,
                                tsox_core::core::text::TextRange::new(0, 0),
                                *d.message,
                                d.args,
                            )));
                        }
                    }
                    let is_resolved = resolved.as_ref().map(|m| m.is_resolved()).unwrap_or(false);
                    let lib_diagnostics_skipped = options.skip_lib_check.is_true()
                        && (file.is_declaration_file
                            || is_external_library_file(&file.file_name));
                    if is_resolved {
                        let resolved_module = resolved.unwrap();
                        // Go 程序层不对解析结果做 realpath（fileloader/filesparser 无
                        // Realpath）；node_modules 符号链接的真实路径化由解析器
                        // createResolvedModuleHandlingSymlink（resolver.go:1193-1206）
                        // 按 isExternalLibraryImport 完成，此处直接取 resolved_file_name
                        let resolved_path = resolved_module.resolved_file_name.clone();
                        // Go fileloader shouldAddFile（fileloader.go:926-937）：解析
                        // 诊断非空或 JS 文件未开 allowJs 时不入程序。file.imports
                        // 均为真实 import/export/require 节点，Go 的 JSDoc-import
                        // 排除项在此恒为真，不单列
                        let is_js_file = !resolved_module.resolved_using_extra_extensions
                            && !tsox_core::tspath::file_extension_is_one_of(
                                &resolved_path,
                                crate::mig::m4v_2::SUPPORTED_TS_EXTENSIONS_WITH_JSON_FL,
                            );
                        let should_add_file = tsox_tsoptions::module::mig::m3i::get_resolution_diagnostic(
                            &options,
                            &resolved_module,
                            &file,
                        )
                        .is_none()
                            && !options.no_resolve.is_true()
                            && !(is_js_file && !allow_js);
                        if !should_add_file {
                            continue;
                        }
                        if resolved_module.is_external_library_import {
                            source_files_found_searching_node_modules
                                .insert(tsox_core::tspath::normalize_path(&resolved_path));
                        }
                        let first_path = resolved_module
                            .package_id
                            .as_ref()
                            .and_then(|pid| pid_first_path.get(pid))
                            .cloned();
                        if first_path.as_deref() != Some(resolved_path.as_str()) {
                            let target = first_path.as_deref().and_then(|first| {
                                let key = tsox_core::tspath::normalize_path(first);
                                by_name
                                    .get(&key)
                                    .or_else(|| by_name.get(first))
                                    .cloned()
                                    .or_else(|| {
                                        load_source_file_with_references(
                                            first,
                                            host.as_ref(),
                                            &mut source_files,
                                            &mut by_name,
                                            &mut diagnostics,
                                            allow_js,
                                        );
                                        by_name
                                            .get(&key)
                                            .or_else(|| by_name.get(first))
                                            .cloned()
                                    })
                            });
                            if let Some(existing) = target {
                                visited.insert(resolved_path.clone());
                                let normalized =
                                    tsox_core::tspath::normalize_path(&resolved_path);
                                by_name.insert(normalized, Arc::clone(&existing));
                                by_name.insert(resolved_path, existing);
                                continue;
                            }
                        }
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
                    } else if ((module_spec.starts_with('.')
                        && !pattern_ambient_module_exists(&source_files, module_spec)
                        && !node_next_needs_extension(
                            &options,
                            &file.file_name,
                            module_spec,
                            &|p| host.fs().read_file(p),
                        ))
                        || (!module_spec.starts_with('.')
                            && !ambient_module_exists(&source_files, module_spec)
                            && !tsox_frontend::ast::pattern_ambient_module_with_attributes_exists(
                                &source_files,
                                module_spec,
                                import_node
                                    .parent()
                                    .as_ref()
                                    .and_then(tsox_frontend::ast::import_attributes_of_declaration)
                                    .as_ref(),
                            )))
                        && !lib_diagnostics_skipped
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
                            if super::program_directive_filter::suppressed_by_preceding_directive(
                                &file,
                                import_node.loc.pos(),
                            ) {
                                continue;
                            }
                            // Go checkImportDeclaration：side-effect import（无 import
                            // clause）且未显式 noUncheckedSideEffectImports=false 时用
                            // TS2882 专用消息；常规导入经 node 核心模块专用文案选择
                            let is_side_effect = side_effect_spec_ids.contains(&import_node.id());
                            // Go checkImportDeclaration：显式
                            // noUncheckedSideEffectImports=false 时副作用导入
                            // 完全不解析、不报错
                            if is_side_effect
                                && options.no_unchecked_side_effect_imports.is_false()
                            {
                                continue;
                            }
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
                        if resolved
                            .as_ref()
                            .is_some_and(|m| m.is_external_library_import)
                        {
                            source_files_found_searching_node_modules.insert(
                                tsox_core::tspath::normalize_path(resolved_path),
                            );
                        }
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

        apply_module_detection_force(&options, host.as_ref(), &source_files);

        let mut binder = Binder::new();
        for file in &source_files {
            binder.bind_source_file(file);
        }
        let symbol_map = std::mem::take(&mut binder.symbol_map);

        let compare_paths_options = tsox_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
            current_directory: host.current_directory().to_string(),
        };
        let program_tracing = opts.tracing.clone();
        let program_opts = ProgramOptions {
            config: opts.config.clone(),
            host: Arc::clone(&host),
            use_source_of_project_reference: opts.use_source_of_project_reference,
            single_threaded: opts.single_threaded,
            create_checker_pool: opts.create_checker_pool,
            typings_location: opts.typings_location,
            project_name: opts.project_name,
            tracing: program_tracing.clone(),
            skip_module_resolution: opts.skip_module_resolution,
        };

        let program = Arc::new(Program {
            options,
            files: source_files.clone(),
            finished_processing: true,
            source_files,
            source_files_by_name: by_name,
            default_library_file_names: default_lib_names,
            diagnostics,
            host,
            config_file_name,
            symbol_map,
            opts: program_opts.clone(),
            resolver: Arc::new(resolver),
            checker_pool: std::sync::OnceLock::new(),
            compiler_checker_pool: std::sync::OnceLock::new(),
            compare_paths_options,
            files_by_path: HashMap::new(),
            project_reference_file_mapper: Arc::new(
                crate::mig::m4x_2::ProjectReferenceFileMapper {
                    opts: program_opts.clone(),
                    host: None,
                    loader: None,
                    config_to_project_reference: HashMap::new(),
                    references_in_config_file: HashMap::new(),
                    source_to_project_reference: HashMap::new(),
                    output_dts_to_project_reference: HashMap::new(),
                    realpath_dts_to_source: Default::default(),
                },
            ),
            missing_files: Vec::new(),
            resolved_modules: HashMap::new(),
            type_resolutions_in_file: HashMap::new(),
            source_file_meta_datas: HashMap::new(),
            jsx_runtime_import_specifiers: HashMap::new(),
            import_helpers_import_specifiers: HashMap::new(),
            lib_files: HashMap::new(),
            source_files_found_searching_node_modules,
            include_processor: None,
            output_file_to_project_reference_source: HashMap::new(),
            redirect_targets_map: HashMap::new(),
            redirect_files_by_path: HashMap::new(),
            content_mapper_diagnostics: Vec::new(),
            program_diagnostics: Vec::new(),
            content_mapper_option_diagnostics: Vec::new(),
            has_emit_blocking_diagnostics: std::collections::HashSet::new(),
            unresolved_imports: crate::mig::m4w_5::LazyValue::default(),
            known_symlinks: crate::mig::m4w_5::LazyValue::default(),
            package_names: crate::mig::m4w_5::LazyValue::default(),
            has_ts_file_once: crate::mig::m4w_5::LazyValue::default(),
            uses_uri_style_node_core_modules: Tristate::Unknown,
            typings_location: program_opts.typings_location.clone(),
            use_source_of_project_reference: program_opts.use_source_of_project_reference,
            tracing: program_tracing,
        });
        let mut program = program;
        Arc::get_mut(&mut program)
            .expect("program is uniquely owned before checker pool init")
            .verify_compiler_options();
        program.init_checker_pool();
        program
    }

    pub fn options(&self) -> &CompilerOptions { ::tsox_core::fntrace::enter("options"); 
        &self.options
    }

    pub fn block_emitting_of_file(&mut self, emit_file_name: &str, diag: Arc<Diagnostic>) { ::tsox_core::fntrace::enter("block_emitting_of_file"); 
        self.has_emit_blocking_diagnostics
            .insert(self.to_path(emit_file_name).0);
        self.program_diagnostics.push(diag);
    }
}

pub fn new_program(opts: ProgramOptions) -> Arc<Program> { ::tsox_core::fntrace::enter("new_program"); 
    Program::new(opts)
}


fn apply_module_detection_force(
    options: &CompilerOptions,
    host: &dyn CompilerHost,
    files: &[Arc<SourceFile>],
) { ::tsox_core::fntrace::enter("apply_module_detection_force"); 
    use tsox_core::core::compiler_options::ModuleDetectionKind;
    let detection = options.get_emit_module_detection_kind();
    if !matches!(
        detection,
        ModuleDetectionKind::Force | ModuleDetectionKind::Auto
    ) {
        return;
    }
    for file in files {
        if file.external_module_indicator.is_some()
            || file.is_declaration_file
            || file.script_kind == ScriptKind::Json
        {
            continue;
        }
        let lower = file.file_name.to_ascii_lowercase();
        let forced_by_extension = lower.ends_with(".cjs")
            || lower.ends_with(".cts")
            || lower.ends_with(".mjs")
            || lower.ends_with(".mts");
        let forced = match detection {
            ModuleDetectionKind::Force => true,
            _ => {
                forced_by_extension
                    || tsox_tsoptions::tsoptions::implied_node_format_of_file(
                        &file.file_name,
                        &|p| host.fs().read_file(p),
                    ) == ModuleKind::ESNext
            }
        };
        if forced {
            let ptr = Arc::as_ptr(file) as *mut SourceFile;
            unsafe {
                (*ptr).external_module_indicator = Some(Arc::clone(&file.node));
            }
        }
    }
}

// Go fileloader isSupportedExtension + GetSupportedExtensions（tsconfigparsing.go:2095）：
// TS 扩展恒支持，JS 扩展仅 allowJs 时支持，resolveJsonModule 追加 .json
fn is_supported_root_extension(canonical_file_name: &str, allow_js: bool, resolve_json_module: bool) -> bool { ::tsox_core::fntrace::enter("is_supported_root_extension"); 
    let ts_groups: [&[&str]; 3] = [
        &[".ts", ".tsx", ".d.ts"],
        &[".cts", ".d.cts"],
        &[".mts", ".d.mts"],
    ];
    let all_groups: [&[&str]; 3] = [
        &[".ts", ".tsx", ".d.ts", ".js", ".jsx"],
        &[".cts", ".d.cts", ".cjs"],
        &[".mts", ".d.mts", ".mjs"],
    ];
    let groups: &[&[&str]] = if allow_js { &all_groups } else { &ts_groups };
    if groups
        .iter()
        .any(|g| g.iter().any(|e| tsox_core::tspath::file_extension_is(canonical_file_name, e)))
    {
        return true;
    }
    resolve_json_module && tsox_core::tspath::file_extension_is(canonical_file_name, ".json")
}

// Go filesparser findSourceFile（filesparser.go:79-104）：allowNonTsExtensions
// 未开且扩展不受支持时不解析该根文件，记 processing 诊断（TS6504/TS6054，
// 挂 Root file specified for compilation 理由链）。
fn root_file_unsupported_extension_diagnostic(
    file_name: &str,
    options: &CompilerOptions,
    allow_js: bool,
    use_case_sensitive_file_names: bool,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
) -> bool { ::tsox_core::fntrace::enter("root_file_unsupported_extension_diagnostic");
    if !tsox_core::tspath::has_extension(file_name) || options.allow_non_ts_extensions.is_true() {
        return false;
    }
    let canonical = tsox_core::tspath::get_canonical_file_name(file_name, use_case_sensitive_file_names);
    if is_supported_root_extension(&canonical, allow_js, options.resolve_json_module.is_true()) {
        return false;
    }
    let is_js = tsox_core::tspath::has_js_file_extension(&canonical);
    let (message, args) = if is_js {
        (
            tsox_core::diagnostics::messages_generated::
                FILE_0_IS_A_JAVASCRIPT_FILE_DID_YOU_MEAN_TO_ENABLE_THE_ALLOWJS_OPTION,
            vec![file_name.to_string()],
        )
    } else {
        let ts_groups: [&[&str]; 3] = [
            &[".ts", ".tsx", ".d.ts"],
            &[".cts", ".d.cts"],
            &[".mts", ".d.mts"],
        ];
        let all_groups: [&[&str]; 3] = [
            &[".ts", ".tsx", ".d.ts", ".js", ".jsx"],
            &[".cts", ".d.cts", ".cjs"],
            &[".mts", ".d.mts", ".mjs"],
        ];
        let flat: Vec<&str> = (if allow_js { &all_groups } else { &ts_groups })
            .iter()
            .flat_map(|g| g.iter().copied())
            .collect();
        (
            tsox_core::diagnostics::messages_generated::
                FILE_0_HAS_AN_UNSUPPORTED_EXTENSION_THE_ONLY_SUPPORTED_EXTENSIONS_ARE_1,
            vec![file_name.to_string(), format!("'{}'", flat.join("', '"))],
        )
    };
    let mut reason = Diagnostic::new(
        None,
        TextRange::default(),
        tsox_core::diagnostics::messages_generated::THE_FILE_IS_IN_THE_PROGRAM_BECAUSE_COLON,
        Vec::new(),
    );
    reason.message_chain = vec![Diagnostic::new(
        None,
        TextRange::default(),
        tsox_core::diagnostics::messages_generated::ROOT_FILE_SPECIFIED_FOR_COMPILATION,
        Vec::new(),
    )];
    let mut diag = Diagnostic::new(None, TextRange::default(), message, args);
    diag.message_chain = vec![reason];
    diagnostics.push(Arc::new(diag));
    true
}
