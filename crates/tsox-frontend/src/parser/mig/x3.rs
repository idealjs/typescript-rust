use crate::parser::parsing_context::Parser;
use crate::ast;
use crate::ast::node_data_generated::NodeData;
use crate::ast::node_flags::NodeFlags;
use std::cell::Cell;
use tsox_core::core::text::TextRange;

thread_local! {
    /// Go parser.hasParseError：parse_error_at_range 置位,finish_node 消费。
    /// 本移植的 parse_error_at_range 尚未接线置位(见 format/span/process.rs 同类说明)
    static HAS_PARSE_ERROR: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn set_has_parse_error() {
    HAS_PARSE_ERROR.with(|c| c.set(true));
}

impl Parser {
    pub(crate) fn finish_node(&mut self, node: &mut ast::Node, pos: usize) -> ast::Node {
        let end = self.node_pos();
        self.finish_node_with_end(node, pos, end)
    }

    pub(crate) fn finish_node_with_end(
        &mut self,
        node: &mut ast::Node,
        pos: usize,
        end: usize,
    ) -> ast::Node {
        node.loc = TextRange::new(pos, end);
        node.flags |= self.context_flags_now();
        if HAS_PARSE_ERROR.with(Cell::get) {
            node.flags |= NodeFlags::ThisNodeHasError;
            HAS_PARSE_ERROR.with(|c| c.set(false));
        }
        std::mem::replace(node, ast::Node::new(ast::SyntaxKind::Unknown, NodeData::Token))
    }
}
