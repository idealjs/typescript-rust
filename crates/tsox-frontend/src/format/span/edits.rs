//! Go span.go 的编辑产出：processPair（规则收集与逆序应用）、
//! applyRuleEdits、processRange、processTrivia、record*。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use crate::format::rule::{RuleAction, RuleFlags, RuleImpl};
use crate::format::rulesmap;
use crate::format::scanner::{FormattingScanner, TextRangeWithKind};
use crate::format::span::{FormatSpanWorker, LineAction};
use crate::format::TextChange;

use super::super::util;

impl FormatSpanWorker {
    /// Go processPair
    #[allow(clippy::too_many_arguments)]
    pub(super) fn process_pair(
        &mut self,
        _scanner: &mut FormattingScanner,
        current_item: TextRangeWithKind,
        current_start_line: usize,
        current_parent: Arc<Node>,
        previous_item: TextRangeWithKind,
        previous_start_line: usize,
        previous_parent: Arc<Node>,
        context_node: Option<Arc<Node>>,
        dynamic_indentation: Option<&crate::format::indenter::IndenterRef>,
    ) -> LineAction {
        self.formatting_context.update_context(
            previous_item.clone(),
            previous_parent.clone(),
            current_item.clone(),
            current_parent.clone(),
            context_node.clone(),
        );

        let mut current_rules: Vec<RuleImpl> = Vec::new();
        let bucket = rulesmap::get_rules(previous_item.kind, current_item.kind);
        let mut rule_action_mask = RuleAction::NONE;
        'outer: for rule in bucket {
            let exclusion = get_rule_action_exclusion(rule_action_mask);
            let accept = RuleAction(!exclusion.0);
            if rule.action.0 & accept.0 != 0 {
                for pred in &rule.context {
                    if !pred(&mut self.formatting_context) {
                        continue 'outer;
                    }
                }
                current_rules.push(rule.clone());
                rule_action_mask = RuleAction(rule_action_mask.0 | rule.action.0);
            }
        }

        let mut trim_trailing_whitespaces = self
            .formatting_context
            .options
            .editor_settings
            .trim_trailing_whitespace;
        let mut line_action = LineAction::None;

        if !current_rules.is_empty() {
            // 逆序应用：先入桶的高优先级规则覆盖低优先级
            for rule in current_rules.iter().rev() {
                line_action = self.apply_rule_edits(
                    rule,
                    &previous_item,
                    previous_start_line,
                    &current_item,
                    current_start_line,
                );
                if let Some(indenter) = dynamic_indentation {
                    let current_parent_is_token =
                        util::token_pos_of_node(&self.source_file, &current_parent)
                            == current_item.loc.pos();
                    match line_action {
                        LineAction::LineRemoved => {
                            if current_parent_is_token {
                                indenter.borrow_mut().recompute_indentation(
                                    false,
                                    &context_node
                                        .clone()
                                        .unwrap_or_else(|| previous_parent.clone()),
                                );
                            }
                        }
                        LineAction::LineAdded => {
                            if current_parent_is_token {
                                indenter.borrow_mut().recompute_indentation(
                                    true,
                                    &context_node.clone().unwrap_or_else(|| previous_parent.clone()),
                                );
                            }
                        }
                        LineAction::None => {}
                    }
                }
                trim_trailing_whitespaces = trim_trailing_whitespaces
                    && !rule.action.intersects(RuleAction::DELETE_SPACE)
                    && rule.flags != RuleFlags::CanDeleteNewLines;
            }
        } else {
            trim_trailing_whitespaces =
                trim_trailing_whitespaces && current_item.kind != SyntaxKind::EndOfFile;
        }

        if current_start_line != previous_start_line && trim_trailing_whitespaces {
            self.trim_trailing_whitespaces_for_lines(
                previous_start_line,
                current_start_line,
                Some(&previous_item),
            );
        }

