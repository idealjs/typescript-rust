//! Lexical scanner, ported from `internal/scanner/scanner.go`.
//!
//! The scanner tokenizes TypeScript source text into `SyntaxKind` tokens.
//! This is a simplified initial port covering identifiers, keywords,
//! numbers, strings, and punctuation. Full escape-sequence and regex
//! scanning will be added incrementally.

use crate::ast::SyntaxKind;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Callback for reporting scan errors.
pub type ErrorCallback = fn(kind: DiagnosticKind, start: usize, length: usize);

/// Simplified diagnostic kinds for the scanner.
#[derive(Debug, Clone, Copy)]
pub enum DiagnosticKind {
    InvalidCharacter,
    UnterminatedStringLiteral,
    UnterminatedTemplateLiteral,
}

/// Keywords mapping (text → SyntaxKind).
static TEXT_TO_KEYWORD: OnceLock<HashMap<&'static str, SyntaxKind>> = OnceLock::new();

/// Punctuation mapping (text → SyntaxKind).
static TEXT_TO_TOKEN: OnceLock<HashMap<&'static str, SyntaxKind>> = OnceLock::new();

fn keywords() -> &'static HashMap<&'static str, SyntaxKind> {
    TEXT_TO_KEYWORD.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("abstract", SyntaxKind::AbstractKeyword);
        m.insert("accessor", SyntaxKind::AccessorKeyword);
        m.insert("any", SyntaxKind::AnyKeyword);
        m.insert("as", SyntaxKind::AsKeyword);
        m.insert("asserts", SyntaxKind::AssertsKeyword);
        m.insert("assert", SyntaxKind::AssertKeyword);
        m.insert("bigint", SyntaxKind::BigIntKeyword);
        m.insert("boolean", SyntaxKind::BooleanKeyword);
        m.insert("break", SyntaxKind::BreakKeyword);
        m.insert("case", SyntaxKind::CaseKeyword);
        m.insert("catch", SyntaxKind::CatchKeyword);
        m.insert("class", SyntaxKind::ClassKeyword);
        m.insert("continue", SyntaxKind::ContinueKeyword);
        m.insert("const", SyntaxKind::ConstKeyword);
        m.insert("constructor", SyntaxKind::ConstructorKeyword);
        m.insert("debugger", SyntaxKind::DebuggerKeyword);
        m.insert("declare", SyntaxKind::DeclareKeyword);
        m.insert("default", SyntaxKind::DefaultKeyword);
        m.insert("defer", SyntaxKind::DeferKeyword);
        m.insert("delete", SyntaxKind::DeleteKeyword);
        m.insert("do", SyntaxKind::DoKeyword);
        m.insert("else", SyntaxKind::ElseKeyword);
        m.insert("enum", SyntaxKind::EnumKeyword);
        m.insert("export", SyntaxKind::ExportKeyword);
        m.insert("extends", SyntaxKind::ExtendsKeyword);
        m.insert("false", SyntaxKind::FalseKeyword);
        m.insert("finally", SyntaxKind::FinallyKeyword);
        m.insert("for", SyntaxKind::ForKeyword);
        m.insert("from", SyntaxKind::FromKeyword);
        m.insert("function", SyntaxKind::FunctionKeyword);
        m.insert("get", SyntaxKind::GetKeyword);
        m.insert("if", SyntaxKind::IfKeyword);
        m.insert("immediate", SyntaxKind::ImmediateKeyword);
        m.insert("implements", SyntaxKind::ImplementsKeyword);
        m.insert("import", SyntaxKind::ImportKeyword);
        m.insert("in", SyntaxKind::InKeyword);
        m.insert("infer", SyntaxKind::InferKeyword);
        m.insert("instanceof", SyntaxKind::InstanceOfKeyword);
        m.insert("interface", SyntaxKind::InterfaceKeyword);
        m.insert("intrinsic", SyntaxKind::IntrinsicKeyword);
        m.insert("is", SyntaxKind::IsKeyword);
        m.insert("keyof", SyntaxKind::KeyOfKeyword);
        m.insert("let", SyntaxKind::LetKeyword);
        m.insert("module", SyntaxKind::ModuleKeyword);
        m.insert("namespace", SyntaxKind::NamespaceKeyword);
        m.insert("never", SyntaxKind::NeverKeyword);
        m.insert("new", SyntaxKind::NewKeyword);
        m.insert("null", SyntaxKind::NullKeyword);
        m.insert("number", SyntaxKind::NumberKeyword);
        m.insert("object", SyntaxKind::ObjectKeyword);
        m.insert("of", SyntaxKind::OfKeyword);
        m.insert("out", SyntaxKind::OutKeyword);
        m.insert("override", SyntaxKind::OverrideKeyword);
        m.insert("package", SyntaxKind::PackageKeyword);
        m.insert("private", SyntaxKind::PrivateKeyword);
        m.insert("protected", SyntaxKind::ProtectedKeyword);
        m.insert("public", SyntaxKind::PublicKeyword);
        m.insert("readonly", SyntaxKind::ReadonlyKeyword);
        m.insert("require", SyntaxKind::RequireKeyword);
        m.insert("global", SyntaxKind::GlobalKeyword);
        m.insert("return", SyntaxKind::ReturnKeyword);
        m.insert("satisfies", SyntaxKind::SatisfiesKeyword);
        m.insert("set", SyntaxKind::SetKeyword);
        m.insert("static", SyntaxKind::StaticKeyword);
        m.insert("string", SyntaxKind::StringKeyword);
        m.insert("super", SyntaxKind::SuperKeyword);
        m.insert("switch", SyntaxKind::SwitchKeyword);
        m.insert("symbol", SyntaxKind::SymbolKeyword);
        m.insert("this", SyntaxKind::ThisKeyword);
        m.insert("throw", SyntaxKind::ThrowKeyword);
        m.insert("true", SyntaxKind::TrueKeyword);
        m.insert("try", SyntaxKind::TryKeyword);
        m.insert("type", SyntaxKind::TypeKeyword);
        m.insert("typeof", SyntaxKind::TypeOfKeyword);
        m.insert("undefined", SyntaxKind::UndefinedKeyword);
        m.insert("unique", SyntaxKind::UniqueKeyword);
        m.insert("unknown", SyntaxKind::UnknownKeyword);
        m.insert("using", SyntaxKind::UsingKeyword);
        m.insert("var", SyntaxKind::VarKeyword);
        m.insert("void", SyntaxKind::VoidKeyword);
        m.insert("while", SyntaxKind::WhileKeyword);
        m.insert("with", SyntaxKind::WithKeyword);
        m.insert("yield", SyntaxKind::YieldKeyword);
        m.insert("async", SyntaxKind::AsyncKeyword);
        m.insert("await", SyntaxKind::AwaitKeyword);
        m
    })
}

