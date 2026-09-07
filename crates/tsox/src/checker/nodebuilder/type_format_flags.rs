#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct TypeFormatFlags(u32);

impl TypeFormatFlags {
    pub const NONE: Self = Self(0);

    pub const WRITE_ARRAY_AS_GENERIC: Self = Self(1 << 1);

    pub const USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE: Self = Self(1 << 2);

    pub const ALLOW_UNIQUE_ES_SYMBOL_TYPE: Self = Self(1 << 3);

    pub const NO_TRUNCATION: Self = Self(1 << 7);

    pub const MULTILINE_OBJECT_LITERALS: Self = Self(1 << 8);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolDisplayPart {
    pub text: String,
    pub kind: DisplayPartKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayPartKind {
    Keyword,
    FunctionName,
    ClassName,
    InterfaceName,
    EnumName,
    TypeParameterName,
    ParameterName,
    PropertyName,
    VariableName,
    Punctuation,
    Space,
    LineBreak,
    Text,
    NumericLiteral,
    StringLiteral,
}

impl DisplayPartKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayPartKind::Keyword => "keyword",
            DisplayPartKind::FunctionName => "functionName",
            DisplayPartKind::ClassName => "className",
            DisplayPartKind::InterfaceName => "interfaceName",
            DisplayPartKind::EnumName => "enumName",
            DisplayPartKind::TypeParameterName => "typeParameterName",
            DisplayPartKind::ParameterName => "parameterName",
            DisplayPartKind::PropertyName => "propertyName",
            DisplayPartKind::VariableName => "variableName",
            DisplayPartKind::Punctuation => "punctuation",
            DisplayPartKind::Space => "space",
            DisplayPartKind::LineBreak => "lineBreak",
            DisplayPartKind::Text => "text",
            DisplayPartKind::NumericLiteral => "numericLiteral",
            DisplayPartKind::StringLiteral => "stringLiteral",
        }
    }
}

impl SymbolDisplayPart {
    pub fn new(text: impl Into<String>, kind: DisplayPartKind) -> Self {
        SymbolDisplayPart {
            text: text.into(),
            kind,
        }
    }
}

pub(crate) fn module_specifier_of_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    for ext in [
        ".d.ts", ".d.mts", ".d.cts", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
    ] {
        if let Some(stem) = base.strip_suffix(ext) {
            return stem.to_string();
        }
    }
    base.to_string()
}

pub(crate) fn is_keyword_type_name(name: &str) -> bool {
    matches!(
        name,
        "any"
            | "unknown"
            | "string"
            | "number"
            | "bigint"
            | "boolean"
            | "symbol"
            | "void"
            | "undefined"
            | "null"
            | "object"
            | "never"
            | "intrinsic"
            | "true"
            | "false"
    )
}

pub(crate) fn push_keyword(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Keyword));
}

pub(crate) fn push_space(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Space));
}

pub(crate) fn push_punctuation(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Punctuation));
}

pub(crate) fn push_part(parts: &mut Vec<SymbolDisplayPart>, text: &str, kind: DisplayPartKind) {
    parts.push(SymbolDisplayPart::new(text, kind));
}

pub(crate) fn display_kind_for_symbol(symbol: &Symbol) -> DisplayPartKind {
    let flags = symbol.flags;
    if flags.intersects(SymbolFlags::Function | SymbolFlags::Method) {
        DisplayPartKind::FunctionName
    } else if flags.intersects(SymbolFlags::Class) {
        DisplayPartKind::ClassName
    } else if flags.intersects(SymbolFlags::Interface) {
        DisplayPartKind::InterfaceName
    } else if flags.intersects(SymbolFlags::ENUM) {
        DisplayPartKind::EnumName
    } else if flags.intersects(SymbolFlags::TypeParameter) {
        DisplayPartKind::TypeParameterName
    } else if flags.intersects(SymbolFlags::Property | SymbolFlags::ACCESSOR) {
        DisplayPartKind::PropertyName
    } else if flags.intersects(SymbolFlags::EnumMember) {
        DisplayPartKind::PropertyName
    } else if flags.intersects(SymbolFlags::VARIABLE) {
        DisplayPartKind::VariableName
    } else {
        DisplayPartKind::Text
    }
}
