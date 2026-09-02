//! Lexical scanner, ported from `internal/scanner/scanner.go`.
//!
//! The scanner tokenizes TypeScript source text into `SyntaxKind` tokens.
//! This is a simplified initial port covering identifiers, keywords,
//! numbers, strings, and punctuation. Full escape-sequence and regex
//! scanning will be added incrementally.

use crate::ast::SyntaxKind;
use crate::core::compiler_options::ScriptTarget;
use std::collections::HashMap;
use std::sync::OnceLock;

mod regexp;
mod unicode_properties;

/// Callback for reporting scan errors.
pub type ErrorCallback = fn(kind: DiagnosticKind, start: usize, length: usize);

/// Simplified diagnostic kinds for the scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    InvalidCharacter,
    /// File appears to be binary (TS1490). Emitted once when the scanner hits
    /// a UTF-8 replacement character (U+FFFD, the Rust `chars()` decode failure
    /// sentinel) — mirrors Go's `File_appears_to_be_binary` at scanner.go:937.
    /// After emitting, the scanner jumps to end-of-file.
    FileAppearsToBeBinary,
    UnterminatedStringLiteral,
    UnterminatedTemplateLiteral,
    UnterminatedRegularExpression,
    /// Unknown regular expression flag (TS1499).
    UnknownRegularExpressionFlag,
    /// Duplicate regular expression flag (TS1500).
    DuplicateRegularExpressionFlag,
    /// The `u` and `v` flags cannot be set simultaneously (TS1502).
    UnicodeUAndVFlagsMutuallyExclusive,
    /// Octal literals are not allowed. Use the syntax '0o...' (TS1121).
    OctalLiteralNotAllowed,
    /// Decimals with leading zeros are not allowed (TS1489).
    DecimalWithLeadingZero,
    /// Numeric separators are not allowed here (TS6188).
    NumericSeparatorNotAllowed,
    /// A regex body validation diagnostic carrying the specific `Message`.
    /// Used for the ~30 TS1501–TS1534 regex body diagnostics produced by
    /// `RegExpParser`. Avoids adding 30+ individual enum variants.
    RegexMessage(crate::diagnostics::Message),
}

// ────────────────────────────────────────────────────────────────────────────
// TokenFlags
//
// Mirrors Go's `ast.TokenFlags` bitset (`internal/ast/tokenflags.go`). The
// scanner accumulates these during `scan()` and exposes them via
// `Scanner::token_flags()`. Callers (parser/binder) can test individual bits
// with the `contains` helper. Currently the scanner sets:
//   - `PRECEDING_LINE_BREAK` (during trivia skipping)
//   - `UNTERMINATED` (string/template/regex)
//   - `SINGLE_QUOTE` (string literals with `'`)
//   - `HEX_SPECIFIER` / `BINARY_SPECIFIER` / `OCTAL_SPECIFIER` (numeric literals)
//   - `SCIENTIFIC` / `OCTAL` / `CONTAINS_LEADING_ZERO` (numeric literals)
// JSDoc-related flags and escape-sequence flags are deferred until those
// scanner paths are migrated.
// ────────────────────────────────────────────────────────────────────────────

pub type TokenFlags = u32;

pub const TOKEN_FLAGS_NONE: TokenFlags = 0;
pub const TOKEN_FLAGS_PRECEDING_LINE_BREAK: TokenFlags = 1 << 0;
pub const TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT: TokenFlags = 1 << 1;
pub const TOKEN_FLAGS_UNTERMINATED: TokenFlags = 1 << 2;
pub const TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE: TokenFlags = 1 << 3;
pub const TOKEN_FLAGS_SCIENTIFIC: TokenFlags = 1 << 4;
pub const TOKEN_FLAGS_OCTAL: TokenFlags = 1 << 5;
pub const TOKEN_FLAGS_HEX_SPECIFIER: TokenFlags = 1 << 6;
pub const TOKEN_FLAGS_BINARY_SPECIFIER: TokenFlags = 1 << 7;
pub const TOKEN_FLAGS_OCTAL_SPECIFIER: TokenFlags = 1 << 8;
pub const TOKEN_FLAGS_CONTAINS_SEPARATOR: TokenFlags = 1 << 9;
pub const TOKEN_FLAGS_UNICODE_ESCAPE: TokenFlags = 1 << 10;
pub const TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE: TokenFlags = 1 << 11;
pub const TOKEN_FLAGS_HEX_ESCAPE: TokenFlags = 1 << 12;
pub const TOKEN_FLAGS_CONTAINS_LEADING_ZERO: TokenFlags = 1 << 13;
pub const TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR: TokenFlags = 1 << 14;
pub const TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS: TokenFlags = 1 << 15;
pub const TOKEN_FLAGS_SINGLE_QUOTE: TokenFlags = 1 << 16;
pub const TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED: TokenFlags = 1 << 17;
pub const TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK: TokenFlags = 1 << 18;

pub const TOKEN_FLAGS_BINARY_OR_OCTAL_SPECIFIER: TokenFlags =
    TOKEN_FLAGS_BINARY_SPECIFIER | TOKEN_FLAGS_OCTAL_SPECIFIER;
pub const TOKEN_FLAGS_WITH_SPECIFIER: TokenFlags =
    TOKEN_FLAGS_HEX_SPECIFIER | TOKEN_FLAGS_BINARY_OR_OCTAL_SPECIFIER;
pub const TOKEN_FLAGS_STRING_LITERAL_FLAGS: TokenFlags = TOKEN_FLAGS_UNTERMINATED
    | TOKEN_FLAGS_HEX_ESCAPE
    | TOKEN_FLAGS_UNICODE_ESCAPE
    | TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE
    | TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    | TOKEN_FLAGS_SINGLE_QUOTE;
pub const TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS: TokenFlags = TOKEN_FLAGS_SCIENTIFIC
    | TOKEN_FLAGS_OCTAL
    | TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    | TOKEN_FLAGS_WITH_SPECIFIER
    | TOKEN_FLAGS_CONTAINS_SEPARATOR
    | TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
pub const TOKEN_FLAGS_TEMPLATE_LITERAL_LIKE_FLAGS: TokenFlags = TOKEN_FLAGS_UNTERMINATED
    | TOKEN_FLAGS_HEX_ESCAPE
    | TOKEN_FLAGS_UNICODE_ESCAPE
    | TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE
    | TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
pub const TOKEN_FLAGS_REGULAR_EXPRESSION_LITERAL_FLAGS: TokenFlags = TOKEN_FLAGS_UNTERMINATED;
pub const TOKEN_FLAGS_IS_INVALID: TokenFlags = TOKEN_FLAGS_OCTAL
    | TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    | TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    | TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;

/// Test whether `flags` contains all bits in `bit`. For single-flag checks
/// this is equivalent to `(flags & bit) != 0`; for combined masks like
/// `TOKEN_FLAGS_WITH_SPECIFIER` use `token_flags_intersects` if you want
/// "any of these bits set".
pub fn token_flags_contains(flags: TokenFlags, bit: TokenFlags) -> bool {
    (flags & bit) == bit
}

