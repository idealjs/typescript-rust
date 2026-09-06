#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::SourceFile;
use crate::compiler;
use crate::core::text::TextRange;
use crate::ls::lsconv::converters::Converters;
use crate::ls::lsutil::{FormatCodeSettings, UserPreferences};
use crate::lsp::lsproto::lsp::{DocumentUri, Location};
use crate::tspath;

use super::host::{AutoImportRegistry, EcmaLineInfo, Host};

pub struct DocumentPositionMapper;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileRange {
    pub file_name: String,
    pub file_range: TextRange,
}

pub struct LanguageService {
    pub(crate) project_path: tspath::Path,
    pub(crate) host: Box<dyn Host>,
    pub(crate) active_config: UserPreferences,
    pub(crate) program: Arc<compiler::Program>,
    pub(crate) converters: Converters,
    pub(crate) document_position_mappers: HashMap<String, DocumentPositionMapper>,
}

impl LanguageService {

    pub fn new(
        project_path: tspath::Path,
        program: Arc<compiler::Program>,
        host: Box<dyn Host>,
        active_file: &str,
    ) -> Self {
        let converters = host.converters();
        let active_config = host.get_preferences(active_file);
        LanguageService {
            project_path,
            host,
            program,
            converters,
            active_config,
            document_position_mappers: HashMap::new(),
        }
    }

    pub fn to_path(&self, file_name: &str) -> tspath::Path {
        tspath::to_path(
            file_name,
            "",
            self.use_case_sensitive_file_names(),
        )
    }

    pub fn get_program(&self) -> Arc<compiler::Program> {
        Arc::clone(&self.program)
    }

    pub fn user_preferences(&self) -> &UserPreferences {
        &self.active_config
    }

    pub fn format_options(&self) -> &FormatCodeSettings {
        &self.active_config.format_code_settings
    }

    pub fn try_get_program_and_file(
        &self,
        file_name: &str,
    ) -> (Arc<compiler::Program>, Option<Arc<SourceFile>>) {
        let program = self.get_program();
        let file = program.get_source_file(file_name);
        (program, file)
    }

    pub fn get_program_and_file(
        &self,
        document_uri: &DocumentUri,
    ) -> (Arc<compiler::Program>, Arc<SourceFile>) {
        let file_name = document_uri.file_name();
        let (program, file) = self.try_get_program_and_file(&file_name);
        let file = file.unwrap_or_else(|| panic!("file not found: {file_name}"));
        (program, file)
    }

    pub fn get_document_position_mapper(
        &self,
        _file_name: &str,
    ) -> Option<&DocumentPositionMapper> {

        None
    }

    pub fn read_file(&self, file_name: &str) -> Option<String> {
        self.host.read_file(file_name)
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.host.use_case_sensitive_file_names()
    }

    pub fn get_ecma_line_info(&self, file_name: &str) -> Option<EcmaLineInfo> {
        self.host.get_ecma_line_info(file_name)
    }

    pub fn get_auto_import_registry(&self) -> AutoImportRegistry {
        self.host.auto_import_registry()
    }

    pub fn create_lsp_range_from_bounds(
        &self,
        pos: usize,
        end: usize,
        script: &dyn crate::ls::lsconv::converters::Script,
    ) -> crate::lsp::lsproto::lsp::Range {
        self.converters.to_lsp_range(script, pos, end)
    }

    pub fn create_lsp_location_from_bounds(
        &self,
        script: &dyn crate::ls::lsconv::converters::Script,
        pos: usize,
        end: usize,
    ) -> Location {
        self.converters.to_lsp_location(script, pos, end)
    }

    pub fn directory_exists(&self, path: &str) -> bool {
        self.host.directory_exists(path)
    }

    pub fn read_directory(
        &self,
        path: &str,
        extensions: &[String],
        includes: &[String],
    ) -> Vec<String> {
        self.host.read_directory(
            "",
            path,
            extensions,
            &[],
            includes,
            -1,
        )
    }

    pub fn get_directories(&self, path: &str) -> Vec<String> {
        self.host.get_directories(path)
    }
}

pub struct ScriptInfo {
    pub file_name: String,
    pub text: String,
}

impl crate::ls::lsconv::converters::Script for ScriptInfo {
    fn file_name(&self) -> &str {
        &self.file_name
    }
    fn text(&self) -> &str {
        &self.text
    }
}
