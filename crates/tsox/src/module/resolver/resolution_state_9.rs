#![allow(unused_imports)]

use super::*;

impl<'a> ResolutionState<'a> {
    pub(crate) fn load_module_from_exports_or_imports(
        &mut self,
        ext: Extensions,
        module_name: &str,
        lookup_table: &packagejson::JsonValue,
        package_directory: &str,
        is_imports: bool,
    ) -> Option<Resolved> {
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
    ) -> Option<Resolved> {
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
                    let combined = tspath::combine_paths(package_directory, &[&resolved_target]);
                    tspath::normalize_path(&combined)
                } else if subpath.is_empty() {
                    let combined = tspath::combine_paths(package_directory, &[target_string]);
                    tspath::normalize_path(&combined)
                } else {
                    let combined = tspath::combine_paths(package_directory, &[target_string]);
                    let combined = tspath::combine_paths(&combined, &[subpath]);
                    tspath::normalize_path(&combined)
                };

                self.load_file_name_from_package_json_field(ext, &final_path)
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
                }
                CONTINUE_SEARCHING
            }

            _ => CONTINUE_SEARCHING,
        }
    }

    pub(crate) fn create_resolved_module(&self, resolved: Option<Resolved>) -> ResolvedModule {
        match resolved {
            Some(r) => {
                let is_external = r.path.contains("/node_modules/");
                ResolvedModule {
                    resolved_file_name: r.path,
                    original_path: r.original_path,
                    extension: r.extension,
                    resolved_using_ts_extension: r.resolved_using_ts_extension,
                    is_external_library_import: is_external,
                    package_id: r.package_id,
                    ..Default::default()
                }
            }
            None => ResolvedModule::default(),
        }
    }
}

pub(crate) fn get_conditions(options: &CompilerOptions, resolution_mode: ModuleKind) -> Vec<String> {
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
