#![allow(unused_imports)]

use crate::checker::grammarchecks::*;

impl Checker {
    pub(crate) fn grammar_error_on_node(&mut self, node: &Arc<Node>, message: &Message) -> bool {
        self.grammar_error_on_node_with_args(node, message, &[])
    }

    pub(crate) fn grammar_error_on_node_with_args(
        &mut self,
        node: &Arc<Node>,
        message: &Message,
        args: &[String],
    ) -> bool {
        let file = self.current_file.clone();
        if file.as_ref().is_some_and(|f| f.has_parse_diagnostics) {
            return false;
        }
        // tsc 每节点 grammar 检查只跑一次；移植侧多入口重入时按 code+loc 去重
        let already = self.diagnostics.get_all().iter().any(|d| {
            d.code == message.code
                && d.loc.pos() == node.loc.pos()
                && d.file.as_ref().map(|f| f.file_name.as_str())
                    == file.as_ref().map(|f| f.file_name.as_str())
        });
        if already {
            return true;
        }
        let diagnostic =
            tsox_frontend::ast::Diagnostic::new(file, node.loc, *message, args.to_vec());
        self.diagnostics.add(diagnostic);
        true
    }

    pub(crate) fn grammar_error_at_pos(
        &mut self,
        _node_for_file: &Arc<Node>,
        start: usize,
        length: usize,
        message: &Message,
    ) -> bool {
        let file = self.current_file.clone();
        if file.as_ref().is_some_and(|f| f.has_parse_diagnostics) {
            return false;
        }
        let loc = tsox_core::core::text::TextRange::new(start, start + length);
        let diagnostic = tsox_frontend::ast::Diagnostic::new(file, loc, *message, Vec::new());
        self.diagnostics.add(diagnostic);
        true
    }

    pub(crate) fn grammar_error_on_first_token(
        &mut self,
        node: &Arc<Node>,
        message: &Message,
    ) -> bool {
        self.grammar_error_on_node(node, message)
    }
}
