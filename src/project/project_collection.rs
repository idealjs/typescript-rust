//! Project collection (1:1 port of Go's `internal/project/projectcollection.go`).

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::tspath::Path;

use super::config_file_registry::ConfigFileRegistry;
use super::overlay_fs::Overlay;
use super::project::{INFERRED_PROJECT_NAME, Project};

/// Tracks the projects and files that API clients have explicitly opened.
///
/// Go: `type APIState struct { ... }`.
#[derive(Clone, Default)]
pub struct APIState {
    pub open_projects: HashMap<Path, i32>,
    pub open_files: HashMap<Path, ApiOpenedFile>,
}

impl APIState {
    pub fn clone_shallow(&self) -> APIState {
        APIState {
            open_projects: self.open_projects.clone(),
            open_files: self.open_files.clone(),
        }
    }

    pub fn equals(&self, other: &APIState) -> bool {
        self.open_projects == other.open_projects && self.open_files == other.open_files
    }
}

/// Tracks a file kept open by API clients along with its ref count.
///
/// Go: `type apiOpenedFile struct { ... }`.
#[derive(Clone, Debug, PartialEq)]
pub struct ApiOpenedFile {
    pub file_name: String,
    pub ref_count: i32,
}

/// A collection of all loaded projects for a snapshot.
///
/// Go: `type ProjectCollection struct { ... }`.
pub struct ProjectCollection {
    pub to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    pub config_file_registry: Option<ConfigFileRegistry>,
    pub file_default_projects: HashMap<Path, Path>,
    pub configured_projects: HashMap<Path, Box<Project>>,
    pub open_files: HashSet<Path>,
    pub inferred_project: Option<Box<Project>>,
    pub api_state: APIState,
}

impl ProjectCollection {
    pub fn new(to_path: Box<dyn Fn(&str) -> Path + Send + Sync>) -> Self {
        ProjectCollection {
            to_path,
            config_file_registry: Some(ConfigFileRegistry::new()),
            file_default_projects: HashMap::new(),
            configured_projects: HashMap::new(),
            open_files: HashSet::new(),
            inferred_project: None,
            api_state: APIState::default(),
        }
    }

    pub fn config_file_registry(&self) -> Option<&ConfigFileRegistry> {
        self.config_file_registry.as_ref()
    }

    pub fn configured_project(&self, path: &Path) -> Option<&Project> {
        self.configured_projects.get(path).map(|p| p.as_ref())
    }

    pub fn get_project_by_path(&self, project_path: &Path) -> Option<&Project> {
        if let Some(project) = self.configured_projects.get(project_path) {
            return Some(project.as_ref());
        }
        if project_path.as_str() == INFERRED_PROJECT_NAME {
            return self.inferred_project.as_deref();
        }
        None
    }

    /// Returns all configured projects in a stable (sorted) order.
    pub fn configured_projects_vec(&self) -> Vec<&Project> {
        let mut projects: Vec<&Project> = self
            .configured_projects
            .values()
            .map(|p| p.as_ref())
            .collect();
        projects.sort_by(|a, b| a.name().cmp(b.name()));
        projects
    }

    /// Returns all projects including the inferred project.
    pub fn projects(&self) -> Vec<&Project> {
        let mut projects = self.configured_projects_vec();
        if let Some(inferred) = &self.inferred_project {
            projects.push(inferred.as_ref());
        }
        projects
    }

    pub fn inferred_project(&self) -> Option<&Project> {
        self.inferred_project.as_deref()
    }

    pub fn get_default_project(&self, _path: &Path) -> Option<&Project> {
        todo!("ProjectCollection::get_default_project requires full integration")
    }

    /// Creates a shallow copy.
    pub fn clone_shallow(&self) -> ProjectCollection {
        todo!("ProjectCollection::clone_shallow requires Box<Project> Clone")
    }
}

/// Computes the set of open file paths from the overlay map.
///
/// Go: `func openFilePaths(overlays map[...]) collections.Set[...]`.
pub fn open_file_paths(overlays: &HashMap<Path, Arc<Overlay>>) -> HashSet<Path> {
    overlays.keys().cloned().collect()
}
