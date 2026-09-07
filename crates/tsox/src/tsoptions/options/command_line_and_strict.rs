#![allow(unused_imports)]

use super::*;

pub(crate) const COMMAND_LINE_AND_STRICT: &[OptionDecl] = &[
    OptionDecl {
        name: "help",
        short_name: Some("h"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Print this message.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "all",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Show all compiler options.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "version",
        short_name: Some("v"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Print the compiler's version.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "init",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Initializes a TypeScript project and creates a tsconfig.json file.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "project",
        short_name: Some("p"),
        kind: OptionKind::String,
        is_file_path: true,
        description: "Compile the project given the path to its configuration file, or to a folder with a 'tsconfig.json'.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "build",
        short_name: Some("b"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Build one or more projects and their dependencies, if out of date.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "watch",
        short_name: Some("w"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Watch input files.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "incremental",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noEmit",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Do not emit outputs.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noCheck",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Disable type checking.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noLib",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "skipLibCheck",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Skip type checking of declaration files.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "skipDefaultLibCheck",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "strict",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Enable all strict type-checking options.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "strictNullChecks",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "strictFunctionTypes",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "strictBindCallApply",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "strictPropertyInitialization",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "strictBuiltinIteratorReturn",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noImplicitAny",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noImplicitThis",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noImplicitOverride",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noUnusedLocals",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noUnusedParameters",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noFallthroughCasesInSwitch",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noUncheckedIndexedAccess",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noPropertyAccessFromIndexSignature",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noErrorTruncation",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noEmitOnError",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "noResolve",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "useUnknownInCatchVariables",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
];
