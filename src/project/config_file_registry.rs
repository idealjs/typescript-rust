#![allow(dead_code)]

use std::collections::HashMap;

use crate::tsoptions::ParsedCommandLine;
use crate::tspath::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingReload {
    None,
    FileNames,
    Full,
}

#[derive(Clone)]
pub struct ConfigFileEntry {
    pub file_name: String,
    pub pending_reload: PendingReload,
    pub command_line: Option<ParsedCommandLine>,

    pub retaining_projects: HashMap<Path, ()>,

    pub retaining_open_files: HashMap<Path, ()>,

    pub retaining_configs: HashMap<Path, ()>,
}

impl ConfigFileEntry {
    pub fn new(has_relative_pattern_capability: bool, file_name: String) -> Self {
        let _ = has_relative_pattern_capability;
        ConfigFileEntry {
            file_name,
            pending_reload: PendingReload::Full,
            command_line: None,
            retaining_projects: HashMap::new(),
            retaining_open_files: HashMap::new(),
            retaining_configs: HashMap::new(),
        }
    }

    pub fn new_extended(file_name: String, extending_config_path: Path) -> Self {
        let mut entry = ConfigFileEntry {
            file_name,
            pending_reload: PendingReload::Full,
            command_line: None,
            retaining_projects: HashMap::new(),
            retaining_open_files: HashMap::new(),
            retaining_configs: HashMap::new(),
        };
        entry.retaining_configs.insert(extending_config_path, ());
        entry
    }
}

#[derive(Clone)]
pub struct ConfigFileNames {
    pub nearest_config_file_name: String,
    pub ancestors: HashMap<String, String>,
}

impl ConfigFileNames {
    pub fn new(nearest_config_file_name: String) -> Self {
        ConfigFileNames {
            nearest_config_file_name,
            ancestors: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct ConfigFileRegistry {
    pub configs: HashMap<Path, ConfigFileEntry>,
    pub config_file_names: HashMap<Path, ConfigFileNames>,
    pub custom_config_file_name: String,
}

impl ConfigFileRegistry {
    pub fn new() -> Self {
        ConfigFileRegistry::default()
    }

    pub fn get_config(&self, path: &Path) -> Option<&ParsedCommandLine> {
        self.configs.get(path).and_then(|e| e.command_line.as_ref())
    }

    pub fn get_config_file_name(&self, path: &Path) -> &str {
        self.config_file_names
            .get(path)
            .map(|e| e.nearest_config_file_name.as_str())
            .unwrap_or("")
    }

    pub fn get_ancestor_config_file_name(&self, path: &Path, higher_than_config: &str) -> &str {
        self.config_file_names
            .get(path)
            .and_then(|e| e.ancestors.get(higher_than_config))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn clone_shallow(&self) -> ConfigFileRegistry {
        ConfigFileRegistry {
            configs: self.configs.clone(),
            config_file_names: self.config_file_names.clone(),
            custom_config_file_name: self.custom_config_file_name.clone(),
        }
    }
}
