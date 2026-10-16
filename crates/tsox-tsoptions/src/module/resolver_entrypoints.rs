#![allow(unused_imports)]

use crate::module::resolver::*;
use crate::modulespecifiers::mig::m3m_3::{Ending, ResolvedEntrypoint};
use crate::packagejson::mig::x12::InfoCacheEntry;
use crate::vfs::vfsmatch_unlimited_depth::{read_directory, UNLIMITED_DEPTH};
use std::sync::Arc;
use tsox_core::tspath;
use tsox_core::tspath::mig::m3i::{compare_paths, get_relative_path_from_directory};

impl Resolver {
    pub fn get_entrypoints_from_package_json_info(
        &self,
        package_json: &InfoCacheEntry,
        package_name: &str,
        enable_directory_search: bool,
    ) -> Option<Vec<Arc<ResolvedEntrypoint>>> { ::tsox_core::fntrace::enter("get_entrypoints_from_package_json_info"); 
        let extensions = Extensions::TYPESCRIPT.union(Extensions::DECLARATION);
        let features = NodeResolutionFeatures::ALL;
        let mut state = ResolutionState {
            name: String::new(),
            containing_directory: String::new(),
            is_config_lookup: false,
            features,
            esm_mode: false,
            conditions: Vec::new(),
            extensions,
            compiler_options: self.compiler_options.as_ref(),
            resolve_package_directory_only: false,
            fs: self.host.fs(),
            current_directory: self.host.get_current_directory(),
            resolved_package_directory: false,
            candidate_ending_is_from_config: false,
            export_target_depth: 0,
            resolution_diagnostics: Vec::new(),
            unresolved_terminal: false,
        };
        let contents_exports = package_json.get_contents().map(|c| &c.path_fields.exports);
        if package_json.exists() && contents_exports.is_some_and(|e| e.json_value.is_present()) {
            let entrypoints = state.load_entrypoints_from_export_map(
                self,
                package_json,
                package_name,
                contents_exports.unwrap(),
            );
            return Some(entrypoints);
        }

        let mut result: Vec<Arc<ResolvedEntrypoint>> = Vec::new();
        let main_resolution = state.load_node_module_from_directory_worker(
            extensions,
            &package_json.package_directory,
            true,
        );

        if main_resolution.as_ref().is_some_and(|r| r.is_resolved()) {
            let main_path = &main_resolution.as_ref().unwrap().path;
            result.push(Arc::new(self.create_resolved_entrypoint_handling_symlink(
                main_path,
                package_name,
                Vec::new(),
                Vec::new(),
                Ending::Fixed,
            )));
        }

        if enable_directory_search {
            let exts = extensions.array();
            let other_files = read_directory(
                self.host.fs(),
                self.host.get_current_directory(),
                &package_json.package_directory,
                &exts,
                &["node_modules"],
                &["**/*"],
                UNLIMITED_DEPTH,
            );

            let compare_paths_options = tspath::ComparePathsOptions {
                use_case_sensitive_file_names: self.host.fs().use_case_sensitive_file_names(),
                current_directory: String::new(),
            };
            for file in other_files {
                if main_resolution.as_ref().is_some_and(|r| r.is_resolved())
                    && compare_paths(
                        &file,
                        &main_resolution.as_ref().unwrap().path,
                        &compare_paths_options,
                    ) == 0
                {
                    continue;
                }

                let relative = get_relative_path_from_directory(
                    &package_json.package_directory,
                    &file,
                    &compare_paths_options,
                );
                let module_specifier = tspath::resolve_path(package_name, &[&relative]);
                result.push(Arc::new(self.create_resolved_entrypoint_handling_symlink(
                    &file,
                    &module_specifier,
                    Vec::new(),
                    Vec::new(),
                    Ending::Changeable,
                )));
            }
        }

        Some(result)
    }

    pub(crate) fn create_resolved_entrypoint_handling_symlink(
        &self,
        file_name: &str,
        module_specifier: &str,
        include_conditions: Vec<String>,
        exclude_conditions: Vec<String>,
        ending: Ending,
    ) -> ResolvedEntrypoint { ::tsox_core::fntrace::enter("create_resolved_entrypoint_handling_symlink"); 
        let mut original_file_name = String::new();
        let mut resolved_file_name = file_name.to_string();
        let real_path = self.host.fs().realpath(file_name);
        if real_path != file_name {
            original_file_name = file_name.to_string();
            resolved_file_name = real_path;
        }
        ResolvedEntrypoint {
            original_file_name,
            resolved_file_name,
            module_specifier: module_specifier.to_string(),
            ending,
            include_conditions,
            exclude_conditions,
        }
    }
}
