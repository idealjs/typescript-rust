//! Go span.go processNode / processChildNodes / processChildNode /
//! computeIndentation / tryComputeIndentationForListItem。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use tsox_core::core::text::TextRange;

use crate::format::indenter::{
    argument_starts_on_same_line_as_previous_argument,
    child_is_unindented_branch_of_conditional_expression,
    child_starts_on_same_line_with_else_in_if_statement, should_indent_child_node, DynamicIndenter,
    IndenterRef,
};
use crate::format::scanner::FormattingScanner;
use crate::format::span::FormatSpanWorker;

use super::super::indenter::has_decorators;
use super::super::lists;
use super::super::util;

impl FormatSpanWorker {
    pub(super) fn get_current_indentation_at_position(&self, pos: usize) -> i64 {
        let start_line_position = util::line_start_position_for_position(&self.source_file, pos);
        util::find_first_non_whitespace_column(
            &self.source_file,
            start_line_position,
            pos,
            self.options.editor_settings.tab_size,
        ) as i64
    }

    /// Go processNode
    pub(super) fn process_node(
        &mut self,
        scanner: &mut FormattingScanner,
        node: &Arc<Node>,
        context_node: &Arc<Node>,
        node_start_line: usize,
        undecorated_node_start_line: usize,
        indentation: i64,
        delta: i64,
    ) {
        if !self
            .original_range
            .overlaps(&util::with_token_start(&self.source_file, node))
        {
            return;
        }

        let node_dynamic_indentation = DynamicIndenter::new_ref(
            Arc::clone(node),
            node_start_line,
            indentation,
            delta,
            self.options.clone(),
            Arc::clone(&self.source_file),
        );

        self.child_context_node = Some(Arc::clone(context_node));

        self.execute_process_node_visitor(
            scanner,
            node,
            &node_dynamic_indentation,
            node_start_line,
            undecorated_node_start_line,
        );

        while scanner.is_on_token() && scanner.get_token_full_start() < self.original_range.end() {
            let info = scanner.read_token_info(Some(node));
            let Some(token) = &info.token else { break };
            if token.loc.end() > node.end().min(self.original_range.end()) {
                break;
            }
            self.consume_token_and_advance_scanner(
                scanner,
                info,
                node,
                &node_dynamic_indentation,
                node,
                false,
            );
        }
    }

    /// Go executeProcessNodeVisitor：经 visit_items 按字段序分发；
    /// 列表经 processChildNodes，标量（含 token）经 processChildNode。
    fn execute_process_node_visitor(
        &mut self,
        scanner: &mut FormattingScanner,
        node: &Arc<Node>,
        indenter: &IndenterRef,
        node_start_line: usize,
        undecorated_node_start_line: usize,
    ) {
        enum Item {
            Node(Arc<Node>),
            List(Arc<crate::ast::node::NodeList>),
            Nodes(Vec<Arc<Node>>),
        }
        let mut items: Vec<Item> = Vec::new();
        super::super::visit_generated::visit_items(node, &mut |item| {
            match item {
                super::super::visit_generated::VisitItem::Node(n) => {
                    items.push(Item::Node(Arc::clone(n)))
                }
                super::super::visit_generated::VisitItem::List(l) => {
                    items.push(Item::List(Arc::clone(l)))
                }
                super::super::visit_generated::VisitItem::Modifiers(m) => {
                    items.push(Item::Nodes(m.list.nodes.clone()))
                }
                super::super::visit_generated::VisitItem::Slice(s) => {
                    items.push(Item::Nodes(s.to_vec()))
                }
            }
        });

        for item in items {
            match item {
                Item::Node(child) => {
                    self.process_child_node(
                        scanner,
                        node,
                        indenter,
                        node_start_line,
                        undecorated_node_start_line,
                        &child,
                        -1,
                        node,
                        indenter,
                        node_start_line,
                        undecorated_node_start_line,
                        false,
                        false,
                    );
                }
                Item::List(list) => {
                    self.process_child_nodes(
                        scanner,
                        node,
                        indenter,
                        node_start_line,
                        undecorated_node_start_line,
                        &list,
                        node,
                        node_start_line,
                        indenter,
                    );
                }
                Item::Nodes(nodes) => {
                    for child in nodes {
                        self.process_child_node(
                            scanner,
                            node,
                            indenter,
                            node_start_line,
                            undecorated_node_start_line,
                            &child,
                            -1,
                            node,
                            indenter,
                            node_start_line,
                            undecorated_node_start_line,
                            false,
                            false,
                        );
                    }
                }
            }
        }
    }

