#![allow(unused_imports)]

use super::*;

impl Scanner {
    pub fn scan_jsdoc_token(&mut self) -> SyntaxKind {
        self.full_start_pos = self.pos;
        self.token_flags = TOKEN_FLAGS_NONE;
        if self.pos >= self.end {
            self.token = SyntaxKind::EndOfFile;
            return self.token;
        }
        self.token_pos = self.pos;
        let (ch, size) = decode_char(&self.text, self.pos);
        self.pos += size;
        self.token = match ch {
            '\t' | '\x0B' | '\x0C' | ' ' => {
                while self.pos < self.end {
                    let (ch2, size2) = decode_char(&self.text, self.pos);
                    if size2 == 0 || !is_whitespace_single_line(ch2) {
                        break;
                    }
                    self.pos += size2;
                }
                SyntaxKind::WhitespaceTrivia
            }
            '@' => SyntaxKind::AtToken,
            '\r' => {
                if self.pos < self.end && self.text.as_bytes()[self.pos] == b'\n' {
                    self.pos += 1;
                }
                self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
                SyntaxKind::NewLineTrivia
            }
            '\n' => {
                self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
                SyntaxKind::NewLineTrivia
            }
            '*' => SyntaxKind::AsteriskToken,
            '{' => SyntaxKind::OpenBraceToken,
            '}' => SyntaxKind::CloseBraceToken,
            '[' => SyntaxKind::OpenBracketToken,
            ']' => SyntaxKind::CloseBracketToken,
            '(' => SyntaxKind::OpenParenToken,
            ')' => SyntaxKind::CloseParenToken,
            '<' => SyntaxKind::LessThanToken,
            '>' => SyntaxKind::GreaterThanToken,
            '=' => SyntaxKind::EqualsToken,
            ',' => SyntaxKind::CommaToken,
            '.' => SyntaxKind::DotToken,
            '`' => SyntaxKind::BacktickToken,
            '#' => SyntaxKind::HashToken,
            '\\' => SyntaxKind::Unknown,
            _ if is_identifier_start(ch) => {
                while self.pos < self.end {
                    let (next_ch, next_size) = decode_char(&self.text, self.pos);
                    if !is_identifier_part(next_ch) && next_ch != '-' {
                        break;
                    }
                    self.pos += next_size;
                }
                let text = &self.text[self.token_pos..self.pos];
                string_to_keyword(text).unwrap_or(SyntaxKind::Identifier)
            }
            _ => SyntaxKind::Unknown,
        };
        self.token_end = self.pos;
        self.token
    }

    pub fn scan_jsdoc_comment_text_token(&mut self, in_backticks: bool) -> SyntaxKind {
        self.full_start_pos = self.pos;
        self.token_flags = TOKEN_FLAGS_NONE;
        if self.pos >= self.end {
            self.token = SyntaxKind::EndOfFile;
            return self.token;
        }
        self.token_pos = self.pos;
        while self.pos < self.end {
            let (ch, size) = decode_char(&self.text, self.pos);
            if is_line_break(ch) || ch == '`' {
                break;
            }
            if !in_backticks {
                if ch == '{' {
                    break;
                } else if ch == '@' {
                    let prev = if self.pos > 0 {
                        decode_char(&self.text, self.pos - size).0
                    } else {
                        '\0'
                    };
                    if is_whitespace_single_line(prev) {
                        let next_pos = self.pos + size;
                        let next = if next_pos < self.end {
                            decode_char(&self.text, next_pos).0
                        } else {
                            '\0'
                        };
                        if is_identifier_start(next) {
                            break;
                        }
                    }
                }
            }
            self.pos += size;
        }
        if self.pos == self.token_pos {
            return self.scan_jsdoc_token();
        }
        self.token = SyntaxKind::JSDocCommentTextToken;
        self.token_end = self.pos;
        self.token
    }
}
