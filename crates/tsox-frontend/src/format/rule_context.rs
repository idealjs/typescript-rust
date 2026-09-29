//! Go format/context.go + rulecontext.go 的移植：FormattingContext 与
//! 规则上下文谓词。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use tsox_core::core::text::TextRange;
use crate::format::scanner::TextRangeWithKind;

use super::util;
use super::FormatCodeSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormatRequestKind {
    FormatDocument,
    FormatSelection,
    FormatOnEnter,
    FormatOnSemicolon,
    FormatOnOpeningCurlyBrace,
    FormatOnClosingCurlyBrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tristate {
    #[default]
    Unknown,
    True,
    False,
}

impl Tristate {
    pub fn is_true(self) -> bool {
        self == Tristate::True
    }
    pub fn is_true_or_unknown(self) -> bool {
        self != Tristate::False
    }
    pub fn is_false(self) -> bool {
        self == Tristate::False
    }
    pub fn is_false_or_unknown(self) -> bool {
        self != Tristate::True
    }
}

pub(crate) struct FormattingContext {
    pub(crate) source_file: Arc<crate::ast::SourceFile>,
    pub(crate) formatting_request_kind: FormatRequestKind,
    pub(crate) options: FormatCodeSettings,

    pub(crate) current_token_span: TextRangeWithKind,
    pub(crate) next_token_span: TextRangeWithKind,
    pub(crate) context_node: Option<Arc<Node>>,
    pub(crate) current_token_parent: Option<Arc<Node>>,
    pub(crate) next_token_parent: Option<Arc<Node>>,

    context_node_all_on_same_line: Tristate,
    next_node_all_on_same_line: Tristate,
    tokens_are_on_same_line: Tristate,
    context_node_block_is_on_one_line: Tristate,
    next_node_block_is_on_one_line: Tristate,
}

impl FormattingContext {
    pub(crate) fn new(
        file: Arc<crate::ast::SourceFile>,
        kind: FormatRequestKind,
        options: FormatCodeSettings,
    ) -> Self {
        Self {
            source_file: file,
            formatting_request_kind: kind,
            options,
            current_token_span: TextRangeWithKind::new(0, 0, SyntaxKind::EndOfFile),
            next_token_span: TextRangeWithKind::new(0, 0, SyntaxKind::EndOfFile),
            context_node: None,
            current_token_parent: None,
            next_token_parent: None,
            context_node_all_on_same_line: Tristate::Unknown,
            next_node_all_on_same_line: Tristate::Unknown,
            tokens_are_on_same_line: Tristate::Unknown,
            context_node_block_is_on_one_line: Tristate::Unknown,
            next_node_block_is_on_one_line: Tristate::Unknown,
        }
    }

    pub(crate) fn update_context(
        &mut self,
        cur: TextRangeWithKind,
        cur_parent: Arc<Node>,
        next: TextRangeWithKind,
        next_parent: Arc<Node>,
        context_node: Option<Arc<Node>>,
    ) {
        self.current_token_span = cur;
        self.current_token_parent = Some(cur_parent);
        self.next_token_span = next;
        self.next_token_parent = Some(next_parent);
        self.context_node = context_node;
        self.context_node_all_on_same_line = Tristate::Unknown;
        self.next_node_all_on_same_line = Tristate::Unknown;
        self.tokens_are_on_same_line = Tristate::Unknown;
        self.context_node_block_is_on_one_line = Tristate::Unknown;
        self.next_node_block_is_on_one_line = Tristate::Unknown;
    }

    fn range_is_on_one_line(&self, range: TextRange) -> bool {
        let start = range.pos().min(self.source_file.text.len());
        let end = range.end().min(self.source_file.text.len());
        util::line_of_position(&self.source_file, start) == util::line_of_position(&self.source_file, end)
    }

    fn node_is_on_one_line(&self, node: &Arc<Node>) -> bool {
        self.range_is_on_one_line(util::with_token_start(&self.source_file, node))
    }

    fn block_is_on_one_line(&self, node: &Arc<Node>) -> bool {
        let open = find_child_of_kind(node, SyntaxKind::OpenBraceToken, &self.source_file);
        let close = find_child_of_kind(node, SyntaxKind::CloseBraceToken, &self.source_file);
        if let (Some(open), Some(close)) = (open, close) {
            let close_start = util::token_pos_of_node(&self.source_file, &close);
            return self.range_is_on_one_line(TextRange::new(open.end(), close_start));
        }
        false
    }

    pub(crate) fn context_node_all_on_same_line(&mut self) -> bool {
        if self.context_node_all_on_same_line == Tristate::Unknown {
            let r = self
                .context_node
                .as_ref()
                .is_some_and(|n| self.node_is_on_one_line(n));
            self.context_node_all_on_same_line = bool_to_tristate(r);
        }
        tristate_to_bool(self.context_node_all_on_same_line)
    }

    pub(crate) fn next_node_all_on_same_line(&mut self) -> bool {
        if self.next_node_all_on_same_line == Tristate::Unknown {
            let r = self
                .next_token_parent
                .as_ref()
                .is_some_and(|n| self.node_is_on_one_line(n));
            self.next_node_all_on_same_line = bool_to_tristate(r);
        }
        tristate_to_bool(self.next_node_all_on_same_line)
    }

    pub(crate) fn tokens_are_on_same_line(&mut self) -> bool {
        if self.tokens_are_on_same_line == Tristate::Unknown {
            let start = self.current_token_span.loc.pos();
            let end = self.next_token_span.loc.end();
            self.tokens_are_on_same_line = bool_to_tristate(self.range_is_on_one_line(TextRange::new(start, end)));
        }
        tristate_to_bool(self.tokens_are_on_same_line)
    }

    pub(crate) fn context_node_block_is_on_one_line(&mut self) -> bool {
        if self.context_node_block_is_on_one_line == Tristate::Unknown {
            let r = self
                .context_node
                .as_ref()
                .is_some_and(|n| self.block_is_on_one_line(n));
            self.context_node_block_is_on_one_line = bool_to_tristate(r);
        }
        tristate_to_bool(self.context_node_block_is_on_one_line)
    }

    pub(crate) fn next_node_block_is_on_one_line(&mut self) -> bool {
        if self.next_node_block_is_on_one_line == Tristate::Unknown {
            let r = self
                .next_token_parent
                .as_ref()
                .is_some_and(|n| self.block_is_on_one_line(n));
            self.next_node_block_is_on_one_line = bool_to_tristate(r);
        }
        tristate_to_bool(self.next_node_block_is_on_one_line)
    }
}

fn bool_to_tristate(b: bool) -> Tristate {
    if b { Tristate::True } else { Tristate::False }
}

fn tristate_to_bool(t: Tristate) -> bool {
    t == Tristate::True
}

/// Go astnav.FindChildOfKind：除直接子节点外，用 scanner 扫子节点之间
/// 与尾部的 token 空档（花括号等 token 不在 AST 上时由扫描合成）。
pub(crate) fn find_child_of_kind(
    node: &Arc<Node>,
    kind: SyntaxKind,
    file: &Arc<crate::ast::SourceFile>,
) -> Option<Arc<Node>> {
    let mut scanner = crate::scanner::Scanner::new(file.text.clone());
    scanner.set_language_variant(file.language_variant);
    let mut last_node_pos = node.pos();
    scanner.set_range(last_node_pos, node.end());
    scanner.scan();

    let synthesize = |scanner: &crate::scanner::Scanner| {
        Some(Arc::new(Node::with_loc(
            kind,
            crate::ast::NodeData::Token,
            TextRange::new(scanner.full_start_pos(), scanner.token_end()),
        )))
    };

    let mut found: Option<Arc<Node>> = None;
    crate::ast::node_data_generated::for_each_child(node, |child| {
        if found.is_some() || child.flags.contains(crate::ast::NodeFlags::Reparsed) {
            return true;
        }
        let mut start_pos = last_node_pos;
        while start_pos < child.pos() {
            if scanner.token() == kind {
                found = synthesize(&scanner);
                return true;
            }
            start_pos = scanner.token_end();
            scanner.scan();
        }
        if child.kind == kind {
            found = Some(Arc::clone(child));
            return true;
        }
        last_node_pos = child.end();
        scanner.set_range(last_node_pos, node.end());
        scanner.scan();
        false
    });
    if found.is_some() {
        return found;
    }

    let mut start_pos = last_node_pos;
    while start_pos < node.end() {
        if scanner.token() == kind {
            return synthesize(&scanner);
        }
        start_pos = scanner.token_end();
        scanner.scan();
    }
    None
}
