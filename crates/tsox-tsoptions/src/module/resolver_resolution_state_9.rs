#![allow(unused_imports)]

use crate::module::resolver::*;

impl<'a> ResolutionState<'a> {
    /// Go tryLoadInputFileForPath：exports/imports 目标落在 outDir/declarationDir
    /// 时，把输出路径反查回 rootDir 下的输入源文件（包自名 + outDir 场景）
    pub(crate) fn try_load_input_file_for_path(
        &mut self,
        final_path: &str,
        entry: &str,
        package_directory: &str,
        is_imports: bool,
    ) -> Option<Resolved> { ::tsox_core::fntrace::enter("try_load_input_file_for_path"); 
        if self.is_config_lookup {
            return CONTINUE_SEARCHING;
        }
        if self.compiler_options.declaration_dir.is_empty()
            && self.compiler_options.out_dir.is_empty()
        {
            return CONTINUE_SEARCHING;
        }
        if final_path.contains("/node_modules/") {
            return CONTINUE_SEARCHING;
        }
        if !self.compiler_options.config_file_path.is_empty() {
            let pkg_dir = tsox_core::tspath::Path::from(package_directory);
            let cfg = tsox_core::tspath::Path::from(
                self.compiler_options.config_file_path.clone(),
            );
            if !pkg_dir.contains_path(&cfg) {
                return CONTINUE_SEARCHING;
            }
        }

        let root_dir: String = if !self.compiler_options.root_dir.is_empty() {
            self.compiler_options.root_dir.clone()
        } else if !self.compiler_options.config_file_path.is_empty() {
            tsox_core::tspath::get_directory_path(&self.compiler_options.config_file_path)
        } else {
            // Go 报「project root is ambiguous」解析诊断（文件按宿主 cwd
            // 相对化，与基线一致）
            let pkg_json = tsox_core::tspath::combine_paths(package_directory, &["package.json"]);
            let pkg_json_display = pkg_json
                .strip_prefix(&format!("{}/", self.current_directory))
                .unwrap_or(pkg_json.as_str())
                .to_string();
            let entry_display = if entry.is_empty() { "." } else { entry };
            use tsox_core::diagnostics::messages_generated::{
                THE_PROJECT_ROOT_IS_AMBIGUOUS_BUT_IS_REQUIRED_TO_RESOLVE_EXPORT_MAP_ENTRY_0_IN_FILE_1_SUPPLY_THE_ROOTDIR_COMPILER_OPTION_TO_DISAMBIGUATE,
                THE_PROJECT_ROOT_IS_AMBIGUOUS_BUT_IS_REQUIRED_TO_RESOLVE_IMPORT_MAP_ENTRY_0_IN_FILE_1_SUPPLY_THE_ROOTDIR_COMPILER_OPTION_TO_DISAMBIGUATE,
            };
            let message_static = if is_imports {
                &THE_PROJECT_ROOT_IS_AMBIGUOUS_BUT_IS_REQUIRED_TO_RESOLVE_IMPORT_MAP_ENTRY_0_IN_FILE_1_SUPPLY_THE_ROOTDIR_COMPILER_OPTION_TO_DISAMBIGUATE
            } else {
                &THE_PROJECT_ROOT_IS_AMBIGUOUS_BUT_IS_REQUIRED_TO_RESOLVE_EXPORT_MAP_ENTRY_0_IN_FILE_1_SUPPLY_THE_ROOTDIR_COMPILER_OPTION_TO_DISAMBIGUATE
            };
            self.resolution_diagnostics.push(DiagAndArgs {
                message: message_static,
                args: vec![entry_display.to_string(), pkg_json_display],
            });
            self.unresolved_terminal = true;
            return CONTINUE_SEARCHING;
        };

        let current_dir = if self.compiler_options.config_file_path.is_empty() {
            root_dir.clone()
        } else {
            self.current_directory.to_string()
        };
        let mut candidate_dirs: Vec<String> = Vec::new();
        if !self.compiler_options.declaration_dir.is_empty() {
            candidate_dirs.push(tsox_core::tspath::get_normalized_absolute_path(
                &tsox_core::tspath::combine_paths(
                    &current_dir,
                    &[&self.compiler_options.declaration_dir],
                ),
                self.current_directory,
            ));
        }
        if !self.compiler_options.out_dir.is_empty()
            && self.compiler_options.out_dir != self.compiler_options.declaration_dir
        {
            candidate_dirs.push(tsox_core::tspath::get_normalized_absolute_path(
                &tsox_core::tspath::combine_paths(&current_dir, &[&self.compiler_options.out_dir]),
                self.current_directory,
            ));
        }

        for candidate_dir in &candidate_dirs {
            let dir_path = tsox_core::tspath::Path::from(candidate_dir.clone());
            let final_p = tsox_core::tspath::Path::from(final_path.to_string());
            if !dir_path.contains_path(&final_p) {
                continue;
            }
            let path_fragment = if final_path.len() > candidate_dir.len() {
                &final_path[candidate_dir.len() + 1..]
            } else {
                ""
            };
            // rootDir 可能是相对形式（./ 等），输入基路径按宿主 cwd 归一
            let combined_relative = tsox_core::tspath::combine_paths(&root_dir, &[path_fragment]);
            let possible_input_base = tsox_core::tspath::get_normalized_absolute_path(
                &combined_relative,
                self.current_directory,
            );
            for out_ext in [
                ".mjs", ".cjs", ".js", ".json", ".d.mts", ".d.cts", ".d.ts",
            ] {
                if !tsox_core::tspath::file_extension_is(&possible_input_base, out_ext) {
                    continue;
                }
                let input_exts: Vec<&str> = match out_ext {
                    ".mjs" | ".d.mts" => vec![".mts", ".mjs"],
                    ".cjs" | ".d.cts" => vec![".cts", ".cjs"],
                    _ => vec![".tsx", ".ts", ".jsx", ".js"],
                };
                for in_ext in input_exts {
                    if !Self::extension_is_ok(self.extensions, in_ext) {
                        continue;
                    }
                    let candidate =
                        tsox_core::tspath::change_extension(&possible_input_base, in_ext);
                    if self.fs.file_exists(&candidate) {
                        if let Some(resolved) =
                            self.load_file_name_from_package_json_field(self.extensions, &candidate)
                        {
                            return Some(resolved);
                        }
                    }
                }
            }
        }
        CONTINUE_SEARCHING
    }

