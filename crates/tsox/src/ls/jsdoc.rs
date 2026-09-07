#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::Checker;

use super::language_service::LanguageService;

#[derive(Debug, Clone)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: String,
}

impl LanguageService {
    pub fn get_symbol_documentation_comment(
        &self,
        _checker: &Checker,
        _symbol: &Arc<Symbol>,
    ) -> String {
        String::new()
    }

    pub fn get_symbol_jsdoc_tags(&self, _symbol: &Arc<Symbol>) -> Vec<JSDocTagInfo> {
        Vec::new()
    }
}

pub fn get_jsdoc(_node: &Arc<Node>) -> Option<Arc<Node>> {
    None
}

pub fn get_jsdoc_or_tag(_checker: &Checker, _node: &Arc<Node>) -> Option<Arc<Node>> {
    None
}

pub fn contains_typedef_tag(_jsdoc: &Arc<Node>) -> bool {
    false
}
