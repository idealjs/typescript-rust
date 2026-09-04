use crate::ast::SyntaxKind;
use crate::core::compiler_options::ScriptTarget;
use std::collections::HashMap;
use std::sync::OnceLock;

mod regexp;
mod unicode_properties;

pub type ErrorCallback = fn(kind: DiagnosticKind, start: usize, length: usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    InvalidCharacter,

    FileAppearsToBeBinary,
    UnterminatedStringLiteral,
    UnterminatedTemplateLiteral,
    UnterminatedRegularExpression,

    UnknownRegularExpressionFlag,

    DuplicateRegularExpressionFlag,

    UnicodeUAndVFlagsMutuallyExclusive,

    OctalLiteralNotAllowed,

    DecimalWithLeadingZero,

    NumericSeparatorNotAllowed,

    RegexMessage(crate::diagnostics::Message),
}

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

pub fn token_flags_contains(flags: TokenFlags, bit: TokenFlags) -> bool {
    (flags & bit) == bit
}

pub fn token_flags_intersects(flags: TokenFlags, mask: TokenFlags) -> bool {
    (flags & mask) != 0
}

static TEXT_TO_KEYWORD: OnceLock<HashMap<&'static str, SyntaxKind>> = OnceLock::new();

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

pub fn string_to_keyword(text: &str) -> Option<SyntaxKind> {
    keywords().get(text).copied()
}

pub fn string_to_token(text: &str) -> Option<SyntaxKind> {
    punctuation().get(text).copied()
}

static TOKEN_TO_TEXT: OnceLock<HashMap<SyntaxKind, &'static str>> = OnceLock::new();

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

#[derive(Clone)]
pub struct Scanner {
    text: String,
    pos: usize,
    end: usize,
    token: SyntaxKind,
    token_pos: usize,
    token_end: usize,

    full_start_pos: usize,
    preceding_line_break: bool,
    has_preceding_line_break: bool,

    binary_marker_pos: Option<usize>,

    token_flags: TokenFlags,

    skip_jsdoc_leading_asterisks: i32,
    error_callback: Option<ErrorCallback>,

    errors: Vec<ScannerError>,

    comment_directives: Vec<CommentDirective>,

    script_target: crate::core::compiler_options::ScriptTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerError {
    pub kind: DiagnosticKind,
    pub pos: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentDirectiveKind {

    ExpectError,

    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentDirective {
    pub pos: usize,
    pub end: usize,
    pub kind: CommentDirectiveKind,
}

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

    pub fn set_script_target(&mut self, target: crate::core::compiler_options::ScriptTarget) {
        self.script_target = target;
    }

    fn report_error(&mut self, kind: DiagnosticKind, pos: usize, length: usize) {
        if let Some(cb) = self.error_callback {
            cb(kind, pos, length);
        }
        self.errors.push(ScannerError { kind, pos, length });
    }

    pub fn take_errors(&mut self) -> Vec<ScannerError> {
        std::mem::take(&mut self.errors)
    }

    pub fn comment_directives(&self) -> &[CommentDirective] {
        &self.comment_directives
    }

    fn process_comment_directive(&mut self, start: usize, end: usize, multiline: bool) {
        let text = self.text.as_bytes();
        let mut pos = start;
        if multiline {

            while pos < end && (text[pos] == b' ' || text[pos] == b'\t') {
                pos += 1;
            }

            while pos < end && (text[pos] == b'/' || text[pos] == b'*') {
                pos += 1;
            }
        } else {

            pos += 2;

            while pos < end && text[pos] == b'/' {
                pos += 1;
            }
        }

        while pos < end && (text[pos] == b' ' || text[pos] == b'\t') {
            pos += 1;
        }

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

    pub fn token(&self) -> SyntaxKind {
        self.token
    }

    pub fn token_pos(&self) -> usize {
        self.token_pos
    }

    pub fn full_start_pos(&self) -> usize {
        self.full_start_pos
    }

    pub fn token_end(&self) -> usize {
        self.token_end
    }

    pub fn token_text(&self) -> &str {
        &self.text[self.token_pos..self.token_end]
    }

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

    pub fn has_preceding_line_break(&self) -> bool {
        self.has_preceding_line_break
    }

    pub fn token_flags(&self) -> TokenFlags {
        self.token_flags
    }

    pub fn has_preceding_jsdoc_comment(&self) -> bool {
        token_flags_contains(self.token_flags, TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT)
    }

    pub fn has_preceding_jsdoc_leading_asterisks(&self) -> bool {
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS,
        )
    }

    pub fn has_preceding_jsdoc_with_deprecated_tag(&self) -> bool {
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED,
        )
    }

    pub fn has_preceding_jsdoc_with_see_or_link(&self) -> bool {
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK,
        )
    }

