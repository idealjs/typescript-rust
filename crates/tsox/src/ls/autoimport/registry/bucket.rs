use crate::collections::set::Set;
use crate::core::tristate::Tristate;
use crate::ls::lsutil::user_preferences::UserPreferences;
use crate::tspath;

use crate::ls::autoimport::export::Export;
use crate::ls::autoimport::index::Index;

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

use std::collections::HashMap;
