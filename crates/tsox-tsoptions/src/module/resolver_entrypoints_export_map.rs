#![allow(unused_imports)]

use crate::module::mig::m3i::{
    is_applicable_versioned_types_key, try_get_js_extension_for_file,
};
use crate::module::resolver::*;
use crate::modulespecifiers::mig::m3m_3::{Ending, ResolvedEntrypoint};
use crate::packagejson::mig::x12::InfoCacheEntry;
use crate::packagejson::{ExportsOrImports, JsonValueType};
use crate::vfs::vfsmatch_unlimited_depth::{read_directory, UNLIMITED_DEPTH};
use std::sync::Arc;
use tsox_core::stringutil::mig::m3m::has_prefix_and_suffix_without_overlap;
use tsox_core::tspath;
use tsox_core::tspath::mig::m3i::change_full_extension;

impl<'a> ResolutionState<'a> {
    pub(crate) fn load_entrypoints_from_export_map(
        &self,
        resolver: &Resolver,
        package_json: &InfoCacheEntry,
        package_name: &str,
        exports: &ExportsOrImports,
    ) -> Vec<Arc<ResolvedEntrypoint>> {
        let mut entrypoints: Vec<Arc<ResolvedEntrypoint>> = Vec::new();
        match exports.json_value.value_type {
            JsonValueType::Array => {
                for element in exports.json_value.as_array() {
                    self.load_entrypoints_from_target_exports(
                        resolver,
                        package_json,
                        package_name,
                        ".",
                        &Vec::new(),
                        &Vec::new(),
                        &ExportsOrImports::from_json_value(element.clone()),
                        &mut entrypoints,
                    );
                }
            }
            JsonValueType::Object => {
                if exports.is_subpaths() {
                    for (subpath, export) in exports.json_value.as_object() {
                        self.load_entrypoints_from_target_exports(
                            resolver,
                            package_json,
                            package_name,
                            subpath,
                            &Vec::new(),
                            &Vec::new(),
                            &ExportsOrImports::from_json_value(export.clone()),
                            &mut entrypoints,
                        );
                    }
                } else {
                    self.load_entrypoints_from_target_exports(
                        resolver,
                        package_json,
                        package_name,
                        ".",
                        &Vec::new(),
                        &Vec::new(),
                        exports,
                        &mut entrypoints,
                    );
                }
            }
            _ => {
                self.load_entrypoints_from_target_exports(
                    resolver,
                    package_json,
                    package_name,
                    ".",
                    &Vec::new(),
                    &Vec::new(),
                    exports,
                    &mut entrypoints,
                );
            }
        }
        entrypoints
    }

