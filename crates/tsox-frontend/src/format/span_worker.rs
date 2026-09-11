//! Go format/span.go 的移植（第一阶段）：token 对处理 + 规则应用。
//!
//! 简化说明：processNode 递归 + dynamicIndenter 尚未移植；此阶段以线性
//! token 流推进，contextNode 取两 token 的最深公共祖先节点（与递归版
//! 语义等价）。间距类规则（Space/DeleteSpace）在此路径下完整生效；
//! 缩进类（InsertNewLine + 缩进重算）待 span_worker_2.rs 补全。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use tsox_core::core::text::TextRange;
use crate::format::rule::{RuleAction, RuleFlags};
use crate::format::rule_context::{FormattingContext, FormatRequestKind};
use crate::format::scanner::{new_formatting_scanner, TextRangeWithKind};
use crate::format::{FormatCodeSettings, TextChange};

pub(crate) fn format_document(
    source_file: &Arc<crate::ast::SourceFile>,
    options: FormatCodeSettings,
) -> Vec<TextChange> {
    format_span(
        source_file,
        TextRange::new(0, source_file.text.len()),
        options,
        FormatRequestKind::FormatDocument,
    )
}

pub(crate) fn format_span(
    source_file: &Arc<crate::ast::SourceFile>,
    span: TextRange,
    options: FormatCodeSettings,
    kind: FormatRequestKind,
) -> Vec<TextChange> {
    let mut worker = FormatSpanWorker {
        source_file: Arc::clone(source_file),
        original_range: span,
        options,
        kind,
        edits: Vec::new(),
    };
    new_formatting_scanner(
        &source_file.text,
        source_file.language_variant,
        span.pos(),
        span.end(),
        &mut worker,
    )
}

pub(crate) struct FormatSpanWorker {
    source_file: Arc<crate::ast::SourceFile>,
    original_range: TextRange,
    options: FormatCodeSettings,
    kind: FormatRequestKind,
    edits: Vec<TextChange>,
}

enum LineAction {
    None,
    LineAdded,
    LineRemoved,
}

impl crate::format::scanner::FormatSpanWorkerLike for FormatSpanWorker {
    fn execute(&mut self, scanner: &mut crate::format::scanner::FormattingScanner) -> Vec<TextChange> {
        let mut context = FormattingContext::new(
            Arc::clone(&self.source_file),
            self.kind,
            self.options.clone(),
        );

        scanner.advance();

        let mut previous: Option<(TextRangeWithKind, Arc<Node>)> = None;
        let mut previous_line = 0usize;

        while scanner.is_on_token() {
            let node_for_token = self.node_at(scanner.get_token_full_start());
            let info = scanner.read_token_info(node_for_token.as_ref());
            let Some(token) = info.token.clone() else { break };
            if token.loc.end() > self.original_range.end() {
                break;
            }
            let line = self.line_of(token.loc.pos());

            if let Some((prev_item, prev_parent)) = &previous {
                let prev_line = previous_line;
                let default_node = Arc::clone(&self.source_file.node);
                let common = self.common_ancestor(
                    prev_parent,
                    node_for_token.as_ref().unwrap_or(&default_node),
                );
                let line_action = self.process_pair(
                    &mut context,
                    token.clone(),
                    line,
                    node_for_token.clone(),
                    prev_item.clone(),
                    prev_line,
                    prev_parent.clone(),
                    common,
                );
                let _ = line_action;
            }

            previous = Some((token, node_for_token.unwrap_or_else(|| Arc::clone(&self.source_file.node))));
            previous_line = line;

            scanner.advance();
        }

        if std::env::var("FMT_DEBUG").is_ok() {
            for e in &self.edits {
                eprintln!("FMTEDIT ({}..{}) => {:?}", e.pos, e.end, e.new_text);
            }
        }
        std::mem::take(&mut self.edits)
    }
}

impl FormatSpanWorker {
    fn line_of(&self, pos: usize) -> usize {
        self.source_file.text.as_bytes()[..pos.min(self.source_file.text.len())]
            .iter()
            .filter(|&&b| b == b'\n')
            .count()
    }

