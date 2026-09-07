#![allow(unused_imports)]

use super::*;

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
        let diagnostic = crate::ast::Diagnostic::new(file, node.loc, *message, args.to_vec());
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
        let loc = crate::core::text::TextRange::new(start, start + length);
        let diagnostic = crate::ast::Diagnostic::new(file, loc, *message, Vec::new());
        self.diagnostics.add(diagnostic);
        true
    }

    pub(crate) fn grammar_error_on_first_token(&mut self, node: &Arc<Node>, message: &Message) -> bool {
        self.grammar_error_on_node(node, message)
    }

}
