#![allow(dead_code)]

use crate::ls::lsconv_converters::Script;
use crate::lsp::lsproto_lsp::Location;
use tsox_core::core::text::TextPos;
use tsox_core::core::text::TextRange;

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
    fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }
    fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }
}

impl LanguageService {
    pub fn get_mapped_location(&self, _file_name: &str, _file_range: TextRange) -> Location { ::tsox_core::fntrace::enter("get_mapped_location"); 
        Location::default()
    }

    pub fn get_script(&self, file_name: &str) -> Option<ScriptInfo> { ::tsox_core::fntrace::enter("get_script"); 
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
    ) -> Option<DocumentPosition> { ::tsox_core::fntrace::enter("try_get_source_position"); 
        None
    }
}
