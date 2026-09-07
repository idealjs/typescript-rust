#![allow(unused_imports)]

use super::*;

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

pub(crate) static TEXT_TO_KEYWORD: OnceLock<HashMap<&'static str, SyntaxKind>> = OnceLock::new();

pub(crate) static TEXT_TO_TOKEN: OnceLock<HashMap<&'static str, SyntaxKind>> = OnceLock::new();

pub(crate) fn keywords() -> &'static HashMap<&'static str, SyntaxKind> {
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

pub(crate) fn punctuation() -> &'static HashMap<&'static str, SyntaxKind> {
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

pub(crate) static TOKEN_TO_TEXT: OnceLock<HashMap<SyntaxKind, &'static str>> = OnceLock::new();
