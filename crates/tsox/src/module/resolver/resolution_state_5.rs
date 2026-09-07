#![allow(unused_imports)]

use super::*;

impl<'a> ResolutionState<'a> {
    pub(crate) fn new(
        name: &str,
        containing_directory: &str,
        is_type_reference_directive: bool,
        resolution_mode: ResolutionMode,
        compiler_options: &'a CompilerOptions,
        fs: &'a dyn FS,
        current_directory: &'a str,
    ) -> Self {
        let extensions = if is_type_reference_directive {
            Extensions::DECLARATION
        } else if compiler_options.no_dts_resolution.is_true() {
            Extensions::IMPLEMENTATION_FILES
        } else {
            Extensions::TYPESCRIPT
                .union(Extensions::JAVASCRIPT)
                .union(Extensions::DECLARATION)
        };
        let extensions =
            if !is_type_reference_directive && compiler_options.get_resolve_json_module() {
                extensions.union(Extensions::JSON)
            } else {
                extensions
            };

        let (features, esm_mode, conditions) = match compiler_options.get_module_resolution_kind() {
            ModuleResolutionKind::Node16 => (
                NodeResolutionFeatures::NODE16_DEFAULT,
                resolution_mode == ModuleKind::ESNext,
                get_conditions(compiler_options, resolution_mode),
            ),
            ModuleResolutionKind::NodeNext => (
                NodeResolutionFeatures::NODE_NEXT_DEFAULT,
                resolution_mode == ModuleKind::ESNext,
                get_conditions(compiler_options, resolution_mode),
            ),
            ModuleResolutionKind::Bundler => (
                NodeResolutionFeatures::BUNDLER_DEFAULT,
                false,
                get_conditions(
                    compiler_options,
                    if resolution_mode == ResolutionMode::None {
                        ModuleKind::ESNext
                    } else {
                        resolution_mode
                    },
                ),
            ),
            _ => (NodeResolutionFeatures::NONE, false, Vec::new()),
        };

        ResolutionState {
            name: name.to_string(),
            containing_directory: containing_directory.to_string(),
            is_config_lookup: false,
            features,
            esm_mode,
            conditions,
            extensions,
            compiler_options,
            resolve_package_directory_only: false,
            fs,
            current_directory,
            resolved_package_directory: false,
            candidate_ending_is_from_config: false,
            export_target_depth: 0,
        }
    }

    pub(crate) fn normalize_path_for_cjs_resolution(directory: &str, name: &str) -> String {
        let combined = tspath::combine_paths(directory, &[name]);

        let last_component = tspath::get_base_file_name(&combined);
        let combined = tspath::normalize_path(&combined);
        if last_component == "." || last_component == ".." {
            tspath::ensure_trailing_directory_separator(&combined)
        } else {
            combined
        }
    }

    pub(crate) fn node_load_module_by_relative_name(
        &mut self,
        extensions: Extensions,
        candidate: &str,
        _consider_package_json: bool,
    ) -> Option<Resolved> {
        if !tspath::has_trailing_directory_separator(candidate) {
            let parent_of_candidate = tspath::get_directory_path(candidate);
            if !self.fs.directory_exists(&parent_of_candidate) {
                return CONTINUE_SEARCHING;
            }
            if let Some(resolved) = self.load_module_from_file(extensions, candidate) {
                return Some(resolved);
            }
        }
        if !self.fs.directory_exists(candidate) {
            return CONTINUE_SEARCHING;
        }

        if self.esm_mode {
            return CONTINUE_SEARCHING;
        }
        self.load_node_module_from_directory(extensions, candidate, true)
    }

    pub(crate) fn load_module_from_file(&self, extensions: Extensions, candidate: &str) -> Option<Resolved> {
        if let Some(resolved) =
            self.load_module_from_file_no_implicit_extensions(extensions, candidate)
        {
            return Some(resolved);
        }

        if !self.esm_mode {
            return self.try_adding_extensions(candidate, extensions, "");
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_module_from_file_no_implicit_extensions(
        &self,
        extensions: Extensions,
        candidate: &str,
    ) -> Option<Resolved> {
        let base = tspath::get_base_file_name(candidate);
        if !base.contains('.') {
            return CONTINUE_SEARCHING;
        }
        let extensionless = tspath::remove_file_extension(candidate);
        if extensionless == candidate {
            return CONTINUE_SEARCHING;
        }
        let extension = &candidate[extensionless.len()..];
        self.try_adding_extensions(&extensionless, extensions, extension)
    }

    pub(crate) fn try_adding_extensions(
        &self,
        extensionless: &str,
        extensions: Extensions,
        original_extension: &str,
    ) -> Option<Resolved> {
        let directory = tspath::get_directory_path(extensionless);
        if !directory.is_empty() && !self.fs.directory_exists(&directory) {
            return CONTINUE_SEARCHING;
        }

        match original_extension {
            ".mjs" | ".mts" | ".d.mts" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".mts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.mts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".mjs", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".cjs" | ".cts" | ".d.cts" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".cts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.cts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".cjs", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".json" => {
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.json.ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JSON) {
                    if let Some(r) = self.try_extension(".json", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            ".tsx" | ".jsx" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".tsx", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".jsx", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".js", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }

            ".ts" | ".d.ts" | ".js" | "" => {
                if extensions.contains(Extensions::TYPESCRIPT) {
                    if let Some(r) = self.try_extension(".ts", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".tsx", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::DECLARATION) {
                    if let Some(r) = self.try_extension(".d.ts", extensionless) {
                        return Some(r);
                    }
                }
                if extensions.contains(Extensions::JAVASCRIPT) {
                    if let Some(r) = self.try_extension(".js", extensionless) {
                        return Some(r);
                    }
                    if let Some(r) = self.try_extension(".jsx", extensionless) {
                        return Some(r);
                    }
                }
                if self.is_config_lookup {
                    if let Some(r) = self.try_extension(".json", extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
            _ => {
                if extensions.contains(Extensions::DECLARATION)
                    && !tspath::is_declaration_file_name(&format!(
                        "{extensionless}{original_extension}"
                    ))
                {
                    let ext = format!(".d{original_extension}.ts");
                    if let Some(r) = self.try_extension(&ext, extensionless) {
                        return Some(r);
                    }
                }
                CONTINUE_SEARCHING
            }
        }
    }

    pub(crate) fn try_extension(&self, extension: &str, extensionless: &str) -> Option<Resolved> {
        let file_name = format!("{extensionless}{extension}");
        if let Some(path) = self.try_file(&file_name) {
            return Some(Resolved {
                path,
                extension: extension.to_string(),
                resolved_using_ts_extension: true,
                ..Default::default()
            });
        }
        CONTINUE_SEARCHING
    }

}
