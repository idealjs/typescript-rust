use std::collections::HashMap;
use std::sync::Arc;

use crate::collections::set::Set;
use crate::collections::syncmap::SyncMap;
use crate::core::tristate::Tristate;
use crate::ls::lsutil::user_preferences::UserPreferences;
use crate::tspath;

use super::export::Export;
use super::index::Index;
use super::{LogTree, RegistryCloneHost, ResolvedEntrypoint};

pub fn known_recursive_search_packages() -> Set<String> {
    let mut s = Set::new();
    for pkg in [
        "@material-ui/core",
        "@material-ui/icons",
        "@sap/cds",
        "@testing-library/react-native",
        "ajv",
        "asap",
        "async",
        "aws-sdk",
        "braintree-web",
        "core-js",
        "core-js-pure",
        "crypto-js",
        "cypress-mochawesome-reporter",
        "dd-trace",
        "dumi",
        "dva",
        "egg-mock",
        "electron-log",
        "es-abstract",
        "es6-promise",
        "eslint-config-taro",
        "expo",
        "expo-router",
        "flow-remove-types",
        "gatsby",
        "glamor",
        "gluegun",
        "graphology-indices",
        "graphology-traversal",
        "graphology-utils",
        "jest-expo",
        "lodash",
        "lodash-es",
        "moment",
        "mz",
        "next",
        "pdfjs-dist",
        "protobufjs",
        "react-app-polyfill",
        "react-dev-utils",
        "react-devtools-inline",
        "recast",
        "semver",
        "stylelint-config-html",
        "umi",
        "web3-provider-engine",
        "webpack",
    ] {
        s.add(pkg.to_string());
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum NewProgramStructure {
    #[default]
    False,
    SameFileNames,
    DifferentFileNames,
}

#[derive(Debug, Clone, Default)]
pub struct BucketBuildPreferences {
    pub file_exclude_patterns: Vec<String>,
    pub auto_import_entrypoint_directory_search: Tristate,
}

impl BucketBuildPreferences {
    pub fn from_user_preferences(_prefs: &UserPreferences) -> BucketBuildPreferences {

        BucketBuildPreferences::default()
    }

    pub fn equal(&self, other: &BucketBuildPreferences) -> bool {
        self.auto_import_entrypoint_directory_search
            == other.auto_import_entrypoint_directory_search
            && unordered_equal(&self.file_exclude_patterns, &other.file_exclude_patterns)
    }

    pub fn clone_prefs(&self) -> BucketBuildPreferences {
        BucketBuildPreferences {
            file_exclude_patterns: self.file_exclude_patterns.clone(),
            auto_import_entrypoint_directory_search: self.auto_import_entrypoint_directory_search,
        }
    }
}

fn unordered_equal(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut a_sorted = a.to_vec();
    let mut b_sorted = b.to_vec();
    a_sorted.sort();
    b_sorted.sort();
    a_sorted == b_sorted
}

#[derive(Debug, Clone, Default)]
pub struct BucketState {
    pub dirty_file: tspath::Path,
    pub multiple_files_dirty: bool,
    pub new_program_structure: NewProgramStructure,
    pub build_preferences: BucketBuildPreferences,
    pub dirty_packages: Option<Set<String>>,
    pub recursive_search_packages: Option<Set<String>>,
}

impl BucketState {
    pub fn dirty(&self) -> bool {
        self.multiple_files_dirty
            || !self.dirty_file.0.is_empty()
            || self.new_program_structure > NewProgramStructure::False
            || self.dirty_packages.as_ref().map(|s| s.len()).unwrap_or(0) > 0
    }

    pub fn dirty_file_path(&self) -> tspath::Path {
        if self.multiple_files_dirty {
            tspath::Path(String::new())
        } else {
            self.dirty_file.clone()
        }
    }

    pub fn dirty_packages(&self) -> Option<&Set<String>> {
        if self.multiple_files_dirty {
            None
        } else {
            self.dirty_packages.as_ref()
        }
    }

    pub fn recursive_search_packages(&self) -> Option<&Set<String>> {
        self.recursive_search_packages.as_ref()
    }

    pub fn possibly_needs_rebuild_for_file(
        &self,
        file: &tspath::Path,
        preferences: &UserPreferences,
    ) -> bool {
        self.new_program_structure > NewProgramStructure::False
            || self.has_dirty_file_besides(file)
            || !self
                .build_preferences
                .equal(&BucketBuildPreferences::from_user_preferences(preferences))
            || self.dirty_packages.as_ref().map(|s| s.len()).unwrap_or(0) > 0
    }

    pub fn has_dirty_file_besides(&self, file: &tspath::Path) -> bool {
        self.multiple_files_dirty || (!self.dirty_file.0.is_empty() && &self.dirty_file != file)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegistryBucket {
    pub state: BucketState,
    pub paths: HashMap<tspath::Path, String>,
    pub package_files: HashMap<String, HashMap<tspath::Path, String>>,
    pub resolved_package_names: Option<Set<String>>,
    pub dependency_names: Option<Set<String>>,
    pub ambient_module_names: HashMap<String, Vec<String>>,
    pub index: Option<Index<Export>>,
}

impl RegistryBucket {

    pub fn new() -> RegistryBucket {
        RegistryBucket {
            state: BucketState {
                multiple_files_dirty: true,
                new_program_structure: NewProgramStructure::DifferentFileNames,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn mark_project_file_dirty(&mut self, file: tspath::Path) {
        if self.state.has_dirty_file_besides(&file) {
            self.state.multiple_files_dirty = true;
        } else {
            self.state.dirty_file = file;
        }
    }

    pub fn mark_node_modules_dirty(&mut self, package_name: &str) {
        if self.state.multiple_files_dirty {
            return;
        }
        if package_name.is_empty() {
            self.state.multiple_files_dirty = true;
            return;
        }
        if self.state.dirty_packages.is_none() {
            self.state.dirty_packages = Some(Set::new());
        }
        if let Some(ref mut dp) = self.state.dirty_packages {
            dp.add(package_name.to_string());
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Directory {
    pub name: String,
    pub package_json: Option<PackageJsonInfoStub>,
    pub has_node_modules: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PackageJsonInfoStub {
    pub exists: bool,
    pub parseable: bool,
}

impl PackageJsonInfoStub {
    pub fn exists(&self) -> bool {
        self.exists
    }
}

pub struct Registry {
    pub to_path: Box<dyn Fn(&str) -> tspath::Path + Send + Sync>,
    pub user_preferences: UserPreferences,
    pub directories: HashMap<tspath::Path, Directory>,
    pub node_modules: HashMap<tspath::Path, RegistryBucket>,
    pub projects: HashMap<tspath::Path, RegistryBucket>,
    pub unique_package_count: usize,
    pub entrypoints: HashMap<tspath::Path, Vec<Arc<ResolvedEntrypoint>>>,
    pub specifier_cache: HashMap<tspath::Path, SyncMap<tspath::Path, String>>,
}

impl Registry {

    pub fn new(
        to_path: Box<dyn Fn(&str) -> tspath::Path + Send + Sync>,
        preferences: UserPreferences,
    ) -> Registry {
        Registry {
            to_path,
            user_preferences: preferences,
            directories: HashMap::new(),
            node_modules: HashMap::new(),
            projects: HashMap::new(),
            unique_package_count: 0,
            entrypoints: HashMap::new(),
            specifier_cache: HashMap::new(),
        }
    }

    pub fn is_prepared_for_importing_file(
        &self,
        file_name: &str,
        project_path: &tspath::Path,
        preferences: &UserPreferences,
    ) -> bool {
        let project_bucket = match self.projects.get(project_path) {
            Some(b) => b,
            None => return false,
        };
        let path = (self.to_path)(file_name);
        if project_bucket
            .state
            .possibly_needs_rebuild_for_file(&path, preferences)
        {
            return false;
        }

        let mut dir_path = path.get_directory_path();
        loop {
            if let Some(dir_bucket) = self.node_modules.get(&dir_path) {
                if dir_bucket
                    .state
                    .possibly_needs_rebuild_for_file(&path, preferences)
                {
                    return false;
                }
            }
            let parent = dir_path.get_directory_path();
            if parent == dir_path {
                break;
            }
            dir_path = parent;
        }
        true
    }

    pub fn node_modules_directories(&self) -> HashMap<tspath::Path, String> {
        let mut dirs = HashMap::new();
        for (dir_path, dir) in &self.directories {
            if dir.has_node_modules {
                let path = tspath::Path(tspath::combine_paths(&dir_path.0, &["node_modules"]));
                let name = tspath::combine_paths(&dir.name, &["node_modules"]);
                dirs.insert(path, name);
            }
        }
        dirs
    }

    pub fn clone_registry(
        &self,
        _change: &RegistryChange,
        _host: &dyn RegistryCloneHost,
        _logger: Option<&LogTree>,
    ) -> Result<Registry, String> {

        todo!("Registry::clone_registry requires registryBuilder infrastructure")
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let mut stats = CacheStats {
            unique_package_count: self.unique_package_count,
            ..Default::default()
        };

        for (path, bucket) in &self.projects {
            let export_count = bucket
                .index
                .as_ref()
                .map(|idx| idx.entries.len())
                .unwrap_or(0);
            stats.project_buckets.push(BucketStats {
                path: path.clone(),
                export_count,
                file_count: bucket.paths.len(),
                state: bucket.state.clone(),
                dependency_names: bucket.dependency_names.clone(),
                package_names: None,
            });
        }

        for (path, bucket) in &self.node_modules {
            let export_count = bucket
                .index
                .as_ref()
                .map(|idx| idx.entries.len())
                .unwrap_or(0);
            let mut package_names = Set::new();
            let mut file_count = 0;
            if !bucket.package_files.is_empty() {
                for (name, paths) in &bucket.package_files {
                    package_names.add(name.clone());
                    file_count += paths.len();
                }
            }
            stats.node_modules_buckets.push(BucketStats {
                path: path.clone(),
                export_count,
                file_count,
                state: bucket.state.clone(),
                dependency_names: bucket.dependency_names.clone(),
                package_names: Some(package_names),
            });
        }

        stats
            .project_buckets
            .sort_by(|a, b| a.path.0.cmp(&b.path.0));
        stats
            .node_modules_buckets
            .sort_by(|a, b| a.path.0.cmp(&b.path.0));

        stats
    }
}

#[derive(Debug, Clone, Default)]
pub struct BucketStats {
    pub path: tspath::Path,
    pub export_count: usize,
    pub file_count: usize,
    pub state: BucketState,
    pub dependency_names: Option<Set<String>>,
    pub package_names: Option<Set<String>>,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub project_buckets: Vec<BucketStats>,
    pub node_modules_buckets: Vec<BucketStats>,
    pub unique_package_count: usize,
}

#[derive(Debug, Default)]
pub struct RegistryChange {
    pub requested_file: tspath::Path,
    pub open_files: HashMap<tspath::Path, String>,
    pub changed: Set<crate::lsp::lsproto::DocumentUri>,
    pub created: Set<crate::lsp::lsproto::DocumentUri>,
    pub deleted: Set<crate::lsp::lsproto::DocumentUri>,
    pub rebuilt_programs: HashMap<tspath::Path, bool>,
    pub user_preferences: Option<UserPreferences>,
}
