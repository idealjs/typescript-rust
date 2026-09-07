#![allow(unused_imports)]

use super::*;

impl<'a> ResolutionState<'a> {
    pub(crate) fn get_package_file(
        &self,
        ext: Extensions,
        candidate: &str,
    ) -> Option<(String, packagejson::Fields)> {
        let pkg_json_path = tspath::combine_paths(candidate, &["package.json"]);
        if !self.fs.file_exists(&pkg_json_path) {
            return None;
        }
        let content = self.fs.read_file(&pkg_json_path)?;
        let fields = packagejson::parse(&content).ok()?;

        if ext.contains(Extensions::DECLARATION) {
            if let Some(typings) = fields.path_fields.typings.get_value() {
                let path =
                    tspath::normalize_path(&tspath::combine_paths(candidate, &[typings.as_str()]));
                return Some((path, fields));
            }
            if let Some(types) = fields.path_fields.types.get_value() {
                let path =
                    tspath::normalize_path(&tspath::combine_paths(candidate, &[types.as_str()]));
                return Some((path, fields));
            }
        }
        if ext.intersects(Extensions::IMPLEMENTATION_FILES | Extensions::DECLARATION) {
            if let Some(main) = fields.path_fields.main.get_value() {
                let path =
                    tspath::normalize_path(&tspath::combine_paths(candidate, &[main.as_str()]));
                return Some((path, fields));
            }
        }
        None
    }

    pub(crate) fn load_file_name_from_package_json_field(
        &self,
        ext: Extensions,
        package_file: &str,
    ) -> Option<Resolved> {
        let extension = tspath::try_get_extension_from_path(package_file);
        if tspath::extension_is_ts(extension)
            && ext.intersects(Extensions::TYPESCRIPT | Extensions::DECLARATION)
        {
            if let Some(path) = self.try_file(package_file) {
                return Some(Resolved {
                    path,
                    extension: extension.to_string(),
                    resolved_using_ts_extension: true,
                    ..Default::default()
                });
            }
            return CONTINUE_SEARCHING;
        }
        self.load_module_from_file_no_implicit_extensions(ext, package_file)
    }

    pub(crate) fn condition_matches(&self, condition: &str) -> bool {
        if condition == "default" || self.conditions.iter().any(|c| c == condition) {
            return true;
        }

        if !self.conditions.iter().any(|c| c == "types") {
            return false;
        }
        false
    }

    pub(crate) fn load_module_from_exports(
        &mut self,
        ext: Extensions,
        subpath: &str,
        package_directory: &str,
        exports: &packagejson::ExportsOrImports,
    ) -> Option<Resolved> {
        if !exports.json_value.is_present() || exports.json_value.is_falsy() {
            return CONTINUE_SEARCHING;
        }

        if subpath == "." {
            match exports.json_value.value_type {
                packagejson::JsonValueType::String | packagejson::JsonValueType::Array => {
                    return self.load_module_from_target_export_or_import(
                        ext,
                        subpath,
                        package_directory,
                        false,
                        &exports.json_value,
                        "",
                        false,
                    );
                }
                packagejson::JsonValueType::Object => {
                    if exports.is_conditions() {
                        return self.load_module_from_target_export_or_import(
                            ext,
                            subpath,
                            package_directory,
                            false,
                            &exports.json_value,
                            "",
                            false,
                        );
                    }
                    if let Some(dot) = exports.json_value.get(".") {
                        return self.load_module_from_target_export_or_import(
                            ext,
                            subpath,
                            package_directory,
                            false,
                            dot,
                            "",
                            false,
                        );
                    }
                }
                _ => {}
            }
        } else if exports.json_value.value_type == packagejson::JsonValueType::Object
            && exports.is_subpaths()
        {
            return self.load_module_from_exports_or_imports(
                ext,
                subpath,
                &exports.json_value,
                package_directory,
                false,
            );
        }
        CONTINUE_SEARCHING
    }

    pub(crate) fn load_module_from_imports(&mut self) -> Option<Resolved> {
        if self.name == "#" {
            return CONTINUE_SEARCHING;
        }

        if self.name.starts_with("#/")
            && !self
                .features
                .contains(NodeResolutionFeatures::ImportsPatternRoot)
        {
            return CONTINUE_SEARCHING;
        }

        let directory_path = tspath::get_normalized_absolute_path(
            &self.containing_directory,
            self.current_directory,
        );
        let (package_directory, fields) = match self.get_package_scope_for_path(&directory_path) {
            Some(s) => s,
            None => return CONTINUE_SEARCHING,
        };

        let imports = &fields.path_fields.imports;
        if !imports.json_value.is_present()
            || imports.json_value.value_type != packagejson::JsonValueType::Object
        {
            return CONTINUE_SEARCHING;
        }

        let name = self.name.clone();
        self.load_module_from_exports_or_imports(
            self.extensions,
            &name,
            &imports.json_value,
            &package_directory,
            true,
        )
    }

    pub(crate) fn load_module_from_self_name_reference(&mut self) -> Option<Resolved> {
        let directory_path = tspath::get_normalized_absolute_path(
            &self.containing_directory,
            self.current_directory,
        );
        let (package_directory, fields) = self.get_package_scope_for_path(&directory_path)?;
        let exports = &fields.path_fields.exports;
        if !exports.json_value.is_present() || exports.json_value.is_falsy() {
            return CONTINUE_SEARCHING;
        }
        let Some(package_name) = fields.header_fields.name.get_value() else {
            return CONTINUE_SEARCHING;
        };

        let parts: Vec<&str> = self.name.split('/').filter(|p| !p.is_empty()).collect();
        let name_parts: Vec<&str> = package_name.split('/').filter(|p| !p.is_empty()).collect();
        if parts.len() < name_parts.len() || parts[..name_parts.len()] != name_parts[..] {
            return CONTINUE_SEARCHING;
        }
        let trailing = &parts[name_parts.len()..];
        let subpath = if trailing.is_empty() {
            ".".to_string()
        } else {
            format!("./{}", trailing.join("/"))
        };
        self.load_module_from_exports(self.extensions, &subpath, &package_directory, exports)
    }

    pub(crate) fn get_package_scope_for_path(&self, directory: &str) -> Option<(String, packagejson::Fields)> {
        let mut dir = directory.to_string();
        loop {
            let pkg_json_path = tspath::combine_paths(&dir, &["package.json"]);
            if self.fs.file_exists(&pkg_json_path) {
                if let Some(content) = self.fs.read_file(&pkg_json_path) {
                    if let Ok(fields) = packagejson::parse(&content) {
                        return Some((dir, fields));
                    }
                }
            }
            let parent = tspath::get_directory_path(&dir);
            if parent == dir {
                break;
            }
            dir = parent;
        }
        None
    }

}
