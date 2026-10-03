#![allow(unused_imports)]

use crate::compiler::{Program, ProgramOptions};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tsox_core::core::tristate::Tristate;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::mig::m3b_2;
use tsox_tsoptions::packagejson;
use tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry;
use tsox_tsoptions::tsoptions::ParsedCommandLine;
use tsox_core::tspath;
use tsox_core::tspath::mig::m3i::compare_paths;
use tsox_core::tspath::ComparePathsOptions;

type PackageJsonInfoCache = HashMap<String, InfoCacheEntry>;

static PACKAGE_JSON_INFO_CACHE: OnceLock<Mutex<PackageJsonInfoCache>> = OnceLock::new();

fn package_json_info_cache() -> &'static Mutex<PackageJsonInfoCache> { ::tsox_core::fntrace::enter("package_json_info_cache"); 
    PACKAGE_JSON_INFO_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn get_package_json_info_entry(
    resolver: &tsox_tsoptions::module::Resolver,
    package_directory: &str,
) -> Option<InfoCacheEntry> { ::tsox_core::fntrace::enter("get_package_json_info_entry"); 
    let package_json_path = tspath::combine_paths(package_directory, &["package.json"]);
    {
        let cache = package_json_info_cache().lock().unwrap();
        if let Some(existing) = cache.get(&package_json_path) {
            return if existing.contents.is_some() {
                Some(existing.with_package_directory(package_directory))
            } else {
                None
            };
        }
    }
    let fs = resolver.host().fs();
    let directory_exists = fs.directory_exists(package_directory);
    let entry = if directory_exists && fs.file_exists(&package_json_path) {
        let content = fs.read_file(&package_json_path).unwrap_or_default();
        InfoCacheEntry {
            package_directory: package_directory.to_string(),
            directory_exists: true,
            contents: Some(packagejson::parse(&content).unwrap_or_default()),
        }
    } else {
        InfoCacheEntry {
            package_directory: package_directory.to_string(),
            directory_exists,
            contents: None,
        }
    };
    let result = entry.with_package_directory(package_directory);
    package_json_info_cache()
        .lock()
        .unwrap()
        .insert(package_json_path, entry);
    result.contents.is_some().then_some(result)
}

pub(crate) fn get_package_scope_for_path(
    resolver: &tsox_tsoptions::module::Resolver,
    typings_location: &str,
    directory: &str,
) -> Option<InfoCacheEntry> { ::tsox_core::fntrace::enter("get_package_scope_for_path"); 
    let mut current = directory.to_string();
    loop {
        if let Some(result) = get_package_json_info_entry(resolver, &current) {
            return Some(result);
        }
        if current == typings_location {
            return None;
        }
        let parent = tspath::get_directory_path(&current);
        if parent == current {
            return None;
        }
        current = parent;
    }
}

pub(crate) fn package_json_cache_entries(
    f: &mut dyn FnMut(&str, &InfoCacheEntry) -> bool,
) { ::tsox_core::fntrace::enter("package_json_cache_entries"); 
    let cache = package_json_info_cache().lock().unwrap();
    for (key, value) in cache.iter() {
        if !f(key, value) {
            break;
        }
    }
}

pub(crate) fn resolve_package_directory(
    resolver: &tsox_tsoptions::module::Resolver,
    module_name: &str,
    containing_file: &str,
) -> Option<tsox_tsoptions::module::ResolvedModule> { ::tsox_core::fntrace::enter("resolve_package_directory"); 
    let fs = resolver.host().fs();
    let mut directory = tspath::get_directory_path(containing_file);
    loop {
        if tspath::get_base_file_name(&directory) != "node_modules" {
            let node_modules_folder = tspath::combine_paths(&directory, &["node_modules"]);
            if fs.directory_exists(&node_modules_folder) {
                if let Some(resolved) =
                    resolve_package_directory_in_node_modules(resolver, module_name, &node_modules_folder)
                {
                    return Some(resolved);
                }
            }
        }
        let parent = tspath::get_directory_path(&directory);
        if parent == directory {
            return None;
        }
        directory = parent;
    }
}

fn resolve_package_directory_in_node_modules(
    resolver: &tsox_tsoptions::module::Resolver,
    module_name: &str,
    node_modules_folder: &str,
) -> Option<tsox_tsoptions::module::ResolvedModule> { ::tsox_core::fntrace::enter("resolve_package_directory_in_node_modules"); 
    let fs = resolver.host().fs();
    let (package_name, _) = tsox_tsoptions::module::parse_package_name(module_name);
    let package_directory = if package_name.is_empty() {
        tspath::combine_paths(node_modules_folder, &[module_name])
    } else {
        tspath::combine_paths(node_modules_folder, &[&package_name])
    };
    if fs.directory_exists(&package_directory) {
        return Some(create_resolved_module_handling_symlink(resolver, package_directory));
    }
    let mangled = tsox_tsoptions::module::mangle_scoped_package_name(module_name);
    let node_modules_at_types = tspath::combine_paths(node_modules_folder, &["@types", &mangled]);
    if fs.directory_exists(&node_modules_at_types) {
        return Some(create_resolved_module_handling_symlink(resolver, node_modules_at_types));
    }
    None
}

