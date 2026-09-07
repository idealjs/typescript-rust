#![allow(unused_imports)]

use super::*;

impl<'a> ResolutionState<'a> {
    pub(crate) fn get_candidate_from_type_root(&self, type_root: &str) -> String {
        let name_for_lookup = if type_root.ends_with("/node_modules/@types")
            || type_root.ends_with("/node_modules/@types/")
        {
            crate::module::mangle_scoped_package_name(&self.name)
        } else {
            self.name.clone()
        };
        tspath::combine_paths(type_root, &[&name_for_lookup])
    }

    pub(crate) fn create_resolved_type_ref(
        &self,
        resolved: Option<Resolved>,
        primary: bool,
    ) -> ResolvedTypeReferenceDirective {
        match resolved {
            Some(r) if !r.path.is_empty() => {
                let is_external = r.path.contains("/node_modules/");
                ResolvedTypeReferenceDirective {
                    resolved_file_name: r.path,
                    primary,
                    package_id: r.package_id,
                    is_external_library_import: is_external,
                    ..Default::default()
                }
            }
            _ => ResolvedTypeReferenceDirective::default(),
        }
    }

    pub(crate) fn load_module_from_nearest_node_modules_directory(
        &mut self,
        types_scope_only: bool,
    ) -> Option<Resolved> {
        let ts_ext = self
            .extensions
            .intersection(Extensions::TYPESCRIPT | Extensions::DECLARATION);
        if !ts_ext.is_empty() {
            if let Some(resolved) = self
                .load_module_from_nearest_node_modules_directory_worker(ts_ext, types_scope_only)
            {
                return Some(resolved);
            }
        }

        let js_ext = self
            .extensions
            .difference(Extensions::TYPESCRIPT | Extensions::DECLARATION);
        if !js_ext.is_empty() {
            if let Some(resolved) = self
                .load_module_from_nearest_node_modules_directory_worker(js_ext, types_scope_only)
            {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_module_from_nearest_node_modules_directory_worker(
        &mut self,
        ext: Extensions,
        types_scope_only: bool,
    ) -> Option<Resolved> {
        let mut directory = self.containing_directory.clone();
        loop {
            if tspath::get_base_file_name(&directory) != "node_modules" {
                if let Some(resolved) = self.load_module_from_immediate_node_modules_directory(
                    ext,
                    &directory,
                    types_scope_only,
                ) {
                    return Some(resolved);
                }
            }
            let parent = tspath::get_directory_path(&directory);
            if parent == directory {
                break;
            }
            directory = parent;
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_module_from_immediate_node_modules_directory(
        &mut self,
        ext: Extensions,
        directory: &str,
        types_scope_only: bool,
    ) -> Option<Resolved> {
        let node_modules_folder = tspath::combine_paths(directory, &["node_modules"]);
        if !self.fs.directory_exists(&node_modules_folder) {
            return CONTINUE_SEARCHING;
        }
        if !types_scope_only {
            let name = self.name.clone();
            if let Some(resolved) = self.load_module_from_specific_node_modules_directory(
                ext,
                &name,
                &node_modules_folder,
            ) {
                return Some(resolved);
            }
        }

        if ext.contains(Extensions::DECLARATION) {
            let node_modules_at_types = tspath::combine_paths(&node_modules_folder, &["@types"]);
            if self.fs.directory_exists(&node_modules_at_types) {
                let mangled = mangle_scoped_package_name(&self.name);
                if let Some(resolved) = self.load_module_from_specific_node_modules_directory(
                    Extensions::DECLARATION,
                    &mangled,
                    &node_modules_at_types,
                ) {
                    return Some(resolved);
                }
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_module_from_specific_node_modules_directory(
        &mut self,
        ext: Extensions,
        module_name: &str,
        node_modules_directory: &str,
    ) -> Option<Resolved> {
        let candidate = tspath::normalize_path(&tspath::combine_paths(
            node_modules_directory,
            &[module_name],
        ));
        let (package_name, rest) = parse_package_name(module_name);
        let package_directory = tspath::combine_paths(node_modules_directory, &[&package_name]);

        let pkg_json_path = tspath::combine_paths(&package_directory, &["package.json"]);
        let package_info_exists = self.fs.file_exists(&pkg_json_path);
        if package_info_exists {
            self.resolved_package_directory = true;
        }

        if self.features.contains(NodeResolutionFeatures::Exports) && package_info_exists {
            if let Some(content) = self.fs.read_file(&pkg_json_path) {
                if let Ok(fields) = packagejson::parse(&content) {
                    let exports = &fields.path_fields.exports;
                    if exports.json_value.is_present() && !exports.json_value.is_falsy() {
                        let subpath = if rest.is_empty() {
                            ".".to_string()
                        } else {
                            format!("./{}", rest)
                        };
                        if let Some(resolved) = self.load_module_from_exports(
                            ext,
                            &subpath,
                            &package_directory,
                            exports,
                        ) {
                            return Some(resolved);
                        }
                    }
                }
            }
        }

        if !rest.is_empty() {
            if let Some(resolved) = self.load_module_from_file(ext, &candidate) {
                return Some(resolved);
            }
            return self.load_node_module_from_directory(ext, &candidate, true);
        }

        let has_exports = self
            .get_package_file(ext, &candidate)
            .map(|(_, f)| f.path_fields.exports.json_value.is_present())
            .unwrap_or(false);

        if !self.esm_mode {
            if let Some(resolved) = self.load_module_from_file(ext, &candidate) {
                return Some(resolved);
            }
        }

        if let Some(resolved) = self.load_node_module_from_directory(ext, &candidate, true) {
            return Some(resolved);
        }

        if package_info_exists && !has_exports && self.esm_mode {
            let index = tspath::combine_paths(&candidate, &["index.js"]);
            if let Some(resolved) = self.load_module_from_file(ext, &index) {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_node_module_from_directory(
        &mut self,
        ext: Extensions,
        candidate: &str,
        consider_package_dir: bool,
    ) -> Option<Resolved> {
        self.load_node_module_from_directory_worker(ext, candidate, consider_package_dir)
    }

    pub(crate) fn load_node_module_from_directory_worker(
        &mut self,
        ext: Extensions,
        candidate: &str,
        _consider_package_dir: bool,
    ) -> Option<Resolved> {
        let pkg_json_path = tspath::combine_paths(candidate, &["package.json"]);
        let package_info_exists = self.fs.file_exists(&pkg_json_path);

        if package_info_exists {
            if let Some(resolved) =
                self.try_load_module_using_package_json_type_versions(ext, candidate)
            {
                return Some(resolved);
            }
            if let Some((package_file, _)) = self.get_package_file(ext, candidate) {
                if let Some(resolved) =
                    self.load_file_name_from_package_json_field(ext, &package_file)
                {
                    return Some(resolved);
                }
            }
        }

        if !self.esm_mode {
            let index = tspath::combine_paths(candidate, &["index"]);
            if let Some(resolved) = self.load_module_from_file(ext, &index) {
                return Some(resolved);
            }
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn try_load_module_using_package_json_type_versions(
        &mut self,
        ext: Extensions,
        candidate: &str,
    ) -> Option<Resolved> {
        let pkg_json_path = tspath::combine_paths(candidate, &["package.json"]);
        let content = self.fs.read_file(&pkg_json_path)?;
        let fields = packagejson::parse(&content).ok()?;
        let tv = &fields.path_fields.types_versions;
        if !tv.is_present() || tv.value_type != packagejson::JsonValueType::Object {
            return CONTINUE_SEARCHING;
        }

        let (_, version_mapping) = tv.as_object().first()?;
        if version_mapping.value_type != packagejson::JsonValueType::Object {
            return CONTINUE_SEARCHING;
        }

        let mut paths_map: HashMap<String, Vec<String>> = HashMap::new();
        for (pattern, targets) in version_mapping.as_object() {
            if targets.value_type == packagejson::JsonValueType::Array {
                let target_strings: Vec<String> = targets
                    .as_array()
                    .iter()
                    .filter(|t| t.value_type == packagejson::JsonValueType::String)
                    .map(|t| t.as_string().to_string())
                    .collect();
                if !target_strings.is_empty() {
                    paths_map.insert(pattern.clone(), target_strings);
                }
            }
        }
        if paths_map.is_empty() {
            return CONTINUE_SEARCHING;
        }
        let parsed = try_parse_patterns(&paths_map);

        let (_, rest) = parse_package_name(&self.name);
        self.try_load_module_using_paths(ext, &rest, candidate, &paths_map, &parsed)
    }

}
