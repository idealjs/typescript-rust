//! Go format/context.go + rulecontext.go 的移植：FormattingContext 与
//! 规则上下文谓词。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use tsox_core::core::text::TextRange;
use crate::format::scanner::TextRangeWithKind;

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
        common_parent: Arc<Node>,
    ) {
        self.current_token_span = cur;
        self.current_token_parent = Some(cur_parent);
        self.next_token_span = next;
        self.next_token_parent = Some(next_parent);
        self.context_node = Some(common_parent);
        self.context_node_all_on_same_line = Tristate::Unknown;
        self.next_node_all_on_same_line = Tristate::Unknown;
        self.tokens_are_on_same_line = Tristate::Unknown;
        self.context_node_block_is_on_one_line = Tristate::Unknown;
        self.next_node_block_is_on_one_line = Tristate::Unknown;
    }

    fn range_is_on_one_line(&self, range: TextRange) -> bool {
        let text = &self.source_file.text;
        let start = range.pos().min(text.len());
        let end = range.end().min(text.len());
        !text.as_bytes()[start..end].contains(&b'\n')
    }

    fn node_is_on_one_line(&self, node: &Arc<Node>) -> bool {
        // Go withTokenStart：token 起点跳过修饰符/前缀 trivia，
        // 近似取名字字段起点，退化用节点起点
        let start = node
            .name()
            .map(|n| n.pos())
            .unwrap_or_else(|| node.pos());
        self.range_is_on_one_line(TextRange::new(start, node.end()))
    }

    fn block_is_on_one_line(&self, node: &Arc<Node>) -> bool {
        let open = find_child_of_kind(node, SyntaxKind::OpenBraceToken);
        let close = find_child_of_kind(node, SyntaxKind::CloseBraceToken);
        if let (Some(open), Some(close)) = (open, close) {
            return self.range_is_on_one_line(TextRange::new(open.end(), close.pos()));
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

pub(crate) fn find_child_of_kind(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    let mut hit = None;
    crate::ast::node_data_generated::for_each_child(node, |c| {
        if c.kind == kind {
            hit = Some(c.clone());
            return true;
        }
        false
    });
    hit
}