fn punctuation() -> &'static HashMap<&'static str, SyntaxKind> {
    TEXT_TO_TOKEN.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("{", SyntaxKind::OpenBraceToken);
        m.insert("}", SyntaxKind::CloseBraceToken);
        m.insert("(", SyntaxKind::OpenParenToken);
        m.insert(")", SyntaxKind::CloseParenToken);
        m.insert("[", SyntaxKind::OpenBracketToken);
        m.insert("]", SyntaxKind::CloseBracketToken);
        m.insert(".", SyntaxKind::DotToken);
        m.insert("...", SyntaxKind::DotDotDotToken);
        m.insert(";", SyntaxKind::SemicolonToken);
        m.insert(",", SyntaxKind::CommaToken);
        m.insert("<", SyntaxKind::LessThanToken);
        m.insert(">", SyntaxKind::GreaterThanToken);
        m.insert("<=", SyntaxKind::LessThanEqualsToken);
        m.insert(">=", SyntaxKind::GreaterThanEqualsToken);
        m.insert("==", SyntaxKind::EqualsEqualsToken);
        m.insert("!=", SyntaxKind::ExclamationEqualsToken);
        m.insert("===", SyntaxKind::EqualsEqualsEqualsToken);
        m.insert("!==", SyntaxKind::ExclamationEqualsEqualsToken);
        m.insert("=>", SyntaxKind::EqualsGreaterThanToken);
        m.insert("+", SyntaxKind::PlusToken);
        m.insert("-", SyntaxKind::MinusToken);
        m.insert("**", SyntaxKind::AsteriskAsteriskToken);
        m.insert("*", SyntaxKind::AsteriskToken);
        m.insert("/", SyntaxKind::SlashToken);
        m.insert("%", SyntaxKind::PercentToken);
        m.insert("++", SyntaxKind::PlusPlusToken);
        m.insert("--", SyntaxKind::MinusMinusToken);
        m.insert("<<", SyntaxKind::LessThanLessThanToken);
        m.insert(">>", SyntaxKind::GreaterThanGreaterThanToken);
        m.insert(">>>", SyntaxKind::GreaterThanGreaterThanGreaterThanToken);
        m.insert("&", SyntaxKind::AmpersandToken);
        m.insert("|", SyntaxKind::BarToken);
        m.insert("^", SyntaxKind::CaretToken);
        m.insert("!", SyntaxKind::ExclamationToken);
        m.insert("~", SyntaxKind::TildeToken);
        m.insert("&&", SyntaxKind::AmpersandAmpersandToken);
        m.insert("||", SyntaxKind::BarBarToken);
        m.insert("?", SyntaxKind::QuestionToken);
        m.insert(":", SyntaxKind::ColonToken);
        m.insert("@", SyntaxKind::AtToken);
        m.insert("??", SyntaxKind::QuestionQuestionToken);
        m.insert("=", SyntaxKind::EqualsToken);
        m.insert("+=", SyntaxKind::PlusEqualsToken);
        m.insert("-=", SyntaxKind::MinusEqualsToken);
        m.insert("*=", SyntaxKind::AsteriskEqualsToken);
        m.insert("**=", SyntaxKind::AsteriskAsteriskEqualsToken);
        m.insert("/=", SyntaxKind::SlashEqualsToken);
        m.insert("%=", SyntaxKind::PercentEqualsToken);
        m.insert("<<=", SyntaxKind::LessThanLessThanEqualsToken);
        m.insert(">>=", SyntaxKind::GreaterThanGreaterThanEqualsToken);
        m.insert(">>>=", SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken);
        m.insert("&=", SyntaxKind::AmpersandEqualsToken);
        m.insert("|=", SyntaxKind::BarEqualsToken);
        m.insert("^=", SyntaxKind::CaretEqualsToken);
        m.insert("||=", SyntaxKind::BarBarEqualsToken);
        m.insert("&&=", SyntaxKind::AmpersandAmpersandEqualsToken);
        m.insert("??=", SyntaxKind::QuestionQuestionEqualsToken);
        m
    })
}

