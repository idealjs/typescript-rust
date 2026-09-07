#![allow(unused_imports)]

use super::*;

pub(crate) const RESOLUTION_AND_OUTPUT: &[OptionDecl] = &[
    OptionDecl {
        name: "target",
        short_name: Some("t"),
        kind: OptionKind::Enum,
        is_file_path: false,
        enum_values: Some(TARGET_ENUM_VALUES),
        description: "Specify ECMAScript target version.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "module",
        short_name: Some("m"),
        kind: OptionKind::Enum,
        is_file_path: false,
        enum_values: Some(MODULE_ENUM_VALUES),
        description: "Specify module code generation.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "moduleResolution",
        short_name: None,
        kind: OptionKind::Enum,
        is_file_path: false,
        enum_values: Some(MODULE_RESOLUTION_ENUM_VALUES),
        description: "Specify module resolution strategy.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "jsx",
        short_name: None,
        kind: OptionKind::Enum,
        is_file_path: false,
        enum_values: Some(JSX_ENUM_VALUES),
        description: "Specify JSX code generation.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "newLine",
        short_name: None,
        kind: OptionKind::Enum,
        is_file_path: false,
        enum_values: Some(NEW_LINE_ENUM_VALUES),
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "moduleDetection",
        short_name: None,
        kind: OptionKind::Enum,
        is_file_path: false,
        enum_values: Some(MODULE_DETECTION_ENUM_VALUES),
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "lib",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: false,
        description: "Specify library files to be included in the compilation.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "types",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "typeRoots",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "rootDirs",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: true,
        is_tsconfig_only: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "paths",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: false,
        is_tsconfig_only: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "outDir",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: true,
        description: "Redirect output structure to the directory.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "outFile",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: true,
        description: "Concatenate and emit output to single file.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "rootDir",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "baseUrl",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "tsBuildInfoFile",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "sourceRoot",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "mapRoot",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "jsxFactory",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "jsxFragmentFactory",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "jsxImportSource",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "reactNamespace",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "generateTrace",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "singleThreaded",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "quiet",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
];