fn create_resolved_module_handling_symlink(
    resolver: &tsox_tsoptions::module::Resolver,
    path: String,
) -> tsox_tsoptions::module::ResolvedModule { ::tsox_core::fntrace::enter("create_resolved_module_handling_symlink"); 
    let fs = resolver.host().fs();
    let is_external_library_import = path.contains("/node_modules/");
    let mut resolved_file_name = path.clone();
    let mut original_path = String::new();
    if !resolver.compiler_options().preserve_symlinks.is_true()
        && is_external_library_import
    {
        let real_path = fs.realpath(&path);
        let compare_options = ComparePathsOptions {
            use_case_sensitive_file_names: fs.use_case_sensitive_file_names(),
            current_directory: resolver.host().get_current_directory().to_string(),
        };
        if compare_paths(&path, &real_path, &compare_options) != 0 {
            original_path = path;
            resolved_file_name = real_path;
        }
    }
    tsox_tsoptions::module::ResolvedModule {
        resolved_file_name,
        original_path,
        extension: String::new(),
        resolved_using_ts_extension: false,
        resolved_using_extra_extensions: false,
        package_id: None,
        is_external_library_import,
        alternate_result: None,
        resolution_diagnostics: Vec::new(),
    }
}

impl crate::mig::m4x_2::HasFileName for tsox_frontend::ast::mig::m3g_3::HasFileNameImpl {
    fn path(&self) -> String { ::tsox_core::fntrace::enter("path"); 
        self.path().0.clone()
    }

    fn file_name(&self) -> String { ::tsox_core::fntrace::enter("file_name"); 
        self.file_name().to_string()
    }
}

impl Program {
    pub fn can_use_project_reference_source(&self) -> bool { ::tsox_core::fntrace::enter("can_use_project_reference_source"); 
        self.use_source_of_project_reference
            && !self
                .options()
                .disable_source_of_project_reference_redirect
                .is_true()
    }

    pub fn content_mapper_project(&self) -> Option<Arc<dyn crate::mig::m3l_cm_2::Project>> { ::tsox_core::fntrace::enter("content_mapper_project"); 
        self.opts.host.content_mapper_project()
    }

    pub fn get_global_typings_cache_location(&self) -> &str { ::tsox_core::fntrace::enter("get_global_typings_cache_location"); 
        &self.typings_location
    }

    pub fn get_nearest_ancestor_directory_with_package_json(&self, dirname: &str) -> String { ::tsox_core::fntrace::enter("get_nearest_ancestor_directory_with_package_json"); 
        if let Some(scoped) =
            get_package_scope_for_path(&self.resolver, &self.typings_location, dirname)
            .filter(|scoped| scoped.exists())
        {
            return scoped.package_directory;
        }
        String::new()
    }

    pub fn get_package_json_info(
        &self,
        pkg_json_path: &str,
    ) -> Option<tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry> { ::tsox_core::fntrace::enter("get_package_json_info"); 
        let directory = tspath::get_directory_path(pkg_json_path);
        if let Some(scoped) =
            get_package_scope_for_path(&self.resolver, &self.typings_location, &directory)
            .filter(|scoped| scoped.exists() && scoped.package_directory == directory)
        {
            return Some(scoped);
        }
        None
    }

    pub fn package_json_cache_entries(
        &self,
        f: &mut dyn FnMut(&str, &tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry) -> bool,
    ) { ::tsox_core::fntrace::enter("package_json_cache_entries"); 
        package_json_cache_entries(f);
    }

