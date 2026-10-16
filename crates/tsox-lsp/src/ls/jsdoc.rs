#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::Symbol;

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
    ) -> String { ::tsox_core::fntrace::enter("get_symbol_documentation_comment"); 
        String::new()
    }

    pub fn get_symbol_jsdoc_tags(&self, _symbol: &Arc<Symbol>) -> Vec<JSDocTagInfo> { ::tsox_core::fntrace::enter("get_symbol_jsdoc_tags"); 
        Vec::new()
    }
}

pub fn get_jsdoc(_node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_jsdoc"); 
    None
}

pub fn get_jsdoc_or_tag(_checker: &Checker, _node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_jsdoc_or_tag"); 
    None
}

pub fn contains_typedef_tag(_jsdoc: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("contains_typedef_tag"); 
    false
}