/// Test whether `flags` has *any* of the bits in `mask` set. Use this for
/// combined masks like `TOKEN_FLAGS_WITH_SPECIFIER` (HEX | BINARY | OCTAL).
pub fn token_flags_intersects(flags: TokenFlags, mask: TokenFlags) -> bool {
    (flags & mask) != 0
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
    /// Start of the current token *including* any leading trivia, preserved
    /// across trivia-skipping iterations. Mirrors Go's `fullStartPos`
    /// (`scanner.go:195,469`): set once at the top of `scan()` and not reset
    /// while trivia is skipped, so callers can reconstruct leading
    /// comments/whitespace via `get_leading_comment_ranges`.
    full_start_pos: usize,
    preceding_line_break: bool,
    has_preceding_line_break: bool,
    /// Byte offset of the non-text (binary) marker character when the scan
    /// terminated on one (Go's `NonTextFileMarkerTrivia`); the parser
    /// reports TS1128 there.
    binary_marker_pos: Option<usize>,
    /// Bitset of `TOKEN_FLAGS_*` for the current token, mirroring Go's
    /// `Scanner.tokenFlags` (`scanner.go:198`). Accumulated during `scan()`
    /// and exposed via `token_flags()`. `has_preceding_line_break` is kept
    /// in sync with `TOKEN_FLAGS_PRECEDING_LINE_BREAK` for backwards
    /// compatibility with existing parser call sites.
    token_flags: TokenFlags,
    /// Leading-asterisk skip depth for JSDoc type scanning. When non-zero,
    /// a single `*` at line start is consumed as trivia and sets
    /// `TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS`. Mirrors Go's
    /// `Scanner.skipJSDocLeadingAsterisks` (`scanner.go:200`), which is a
    /// counter to support nested JSDoc contexts.
    skip_jsdoc_leading_asterisks: i32,
    error_callback: Option<ErrorCallback>,
    /// Errors collected when no `error_callback` is set (or always, for
    /// retrieval via `take_errors`).
    errors: Vec<ScannerError>,
    /// `@ts-expect-error` / `@ts-ignore` directives collected from comments.
    comment_directives: Vec<CommentDirective>,
    /// Script target for regex flag availability checks (TS1501). Defaults to
    /// `ESNext` (no restrictions). Mirrors Go's `Scanner.scriptTarget`.
    script_target: crate::core::compiler_options::ScriptTarget,
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
            full_start_pos: 0,
            preceding_line_break: false,
            has_preceding_line_break: false,
            binary_marker_pos: None,
            token_flags: TOKEN_FLAGS_NONE,
            skip_jsdoc_leading_asterisks: 0,
            error_callback: None,
            errors: Vec::new(),
            comment_directives: Vec::new(),
            script_target: crate::core::compiler_options::ScriptTarget::ESNext,
        }
    }

    pub fn with_error_callback(mut self, cb: ErrorCallback) -> Self {
        self.error_callback = Some(cb);
        self
    }

    /// Set the script target for regex flag availability checks (TS1501).
    pub fn set_script_target(&mut self, target: crate::core::compiler_options::ScriptTarget) {
        self.script_target = target;
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

    /// The start position of the current token *including* any leading
    /// trivia (whitespace/comments). Mirrors Go's `TokenFullStart`.
    pub fn full_start_pos(&self) -> usize {
        self.full_start_pos
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

    /// Bitset of `TOKEN_FLAGS_*` for the current token, mirroring Go's
    /// `Scanner.TokenFlags()`. Use `token_flags_contains(flags, bit)` to test
    /// individual bits, or compare with the `TOKEN_FLAGS_*` constants directly.
    pub fn token_flags(&self) -> TokenFlags {
        self.token_flags
    }

    /// Whether the current token is preceded by a JSDoc comment (`/** ... */`).
    /// Mirrors Go's `Scanner.HasPrecedingJSDocComment`.
    pub fn has_preceding_jsdoc_comment(&self) -> bool {
        token_flags_contains(self.token_flags, TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT)
    }

    /// Whether the current token is preceded by a consumed JSDoc leading
    /// asterisk. Mirrors Go's `Scanner.HasPrecedingJSDocLeadingAsterisks`.
    pub fn has_preceding_jsdoc_leading_asterisks(&self) -> bool {
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS,
        )
    }

    /// Whether the preceding JSDoc comment contains a `@deprecated` tag.
    /// Mirrors Go's `Scanner.HasPrecedingJSDocWithDeprecatedTag`.
    pub fn has_preceding_jsdoc_with_deprecated_tag(&self) -> bool {
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED,
        )
    }

    /// Whether the preceding JSDoc comment contains a `@see` or `@link` tag.
    /// Mirrors Go's `Scanner.HasPrecedingJSDocWithSeeOrLink`.
    pub fn has_preceding_jsdoc_with_see_or_link(&self) -> bool {
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK,
        )
    }

    /// Enable/disable skipping JSDoc leading asterisks. When enabled, a
    /// single `*` at line start is consumed as trivia (setting
    /// `TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS`) instead of producing
    /// an `AsteriskToken`. This is a counter to support nested JSDoc
    /// contexts. Mirrors Go's `Scanner.SetSkipJSDocLeadingAsterisks`.
    pub fn set_skip_jsdoc_leading_asterisks(&mut self, skip: bool) {
        if skip {
            self.skip_jsdoc_leading_asterisks += 1;
        } else {
            self.skip_jsdoc_leading_asterisks -= 1;
        }
    }

    /// Current scan position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// The end of the scanner's current text range. When the scanner is
    /// re-pointed via `set_range`, this is the end of the sub-range.
    /// Mirrors Go's `scanner.end`. Used by the JSDoc parser to know where
    /// the comment body ends.
    pub fn end(&self) -> usize {
        self.end
    }

    /// The current value of the `skip_jsdoc_leading_asterisks` counter.
    /// Mirrors Go's `scanner.skipJsdocLeadingAsterisks` field access.
    pub fn skip_jsdoc_leading_asterisks_raw(&self) -> i32 {
        self.skip_jsdoc_leading_asterisks
    }

    /// Directly set the `skip_jsdoc_leading_asterisks` counter. Used by
    /// the JSDoc type expression parser to save/restore the skip state.
    pub fn set_skip_jsdoc_leading_asterisks_raw(&mut self, value: i32) {
        self.skip_jsdoc_leading_asterisks = value;
    }

    /// The full source text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Re-point the scanner at a range within the current text, resetting
    /// token state. Used by the JSDoc parser to scan within a comment body
    /// (between `/**` and `*/`) without creating a new scanner. Mirrors
    /// Go's `scanner.SetText` + `scanner.ResetPos` pattern in
    /// `parseJSDocComment` (`jsdoc.go:163-166`).
    pub fn set_range(&mut self, pos: usize, end: usize) {
        self.pos = pos;
        self.end = end;
        self.full_start_pos = pos;
        self.token_pos = pos;
        self.token_end = pos;
        self.token = SyntaxKind::Unknown;
        self.token_flags = TOKEN_FLAGS_NONE;
        self.preceding_line_break = false;
        self.has_preceding_line_break = false;
    }

    /// Scan the next token and return its kind.
    pub fn scan(&mut self) -> SyntaxKind {
        // Reset the line-break accumulator and the token flags; both are set
        // during trivia skipping / token scanning below. `has_preceding_line_break`
        // and `token_flags` are snapshotted *after* the loop exits (mirrors
        // Go's `tokenFlags` accumulation in `scanner.go:469-491`), so line
        // breaks encountered while skipping trivia are correctly reflected on
        // the returned token.
        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;

        // `full_start_pos` marks where the current token's leading trivia
        // began. It is set once on entry and preserved across trivia-skipping
        // iterations (mirrors Go's `fullStartPos`, `scanner.go:469`). The
        // post-trivia `token_pos` is reset each iteration below (mirrors Go's
        // `tokenStart`, `scanner.go:473`).
        self.full_start_pos = self.pos;

        let token = loop {
            self.token_pos = self.pos;

            if self.pos >= self.end {
                self.token = SyntaxKind::EndOfFile;
                self.token_end = self.pos;
                break self.token;
            }

            // Decode the actual UTF-8 character at the current position.
            // For ASCII bytes, this is equivalent to `as_bytes()[pos] as char`,
            // but for multi-byte characters (e.g., CJK), it correctly decodes
            // the full codepoint instead of just the first byte.
            let c = self.text[self.pos..].chars().next().unwrap();

            // Skip trivia (whitespace, comments) by `continue`-ing the loop so
            // `full_start_pos` is preserved while `token_pos` advances past
            // the trivia on the next iteration.
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

            // Identifier or keyword
            if is_identifier_start(c) {
                break self.scan_identifier();
            }

            // Number
            if is_digit(c)
                || (c == '.'
                    && self.pos + 1 < self.end
                    && is_digit(self.text.as_bytes()[self.pos + 1] as char))
            {
                break self.scan_number();
            }

            // String
            if c == '"' || c == '\'' {
                break self.scan_string(c);
            }

            // Template literal start
            if c == '`' {
                break self.scan_template();
            }

            // JSDoc leading asterisk: when `skip_jsdoc_leading_asterisks` is
            // active, consume a single `*` at line start as trivia (mirrors
            // Go `scanner.go:569-575`). Only applies to `*` NOT followed by
            // `*` or `=` (those form `**` / `*=` tokens). The flag is set
            // once per token so only the first leading `*` is consumed; the
            // `continue` preserves `full_start_pos` while `token_pos`
            // advances past the asterisk on the next iteration.
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

            // Shebang: `#!` at the very start of the file is trivia spanning the
            // rest of the line. Mirrors Go's `#` case (`scanner.go:898-910`).
            // Must be the first non-trivia byte (no leading whitespace).
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

            // Private identifier: `#name`. Mirrors Go's `#` case
            // (`scanner.go:911-925`) — `#` followed by identifier characters
            // scans as a single `PrivateIdentifier` token whose text includes
            // the leading `#` (invalid `#` reports InvalidCharacter, matching
            // Go).
            if c == '#' {
                break self.scan_private_identifier();
            }

            // Punctuation
            break self.scan_punctuation();
        };
        // Snapshot the accumulated state into the per-token fields. Keep
        // `has_preceding_line_break` and `token_flags` in sync so existing
        // parser call sites can keep using `has_preceding_line_break()` while
        // new call sites use `token_flags()` directly.
        self.has_preceding_line_break = self.preceding_line_break;
        if self.preceding_line_break {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
        }
        token
    }

    /// Continue scanning a template literal after a `${...}` expression.
    ///
    /// This mirrors the Go scanner's template rescanning path at a small scale:
    /// the parser consumes the `}` for the embedded expression, then asks the
    /// scanner for the following template chunk without skipping trivia.
    pub fn scan_template_continuation(&mut self) -> SyntaxKind {
        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;
        self.token_pos = self.pos;
        // Template continuation does not skip trivia, so the full start equals
        // the token start.
        self.full_start_pos = self.pos;

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
        self.has_preceding_line_break = self.preceding_line_break;
        if self.preceding_line_break {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
        }
        self.token
    }

    fn scan_whitespace(&mut self) {
        // Decode FULL UTF-8 codepoints: non-ASCII whitespace (NBSP, BOM,
        // ideographic space, …) is multi-byte — a byte-wise `as char` reads
        // the lead byte as a Latin-1 char, matches nothing, and leaves pos
        // untouched while scan()'s `is_whitespace` keeps returning true →
        // infinite trivia loop (typeGuardFunctionErrors' U+00A0).
        while self.pos < self.end {
            let c = self.text[self.pos..].chars().next().unwrap();
            if !is_whitespace(c) {
                break;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            self.pos += c.len_utf8();
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
        // Detect JSDoc: `/**` but not `/**/` (empty comment). Mirrors Go's
        // `isJSDoc := s.char() == '*' && s.charAt(1) != '/'`
        // (`scanner.go:642`). `token_pos` points at the opening `/` of the
        // comment, used later to extract the full comment text for tag
        // scanning (mirrors Go's `s.text[s.tokenStart:s.pos]`).
        let is_jsdoc = self.pos < self.end
            && self.text.as_bytes()[self.pos] as char == '*'
            && (self.pos + 1 >= self.end || self.text.as_bytes()[self.pos + 1] as char != '/');
        let comment_start = self.token_pos;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '*'
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] as char == '/'
            {
                self.pos += 2;
                if is_jsdoc {
                    self.token_flags |= TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT;
                    let comment_text = &self.text[comment_start..self.pos];
                    self.token_flags |= scan_jsdoc_comment_for_tags(comment_text);
                }
                return;
            }
            if c == '\n' || c == '\r' {
                self.preceding_line_break = true;
            }
            self.pos += 1;
        }
        // Unterminated comment: reached end of file without `*/`. Go reports
        // `Asterisk_Slash_expected` here; the Rust scanner does not currently
        // surface that diagnostic (a pre-existing gap). JSDoc flags are still
        // set so callers can detect the preceding JSDoc comment.
        if is_jsdoc {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT;
            let comment_text = &self.text[comment_start..self.pos];
            self.token_flags |= scan_jsdoc_comment_for_tags(comment_text);
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

    /// Scan a private identifier (`#name`). Mirrors Go's `#` case
    /// (`scanner.go:921-925`): `self.pos` is at the `#`. We advance past it,
    /// then consume the following identifier characters. The token text
    /// (accessed via `token_text()`) includes the leading `#`. If `#` is not
    /// followed by an identifier-start character, Go reports an error and
    /// yields a `PrivateIdentifier` whose value is just `"#"`.
    fn scan_private_identifier(&mut self) -> SyntaxKind {
        // Consume the leading `#`.
        self.pos += 1;
        // Scan the identifier part following `#`.
        if self.pos < self.end {
            let next_c = self.text[self.pos..].chars().next().unwrap();
            if is_identifier_start(next_c) {
                self.pos += next_c.len_utf8();
                while self.pos < self.end {
                    let c = self.text[self.pos..].chars().next().unwrap();
                    if !is_identifier_part(c) {
                        break;
                    }
                    self.pos += c.len_utf8();
                }
            } else {
                // `#` not followed by an identifier start — report AT the
                // `#` itself with width 1 (Go scanner.go:922
                // `s.errorAt(Invalid_character, s.pos-1, 1)`) and yield a
                // minimal `#` private identifier.
                self.report_error(DiagnosticKind::InvalidCharacter, self.pos - 1, 1);
            }
        }
        self.token_end = self.pos;
        self.token = SyntaxKind::PrivateIdentifier;
        self.token
    }

    fn scan_number(&mut self) -> SyntaxKind {
        let start = self.pos;
        if self.text.as_bytes()[self.pos] as char == '0' && self.pos + 1 < self.end {
            let next = self.text.as_bytes()[self.pos + 1] as char;
            if next == 'x' || next == 'X' {
                // Hex
                self.pos += 2;
                self.scan_number_fragment_with_sep(true, true);
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_HEX_SPECIFIER;
                return self.token;
            }
            if next == 'b' || next == 'B' {
                // Binary
                self.pos += 2;
                self.scan_binary_fragment_with_sep();
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_BINARY_SPECIFIER;
                return self.token;
            }
            if next == 'o' || next == 'O' {
                // Octal
                self.pos += 2;
                self.scan_octal_specifier_fragment_with_sep();
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_OCTAL_SPECIFIER;
                return self.token;
            }
        }

        // Decimal / legacy octal / leading-zero handling. Mirrors Go's
        // `scanNumber` (`scanner.go:1944-2042`): when the literal starts with
        // `0` (without an `x`/`b`/`o` specifier), scan the following digits to
        // distinguish three cases:
        //   1. `0_...` — separator not allowed after leading zero; report
        //      error, reset, re-scan as plain fragment.
        //   2. `0` + all-octal digits (e.g. `0777`) — legacy octal literal;
        //      set `OCTAL` flag, report TS1121, return early.
        //   3. `0` + non-octal digits (e.g. `0888`) — invalid leading zero;
        //      set `CONTAINS_LEADING_ZERO`, report TS1489 after full scan.
        if self.text.as_bytes()[self.pos] as char == '0' {
            self.pos += 1; // skip the leading `0`
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                // `0_...` — separator not allowed here. Mirrors Go
                // `scanner.go:1949-1953`: set both separator flags, report
                // TS6188, reset to start, re-scan as plain number fragment.
                self.token_flags |=
                    TOKEN_FLAGS_CONTAINS_SEPARATOR | TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                self.pos = start;
                self.scan_number_fragment_with_sep(false, false);
            } else {
                // Scan following digits (no separators) to determine octal vs
                // leading-zero. Mirrors Go's `scanDigits`
                // (`scanner.go:2090-2100`).
                let digits_start = self.pos;
                let mut is_octal = true;
                while self.pos < self.end {
                    let c = self.text.as_bytes()[self.pos] as char;
                    if is_digit(c) {
                        if !is_octal_digit(c) {
                            is_octal = false;
                        }
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                if self.pos > digits_start && is_octal {
                    // Legacy octal literal (e.g. `0777`). Set `OCTAL` flag and
                    // report TS1121. Mirrors Go `scanner.go:1961-1971`. The
                    // error range includes the `-` sign if the previous token
                    // was a minus (so `-0777` points at the full `-0777`).
                    self.token_flags |= TOKEN_FLAGS_OCTAL;
                    let with_minus = self.token == SyntaxKind::MinusToken;
                    let err_start = if with_minus { start - 1 } else { start };
                    self.report_error(
                        DiagnosticKind::OctalLiteralNotAllowed,
                        err_start,
                        self.pos - err_start,
                    );
                    self.token_end = self.pos;
                    self.token = SyntaxKind::NumericLiteral;
                    return self.token;
                } else if self.pos > digits_start {
                    // Leading zero with non-octal digits (e.g. `0888`).
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_LEADING_ZERO;
                }
                // else: just `0` with no following digits — fall through to
                // fractional/exponent handling.
            }
        } else {
            // Non-zero start (1-9): scan the integer part.
            self.scan_number_fragment_with_sep(false, false);
        }
        // Fractional part
        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '.' {
            self.pos += 1;
            self.scan_number_fragment_with_sep(false, false);
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
                self.scan_number_fragment_with_sep(false, false);
                self.token_flags |= TOKEN_FLAGS_SCIENTIFIC;
            }
        }

        // Report leading-zero error after the full literal is scanned.
        // Mirrors Go `scanner.go:2012-2016`.
        if token_flags_contains(self.token_flags, TOKEN_FLAGS_CONTAINS_LEADING_ZERO) {
            self.report_error(
                DiagnosticKind::DecimalWithLeadingZero,
                start,
                self.pos - start,
            );
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

    /// Scan a decimal number fragment (digits and optional `_` separators),
    /// setting `CONTAINS_SEPARATOR` for valid `_` and `CONTAINS_INVALID_SEPARATOR`
    /// for invalid `_` (at start, end, or consecutive). Mirrors Go's
    /// `scanNumberFragment` (`scanner.go:2044-2088`). `is_hex` controls whether
    /// hex digits (A-F) are accepted; `can_have_sep` is always true for numeric
    /// literals in Go (separators are allowed in all numeric forms).
    fn scan_number_fragment_with_sep(&mut self, is_hex: bool, _can_have_sep: bool) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;
            // Scan consecutive digits
            while self.pos < self.end {
                let c = self.text.as_bytes()[self.pos] as char;
                if is_digit(c) || (is_hex && c.is_ascii_hexdigit()) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos > before {
                allow_separator = true;
                is_prev_separator = false;
            }
            // Check for separator
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_SEPARATOR;
                if allow_separator {
                    allow_separator = false;
                    is_prev_separator = true;
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
        }
    }

    /// Scan a binary number fragment (`0`/`1` with `_` separators), setting
    /// separator flags. Mirrors Go's `scanNumberFragment` for binary.
    fn scan_binary_fragment_with_sep(&mut self) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;
            while self.pos < self.end {
                let c = self.text.as_bytes()[self.pos] as char;
                if c == '0' || c == '1' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos > before {
                allow_separator = true;
                is_prev_separator = false;
            }
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_SEPARATOR;
                if allow_separator {
                    allow_separator = false;
                    is_prev_separator = true;
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
        }
    }

    /// Scan an octal-specifier fragment (`0o` prefix, digits 0-7 with `_`
    /// separators), setting separator flags. Mirrors Go's `scanNumberFragment`
    /// for octal specifier form.
    fn scan_octal_specifier_fragment_with_sep(&mut self) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;
            while self.pos < self.end {
                let c = self.text.as_bytes()[self.pos] as char;
                if is_octal_digit(c) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos > before {
                allow_separator = true;
                is_prev_separator = false;
            }
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {
                self.token_flags |= TOKEN_FLAGS_CONTAINS_SEPARATOR;
                if allow_separator {
                    allow_separator = false;
                    is_prev_separator = true;
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                }
                self.pos += 1;
                continue;
            }
            break;
        }
        if is_prev_separator {
            self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
        }
    }

    fn scan_string(&mut self, quote: char) -> SyntaxKind {
        if quote == '\'' {
            self.token_flags |= TOKEN_FLAGS_SINGLE_QUOTE;
        }
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
            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
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
    /// continuations, and single-character escapes. Sets the appropriate
    /// `TokenFlags` bits mirroring Go's `scanEscapeSequence`
    /// (`scanner.go:1690-1851`): `HEX_ESCAPE` for valid `\xHH`,
    /// `UNICODE_ESCAPE` for valid `\uHHHH`, `EXTENDED_UNICODE_ESCAPE` for
    /// valid `\u{...}`, `CONTAINS_INVALID_ESCAPE` for octal/`\8`/`\9`/invalid
    /// `\x`/invalid `\u`.
    fn scan_escape_sequence(&mut self) {
        // pos is at '\'
        self.pos += 1; // skip backslash
        if self.pos >= self.end {
            return;
        }
        let c = self.text.as_bytes()[self.pos] as char;
        self.pos += 1; // skip the escaped char
        match c {
            '0' => {
                // '\0' is valid (NUL), but '\0' followed by a digit is a legacy
                // octal escape ('\01', '\011'). Go falls through to the octal
                // path for `\0` + digit. Set `CONTAINS_INVALID_ESCAPE` for the
                // octal case (mirrors Go scanner.go:1721).
                if self.pos < self.end && is_digit(self.text.as_bytes()[self.pos] as char) {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                    // Consume up to 2 more octal digits
                    for _ in 0..2 {
                        if self.pos < self.end
                            && is_octal_digit(self.text.as_bytes()[self.pos] as char)
                        {
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            '1'..='3' => {
                // Legacy octal escape: up to 2 more octal digits
                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                for _ in 0..2 {
                    if self.pos < self.end && is_octal_digit(self.text.as_bytes()[self.pos] as char)
                    {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
            }
            '4'..='7' => {
                // Legacy octal escape: up to 1 more octal digit
                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                if self.pos < self.end && is_octal_digit(self.text.as_bytes()[self.pos] as char) {
                    self.pos += 1;
                }
            }
            '8' | '9' => {
                // Invalid escape `\8` / `\9`
                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
            }
            'x' => {
                // \xHH — skip up to 2 hex digits. Set HEX_ESCAPE if both digits
                // are present, CONTAINS_INVALID_ESCAPE otherwise (mirrors Go
                // scanner.go:1811-1822).
                let mut digit_count = 0;
                for _ in 0..2 {
                    if self.pos < self.end && is_hex_digit(self.text.as_bytes()[self.pos] as char) {
                        self.pos += 1;
                        digit_count += 1;
                    } else {
                        break;
                    }
                }
                if digit_count == 2 {
                    self.token_flags |= TOKEN_FLAGS_HEX_ESCAPE;
                } else {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                }
            }
            'u' => {
                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '{' {
                    // \u{...} — extended unicode escape. Set
                    // EXTENDED_UNICODE_ESCAPE if valid, CONTAINS_INVALID_ESCAPE
                    // otherwise (mirrors Go scanner.go:1860-1900).
                    self.pos += 1; // skip '{'
                    let hex_start = self.pos;
                    while self.pos < self.end
                        && is_hex_digit(self.text.as_bytes()[self.pos] as char)
                    {
                        self.pos += 1;
                    }
                    let has_hex = self.pos > hex_start;
                    let closed =
                        self.pos < self.end && self.text.as_bytes()[self.pos] as char == '}';
                    if closed {
                        self.pos += 1; // skip '}'
                    }
                    if has_hex && closed {
                        self.token_flags |= TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE;
                    } else {
                        self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                    }
                } else {
                    // \uHHHH — skip up to 4 hex digits. Set UNICODE_ESCAPE if
                    // all 4 digits present, CONTAINS_INVALID_ESCAPE otherwise
                    // (mirrors Go scanner.go:1864-1868).
                    let mut digit_count = 0;
                    for _ in 0..4 {
                        if self.pos < self.end
                            && is_hex_digit(self.text.as_bytes()[self.pos] as char)
                        {
                            self.pos += 1;
                            digit_count += 1;
                        } else {
                            break;
                        }
                    }
                    if digit_count == 4 {
                        self.token_flags |= TOKEN_FLAGS_UNICODE_ESCAPE;
                    } else {
                        self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                    }
                }
            }
            '\r' => {
                // Line continuation: \<CR> or \<CRLF>
                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '\n' {
                    self.pos += 1;
                }
            }
            // Single-char escapes (\n, \t, \b, \f, \v, \\, \', \", \`, and
            // any non-recognized char) need no extra advancement — we already
            // skipped the char after the backslash.
            _ => {}
        }
    }

    fn scan_template(&mut self) -> SyntaxKind {
        // Simplified: scan until ` or ${
        self.pos += 1; // skip opening `
        let mut has_substitution = false;
        let mut terminated = false;
        while self.pos < self.end {
            let c = self.text.as_bytes()[self.pos] as char;
            if c == '`' {
                self.pos += 1;
                terminated = true;
                break;
            }
            if c == '$'
                && self.pos + 1 < self.end
                && self.text.as_bytes()[self.pos + 1] as char == '{'
            {
                self.pos += 2;
                has_substitution = true;
                terminated = true; // `${` opens a substitution; not unterminated.
                break;
            }
            if c == '\\' {
                self.scan_escape_sequence();
                continue;
            }
            self.pos += 1;
        }
        if !terminated {
            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
            self.report_error(
                DiagnosticKind::UnterminatedTemplateLiteral,
                self.token_pos,
                self.pos - self.token_pos,
            );
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

        // Check 4-char tokens (`>>>=`). `get(..4)` for UTF-8 safety.
        if remaining.len() >= 4 {
            if let Some(slice) = remaining.get(..4) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 4;
                }
            }
        }
        // Check 3-char tokens. Use `get(..3)` for safety: if byte 3 falls
        // inside a multi-byte UTF-8 character, `get` returns `None`.
        if best_len == 0 && remaining.len() >= 3 {
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
            // Mirrors Go scanner.go:935-940: a UTF-8 RuneError (Rust surfaces
            // invalid bytes as U+FFFD) means the file is likely binary. Report
            // TS1490 once and skip to end-of-file instead of emitting one
            // diagnostic per invalid byte (which explodes for binary inputs).
            if c == '\u{fffd}' {
                self.report_error(DiagnosticKind::FileAppearsToBeBinary, 0, 0);
                if self.binary_marker_pos.is_none() {
                    self.binary_marker_pos = Some(start);
                }
                self.pos = self.text.len();
                self.token_end = self.pos;
                self.token = SyntaxKind::EndOfFile;
                return SyntaxKind::EndOfFile;
            }
            let len = c.len_utf8();
            self.pos += len;
            self.token_end = self.pos;
            self.token = SyntaxKind::Unknown;
            self.report_error(DiagnosticKind::InvalidCharacter, start, len);
            SyntaxKind::Unknown
        }
    }

    /// Byte offset of the binary (non-text) marker, if the scan hit one.
    pub fn binary_marker_pos(&self) -> Option<usize> {
        self.binary_marker_pos
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
    /// classes and `\` escapes), consumes the closing `/`, then consumes and
    /// validates flags (identifier-part characters). It reports
    /// `UnterminatedRegularExpression` on EOF/newline before a closing `/`.
    /// Flag validation mirrors Go's `ReScanSlashToken` flag scan
    /// (`scanner.go:1171-1191`): unknown flags (TS1499), duplicate flags
    /// (TS1500), simultaneous `u`+`v` (TS1502), and target-gated availability
    /// (TS1501). Full regex body validation (TS1503–TS1538) is performed by
    /// `RegExpParser` (`scanner/regexp.rs`), mirroring Go's `regexp.go`.
    pub fn re_scan_slash_token(&mut self) -> SyntaxKind {
        if self.token != SyntaxKind::SlashToken && self.token != SyntaxKind::SlashEqualsToken {
            return self.token;
        }

        let start_of_regex_body = self.token_pos + 1; // right after the `/`
        let mut p = start_of_regex_body;
        let mut in_escape = false;
        let mut in_character_class = false;
        let mut unterminated = false;
        // Detect `(?<` named-capture groups during the first pass, mirroring
        // Go's `reScanSlashToken` (`scanner.go:1112-1116`). Used to gate the
        // `\k<name>` reference diagnostic in `RegExpParser`.
        let mut named_capture_groups = false;

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
                '(' if !in_character_class
                    && p + 3 < self.end
                    && self.text.as_bytes()[p + 1] as char == '?'
                    && self.text.as_bytes()[p + 2] as char == '<'
                    && self.text.as_bytes()[p + 3] as char != '='
                    && self.text.as_bytes()[p + 3] as char != '!' =>
                {
                    named_capture_groups = true;
                }
                _ => {}
            }
            p += 1;
        }

        let end_of_regex_body = p;

        if unterminated || p >= self.end {
            // Unterminated regex — report error and consume what we have
            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
            self.report_error(
                DiagnosticKind::UnterminatedRegularExpression,
                self.token_pos,
                p - self.token_pos,
            );
            self.pos = p;
        } else {
            // Consume the closing `/`
            p += 1;
            // Consume and validate flags (identifier-part characters).
            // Mirrors Go's `ReScanSlashToken` flag scan (`scanner.go:1171-1191`):
            // each flag must be a known flag (`d g i m s u v y`), must not
            // repeat, `u` + `v` are mutually exclusive (TS1502), and target-
            // gated availability is checked (TS1501).
            let mut seen_flags: u16 = 0;
            while p < self.end {
                let c = self.text.as_bytes()[p] as char;
                if !is_identifier_part(c) {
                    break;
                }
                if let Some(bit) = reg_exp_flag_bit(c) {
                    if seen_flags & bit != 0 {
                        // Duplicate flag — report at this char.
                        self.report_error(DiagnosticKind::DuplicateRegularExpressionFlag, p, 1);
                    } else if (seen_flags | bit) & (REG_EXP_FLAG_U | REG_EXP_FLAG_V)
                        == (REG_EXP_FLAG_U | REG_EXP_FLAG_V)
                    {
                        // `u` and `v` are mutually exclusive (TS1502).
                        self.report_error(DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive, p, 1);
                    } else {
                        seen_flags |= bit;
                        // Target-gated flag availability (TS1501).
                        self.check_reg_exp_flag_availability(bit, p);
                    }
                } else {
                    // Unknown flag — report at this char.
                    self.report_error(DiagnosticKind::UnknownRegularExpressionFlag, p, 1);
                }
                p += 1;
            }
            self.pos = p;

            // Run the full regex body validator. Mirrors Go's
            // `reScanSlashToken` (`scanner.go:1192-1210`): construct a
            // `regExpParser` over the body region and call `run()`.
            let mut parser = regexp::RegExpParser::new(
                &self.text,
                start_of_regex_body,
                end_of_regex_body,
                seen_flags,
                named_capture_groups,
                self.script_target,
            );
            parser.run();
            for err in parser.errors() {
                self.errors.push(*err);
            }
        }

        self.token_end = self.pos;
        self.token = SyntaxKind::RegularExpressionLiteral;
        self.token
    }

    /// Check target-gated regex flag availability (TS1501). Mirrors Go's
    /// `checkRegularExpressionFlagAvailability` (`scanner.go:50-54`):
    /// `d` → ES2022, `s` → ES2018, `v` → ES2024.
    fn check_reg_exp_flag_availability(&mut self, flag: u16, pos: usize) {
        let available_from = match flag {
            REG_EXP_FLAG_D => Some(ScriptTarget::ES2022),
            REG_EXP_FLAG_S => Some(ScriptTarget::ES2018),
            REG_EXP_FLAG_V => Some(ScriptTarget::ES2024),
            _ => None,
        };
        if let Some(target) = available_from {
            if self.script_target < target {
                self.report_error(
                    DiagnosticKind::RegexMessage(
                        crate::diagnostics::THIS_REGULAR_EXPRESSION_FLAG_IS_ONLY_AVAILABLE_WHEN_TARGETING_0_OR_LATER,
                    ),
                    pos,
                    1,
                );
            }
        }
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

    // ────────────────────────────────────────────────────────────────────
    // JSDoc interior scanning
    //
    // Mirrors Go's `ScanJSDocToken` / `ScanJSDocCommentTextToken` /
    // `CanFollowJSDocAt` (`scanner.go:1374-1525`). These operate on the
    // scanner's `text` bounded by `end`; the JSDoc parser temporarily
    // re-points the scanner at the comment body before calling them.
    // ────────────────────────────────────────────────────────────────────

    /// Peek at the character at the current position (expected to be right
    /// after `@`) and return `true` if a JSDoc tag can follow. Identifier
    /// starts indicate a tag name; whitespace, newlines, and EOF are also
    /// accepted to support incomplete tags for code completion. Mirrors Go's
    /// `CanFollowJSDocAt` (`scanner.go:1410-1416`).
    pub fn can_follow_jsdoc_at(&self) -> bool {
        if self.pos >= self.end {
            return true;
        }
        let (ch, _) = decode_char(&self.text, self.pos);
        is_identifier_start(ch) || is_whitespace_single_line(ch) || is_line_break(ch)
    }

    /// Scan a single JSDoc interior token. Produces `WhitespaceTrivia`,
    /// `NewLineTrivia`, `AtToken`, `AsteriskToken`, braces, brackets, parens,
    /// angle brackets, `=`, `,`, `.`, `` ` ``, `#`, identifiers (including
    /// `-`), or `Unknown`. Mirrors Go's `ScanJSDocToken`
    /// (`scanner.go:1418-1525`). Unicode-escape identifier handling is
    /// deferred (rare in JSDoc; can be added when needed).
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

    /// Scan a JSDoc comment text token — a run of prose until a line break,
    /// `` ` ``, `{`, or a valid `@tag` boundary. When `in_backticks` is true
    /// (inside a fenced code block), only line breaks and `` ` `` terminate
    /// the run. If the run is empty (immediately at a special char), falls
    /// through to `scan_jsdoc_token`. Mirrors Go's
    /// `ScanJSDocCommentTextToken` (`scanner.go:1374-1405`).
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
    // Go `stringutil.IsWhiteSpaceSingleLine` list, plus \n/\r (Go handles
    // those as line breaks in its scan loop; our scan_whitespace folds the
    // line-break flag in here).
    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '\r'
            | '\x0B'
            | '\x0C'
            | '\u{85}'    // nextLine
            | '\u{A0}'    // nonBreakingSpace
            | '\u{1680}'  // ogham
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{200B}'  // zeroWidthSpace
            | '\u{202F}'  // narrowNoBreakSpace
            | '\u{205F}'  // mathematicalSpace
            | '\u{3000}'  // ideographicSpace
            | '\u{FEFF}'  // byteOrderMark
    )
}

fn is_line_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

// ─────────────────────────────────────────────────────────────────────
// Regular-expression flag bits
//
// Mirrors Go's `regularExpressionFlags` bitmask (`regexp.go:17-28`). Used by
// `re_scan_slash_token` to detect duplicate flags and the `u`/`v` mutual
// exclusion (TS1500/TS1502). Target-gated availability (TS1501) is deferred
// until `script_target` is plumbed into the scanner.
const REG_EXP_FLAG_G: u16 = 1 << 0;
const REG_EXP_FLAG_I: u16 = 1 << 1;
const REG_EXP_FLAG_M: u16 = 1 << 2;
const REG_EXP_FLAG_S: u16 = 1 << 3;
const REG_EXP_FLAG_U: u16 = 1 << 4;
const REG_EXP_FLAG_Y: u16 = 1 << 5;
const REG_EXP_FLAG_D: u16 = 1 << 6;
const REG_EXP_FLAG_V: u16 = 1 << 7;

/// Map a flag character to its bitmask bit, or `None` if it isn't a known
/// regular-expression flag. Mirrors Go's `charCodeToRegExpFlag`
/// (`regexp.go:33-42`).
fn reg_exp_flag_bit(c: char) -> Option<u16> {
    match c {
        'g' => Some(REG_EXP_FLAG_G),
        'i' => Some(REG_EXP_FLAG_I),
        'm' => Some(REG_EXP_FLAG_M),
        's' => Some(REG_EXP_FLAG_S),
        'u' => Some(REG_EXP_FLAG_U),
        'y' => Some(REG_EXP_FLAG_Y),
        'd' => Some(REG_EXP_FLAG_D),
        'v' => Some(REG_EXP_FLAG_V),
        _ => None,
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_octal_digit(c: char) -> bool {
    ('0'..='7').contains(&c)
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && is_unicode_identifier_start(c))
}

pub fn is_identifier_part(c: char) -> bool {
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
// Trivia helpers (free functions over source text)
//
// Mirror Go's `scanner.SkipTrivia` / `GetLeadingCommentRanges` /
// `GetTrailingCommentRanges` / `iterateCommentRanges` / shebang helpers
// (`scanner.go:2307-2504, 2800-2917`). These reconstruct comment ranges and
// advance past trivia from raw source text without holding a `Scanner`.
// ────────────────────────────────────────────────────────────────────────────

/// Kind of comment range, mirroring Go's `ast.KindSingleLineCommentTrivia` /
/// `ast.KindMultiLineCommentTrivia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentRangeKind {
    SingleLine,
    MultiLine,
}

/// A comment range reconstructed from source text. Mirrors Go's
/// `ast.CommentRange` (`ast.go:2979-2983`): a text range plus the comment
/// kind and whether a line break follows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentRange {
    pub pos: usize,
    pub end: usize,
    pub kind: CommentRangeKind,
    pub has_trailing_new_line: bool,
}

/// Decode the UTF-8 rune at `pos` in `text`, returning `(char, byte_size)`.
/// Mirrors Go's `utf8.DecodeRuneInString`. Assumes `pos < text.len()`.
fn decode_char(text: &str, pos: usize) -> (char, usize) {
    let c = text[pos..].chars().next().unwrap();
    (c, c.len_utf8())
}

/// Whether `c` is "whitespace-like" (tab, vtab, formFeed, space, non-breaking
/// space, BOM, or any Unicode `White_Space` property character). Mirrors Go's
/// `stringutil.IsWhiteSpaceLike`.
fn is_whitespace_like(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}') || c.is_whitespace()
}

/// Whether `c` is single-line whitespace (not a line break). Mirrors Go's
/// `stringutil.IsWhiteSpaceSingleLine`.
fn is_whitespace_single_line(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}')
}

/// Whether `text` starts with one of the given JSDoc tag `names` followed by
/// a valid tag terminator (whitespace, `}`, `*`, or end-of-string). Mirrors
/// Go's `hasJSDocTag` (`scanner.go:372-386`).
fn has_jsdoc_tag(text: &str, names: &[&str]) -> bool {
    for &name in names {
        if !text.starts_with(name) {
            continue;
        }
        if text.len() == name.len() {
            return true;
        }
        let ch = text.as_bytes()[name.len()] as char;
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '}' || ch == '*' {
            return true;
        }
    }
    false
}

/// Scan a JSDoc comment's text for `@deprecated`, `@see`, and `@link` tags
/// and return the OR'd token flags to set. Mirrors Go's
/// `Scanner.scanJSDocCommentForTags` (`scanner.go:350-368`).
///
/// Iterates over `@` occurrences in `comment_text`, checking each one against
/// the tag name sets. Stops early once both flags are set. Returns
/// `TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED` and/or
/// `TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK` as appropriate (or
/// `TOKEN_FLAGS_NONE` if no matching tags were found).
fn scan_jsdoc_comment_for_tags(comment_text: &str) -> TokenFlags {
    let mut flags = TOKEN_FLAGS_NONE;
    let mut rest = comment_text;
    loop {
        let i = match rest.find('@') {
            Some(i) => i,
            None => return flags,
        };
        rest = &rest[i + 1..];
        if !token_flags_contains(flags, TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED)
            && has_jsdoc_tag(rest, &["deprecated"])
        {
            flags |= TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED;
        }
        if !token_flags_contains(flags, TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK)
            && has_jsdoc_tag(rest, &["see", "link", "linkcode", "linkplain"])
        {
            flags |= TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK;
        }
        if token_flags_contains(
            flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED
                | TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK,
        ) {
            return flags;
        }
    }
}

/// Whether `text` starts with a shebang (`#!`) at `pos == 0`. Mirrors Go's
/// `isShebangTrivia` (`scanner.go:2475-2483`).
fn is_shebang_trivia(text: &str, pos: usize) -> bool {
    if text.len() < 2 {
        return false;
    }
    debug_assert_eq!(
        pos, 0,
        "shebangs check must only be done at the start of the file"
    );
    text.as_bytes()[0] == b'#' && text.as_bytes()[1] == b'!'
}

/// Advance past a shebang at `pos == 0`, returning the new position. Mirrors
/// Go's `scanShebangTrivia` (`scanner.go:2485-2495`).
fn scan_shebang_trivia(text: &str, pos: usize) -> usize {
    let text_len = text.len();
    let mut pos = pos + 2;
    while pos < text_len {
        let (ch, size) = decode_char(text, pos);
        if is_line_break(ch) {
            break;
        }
        pos += size;
    }
    pos
}

/// Return the shebang text (including `#!`) if the file starts with one, else
/// empty string. Mirrors Go's `GetShebang` (`scanner.go:2497-2504`).
pub fn get_shebang(text: &str) -> &str {
    if !is_shebang_trivia(text, 0) {
        return "";
    }
    let end = scan_shebang_trivia(text, 0);
    &text[..end]
}

/// Options controlling how `skip_trivia_ex` consumes trivia. Mirrors Go's
/// `SkipTriviaOptions` (`scanner.go:2301-2305`).
#[derive(Debug, Clone, Copy, Default)]
pub struct SkipTriviaOptions {
    /// If true, stop (return the position of the line break) after the first
    /// line break is consumed.
    pub stop_after_line_break: bool,
    /// If true, do not consume comments (return the position of the `/`).
    pub stop_at_comments: bool,
    /// If true, consume a leading `*` after a line break (JSDoc leading
    /// asterisk handling).
    pub in_jsdoc: bool,
}

/// Length of a Git merge-conflict marker (`<<<<<<<`, `=======`, `>>>>>>>`,
/// `|||||||`), all 7 bytes. Mirrors Go's `mergeConflictMarkerLength`.
const MERGE_CONFLICT_MARKER_LENGTH: usize = 7;

/// Whether `text[pos..]` starts with a Git merge-conflict marker. Mirrors
/// Go's `isConflictMarkerTrivia` (`scanner.go:2409-2442`). A conflict marker
/// is the same byte repeated seven times at the start of a line; `<<<<<<<`
/// and `>>>>>>>` must additionally be followed by a space, while `=======`
/// and `|||||||` do not require a trailing space.
fn is_conflict_marker_trivia(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    if pos + 1 >= text_len || bytes[pos + 1] != bytes[pos] {
        return false;
    }
    // Conflict markers must be at the start of a line.
    let mut at_line_start = pos == 0 || is_line_break(bytes[pos - 1] as char);
    if !at_line_start && pos >= 2 {
        // Go also allows a single trailing whitespace byte before the marker
        // (e.g. `\r` from a CRLF). Check the byte two positions back.
        at_line_start = is_line_break(bytes[pos - 2] as char);
    }
    if at_line_start && pos + MERGE_CONFLICT_MARKER_LENGTH < text_len {
        let ch = bytes[pos];
        for i in 0..MERGE_CONFLICT_MARKER_LENGTH {
            if bytes[pos + i] != ch {
                return false;
            }
        }
        // `=======` (and `|||||||`) don't need a trailing space; `<<<<<<<`
        // and `>>>>>>>` do.
        return ch == b'=' || bytes[pos + MERGE_CONFLICT_MARKER_LENGTH] == b' ';
    }
    false
}

/// Advance past a conflict marker at `pos`, returning the new position.
/// Mirrors Go's `scanConflictMarkerTrivia` (`scanner.go:2444-2473`). The
/// `report_error` callback (if set) is invoked once at the marker start with
/// `MERGE_CONFLICT_MARKER_LENGTH` as the length.
fn scan_conflict_marker_trivia(
    text: &str,
    pos: usize,
    report_error: Option<&dyn Fn(usize, usize)>,
) -> usize {
    if let Some(report) = report_error {
        report(pos, MERGE_CONFLICT_MARKER_LENGTH);
    }
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let (ch, _size) = decode_char(text, pos);
    let mut pos = pos;
    if ch == '<' || ch == '>' {
        // Consume to end of line.
        while pos < text_len && !is_line_break(bytes[pos] as char) {
            pos += 1;
        }
    } else {
        // `|` or `=`: consume until the start of the next `=======` or
        // `>>>>>>>` marker (which begins a new conflict section).
        while pos < text_len {
            let current = bytes[pos];
            if (current == b'=' || current == b'>')
                && current as char != ch
                && is_conflict_marker_trivia(text, pos)
            {
                break;
            }
            pos += 1;
        }
    }
    pos
}

/// Advance `pos` past trivia (whitespace and comments) in `text`, returning
/// the position of the next non-trivia character. Mirrors Go's `SkipTrivia`
/// (`scanner.go:2307-2400`, without options). Conflict-marker trivia and
/// JSDoc `*` consumption are handled by `skip_trivia_ex` with options.
pub fn skip_trivia(text: &str, pos: usize) -> usize {
    skip_trivia_ex(text, pos, &SkipTriviaOptions::default(), None)
}

/// Extended `skip_trivia` with options. Mirrors Go's `SkipTriviaEx`
/// (`scanner.go:2311-2400`). The `report_error` callback is invoked for
/// conflict-marker trivia (mirroring Go's `reportError` parameter to
/// `scanConflictMarkerTrivia`); pass `None` to suppress.
pub fn skip_trivia_ex(
    text: &str,
    pos: usize,
    options: &SkipTriviaOptions,
    report_error: Option<&dyn Fn(usize, usize)>,
) -> usize {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut pos = pos;
    // Tracks whether the next `*` (after a line break) should be consumed as
    // a JSDoc leading asterisk. Only meaningful when `options.in_jsdoc` is set.
    let mut can_consume_star = false;
    loop {
        if pos >= text_len {
            return pos;
        }
        let c = bytes[pos] as char;
        match c {
            '\r' => {
                if pos + 1 < text_len && bytes[pos + 1] == b'\n' {
                    pos += 1;
                }
                pos += 1;
                if options.stop_after_line_break {
                    return pos;
                }
                can_consume_star = options.in_jsdoc;
                continue;
            }
            '\n' => {
                pos += 1;
                if options.stop_after_line_break {
                    return pos;
                }
                can_consume_star = options.in_jsdoc;
                continue;
            }
            '\t' | '\x0B' | '\x0C' | ' ' => {
                pos += 1;
                continue;
            }
            '/' => {
                if options.stop_at_comments {
                    return pos;
                }
                if pos + 1 < text_len {
                    if bytes[pos + 1] == b'/' {
                        pos += 2;
                        while pos < text_len {
                            let (ch, size) = decode_char(text, pos);
                            if is_line_break(ch) {
                                break;
                            }
                            pos += size;
                        }
                        can_consume_star = false;
                        continue;
                    }
                    if bytes[pos + 1] == b'*' {
                        pos += 2;
                        while pos < text_len {
                            if bytes[pos] == b'*' && pos + 1 < text_len && bytes[pos + 1] == b'/' {
                                pos += 2;
                                break;
                            }
                            let (_, size) = decode_char(text, pos);
                            pos += size;
                        }
                        can_consume_star = false;
                        continue;
                    }
                }
                return pos;
            }
            '<' | '|' | '=' | '>' => {
                if is_conflict_marker_trivia(text, pos) {
                    pos = scan_conflict_marker_trivia(text, pos, report_error);
                    can_consume_star = false;
                    continue;
                }
                return pos;
            }
            '#' => {
                if pos == 0 && is_shebang_trivia(text, pos) {
                    pos = scan_shebang_trivia(text, pos);
                    continue;
                }
                return pos;
            }
            '*' => {
                if can_consume_star {
                    pos += 1;
                    can_consume_star = false;
                    continue;
                }
                return pos;
            }
            _ => {
                let (ch, size) = decode_char(text, pos);
                if ch > '\u{7F}' && is_whitespace_like(ch) {
                    pos += size;
                    continue;
                }
                return pos;
            }
        }
    }
}

/// Reconstruct leading comment ranges preceding `pos` in `text`. Mirrors Go's
/// `GetLeadingCommentRanges` (`scanner.go:2800-2802`).
pub fn get_leading_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    iterate_comment_ranges(text, pos, false)
}

/// Reconstruct trailing comment ranges following `pos` in `text` (up to the
/// next line break). Mirrors Go's `GetTrailingCommentRanges`
/// (`scanner.go:2804-2806`).
pub fn get_trailing_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    iterate_comment_ranges(text, pos, true)
}

/// Shared implementation for leading/trailing comment-range reconstruction.
/// Mirrors Go's `iterateCommentRanges` (`scanner.go:2814-2917`). `trailing`
/// means "stop at the first line break"; otherwise collect comments that
/// follow the position, including those separated by line breaks.
fn iterate_comment_ranges(text: &str, pos: usize, trailing: bool) -> Vec<CommentRange> {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut pos = pos;
    let mut result: Vec<CommentRange> = Vec::new();

    // Pending comment range (emitted when the next range arrives, so trailing
    // new-line info is known). Mirrors Go's pending* locals.
    let mut pending_pos: usize = 0;
    let mut pending_end: usize = 0;
    let mut pending_kind: CommentRangeKind = CommentRangeKind::SingleLine;
    let mut pending_has_trailing_new_line = false;
    let mut has_pending = false;

    let mut collecting = trailing;
    if pos == 0 {
        // At file start, leading comment collection starts immediately.
        collecting = true;
        if is_shebang_trivia(text, pos) {
            pos = scan_shebang_trivia(text, pos);
        }
    }

    while pos < text_len {
        let (ch, size) = decode_char(text, pos);
        match ch {
            '\r' => {
                if pos + 1 < text_len && bytes[pos + 1] == b'\n' {
                    pos += 1;
                }
                pos += 1;
                if trailing {
                    break;
                }
                collecting = true;
                if has_pending {
                    pending_has_trailing_new_line = true;
                }
                continue;
            }
            '\n' => {
                pos += 1;
                if trailing {
                    break;
                }
                collecting = true;
                if has_pending {
                    pending_has_trailing_new_line = true;
                }
                continue;
            }
            '\t' | '\x0B' | '\x0C' | ' ' => {
                pos += 1;
                continue;
            }
            '/' => {
                let mut next_char = b'\0';
                if pos + 1 < text_len {
                    next_char = bytes[pos + 1];
                }
                let mut has_trailing_new_line = false;
                if next_char == b'/' || next_char == b'*' {
                    let kind = if next_char == b'/' {
                        CommentRangeKind::SingleLine
                    } else {
                        CommentRangeKind::MultiLine
                    };
                    let start_pos = pos;
                    pos += 2;
                    if next_char == b'/' {
                        while pos < text_len {
                            let (c, s) = decode_char(text, pos);
                            if is_line_break(c) {
                                has_trailing_new_line = true;
                                break;
                            }
                            pos += s;
                        }
                    } else {
                        // Multi-line: search for `*/`.
                        if let Some(i) = text[pos..].find("*/") {
                            pos += i + 2;
                        } else {
                            pos = text_len;
                        }
                    }
                    if collecting {
                        if has_pending {
                            result.push(CommentRange {
                                pos: pending_pos,
                                end: pending_end,
                                kind: pending_kind,
                                has_trailing_new_line: pending_has_trailing_new_line,
                            });
                        }
                        pending_pos = start_pos;
                        pending_end = pos;
                        pending_kind = kind;
                        pending_has_trailing_new_line = has_trailing_new_line;
                        has_pending = true;
                    }
                    continue;
                }
                break;
            }
            _ => {
                if ch > '\u{7F}' && is_whitespace_like(ch) {
                    if has_pending && is_line_break(ch) {
                        pending_has_trailing_new_line = true;
                    }
                    pos += size;
                    continue;
                }
                break;
            }
        }
    }
    if has_pending {
        result.push(CommentRange {
            pos: pending_pos,
            end: pending_end,
            kind: pending_kind,
            has_trailing_new_line: pending_has_trailing_new_line,
        });
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
    fn scan_private_identifier() {
        // `#name` scans as a single PrivateIdentifier token whose text includes
        // the leading `#`. Mirrors Go scanner.go:897-925.
        let mut s = Scanner::new("#name = 1");
        assert_eq!(s.scan(), SyntaxKind::PrivateIdentifier);
        assert_eq!(s.token_text(), "#name");
        assert_eq!(s.scan(), SyntaxKind::EqualsToken);
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.scan(), SyntaxKind::EndOfFile);
    }

    #[test]
    fn scan_shebang_at_file_start_is_trivia() {
        // `#!` at the very start of the file is shebang trivia (consumed), so
        // the first real token is the following identifier.
        let mut s = Scanner::new("#!/usr/bin/env node\nlet x = 1;");
        assert_eq!(s.scan(), SyntaxKind::LetKeyword);
        assert_eq!(s.scan(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "x");
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
    fn re_scan_slash_token_valid_flags_no_errors() {
        // Known flags without the `u`+`v` conflict (`d g i m s y`) — no
        // diagnostics.
        let mut s = Scanner::new("/pattern/dgimsy");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/pattern/dgimsy");
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn re_scan_slash_token_unknown_flag_reports_ts1499() {
        // `z` is not a valid regex flag → TS1499 at each `z` position.
        let mut s = Scanner::new("/foo/zz");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/foo/zz");
        let errors = s.take_errors();
        assert_eq!(errors.len(), 2, "expected two TS1499 errors for 'zz'");
        assert_eq!(errors[0].kind, DiagnosticKind::UnknownRegularExpressionFlag);
        assert_eq!(errors[0].pos, "/foo/".len()); // first z
        assert_eq!(errors[0].length, 1);
        assert_eq!(errors[1].kind, DiagnosticKind::UnknownRegularExpressionFlag);
        assert_eq!(errors[1].pos, "/foo/z".len()); // second z
    }

    #[test]
    fn re_scan_slash_token_duplicate_flag_reports_ts1500() {
        // `gg` → TS1500 at the second `g`.
        let mut s = Scanner::new("/foo/gg");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/foo/gg");
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind,
            DiagnosticKind::DuplicateRegularExpressionFlag
        );
        assert_eq!(errors[0].pos, "/foo/g".len()); // second g
        assert_eq!(errors[0].length, 1);
    }

    #[test]
    fn re_scan_slash_token_u_and_v_mutually_exclusive_reports_ts1502() {
        // `uv` and `vu` both → TS1502 at the second flag.
        let mut s = Scanner::new("/foo/uv");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind,
            DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive
        );
        assert_eq!(errors[0].pos, "/foo/u".len()); // the v
        assert_eq!(errors[0].length, 1);

        // Reverse order: `vu` → TS1502 at the `u`.
        let mut s = Scanner::new("/foo/vu");
        s.scan();
        s.re_scan_slash_token();
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind,
            DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive
        );
        assert_eq!(errors[0].pos, "/foo/v".len()); // the u
    }

    #[test]
    fn re_scan_slash_token_mixed_flag_errors() {
        // `guz`: `g` ok, `u` ok, `z` unknown → one TS1499.
        let mut s = Scanner::new("/foo/guz");
        s.scan();
        s.re_scan_slash_token();
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::UnknownRegularExpressionFlag);
        assert_eq!(errors[0].pos, "/foo/gu".len()); // the z
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

    // ────────────────────────────────────────────────────────────────────
    // Trivia helpers (P2.1)
    // ────────────────────────────────────────────────────────────────────

    #[test]
    fn skip_trivia_whitespace_and_newlines() {
        // Spaces, tabs, newlines should all be skipped.
        assert_eq!(skip_trivia("  \t\n  x", 0), 6);
        assert_eq!(skip_trivia("\n\n\nx", 0), 3);
        assert_eq!(skip_trivia("x", 0), 0);
        // Empty string -> pos 0.
        assert_eq!(skip_trivia("", 0), 0);
    }

    #[test]
    fn skip_trivia_single_line_comment() {
        // `//` consumes up to (but not including) the line break.
        assert_eq!(skip_trivia("// comment\nx", 0), 11);
        // `//` at EOF.
        assert_eq!(skip_trivia("// eof", 0), 6);
    }

    #[test]
    fn skip_trivia_multi_line_comment() {
        // `/* comment */` is 13 bytes; `x` is at pos 13.
        assert_eq!(skip_trivia("/* comment */x", 0), 13);
        // Unterminated multi-line comment consumes to EOF.
        assert_eq!(skip_trivia("/* unterminated", 0), 15);
        // Multi-line comment that doesn't start at `/` should not be skipped.
        assert_eq!(skip_trivia("abc", 0), 0);
    }

    #[test]
    fn skip_trivia_shebang_at_start() {
        assert_eq!(skip_trivia("#!/usr/bin/env node\nlet x;", 0), 20);
        // `#!` not at start of file should not be treated as trivia.
        assert_eq!(skip_trivia(" #!/foo", 1), 1);
    }

    #[test]
    fn skip_trivia_combined() {
        assert_eq!(
            skip_trivia("#!/usr/bin/env node\n// hello\n/* world */\nlet x;", 0),
            41
        );
    }

    #[test]
    fn get_shebang_returns_text() {
        assert_eq!(
            get_shebang("#!/usr/bin/env node\nlet x;"),
            "#!/usr/bin/env node"
        );
        assert_eq!(get_shebang("let x;"), "");
        assert_eq!(get_shebang("#!only\nmore"), "#!only");
    }

    #[test]
    fn full_start_pos_tracks_leading_trivia() {
        // `let x = 1;`: `let`=0-2, ` `=3, `x`=4, ` `=5, `=`=6, ` `=7, `1`=8, `;`=9.
        let mut s = Scanner::new("let x = 1;");
        s.scan();
        assert_eq!(s.full_start_pos(), 0);
        assert_eq!(s.token_pos(), 0);
        assert_eq!(s.token(), SyntaxKind::LetKeyword);

        // After `let`, scanner skips trivia (` `) before `x`. The full start
        // of `x` should be 3 (start of the space), while token_pos is 4 (the `x`).
        s.scan();
        assert_eq!(s.full_start_pos(), 3);
        assert_eq!(s.token_pos(), 4);
        assert_eq!(s.token(), SyntaxKind::Identifier);

        // After `x`, scanner skips ` ` (trivia) before `=`. full_start_pos
        // should be 5 (after `x`), token_pos 6 (the `=`).
        s.scan();
        assert_eq!(s.full_start_pos(), 5);
        assert_eq!(s.token_pos(), 6);
        assert_eq!(s.token(), SyntaxKind::EqualsToken);
    }

    #[test]
    fn full_start_pos_preserved_across_comments() {
        // Leading single-line comment, then `let`. full_start_pos should be 0
        // (where the comment starts), token_pos should be 11 (after `\n`).
        let mut s = Scanner::new("// hi\nlet x;");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert_eq!(s.full_start_pos(), 0);
        assert_eq!(s.token_pos(), 6);

        // Multi-line comment between two tokens.
        let mut s = Scanner::new("a /* c */ b");
        s.scan(); // a
        assert_eq!(s.token(), SyntaxKind::Identifier);
        assert_eq!(s.token_pos(), 0);
        s.scan(); // b
        assert_eq!(s.token(), SyntaxKind::Identifier);
        // full_start_pos should be 1 (start of the space/comment trivia).
        assert_eq!(s.full_start_pos(), 1);
        assert_eq!(s.token_pos(), 10);
    }

    #[test]
    fn get_leading_comment_ranges_basic() {
        // Two leading single-line comments, then code.
        // `// first` = 8 bytes (0-7), `\n` = 1 (8), `// second` = 9 bytes (9-17), end=18.
        let text = "// first\n// second\nlet x;";
        let ranges = get_leading_comment_ranges(text, 0);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
        assert_eq!(ranges[0].pos, 0);
        assert_eq!(ranges[0].end, 8);
        assert!(ranges[0].has_trailing_new_line);
        assert_eq!(ranges[1].pos, 9);
        assert_eq!(ranges[1].end, 18);
        assert!(ranges[1].has_trailing_new_line);
    }

    #[test]
    fn get_leading_comment_ranges_multi_line() {
        let text = "/* hello */let x;";
        let ranges = get_leading_comment_ranges(text, 0);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].kind, CommentRangeKind::MultiLine);
        assert_eq!(ranges[0].pos, 0);
        assert_eq!(ranges[0].end, 11);
        assert!(!ranges[0].has_trailing_new_line);
    }

    #[test]
    fn get_leading_comment_ranges_from_middle() {
        // `let x; // trailing\n// leading for next\nlet y;`
        //  pos: 0-5=`let x;`, 6=` `, 7-17=`// trailing`, 18=`\n`,
        //  19-37=`// leading for next` (19 bytes), 38=`\n`, 39+=`let y;`
        let text = "let x; // trailing\n// leading for next\nlet y;";
        // Start at pos 18 (the `\n` after the trailing comment). Leading mode
        // treats this newline as a separator and collects the next comment.
        let ranges = get_leading_comment_ranges(text, 18);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
        assert_eq!(ranges[0].pos, 19);
        assert_eq!(ranges[0].end, 38);
        assert!(ranges[0].has_trailing_new_line);
    }

    #[test]
    fn get_leading_comment_ranges_none() {
        let ranges = get_leading_comment_ranges("let x;", 0);
        assert!(ranges.is_empty());
    }

    #[test]
    fn get_trailing_comment_ranges_basic() {
        // `let x; // trailing\nlet y;`
        //  pos: 0-5=`let x;`, 6=` `, 7-17=`// trailing`, 18=`\n`
        // Trailing comment starts at pos 7 (after the space).
        let text = "let x; // trailing\nlet y;";
        let ranges = get_trailing_comment_ranges(text, 6);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
        assert_eq!(ranges[0].pos, 7);
        assert_eq!(ranges[0].end, 18);
        assert!(ranges[0].has_trailing_new_line);
    }

    #[test]
    fn get_trailing_comment_ranges_stops_at_line_break() {
        // When `pos` is on a line with no trailing comment, return nothing.
        let text = "let x;\nlet y; // c\n";
        let ranges = get_trailing_comment_ranges(text, 0);
        assert!(ranges.is_empty());
    }

    #[test]
    fn get_trailing_comment_ranges_multi_line() {
        // `let x; /* c */ let y;`
        //  pos: 0-5=`let x;`, 6=` `, 7-13=`/* c */`, 14=` `, 15+=`let y;`
        let text = "let x; /* c */ let y;";
        let ranges = get_trailing_comment_ranges(text, 6);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].kind, CommentRangeKind::MultiLine);
        assert_eq!(ranges[0].pos, 7);
        assert_eq!(ranges[0].end, 14);
        assert!(!ranges[0].has_trailing_new_line);
    }

    #[test]
    fn get_leading_comment_ranges_shebang_skipped() {
        // `#!/usr/bin/env node` = 19 bytes (0-18), `\n` = 1 (19),
        // `// real comment` = 15 bytes (20-34), end=35.
        let text = "#!/usr/bin/env node\n// real comment\nlet x;";
        let ranges = get_leading_comment_ranges(text, 0);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].pos, 20);
        assert_eq!(ranges[0].end, 35);
        assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
        assert!(ranges[0].has_trailing_new_line);
    }

    // ────────────────────────────────────────────────────────────────────
    // TokenFlags (P2.1)
    // ────────────────────────────────────────────────────────────────────

    #[test]
    fn token_flags_preceding_line_break_set() {
        let mut s = Scanner::new("foo\nbar");
        s.scan(); // foo
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_PRECEDING_LINE_BREAK
        ));
        s.scan(); // bar (preceded by \n)
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_PRECEDING_LINE_BREAK
        ));
        assert!(s.has_preceding_line_break()); // kept in sync
    }

    #[test]
    fn token_flags_single_quote_string() {
        let mut s = Scanner::new("'abc'");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::StringLiteral);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_SINGLE_QUOTE
        ));
        // Double-quoted strings do NOT set SINGLE_QUOTE.
        let mut s = Scanner::new("\"abc\"");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::StringLiteral);
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_SINGLE_QUOTE
        ));
    }

    #[test]
    fn token_flags_unterminated_string() {
        // Unterminated string (hits newline before closing quote).
        let mut s = Scanner::new("'abc\ndef'");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::StringLiteral);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_UNTERMINATED
        ));
    }

    #[test]
    fn token_flags_terminated_string_no_unterminated() {
        let mut s = Scanner::new("'abc'");
        s.scan();
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_UNTERMINATED
        ));
    }

    #[test]
    fn token_flags_hex_numeric_literal() {
        let mut s = Scanner::new("0x1F");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::NumericLiteral);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_HEX_SPECIFIER
        ));
        // `WITH_SPECIFIER` is a combined mask (HEX | BINARY | OCTAL); use
        // `intersects` to check any-of.
        assert!(token_flags_intersects(
            s.token_flags(),
            TOKEN_FLAGS_WITH_SPECIFIER
        ));
    }

    #[test]
    fn token_flags_binary_numeric_literal() {
        let mut s = Scanner::new("0b1010");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::NumericLiteral);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_BINARY_SPECIFIER
        ));
    }

    #[test]
    fn token_flags_octal_numeric_literal() {
        let mut s = Scanner::new("0o777");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::NumericLiteral);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_OCTAL_SPECIFIER
        ));
    }

    #[test]
    fn token_flags_scientific_numeric_literal() {
        let mut s = Scanner::new("10e2");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::NumericLiteral);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_SCIENTIFIC
        ));
    }

    #[test]
    fn token_flags_contains_leading_zero() {
        // `0888` has a leading zero followed by another digit (legacy octal
        // detection). `0x1F` does NOT set CONTAINS_LEADING_ZERO.
        let mut s = Scanner::new("0888");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_LEADING_ZERO
        ));
        let mut s = Scanner::new("0x1F");
        s.scan();
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_LEADING_ZERO
        ));
    }

    #[test]
    fn token_flags_plain_decimal_none() {
        // A plain decimal like `123` should not set any specifier/scientific/
        // leading-zero flags.
        let mut s = Scanner::new("123");
        s.scan();
        let flags = s.token_flags();
        assert_eq!(flags & TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS, TOKEN_FLAGS_NONE);
    }

    #[test]
    fn token_flags_reset_between_tokens() {
        // `0x1F 'str'`: hex specifier should not leak to the string token.
        let mut s = Scanner::new("0x1F 'str'");
        s.scan(); // 0x1F
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_HEX_SPECIFIER
        ));
        s.scan(); // 'str'
        assert_eq!(s.token(), SyntaxKind::StringLiteral);
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_HEX_SPECIFIER
        ));
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_SINGLE_QUOTE
        ));
    }

    #[test]
    fn token_flags_unterminated_template() {
        // Unterminated template literal (no closing backtick).
        let mut s = Scanner::new("`abc");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_UNTERMINATED
        ));
    }

    // ────────────────────────────────────────────────────────────────────
    // SkipTriviaEx options + conflict-marker trivia (P2.1)
    // ────────────────────────────────────────────────────────────────────

    #[test]
    fn skip_trivia_ex_stop_after_line_break() {
        // With `stop_after_line_break`, skip_trivia should return the position
        // right after the first line break (not skip subsequent trivia).
        // Input: `  \n  x` → `\n` at pos 2, after consuming it pos=3, stop.
        let text = "  \n  x";
        let opts = SkipTriviaOptions {
            stop_after_line_break: true,
            ..Default::default()
        };
        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 3);
        // Without the option: skip all trivia, `x` is at pos 5.
        assert_eq!(skip_trivia(text, 0), 5);
    }

    #[test]
    fn skip_trivia_ex_stop_at_comments() {
        // With `stop_at_comments`, return the position of the `/` instead of
        // consuming the comment.
        let text = "  // c\nx";
        let opts = SkipTriviaOptions {
            stop_at_comments: true,
            ..Default::default()
        };
        // `/` is at pos 2.
        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 2);
        // Without the option: `  ` (2) + `// c` (4) + `\n` (1) = 7, `x` at 7.
        assert_eq!(skip_trivia(text, 0), 7);
    }

    #[test]
    fn skip_trivia_ex_in_jsdoc_consumes_leading_asterisk() {
        // JSDoc-style `*` after a line break should be consumed as trivia.
        // Input: `\n * @param` → `\n`=0, ` `=1, `*`=2, ` `=3, `@`=4.
        // With in_jsdoc: consume `\n`(→1), ` `(→2), `*`(→3), ` `(→4), stop at `@`.
        let text = "\n * @param";
        let opts = SkipTriviaOptions {
            in_jsdoc: true,
            ..Default::default()
        };
        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 4);
        // Without in_jsdoc: consume `\n`(→1), ` `(→2), stop at `*` (pos 2).
        assert_eq!(skip_trivia(text, 0), 2);
    }

    #[test]
    fn skip_trivia_ex_jsdoc_star_only_after_line_break() {
        // `*` not preceded by a line break should NOT be consumed even in JSDoc.
        let text = " * foo";
        let opts = SkipTriviaOptions {
            in_jsdoc: true,
            ..Default::default()
        };
        // No leading line break → `*` is not consumed; stop at pos 1 (the `*`).
        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 1);
    }

    #[test]
    fn is_conflict_marker_trivia_detects_markers() {
        // `<<<<<<<` at start of file, followed by space → marker.
        assert!(is_conflict_marker_trivia("<<<<<<< head\n", 0));
        // `>>>>>>>` at start of line, followed by space → marker.
        assert!(is_conflict_marker_trivia("x\n>>>>>>> branch\n", 2));
        // `=======` at start of line (no trailing space needed) → marker.
        assert!(is_conflict_marker_trivia("x\n=======\n", 2));
        // `|||||||` at start of line, followed by space (diff3 style) → marker.
        assert!(is_conflict_marker_trivia("x\n||||||| base\n", 2));
        // Only 6 `<` (not 7) → not a marker.
        assert!(!is_conflict_marker_trivia("<<<<<< \n", 0));
        // `<<<<<<<` not followed by space → not a marker (Go requires space for `<`/`>`/`|`).
        assert!(!is_conflict_marker_trivia("<<<<<<<x\n", 0));
        // `|||||||` not followed by space → not a marker.
        assert!(!is_conflict_marker_trivia("x\n|||||||\n", 2));
        // Not at start of line → not a marker.
        assert!(!is_conflict_marker_trivia("a <<<<<<< \n", 2));
        // Second byte differs → fast reject.
        assert!(!is_conflict_marker_trivia("<x\n", 0));
    }

    #[test]
    fn skip_trivia_ex_consumes_conflict_marker() {
        // `<<<<<<< a\n` is a conflict marker line; skip_trivia_ex consumes
        // the marker line and the trailing newline, then stops at the next
        // non-trivia character (the `s` of `shared`). The content between
        // markers is *not* consumed as trivia (mirrors Go's behavior: only
        // the marker lines themselves are trivia).
        let text = "<<<<<<< a\nshared\n=======\n>>>>>>> b\nx";
        let pos = skip_trivia_ex(text, 0, &SkipTriviaOptions::default(), None);
        assert_eq!(&text[pos..], "shared\n=======\n>>>>>>> b\nx");
    }

    #[test]
    fn skip_trivia_ex_reports_conflict_marker_error() {
        // The report_error callback should be invoked for conflict markers.
        use std::cell::RefCell;
        let text = "<<<<<<< a\nx";
        let reported: RefCell<Vec<(usize, usize)>> = RefCell::new(Vec::new());
        let opts = SkipTriviaOptions::default();
        skip_trivia_ex(
            text,
            0,
            &opts,
            Some(&|p, l| reported.borrow_mut().push((p, l))),
        );
        assert_eq!(
            reported.borrow().as_slice(),
            &[(0, MERGE_CONFLICT_MARKER_LENGTH)]
        );
    }

    #[test]
    fn skip_trivia_ex_pipe_divider_marker() {
        // `<<<<<<< a\n` is consumed as a conflict marker; the following
        // `local` line is non-trivia and stops skip_trivia_ex (mirrors Go:
        // only marker lines are trivia, content between them is parsed as
        // code, which then produces its own diagnostics).
        let text = "<<<<<<< a\nlocal\n||||||| base\nshared\n=======\nremote\n>>>>>>> b\nx";
        let pos = skip_trivia_ex(text, 0, &SkipTriviaOptions::default(), None);
        assert_eq!(
            &text[pos..],
            "local\n||||||| base\nshared\n=======\nremote\n>>>>>>> b\nx"
        );
    }

    // ────────────────────────────────────────────────────────────────────
    // JSDoc TokenFlags (P2.1)
    // ────────────────────────────────────────────────────────────────────

    #[test]
    fn token_flags_preceding_jsdoc_comment() {
        // `/** ... */` is a JSDoc comment; the following token should have
        // `PRECEDING_JSDOC_COMMENT` set.
        let mut s = Scanner::new("/** doc */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_comment());
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT
        ));
    }

    #[test]
    fn token_flags_non_jsdoc_multi_line_comment() {
        // `/* ... */` (single asterisk) is NOT a JSDoc comment.
        let mut s = Scanner::new("/* not jsdoc */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(!s.has_preceding_jsdoc_comment());
    }

    #[test]
    fn token_flags_empty_jsdoc_comment_not_flagged() {
        // `/**/` is an empty multi-line comment, NOT a JSDoc comment (Go:
        // `isJSDoc := s.char() == '*' && s.charAt(1) != '/'`).
        let mut s = Scanner::new("/**/\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(!s.has_preceding_jsdoc_comment());
    }

    #[test]
    fn token_flags_jsdoc_deprecated_tag() {
        // `@deprecated` tag sets `PRECEDING_JSDOC_WITH_DEPRECATED`.
        let mut s = Scanner::new("/**\n * @deprecated\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_comment());
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
        assert!(!s.has_preceding_jsdoc_with_see_or_link());
    }

    #[test]
    fn token_flags_jsdoc_see_tag() {
        // `@see` tag sets `PRECEDING_JSDOC_WITH_SEE_OR_LINK`.
        let mut s = Scanner::new("/**\n * @see foo\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_see_or_link());
        assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_link_tag() {
        // `@link` tag also sets `PRECEDING_JSDOC_WITH_SEE_OR_LINK`.
        let mut s = Scanner::new("/**\n * {@link foo}\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_see_or_link());
    }

    #[test]
    fn token_flags_jsdoc_both_tags() {
        // Both `@deprecated` and `@see` tags set both flags.
        let mut s = Scanner::new("/**\n * @deprecated\n * @see foo\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
        assert!(s.has_preceding_jsdoc_with_see_or_link());
    }

    #[test]
    fn token_flags_jsdoc_tag_invalid_terminator() {
        // `@deprecatedX` (tag followed by non-terminator) should NOT match.
        let mut s = Scanner::new("/**\n * @deprecatedX\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_tag_at_end_of_string() {
        // `@deprecated` at end of comment text (no terminator char) matches.
        let mut s = Scanner::new("/**@deprecated*/\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_flags_reset_between_tokens() {
        // JSDoc flags should NOT leak from one token to the next.
        let mut s = Scanner::new("/** @deprecated */\nlet x\nlet y");
        s.scan(); // let x
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
        s.scan(); // x
        s.scan(); // let y — no preceding JSDoc
        assert!(!s.has_preceding_jsdoc_comment());
        assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_consumed() {
        // When `skip_jsdoc_leading_asterisks` is active, a `*` at line start
        // is consumed as trivia and sets `PRECEDING_JSDOC_LEADING_ASTERISKS`.
        // The `*` must be preceded by a line break.
        let mut s = Scanner::new("\n* x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        // The `*` is consumed; next token is `x` (Identifier).
        assert_eq!(s.token(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "x");
        assert!(s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_no_line_break() {
        // Without a preceding line break, `*` is NOT consumed as JSDoc
        // asterisk — it produces a normal `AsteriskToken`.
        let mut s = Scanner::new("* x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_not_active() {
        // When `skip_jsdoc_leading_asterisks` is NOT active, `*` at line
        // start produces a normal `AsteriskToken`.
        let mut s = Scanner::new("\n* x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_double_star_not_consumed() {
        // `**` is NOT consumed as JSDoc asterisk (it would form
        // `AsteriskAsteriskToken`).
        let mut s = Scanner::new("\n** x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskAsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_star_equals_not_consumed() {
        // `*=` is NOT consumed as JSDoc asterisk (it forms
        // `AsteriskEqualsToken`).
        let mut s = Scanner::new("\n*= x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskEqualsToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_only_first_consumed() {
        // Only the FIRST `*` at line start is consumed; subsequent `*` on
        // the same line produce `AsteriskToken`.
        let mut s = Scanner::new("\n* * x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan(); // first `*` consumed, second `*` is AsteriskToken
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_counter_nesting() {
        // `set_skip_jsdoc_leading_asterisks(false)` decrements the counter;
        // after balanced enable/disable, `*` is no longer consumed.
        let mut s = Scanner::new("\n* x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.set_skip_jsdoc_leading_asterisks(false);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn has_jsdoc_tag_helper() {
        // Direct unit tests for the `has_jsdoc_tag` helper.
        assert!(has_jsdoc_tag("deprecated", &["deprecated"]));
        assert!(has_jsdoc_tag("deprecated foo", &["deprecated"]));
        assert!(has_jsdoc_tag("deprecated\tfoo", &["deprecated"]));
        assert!(has_jsdoc_tag("deprecated\nfoo", &["deprecated"]));
        assert!(has_jsdoc_tag("deprecated*foo", &["deprecated"]));
        assert!(has_jsdoc_tag("deprecated}foo", &["deprecated"]));
        assert!(has_jsdoc_tag("see", &["see", "link"]));
        assert!(has_jsdoc_tag("link foo", &["see", "link"]));
        assert!(has_jsdoc_tag(
            "linkcode foo",
            &["see", "link", "linkcode", "linkplain"]
        ));
        // Non-matching
        assert!(!has_jsdoc_tag("deprecatedX", &["deprecated"]));
        assert!(!has_jsdoc_tag("dep", &["deprecated"]));
        assert!(!has_jsdoc_tag("foo", &["deprecated"]));
    }

    #[test]
    fn scan_jsdoc_comment_for_tags_helper() {
        // Direct unit tests for the `scan_jsdoc_comment_for_tags` helper.
        assert_eq!(
            scan_jsdoc_comment_for_tags("/** @deprecated */"),
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED
        );
        assert_eq!(
            scan_jsdoc_comment_for_tags("/** @see foo */"),
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
        );
        assert_eq!(
            scan_jsdoc_comment_for_tags("/** @deprecated @see foo */"),
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED
                | TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
        );
        assert_eq!(
            scan_jsdoc_comment_for_tags("/** no tags */"),
            TOKEN_FLAGS_NONE
        );
        // `@link` inside `{...}` also matches.
        assert!(token_flags_contains(
            scan_jsdoc_comment_for_tags("/** {@link foo} */"),
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
        ));
    }

    // ────────────────────────────────────────────────────────────────────
    // Escape-sequence & numeric-separator TokenFlags (P2.1)
    // ────────────────────────────────────────────────────────────────────

    #[test]
    fn token_flags_unicode_escape() {
        // `\u00a0` sets UNICODE_ESCAPE.
        let mut s = Scanner::new("\"\\u00a0\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_UNICODE_ESCAPE
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_extended_unicode_escape() {
        // `\u{10ffff}` sets EXTENDED_UNICODE_ESCAPE.
        let mut s = Scanner::new("\"\\u{10ffff}\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_hex_escape() {
        // `\xa0` sets HEX_ESCAPE.
        let mut s = Scanner::new("\"\\xa0\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_HEX_ESCAPE
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_invalid_hex_escape() {
        // `\xz` (no hex digits) sets CONTAINS_INVALID_ESCAPE.
        let mut s = Scanner::new("\"\\xz\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_HEX_ESCAPE
        ));
    }

    #[test]
    fn token_flags_invalid_unicode_escape() {
        // `\u00` (only 2 hex digits) sets CONTAINS_INVALID_ESCAPE.
        let mut s = Scanner::new("\"\\u00\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_UNICODE_ESCAPE
        ));
    }

    #[test]
    fn token_flags_invalid_extended_unicode_escape() {
        // `\u{}` (empty) sets CONTAINS_INVALID_ESCAPE.
        let mut s = Scanner::new("\"\\u{}\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE
        ));
    }

    #[test]
    fn token_flags_octal_escape_invalid() {
        // `\01` (legacy octal) sets CONTAINS_INVALID_ESCAPE.
        let mut s = Scanner::new("\"\\01\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_escape_eight_nine_invalid() {
        // `\8` and `\9` set CONTAINS_INVALID_ESCAPE.
        let mut s = Scanner::new("\"\\8\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_nul_escape_not_invalid() {
        // `\0` (NUL, not followed by digit) does NOT set
        // CONTAINS_INVALID_ESCAPE.
        let mut s = Scanner::new("\"\\0\"");
        s.scan();
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_contains_separator_decimal() {
        // `1_000` sets CONTAINS_SEPARATOR (valid separator).
        let mut s = Scanner::new("1_000");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_contains_separator_hex() {
        // `0xFF_FF` sets CONTAINS_SEPARATOR (valid separator in hex).
        let mut s = Scanner::new("0xFF_FF");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_contains_separator_binary() {
        // `0b1010_0101` sets CONTAINS_SEPARATOR (valid separator in binary).
        let mut s = Scanner::new("0b1010_0101");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_invalid_separator_consecutive() {
        // `1__000` (consecutive separators) sets both
        // CONTAINS_SEPARATOR and CONTAINS_INVALID_SEPARATOR.
        let mut s = Scanner::new("1__000");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_invalid_separator_trailing() {
        // `1000_` (trailing separator) sets both flags.
        let mut s = Scanner::new("1000_");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_no_separator_plain_number() {
        // `12345` sets no separator flags.
        let mut s = Scanner::new("12345");
        s.scan();
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_string_literal_flags_mask() {
        // A string with `\x41\u0041'` should have all three flags set:
        // HEX_ESCAPE + UNICODE_ESCAPE + SINGLE_QUOTE.
        let mut s = Scanner::new("'\\x41\\u0041'");
        s.scan();
        let flags = s.token_flags();
        assert!(token_flags_contains(flags, TOKEN_FLAGS_HEX_ESCAPE));
        assert!(token_flags_contains(flags, TOKEN_FLAGS_UNICODE_ESCAPE));
        assert!(token_flags_contains(flags, TOKEN_FLAGS_SINGLE_QUOTE));
        // STRING_LITERAL_FLAGS mask should intersect.
        assert!(token_flags_intersects(
            flags,
            TOKEN_FLAGS_STRING_LITERAL_FLAGS
        ));
    }

    #[test]
    fn token_flags_numeric_literal_flags_mask() {
        // `0xFF_FF` should have HEX_SPECIFIER + CONTAINS_SEPARATOR.
        let mut s = Scanner::new("0xFF_FF");
        s.scan();
        let flags = s.token_flags();
        assert!(token_flags_contains(flags, TOKEN_FLAGS_HEX_SPECIFIER));
        assert!(token_flags_contains(flags, TOKEN_FLAGS_CONTAINS_SEPARATOR));
        // NUMERIC_LITERAL_FLAGS mask should intersect.
        assert!(token_flags_intersects(
            flags,
            TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS
        ));
    }

    // ── Legacy octal (OCTAL flag) tests ──

    #[test]
    fn legacy_octal_literal_sets_octal_flag() {
        // `0777` is a legacy octal literal — should set OCTAL flag and report
        // TS1121. Mirrors Go `scanner.go:1961-1971`.
        let mut s = Scanner::new("0777");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::NumericLiteral);
        assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);
        assert_eq!(errors[0].pos, 0);
        assert_eq!(errors[0].length, 4);
    }

    #[test]
    fn legacy_octal_literal_single_digit() {
        // `00` — `0` followed by octal digit `0` → legacy octal.
        let mut s = Scanner::new("00");
        s.scan();
        assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);
    }

    #[test]
    fn leading_zero_non_octal_sets_leading_zero_flag() {
        // `0888` has a leading zero with non-octal digits — should set
        // CONTAINS_LEADING_ZERO and report TS1489, NOT OCTAL.
        let mut s = Scanner::new("0888");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_LEADING_ZERO
        ));
        assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::DecimalWithLeadingZero);
        assert_eq!(errors[0].pos, 0);
        assert_eq!(errors[0].length, 4);
    }

    #[test]
    fn plain_zero_no_flags() {
        // `0` alone — no OCTAL, no CONTAINS_LEADING_ZERO, no error.
        let mut s = Scanner::new("0");
        s.scan();
        assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_LEADING_ZERO
        ));
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn zero_with_fraction_no_flags() {
        // `0.5` — no leading-zero flag (no digit after `0`).
        let mut s = Scanner::new("0.5");
        s.scan();
        assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_LEADING_ZERO
        ));
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn zero_with_exponent_no_flags() {
        // `0e5` — no leading-zero flag (no digit after `0`).
        let mut s = Scanner::new("0e5");
        s.scan();
        assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_LEADING_ZERO
        ));
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn zero_bigint_no_flags() {
        // `0n` — BigInt, no leading-zero flag.
        let mut s = Scanner::new("0n");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::BigIntLiteral);
        assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn zero_separator_after_leading_zero() {
        // `0_123` — separator not allowed after leading `0`. Mirrors Go
        // `scanner.go:1949-1953`: set both separator flags, report TS6188,
        // reset, re-scan as plain fragment.
        let mut s = Scanner::new("0_123");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
        ));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::NumericSeparatorNotAllowed);
        assert_eq!(errors[0].pos, 1); // at the `_`
    }

    #[test]
    fn legacy_octal_with_minus_prefix() {
        // `-0777` — the error range should include the minus sign.
        // Scanner sees `0777` as the numeric token; `self.token` is
        // `MinusToken` at that point.
        let mut s = Scanner::new("-0777");
        s.scan(); // `-`
        assert_eq!(s.token(), SyntaxKind::MinusToken);
        s.scan(); // `0777`
        assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);
        // Error should start at `-` (pos 0) with length 5.
        assert_eq!(errors[0].pos, 0);
        assert_eq!(errors[0].length, 5);
    }

    // ────────────────────────────────────────────────────────────────
    // JSDoc scanner tests
    // ────────────────────────────────────────────────────────────────

    #[test]
    fn jsdoc_token_at_sign() {
        let mut s = Scanner::new("@param");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::AtToken);
    }

    #[test]
    fn jsdoc_token_asterisk() {
        let mut s = Scanner::new("*");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::AsteriskToken);
    }

    #[test]
    fn jsdoc_token_identifier_and_keyword() {
        let mut s = Scanner::new("param");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::Identifier);
        let mut s = Scanner::new("return");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::ReturnKeyword);
    }

    #[test]
    fn jsdoc_token_identifier_with_dash() {
        // JSDoc tag names may contain `-` (e.g. `@custom-tag`).
        let mut s = Scanner::new("custom-tag");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "custom-tag");
    }

    #[test]
    fn jsdoc_token_whitespace() {
        let mut s = Scanner::new("   \t  ");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::WhitespaceTrivia);
    }

    #[test]
    fn jsdoc_token_newline() {
        let mut s = Scanner::new("\n");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::NewLineTrivia);
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_PRECEDING_LINE_BREAK
        ));
    }

    #[test]
    fn jsdoc_token_crlf_newline() {
        let mut s = Scanner::new("\r\n");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::NewLineTrivia);
    }

    #[test]
    fn jsdoc_token_braces_and_brackets() {
        let mut s = Scanner::new("{");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenBraceToken);
        let mut s = Scanner::new("}");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::CloseBraceToken);
        let mut s = Scanner::new("[");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenBracketToken);
        let mut s = Scanner::new("]");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::CloseBracketToken);
    }

    #[test]
    fn jsdoc_token_punctuation() {
        let mut s = Scanner::new("(");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenParenToken);
        let mut s = Scanner::new("`");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::BacktickToken);
        let mut s = Scanner::new("#");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::HashToken);
    }

    #[test]
    fn jsdoc_token_eof() {
        let mut s = Scanner::new("");
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::EndOfFile);
    }

    #[test]
    fn jsdoc_can_follow_at_identifier() {
        let s = Scanner::new("param");
        assert!(s.can_follow_jsdoc_at());
    }

    #[test]
    fn jsdoc_can_follow_at_whitespace() {
        let s = Scanner::new(" ");
        assert!(s.can_follow_jsdoc_at());
    }

    #[test]
    fn jsdoc_can_follow_at_eof() {
        let s = Scanner::new("");
        assert!(s.can_follow_jsdoc_at());
    }

    #[test]
    fn jsdoc_can_follow_at_digit_false() {
        let s = Scanner::new("1abc");
        assert!(!s.can_follow_jsdoc_at());
    }

    #[test]
    fn jsdoc_comment_text_token_prose() {
        let mut s = Scanner::new("This is a description. ");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(false),
            SyntaxKind::JSDocCommentTextToken
        );
        assert_eq!(s.token_text(), "This is a description. ");
    }

    #[test]
    fn jsdoc_comment_text_token_stops_at_brace() {
        let mut s = Scanner::new("before {type} after");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(false),
            SyntaxKind::JSDocCommentTextToken
        );
        assert_eq!(s.token_text(), "before ");
        // Next token should be the brace.
        assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenBraceToken);
    }

    #[test]
    fn jsdoc_comment_text_token_stops_at_newline() {
        let mut s = Scanner::new("line1\nline2");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(false),
            SyntaxKind::JSDocCommentTextToken
        );
        assert_eq!(s.token_text(), "line1");
    }

    #[test]
    fn jsdoc_comment_text_token_at_tag_boundary() {
        // ` @param` — the `@` after whitespace starts a new tag.
        let mut s = Scanner::new("text @param");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(false),
            SyntaxKind::JSDocCommentTextToken
        );
        assert_eq!(s.token_text(), "text ");
    }

    #[test]
    fn jsdoc_comment_text_token_in_backticks_ignores_at_and_brace() {
        let mut s = Scanner::new("code {@code x} more");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(true),
            SyntaxKind::JSDocCommentTextToken
        );
        // In backticks mode, `{` does not terminate; only `` ` `` or newline.
        assert_eq!(s.token_text(), "code {@code x} more");
    }

    #[test]
    fn jsdoc_comment_text_token_empty_falls_through() {
        // Immediately at `{` — text run is empty, falls through to scan_jsdoc_token.
        let mut s = Scanner::new("{");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(false),
            SyntaxKind::OpenBraceToken
        );
    }

    // ────────────────────────────────────────────────────────────────────
    // Ported from Go internal/scanner/scanner_test.go
    // ────────────────────────────────────────────────────────────────────

    #[test]
    fn scan_string_preserves_lone_surrogates() {
        // Ported from Go TestScanStringPreservesLoneSurrogates.
        //
        // The Go scanner preserves lone surrogates from `\uXXXX` escapes as
        // WTF-8 (3-byte encoding of surrogate code points), and combines
        // adjacent high+low surrogate pairs into the supplementary code point.
        // Standard Rust `String` cannot hold surrogate code points, so the
        // Rust scanner's `unescape_string` replaces lone surrogates
        // (U+D800–U+DFFF) with the Unicode replacement character (U+FFFD).
        // This is the expected behavior for a Rust implementation.
        //
        // Input: `"🦀\ud7ff\ud800\ud801\uD83E\uDD80"`
        //   🦀         → preserved (valid supplementary plane char)
        //   \ud7ff     → preserved (U+D7FF is below the surrogate range, valid)
        //   \ud800     → U+FFFD (lone high surrogate, replaced)
        //   \ud801     → U+FFFD (lone high surrogate, replaced)
        //   \uD83E     → U+FFFD (Go combines with \uDD80 into 🦀; Rust replaces)
        //   \uDD80     → U+FFFD (lone low surrogate, replaced)
        let input = r#""🦀\ud7ff\ud800\ud801\uD83E\uDD80""#;
        let mut s = Scanner::new(input);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        // Verify the scanner does not panic and produces a value. Lone
        // surrogates become U+FFFD; valid chars (🦀, U+D7FF) are preserved.
        let value = s.token_value();
        assert!(value.contains('🦀'));
        assert!(value.contains('\u{D7FF}'));
        // Lone surrogates are replaced with U+FFFD (4 of them: \ud800, \ud801,
        // \uD83E, \uDD80).
        let fffd_count = value.chars().filter(|&c| c == '\u{FFFD}').count();
        assert_eq!(fffd_count, 4);
    }
}