    pub fn set_skip_jsdoc_leading_asterisks(&mut self, skip: bool) {
        if skip {
            self.skip_jsdoc_leading_asterisks += 1;
        } else {
            self.skip_jsdoc_leading_asterisks -= 1;
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn skip_jsdoc_leading_asterisks_raw(&self) -> i32 {
        self.skip_jsdoc_leading_asterisks
    }

    pub fn set_skip_jsdoc_leading_asterisks_raw(&mut self, value: i32) {
        self.skip_jsdoc_leading_asterisks = value;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

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

    pub fn scan(&mut self) -> SyntaxKind {

        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;

        self.full_start_pos = self.pos;

        let token = loop {
            self.token_pos = self.pos;

            if self.pos >= self.end {
                self.token = SyntaxKind::EndOfFile;
                self.token_end = self.pos;
                break self.token;
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

            break self.scan_punctuation();
        };

        self.has_preceding_line_break = self.preceding_line_break;
        if self.preceding_line_break {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_LINE_BREAK;
        }
        token
    }

    pub fn scan_template_continuation(&mut self) -> SyntaxKind {
        self.preceding_line_break = false;
        self.token_flags = TOKEN_FLAGS_NONE;
        self.token_pos = self.pos;

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

        self.pos += 2;

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

        if is_jsdoc {
            self.token_flags |= TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT;
            let comment_text = &self.text[comment_start..self.pos];
            self.token_flags |= scan_jsdoc_comment_for_tags(comment_text);
        }
    }

    fn scan_identifier(&mut self) -> SyntaxKind {
        let start = self.pos;

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

    fn scan_private_identifier(&mut self) -> SyntaxKind {

        self.pos += 1;

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

                self.pos += 2;
                self.scan_number_fragment_with_sep(true, true);
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_HEX_SPECIFIER;
                return self.token;
            }
            if next == 'b' || next == 'B' {

                self.pos += 2;
                self.scan_binary_fragment_with_sep();
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_BINARY_SPECIFIER;
                return self.token;
            }
            if next == 'o' || next == 'O' {

                self.pos += 2;
                self.scan_octal_specifier_fragment_with_sep();
                self.token_end = self.pos;
                self.token = SyntaxKind::NumericLiteral;
                self.token_flags |= TOKEN_FLAGS_OCTAL_SPECIFIER;
                return self.token;
            }
        }

        if self.text.as_bytes()[self.pos] as char == '0' {
            self.pos += 1;
            if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '_' {

                self.token_flags |=
                    TOKEN_FLAGS_CONTAINS_SEPARATOR | TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR;
                self.report_error(DiagnosticKind::NumericSeparatorNotAllowed, self.pos, 1);
                self.pos = start;
                self.scan_number_fragment_with_sep(false, false);
            } else {

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

                    self.token_flags |= TOKEN_FLAGS_CONTAINS_LEADING_ZERO;
                }

            }
        } else {

            self.scan_number_fragment_with_sep(false, false);
        }

        if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '.' {
            self.pos += 1;
            self.scan_number_fragment_with_sep(false, false);
        }

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

        if token_flags_contains(self.token_flags, TOKEN_FLAGS_CONTAINS_LEADING_ZERO) {
            self.report_error(
                DiagnosticKind::DecimalWithLeadingZero,
                start,
                self.pos - start,
            );
        }

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

    fn scan_number_fragment_with_sep(&mut self, is_hex: bool, _can_have_sep: bool) {
        let mut allow_separator = false;
        let mut is_prev_separator = false;
        loop {
            let before = self.pos;

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
        self.pos += 1;
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

    fn scan_escape_sequence(&mut self) {

        self.pos += 1;
        if self.pos >= self.end {
            return;
        }
        let c = self.text.as_bytes()[self.pos] as char;
        self.pos += 1;
        match c {
            '0' => {

                if self.pos < self.end && is_digit(self.text.as_bytes()[self.pos] as char) {
                    self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;

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

                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                if self.pos < self.end && is_octal_digit(self.text.as_bytes()[self.pos] as char) {
                    self.pos += 1;
                }
            }
            '8' | '9' => {

                self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
            }
            'x' => {

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

                    self.pos += 1;
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
                        self.pos += 1;
                    }
                    if has_hex && closed {
                        self.token_flags |= TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE;
                    } else {
                        self.token_flags |= TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
                    }
                } else {

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

                if self.pos < self.end && self.text.as_bytes()[self.pos] as char == '\n' {
                    self.pos += 1;
                }
            }

            _ => {}
        }
    }

    fn scan_template(&mut self) -> SyntaxKind {

        self.pos += 1;
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
                terminated = true;
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

        let remaining = &self.text[start..];
        let mut best_match: Option<SyntaxKind> = None;
        let mut best_len = 0;

        if remaining.len() >= 4 {
            if let Some(slice) = remaining.get(..4) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 4;
                }
            }
        }

        if best_len == 0 && remaining.len() >= 3 {
            if let Some(slice) = remaining.get(..3) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 3;
                }
            }
        }

        if best_len == 0 && remaining.len() >= 2 {
            if let Some(slice) = remaining.get(..2) {
                if let Some(kind) = string_to_token(slice) {
                    best_match = Some(kind);
                    best_len = 2;
                }
            }
        }

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

            let c = self.text[start..].chars().next().unwrap();

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

    pub fn binary_marker_pos(&self) -> Option<usize> {
        self.binary_marker_pos
    }

    pub fn rewind(&mut self) {
        self.pos = self.token_pos;
    }

    pub fn re_scan_greater_than(&mut self) -> SyntaxKind {
        let token = self.token;
        if token == SyntaxKind::GreaterThanToken {
            return token;
        }

        match token {
            SyntaxKind::GreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanToken
            | SyntaxKind::GreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {}
            _ => return token,
        }

        self.pos = self.token_pos + 1;
        self.token_end = self.pos;
        self.token = SyntaxKind::GreaterThanToken;
        SyntaxKind::GreaterThanToken
    }

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

    pub fn re_scan_slash_token(&mut self) -> SyntaxKind {
        if self.token != SyntaxKind::SlashToken && self.token != SyntaxKind::SlashEqualsToken {
            return self.token;
        }

        let start_of_regex_body = self.token_pos + 1;
        let mut p = start_of_regex_body;
        let mut in_escape = false;
        let mut in_character_class = false;
        let mut unterminated = false;

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
                    break;
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

            self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
            self.report_error(
                DiagnosticKind::UnterminatedRegularExpression,
                self.token_pos,
                p - self.token_pos,
            );
            self.pos = p;
        } else {

            p += 1;

            let mut seen_flags: u16 = 0;
            while p < self.end {

                let c = self.text[p..].chars().next().unwrap_or('\0');
                let c_len = c.len_utf8();
                if !is_identifier_part(c) {
                    break;
                }
                if let Some(bit) = reg_exp_flag_bit(c) {
                    if seen_flags & bit != 0 {

                        self.report_error(DiagnosticKind::DuplicateRegularExpressionFlag, p, 1);
                    } else if (seen_flags | bit) & (REG_EXP_FLAG_U | REG_EXP_FLAG_V)
                        == (REG_EXP_FLAG_U | REG_EXP_FLAG_V)
                    {

                        self.report_error(DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive, p, 1);
                    } else {
                        seen_flags |= bit;

                        self.check_reg_exp_flag_availability(bit, p);
                    }
                } else {

                    self.report_error(DiagnosticKind::UnknownRegularExpressionFlag, p, c_len);
                }
                p += c_len;
            }
            self.pos = p;

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

    pub fn scan_jsx_token(&mut self) -> SyntaxKind {
        self.scan_jsx_token_ex(true)
    }

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

        let mut first_non_whitespace = 0usize;
        let start = self.pos;

        while self.pos < self.end {
            let ch = self.text[self.pos..].chars().next().unwrap();
            let size = ch.len_utf8();

            if ch == '{' || ch == '<' {
                break;
            }

            if is_jsx_line_break(ch) && first_non_whitespace == 0 {
                first_non_whitespace = usize::MAX;
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
        let _ = start;
        self.token
    }

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

            let text = self.token_text();
            self.token = string_to_keyword(text).unwrap_or(SyntaxKind::Identifier);
        }
        self.token
    }

    pub fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {

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

        self.scan()
    }

    pub fn can_follow_jsdoc_at(&self) -> bool {
        if self.pos >= self.end {
            return true;
        }
        let (ch, _) = decode_char(&self.text, self.pos);
        is_identifier_start(ch) || is_whitespace_single_line(ch) || is_line_break(ch)
    }

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

fn is_jsx_line_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

fn is_jsx_whitespace_like(c: char) -> bool {

    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}') || c.is_whitespace()
}

fn is_identifier_or_keyword_token(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || is_keyword(token)
}

fn is_keyword(token: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_keyword_kind(token)
}

fn is_whitespace(c: char) -> bool {

    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '\r'
            | '\x0B'
            | '\x0C'
            | '\u{85}'
            | '\u{A0}'
            | '\u{1680}'
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
            | '\u{200B}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
    )
}

fn is_line_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

const REG_EXP_FLAG_G: u16 = 1 << 0;
const REG_EXP_FLAG_I: u16 = 1 << 1;
const REG_EXP_FLAG_M: u16 = 1 << 2;
const REG_EXP_FLAG_S: u16 = 1 << 3;
const REG_EXP_FLAG_U: u16 = 1 << 4;
const REG_EXP_FLAG_Y: u16 = 1 << 5;
const REG_EXP_FLAG_D: u16 = 1 << 6;
const REG_EXP_FLAG_V: u16 = 1 << 7;

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

