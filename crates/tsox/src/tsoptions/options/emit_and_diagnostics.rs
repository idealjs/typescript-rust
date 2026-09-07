#![allow(unused_imports)]

use super::*;

pub(crate) const EMIT_AND_DIAGNOSTICS: &[OptionDecl] = &[
    OptionDecl {
        name: "exactOptionalPropertyTypes",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "esModuleInterop",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "allowSyntheticDefaultImports",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "allowJs",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "checkJs",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "composite",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        is_tsconfig_only: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "declaration",
        short_name: Some("d"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Generates corresponding '.d.ts' file.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "declarationMap",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "declarationDir",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "emitDeclarationOnly",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "sourceMap",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Generates corresponding '.map' file.",
        show_in_simplified_help: true,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "inlineSourceMap",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "inlineSources",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "removeComments",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "isolatedModules",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "isolatedDeclarations",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "verbatimModuleSyntax",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "preserveConstEnums",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "importHelpers",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "experimentalDecorators",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "emitDecoratorMetadata",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "forceConsistentCasingInFileNames",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "listFiles",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "listFilesOnly",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "listEmittedFiles",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "explainFiles",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "extendedDiagnostics",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "diagnostics",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "pretty",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "showConfig",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "ignoreConfig",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "locale",
        short_name: None,
        kind: OptionKind::String,
        is_file_path: false,
        is_command_line_only: true,
        extra_validation: ExtraValidation::Locale,
        ..DEFAULT_DECL
    },
];
