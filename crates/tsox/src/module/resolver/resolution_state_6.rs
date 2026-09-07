#![allow(unused_imports)]

use super::*;

impl<'a> ResolutionState<'a> {
    pub(crate) fn try_file(&self, file_name: &str) -> Option<String> {
        if self.compiler_options.module_suffixes.is_empty() {
            if self.fs.file_exists(file_name) {
                return Some(file_name.to_string());
            }
            return None;
        }
        let ext = tspath::try_get_extension_from_path(file_name);
        let file_name_no_ext = tspath::remove_extension(file_name, ext);
        for suffix in &self.compiler_options.module_suffixes {
            let path = format!("{file_name_no_ext}{suffix}{ext}");
            if self.fs.file_exists(&path) {
                return Some(path);
            }
        }
        None
    }

    pub(crate) fn get_paths_base_path(&self) -> String {
        if !self.compiler_options.paths_base_path.is_empty() {
            return self.compiler_options.paths_base_path.clone();
        }
        if !self.compiler_options.base_url.is_empty() {
            return self.compiler_options.base_url.clone();
        }
        if !self.compiler_options.config_file_path.is_empty() {
            return tspath::get_directory_path(&self.compiler_options.config_file_path);
        }
        self.current_directory.to_string()
    }

    pub(crate) fn try_load_module_using_optional_resolution_settings(&mut self) -> Option<Resolved> {
        if let Some(r) = self.try_load_module_using_paths_if_eligible() {
            return Some(r);
        }
        if !tspath::is_external_module_name_relative(&self.name) {
            if !self.compiler_options.base_url.is_empty() {
                let candidate = tspath::normalize_path(&tspath::combine_paths(
                    &self.compiler_options.base_url,
                    &[&self.name],
                ));
                if let Some(r) =
                    self.node_load_module_by_relative_name(self.extensions, &candidate, true)
                {
                    return Some(r);
                }
            }

            return CONTINUE_SEARCHING;
        }

        self.try_load_module_using_root_dirs()
    }

    pub(crate) fn try_load_module_using_paths_if_eligible(&mut self) -> Option<Resolved> {
        let paths = match &self.compiler_options.paths {
            Some(p) if !p.is_empty() && !tspath::path_is_relative(&self.name) => p,
            _ => return CONTINUE_SEARCHING,
        };
        let base_directory = self.get_paths_base_path();
        let parsed_patterns = try_parse_patterns(paths);
        let name = self.name.clone();
        self.try_load_module_using_paths(
            self.extensions,
            &name,
            &base_directory,
            paths,
            &parsed_patterns,
        )
    }

