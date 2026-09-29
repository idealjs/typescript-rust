#![allow(dead_code, unused_imports, unused_variables)]

use crate::ast::{Node, NodeList, SyntaxKind};

pub fn get_text_of_jsdoc_comment(comment: Option<&NodeList>) -> String {
    let Some(comment) = comment else {
        return String::new();
    };
    let mut b = String::new();
    for n in &comment.nodes {
        match n.kind {
            SyntaxKind::JSDocText => b.push_str(&n.text()),
            SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkCode | SyntaxKind::JSDocLinkPlain => {
                b.push_str(&super::m3i::get_text_of_node(n));
            }
            _ => {}
        }
    }
    b.trim_end().to_string()
}
