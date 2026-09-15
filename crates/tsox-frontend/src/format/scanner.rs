//! Go format/scanner.go 的移植：格式化扫描器。
//! 逐 token 产出 leading trivia / token / trailing trivia 三段，
//! 支持按 AST 容器节点触发的重扫动作（大于号/正则/模板/JSX）。

use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::SyntaxKind;
use crate::scanner::is_jsx_line_break::is_keyword;
use tsox_core::core::text::TextRange;
use crate::scanner::Scanner;

#[derive(Debug, Clone)]
pub(crate) struct TextRangeWithKind {
    pub(crate) loc: TextRange,
    pub(crate) kind: SyntaxKind,
}

impl TextRangeWithKind {
    pub(crate) fn new(pos: usize, end: usize, kind: SyntaxKind) -> Self {
        Self { loc: TextRange::new(pos, end), kind }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TokenInfo {
    pub(crate) leading_trivia: Vec<TextRangeWithKind>,
    pub(crate) token: Option<TextRangeWithKind>,
    pub(crate) trailing_trivia: Vec<TextRangeWithKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScanAction {
    Scan,
    RescanGreaterThanToken,
    RescanSlashToken,
    RescanTemplateToken,
    RescanJsxIdentifier,
    RescanJsxText,
    RescanJsxAttributeValue,
}

pub(crate) struct FormattingScanner {
    s: Scanner,
    start_pos: usize,
    end_pos: usize,
    saved_pos: usize,
    has_last_token_info: bool,
    last_token_info: TokenInfo,
    last_scan_action: ScanAction,
    pub(crate) leading_trivia: Vec<TextRangeWithKind>,
    pub(crate) trailing_trivia: Vec<TextRangeWithKind>,
    pub(crate) was_new_line: bool,
}

pub(crate) fn new_formatting_scanner(
    text: &str,
    language_variant: crate::ast::LanguageVariant,
    start_pos: usize,
    end_pos: usize,
    worker: &mut dyn FormatSpanWorkerLike,
) -> Vec<crate::format::TextChange> {
    let mut scan = Scanner::new(text.to_string());
    // Go newFormattingScanner：SetSkipTrivia(false)，trivia 作为 token 产出
    scan.set_skip_trivia(false);
    scan.set_language_variant(language_variant);
    scan.set_range(start_pos, end_pos);

    let mut fmt_scn = FormattingScanner {
        s: scan,
        start_pos,
        end_pos,
        saved_pos: start_pos,
        has_last_token_info: false,
        last_token_info: TokenInfo::default(),
        last_scan_action: ScanAction::Scan,
        leading_trivia: Vec::new(),
        trailing_trivia: Vec::new(),
        was_new_line: true,
    };

    let res = worker.execute(&mut fmt_scn);
    res
}

/// worker 需要的扫描器接口（避免把具体 worker 类型耦合进扫描器）
pub(crate) trait FormatSpanWorkerLike {
    fn execute(&mut self, s: &mut FormattingScanner) -> Vec<crate::format::TextChange>;
}

impl FormattingScanner {
    pub(crate) fn advance(&mut self) {
        self.has_last_token_info = false;
        let is_started = self.s.full_start_pos() != self.start_pos;

        if is_started {
            self.was_new_line = self
                .trailing_trivia
                .last()
                .is_some_and(|t| t.kind == SyntaxKind::NewLineTrivia);
        } else {
            self.s.scan();
        }

        self.leading_trivia = Vec::new();
        self.trailing_trivia = Vec::new();

        let mut pos = self.s.full_start_pos();

        // 读 leading trivia 与 token
        while pos < self.end_pos {
            let t = self.s.token();
            if !is_trivia(t) {
                break;
            }
            // 消费 leading trivia
            self.s.scan();
            let item = TextRangeWithKind::new(pos, self.s.full_start_pos(), t);
            pos = self.s.full_start_pos();
            self.leading_trivia.push(item);
        }

        self.saved_pos = self.s.token_pos();
    }

    pub(crate) fn read_token_info(&mut self, n: Option<&Arc<Node>>) -> TokenInfo {
        let last_kind = self
            .last_token_info
            .token
            .as_ref()
            .map(|t| t.kind);
        let expected_scan_action = n
            .map(|n| expected_scan_action_for(n, last_kind))
            .unwrap_or(ScanAction::Scan);
        if std::env::var_os("TSOX_DEBUG_FMT").is_some() {
            eprintln!(
                "[rti] n={:?} act={:?} last_act={:?}",
                n.map(|n| n.kind),
                expected_scan_action,
                self.last_scan_action
            );
        }

        if self.has_last_token_info && expected_scan_action == self.last_scan_action {
            let mut info = self.last_token_info.clone();
            if let Some(n) = n {
                fix_token_kind(&mut info, n);
            }
            self.last_token_info = info.clone();
            return info;
        }

        // Go：fullStart != savedPos 时重扫；skip 留下的零宽 Unknown 态
        // 原样返回，由 trailing trivia 收集与 advance 推进扫描器
        if self.s.full_start_pos() != self.saved_pos {
            let end = self.s.end();
            self.s.set_range(self.saved_pos, end);
            self.s.scan();
        }

        let current_token = self.get_next_token(n, expected_scan_action);

        let token = TextRangeWithKind::new(self.s.full_start_pos(), self.s.token_end(), current_token);
        if std::env::var_os("TSOX_DEBUG_FMT").is_some() {
            eprintln!(
                "[fmt-tok] {:?} [{}..{}]",
                token.kind, token.loc.pos(), token.loc.end()
            );
        }

        // 消费 trailing trivia
        self.trailing_trivia = Vec::new();
        while self.s.full_start_pos() < self.end_pos {
            let current = self.s.scan();
            if !is_trivia(current) {
                break;
            }
            let trivia =
                TextRangeWithKind::new(self.s.full_start_pos(), self.s.token_end(), current);
            self.trailing_trivia.push(trivia);

            if current == SyntaxKind::NewLineTrivia {
                // 越过换行
                self.s.scan();
                break;
            }
        }

        self.has_last_token_info = true;
        let mut info = TokenInfo {
            leading_trivia: self.leading_trivia.clone(),
            token: Some(token),
            trailing_trivia: self.trailing_trivia.clone(),
        };
        if let Some(n) = n {
            fix_token_kind(&mut info, n);
        }
        self.last_token_info = info.clone();
        info
    }

    fn get_next_token(
        &mut self,
        n: Option<&Arc<Node>>,
        expected_scan_action: ScanAction,
    ) -> SyntaxKind {
        let token = self.s.token();
        self.last_scan_action = ScanAction::Scan;
        let kind_of = |n: &Arc<Node>| n.kind;
        match expected_scan_action {
            ScanAction::RescanGreaterThanToken => {
                if token == SyntaxKind::GreaterThanToken {
                    self.last_scan_action = ScanAction::RescanGreaterThanToken;
                    return self.s.re_scan_greater_than();
                }
            }
            ScanAction::RescanSlashToken => {
                if token == SyntaxKind::SlashToken || token == SyntaxKind::SlashEqualsToken {
                    self.last_scan_action = ScanAction::RescanSlashToken;
                    return self.s.re_scan_slash_token();
                }
            }
            ScanAction::RescanTemplateToken => {
                if token == SyntaxKind::CloseBraceToken {
                    self.last_scan_action = ScanAction::RescanTemplateToken;
                    return self.s.re_scan_template_token();
                }
                // Go 靠 ResetTokenState(savedPos) 把被 trailing trivia 推过的
                // 扫描器拉回 `}` 再重扫；这里回到上一消费 token 的起点补重扫
                if self
                    .last_token_info
                    .token
                    .as_ref()
                    .is_some_and(|t| t.kind == SyntaxKind::CloseBraceToken)
                {
                    let start = self.last_token_info.token.as_ref().unwrap().loc.pos();
                    self.s.set_range(start, self.s.end());
                    self.s.scan();
                    if self.s.token() == SyntaxKind::CloseBraceToken {
                        self.last_scan_action = ScanAction::RescanTemplateToken;
                        return self.s.re_scan_template_token();
                    }
                }
            }
            ScanAction::RescanJsxIdentifier => {
                self.last_scan_action = ScanAction::RescanJsxIdentifier;
                return self.s.scan_jsx_identifier();
            }
            ScanAction::RescanJsxText => {
                self.last_scan_action = ScanAction::RescanJsxText;
                return self.s.re_scan_jsx_token(false);
            }
            ScanAction::RescanJsxAttributeValue => {
                self.last_scan_action = ScanAction::RescanJsxAttributeValue;
                return self.s.re_scan_jsx_attribute_value();
            }
            ScanAction::Scan => {}
        }
        if let Some(n) = n {
            let _ = kind_of(n);
        }
        token
    }

    pub(crate) fn read_eof_token_range(&self) -> TextRangeWithKind {
        TextRangeWithKind::new(self.s.full_start_pos(), self.s.token_end(), SyntaxKind::EndOfFile)
    }

    pub(crate) fn is_on_token(&self) -> bool {
        let mut current = self.s.token();
        if self.has_last_token_info {
            if let Some(tok) = &self.last_token_info.token {
                current = tok.kind;
            }
        }
        current != SyntaxKind::EndOfFile && !is_trivia(current)
    }

    pub(crate) fn is_on_eof(&self) -> bool {
        let mut current = self.s.token();
        if self.has_last_token_info {
            if let Some(tok) = &self.last_token_info.token {
                current = tok.kind;
            }
        }
        current == SyntaxKind::EndOfFile
    }

    pub(crate) fn skip_to_end_of(&mut self, pos: usize) {
        let end = self.s.end();
        self.s.set_range(pos, end);
        self.saved_pos = self.s.full_start_pos();
        self.last_scan_action = ScanAction::Scan;
        self.has_last_token_info = false;
        self.was_new_line = false;
        self.leading_trivia = Vec::new();
        self.trailing_trivia = Vec::new();
    }

    pub(crate) fn skip_to_start_of(&mut self, pos: usize) {
        let end = self.s.end();
        self.s.set_range(pos, end);
        self.saved_pos = self.s.full_start_pos();
        self.last_scan_action = ScanAction::Scan;
        self.has_last_token_info = false;
        self.was_new_line = false;
        self.leading_trivia = Vec::new();
        self.trailing_trivia = Vec::new();
    }

    pub(crate) fn get_current_leading_trivia(&self) -> &[TextRangeWithKind] {
        &self.leading_trivia
    }

    pub(crate) fn last_trailing_trivia_was_new_line(&self) -> bool {
        self.was_new_line
    }

    pub(crate) fn get_token_full_start(&self) -> usize {
        if self.has_last_token_info {
            if let Some(tok) = &self.last_token_info.token {
                return tok.loc.pos();
            }
        }
        self.s.full_start_pos()
    }

    #[allow(dead_code)]
    pub(crate) fn get_token_end(&self) -> usize {
        if self.has_last_token_info {
            if let Some(tok) = &self.last_token_info.token {
                return tok.loc.end();
            }
        }
        self.s.token_end()
    }
}

fn is_trivia(kind: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_trivia_kind(kind)
}

fn expected_scan_action_for(n: &Arc<Node>, last_token_kind: Option<SyntaxKind>) -> ScanAction {
    if should_rescan_greater_than_token(n) {
        ScanAction::RescanGreaterThanToken
    } else if should_rescan_slash_token(n) {
        ScanAction::RescanSlashToken
    } else if should_rescan_template_token(n) {
        ScanAction::RescanTemplateToken
    } else if should_rescan_jsx_identifier(n) {
        ScanAction::RescanJsxIdentifier
    } else if should_rescan_jsx_text(n, last_token_kind) {
        ScanAction::RescanJsxText
    } else if should_rescan_jsx_attribute_value(n) {
        ScanAction::RescanJsxAttributeValue
    } else {
        ScanAction::Scan
    }
}

fn should_rescan_jsx_text(n: &Arc<Node>, last_token_kind: Option<SyntaxKind>) -> bool {
    if n.kind == SyntaxKind::JsxText {
        return true;
    }
    if n.kind != SyntaxKind::JsxElement {
        return false;
    }
    last_token_kind == Some(SyntaxKind::JsxText)
}

fn should_rescan_jsx_attribute_value(n: &Arc<Node>) -> bool {
    let Some(parent) = n.parent() else {
        return false;
    };
    if parent.kind != SyntaxKind::JsxAttribute {
        return false;
    }
    let crate::ast::node_data_generated::NodeData::JsxAttribute(d) = &parent.data else {
        return false;
    };
    d.initializer.as_ref().is_some_and(|i| Arc::ptr_eq(i, n))
}

fn should_rescan_greater_than_token(n: &Arc<Node>) -> bool {
    use SyntaxKind::*;
    matches!(
        n.kind,
        GreaterThanEqualsToken
            | GreaterThanGreaterThanEqualsToken
            | GreaterThanGreaterThanGreaterThanEqualsToken
            | GreaterThanGreaterThanGreaterThanToken
            | GreaterThanGreaterThanToken
    )
}

fn should_rescan_slash_token(n: &Arc<Node>) -> bool {
    n.kind == SyntaxKind::RegularExpressionLiteral
}

fn should_rescan_template_token(n: &Arc<Node>) -> bool {
    matches!(n.kind, SyntaxKind::TemplateMiddle | SyntaxKind::TemplateTail)
}

fn should_rescan_jsx_identifier(n: &Arc<Node>) -> bool {
    use SyntaxKind::*;
    let Some(parent) = n.parent() else {
        return false;
    };
    match parent.kind {
        JsxAttribute | JsxOpeningElement | JsxClosingElement | JsxSelfClosingElement
        | JsxNamespacedName => {
            is_keyword_kind(n.kind) || n.kind == Identifier
        }
        PropertyAccessExpression => {
            (is_keyword_kind(n.kind) || n.kind == Identifier) && is_leftmost_jsx_tag_name(n)
        }
        _ => false,
    }
}

fn is_leftmost_jsx_tag_name(node: &Arc<Node>) -> bool {
    let mut n = node.clone();
    loop {
        let Some(parent) = n.parent() else {
            return false;
        };
        if is_jsx_tag_name(&n) {
            return true;
        }
        if parent.kind == SyntaxKind::PropertyAccessExpression
            && parent
                .expression()
                .is_some_and(|e| Arc::ptr_eq(&e, &n))
        {
            n = parent;
            continue;
        }
        return false;
    }
}

fn is_jsx_tag_name(n: &Arc<Node>) -> bool {
    let Some(parent) = n.parent() else {
        return false;
    };
    matches!(
        parent.kind,
        SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxClosingElement
            | SyntaxKind::JsxSelfClosingElement
    )
}

fn is_keyword_kind(kind: SyntaxKind) -> bool {
    is_keyword(kind)
}

fn fix_token_kind(info: &mut TokenInfo, container: &Arc<Node>) {
    if let Some(tok) = &mut info.token {
        if crate::ast::node_data_generated::is_token_kind(container.kind) && tok.kind != container.kind {
            tok.kind = container.kind;
        }
    }
}