/// Look up a keyword by text. Returns `None` if not a keyword.
pub fn string_to_keyword(text: &str) -> Option<SyntaxKind> {
    keywords().get(text).copied()
}

/// Look up a punctuation token by text.
pub fn string_to_token(text: &str) -> Option<SyntaxKind> {
    punctuation().get(text).copied()
}

/// The lexical scanner.
///
/// Mirrors `scanner.Scanner` in Go.
pub struct Scanner {
    text: String,
    pos: usize,
    end: usize,
    token: SyntaxKind,
    token_pos: usize,
    token_end: usize,
    preceding_line_break: bool,
    has_preceding_line_break: bool,
    error_callback: Option<ErrorCallback>,
}

/// Scanner options.
#[derive(Debug, Clone, Default)]
pub struct ScannerOptions {
    pub language_variant: crate::ast::LanguageVariant,
    pub skip_trivia: bool,
}

impl Scanner {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let len = text.len();
        Self {
            text,
            pos: 0,
            end: len,
            token: SyntaxKind::Unknown,
            token_pos: 0,
            token_end: 0,
            preceding_line_break: false,
            has_preceding_line_break: false,
            error_callback: None,
        }
    }

    pub fn with_error_callback(mut self, cb: ErrorCallback) -> Self {
        self.error_callback = Some(cb);
        self
    }

    /// The current token's kind.
    pub fn token(&self) -> SyntaxKind {
        self.token
    }

    /// The start position of the current token.
    pub fn token_pos(&self) -> usize {
        self.token_pos
    }

    /// The end position of the current token.
    pub fn token_end(&self) -> usize {
        self.token_end
    }

    /// The text of the current token.
    pub fn token_text(&self) -> &str {
        &self.text[self.token_pos..self.token_end]
    }

    /// The value of the current token (for string/template literals, this is
    /// the unquoted, unescaped value; for other tokens, same as `token_text`).
    pub fn token_value(&self) -> String {
        let text = self.token_text();
        if text.len() >= 2 {
            let first = text.as_bytes()[0];
            let last = text.as_bytes()[text.len() - 1];
            if (first == b'"' && last == b'"')
                || (first == b'\'' && last == b'\'')
                || (first == b'`' && last == b'`')
            {
                return unescape_string(&text[1..text.len() - 1]);
            }
        }
        text.to_string()
    }

    /// Whether the current token is preceded by a line break.
    pub fn has_preceding_line_break(&self) -> bool {
        self.has_preceding_line_break
    }

    /// Current scan position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// The full source text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Scan the next token and return its kind.
    pub fn scan(&mut self) -> SyntaxKind {
        self.has_preceding_line_break = self.preceding_line_break;
        self.preceding_line_break = false;

        self.token_pos = self.pos;

        if self.pos >= self.end {
            self.token = SyntaxKind::EndOfFile;
            self.token_end = self.pos;
            return self.token;
        }

        // Decode the actual UTF-8 character at the current position.
        // For ASCII bytes, this is equivalent to `as_bytes()[pos] as char`,
        // but for multi-byte characters (e.g., CJK), it correctly decodes
        // the full codepoint instead of just the first byte.
        let c = self.text[self.pos..].chars().next().unwrap();

        // Skip trivia (whitespace, comments) if applicable
        if is_whitespace(c) {
            self.scan_whitespace();
            return self.scan();
        }

        if c == '/' && self.pos + 1 < self.end {
            let next = self.text.as_bytes()[self.pos + 1] as char;
            if next == '/' {
                self.scan_single_line_comment();
                return self.scan();
            }
            if next == '*' {
                self.scan_multi_line_comment();
                return self.scan();
            }
        }

        // Identifier or keyword
        if is_identifier_start(c) {
            return self.scan_identifier();
        }

        // Number
        if is_digit(c) || (c == '.' && self.pos + 1 < self.end && is_digit(self.text.as_bytes()[self.pos + 1] as char)) {
            return self.scan_number();
        }

        // String
        if c == '"' || c == '\'' {
            return self.scan_string(c);
        }

        // Template literal start
        if c == '`' {
            return self.scan_template();
        }

        // Punctuation
        self.scan_punctuation()
    }

    fn scan_whitespace(&mut self) {
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if !is_whitespace(c) {
                break;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            self.pos += 1;
        }
    }

    fn scan_single_line_comment(&mut self) {
        // Skip //
        self.pos += 2;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '\n' || c == '\r' {
                break;
            }
            self.pos += 1;
        }
    }

    fn scan_multi_line_comment(&mut self) {
        // Skip /*
        self.pos += 2;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '*' && self.pos + 1 < self.end && self.text.as_bytes()[self.pos + 1] as char == '/' {
                self.pos += 2;
                break;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            self.pos += 1;
        }
    }

    fn scan_identifier(&mut self) -> SyntaxKind {
        let start = self.pos;
        // Advance past the first character (already validated as identifier start).
        // Use len_utf8() to correctly handle multi-byte characters (e.g., CJK).
        let first_c = self.text[self.pos..].chars().next().unwrap();
        self.pos += first_c.len_utf8();
        while self.pos < self.end {
            let c = self.text[self.pos..].chars().next().unwrap();
            if !is_identifier_part(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        self.token_end = self.pos;
        let text = &self.text[start..self.pos];
        self.token = string_to_keyword(text).unwrap_or(SyntaxKind::Identifier);
        self.token
    }

    fn scan_number(&mut self) -> SyntaxKind {
        let start = self.pos;
        if self.text.as_bytes()[self.pos] as char == '0'
            && self.pos + 1 < self.end
        {
            let next = self.text.as_bytes()[self.pos + 1] as char;
            if next == 'x' || next == 'X' {
                // Hex
                self.pos += 2;
                while self.pos < self.end && is_hex_digit(self.text.as_bytes()[self.pos] as char) {
                    self.pos += 1;
                }
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                return self.token;
            }
            if next == 'b' || next == 'B' {
                // Binary
                self.pos += 2;
                while self.pos < self.end && (self.text.as_bytes()[self.pos] as char == '0' || self.text.as_bytes()[self.pos] as char == '1') {
                    self.pos += 1;
                }
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                return self.token;
            }
            if next == 'o' || next == 'O' {
                // Octal
                self.pos += 2;
                while self.pos < self.end && (self.text.as_bytes()[self.pos] as char >= '0' && self.text.as_bytes()[self.pos] as char <= '7') {
                    self.pos += 1;
                }
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                return self.token;
            }
        }

        // Decimal
        while self.pos < self.end && is_digit(self.text.as_bytes()[self.pos] as char) {
            self.pos += 1;
        }
        // Fractional part
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '.' {
            self.pos += 1;
            while self.pos < self.end && is_digit(self.text.as_bytes()[self.pos] as char) {
                self.pos += 1;
            }
        }
        // Exponent
        if self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == 'e' || c == 'E' {
                self.pos += 1;
                if self.pos < self.end {
                    let sign = self.text.as_bytes()[self.pos] as char;
                    if sign == '+' || sign == '-' {
                        self.pos += 1;
                    }
                }
                while self.pos < self.end && is_digit(self.text.as_bytes()[self.pos] as char) {
                    self.pos += 1;
                }
            }
        }

        // BigInt suffix
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == 'n' {
            self.pos += 1;
            self.token_end = self.pos;
            self.token = SyntaxKind::BigIntLiteral;
            return self.token;
        }

        let _ = start;
        self.token_end = self.pos;
        self.token = SyntaxKind::NumericLiteral;
        self.token
    }

    fn scan_string(&mut self, quote: char) -> SyntaxKind {
        self.pos += 1; // skip opening quote
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == quote {
                self.pos += 1;
                break;
            }
            if c == '\\' {
                self.pos += 2; // skip escape sequence (simplified)
                continue;
            }
            if c == '\n' || c == '\r' {
                // Unterminated string
                if let Some(cb) = self.error_callback {
                    cb(DiagnosticKind::UnterminatedStringLiteral, self.token_pos, self.pos - self.token_pos);
                }
                break;
            }
            self.pos += 1;
        }
        self.token_end = self.pos;
        self.token = if quote == '"' {
            SyntaxKind::StringLiteral
        } else {
            SyntaxKind::StringLiteral
        };
        self.token
    }

    fn scan_template(&mut self) -> SyntaxKind {
        // Simplified: scan until ` or ${
        self.pos += 1; // skip opening `
        let mut has_substitution = false;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '`' {
                self.pos += 1;
                break;
            }
            if c == '$' && self.pos + 1 < self.end && self.text.as_bytes()[self.pos + 1] as char == '{' {
                self.pos += 2;
                has_substitution = true;
                break;
            }
            if c == '\\' {
                self.pos += 2;
                continue;
            }
            self.pos += 1;
        }
        self.token_end = self.pos;
        self.token = if has_substitution {
            SyntaxKind::TemplateHead
        } else {
            SyntaxKind::NoSubstitutionTemplateLiteral
        };
        self.token
    }

    fn scan_punctuation(&mut self) -> SyntaxKind {
        let start = self.pos;
        // Try to match the longest punctuation token
        let remaining = &self.text[start..];
        let mut best_match: Option<SyntaxKind> = None;
        let mut best_len = 0;

        // Check 3-char tokens. Use `get(..3)` for safety: if byte 3 falls
        // inside a multi-byte UTF-8 character, `get` returns `None`.
        if remaining.len() >= 3 {
            if let Some(slice) = remaining.get(..3) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 3;
                }
            }
        }
        // Check 2-char tokens
        if best_len == 0 && remaining.len() >= 2 {
            if let Some(slice) = remaining.get(..2) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 2;
                }
            }
        }
        // Check 1-char tokens
        if best_len == 0 && remaining.len() >= 1 {
            if let Some(kind) = string_to_token(&remaining[..1]) {
                best_match = Some(kind);
                best_len = 1;
            }
        }

        if let Some(kind) = best_match {
            self.pos += best_len;
            self.token_end = self.pos;
            self.token = kind;
            kind
        } else {
            // Unknown character — advance by the full UTF-8 character length
            // to avoid leaving pos in the middle of a multi-byte character.
            let c = self.text[start..].chars().next().unwrap();
            let len = c.len_utf8();
            self.pos += len;
            self.token_end = self.pos;
            self.token = SyntaxKind::Unknown;
            if let Some(cb) = self.error_callback {
                cb(DiagnosticKind::InvalidCharacter, start, len);
            }
            SyntaxKind::Unknown
        }
    }

    /// Revert to the position before the last scan.
    pub fn rewind(&mut self) {
        self.pos = self.token_pos;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Character classification helpers
// ────────────────────────────────────────────────────────────────────────────

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C' | '\u{A0}' | '\u{FEFF}')
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$' || (!c.is_ascii() && is_unicode_identifier_start(c))
}