    pub fn get_redirect_targets(&self, path: &str) -> Vec<String> { ::tsox_core::fntrace::enter("get_redirect_targets"); 
        self.redirect_targets_map
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_source_of_project_reference_if_output_included(
        &self,
        file: &Arc<SourceFile>,
    ) -> String { ::tsox_core::fntrace::enter("get_source_of_project_reference_if_output_included"); 
        if let Some(source) = self
            .output_file_to_project_reference_source
            .get(m3b_2::path(file).as_str())
        {
            return source.clone();
        }
        file.file_name.clone()
    }

    pub fn get_project_reference_from_source(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> { ::tsox_core::fntrace::enter("get_project_reference_from_source"); 
        self.project_reference_file_mapper
            .get_project_reference_from_source(&tspath::Path(path.to_string()))
    }

    pub fn is_source_from_project_reference(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("is_source_from_project_reference"); 
        self.project_reference_file_mapper
            .is_source_from_project_reference(&tspath::Path(path.to_string()))
    }

    pub fn get_project_reference_from_output_dts(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference> { ::tsox_core::fntrace::enter("get_project_reference_from_output_dts"); 
        self.project_reference_file_mapper
            .get_project_reference_from_output_dts(&tspath::Path(path.to_string()))
    }

    pub fn get_resolved_project_reference_for(
        &self,
        path: &str,
    ) -> (Option<Arc<ParsedCommandLine>>, bool) { ::tsox_core::fntrace::enter("get_resolved_project_reference_for"); 
        self.project_reference_file_mapper
            .get_resolved_reference_for(&tspath::Path(path.to_string()))
    }

    pub fn get_redirect_for_resolution(
        &self,
        file: &Arc<SourceFile>,
    ) -> Option<Arc<ParsedCommandLine>> { ::tsox_core::fntrace::enter("get_redirect_for_resolution"); 
        let (redirect, _) = self
            .project_reference_file_mapper
            .get_redirect_for_resolution(file);
        redirect
    }

    pub fn get_parse_file_redirect(&self, file_name: &str) -> String { ::tsox_core::fntrace::enter("get_parse_file_redirect"); 
        self.project_reference_file_mapper
            .get_parse_file_redirect(&tsox_frontend::ast::mig::m3g_3::new_has_file_name(
                file_name.to_string(),
                self.to_path(file_name),
            ))
    }

    pub fn get_resolved_project_references(&self) -> Vec<Option<Arc<ParsedCommandLine>>> { ::tsox_core::fntrace::enter("get_resolved_project_references"); 
        self.project_reference_file_mapper.get_resolved_project_references()
    }

    pub fn range_resolved_project_reference(
        &self,
        f: &mut dyn FnMut(
            &tspath::Path,
            Option<&Arc<ParsedCommandLine>>,
            Option<&Arc<ParsedCommandLine>>,
            usize,
        ) -> bool,
    ) -> bool { ::tsox_core::fntrace::enter("range_resolved_project_reference"); 
        self.project_reference_file_mapper
            .range_resolved_project_reference(f)
    }

    pub fn range_resolved_project_reference_in_child_config(
        &self,
        child_config: Option<&Arc<ParsedCommandLine>>,
        f: &mut dyn FnMut(
            &tspath::Path,
            Option<&Arc<ParsedCommandLine>>,
            Option<&Arc<ParsedCommandLine>>,
            usize,
        ) -> bool,
    ) -> bool { ::tsox_core::fntrace::enter("range_resolved_project_reference_in_child_config"); 
        self.project_reference_file_mapper
            .range_resolved_project_reference_in_child_config(child_config, f)
    }

    pub fn uses_uri_style_node_core_modules(&self) -> Tristate { ::tsox_core::fntrace::enter("uses_uri_style_node_core_modules"); 
        self.uses_uri_style_node_core_modules
    }

    pub fn get_source_file_from_reference(
        &self,
        origin: &Arc<SourceFile>,
        r: &tsox_frontend::ast::node_source_file::FileReference,
    ) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file_from_reference"); 
        let file_name = tspath::resolve_path(
            &tspath::get_directory_path(&origin.file_name),
            &[r.file_name.as_str()],
        );
        let content_mapper_extensions = self.content_mapper_extensions();
        let extra_extensions: Vec<&str> = content_mapper_extensions
            .iter()
            .map(|s| s.as_str())
            .collect();
        let supported_extensions_base = tsox_tsoptions::mig::m5i_3::get_supported_extensions(
            self.options(),
            &extra_extensions,
        );
        let supported_extensions = tsox_tsoptions::mig::m5i_3::get_supported_extensions_with_json_if_resolve_json_module(
            self.options(),
            &supported_extensions_base,
        );
        let allow_non_ts_extensions = self.options().allow_non_ts_extensions.is_true();
        if tspath::has_extension(&file_name) {
            if !allow_non_ts_extensions {
                let canonical_file_name =
                    tspath::get_canonical_file_name(&file_name, self.use_case_sensitive_file_names());
                let mut supported = false;
                for group in &supported_extensions {
                    let group_refs: Vec<&str> = group.iter().map(|s| s.as_str()).collect();
                    if tspath::file_extension_is_one_of(&canonical_file_name, &group_refs) {
                        supported = true;
                        break;
                    }
                }
                if !supported {
                    return None;
                }
            }

            return self.get_source_file_for_resolved_module(&file_name);
        }
        if allow_non_ts_extensions {
            let extensionless = self.get_source_file_for_resolved_module(&file_name);
            if extensionless.is_some() {
                return extensionless;
            }
        }

        for ext in &supported_extensions[0] {
            let result = self.get_source_file_for_resolved_module(&format!("{file_name}{ext}"));
            if result.is_some() {
                return result;
            }
        }
        None
    }
}
