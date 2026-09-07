#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile, Symbol};

use super::language_service::LanguageService;

#[derive(Debug)]
pub enum LsError {
    NoSourceFile(String),
    NoTokenAtPosition(String),
}

impl std::fmt::Display for LsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LsError::NoSourceFile(name) => write!(f, "source file not found: {name}"),
            LsError::NoTokenAtPosition(loc) => {
                write!(f, "no token found at position: {loc}")
            }
        }
    }
}

impl std::error::Error for LsError {}

impl LanguageService {
    pub fn get_symbol_at_position(
        &self,
        _file_name: &str,
        _position: usize,
    ) -> Result<Option<Arc<Symbol>>, LsError> {
        let (_program, file) = self.try_get_program_and_file(_file_name);
        let file = file.ok_or_else(|| LsError::NoSourceFile(_file_name.to_string()))?;
        let _ = file;

        Ok(None)
    }

    pub fn get_symbol_at_location(&self, _node: &Arc<Node>) -> Option<Arc<Symbol>> {
        None
    }

    pub fn get_type_of_symbol(&self, _symbol: &Arc<Symbol>) -> Option<Arc<crate::checker::Type>> {
        None
    }
}

pub fn get_source_file_of_node(_node: &Arc<Node>) -> Option<Arc<SourceFile>> {
    None
}
