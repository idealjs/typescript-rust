#![allow(dead_code)]

use crate::lsp::lsproto_lsp::DocumentUri;
use tsox_compile::compiler::Program;

use super::language_service::LanguageService;
use super::types::RenameFile;

pub type PathUpdater = Box<dyn Fn(&str) -> (String, bool)>;

pub struct ToImport {
    pub new_file_name: String,
    pub updated: bool,
}

impl LanguageService {
    pub fn get_edits_for_file_rename(
        &self,
        _old_uri: &DocumentUri,
        _new_uri: &DocumentUri,
    ) -> Vec<RenameFile> { ::tsox_core::fntrace::enter("get_edits_for_file_rename"); 
        Vec::new()
    }

    pub fn create_path_updater(&self, _old_path: &str, _new_path: &str) -> PathUpdater { ::tsox_core::fntrace::enter("create_path_updater"); 
        Box::new(|path: &str| (path.to_string(), false))
    }

    pub fn update_tsconfig_files(
        &self,
        _program: &Program,
        _old_to_new: &PathUpdater,
        _old_path: &str,
        _new_path: &str,
    ) { ::tsox_core::fntrace::enter("update_tsconfig_files"); 
    }

    pub fn update_imports_for_file_rename(&self, _program: &Program, _old_to_new: &PathUpdater) { ::tsox_core::fntrace::enter("update_imports_for_file_rename"); }
}