    fn extension_is_ok(ext: Extensions, extension: &str) -> bool { ::tsox_core::fntrace::enter("extension_is_ok"); 
        (ext.contains(Extensions::JAVASCRIPT)
            && matches!(extension, ".js" | ".jsx" | ".mjs" | ".cjs"))
            || (ext.contains(Extensions::TYPESCRIPT)
                && matches!(extension, ".ts" | ".tsx" | ".mts" | ".cts"))
            || (ext.contains(Extensions::DECLARATION)
                && matches!(extension, ".d.ts" | ".d.mts" | ".d.cts"))
            || (ext.contains(Extensions::JSON) && extension == ".json")
    }

    fn load_file_name_from_package_json_field_checked(
        &mut self,
        ext: Extensions,
        final_path: &str,
        package_directory: &str,
        _entry: &str,
        is_imports: bool,
    ) -> Option<Resolved> { ::tsox_core::fntrace::enter("load_file_name_from_package_json_field_checked"); 
        if let Some(input_link) =
            self.try_load_input_file_for_path(final_path, _entry, package_directory, is_imports)
        {
            return Some(input_link);
        }
        if self.unresolved_terminal {
            return CONTINUE_SEARCHING;
        }
        self.load_file_name_from_package_json_field(ext, final_path)
    }

