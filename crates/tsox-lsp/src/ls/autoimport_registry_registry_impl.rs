use crate::ls::autoimport_registry_bucket::BucketState;
use crate::ls::autoimport_registry_bucket::RegistryBucket;
use crate::ls::lsutil_user_preferences::UserPreferences;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;

use crate::ls::autoimport::{LogTree, RegistryCloneHost, ResolvedEntrypoint};

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
    pub to_path: Box<dyn Fn(&str) -> tsox_core::tspath::Path + Send + Sync>,
    pub user_preferences: UserPreferences,
    pub directories: HashMap<tsox_core::tspath::Path, Directory>,
    pub node_modules: HashMap<tsox_core::tspath::Path, RegistryBucket>,
    pub projects: HashMap<tsox_core::tspath::Path, RegistryBucket>,
    pub unique_package_count: usize,
    pub entrypoints: HashMap<tsox_core::tspath::Path, Vec<Arc<ResolvedEntrypoint>>>,
    pub specifier_cache: HashMap<tsox_core::tspath::Path, SyncMap<tsox_core::tspath::Path, String>>,
}

impl Registry {
    pub fn new(
        to_path: Box<dyn Fn(&str) -> tsox_core::tspath::Path + Send + Sync>,
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
        project_path: &tsox_core::tspath::Path,
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

    pub fn node_modules_directories(&self) -> HashMap<tsox_core::tspath::Path, String> {
        let mut dirs = HashMap::new();
        for (dir_path, dir) in &self.directories {
            if dir.has_node_modules {
                let path = tsox_core::tspath::Path(tsox_core::tspath::combine_paths(
                    &dir_path.0,
                    &["node_modules"],
                ));
                let name = tsox_core::tspath::combine_paths(&dir.name, &["node_modules"]);
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
    pub path: tsox_core::tspath::Path,
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
    pub requested_file: tsox_core::tspath::Path,
    pub open_files: HashMap<tsox_core::tspath::Path, String>,
    pub changed: Set<crate::lsp::lsproto::DocumentUri>,
    pub created: Set<crate::lsp::lsproto::DocumentUri>,
    pub deleted: Set<crate::lsp::lsproto::DocumentUri>,
    pub rebuilt_programs: HashMap<tsox_core::tspath::Path, bool>,
    pub user_preferences: Option<UserPreferences>,
}