    /// Go processChildNodes
    #[allow(clippy::too_many_arguments)]
    fn process_child_nodes(
        &mut self,
        scanner: &mut FormattingScanner,
        node: &Arc<Node>,
        _indenter: &IndenterRef,
        node_start_line: usize,
        undecorated_node_start_line: usize,
        nodes: &Arc<crate::ast::node::NodeList>,
        parent: &Arc<Node>,
        parent_start_line: usize,
        parent_dynamic_indentation: &IndenterRef,
    ) {
        let list_start_token = lists::open_token_for_list(parent, nodes);

        let mut list_dynamic_indentation = Arc::clone(parent_dynamic_indentation);
        let mut start_line = parent_start_line;

        if !self.original_range.overlaps(&nodes.loc) {
            if nodes.loc.end() < self.original_range.pos()
                && nodes
                    .nodes
                    .first()
                    .is_none_or(|first| !first.flags.contains(crate::ast::NodeFlags::Reparsed))
            {
                scanner.skip_to_end_of(nodes.loc.end());
            }
            return;
        }

        if list_start_token != SyntaxKind::Unknown {
            while scanner.is_on_token()
                && scanner.get_token_full_start() < self.original_range.end()
            {
                let token_info = scanner.read_token_info(Some(parent));
                let Some(token) = token_info.token.clone() else { break };
                if token.loc.end() > nodes.loc.pos() {
                    break;
                } else if token.kind == list_start_token {
                    start_line = util::line_of_position(&self.source_file, token.loc.pos());
                    self.consume_token_and_advance_scanner(
                        scanner,
                        token_info,
                        parent,
                        parent_dynamic_indentation,
                        parent,
                        false,
                    );

                    let indentation_on_list_start_token =
                        if self.indentation_on_last_indented_line != -1 {
                            self.indentation_on_last_indented_line
                        } else {
                            self.get_current_indentation_at_position(token.loc.pos())
                        };

                    list_dynamic_indentation = DynamicIndenter::new_ref(
                        Arc::clone(parent),
                        parent_start_line,
                        indentation_on_list_start_token,
                        self.options.editor_settings.indent_size as i64,
                        self.options.clone(),
                        Arc::clone(&self.source_file),
                    );
                } else {
                    self.consume_token_and_advance_scanner(
                        scanner,
                        token_info,
                        parent,
                        parent_dynamic_indentation,
                        parent,
                        false,
                    );
                }
            }
        }

        let mut inherited_indentation: i64 = -1;
        for (i, child) in nodes.nodes.iter().enumerate() {
            inherited_indentation = self.process_child_node(
                scanner,
                node,
                _indenter,
                node_start_line,
                undecorated_node_start_line,
                child,
                inherited_indentation,
                node,
                &list_dynamic_indentation,
                start_line,
                start_line,
                true,
                i == 0,
            );
        }

        let list_end_token = lists::close_token_for_open_token(list_start_token);
        if list_end_token != SyntaxKind::Unknown
            && scanner.is_on_token()
            && scanner.get_token_full_start() < self.original_range.end()
        {
            let mut token_info = scanner.read_token_info(Some(parent));
            if token_info
                .token
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::CommaToken)
            {
                self.consume_token_and_advance_scanner(
                    scanner,
                    token_info,
                    parent,
                    &list_dynamic_indentation,
                    parent,
                    false,
                );
                if scanner.is_on_token() {
                    token_info = scanner.read_token_info(Some(parent));
                } else {
                    return;
                }
            }

            let is_end = token_info
                .token
                .as_ref()
                .is_some_and(|t| t.kind == list_end_token && t.loc.contained_by(&parent.loc));
            if is_end {
                self.consume_token_and_advance_scanner(
                    scanner,
                    token_info,
                    parent,
                    &list_dynamic_indentation,
                    parent,
                    true,
                );
            }
        }
    }

    /// Go processChildNode
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    fn process_child_node(
        &mut self,
        scanner: &mut FormattingScanner,
        node: &Arc<Node>,
        _indenter: &IndenterRef,
        _node_start_line: usize,
        _undecorated_node_start_line: usize,
        child: &Arc<Node>,
        mut inherited_indentation: i64,
        parent: &Arc<Node>,
        parent_dynamic_indentation: &IndenterRef,
        parent_start_line: usize,
        undecorated_parent_start_line: usize,
        is_list_item: bool,
        is_first_list_item: bool,
    ) -> i64 {
        if util::node_is_missing(child) || child.flags.contains(crate::ast::NodeFlags::Reparsed) {
            return inherited_indentation;
        }
        let child_start_pos = util::token_pos_of_node(&self.source_file, child);
        let child_start_line = util::line_of_position(&self.source_file, child_start_pos);

        let mut undecorated_child_start_line = child_start_line;
        if has_decorators(child) {
            undecorated_child_start_line = util::line_of_position(
                &self.source_file,
                super::super::span::get_non_decorator_token_pos_of_node(&self.source_file, child),
            );
        }

        let is_error_member_list_element = (child
            .flags
            .contains(crate::ast::NodeFlags::ThisNodeHasError)
            || self.member_gap_has_parse_error(child_start_pos))
            && lists::is_member_list_element(parent, child, &self.source_file);
        let mut child_indentation_amount: i64 = -1;

        if !is_error_member_list_element
            && is_list_item
            && parent.loc.contained_by(&self.original_range)
        {
            child_indentation_amount = self.try_compute_indentation_for_list_item(
                child_start_pos,
                child.end(),
                parent_start_line,
                inherited_indentation,
            );
            if child_indentation_amount != -1 {
                inherited_indentation = child_indentation_amount;
            }
        }

        // child 在目标 range 之外：不深入
        if !self.original_range.overlaps(&child.loc) {
            if child.end() < self.original_range.pos() {
                scanner.skip_to_end_of(child.end());
            }
            return inherited_indentation;
        }

        if child.loc.len() == 0 {
            return inherited_indentation;
        }

        // 消费位于 child 之前的父节点 token
        while scanner.is_on_token() && scanner.get_token_full_start() < self.original_range.end() {
            let token_info = scanner.read_token_info(Some(node));
            let Some(token) = &token_info.token else { break };
            if token.loc.end() > self.original_range.end() {
                return inherited_indentation;
            }
            if token.loc.end() > child_start_pos {
                if token.loc.pos() > child_start_pos {
                    scanner.skip_to_start_of(child.loc.pos());
                }
                break;
            }
            self.consume_token_and_advance_scanner(
                scanner,
                token_info,
                node,
                parent_dynamic_indentation,
                node,
                false,
            );
        }

        if !scanner.is_on_token() || scanner.get_token_full_start() >= self.original_range.end() {
            return inherited_indentation;
        }

        if crate::ast::node_data_generated::is_token_kind(child.kind) {
            // token 子节点不影响缩进，用父级缩进作用域消费
            let token_info = scanner.read_token_info(Some(child));
            if child.kind != SyntaxKind::JsxText {
                self.consume_token_and_advance_scanner(
                    scanner,
                    token_info,
                    node,
                    parent_dynamic_indentation,
                    child,
                    false,
                );
                return inherited_indentation;
            }
        }

        let effective_parent_start_line = if child.kind == SyntaxKind::Decorator {
            child_start_line
        } else {
            undecorated_parent_start_line
        };
        let (child_indentation, delta) = if is_error_member_list_element {
            (self.get_current_indentation_at_position(child_start_pos), 0)
        } else {
            self.compute_indentation(
                child,
                child_start_line,
                child_indentation_amount,
                parent_dynamic_indentation,
                effective_parent_start_line,
            )
        };

        let context_node = self
            .child_context_node
            .clone()
            .unwrap_or_else(|| Arc::clone(&self.source_file.node));
        self.process_node(
            scanner,
            child,
            &context_node,
            child_start_line,
            undecorated_child_start_line,
            child_indentation,
            delta,
        );

        self.child_context_node = Some(Arc::clone(node));

        if is_first_list_item
            && parent.kind == SyntaxKind::ArrayLiteralExpression
            && inherited_indentation == -1
        {
            inherited_indentation = child_indentation;
        }

        inherited_indentation
    }

    /// Go computeIndentation
    fn compute_indentation(
        &self,
        node: &Arc<Node>,
        start_line: usize,
        inherited_indentation: i64,
        parent_dynamic_indentation: &IndenterRef,
        effective_parent_start_line: usize,
    ) -> (i64, i64) {
        let mut delta: i64 = 0;
        // Go 此处传 nil sourceFile
        if should_indent_child_node(&self.options, node, None, None, false) {
            delta = self.options.editor_settings.indent_size as i64;
        }

        if effective_parent_start_line == start_line {
            let mut indentation = self.indentation_on_last_indented_line;
            if start_line as i64 != self.last_indented_line {
                indentation = parent_dynamic_indentation.borrow().get_indentation();
            }
            let parent_delta = parent_dynamic_indentation.borrow().get_delta(Some(node));
            delta = (self.options.editor_settings.indent_size as i64).min(parent_delta + delta);
            return (indentation, delta);
        } else if inherited_indentation == -1 {
            let parent = node.parent().unwrap_or_else(|| Arc::clone(node));
            if node.kind == SyntaxKind::OpenParenToken
                && start_line as i64 == self.last_indented_line
            {
                let d = parent_dynamic_indentation.borrow().get_delta(Some(node));
                return (self.indentation_on_last_indented_line, d);
            } else if child_starts_on_same_line_with_else_in_if_statement(
                &parent,
                node,
                start_line,
                &self.source_file,
            ) || child_is_unindented_branch_of_conditional_expression(
                &parent,
                node,
                start_line,
                &self.source_file,
            ) || argument_starts_on_same_line_as_previous_argument(
                &parent,
                node,
                start_line,
                &self.source_file,
            ) {
                return (parent_dynamic_indentation.borrow().get_indentation(), delta);
            } else {
                let i = parent_dynamic_indentation.borrow().get_indentation();
                if i == -1 {
                    return (parent_dynamic_indentation.borrow().get_indentation(), delta);
                }
                let d = parent_dynamic_indentation.borrow().get_delta(Some(node));
                return (i + d, delta);
            }
        }

        (inherited_indentation, delta)
    }

    /// Go tryComputeIndentationForListItem
    /// Go 的 ThisNodeHasError 标记（finishNode 时 hasParseError）在本移植的
    /// parser 中未落地；扫描期错误（冲突标记等）落在成员与前一 token 的间隙
    /// 时，Go 同样会把该成员标记为 error member，此处按间隙重叠补齐判定。
    fn member_gap_has_parse_error(&self, child_start_pos: usize) -> bool {
        if self.source_file.parse_error_spans.is_empty() {
            return false;
        }
        let gap_start = crate::astnav::find_preceding_token(&self.source_file.node, child_start_pos)
            .map(|t| t.end())
            .unwrap_or(0);
        self.source_file
            .parse_error_spans
            .iter()
            .any(|e| e.pos() < child_start_pos && e.end() > gap_start)
    }

    fn try_compute_indentation_for_list_item(
        &self,
        start_pos: usize,
        end_pos: usize,
        parent_start_line: usize,
        inherited_indentation: i64,
    ) -> i64 {
        let r2 = TextRange::new(start_pos, end_pos);
        if self.original_range.overlaps(&r2) || r2.contained_by(&self.original_range) {
            if inherited_indentation != -1 {
                return inherited_indentation;
            }
        } else {
            let start_line = util::line_of_position(&self.source_file, start_pos);
            let column = self.get_current_indentation_at_position(start_pos);
            if start_line != parent_start_line || start_pos as i64 == column {
                let base_indent_size = self.options.editor_settings.base_indent_size as i64;
                if base_indent_size > column {
                    return base_indent_size;
                }
                return column;
            }
        }
        -1
    }
}

