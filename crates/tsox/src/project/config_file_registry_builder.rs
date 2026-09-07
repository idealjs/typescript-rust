#![allow(dead_code)]

use std::collections::HashSet;

use crate::tspath::Path;

use super::config_file_registry::ConfigFileRegistry;
use super::file_change::FileChangeSummary;

#[derive(Default)]
pub struct ChangeFileResult {
    pub affected_projects: HashSet<Path>,
    pub affected_files: HashSet<Path>,
}

impl ChangeFileResult {
    pub fn is_empty(&self) -> bool {
        self.affected_projects.is_empty() && self.affected_files.is_empty()
    }
}

pub struct ConfigFileRegistryBuilder {
    pub has_relative_pattern_capability: bool,
    pub snapshot_id: u64,
    pub session_options: super::compiler_host::SessionOptions,
    pub custom_config_file_name: String,
    pub custom_config_file_name_changed: bool,
    pub base: ConfigFileRegistry,
}

impl ConfigFileRegistryBuilder {
    pub fn new(
        has_relative_pattern_capability: bool,
        old_config_file_registry: ConfigFileRegistry,
        snapshot_id: u64,
        session_options: super::compiler_host::SessionOptions,
        custom_config_file_name: String,
    ) -> Self {
        let custom_config_file_name_changed =
            custom_config_file_name != old_config_file_registry.custom_config_file_name;
        ConfigFileRegistryBuilder {
            has_relative_pattern_capability,
            snapshot_id,
            session_options,
            custom_config_file_name,
            custom_config_file_name_changed,
            base: old_config_file_registry,
        }
    }

    pub fn finalize(&self) -> ConfigFileRegistry {
        if !self.custom_config_file_name_changed {
            return self.base.clone_shallow();
        }
        let mut registry = self.base.clone_shallow();
        registry.custom_config_file_name = self.custom_config_file_name.clone();
        registry
    }

    pub fn did_change_files(&self, _summary: &FileChangeSummary) -> ChangeFileResult {
        ChangeFileResult::default()
    }

    pub fn did_change_custom_config_file_name(&self) -> bool {
        self.custom_config_file_name_changed
    }

    pub fn cleanup(&self) {}

    pub fn is_config_base_name(&self, base_name: &str) -> bool {
        base_name == "tsconfig.json"
            || base_name == "jsconfig.json"
            || (!self.custom_config_file_name.is_empty()
                && base_name == self.custom_config_file_name)
    }
}
