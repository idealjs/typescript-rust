#![allow(unused_imports)]

use super::*;

pub(crate) fn binary_precedence(token: SyntaxKind) -> u8 {
    match token {
        SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken => 1,
        SyntaxKind::AmpersandAmpersandToken => 2,
        SyntaxKind::BarToken => 3,
        SyntaxKind::CaretToken => 4,
        SyntaxKind::AmpersandToken => 5,
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken => 6,
        SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword => 7,
        SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => 8,
        SyntaxKind::PlusToken | SyntaxKind::MinusToken => 9,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken => 10,
        SyntaxKind::AsteriskAsteriskToken => 11,
        _ => 0,
    }
}

pub(crate) fn is_assignment_operator(token: SyntaxKind) -> bool {
    matches!(
        token,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
    )
}

pub(crate) fn is_keyword(token: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_keyword_kind(token)
}

pub(crate) fn is_identifier_or_keyword(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || token == SyntaxKind::PrivateIdentifier || is_keyword(token)
}

pub(crate) fn is_reserved_word_kind(token: SyntaxKind) -> bool {
    matches!(
        token,
        SyntaxKind::BreakKeyword
            | SyntaxKind::CaseKeyword
            | SyntaxKind::CatchKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::ContinueKeyword
            | SyntaxKind::DebuggerKeyword
            | SyntaxKind::DefaultKeyword
            | SyntaxKind::DeleteKeyword
            | SyntaxKind::DoKeyword
            | SyntaxKind::ElseKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::ExportKeyword
            | SyntaxKind::ExtendsKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::FinallyKeyword
            | SyntaxKind::ForKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::IfKeyword
            | SyntaxKind::ImportKeyword
            | SyntaxKind::InKeyword
            | SyntaxKind::InstanceOfKeyword
            | SyntaxKind::NewKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ReturnKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::SwitchKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::ThrowKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::TryKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::VarKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::WhileKeyword
            | SyntaxKind::WithKeyword
    )
}

pub(crate) const KEYWORD_SUGGESTIONS: &[&str] = &[
    "abstract",
    "accessor",
    "any",
    "as",
    "asserts",
    "bigint",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "continue",
    "const",
    "constenum",
    "constructor",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "global",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "instanceof",
    "interface",
    "intrinsic",
    "is",
    "keyof",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "package",
    "private",
    "protected",
    "public",
    "override",
    "out",
    "readonly",
    "return",
    "satisfies",
    "set",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "unique",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "async",
    "await",
    "of",
];