    unicode_ident::is_xid_start(c)
}

fn is_unicode_identifier_part(c: char) -> bool {

    unicode_ident::is_xid_continue(c) || c == '\u{200C}' || c == '\u{200D}'
}

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
                Some('\n') => {}
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentRangeKind {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentRange {
    pub pos: usize,
    pub end: usize,
    pub kind: CommentRangeKind,
    pub has_trailing_new_line: bool,
}

fn decode_char(text: &str, pos: usize) -> (char, usize) {
    let c = text[pos..].chars().next().unwrap();
    (c, c.len_utf8())
}

fn is_whitespace_like(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}') || c.is_whitespace()
}

fn is_whitespace_single_line(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}')
}

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

pub fn get_shebang(text: &str) -> &str {
    if !is_shebang_trivia(text, 0) {
        return "";
    }
    let end = scan_shebang_trivia(text, 0);
    &text[..end]
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkipTriviaOptions {

    pub stop_after_line_break: bool,

    pub stop_at_comments: bool,

    pub in_jsdoc: bool,
}

const MERGE_CONFLICT_MARKER_LENGTH: usize = 7;

fn is_conflict_marker_trivia(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    if pos + 1 >= text_len || bytes[pos + 1] != bytes[pos] {
        return false;
    }

    let mut at_line_start = pos == 0 || is_line_break(bytes[pos - 1] as char);
    if !at_line_start && pos >= 2 {

        at_line_start = is_line_break(bytes[pos - 2] as char);
    }
    if at_line_start && pos + MERGE_CONFLICT_MARKER_LENGTH < text_len {
        let ch = bytes[pos];
        for i in 0..MERGE_CONFLICT_MARKER_LENGTH {
            if bytes[pos + i] != ch {
                return false;
            }
        }

        return ch == b'=' || bytes[pos + MERGE_CONFLICT_MARKER_LENGTH] == b' ';
    }
    false
}

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

        while pos < text_len && !is_line_break(bytes[pos] as char) {
            pos += 1;
        }
    } else {

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

pub fn skip_trivia(text: &str, pos: usize) -> usize {
    skip_trivia_ex(text, pos, &SkipTriviaOptions::default(), None)
}

pub fn skip_trivia_ex(
    text: &str,
    pos: usize,
    options: &SkipTriviaOptions,
    report_error: Option<&dyn Fn(usize, usize)>,
) -> usize {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut pos = pos;

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

pub fn get_leading_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    iterate_comment_ranges(text, pos, false)
}

pub fn get_trailing_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    iterate_comment_ranges(text, pos, true)
}

