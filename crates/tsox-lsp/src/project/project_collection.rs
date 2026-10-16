#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tsox_core::tspath::Path;

use super::config_file_registry::ConfigFileRegistry;
use super::overlay_fs::Overlay;
use super::project::{INFERRED_PROJECT_NAME, Project};

#[derive(Clone, Default)]
pub struct APIState {
    pub open_projects: HashMap<Path, i32>,
    pub open_files: HashMap<Path, ApiOpenedFile>,
}

impl APIState {
    pub fn clone_shallow(&self) -> APIState { ::tsox_core::fntrace::enter("clone_shallow"); 
        APIState {
            open_projects: self.open_projects.clone(),
            open_files: self.open_files.clone(),
        }
    }

    pub fn equals(&self, other: &APIState) -> bool { ::tsox_core::fntrace::enter("equals"); 
        self.open_projects == other.open_projects && self.open_files == other.open_files
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApiOpenedFile {
    pub file_name: String,
    pub ref_count: i32,
}

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
    pub fn new(to_path: Box<dyn Fn(&str) -> Path + Send + Sync>) -> Self { ::tsox_core::fntrace::enter("new"); 
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

    pub fn config_file_registry(&self) -> Option<&ConfigFileRegistry> { ::tsox_core::fntrace::enter("config_file_registry"); 
        self.config_file_registry.as_ref()
    }

    pub fn configured_project(&self, path: &Path) -> Option<&Project> { ::tsox_core::fntrace::enter("configured_project"); 
        self.configured_projects.get(path).map(|p| p.as_ref())
    }

    pub fn get_project_by_path(&self, project_path: &Path) -> Option<&Project> { ::tsox_core::fntrace::enter("get_project_by_path"); 
        if let Some(project) = self.configured_projects.get(project_path) {
            return Some(project.as_ref());
        }
        if project_path.as_str() == INFERRED_PROJECT_NAME {
            return self.inferred_project.as_deref();
        }
        None
    }

    pub fn configured_projects_vec(&self) -> Vec<&Project> { ::tsox_core::fntrace::enter("configured_projects_vec"); 
        let mut projects: Vec<&Project> = self
            .configured_projects
            .values()
            .map(|p| p.as_ref())
            .collect();
        projects.sort_by(|a, b| a.name().cmp(b.name()));
        projects
    }

    pub fn projects(&self) -> Vec<&Project> { ::tsox_core::fntrace::enter("projects"); 
        let mut projects = self.configured_projects_vec();
        if let Some(inferred) = &self.inferred_project {
            projects.push(inferred.as_ref());
        }
        projects
    }

    pub fn inferred_project(&self) -> Option<&Project> { ::tsox_core::fntrace::enter("inferred_project"); 
        self.inferred_project.as_deref()
    }

    pub fn get_default_project(&self, _path: &Path) -> Option<&Project> { ::tsox_core::fntrace::enter("get_default_project"); 
        todo!("ProjectCollection::get_default_project requires full integration")
    }

    pub fn clone_shallow(&self) -> ProjectCollection { ::tsox_core::fntrace::enter("clone_shallow"); 
        todo!("ProjectCollection::clone_shallow requires Box<Project> Clone")
    }
}

pub fn open_file_paths(overlays: &HashMap<Path, Arc<Overlay>>) -> HashSet<Path> { ::tsox_core::fntrace::enter("open_file_paths"); 
    overlays.keys().cloned().collect()
}
