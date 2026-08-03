//! JSDoc support (1:1 port of Go's `internal/ls/jsdoc.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::Checker;

use super::language_service::LanguageService;

/// JSDoc tag info (name + rendered text).
#[derive(Debug, Clone)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: String,
}

impl LanguageService {
    /// Render a symbol's documentation comment as plain text.
    ///
    /// Mirrors `GetSymbolDocumentationComment`.
    pub fn get_symbol_documentation_comment(
        &self,
        _checker: &Checker,
        _symbol: &Arc<Symbol>,
    ) -> String {
        // TODO: requires getDocumentationFromDeclaration
        String::new()
    }

    /// Collect a symbol's JSDoc tags.
    ///
    /// Mirrors `GetSymbolJSDocTags`.
    pub fn get_symbol_jsdoc_tags(&self, _symbol: &Arc<Symbol>) -> Vec<JSDocTagInfo> {
        // TODO: requires JSDoc tag extraction
        Vec::new()
    }
}

/// Get the last JSDoc node attached to a node.
///
/// Mirrors `getJSDoc`.
pub fn get_jsdoc(_node: &Arc<Node>) -> Option<Arc<Node>> {
    // TODO: requires node.JSDoc()
    None
}

/// Get the JSDoc or a matching JSDoc tag for a node.
///
/// Mirrors `getJSDocOrTag`.
pub fn get_jsdoc_or_tag(_checker: &Checker, _node: &Arc<Node>) -> Option<Arc<Node>> {
    // TODO: requires parameter/template-tag matching
    None
}

/// Check if a JSDoc node contains a `@typedef` or `@callback` tag.
///
/// Mirrors `containsTypedefTag`.
pub fn contains_typedef_tag(_jsdoc: &Arc<Node>) -> bool {
    // TODO: requires JSDoc tag traversal
    false
}
