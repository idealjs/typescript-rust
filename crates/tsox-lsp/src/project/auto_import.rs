#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use tsox_compile::compiler::Program;
use tsox_core::tspath::Path;
use tsox_frontend::ast::SourceFile;
use tsox_tsoptions::vfs::FS;

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
    pub fn new(to_path: Box<dyn Fn(&str) -> Path + Send + Sync>) -> Self { ::tsox_core::fntrace::enter("new"); 
        AutoImportRegistry { _to_path: to_path }
    }

    pub fn is_prepared_for_importing_file(
        &self,
        _file_name: &str,
        _project_path: &Path,
        _prefs: &str,
    ) -> bool { ::tsox_core::fntrace::enter("is_prepared_for_importing_file"); 
        false
    }
}

pub struct AutoImportRegistryCloneHost {
    _files: Mutex<Vec<()>>,
    current_directory: String,
}

impl AutoImportRegistryCloneHost {
    pub fn new(current_directory: String) -> Self { ::tsox_core::fntrace::enter("new"); 
        AutoImportRegistryCloneHost {
            _files: Mutex::new(Vec::new()),
            current_directory,
        }
    }
}

impl RegistryCloneHost for AutoImportRegistryCloneHost {
    fn fs(&self) -> &dyn FS { ::tsox_core::fntrace::enter("fs"); 
        todo!("AutoImportRegistryCloneHost::fs requires snapshotFSBuilder integration")
    }

    fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        &self.current_directory
    }

    fn get_default_project(&self, _path: &Path) -> (Path, Option<Arc<Program>>) { ::tsox_core::fntrace::enter("get_default_project"); 
        (Path::default(), None)
    }

    fn get_package_json(&self, _file_name: &str) -> Option<PackageJsonInfoCacheEntry> { ::tsox_core::fntrace::enter("get_package_json"); 
        None
    }

    fn get_program_for_project(&self, _project_path: &Path) -> Option<Arc<Program>> { ::tsox_core::fntrace::enter("get_program_for_project"); 
        None
    }

    fn get_source_file(&self, _file_name: &str, _path: &Path) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        None
    }

    fn dispose(&self) { ::tsox_core::fntrace::enter("dispose"); 
        self._files.lock().unwrap().clear();
    }
}