    pub(crate) fn load_module_from_exports_or_imports(
        &mut self,
        ext: Extensions,
        module_name: &str,
        lookup_table: &packagejson::JsonValue,
        package_directory: &str,
        is_imports: bool,
    ) -> Option<Resolved> { ::tsox_core::fntrace::enter("load_module_from_exports_or_imports"); 
        let entries = lookup_table.as_object();

        if !module_name.ends_with('/') && !module_name.contains('*') {
            for (key, value) in entries {
                if key == module_name {
                    return self.load_module_from_target_export_or_import(
                        ext,
                        module_name,
                        package_directory,
                        is_imports,
                        value,
                        "",
                        false,
                    );
                }
            }
        }

        let mut expanding_keys: Vec<(&String, &packagejson::JsonValue)> = entries
            .iter()
            .filter(|(k, _)| k.matches('*').count() == 1 || k.ends_with('/'))
            .map(|(k, v)| (k, v))
            .collect();
        expanding_keys.sort_by(|(a, _), (b, _)| crate::module::compare_pattern_keys(a, b));

        for (potential_target, target) in expanding_keys {
            if potential_target.contains('*') {
                let star_pos = potential_target.find('*').unwrap();
                let prefix = &potential_target[..star_pos];
                let suffix = &potential_target[star_pos + 1..];
                if !suffix.is_empty() {
                    if module_name.starts_with(prefix)
                        && module_name.ends_with(suffix)
                        && module_name.len() >= prefix.len() + suffix.len()
                    {
                        let subpath = &module_name[prefix.len()..module_name.len() - suffix.len()];
                        return self.load_module_from_target_export_or_import(
                            ext,
                            module_name,
                            package_directory,
                            is_imports,
                            target,
                            subpath,
                            true,
                        );
                    }
                } else if module_name.starts_with(prefix) {
                    let subpath = &module_name[prefix.len()..];
                    return self.load_module_from_target_export_or_import(
                        ext,
                        module_name,
                        package_directory,
                        is_imports,
                        target,
                        subpath,
                        true,
                    );
                }
            } else if potential_target.ends_with('/')
                && module_name.starts_with(potential_target.as_str())
            {
                let subpath = &module_name[potential_target.len()..];
                return self.load_module_from_target_export_or_import(
                    ext,
                    module_name,
                    package_directory,
                    is_imports,
                    target,
                    subpath,
                    false,
                );
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_module_from_target_export_or_import(
        &mut self,
        ext: Extensions,
        module_name: &str,
        package_directory: &str,
        is_imports: bool,
        target: &packagejson::JsonValue,
        subpath: &str,
        is_pattern: bool,
    ) -> Option<Resolved> { ::tsox_core::fntrace::enter("load_module_from_target_export_or_import"); 
        if self.export_target_depth >= 16 {
            return CONTINUE_SEARCHING;
        }
        match target.value_type {
            packagejson::JsonValueType::String => {
                let target_string = target.as_string();

                if !is_pattern && !subpath.is_empty() && !target_string.ends_with('/') {
                    return CONTINUE_SEARCHING;
                }

                if !is_imports && !target_string.starts_with("./") {
                    return CONTINUE_SEARCHING;
                }

                let parts: Vec<&str> = target_string.split('/').collect();
                if parts
                    .iter()
                    .skip(1)
                    .any(|p| *p == ".." || *p == "node_modules")
                {
                    return CONTINUE_SEARCHING;
                }

                let final_path = if is_pattern {
                    let resolved_target = target_string.replacen('*', subpath, 1);
                    let combined =
                        tsox_core::tspath::combine_paths(package_directory, &[&resolved_target]);
                    tsox_core::tspath::normalize_path(&combined)
                } else if subpath.is_empty() {
                    let combined =
                        tsox_core::tspath::combine_paths(package_directory, &[target_string]);
                    tsox_core::tspath::normalize_path(&combined)
                } else {
                    let combined =
                        tsox_core::tspath::combine_paths(package_directory, &[target_string]);
                    let combined = tsox_core::tspath::combine_paths(&combined, &[subpath]);
                    tsox_core::tspath::normalize_path(&combined)
                };

                self.load_file_name_from_package_json_field_checked(ext, &final_path, package_directory, subpath, is_imports)
            }

            packagejson::JsonValueType::Object => {
                for (condition, sub_target) in target.as_object() {
                    if self.condition_matches(condition) {
                        self.export_target_depth += 1;
                        let result = self.load_module_from_target_export_or_import(
                            ext,
                            module_name,
                            package_directory,
                            is_imports,
                            sub_target,
                            subpath,
                            is_pattern,
                        );
                        self.export_target_depth -= 1;
                        if let Some(result) = result {
                            return Some(result);
                        }
                        if self.unresolved_terminal {
                            return CONTINUE_SEARCHING;
                        }
                    }
                }
                CONTINUE_SEARCHING
            }

            packagejson::JsonValueType::Array => {
                for elem in target.as_array() {
                    self.export_target_depth += 1;
                    let result = self.load_module_from_target_export_or_import(
                        ext,
                        module_name,
                        package_directory,
                        is_imports,
                        elem,
                        subpath,
                        is_pattern,
                    );
                    self.export_target_depth -= 1;
                    if let Some(result) = result {
                        return Some(result);
                    }
                    if self.unresolved_terminal {
                        return CONTINUE_SEARCHING;
                    }
                }
                CONTINUE_SEARCHING
            }

            _ => CONTINUE_SEARCHING,
        }
    }

    pub(crate) fn create_resolved_module(
        &self,
        resolved: Option<Resolved>,
        is_external_library_import: bool,
    ) -> ResolvedModule { ::tsox_core::fntrace::enter("create_resolved_module"); 
        match resolved {
            Some(r) => {
                let package_id =
                    r.package_id.or_else(|| compute_package_id(self.fs, &r.path));
                ResolvedModule {
                    resolved_file_name: r.path,
                    original_path: r.original_path,
                    extension: r.extension,
                    resolved_using_ts_extension: r.resolved_using_ts_extension,
                    is_external_library_import,
                    package_id,
                    ..Default::default()
                }
            }
            None => ResolvedModule::default(),
        }
    }

    // Go createResolvedModuleHandlingSymlink：preserveSymlinks 未开启时，对
    // 非相对名且落在 node_modules 的解析结果按 realpath 回写真实路径，
    // 符号链接路径记入 originalPath；isExternalLibraryImport 仍按 swap 前
    // 的符号链接路径判定（Go 同序）
    pub(crate) fn create_resolved_module_handling_symlink(
        &self,
        resolved: Option<Resolved>,
    ) -> ResolvedModule { ::tsox_core::fntrace::enter("create_resolved_module_handling_symlink"); 
        let mut resolved = resolved;
        let is_external_library_import = resolved
            .as_ref()
            .is_some_and(|r| r.path.contains("/node_modules/"));
        if is_external_library_import
            && self.compiler_options.preserve_symlinks
                != tsox_core::core::tristate::Tristate::True
            && resolved
                .as_ref()
                .is_some_and(|r| r.original_path.is_empty())
            && !tsox_core::tspath::is_external_module_name_relative(&self.name)
        {
            let r = resolved.as_mut().unwrap();
            let (original_path, resolved_file_name) =
                self.get_original_and_resolved_file_name(&r.path);
            if !original_path.is_empty() {
                r.path = resolved_file_name;
                r.original_path = original_path;
            }
        }
        self.create_resolved_module(resolved, is_external_library_import)
    }

    // Go getOriginalAndResolvedFileName：realpath 与原路径仅大小写差异时
    // 保留原路径（forceConsistentCasingInFileNames 报错需要原拼写）
    fn get_original_and_resolved_file_name(&self, file_name: &str) -> (String, String) { ::tsox_core::fntrace::enter("get_original_and_resolved_file_name"); 
        let resolved_file_name =
            tsox_core::tspath::normalize_path(&self.fs.realpath(file_name));
        let compare_paths_options = tsox_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: self.fs.use_case_sensitive_file_names(),
            current_directory: self.current_directory.to_string(),
        };
        if tsox_core::tspath::mig::m3i::compare_paths(
            file_name,
            &resolved_file_name,
            &compare_paths_options,
        ) == 0
        {
            return (String::new(), file_name.to_string());
        }
        (file_name.to_string(), resolved_file_name)
    }
}

pub(crate) fn get_conditions(
    options: &CompilerOptions,
    resolution_mode: ModuleKind,
) -> Vec<String> { ::tsox_core::fntrace::enter("get_conditions"); 
    let mut conditions = Vec::new();
    if resolution_mode == ModuleKind::ESNext {
        conditions.push("import".to_string());
    } else {
        conditions.push("require".to_string());
    }
    if !options.no_dts_resolution.is_true() {
        conditions.push("types".to_string());
    }
    if options.get_module_resolution_kind() != ModuleResolutionKind::Bundler {
        conditions.push("node".to_string());
    }

    for custom in &options.custom_conditions {
        conditions.push(custom.clone());
    }
    conditions
}