    pub(crate) fn try_load_module_using_paths(
        &mut self,
        extensions: Extensions,
        module_name: &str,
        containing_directory: &str,
        paths: &std::collections::HashMap<String, Vec<String>>,
        parsed_patterns: &ParsedPatterns,
    ) -> Option<Resolved> {
        if let Some(matched_pattern) = match_pattern_or_exact(parsed_patterns, module_name) {
            let matched_star = matched_pattern.matched_text(module_name);
            if let Some(substitutions) = paths.get(&matched_pattern.text) {
                for subst in substitutions {
                    let path = subst.replace('*', &matched_star);
                    let candidate = tspath::normalize_path(&tspath::combine_paths(
                        containing_directory,
                        &[&path],
                    ));

                    let extension_from_subst = tspath::try_get_extension_from_path(subst);
                    if !extension_from_subst.is_empty() {
                        if let Some(p) = self.try_file(&candidate) {
                            return Some(Resolved {
                                path: p,
                                extension: extension_from_subst.to_string(),
                                ..Default::default()
                            });
                        }
                    }

                    let saved = self.candidate_ending_is_from_config;
                    if !extension_from_subst.is_empty() {
                        self.candidate_ending_is_from_config = true;
                    }
                    let result =
                        self.node_load_module_by_relative_name(extensions, &candidate, true);
                    self.candidate_ending_is_from_config = saved;
                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn try_load_module_using_root_dirs(&mut self) -> Option<Resolved> {
        if self.compiler_options.root_dirs.is_empty() {
            return CONTINUE_SEARCHING;
        }
        let candidate = tspath::normalize_path(&tspath::combine_paths(
            &self.containing_directory,
            &[&self.name],
        ));

        let mut matched_normalized_prefix = String::new();
        for root_dir in &self.compiler_options.root_dirs {
            let mut normalized_root = tspath::normalize_path(root_dir);
            if !normalized_root.ends_with('/') {
                normalized_root.push('/');
            }
            if candidate.starts_with(&normalized_root)
                && matched_normalized_prefix.len() < normalized_root.len()
            {
                matched_normalized_prefix = normalized_root;
            }
        }
        if matched_normalized_prefix.is_empty() {
            return CONTINUE_SEARCHING;
        }
        let suffix = &candidate[matched_normalized_prefix.len()..];

        if let Some(r) = self.node_load_module_by_relative_name(self.extensions, &candidate, true) {
            return Some(r);
        }

        let matched_root_normalized = tspath::normalize_path(
            &matched_normalized_prefix[..matched_normalized_prefix.len().saturating_sub(1)],
        );
        for root_dir in &self.compiler_options.root_dirs {
            let normalized = tspath::normalize_path(root_dir);
            if normalized == matched_root_normalized {
                continue;
            }
            let alternate = tspath::normalize_path(&tspath::combine_paths(&normalized, &[suffix]));
            if let Some(r) =
                self.node_load_module_by_relative_name(self.extensions, &alternate, true)
            {
                return Some(r);
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn resolve_node_like(mut self) -> ResolvedModule {
        let result = self.resolve_node_like_worker();
        if result.is_none() {
            if !tspath::is_external_module_name_relative(&self.name)
                && !self.features.contains(NodeResolutionFeatures::Exports)
                && self
                    .extensions
                    .intersects(Extensions::TYPESCRIPT | Extensions::DECLARATION)
            {
                self.features |= NodeResolutionFeatures::ALL;
                if let Some(alt) = self.resolve_node_like_worker() {
                    return ResolvedModule {
                        alternate_result: Some(alt.path),
                        ..Default::default()
                    };
                }
            }
            return self.create_resolved_module(None);
        }
        self.create_resolved_module(result)
    }

    pub(crate) fn resolve_node_like_worker(&mut self) -> Option<Resolved> {
        if let Some(resolved) = self.try_load_module_using_optional_resolution_settings() {
            return Some(resolved);
        }

        if !tspath::is_external_module_name_relative(&self.name)
            && self.name.starts_with('#')
            && self.features.contains(NodeResolutionFeatures::Imports)
        {
            if let Some(resolved) = self.load_module_from_imports() {
                return Some(resolved);
            }
        }

        if !tspath::is_external_module_name_relative(&self.name)
            && self.features.contains(NodeResolutionFeatures::SelfName)
        {
            if let Some(resolved) = self.load_module_from_self_name_reference() {
                return Some(resolved);
            }
        }
        if tspath::is_external_module_name_relative(&self.name) {
            let candidate =
                Self::normalize_path_for_cjs_resolution(&self.containing_directory, &self.name);
            return self.node_load_module_by_relative_name(self.extensions, &candidate, true);
        }

        if let Some(resolved) = self.load_module_from_nearest_node_modules_directory(false) {
            return Some(resolved);
        }

        if self.extensions.contains(Extensions::DECLARATION) {
            if let Some(resolved) = self.resolve_from_type_root() {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn resolve_from_type_root(&mut self) -> Option<Resolved> {
        let (type_roots, _) =
            get_effective_type_roots(self.compiler_options, self.current_directory);
        for type_root in &type_roots {
            if !self.fs.directory_exists(type_root) {
                continue;
            }
            let package_directory = tspath::combine_paths(type_root, &[&self.name]);
            if self.fs.directory_exists(&package_directory) {
                let result = self.load_node_module_from_directory_worker(
                    Extensions::DECLARATION,
                    &package_directory,
                    false,
                );
                if result.is_some() {
                    return result;
                }
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn resolve_type_reference_directive(
        &mut self,
        type_roots: &[String],
        from_config: bool,
        from_inferred_types_containing_file: bool,
    ) -> ResolvedTypeReferenceDirective {
        if !type_roots.is_empty() {
            for type_root in type_roots {
                if !self.fs.directory_exists(type_root) {
                    continue;
                }
                let candidate = self.get_candidate_from_type_root(type_root);
                if from_config {
                    if let Some(resolved) =
                        self.load_module_from_file(Extensions::DECLARATION, &candidate)
                    {
                        return self.create_resolved_type_ref(Some(resolved), true);
                    }
                }
                if let Some(resolved) =
                    self.load_node_module_from_directory(Extensions::DECLARATION, &candidate, true)
                {
                    return self.create_resolved_type_ref(Some(resolved), true);
                }
            }
        }

        if !from_config || !from_inferred_types_containing_file {
            let resolved = if tspath::is_external_module_name_relative(&self.name) {
                let candidate =
                    Self::normalize_path_for_cjs_resolution(&self.containing_directory, &self.name);
                self.node_load_module_by_relative_name(Extensions::DECLARATION, &candidate, true)
            } else {
                self.load_module_from_nearest_node_modules_directory(false)
            };
            return self.create_resolved_type_ref(resolved, false);
        }

        ResolvedTypeReferenceDirective::default()
    }

}
