#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ast::{Node, NodeList, SyntaxKind};
use crate::parser::parsing_context::Parser;

impl Parser {
    pub fn is_javascript(&self) -> bool { ::tsox_core::fntrace::enter("is_javascript"); 
        self.javascript_file
    }
}

pub fn is_reserved_word(token: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_reserved_word"); 
    let lo = SyntaxKind::BreakKeyword as u32;
    let hi = SyntaxKind::WithKeyword as u32;
    let v = token as u32;
    lo <= v && v <= hi
}

// isMissingNodeList/createMissingList → 架构差异:缺失列表以 Option/missing 标记表达
pub fn is_missing_node_list(list: Option<&NodeList>) -> bool { ::tsox_core::fntrace::enter("is_missing_node_list"); 
    list.is_none()
}

pub fn is_jsdoc_like_text(text: &str) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_like_text"); 
    let b = text.as_bytes();
    b.len() >= 4 && b[1] == b'*' && b[2] == b'*' && b[3] != b'/'
}