    fn load_entrypoints_from_target_exports(
        &self,
        resolver: &Resolver,
        package_json: &InfoCacheEntry,
        package_name: &str,
        subpath: &str,
        include_conditions: &[String],
        exclude_conditions: &[String],
        exports: &ExportsOrImports,
        entrypoints: &mut Vec<Arc<ResolvedEntrypoint>>,
    ) {
        match exports.json_value.value_type {
            JsonValueType::String => {
                let s = exports.json_value.as_string();
                if !s.starts_with("./") {
                    return;
                }
                if s.contains('*') {
                    self.load_entrypoints_from_pattern_target(
                        resolver,
                        package_json,
                        package_name,
                        subpath,
                        include_conditions,
                        exclude_conditions,
                        s,
                        entrypoints,
                    );
                } else {
                    self.load_entrypoints_from_file_target(
                        resolver,
                        package_json,
                        package_name,
                        subpath,
                        include_conditions,
                        exclude_conditions,
                        s,
                        entrypoints,
                    );
                }
            }
            JsonValueType::Array => {
                for element in exports.json_value.as_array() {
                    self.load_entrypoints_from_target_exports(
                        resolver,
                        package_json,
                        package_name,
                        subpath,
                        include_conditions,
                        exclude_conditions,
                        &ExportsOrImports::from_json_value(element.clone()),
                        entrypoints,
                    );
                }
            }
            JsonValueType::Object => {
                let mut prev_conditions: Vec<&str> = Vec::new();
                for (condition, export) in exports.json_value.as_object() {
                    if exclude_conditions.iter().any(|c| c == condition) {
                        continue;
                    }

                    let condition_always_matches = condition == "default"
                        || condition == "types"
                        || is_applicable_versioned_types_key(condition);
                    let (mut new_include_conditions, mut new_exclude_conditions) =
                        (include_conditions.to_vec(), exclude_conditions.to_vec());
                    if !condition_always_matches {
                        new_include_conditions.push(condition.to_string());
                        for prev in &prev_conditions {
                            new_exclude_conditions.push((*prev).to_string());
                        }
                    }

                    prev_conditions.push(condition);
                    self.load_entrypoints_from_target_exports(
                        resolver,
                        package_json,
                        package_name,
                        subpath,
                        &new_include_conditions,
                        &new_exclude_conditions,
                        &ExportsOrImports::from_json_value(export.clone()),
                        entrypoints,
                    );
                    if condition_always_matches {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn load_entrypoints_from_pattern_target(
        &self,
        resolver: &Resolver,
        package_json: &InfoCacheEntry,
        package_name: &str,
        subpath: &str,
        include_conditions: &[String],
        exclude_conditions: &[String],
        s: &str,
        entrypoints: &mut Vec<Arc<ResolvedEntrypoint>>,
    ) {
        if s.matches('*').count() != 1 {
            return;
        }
        let pattern_path = tspath::resolve_path(&package_json.package_directory, &[s]);
        let Some((leading_slice, trailing_slice)) = pattern_path.split_once('*') else {
            return;
        };
        let case_sensitive = resolver.host.fs().use_case_sensitive_file_names();
        let change = change_full_extension(&s.replacen('*', "**/*", 1), ".*");
        let includes = [change.as_str()];
        let exts = self.extensions.array();
        let files = read_directory(
            resolver.host.fs(),
            resolver.host.get_current_directory(),
            &package_json.package_directory,
            &exts,
            &[],
            &includes,
            UNLIMITED_DEPTH,
        );
        for file in files {
            let Some(matched_star) = self.get_matched_star_for_pattern_entrypoint(
                resolver,
                &file,
                leading_slice,
                trailing_slice,
                case_sensitive,
            ) else {
                continue;
            };
            let module_specifier =
                tspath::resolve_path(package_name, &[subpath.replacen('*', &matched_star, 1).as_str()]);
            let ending = if s.ends_with('*') {
                Ending::ExtensionChangeable
            } else {
                Ending::Fixed
            };
            entrypoints.push(Arc::new(resolver.create_resolved_entrypoint_handling_symlink(
                &file,
                &module_specifier,
                include_conditions.to_vec(),
                exclude_conditions.to_vec(),
                ending,
            )));
        }
    }

    fn load_entrypoints_from_file_target(
        &self,
        resolver: &Resolver,
        package_json: &InfoCacheEntry,
        package_name: &str,
        subpath: &str,
        include_conditions: &[String],
        exclude_conditions: &[String],
        s: &str,
        entrypoints: &mut Vec<Arc<ResolvedEntrypoint>>,
    ) {
        let components = tspath::get_path_components(s, "");
        let escapes = components[2..].iter().any(|p| p == ".." || p == "." || p == "node_modules");
        if escapes {
            return;
        }
        let resolved_target = tspath::resolve_path(&package_json.package_directory, &[s]);
        if let Some(result) =
            self.load_file_name_from_package_json_field(self.extensions, &resolved_target)
        {
            if result.is_resolved() {
                let module_specifier = tspath::resolve_path(package_name, &[subpath]);
                let ending = if s.ends_with('*') {
                    Ending::ExtensionChangeable
                } else {
                    Ending::Fixed
                };
                entrypoints.push(Arc::new(resolver.create_resolved_entrypoint_handling_symlink(
                    &result.path,
                    &module_specifier,
                    include_conditions.to_vec(),
                    exclude_conditions.to_vec(),
                    ending,
                )));
            }
        }
    }

    fn get_matched_star_for_pattern_entrypoint(
        &self,
        resolver: &Resolver,
        file: &str,
        leading_slice: &str,
        trailing_slice: &str,
        case_sensitive: bool,
    ) -> Option<String> {
        if has_prefix_and_suffix_without_overlap(file, leading_slice, trailing_slice, case_sensitive) {
            return Some(file[leading_slice.len()..file.len() - trailing_slice.len()].to_string());
        }

        let js_extension = try_get_js_extension_for_file(file, self.compiler_options);
        if !js_extension.is_empty() {
            let swapped = change_full_extension(file, js_extension);
            if has_prefix_and_suffix_without_overlap(
                &swapped,
                leading_slice,
                trailing_slice,
                case_sensitive,
            ) {
                return Some(
                    swapped[leading_slice.len()..swapped.len() - trailing_slice.len()].to_string(),
                );
            }
        }

        None
    }
}
