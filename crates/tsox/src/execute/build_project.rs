#![allow(unused_imports)]

use super::*;

pub(crate) fn build_project(
    sys: &dyn System,
    project: &str,
    compiler_options: &CompilerOptions,
    build_options: &BuildOptions,
    pretty: bool,
    locale: Option<&Locale>,
    seen_projects: &mut HashSet<String>,
    building: &mut HashSet<String>,
    cycle_stack: &mut Vec<String>,
) -> CommandLineResult {
    let config_file_name = match resolve_project_config(sys, project) {
        Ok(config) => config,
        Err(diag) => {
            let mut writer = sys.writer();
            let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty, locale));
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
            };
        }
    };

    let normalized_config = tspath::normalize_path(&config_file_name);

    if seen_projects.contains(&normalized_config) {
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    if building.contains(&normalized_config) {
        let mut writer = sys.writer();
        let diag = compiler_diagnostic(
            PROJECT_REFERENCES_MAY_NOT_FORM_A_CIRCULAR_GRAPH_CYCLE_DETECTED_COLON_0,
            vec![cycle_stack.join("\n")],
        );
        let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty, locale));
        return CommandLineResult {
            status: ExitStatus::ProjectReferenceCycle_OutputsSkipped,
        };
    }

    building.insert(normalized_config.clone());
    cycle_stack.push(normalized_config.clone());

    let config = get_parsed_command_line_of_config_file(
        &normalized_config,
        compiler_options,
        sys.current_directory(),
        sys.fs().as_ref(),
    );
    if !config.errors.is_empty() {
        let mut writer = sys.writer();
        for e in &config.errors {
            let _ = writeln!(writer, "{}", format_diagnostic(e, pretty, locale));
        }
        cycle_stack.pop();
        building.remove(&normalized_config);
        seen_projects.insert(normalized_config);
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let mut status = ExitStatus::Success;
    for reference in resolve_project_references(&config) {
        let result = build_project(
            sys,
            &reference,
            compiler_options,
            build_options,
            pretty,
            locale,
            seen_projects,
            building,
            cycle_stack,
        );
        status = status.max(result.status);
    }

    if status < ExitStatus::ProjectReferenceCycle_OutputsSkipped && !config.file_names.is_empty() {
        let ts_build_info_file = BuildInfo::get_ts_build_info_file_path(
            &normalized_config,
            &config.compiler_options.out_dir,
            &config.compiler_options.ts_build_info_file,
        );

        let fs = sys.fs();

        let files_with_content: Vec<(String, String)> = config
            .file_names
            .iter()
            .filter_map(|f| fs.read_file(f).map(|content| (f.clone(), content)))
            .collect();

        let options_hash = compute_options_signature(&config.compiler_options);

        let force = build_options.force.is_true();
        if !build_options.clean.is_true() && !force {
            if let Some(json) = fs.read_file(&ts_build_info_file) {
                if let Ok(build_info) = serde_json::from_str::<BuildInfo>(&json) {
                    if build_info.is_up_to_date(&files_with_content, &options_hash) {
                        if build_options.verbose.is_true() {
                            let mut writer = sys.writer();
                            let _ =
                                writeln!(writer, "Project '{}' is up to date.", normalized_config);
                        }
                        cycle_stack.pop();
                        building.remove(&normalized_config);
                        seen_projects.insert(normalized_config);
                        return CommandLineResult {
                            status: ExitStatus::Success,
                        };
                    }
                }
            }
        }

        if build_options.verbose.is_true() {
            let mut writer = sys.writer();
            let _ = writeln!(writer, "Project '{}' is being built.", normalized_config);
        }

        if build_options.dry.is_true() {
            cycle_stack.pop();
            building.remove(&normalized_config);
            seen_projects.insert(normalized_config);
            return CommandLineResult {
                status: ExitStatus::Success,
            };
        }

        let result = perform_compilation(sys, config, pretty, locale);
        status = status.max(result.status);

        if result.status == ExitStatus::Success && !build_options.clean.is_true() {
            let build_info =
                BuildInfo::new(&files_with_content, &normalized_config, &options_hash, &[]);
            if let Ok(json) = serde_json::to_string(&build_info) {
                if let Some(parent) = std::path::Path::new(&ts_build_info_file).parent() {
                    if !parent.as_os_str().is_empty() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                }
                let _ = fs.write_file(&ts_build_info_file, &json);
            }
        }
    }

    cycle_stack.pop();
    building.remove(&normalized_config);
    seen_projects.insert(normalized_config);

    CommandLineResult { status }
}

pub(crate) fn resolve_project_config(
    sys: &dyn System,
    project: &str,
) -> Result<String, Diagnostic> {
    if sys.fs().directory_exists(project) {
        let config = tspath::combine_paths(project, &["tsconfig.json"]);
        if !sys.fs().file_exists(&config) {
            return Err(compiler_diagnostic(
                CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0,
                vec![config],
            ));
        }
        Ok(config)
    } else if sys.fs().file_exists(project) {
        Ok(project.to_string())
    } else {
        Err(compiler_diagnostic(
            CANNOT_READ_FILE_0,
            vec![project.to_string()],
        ))
    }
}

pub(crate) fn resolve_project_references(config: &ParsedCommandLine) -> Vec<String> {
    let config_dir = tspath::get_directory_path(&config.config_file_name);
    config
        .references
        .iter()
        .map(|reference| {
            resolve_config_file_name_of_project_reference(&config_dir, &reference.path)
        })
        .collect()
}

pub(crate) fn resolve_config_file_name_of_project_reference(
    config_dir: &str,
    path: &str,
) -> String {
    let resolved = tspath::get_normalized_absolute_path(path, config_dir);
    if tspath::file_extension_is(&resolved, ".json") {
        resolved
    } else {
        tspath::combine_paths(&resolved, &["tsconfig.json"])
    }
}
