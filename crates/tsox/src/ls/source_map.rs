#![allow(dead_code)]

use crate::core::text::{TextPos, TextRange};
use crate::ls::lsconv::converters::Script;
use crate::lsp::lsproto::lsp::Location;

use super::language_service::LanguageService;

#[derive(Debug, Clone)]
pub struct DocumentPosition {
    pub file_name: String,
    pub pos: TextPos,
}

pub struct ScriptInfo {
    pub file_name: String,
    pub text: String,
}

impl Script for ScriptInfo {
    fn file_name(&self) -> &str {
        &self.file_name
    }
    fn text(&self) -> &str {
        &self.text
    }
}

impl LanguageService {
    pub fn get_mapped_location(&self, _file_name: &str, _file_range: TextRange) -> Location {
        Location::default()
    }

    pub fn get_script(&self, file_name: &str) -> Option<ScriptInfo> {
        let text = self.read_file(file_name)?;
        Some(ScriptInfo {
            file_name: file_name.to_string(),
            text,
        })
    }

    pub fn try_get_source_position(
        &self,
        _file_name: &str,
        _position: TextPos,
    ) -> Option<DocumentPosition> {
        None
    }
}
