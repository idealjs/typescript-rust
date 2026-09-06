#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use crate::ast::SourceFile;
use crate::compiler::Program;
use crate::tspath::Path;
use crate::vfs::FS;

#[derive(Clone, Debug, Default)]
pub struct PackageJsonInfoCacheEntry {
    pub directory_exists: bool,
    pub package_directory: String,
}

pub trait RegistryCloneHost: Send + Sync {
    fn fs(&self) -> &dyn FS;
    fn get_current_directory(&self) -> &str;
    fn get_default_project(&self, path: &Path) -> (Path, Option<Arc<Program>>);
    fn get_package_json(&self, file_name: &str) -> Option<PackageJsonInfoCacheEntry>;
    fn get_program_for_project(&self, project_path: &Path) -> Option<Arc<Program>>;
    fn get_source_file(&self, file_name: &str, path: &Path) -> Option<Arc<SourceFile>>;
    fn dispose(&self);
}

pub struct AutoImportRegistry {
    _to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
}

impl AutoImportRegistry {
    pub fn new(to_path: Box<dyn Fn(&str) -> Path + Send + Sync>) -> Self {
        AutoImportRegistry { _to_path: to_path }
    }

    pub fn is_prepared_for_importing_file(
        &self,
        _file_name: &str,
        _project_path: &Path,
        _prefs: &str,
    ) -> bool {
        false
    }
}

pub struct AutoImportRegistryCloneHost {
    _files: Mutex<Vec<()>>,
    current_directory: String,
}

impl AutoImportRegistryCloneHost {
    pub fn new(current_directory: String) -> Self {
        AutoImportRegistryCloneHost {
            _files: Mutex::new(Vec::new()),
            current_directory,
        }
    }
}

impl RegistryCloneHost for AutoImportRegistryCloneHost {
    fn fs(&self) -> &dyn FS {
        todo!("AutoImportRegistryCloneHost::fs requires snapshotFSBuilder integration")
    }

    fn get_current_directory(&self) -> &str {
        &self.current_directory
    }

    fn get_default_project(&self, _path: &Path) -> (Path, Option<Arc<Program>>) {
        (Path::default(), None)
    }

    fn get_package_json(&self, _file_name: &str) -> Option<PackageJsonInfoCacheEntry> {
        None
    }

    fn get_program_for_project(&self, _project_path: &Path) -> Option<Arc<Program>> {
        None
    }

    fn get_source_file(&self, _file_name: &str, _path: &Path) -> Option<Arc<SourceFile>> {
        None
    }

    fn dispose(&self) {
        self._files.lock().unwrap().clear();
    }
}
