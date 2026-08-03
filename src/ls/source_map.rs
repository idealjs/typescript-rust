//! Source-map location mapping (1:1 port of Go's `internal/ls/source_map.go`).

#![allow(dead_code)]

use crate::core::text::{TextPos, TextRange};
use crate::ls::lsconv::converters::{Converters, Script};
use crate::lsp::lsproto::lsp::Location;

use super::language_service::LanguageService;

/// A document position (file name + offset).
#[derive(Debug, Clone)]
pub struct DocumentPosition {
    pub file_name: String,
    pub pos: TextPos,
}

/// Simple script implementation for source-map text lookups.
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
    /// Map a location through source maps (e.g. `.d.ts` → source).
    ///
    /// Mirrors `getMappedLocation`.
    pub fn get_mapped_location(&self, _file_name: &str, _file_range: TextRange) -> Location {
        // TODO: requires sourcemap DocumentPositionMapper
        Location::default()
    }

    /// Get the script for a file name.
    ///
    /// Mirrors `getScript`.
    pub fn get_script(&self, file_name: &str) -> Option<ScriptInfo> {
        let text = self.read_file(file_name)?;
        Some(ScriptInfo {
            file_name: file_name.to_string(),
            text,
        })
    }

    /// Try to get the source position for a generated position.
    ///
    /// Mirrors `tryGetSourcePosition`.
    pub fn try_get_source_position(
        &self,
        _file_name: &str,
        _position: TextPos,
    ) -> Option<DocumentPosition> {
        // TODO: requires sourcemap DocumentPositionMapper
        None
    }
}
