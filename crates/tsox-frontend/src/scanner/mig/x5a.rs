#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_synthesized::node_is_missing;
use crate::ast::utilities_types::is_jsdoc_node;
use crate::scanner::is_conflict_marker_trivia::skip_trivia_ex;
use crate::scanner::is_jsx_line_break::SkipTriviaOptions;
use crate::ast::node_source_file::SourceFile;

pub fn get_token_pos_of_node(
    node: &Arc<Node>,
    source_file: &SourceFile,
    include_jsdoc: bool,
) -> usize { ::tsox_core::fntrace::enter("get_token_pos_of_node"); 
    if node_is_missing(Some(node)) {
        return node.pos();
    }
    if is_jsdoc_node(node) || node.kind == SyntaxKind::JsxText {
        return skip_trivia_ex(
            &source_file.text,
            node.pos(),
            &SkipTriviaOptions {
                stop_at_comments: true,
                ..SkipTriviaOptions::default()
            },
            None,
        );
    }
    if include_jsdoc {
        let jsdoc = node.jsdoc(source_file);
        if !jsdoc.is_empty() {
            return get_token_pos_of_node(&jsdoc[0], source_file, false);
        }
    }
    let in_jsdoc = node.flags.intersects(crate::ast::node_flags::NodeFlags::JSDoc);
    skip_trivia_ex(
        &source_file.text,
        node.pos(),
        &SkipTriviaOptions {
            in_jsdoc,
            ..SkipTriviaOptions::default()
        },
        None,
    )
}
