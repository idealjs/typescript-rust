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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    InvalidCharacter,
    UnterminatedStringLiteral,
    UnterminatedTemplateLiteral,
    UnterminatedRegularExpression,
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
        m.insert("</", SyntaxKind::LessThanSlashToken);
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
        m.insert("?.", SyntaxKind::QuestionDotToken);
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
        m.insert(
            ">>>=",
            SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken,
        );
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

static TOKEN_TO_TEXT: OnceLock<HashMap<SyntaxKind, &'static str>> = OnceLock::new();

/// Return the source-text representation of a token kind, matching Go's
/// `scanner.TokenToString`. For punctuation this is the punctuation sequence
/// (e.g., `CommaToken` → `","`); for keywords it is the keyword text (e.g.,
/// `ClassKeyword` → `"class"`); for `Identifier` it returns `"identifier"`.
/// Tokens not in the keyword/punctuation tables return `""`.
pub fn token_to_string(token: SyntaxKind) -> &'static str {
    TOKEN_TO_TEXT
        .get_or_init(|| {
            let mut m = HashMap::new();
            for (&text, &kind) in keywords().iter() {
                m.insert(kind, text);
            }
            for (&text, &kind) in punctuation().iter() {
                m.insert(kind, text);
            }
            m.insert(SyntaxKind::Identifier, "identifier");
            m.insert(SyntaxKind::EndOfFile, "end of file");
            m.insert(SyntaxKind::NumericLiteral, "numeric literal");
            m.insert(SyntaxKind::StringLiteral, "string literal");
            m.insert(SyntaxKind::BigIntLiteral, "bigint literal");
            m.insert(
                SyntaxKind::RegularExpressionLiteral,
                "regular expression literal",
            );
            m.insert(SyntaxKind::TemplateHead, "template literal");
            m.insert(SyntaxKind::TemplateMiddle, "template literal");
            m.insert(SyntaxKind::TemplateTail, "template literal");
            m.insert(
                SyntaxKind::NoSubstitutionTemplateLiteral,
                "template literal",
            );
            m.insert(SyntaxKind::Unknown, "unknown");
            m
        })
        .get(&token)
        .copied()
        .unwrap_or("")
}

/// The lexical scanner.
///
/// Mirrors `scanner.Scanner` in Go.
#[derive(Clone)]
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
    /// Errors collected when no `error_callback` is set (or always, for
    /// retrieval via `take_errors`).
    errors: Vec<ScannerError>,
    /// `@ts-expect-error` / `@ts-ignore` directives collected from comments.
    comment_directives: Vec<CommentDirective>,
}

/// A scanner error: kind + position + length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerError {
    pub kind: DiagnosticKind,
    pub pos: usize,
    pub length: usize,
}

/// Kind of comment directive (`@ts-*` pragma in comments).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentDirectiveKind {
    /// `@ts-expect-error`
    ExpectError,
    /// `@ts-ignore`
    Ignore,
}

