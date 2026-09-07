#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingContext {
    SourceElements,
    BlockStatements,
    SwitchClauses,
    SwitchClauseStatements,
    TypeMembers,
    ClassMembers,
    EnumMembers,
    HeritageClauseElement,
    VariableDeclarations,
    ObjectBindingElements,
    ArrayBindingElements,
    ArgumentExpressions,
    ObjectLiteralMembers,
    JsxAttributes,
    JsxChildren,
    ArrayLiteralMembers,
    Parameters,
    JSDocParameters,
    RestProperties,
    TypeParameters,
    TypeArguments,
    TupleElementTypes,
    HeritageClauses,
    ImportOrExportSpecifiers,
    ImportAttributes,
    JSDocComment,
}

pub struct Parser {
    pub(crate) scanner: Scanner,
    pub(crate) token: SyntaxKind,
    pub(crate) diagnostics: Vec<ParserDiagnostic>,
    pub(crate) language_variant: LanguageVariant,

    pub(crate) last_template_literal_was_middle: bool,

    pub(crate) yield_context: bool,

    pub(crate) await_context: bool,

    pub(crate) parsing_contexts: u32,
}

#[derive(Debug, Clone)]
pub struct ParserDiagnostic {
    pub message: Message,
    pub message_args: Vec<String>,
    pub range: TextRange,
}

pub fn script_kind_from_file_name(file_name: &str) -> ScriptKind {
    let ext = file_name.rfind('.').map(|i| &file_name[i..]).unwrap_or("");
    match ext {
        ".ts" | ".mts" | ".cts" => ScriptKind::Ts,
        ".tsx" => ScriptKind::Tsx,
        ".js" | ".mjs" | ".cjs" => ScriptKind::Js,
        ".jsx" => ScriptKind::Jsx,
        ".json" => ScriptKind::Json,
        _ => ScriptKind::Unknown,
    }
}