fn iterate_comment_ranges(text: &str, pos: usize, trailing: bool) -> Vec<CommentRange> {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut pos = pos;
    let mut result: Vec<CommentRange> = Vec::new();

    let mut pending_pos: usize = 0;
    let mut pending_end: usize = 0;
    let mut pending_kind: CommentRangeKind = CommentRangeKind::SingleLine;
    let mut pending_has_trailing_new_line = false;
    let mut has_pending = false;

    let mut collecting = trailing;
    if pos == 0 {

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

        let mut s = Scanner::new("#name = 1");
        assert_eq!(s.scan(), SyntaxKind::PrivateIdentifier);
        assert_eq!(s.token_text(), "#name");
        assert_eq!(s.scan(), SyntaxKind::EqualsToken);
        assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(s.scan(), SyntaxKind::EndOfFile);
    }

    #[test]
    fn scan_shebang_at_file_start_is_trivia() {

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

        let mut s = Scanner::new(r#""\x22""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_text(), r#""\x22""#);
        assert_eq!(s.token_value(), "\"");

        let mut s = Scanner::new(r#""\u{1F600}""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "\u{1F600}");

        let mut s = Scanner::new(r#""\u0041""#);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "A");

        let mut s = Scanner::new("\"hello\\\nworld\"");
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);
        assert_eq!(s.token_value(), "helloworld");

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
        s.scan();
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

        let mut s = Scanner::new(r"/[\/]/");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), r"/[\/]/");
    }

    #[test]
    fn re_scan_slash_token_regex_with_escape() {

        let mut s = Scanner::new(r"/a\/b/");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), r"/a\/b/");
    }

    #[test]
    fn re_scan_slash_token_slash_equals() {

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

        let mut s = Scanner::new("/pattern/dgimsy");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/pattern/dgimsy");
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn re_scan_slash_token_unknown_flag_reports_ts1499() {

        let mut s = Scanner::new("/foo/zz");
        s.scan();
        s.re_scan_slash_token();
        assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
        assert_eq!(s.token_text(), "/foo/zz");
        let errors = s.take_errors();
        assert_eq!(errors.len(), 2, "expected two TS1499 errors for 'zz'");
        assert_eq!(errors[0].kind, DiagnosticKind::UnknownRegularExpressionFlag);
        assert_eq!(errors[0].pos, "/foo/".len());
        assert_eq!(errors[0].length, 1);
        assert_eq!(errors[1].kind, DiagnosticKind::UnknownRegularExpressionFlag);
        assert_eq!(errors[1].pos, "/foo/z".len());
    }

    #[test]
    fn re_scan_slash_token_duplicate_flag_reports_ts1500() {

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
        assert_eq!(errors[0].pos, "/foo/g".len());
        assert_eq!(errors[0].length, 1);
    }

    #[test]
    fn re_scan_slash_token_u_and_v_mutually_exclusive_reports_ts1502() {

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
        assert_eq!(errors[0].pos, "/foo/u".len());
        assert_eq!(errors[0].length, 1);

        let mut s = Scanner::new("/foo/vu");
        s.scan();
        s.re_scan_slash_token();
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind,
            DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive
        );
        assert_eq!(errors[0].pos, "/foo/v".len());
    }

    #[test]
    fn re_scan_slash_token_mixed_flag_errors() {

        let mut s = Scanner::new("/foo/guz");
        s.scan();
        s.re_scan_slash_token();
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::UnknownRegularExpressionFlag);
        assert_eq!(errors[0].pos, "/foo/gu".len());
    }

    #[test]
    fn comment_directive_ts_expect_error_single_line() {
        let mut s = Scanner::new("// @ts-expect-error\n");
        s.scan();
        let directives = s.comment_directives();
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].kind, CommentDirectiveKind::ExpectError);
        assert_eq!(directives[0].pos, 0);
        assert_eq!(directives[0].end, 19);
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

    #[test]
    fn skip_trivia_whitespace_and_newlines() {

        assert_eq!(skip_trivia("  \t\n  x", 0), 6);
        assert_eq!(skip_trivia("\n\n\nx", 0), 3);
        assert_eq!(skip_trivia("x", 0), 0);

        assert_eq!(skip_trivia("", 0), 0);
    }

    #[test]
    fn skip_trivia_single_line_comment() {

        assert_eq!(skip_trivia("// comment\nx", 0), 11);

        assert_eq!(skip_trivia("// eof", 0), 6);
    }

    #[test]
    fn skip_trivia_multi_line_comment() {

        assert_eq!(skip_trivia("/* comment */x", 0), 13);

        assert_eq!(skip_trivia("/* unterminated", 0), 15);

        assert_eq!(skip_trivia("abc", 0), 0);
    }

    #[test]
    fn skip_trivia_shebang_at_start() {
        assert_eq!(skip_trivia("#!/usr/bin/env node\nlet x;", 0), 20);

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

        let mut s = Scanner::new("let x = 1;");
        s.scan();
        assert_eq!(s.full_start_pos(), 0);
        assert_eq!(s.token_pos(), 0);
        assert_eq!(s.token(), SyntaxKind::LetKeyword);

        s.scan();
        assert_eq!(s.full_start_pos(), 3);
        assert_eq!(s.token_pos(), 4);
        assert_eq!(s.token(), SyntaxKind::Identifier);

        s.scan();
        assert_eq!(s.full_start_pos(), 5);
        assert_eq!(s.token_pos(), 6);
        assert_eq!(s.token(), SyntaxKind::EqualsToken);
    }

    #[test]
    fn full_start_pos_preserved_across_comments() {

        let mut s = Scanner::new("// hi\nlet x;");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert_eq!(s.full_start_pos(), 0);
        assert_eq!(s.token_pos(), 6);

        let mut s = Scanner::new("a /* c */ b");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::Identifier);
        assert_eq!(s.token_pos(), 0);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::Identifier);

        assert_eq!(s.full_start_pos(), 1);
        assert_eq!(s.token_pos(), 10);
    }

    #[test]
    fn get_leading_comment_ranges_basic() {

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

        let text = "let x; // trailing\n// leading for next\nlet y;";

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

        let text = "let x;\nlet y; // c\n";
        let ranges = get_trailing_comment_ranges(text, 0);
        assert!(ranges.is_empty());
    }

    #[test]
    fn get_trailing_comment_ranges_multi_line() {

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

        let text = "#!/usr/bin/env node\n// real comment\nlet x;";
        let ranges = get_leading_comment_ranges(text, 0);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].pos, 20);
        assert_eq!(ranges[0].end, 35);
        assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
        assert!(ranges[0].has_trailing_new_line);
    }

    #[test]
    fn token_flags_preceding_line_break_set() {
        let mut s = Scanner::new("foo\nbar");
        s.scan();
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_PRECEDING_LINE_BREAK
        ));
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_PRECEDING_LINE_BREAK
        ));
        assert!(s.has_preceding_line_break());
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

        let mut s = Scanner::new("123");
        s.scan();
        let flags = s.token_flags();
        assert_eq!(flags & TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS, TOKEN_FLAGS_NONE);
    }

    #[test]
    fn token_flags_reset_between_tokens() {

        let mut s = Scanner::new("0x1F 'str'");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_HEX_SPECIFIER
        ));
        s.scan();
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

        let mut s = Scanner::new("`abc");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_UNTERMINATED
        ));
    }

    #[test]
    fn skip_trivia_ex_stop_after_line_break() {

        let text = "  \n  x";
        let opts = SkipTriviaOptions {
            stop_after_line_break: true,
            ..Default::default()
        };
        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 3);

        assert_eq!(skip_trivia(text, 0), 5);
    }

    #[test]
    fn skip_trivia_ex_stop_at_comments() {

        let text = "  // c\nx";
        let opts = SkipTriviaOptions {
            stop_at_comments: true,
            ..Default::default()
        };

        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 2);

        assert_eq!(skip_trivia(text, 0), 7);
    }

    #[test]
    fn skip_trivia_ex_in_jsdoc_consumes_leading_asterisk() {

        let text = "\n * @param";
        let opts = SkipTriviaOptions {
            in_jsdoc: true,
            ..Default::default()
        };
        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 4);

        assert_eq!(skip_trivia(text, 0), 2);
    }

    #[test]
    fn skip_trivia_ex_jsdoc_star_only_after_line_break() {

        let text = " * foo";
        let opts = SkipTriviaOptions {
            in_jsdoc: true,
            ..Default::default()
        };

        assert_eq!(skip_trivia_ex(text, 0, &opts, None), 1);
    }

    #[test]
    fn is_conflict_marker_trivia_detects_markers() {

        assert!(is_conflict_marker_trivia("<<<<<<< head\n", 0));

        assert!(is_conflict_marker_trivia("x\n>>>>>>> branch\n", 2));

        assert!(is_conflict_marker_trivia("x\n=======\n", 2));

        assert!(is_conflict_marker_trivia("x\n||||||| base\n", 2));

        assert!(!is_conflict_marker_trivia("<<<<<< \n", 0));

        assert!(!is_conflict_marker_trivia("<<<<<<<x\n", 0));

        assert!(!is_conflict_marker_trivia("x\n|||||||\n", 2));

        assert!(!is_conflict_marker_trivia("a <<<<<<< \n", 2));

        assert!(!is_conflict_marker_trivia("<x\n", 0));
    }

    #[test]
    fn skip_trivia_ex_consumes_conflict_marker() {

        let text = "<<<<<<< a\nshared\n=======\n>>>>>>> b\nx";
        let pos = skip_trivia_ex(text, 0, &SkipTriviaOptions::default(), None);
        assert_eq!(&text[pos..], "shared\n=======\n>>>>>>> b\nx");
    }

    #[test]
    fn skip_trivia_ex_reports_conflict_marker_error() {

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

        let text = "<<<<<<< a\nlocal\n||||||| base\nshared\n=======\nremote\n>>>>>>> b\nx";
        let pos = skip_trivia_ex(text, 0, &SkipTriviaOptions::default(), None);
        assert_eq!(
            &text[pos..],
            "local\n||||||| base\nshared\n=======\nremote\n>>>>>>> b\nx"
        );
    }

    #[test]
    fn token_flags_preceding_jsdoc_comment() {

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

        let mut s = Scanner::new("/* not jsdoc */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(!s.has_preceding_jsdoc_comment());
    }

    #[test]
    fn token_flags_empty_jsdoc_comment_not_flagged() {

        let mut s = Scanner::new("/**/\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(!s.has_preceding_jsdoc_comment());
    }

    #[test]
    fn token_flags_jsdoc_deprecated_tag() {

        let mut s = Scanner::new("/**\n * @deprecated\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_comment());
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
        assert!(!s.has_preceding_jsdoc_with_see_or_link());
    }

    #[test]
    fn token_flags_jsdoc_see_tag() {

        let mut s = Scanner::new("/**\n * @see foo\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_see_or_link());
        assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_link_tag() {

        let mut s = Scanner::new("/**\n * {@link foo}\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_see_or_link());
    }

    #[test]
    fn token_flags_jsdoc_both_tags() {

        let mut s = Scanner::new("/**\n * @deprecated\n * @see foo\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
        assert!(s.has_preceding_jsdoc_with_see_or_link());
    }

    #[test]
    fn token_flags_jsdoc_tag_invalid_terminator() {

        let mut s = Scanner::new("/**\n * @deprecatedX\n */\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_tag_at_end_of_string() {

        let mut s = Scanner::new("/**@deprecated*/\nlet x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::LetKeyword);
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_flags_reset_between_tokens() {

        let mut s = Scanner::new("/** @deprecated */\nlet x\nlet y");
        s.scan();
        assert!(s.has_preceding_jsdoc_with_deprecated_tag());
        s.scan();
        s.scan();
        assert!(!s.has_preceding_jsdoc_comment());
        assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_consumed() {

        let mut s = Scanner::new("\n* x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();

        assert_eq!(s.token(), SyntaxKind::Identifier);
        assert_eq!(s.token_text(), "x");
        assert!(s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_no_line_break() {

        let mut s = Scanner::new("* x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_not_active() {

        let mut s = Scanner::new("\n* x");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_double_star_not_consumed() {

        let mut s = Scanner::new("\n** x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskAsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_star_equals_not_consumed() {

        let mut s = Scanner::new("\n*= x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskEqualsToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_only_first_consumed() {

        let mut s = Scanner::new("\n* * x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn token_flags_jsdoc_leading_asterisk_counter_nesting() {

        let mut s = Scanner::new("\n* x");
        s.set_skip_jsdoc_leading_asterisks(true);
        s.set_skip_jsdoc_leading_asterisks(false);
        s.scan();
        assert_eq!(s.token(), SyntaxKind::AsteriskToken);
        assert!(!s.has_preceding_jsdoc_leading_asterisks());
    }

    #[test]
    fn has_jsdoc_tag_helper() {

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

        assert!(!has_jsdoc_tag("deprecatedX", &["deprecated"]));
        assert!(!has_jsdoc_tag("dep", &["deprecated"]));
        assert!(!has_jsdoc_tag("foo", &["deprecated"]));
    }

    #[test]
    fn scan_jsdoc_comment_for_tags_helper() {

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

        assert!(token_flags_contains(
            scan_jsdoc_comment_for_tags("/** {@link foo} */"),
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
        ));
    }

    #[test]
    fn token_flags_unicode_escape() {

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

        let mut s = Scanner::new("\"\\01\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_escape_eight_nine_invalid() {

        let mut s = Scanner::new("\"\\8\"");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_nul_escape_not_invalid() {

        let mut s = Scanner::new("\"\\0\"");
        s.scan();
        assert!(!token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
        ));
    }

    #[test]
    fn token_flags_contains_separator_decimal() {

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

        let mut s = Scanner::new("0b1010_0101");
        s.scan();
        assert!(token_flags_contains(
            s.token_flags(),
            TOKEN_FLAGS_CONTAINS_SEPARATOR
        ));
    }

    #[test]
    fn token_flags_invalid_separator_consecutive() {

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

        let mut s = Scanner::new("'\\x41\\u0041'");
        s.scan();
        let flags = s.token_flags();
        assert!(token_flags_contains(flags, TOKEN_FLAGS_HEX_ESCAPE));
        assert!(token_flags_contains(flags, TOKEN_FLAGS_UNICODE_ESCAPE));
        assert!(token_flags_contains(flags, TOKEN_FLAGS_SINGLE_QUOTE));

        assert!(token_flags_intersects(
            flags,
            TOKEN_FLAGS_STRING_LITERAL_FLAGS
        ));
    }

    #[test]
    fn token_flags_numeric_literal_flags_mask() {

        let mut s = Scanner::new("0xFF_FF");
        s.scan();
        let flags = s.token_flags();
        assert!(token_flags_contains(flags, TOKEN_FLAGS_HEX_SPECIFIER));
        assert!(token_flags_contains(flags, TOKEN_FLAGS_CONTAINS_SEPARATOR));

        assert!(token_flags_intersects(
            flags,
            TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS
        ));
    }

    #[test]
    fn legacy_octal_literal_sets_octal_flag() {

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

        let mut s = Scanner::new("00");
        s.scan();
        assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);
    }

    #[test]
    fn leading_zero_non_octal_sets_leading_zero_flag() {

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

        let mut s = Scanner::new("0n");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::BigIntLiteral);
        assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        assert!(s.take_errors().is_empty());
    }

    #[test]
    fn zero_separator_after_leading_zero() {

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
        assert_eq!(errors[0].pos, 1);
    }

    #[test]
    fn legacy_octal_with_minus_prefix() {

        let mut s = Scanner::new("-0777");
        s.scan();
        assert_eq!(s.token(), SyntaxKind::MinusToken);
        s.scan();
        assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
        let errors = s.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);

        assert_eq!(errors[0].pos, 0);
        assert_eq!(errors[0].length, 5);
    }

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

        assert_eq!(s.token_text(), "code {@code x} more");
    }

    #[test]
    fn jsdoc_comment_text_token_empty_falls_through() {

        let mut s = Scanner::new("{");
        assert_eq!(
            s.scan_jsdoc_comment_text_token(false),
            SyntaxKind::OpenBraceToken
        );
    }

    #[test]
    fn scan_string_preserves_lone_surrogates() {

        let input = r#""🦀\ud7ff\ud800\ud801\uD83E\uDD80""#;
        let mut s = Scanner::new(input);
        assert_eq!(s.scan(), SyntaxKind::StringLiteral);

        let value = s.token_value();
        assert!(value.contains('🦀'));
        assert!(value.contains('\u{D7FF}'));

        let fffd_count = value.chars().filter(|&c| c == '\u{FFFD}').count();
        assert_eq!(fffd_count, 4);
    }
}
