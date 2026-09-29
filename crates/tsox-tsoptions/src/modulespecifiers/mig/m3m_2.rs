use tsox_core::collections::ordered_map::OrderedMap;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_core::core::mig::m3j_2::index_after;
use tsox_core::stringutil::mig::m3m::has_prefix;
use tsox_core::tspath;
use tsox_core::tspath::ComparePathsOptions;

use crate::modulespecifiers::types::ModulePath;
use crate::modulespecifiers::types::ModuleSpecifierEnding;
use crate::modulespecifiers::types::ModuleSpecifierGenerationHost;
use crate::modulespecifiers::types::ModuleSpecifierOptions;
use crate::modulespecifiers::types::SourceFileForSpecifierGeneration;
use crate::modulespecifiers::types::UserPreferences;
use crate::packagejson::ExportsOrImports;
use super::m3l::get_each_file_name_of_module;
use super::m3l::get_module_specifier_preferences;
use super::m3m::compare_paths_by_redirect;
use super::w12::process_ending;
use tsox_frontend::ast::SourceFile;

impl SourceFileForSpecifierGeneration for SourceFile {
    fn path(&self) -> &str {
        &self.file_name
    }

    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn is_js(&self) -> bool {
        matches!(
            self.script_kind,
            tsox_frontend::ast::ScriptKind::Js | tsox_frontend::ast::ScriptKind::Jsx
        )
    }
}

pub struct NodeModulePathParts {
    pub top_level_node_modules_index: usize,
    pub top_level_package_name_index: usize,
    pub package_root_index: usize,
    pub file_name_index: usize,
}

pub struct Info {
    pub use_case_sensitive_file_names: bool,
    pub importing_source_file_name: String,
    pub source_directory: String,
}

pub fn get_info(
    importing_source_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
) -> Info {
    Info {
        use_case_sensitive_file_names: host.use_case_sensitive_file_names(),
        importing_source_file_name: importing_source_file_name.to_string(),
        source_directory: tspath::get_directory_path(importing_source_file_name),
    }
}

#[derive(PartialEq, PartialOrd)]
enum NodeModulesPathParseState {
    BeforeNodeModules,
    NodeModules,
    Scope,
    PackageContent,
}

pub fn get_node_module_path_parts(full_path: &str) -> Option<NodeModulePathParts> {
    let mut top_level_node_modules_index = 0;
    let mut top_level_package_name_index = 0;
    let mut package_root_index = 0;
    let mut file_name_index;

    let mut part_start = 0usize;
    let mut part_end = 0i64;
    let mut state = NodeModulesPathParseState::BeforeNodeModules;

    while part_end >= 0 {
        part_start = part_end as usize;
        part_end = match index_after(full_path, "/", part_start + 1) {
            Some(idx) => idx as i64,
            None => -1,
        };
        match state {
            NodeModulesPathParseState::BeforeNodeModules => {
                if full_path[part_start..].starts_with("/node_modules/") {
                    top_level_node_modules_index = part_start;
                    top_level_package_name_index = part_end as usize;
                    state = NodeModulesPathParseState::NodeModules;
                }
            }
            NodeModulesPathParseState::NodeModules
            | NodeModulesPathParseState::Scope => {
                if state == NodeModulesPathParseState::NodeModules
                    && full_path.as_bytes().get(part_start + 1) == Some(&b'@')
                {
                    state = NodeModulesPathParseState::Scope;
                } else {
                    package_root_index = part_end as usize;
                    state = NodeModulesPathParseState::PackageContent;
                }
            }
            NodeModulesPathParseState::PackageContent => {
                if full_path[part_start..].starts_with("/node_modules/") {
                    state = NodeModulesPathParseState::NodeModules;
                } else {
                    state = NodeModulesPathParseState::PackageContent;
                }
            }
        }
    }

    file_name_index = part_start;

    if state > NodeModulesPathParseState::NodeModules {
        return Some(NodeModulePathParts {
            top_level_node_modules_index,
            top_level_package_name_index,
            package_root_index,
            file_name_index,
        });
    }
    None
}