        line_action
    }

    /// Go applyRuleEdits
    pub(super) fn apply_rule_edits(
        &mut self,
        rule: &RuleImpl,
        previous_range: &TextRangeWithKind,
        previous_start_line: usize,
        current_range: &TextRangeWithKind,
        current_start_line: usize,
    ) -> LineAction {
        let on_later_line = current_start_line != previous_start_line;
        let action = rule.action;
        // Go switch 是动作值相等匹配（非位测试），复合动作不命中任何分支
        if action == RuleAction::STOP_PROCESSING_SPACE_ACTIONS {
            return LineAction::None;
        }
        if action == RuleAction::DELETE_SPACE {
            let gap = current_range.loc.pos() as i64 - previous_range.loc.end() as i64;
            if gap > 0 {
                self.record_delete(previous_range.loc.end(), gap as usize);
                if on_later_line {
                    return LineAction::LineRemoved;
                }
                return LineAction::None;
            }
            return LineAction::None;
        }
        if action == RuleAction::DELETE_TOKEN {
            self.record_delete(previous_range.loc.pos(), previous_range.loc.len());
            return LineAction::None;
        }
        if action == RuleAction::INSERT_NEW_LINE {
            if rule.flags != RuleFlags::CanDeleteNewLines
                && previous_start_line != current_start_line
            {
                return LineAction::None;
            }
            let line_delta = current_start_line as i64 - previous_start_line as i64;
            if line_delta != 1 {
                let gap = current_range.loc.pos() as i64 - previous_range.loc.end() as i64;
                if gap < 0 {
                    return LineAction::None;
                }
                let new_line = self.new_line_character.clone();
                self.record_replace(previous_range.loc.end(), gap as usize, &new_line);
                if on_later_line {
                    return LineAction::None;
                }
                return LineAction::LineAdded;
            }
            return LineAction::None;
        }
        if action == RuleAction::INSERT_SPACE {
            if rule.flags != RuleFlags::CanDeleteNewLines
                && previous_start_line != current_start_line
            {
                return LineAction::None;
            }
            // Go int 可为负（乱序 pair）；负 delta 不产出编辑
            let pos_delta = current_range.loc.pos() as i64 - previous_range.loc.end() as i64;
            if pos_delta < 0 {
                return LineAction::None;
            }
            let pos_delta = pos_delta as usize;
            let next_is_space = self
                .source_file
                .text
                .as_bytes()
                .get(previous_range.loc.end())
                .is_some_and(|&b| b == b' ');
            if pos_delta != 1 || !next_is_space {
                self.record_replace(previous_range.loc.end(), pos_delta, " ");
                if on_later_line {
                    return LineAction::LineRemoved;
                }
                return LineAction::None;
            }
            return LineAction::None;
        }
        if action == RuleAction::INSERT_TRAILING_SEMICOLON {
            self.record_insert(previous_range.loc.end(), ";");
        }
        LineAction::None
    }

    /// Go processRange
    pub(super) fn process_range(
        &mut self,
        scanner: &mut FormattingScanner,
        r: TextRangeWithKind,
        range_start_line: usize,
        range_start_character: usize,
        parent: Arc<Node>,
        context_node: Option<Arc<Node>>,
        dynamic_indentation: Option<&crate::format::indenter::IndenterRef>,
    ) -> LineAction {
        let range_has_error = (self.range_contains_error)(r.loc);
        let mut line_action = LineAction::None;
        if !range_has_error {
            if self.previous_range_is_zero() {
                // span 开头到首个 token 之间的行做尾随空白修剪
                let original_start_line =
                    util::line_of_position(&self.source_file, self.original_range.pos());
                self.trim_trailing_whitespaces_for_lines(original_start_line, range_start_line, None);
            } else {
                let previous = self.previous_range.clone().expect("previous range");
                let previous_parent = self.previous_parent.clone().expect("previous parent");
                line_action = self.process_pair(
                    scanner,
                    r.clone(),
                    range_start_line,
                    parent.clone(),
                    previous,
                    self.previous_range_start_line,
                    previous_parent,
                    context_node,
                    dynamic_indentation,
                );
            }
        }

        self.previous_range_trivia_end = r.loc.end();
        self.previous_range = Some(r);
        self.previous_parent = Some(parent.clone());
        self.previous_range_start_line = range_start_line;
        self.previous_range_start_character = range_start_character;
        line_action
    }

    /// Go processTrivia
    pub(super) fn process_trivia(
        &mut self,
        scanner: &mut FormattingScanner,
        trivia: &[TextRangeWithKind],
        parent: Arc<Node>,
        context_node: Option<Arc<Node>>,
        dynamic_indentation: Option<&crate::format::indenter::IndenterRef>,
    ) {
        for trivia_item in trivia {
            if util::is_comment(trivia_item.kind)
                && trivia_item.loc.contained_by(&self.original_range)
            {
                let (start_line, start_char) =
                    util::line_and_byte_offset_of_position(&self.source_file, trivia_item.loc.pos());
                self.process_range(
                    scanner,
                    trivia_item.clone(),
                    start_line,
                    start_char,
                    Arc::clone(&parent),
                    context_node.clone(),
                    dynamic_indentation,
                );
            }
        }
    }

    pub(super) fn record_delete(&mut self, start: usize, length: usize) {
        if length != 0 {
            self.edits.push(TextChange { pos: start, end: start + length, new_text: String::new() });
        }
    }

    pub(super) fn record_replace(&mut self, start: usize, length: usize, new_text: &str) {
        if length != 0 || !new_text.is_empty() {
            self.edits.push(TextChange {
                pos: start,
                end: start + length,
                new_text: new_text.to_string(),
            });
        }
    }

    pub(super) fn record_insert(&mut self, start: usize, text: &str) {
        if !text.is_empty() {
            self.edits.push(TextChange { pos: start, end: start, new_text: text.to_string() });
        }
    }
}

/// Go getRuleActionExclusion
fn get_rule_action_exclusion(action: RuleAction) -> RuleAction {
    let mut mask = RuleAction::NONE;
    if action.intersects(RuleAction::STOP_PROCESSING_SPACE_ACTIONS) {
        mask = RuleAction(mask.0 | RuleAction::MODIFY_SPACE_ACTION.0);
    }
    if action.intersects(RuleAction::STOP_PROCESSING_TOKEN_ACTIONS) {
        mask = RuleAction(mask.0 | RuleAction::MODIFY_TOKEN_ACTION.0);
    }
    if action.intersects(RuleAction::MODIFY_SPACE_ACTION) {
        mask = RuleAction(mask.0 | RuleAction::MODIFY_SPACE_ACTION.0);
    }
    if action.intersects(RuleAction::MODIFY_TOKEN_ACTION) {
        mask = RuleAction(mask.0 | RuleAction::MODIFY_TOKEN_ACTION.0);
    }
    mask
}