fn is_identifier_part(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$' || (!c.is_ascii() && is_unicode_identifier_part(c))
}

fn is_unicode_identifier_start(c: char) -> bool {
    // Simplified: use Unicode category checking
    // A more complete implementation would use the unicode-ident crate
    c.is_alphabetic()
}

fn is_unicode_identifier_part(c: char) -> bool {
    c.is_alphanumeric() || c == '\u{200C}' || c == '\u{200D}'
}

/// Unescape a string value (handle common escape sequences).
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('v') => result.push('\u{000B}'),
                Some('0') => result.push('\0'),
                Some('x') => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(n) = u32::from_str_radix(&hex, 16) {
                        result.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                    }
                }
                Some('u') => {
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
                        if let Ok(n) = u32::from_str_radix(&hex, 16) {
                            result.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                        }
                    } else {
                        let hex: String = chars.by_ref().take(4).collect();
                        if let Ok(n) = u32::from_str_radix(&hex, 16) {
                            result.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                        }
                    }
                }
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some('`') => result.push('`'),
                Some('\n') => {} // line continuation
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_identifiers_and_keywords() {
        let mut s = Scanner::new("foo const let");
        assert_eq!(s.scan(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "foo");
        assert_eq!(s.scan(), SyntaxKind::ConstKeyword);
        assert_eq!(s.token_text(), "const");
        assert_eq!(s.scan(), SyntaxKind::LetKeyword);
        assert_eq!(s.token_text(), "let");
        assert_eq!(s.scan(), SyntaxKind::EndOfFile);
    }

    #[test]
    fn scan_numbers() {
        let mut s = Scanner::new("42 3.14 0x1F 0b101 0o77 100n");
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.token_text(), "42");
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.token_text(), "3.14");
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.token_text(), "0x1F");
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.token_text(), "0b101");
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.token_text(), "0o77");
        assert_eq!(s.scan(), SyntaxKind::BigIntLiteral);
        assert_eq!(s.token_text(), "100n");
    }

    #[test]
    fn scan_strings() {
        let mut s = Scanner::new("\"hello\" 'world'");
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_text(), "\"hello\"");
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_text(), "'world'");
    }

    #[test]
    fn scan_punctuation() {
        let mut s = Scanner::new("=> === ... ??=");
        assert_eq!(s.scan(), SyntaxKind::EqualsGreaterThanToken);
        assert_eq!(s.scan(), SyntaxKind::EqualsEqualsEqualsToken);
        assert_eq!(s.scan(), SyntaxKind::DotDotDotToken);
        assert_eq!(s.scan(), SyntaxKind::QuestionQuestionEqualsToken);
    }

    #[test]
    fn scan_comments() {
        let mut s = Scanner::new("// comment\nfoo /* block */ bar");
        assert_eq!(s.scan(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "foo");
        assert!(s.has_preceding_line_break());
        assert_eq!(s.scan(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "bar");
    }

    #[test]
    fn scan_template_literal() {
        let mut s = Scanner::new("`hello`");
        assert_eq!(s.scan(), SyntaxKind::NoSubstitutionTemplateLiteral);
        assert_eq!(s.token_text(), "`hello`");

        let mut s = Scanner::new("`hello ${");
        assert_eq!(s.scan(), SyntaxKind::TemplateHead);
    }

    #[test]
    fn keyword_lookup() {
        assert_eq!(string_to_keyword("class"), Some(SyntaxKind::ClassKeyword));
        assert_eq!(string_to_keyword("foobar"), None);
    }
}