    fn node_at(&self, pos: usize) -> Option<Arc<Node>> {
        // 最深包含 pos 的节点（token 起点）
        fn descend(node: &Arc<Node>, pos: usize) -> Option<Arc<Node>> {
            let mut result: Option<Arc<Node>> = None;
            if node.pos() <= pos && pos < node.end() {
                result = Some(node.clone());
            }
            let mut deeper = None;
            crate::ast::node_data_generated::for_each_child(node, |c| {
                if c.pos() <= pos && pos < c.end() {
                    deeper = descend(c, pos);
                    return true;
                }
                false
            });
            deeper.or(result)
        }
        descend(&self.source_file.node, pos)
    }

    fn common_ancestor(&self, a: &Arc<Node>, b: &Arc<Node>) -> Arc<Node> {
        if Arc::ptr_eq(a, b) {
            return a.clone();
        }
        // a 的祖先链收集
        let mut chain: Vec<Arc<Node>> = Vec::new();
        let mut cur = Some(a.clone());
        while let Some(n) = cur {
            chain.push(n.clone());
            cur = n.parent.clone();
        }
        let mut b_cur = Some(b.clone());
        while let Some(n) = b_cur {
            if let Some(pos) = chain.iter().position(|x| Arc::ptr_eq(x, &n)) {
                return chain[pos].clone();
            }
            b_cur = n.parent.clone();
        }
        Arc::clone(&self.source_file.node)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_pair(
        &mut self,
        context: &mut FormattingContext,
        current: TextRangeWithKind,
        current_line: usize,
        current_parent: Option<Arc<Node>>,
        previous: TextRangeWithKind,
        previous_line: usize,
        previous_parent: Arc<Node>,
        common: Arc<Node>,
    ) -> LineAction {
        context.update_context(
            previous.clone(),
            previous_parent.clone(),
            current.clone(),
            current_parent.clone().unwrap_or_else(|| Arc::clone(&previous_parent)),
            common,
        );

        // Go getRules + getRuleActionExclusion：按优先序收集规则，
        // 已见动作位的同类后续规则被排除（防止多重编辑叠加）
        let mut accepted: Vec<crate::format::rule::RuleImpl> = Vec::new();
        let mut action_mask = RuleAction::NONE;
        for spec in &crate::format::rules::get_all_rules() {
            if !spec.left_token_range.contains(previous.kind)
                || !spec.right_token_range.contains(current.kind)
            {
                continue;
            }
            let exclusion = get_rule_action_exclusion(action_mask);
            if (spec.rule.action.0 & !exclusion.0) != spec.rule.action.0 {
                continue;
            }
            let mut ok = true;
            for pred in &spec.rule.context {
                if !pred(context) {
                    ok = false;
                    break;
                }
            }
            if ok {
                action_mask = RuleAction(action_mask.0 | spec.rule.action.0);
                accepted.push(spec.rule.clone());
            }
        }

        if std::env::var("FMT_DEBUG").is_ok() {
            eprintln!(
                "FMTPAIR prev=({}..{} {:?}) cur=({}..{} {:?})",
                previous.loc.pos(), previous.loc.end(), previous.kind,
                current.loc.pos(), current.loc.end(), current.kind,
            );
        }
        let mut trim_trailing = self.options.editor_settings.trim_trailing_whitespace;
        let mut line_action = LineAction::None;

        if !accepted.is_empty() {
            // 逆序应用：低优先级先执行，高优先级覆盖
            for rule in accepted.iter().rev() {
                line_action = self.apply_rule_edits(
                    rule,
                    &previous,
                    previous_line,
                    &current,
                    current_line,
                );
                trim_trailing = trim_trailing
                    && !rule.action.intersects(RuleAction::DELETE_SPACE)
                    && rule.flags != RuleFlags::CanDeleteNewLines;
            }
        } else {
            trim_trailing = trim_trailing && current.kind != SyntaxKind::EndOfFile;
        }

        if current_line != previous_line && trim_trailing {
            self.trim_trailing_whitespaces_for_line(previous_line, previous.loc.end());
        }

        line_action
    }

    fn apply_rule_edits(
        &mut self,
        rule: &crate::format::rule::RuleImpl,
        previous: &TextRangeWithKind,
        previous_line: usize,
        current: &TextRangeWithKind,
        current_line: usize,
    ) -> LineAction {
        let on_later_line = current_line != previous_line;
        match rule.action {
            a if a.contains(RuleAction::STOP_PROCESSING_SPACE_ACTIONS) => LineAction::None,
            a if a.contains(RuleAction::DELETE_SPACE) => {
                if previous.loc.end() != current.loc.pos() {
                    let count = current.loc.pos() - previous.loc.end();
                    self.edits.push(TextChange {
                        pos: previous.loc.end(),
                        end: current.loc.pos(),
                        new_text: String::new(),
                    });
                    let _ = count;
                    if on_later_line {
                        return LineAction::LineRemoved;
                    }
                }
                LineAction::None
            }
            a if a.contains(RuleAction::DELETE_TOKEN) => {
                self.edits.push(TextChange {
                    pos: previous.loc.pos(),
                    end: previous.loc.end(),
                    new_text: String::new(),
                });
                LineAction::None
            }
            a if a.contains(RuleAction::INSERT_NEW_LINE) => {
                if rule.flags != RuleFlags::CanDeleteNewLines && previous_line != current_line {
                    return LineAction::None;
                }
                let line_delta = current_line as i64 - previous_line as i64;
                if line_delta != 1 {
                    self.edits.push(TextChange {
                        pos: previous.loc.end(),
                        end: current.loc.pos(),
                        new_text: "\n".to_string(),
                    });
                    if on_later_line {
                        return LineAction::None;
                    }
                    return LineAction::LineAdded;
                }
                LineAction::None
            }
            a if a.contains(RuleAction::INSERT_SPACE) => {
                if rule.flags != RuleFlags::CanDeleteNewLines && previous_line != current_line {
                    return LineAction::None;
                }
                let pos_delta = current.loc.pos() - previous.loc.end();
                let has_space = self
                    .source_file
                    .text
                    .as_bytes()
                    .get(previous.loc.end())
                    == Some(&b' ');
                if pos_delta != 1 || !has_space {
                    self.edits.push(TextChange {
                        pos: previous.loc.end(),
                        end: current.loc.pos(),
                        new_text: " ".to_string(),
                    });
                    if on_later_line {
                        return LineAction::LineRemoved;
                    }
                }
                LineAction::None
            }
            a if a.contains(RuleAction::INSERT_TRAILING_SEMICOLON) => {
                self.edits.push(TextChange {
                    pos: previous.loc.end(),
                    end: previous.loc.end(),
                    new_text: ";".to_string(),
                });
                LineAction::None
            }
            _ => LineAction::None,
        }
    }

    fn trim_trailing_whitespaces_for_line(&mut self, line: usize, limit: usize) {
        let text = &self.source_file.text;
        let mut line_start = 0usize;
        let mut cur_line = 0usize;
        for (i, b) in text.as_bytes().iter().enumerate() {
            if cur_line == line {
                break;
            }
            if *b == b'\n' {
                cur_line += 1;
                line_start = i + 1;
            }
        }
        let line_end = text.as_bytes()[line_start..limit.min(text.len())]
            .iter()
            .rposition(|&b| b != b' ' && b != b'\t')
            .map(|p| line_start + p + 1)
            .unwrap_or(line_start);
        if line_end < limit {
            let has_ws = text.as_bytes()[line_start..limit.min(text.len())]
                .iter()
                .any(|&b| b == b' ' || b == b'\t');
            if has_ws && line_end < limit {
                self.edits.push(TextChange {
                    pos: line_end,
                    end: limit,
                    new_text: String::new(),
                });
            }
        }
    }
}

/// Go getRuleActionExclusion：已见动作位生成后续规则的排除掩码
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
