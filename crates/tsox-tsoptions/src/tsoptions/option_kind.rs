#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Boolean,
    String,
    Number,
    List,
    Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraValidation {
    None,
    Locale,

    MinValue,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionDecl {
    pub name: &'static str,
    pub short_name: Option<&'static str>,
    pub kind: OptionKind,
    pub is_file_path: bool,

    pub is_tsconfig_only: bool,

    pub is_command_line_only: bool,

    pub extra_validation: ExtraValidation,

    pub min_value: Option<i64>,

    pub enum_values: Option<&'static [&'static str]>,

    pub description: &'static str,

    pub show_in_simplified_help: bool,
}

pub(crate) const DEFAULT_DECL: OptionDecl = OptionDecl {
    name: "",
    short_name: None,
    kind: OptionKind::Boolean,
    is_file_path: false,
    is_tsconfig_only: false,
    is_command_line_only: false,
    extra_validation: ExtraValidation::None,
    min_value: None,
    enum_values: None,
    description: "",
    show_in_simplified_help: false,
};

pub(crate) static TARGET_ENUM_VALUES: &[&str] = &[
    "es3", "es5", "es6", "es2015", "es2016", "es2017", "es2018", "es2019", "es2020", "es2021",
    "es2022", "es2023", "es2024", "es2025", "esnext",
];
pub(crate) static MODULE_ENUM_VALUES: &[&str] = &[
    "commonjs", "amd", "system", "umd", "es6", "es2015", "es2020", "es2022", "esnext", "node16",
    "node18", "node20", "nodenext", "preserve",
];
pub(crate) static MODULE_RESOLUTION_ENUM_VALUES: &[&str] =
    &["node16", "nodenext", "bundler", "classic", "node", "node10"];
pub(crate) static JSX_ENUM_VALUES: &[&str] = &[
    "preserve",
    "react-native",
    "react-jsx",
    "react-jsxdev",
    "react",
];
pub(crate) static NEW_LINE_ENUM_VALUES: &[&str] = &["crlf", "lf"];
pub(crate) static MODULE_DETECTION_ENUM_VALUES: &[&str] = &["auto", "legacy", "force"];