pub fn get_node_modules_package_name(
    compiler_options: &CompilerOptions,
    importing_source_file: &SourceFile,
    node_modules_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
    preferences: &UserPreferences,
    options: &ModuleSpecifierOptions,
) -> String {
    let info = get_info(&importing_source_file.file_name, host);
    let module_paths = get_all_module_paths(
        &info,
        node_modules_file_name,
        host,
        compiler_options,
        preferences,
        options,
    );
    for module_path in &module_paths {
        let result = try_get_module_name_as_node_module(
            module_path,
            &info,
            importing_source_file,
            host,
            compiler_options,
            preferences,
            true,
            options.override_import_mode,
        );
        if !result.is_empty() {
            return result;
        }
    }
    String::new()
}

pub fn get_all_module_paths(
    info: &Info,
    imported_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
    compiler_options: &CompilerOptions,
    _preferences: &UserPreferences,
    _options: &ModuleSpecifierOptions,
) -> Vec<ModulePath> {
    get_all_module_paths_worker(info, imported_file_name, host, compiler_options)
}

fn sort_module_paths(paths: &mut [ModulePath], use_case_sensitive_file_names: bool) {
    paths.sort_by(|a, b| {
        compare_paths_by_redirect(a, b, use_case_sensitive_file_names).cmp(&0)
    });
}

fn get_all_module_paths_worker(
    info: &Info,
    imported_file_name: &str,
    host: &dyn ModuleSpecifierGenerationHost,
    _compiler_options: &CompilerOptions,
) -> Vec<ModulePath> {
    let mut all_file_names: OrderedMap<String, ModulePath> = OrderedMap::new();
    let paths = get_each_file_name_of_module(
        &info.importing_source_file_name,
        imported_file_name,
        host,
        true,
    );
    for p in paths {
        all_file_names.set(p.file_name.clone(), p);
    }

    let use_case_sensitive_file_names = info.use_case_sensitive_file_names;
    let mut sorted_paths: Vec<ModulePath> = Vec::with_capacity(all_file_names.len());
    let mut directory = info.source_directory.clone();
    while !all_file_names.is_empty() {
        let directory_start = tspath::ensure_trailing_directory_separator(&directory);
        let mut paths_in_directory: Vec<ModulePath> = Vec::new();
        let keys: Vec<String> = all_file_names.keys().cloned().collect();
        for file_name in &keys {
            if file_name.starts_with(directory_start.as_str()) {
                if let Some(p) = all_file_names.delete(file_name) {
                    paths_in_directory.push(p);
                }
            }
        }
        if !paths_in_directory.is_empty() {
            sort_module_paths(&mut paths_in_directory, use_case_sensitive_file_names);
            sorted_paths.append(&mut paths_in_directory);
        }
        let new_directory = tspath::get_directory_path(&directory);
        if new_directory == directory {
            break;
        }
        directory = new_directory;
    }
    if !all_file_names.is_empty() {
        let mut remaining: Vec<ModulePath> = all_file_names.values().cloned().collect();
        sort_module_paths(&mut remaining, use_case_sensitive_file_names);
        sorted_paths.append(&mut remaining);
    }
    sorted_paths
}

struct PkgJsonDirAttemptResult {
    module_file_to_try: String,
    package_root_path: String,
    blocked_by_exports: bool,
    verbatim_from_exports: bool,
}

fn try_directory_with_package_json(
    parts: &NodeModulePathParts,
    path_obj: &ModulePath,
) -> PkgJsonDirAttemptResult {
    let file_len = path_obj.file_name.len();
    let mut root_idx = parts.package_root_index;
    if root_idx > file_len {
        root_idx = file_len;
    }
    let package_root_path = path_obj.file_name[..root_idx].to_string();
    let name_index = (parts.package_root_index + 1).min(file_len);
    let file_name = &path_obj.file_name[name_index..];
    let module_file_to_try = path_obj.file_name.clone();
    // host 无 package.json 读取能力,等价于 Go packageJson == nil 分支
    if matches!(file_name, "index.d.ts" | "index.js" | "index.ts" | "index.tsx") {
        return PkgJsonDirAttemptResult {
            module_file_to_try,
            package_root_path,
            blocked_by_exports: false,
            verbatim_from_exports: false,
        };
    }
    PkgJsonDirAttemptResult {
        module_file_to_try,
        package_root_path: String::new(),
        blocked_by_exports: false,
        verbatim_from_exports: false,
    }
}