/// A comment directive collected from comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentDirective {
    pub pos: usize,
    pub end: usize,
    pub kind: CommentDirectiveKind,
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
            errors: Vec::new(),
            comment_directives: Vec::new(),
        }
    }

    pub fn with_error_callback(mut self, cb: ErrorCallback) -> Self {
        self.error_callback = Some(cb);
        self
    }

    /// Report a scanner error. Calls the error callback if set, and always
    /// stores the error for later retrieval via `take_errors`.
    fn report_error(&mut self, kind: DiagnosticKind, pos: usize, length: usize) {
        if let Some(cb) = self.error_callback {
            cb(kind, pos, length);
        }
        self.errors.push(ScannerError { kind, pos, length });
    }

    /// Drain and return all collected scanner errors.
    pub fn take_errors(&mut self) -> Vec<ScannerError> {
        std::mem::take(&mut self.errors)
    }

    /// All collected comment directives (`@ts-expect-error` / `@ts-ignore`).
    pub fn comment_directives(&self) -> &[CommentDirective] {
        &self.comment_directives
    }

    /// Process a comment for `@ts-*` directives.
    /// Mirrors Go's `Scanner.processCommentDirective`.
    fn process_comment_directive(&mut self, start: usize, end: usize, multiline: bool) {
        let text = self.text.as_bytes();
        let mut pos = start;
        if multiline {
            // Skip whitespace
            while pos < end && (text[pos] == b' ' || text[pos] == b'\t') {
                pos += 1;
            }
            // Skip combinations of / and *
            while pos < end && (text[pos] == b'/' || text[pos] == b'*') {
                pos += 1;
            }
        } else {
            // Skip opening //
            pos += 2;
            // Skip another / if present (for /// triple-slash)
            while pos < end && text[pos] == b'/' {
                pos += 1;
            }
        }
        // Skip whitespace
        while pos < end && (text[pos] == b' ' || text[pos] == b'\t') {
            pos += 1;
        }
        // Directive must start with '@'
        if !(pos < end && text[pos] == b'@') {
            return;
        }
        pos += 1;
        let rest = &self.text[pos..end];
        let kind = if rest.starts_with("ts-expect-error") {
            CommentDirectiveKind::ExpectError
        } else if rest.starts_with("ts-ignore") {
            CommentDirectiveKind::Ignore
        } else {
            return;
        };
        self.comment_directives.push(CommentDirective {
            pos: start,
            end,
            kind,
        });
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
                let comment_start = self.pos;
                self.scan_single_line_comment();
                self.process_comment_directive(comment_start, self.pos, false);
                return self.scan();
            }
            if next == '*' {
                let comment_start = self.pos;
                self.scan_multi_line_comment();
                self.process_comment_directive(comment_start, self.pos, true);
                return self.scan();
            }
        }

        // Identifier or keyword
        if is_identifier_start(c) {
            return self.scan_identifier();
        }

        // Number
        if is_digit(c)
            || (c == '.'
                && self.pos + 1 < self.end
                && is_digit(self.text.as_bytes()[self.pos + 1] as char))
        {
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

    /// Continue scanning a template literal after a `${...}` expression.
    ///
    /// This mirrors the Go scanner's template rescanning path at a small scale:
    /// the parser consumes the `}` for the embedded expression, then asks the
    /// scanner for the following template chunk without skipping trivia.
    pub fn scan_template_continuation(&mut self) -> SyntaxKind {
        self.has_preceding_line_break = self.preceding_line_break;
        self.preceding_line_break = false;
        self.token_pos = self.pos;

        let mut has_substitution = false;
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
                has_substitution = true;
                break;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            if c == '\\' {
                self.pos = (self.pos + 2).min(self.end);
                continue;
            }
            self.pos += 1;
        }

        self.token_end = self.pos;
        self.token = if has_substitution {
            SyntaxKind::TemplateMiddle
        } else {
            SyntaxKind::TemplateTail
        };
        self.token
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
            if c == '*'
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] as char == '/'
            {
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
        if self.text.as_bytes()[self.pos] as char == '0' && self.pos + 1 < self.end {
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
                while self.pos < self.end
                    && (self.text.as_bytes()[self.pos] as char == '0'
                        || self.text.as_bytes()[self.pos] as char == '1')
                {
                    self.pos += 1;
                }
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                return self.token;
            }
            if next == 'o' || next == 'O' {
                // Octal
                self.pos += 2;
                while self.pos < self.end
                    && (self.text.as_bytes()[self.pos] as char >= '0'
                        && self.text.as_bytes()[self.pos] as char <= '7')
                {
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
        let mut terminated = false;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == quote {
                self.pos += 1;
                terminated = true;
                break;
            }
            if c == '\\' {
                self.scan_escape_sequence();
                continue;
            }
            if c == '\n' || c == '\r' {
                // Unterminated string (hit newline before closing quote)
                break;
            }
            self.pos += 1;
        }
        if !terminated {
            self.report_error(
                DiagnosticKind::UnterminatedStringLiteral,
                self.token_pos,
                self.pos - self.token_pos,
            );
        }
        self.token_end = self.pos;
        self.token = SyntaxKind::StringLiteral;
        self.token
    }

    /// Advance `pos` past a `\`-escape sequence. Called when `self.pos` is at
    /// the backslash. Handles `\xHH`, `\uHHHH`, `\u{...}`, octal escapes, line
    /// continuations, and single-character escapes.
    fn scan_escape_sequence(&mut self) {
        // pos is at '\'
        self.pos += 1; // skip backslash
        if self.pos >= self.end {
            return;
        }
        let c = self.text.as_bytes()[self.pos] as char;
        self.pos += 1; // skip the escaped char
        match c {
            'x' => {
                // \xHH — skip up to 2 hex digits
                for _ in 0..2 {
                    if self.pos < self.end && is_hex_digit(self.text.as_bytes()[self.pos] as char) {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
            }
            'u' => {
                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '{' {
                    // \u{...} — skip until '}'
                    while self.pos < self.end && self.text.as_bytes()[self.pos] as char != '}' {
                        self.pos += 1;
                    }
                    if self.pos < self.end {
                        self.pos += 1; // skip '}'
                    }
                } else {
                    // \uHHHH — skip up to 4 hex digits
                    for _ in 0..4 {
                        if self.pos < self.end
                            && is_hex_digit(self.text.as_bytes()[self.pos] as char)
                        {
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            '\r' => {
                // Line continuation: \<CR> or \<CRLF>
                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '\n' {
                    self.pos += 1;
                }
            }
            // Single-char escapes (\n, \t, \b, \f, \v, \0, \\, \', \", \`, and
            // any non-recognized char) need no extra advancement — we already
            // skipped the char after the backslash.
            _ => {}
        }
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
            if c == '$'
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] as char == '{'
            {
                self.pos += 2;
                has_substitution = true;
                break;
            }
            if c == '\\' {
                self.scan_escape_sequence();
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
        // Check 1-char tokens. `remaining` may start with a multi-byte UTF-8
        // character, so slice by the first char's byte length instead of `..1`.
        if best_len == 0 {
            let first_len = remaining.chars().next().map(char::len_utf8).unwrap_or(0);
            if first_len == 1 {
                if let Some(kind) = string_to_token(&remaining[..first_len]) {
                    best_match = Some(kind);
                    best_len = first_len;
                }
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
            self.report_error(DiagnosticKind::InvalidCharacter, start, len);
            SyntaxKind::Unknown
        }
    }

    /// Revert to the position before the last scan.
    pub fn rewind(&mut self) {
        self.pos = self.token_pos;
    }

    /// Re-scan a `>>` (or `>>>`) token as one (or two) `>` tokens, leaving
    /// the remainder for the next scan.
    ///
    /// Go: `reScanGreaterThanToken`. Used after parsing type arguments so that
    /// `Array<Array<T>>` works: the scanner produces `>>` as a single token,
    /// but the parser needs two `>` tokens to close two levels of generics.
    pub fn re_scan_greater_than(&mut self) -> SyntaxKind {
        let token = self.token;
        if token == SyntaxKind::GreaterThanToken {
            return token;
        }
        // Only rescan `>>`-family tokens
        match token {
            SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {}
            _ => return token,
        }
        // Move back one character (the second `>`)
        self.pos = self.token_pos + 1;
        self.token_end = self.pos;
        self.token = SyntaxKind::GreaterThanToken;
        SyntaxKind::GreaterThanToken
    }

    /// Get the token that would remain after a `re_scan_greater_than` call.
    ///
    /// Go: after `reScanGreaterThanToken`, the next `scan()` produces the
    /// "remainder" token (e.g. `>` from `>>`, `=` from `>=`, etc.).
    /// This method returns what that remainder would be, without consuming it.
    pub fn re_scan_greater_than_remainder(&self) -> Option<SyntaxKind> {
        match self.token {
            SyntaxKind::GreaterThanGreaterThanToken => Some(SyntaxKind::GreaterThanToken),
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken => {
                Some(SyntaxKind::GreaterThanGreaterThanToken)
            }
            SyntaxKind::GreaterThanEqualsToken => Some(SyntaxKind::EqualsToken),
            SyntaxKind::GreaterThanGreaterThanEqualsToken => {
                Some(SyntaxKind::GreaterThanEqualsToken)
            }
            SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
                Some(SyntaxKind::GreaterThanGreaterThanEqualsToken)
            }
            _ => None,
        }
    }

    /// Re-scan the current `/` or `/=` token as a regular expression literal.
    ///
    /// Mirrors Go's `Scanner.ReScanSlashToken`. The parser calls this when it
    /// is in a primary-expression position and the scanner has produced a
    /// `SlashToken` or `SlashEqualsToken`. The `=` of `/=` becomes the first
    /// character of the regex pattern (e.g. `/=/` is a regex matching `=`).
    ///
    /// This implementation scans the pattern body (handling `[...]` character
    /// classes and `\` escapes), consumes the closing `/`, then consumes flags
    /// (identifier-part characters). It reports `UnterminatedRegularExpression`
    /// on EOF/newline before a closing `/`. Full regex body validation (the
    /// `regExpParser` in Go's `regexp.go`) is deferred to a later task.
    pub fn re_scan_slash_token(&mut self) -> SyntaxKind {
        if self.token != SyntaxKind::SlashToken && self.token != SyntaxKind::SlashEqualsToken {
            return self.token;
        }

        let start_of_regex_body = self.token_pos + 1; // right after the `/`
        let mut p = start_of_regex_body;
        let mut in_escape = false;
        let mut in_character_class = false;
        let mut unterminated = false;

        while p < self.end {
            let c = self.text.as_bytes()[p] as char;
            if is_line_break(c) {
                unterminated = true;
                break;
            }
            if in_escape {
                in_escape = false;
                p += 1;
                continue;
            }
            match c {
                '\\' => {
                    in_escape = true;
                }
                '/' if !in_character_class => {
                    break; // end of regex body
                }
                '[' => {
                    in_character_class = true;
                }
                ']' if in_character_class => {
                    in_character_class = false;
                }
                _ => {}
            }
            p += 1;
        }

        if unterminated || p >= self.end {
            // Unterminated regex — report error and consume what we have
            self.report_error(
                DiagnosticKind::UnterminatedRegularExpression,
                self.token_pos,
                p - self.token_pos,
            );
            self.pos = p;
        } else {
            // Consume the closing `/`
            p += 1;
            // Consume flags (identifier-part characters)
            while p < self.end {
                let c = self.text.as_bytes()[p] as char;
                if !is_identifier_part(c) {
                    break;
                }
                p += 1;
            }
            self.pos = p;
        }

        self.token_end = self.pos;
        self.token = SyntaxKind::RegularExpressionLiteral;
        self.token
    }

    /// Scan a JSX token. Mirrors Go's `ScanJsxToken`/`ScanJsxTokenEx`.
    ///
    /// Produces `LessThanToken`, `LessThanSlashToken`, `OpenBraceToken`,
    /// `JsxText`, `JsxTextAllWhiteSpaces`, or `EndOfFile`.
    pub fn scan_jsx_token(&mut self) -> SyntaxKind {
        self.scan_jsx_token_ex(true)
    }

    /// Scan a JSX token with optional multiline text control.
    /// When `allow_multiline_jsx_text` is false, JSX text stops at each line
    /// (used by the formatter for correct indentation).
    pub fn scan_jsx_token_ex(&mut self, allow_multiline_jsx_text: bool) -> SyntaxKind {
        self.has_preceding_line_break = self.preceding_line_break;
        self.preceding_line_break = false;
        self.token_pos = self.pos;

        if self.pos >= self.end {
            self.token_end = self.pos;
            self.token = SyntaxKind::EndOfFile;
            return self.token;
        }

        let c = self.text.as_bytes()[self.pos] as char;

        if c == '<' {
            if self.pos + 1 < self.end && self.text.as_bytes()[self.pos + 1] == b'/' {
                self.pos += 2;
            } else {
                self.pos += 1;
            }
            self.token_end = self.pos;
            self.token = if c == '<'
                && self.pos > self.token_pos + 1
                && self.text.as_bytes()[self.token_pos + 1] == b'/'
            {
                SyntaxKind::LessThanSlashToken
            } else {
                SyntaxKind::LessThanToken
            };
            return self.token;
        }

        if c == '{' {
            self.pos += 1;
            self.token_end = self.pos;
            self.token = SyntaxKind::OpenBraceToken;
            return self.token;
        }

        // JSX text: scan until '<', '{', or EOF
        let mut first_non_whitespace = 0usize;
        let start = self.pos;

        while self.pos < self.end {
            let ch = self.text[self.pos..].chars().next().unwrap();
            let size = ch.len_utf8();

            if ch == '{' || ch == '<' {
                break;
            }

            if is_jsx_line_break(ch) && first_non_whitespace == 0 {
                first_non_whitespace = usize::MAX; // -1 sentinel
            } else if !allow_multiline_jsx_text
                && is_jsx_line_break(ch)
                && first_non_whitespace > 0
                && first_non_whitespace != usize::MAX
            {
                break;
            } else if !is_jsx_whitespace_like(ch) {
                first_non_whitespace = self.pos;
            }

            self.pos += size;
        }

        self.token_end = self.pos;
        self.token = if first_non_whitespace == usize::MAX {
            SyntaxKind::JsxTextAllWhiteSpaces
        } else {
            SyntaxKind::JsxText
        };
        let _ = start; // token_text uses token_pos..token_end
        self.token
    }

    /// Extend the current identifier/keyword token with JSX identifier parts
    /// (dashes, colons, and continuation identifier characters).
    /// Mirrors Go's `ScanJsxIdentifier`.
    pub fn scan_jsx_identifier(&mut self) -> SyntaxKind {
        if is_identifier_or_keyword_token(self.token) {
            loop {
                if self.pos >= self.end {
                    break;
                }
                let c = self.text.as_bytes()[self.pos] as char;
                if c == '-' {
                    self.pos += 1;
                    continue;
                }
                // Try to scan identifier parts (unicode escapes, identifier chars)
                let old_pos = self.pos;
                if is_identifier_part(c) {
                    self.pos += c.len_utf8();
                    while self.pos < self.end {
                        let next = self.text[self.pos..].chars().next().unwrap();
                        if is_identifier_part(next) {
                            self.pos += next.len_utf8();
                        } else {
                            break;
                        }
                    }
                }
                if self.pos == old_pos {
                    break;
                }
            }
            self.token_end = self.pos;
            // Re-classify the token: if it's a keyword, keep it; otherwise Identifier
            let text = self.token_text();
            self.token = string_to_keyword(text).unwrap_or(SyntaxKind::Identifier);
        }
        self.token
    }

    /// Scan a JSX attribute value (a quoted string or fall through to regular
    /// scan for `{`). Mirrors Go's `ScanJsxAttributeValue`.
    pub fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        // Skip whitespace between '=' and the value
        while self.pos < self.end {
            let c = self.text[self.pos..].chars().next().unwrap();
            if !is_jsx_whitespace_like(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        self.token_pos = self.pos;

        if self.pos >= self.end {
            self.token_end = self.pos;
            self.token = SyntaxKind::EndOfFile;
            return self.token;
        }

        let c = self.text.as_bytes()[self.pos] as char;
        if c == '"' || c == '\'' {
            return self.scan_string(c);
        }
        // Fall back to regular scan for `{` or anything else
        self.scan()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JSX helper functions
// ────────────────────────────────────────────────────────────────────────────

fn is_jsx_line_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

fn is_jsx_whitespace_like(c: char) -> bool {
    // TypeScript's isWhiteSpaceLike: tab, vtab, formFeed, space, non-breaking space,
    // BOM, and any Unicode "White_Space" property character.
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}') || c.is_whitespace()
}

fn is_identifier_or_keyword_token(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || is_keyword(token)
}

fn is_keyword(token: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_keyword_kind(token)
}

// ────────────────────────────────────────────────────────────────────────────
// Character classification helpers
// ────────────────────────────────────────────────────────────────────────────

fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C' | '\u{A0}' | '\u{FEFF}'
    )
}

fn is_line_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && is_unicode_identifier_start(c))
}

fn is_identifier_part(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && is_unicode_identifier_part(c))
}

fn is_unicode_identifier_start(c: char) -> bool {
    // XID_Start matches ECMAScript's ID_Start (both derive from Unicode TR31
    // with NFKC normalization). The unicode-ident crate provides precise
    // tables generated from the Unicode Character Database.
    unicode_ident::is_xid_start(c)
}

fn is_unicode_identifier_part(c: char) -> bool {
    // XID_Continue matches ECMAScript's ID_Continue. ZWNJ (U+200C) and ZWJ
    // (U+200D) are additionally allowed in ECMAScript identifiers.
    unicode_ident::is_xid_continue(c) || c == '\u{200C}' || c == '\u{200D}'
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
    use std::sync::{Mutex, OnceLock};

    static RECORDED_ERRORS: OnceLock<Mutex<Vec<(DiagnosticKind, usize, usize)>>> = OnceLock::new();

    fn recorded_errors() -> &'static Mutex<Vec<(DiagnosticKind, usize, usize)>> {
        RECORDED_ERRORS.get_or_init(|| Mutex::new(Vec::new()))
    }

    fn record_error(kind: DiagnosticKind, start: usize, length: usize) {
        recorded_errors()
            .lock()
            .unwrap()
            .push((kind, start, length));
    }

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
    fn scan_string_escape_sequences() {
        // \x22 is the hex escape for " — must not prematurely end the string
        let mut s = Scanner::new(r#""\x22""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_text(), r#""\x22""#);
        assert_eq!(s.token_value(), "\"");

        // \u{1F600} extended unicode escape
        let mut s = Scanner::new(r#""\u{1F600}""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "\u{1F600}");

        // \u0041 4-digit unicode escape
        let mut s = Scanner::new(r#""\u0041""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "A");

        // line continuation: backslash + newline is skipped
        let mut s = Scanner::new("\"hello\\\nworld\"");
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "helloworld");

        // escaped backslash followed by quote char
        let mut s = Scanner::new(r#""\\\"""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "\\\"");
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
    fn scan_non_ascii_unknown_characters_do_not_split_utf8() {
        recorded_errors().lock().unwrap().clear();
        let mut s = Scanner::new("· 中 🦀").with_error_callback(record_error);

        assert_eq!(s.scan(), SyntaxKind::Unknown);
        assert_eq!(s.token_text(), "·");

        assert_eq!(s.scan(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "中");

        assert_eq!(s.scan(), SyntaxKind::Unknown);
        assert_eq!(s.token_text(), "🦀");

        assert_eq!(s.scan(), SyntaxKind::EndOfFile);

        let errors = recorded_errors().lock().unwrap();
        assert_eq!(
            errors.as_slice(),
            &[
                (DiagnosticKind::InvalidCharacter, 0, "·".len()),
                (DiagnosticKind::InvalidCharacter, "· 中 ".len(), "🦀".len()),
            ]
        );
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

    #[test]
    fn re_scan_slash_token_basic_regex() {
        let mut s = Scanner::new("/foo/g");
        s.scan(); // SlashToken
        assert_eq!(s.token(), SyntaxKind::SlashToken);
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/foo/g");
    }

    #[test]
    fn re_scan_slash_token_regex_with_flags() {
        let mut s = Scanner::new("/pattern/gim");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/pattern/gim");
    }

    #[test]
    fn re_scan_slash_token_regex_with_char_class() {
        // `/` inside `[...]` should not end the regex
        let mut s = Scanner::new(r"/[\/]/");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), r"/[\/]/");
    }

    #[test]
    fn re_scan_slash_token_regex_with_escape() {
        // Escaped `/` should not end the regex
        let mut s = Scanner::new(r"/a\/b/");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), r"/a\/b/");
    }

    #[test]
    fn re_scan_slash_token_slash_equals() {
        // `/=` should be rescanned: `=` is the first char of the pattern
        let mut s = Scanner::new("/=/");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::SlashEqualsToken);
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/=/");
    }

    #[test]
    fn re_scan_slash_token_unterminated() {
        let mut s = Scanner::new("/foo");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind,
            DiagnosticKind::UnterminatedRegularExpression
        );
    }

    #[test]
    fn re_scan_slash_token_unterminated_newline() {
        let mut s = Scanner::new("/foo\nbar");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind,
            DiagnosticKind::UnterminatedRegularExpression
        );
    }

    #[test]
    fn comment_directive_ts_expect_error_single_line() {
        let mut s = Scanner::new("// @ts-expect-error\n");
        s.scan();
        let directives = s.comment_directives();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, CommentDirectiveKind::ExpectError);
        assert_eq!(directives[0].pos, 0);
        assert_eq!(directives[0].end, 19); // before the \n
    }

    #[test]
    fn comment_directive_ts_ignore_single_line() {
        let mut s = Scanner::new("// @ts-ignore");
        s.scan();
        let directives = s.comment_directives();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, CommentDirectiveKind::Ignore);
    }

    #[test]
    fn comment_directive_triple_slash_ts_ignore() {
        let mut s = Scanner::new("/// @ts-ignore");
        s.scan();
        let directives = s.comment_directives();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, CommentDirectiveKind::Ignore);
    }

    #[test]
    fn comment_directive_multiline_ts_expect_error() {
        let mut s = Scanner::new("/* @ts-expect-error */");
        s.scan();
        let directives = s.comment_directives();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, CommentDirectiveKind::ExpectError);
    }

    #[test]
    fn comment_directive_no_directive_for_regular_comment() {
        let mut s = Scanner::new("// just a regular comment");
        s.scan();
        assert!(s.comment_directives().is_empty());

        let mut s = Scanner::new("/* block comment */");
        s.scan();
        assert!(s.comment_directives().is_empty());
    }

    #[test]
    fn comment_directive_multiple_in_source() {
        let mut s = Scanner::new("// @ts-ignore\nlet x = 1;\n// @ts-expect-error\n");
        while s.scan() != SyntaxKind::EndOfFile {}
        let directives = s.comment_directives();
        assert_eq!(directives.len(), 2);
        assert_eq!(directives[0].kind, CommentDirectiveKind::Ignore);
        assert_eq!(directives[1].kind, CommentDirectiveKind::ExpectError);
    }
}
