#![allow(unused_imports)]

use crate::scanner::impl_chunk::*;

impl Scanner {
    /// Go scan() 在 skipTrivia=false 时产出 trivia token 的分支
    fn try_scan_trivia_token(&mut self) -> Option<SyntaxKind> {
        use SyntaxKind::*;
        if self.pos >= self.end {
            return None;
        }
        let b = self.text.as_bytes()[self.pos];
        self.token_pos = self.pos;
        match b {
            b' ' | b'\t' | 0x0B | 0x0C => {
                self.pos += 1;
                while self.pos < self.end {
                    let ch = self.text.as_bytes()[self.pos];
                    if !matches!(ch, b' ' | b'\t' | 0x0B | 0x0C) {
                        break;
                    }
                    self.pos += 1;
                }
                self.token = WhitespaceTrivia;
                self.token_end = self.pos;
                Some(WhitespaceTrivia)
            }
            b'\n' | b'\r' => {
                self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
                if b == b'\r'
                    && self.pos + 1 < self.end
                    && self.text.as_bytes()[self.pos + 1] == b'\n'
                {
                    self.pos += 2;
                } else {
                    self.pos += 1;
                }
                self.token = NewLineTrivia;
                self.token_end = self.pos;
                Some(NewLineTrivia)
            }
            b'/' if self.text.as_bytes().get(self.pos + 1) == Some(&b'*') => {
                self.pos += 2;
                let mut closed = false;
                while self.pos < self.end {
                    if self.text.as_bytes()[self.pos] == b'*'
                        && self.text.as_bytes().get(self.pos + 1) == Some(&b'/')
                    {
                        self.pos += 2;
                        closed = true;
                        break;
                    }
                    let ch = self.text.as_bytes()[self.pos] as char;
                    if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                        self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
                    }
                    self.pos += 1;
                }
                let _ = closed;
                self.token = MultiLineCommentTrivia;
                self.token_end = self.pos;
                Some(MultiLineCommentTrivia)
            }
            b'/' if self.text.as_bytes().get(self.pos + 1) == Some(&b'/') => {
                self.pos += 2;
                while self.pos < self.end {
                    let ch = self.text.as_bytes()[self.pos] as char;
                    if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                        break;
                    }
                    self.pos += 1;
                }
                self.token = SingleLineCommentTrivia;
                self.token_end = self.pos;
                Some(SingleLineCommentTrivia)
            }
            _ => None,
        }
    }

    pub fn scan(&mut self) -> SyntaxKind {
        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;
        self.identifier_value = None;

        self.full_start_pos = self.pos;

        if !self.skip_trivia {
            if let Some(kind) = self.try_scan_trivia_token() {
                return kind;
            }
        }

        let token = loop {
            self.token_pos = self.pos;

            if self.pos >= self.end {
                self.token = SyntaxKind::EndOfFile;
                self.token_end = self.pos;
                break self.token;
            }

            let b = self.text.as_bytes()[self.pos];

            // ASCII fast path: dispatch on the raw byte; non-ASCII falls
            // through to the char decode below.
            if b < 128 {
                match b {
                    b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C => {
                        self.scan_whitespace();
                        continue;
                    }
                    b'/' => {
                        let next = *self.text.as_bytes().get(self.pos + 1).unwrap_or(&0);
                        if next == b'/' {
                            let comment_start = self.pos;
                            self.scan_single_line_comment();
                            self.process_comment_directive(comment_start, self.pos, false);
                            continue;
                        }
                        if next == b'*' {
                            let comment_start = self.pos;
                            self.scan_multi_line_comment();
                            self.process_comment_directive(comment_start, self.pos, true);
                            continue;
                        }
                    }
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                        break self.scan_identifier();
                    }
                    b'0'..=b'9' => {
                        break self.scan_number();
                    }
                    b'.' if self.pos + 1 < self.end
                        && self.text.as_bytes()[self.pos + 1].is_ascii_digit() =>
                    {
                        break self.scan_number();
                    }
                    b'"' | b'\'' => {
                        break self.scan_string(b as char);
                    }
                    b'`' => {
                        break self.scan_template();
                    }
                    b'\\' if *self.text.as_bytes().get(self.pos + 1).unwrap_or(&0) == b'u' => {
                        break self.scan_identifier();
                    }
                    _ => {}
                }
            }

            let c = self.text[self.pos..].chars().next().unwrap();

            if is_whitespace(c) {
                self.scan_whitespace();
                continue;
            }

            if c == '/' && self.pos + 1 < self.end {
                let next = self.text.as_bytes()[self.pos + 1] as char;
                if next == '/' {
                    let comment_start = self.pos;
                    self.scan_single_line_comment();
                    self.process_comment_directive(comment_start, self.pos, false);
                    continue;
                }
                if next == '*' {
                    let comment_start = self.pos;
                    self.scan_multi_line_comment();
                    self.process_comment_directive(comment_start, self.pos, true);
                    continue;
                }
            }

            if is_identifier_start(c) {
                break self.scan_identifier();
            }

            if is_digit(c)
                || (c == '.'
                    && self.pos + 1 < self.end
                    && is_digit(self.text.as_bytes()[self.pos + 1] as char))
            {
                break self.scan_number();
            }

            if c == '"' || c == '\'' {
                break self.scan_string(c);
            }

            if c == '`' {
                break self.scan_template();
            }

            if c == '*'
                && self.skip_jsdoc_leading_asterisks != 0
                && self.preceding_line_break
                && !token_flags_contains(
                    self.token_flags,
                    TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS,
                )
            {
                let next = if self.pos + 1 < self.end {
                    self.text.as_bytes()[self.pos + 1] as char
                } else {
                    '\0'
                };
                if next != '*' && next != '=' {
                    self.pos += 1;
                    self.token_flags |= TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS;
                    continue;
                }
            }

            if c == '#'
                && self.pos == 0
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] == b'!'
            {
                self.pos += 2;
                while self.pos < self.end {
                    let ch = self.text[self.pos..].chars().next().unwrap();
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    self.pos += ch.len_utf8();
                }
                continue;
            }

            if c == '#' {
                break self.scan_private_identifier();
            }

            // Go：`<<<<<<<`/`|||||||`/`=======`/`>>>>>>>` 行首冲突标记（scan_punctuation 前）
            let cb = c as u8;
            if matches!(cb, b'<' | b'=' | b'>' | b'|')
                && cb == *self.text.as_bytes().get(self.pos + 1).unwrap_or(&0)
                && crate::scanner::is_conflict_marker_trivia(&self.text, self.pos)
            {
                self.report_error(
                    DiagnosticKind::RegexMessage(tsox_core::diagnostics::MERGE_CONFLICT_MARKER_ENCOUNTERED),
                    self.pos,
                    MERGE_CONFLICT_MARKER_LENGTH,
                );
                let end = crate::scanner::scan_conflict_marker_trivia(&self.text, self.pos, None);
                self.pos = end;
                self.token_end = end;
                self.token = SyntaxKind::ConflictMarkerTrivia;
                if self.skip_trivia {
                    continue;
                }
                break self.token;
            }

            break self.scan_punctuation();
        };

        self.has_preceding_line_break = self.preceding_line_break;
        if self.preceding_line_break {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
        }
        token
    }

    pub fn scan_template_continuation(&mut self) -> SyntaxKind {
        self.scan_template_continuation_ex(false)
    }

    pub fn scan_template_continuation_ex(&mut self, report_escape_errors: bool) -> SyntaxKind {
        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;
        // Go ReScanTemplateToken：从 `}` 的 tokenStart 重扫，模板段 token 覆盖 `}`
        self.pos = self.token_pos;
        self.token_pos = self.pos;
        self.full_start_pos = self.pos;

        // Go ReScanTemplateToken：段落类型只由结束原因决定——闭反引号=Tail，
        // `${`=Middle；前导 `}` 只是「来自替换」的起点，不决定段落类型
        let mut ended_at_substitution = false;
        if self.pos < self.end && self.text.as_bytes()[self.pos] == b'}' {
            self.pos += 1;
        }
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '`' {
                self.pos += 1;
                break;
            }
            if c == '$'
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] as char == '{'
            {
                self.pos += 2;
                ended_at_substitution = true;
                break;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            if c == '\\' {
                self.scan_escape_sequence(report_escape_errors);
                continue;
            }
            self.pos += 1;
        }

        self.token_end = self.pos;
        self.token = if ended_at_substitution {
            SyntaxKind::TemplateMiddle
        } else {
            SyntaxKind::TemplateTail
        };
        self.has_preceding_line_break = self.preceding_line_break;
        if self.preceding_line_break {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
        }
        self.token
    }

    pub(crate) fn scan_whitespace(&mut self) {
        let bytes = self.text.as_bytes();
        while self.pos < self.end {
            let b = bytes[self.pos];
            if b == b' ' || b == b'\t' || b == 0x0B || b == 0x0C {
                self.pos += 1;
                continue;
            }
            if b == b'\n' || b == b'\r' {
                self.preceding_line_break = true;
                self.pos += 1;
                continue;
            }
            if b < 128 {
                break;
            }
            let c = self.text[self.pos..].chars().next().unwrap();
            if !is_whitespace(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
    }

    pub(crate) fn scan_single_line_comment(&mut self) {
        let bytes = self.text.as_bytes();
        let mut p = self.pos + 2;
        while p < self.end {
            let b = bytes[p];
            if b == b'\n' || b == b'\r' {
                break;
            }
            p += 1;
        }
        self.pos = p;
    }

    pub(crate) fn scan_multi_line_comment(&mut self) {
        let bytes = self.text.as_bytes();
        self.pos += 2;

        let is_jsdoc = self.pos < self.end
            && bytes[self.pos] == b'*'
            && (self.pos + 1 >= self.end || bytes[self.pos + 1] != b'/');
        let comment_start = self.token_pos;
        while self.pos + 1 < self.end {
            if bytes[self.pos] == b'*' && bytes[self.pos + 1] == b'/' {
                self.pos += 2;
                if is_jsdoc {
                    self.token_flags |= TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT;
                    let comment_text = &self.text[comment_start..self.pos];
                    self.token_flags |= scan_jsdoc_comment_for_tags(comment_text);
                }
                return;
            }
            if bytes[self.pos] == b'\n' || bytes[self.pos] == b'\r' {
                self.preceding_line_break = true;
            }
            self.pos += 1;
        }
        self.pos = self.end;
        if is_jsdoc {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT;
            let comment_text = &self.text[comment_start..self.pos];
            self.token_flags |= scan_jsdoc_comment_for_tags(comment_text);
        }
    }
}