pub fn try_get_module_name_as_node_module(
    path_obj: &ModulePath,
    info: &Info,
    importing_source_file: &dyn SourceFileForSpecifierGeneration,
    host: &dyn ModuleSpecifierGenerationHost,
    options: &CompilerOptions,
    user_preferences: &UserPreferences,
    package_name_only: bool,
    _override_mode: ResolutionMode,
) -> String {
    let parts = match get_node_module_path_parts(&path_obj.file_name) {
        Some(parts) => parts,
        None => return String::new(),
    };

    // Simplify the full file path to something that can be resolved by Node.
    let preferences =
        get_module_specifier_preferences(user_preferences, host, options, importing_source_file, "");
    let allowed_endings = (preferences.get_allowed_endings_in_preferred_order)(ResolutionMode::None);

    let case_sensitive = host.use_case_sensitive_file_names();
    let mut module_specifier = path_obj.file_name.clone();
    let mut is_package_root_path = false;
    if !package_name_only {
        let mut package_root_index = parts.package_root_index as i64;
        let mut module_file_name = String::new();
        loop {
            // If the module could be imported by a directory name, use that directory's name
            let pkg_json = try_directory_with_package_json(&parts, path_obj);
            let module_file_to_try = pkg_json.module_file_to_try;
            let package_root_path = pkg_json.package_root_path;
            if pkg_json.blocked_by_exports {
                return String::new();
            }
            if pkg_json.verbatim_from_exports {
                return module_file_to_try;
            }
            if !package_root_path.is_empty() {
                module_specifier = package_root_path;
                is_package_root_path = true;
                break;
            }
            if module_file_name.is_empty() {
                module_file_name = module_file_to_try;
            }
            // try with next level of directory
            package_root_index = match index_after(
                &path_obj.file_name,
                "/",
                (package_root_index + 1).max(0) as usize,
            ) {
                Some(idx) => idx as i64,
                None => -1,
            };
            if package_root_index == -1 {
                module_specifier = process_ending(&module_file_name, &allowed_endings, options, host);
                break;
            }
        }
    }

    if path_obj.is_redirect && !is_package_root_path {
        return String::new();
    }

    // Get a path that's relative to node_modules or the importing file's path
    // if node_modules folder is in this folder or any of its parent folders, no need to keep it.
    let path_to_top_level_node_modules =
        &module_specifier[..parts.top_level_node_modules_index.min(module_specifier.len())];

    if !has_prefix(&info.source_directory, path_to_top_level_node_modules, case_sensitive) {
        return String::new();
    }

    // If the module was found in @types, get the actual Node package name
    let name_index = (parts.top_level_package_name_index + 1).min(module_specifier.len());
    let node_modules_directory_name = &module_specifier[name_index..];
    crate::module::get_package_name_from_types_package_name(node_modules_directory_name)
}

pub fn all_keys_start_with_dot(obj: &OrderedMap<String, ExportsOrImports>) -> bool {
    for k in obj.keys() {
        if !k.starts_with('.') {
            return false;
        }
    }
    true
}

pub fn get_package_name_from_directory(file_or_directory_path: &str) -> String {
    let idx = match file_or_directory_path.rfind("/node_modules/") {
        Some(idx) => idx,
        None => return String::new(),
    };

    let basename = &file_or_directory_path[idx + "/node_modules/".len()..];
    if basename.starts_with('.') {
        return String::new();
    }

    let next_slash = basename.find('/');
    let next_slash = match next_slash {
        Some(s) => s,
        None => return basename.to_string(),
    };

    if !basename.starts_with('@') || next_slash == basename.len() - 1 {
        return basename[..next_slash].to_string();
    }

    let second_slash = basename[next_slash + 1..].find('/');
    match second_slash {
        None => basename.to_string(),
        Some(second_slash) => basename[..next_slash + 1 + second_slash].to_string(),
    }
}
