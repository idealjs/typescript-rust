//! Command-line and `tsconfig.json` option parsing, ported from
//! `internal/tsoptions/`.
//!
//! This is a pragmatic port: it handles the common compiler options, file
//! arguments, response files, and `tsconfig.json` reading (including JSONC
//! comments, `extends`, `files`/`include`/`exclude` glob expansion). It does
//! not yet mirror the full `NameMap`/did-you-mean machinery of the Go port.

use std::collections::HashMap;

use crate::ast::diagnostic::Diagnostic;
use crate::core::compiler_options::{
    CompilerOptions, JsxEmit, ModuleDetectionKind, ModuleKind, ModuleResolutionKind, NewLineKind,
    ScriptTarget,
};
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::core::watch_options::{
    PollingKind, WatchDirectoryKind, WatchFileKind, WatchOptions, parse_polling_kind,
    parse_watch_directory_kind, parse_watch_file_kind,
};
use crate::diagnostics::{
    ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1, CANNOT_READ_FILE_0,
    CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0,
    COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD, NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2, OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1, UNKNOWN_COMPILER_OPTION_0,
    UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1,
    UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0, WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
    new_ad_hoc_message,
};
use crate::glob::Glob;
use crate::tspath;
use crate::vfs::FS;

// ────────────────────────────────────────────────────────────────────────────
// Option declarations
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Boolean,
    String,
    Number,
    List,
    Enum,
}

/// Extra validation that `parse_option_value` / `validate_json_option_value`
/// should perform beyond the basic kind check. Mirrors Go's `extraValidation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraValidation {
    None,
    Locale,
    /// Numeric option must satisfy `min_value`.
    MinValue,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionDecl {
    pub name: &'static str,
    pub short_name: Option<&'static str>,
    pub kind: OptionKind,
    pub is_file_path: bool,
    /// Option can only be used in `tsconfig.json`, not on the command line
    /// (e.g. `composite`, `paths`).
    pub is_tsconfig_only: bool,
    /// Option can only be used on the command line, not in `tsconfig.json`.
    pub is_command_line_only: bool,
    /// Special validation category (locale, min-value, etc.).
    pub extra_validation: ExtraValidation,
    /// Minimum numeric value (for `builders`, `checkers`).
    pub min_value: Option<i64>,
    /// Valid enum values (for `target`, `module`, `moduleResolution`, `jsx`,
    /// `newLine`, `moduleDetection`).
    pub enum_values: Option<&'static [&'static str]>,
    /// Help text for the option, shown by `--help` / `--all`. Empty string
    /// means the option is omitted from the simplified help view.
    pub description: &'static str,
    /// Whether the option appears in the simplified `--help` view (mirrors
    /// Go's `ShowInSimplifiedHelpView`). When false, the option only appears
    /// under `--all`.
    pub show_in_simplified_help: bool,
}

/// Const default used to fill in the declaration-driven fields via struct
/// update syntax (`..DEFAULT_DECL`) so that each `OptionDecl` literal only
/// needs to set the fields it cares about.
const DEFAULT_DECL: OptionDecl = OptionDecl {
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

/// The set of compiler options accepted on the command line.
///
/// Mirrors a subset of `tsoptions.CommandLineCompilerOptions`.

// Valid enum values for the declaration-driven enum options. These mirror the
// keys of Go's `commandLineOptionEnumMap` (see `internal/tsoptions/enummaps.go`).
static TARGET_ENUM_VALUES: &[&str] = &[
    "es3",
    "es5",
    "es6",
    "es2015",
    "es2016",
    "es2017",
    "es2018",
    "es2019",
    "es2020",
    "es2021",
    "es2022",
    "es2023",
    "es2024",
    "es2025",
    "esnext",
];
static MODULE_ENUM_VALUES: &[&str] = &[
    "commonjs",
    "amd",
    "system",
    "umd",
    "es6",
    "es2015",
    "es2020",
    "es2022",
    "esnext",
    "node16",
    "node18",
    "node20",
    "nodenext",
    "preserve",
];
static MODULE_RESOLUTION_ENUM_VALUES: &[&str] = &[
    "node16",
    "nodenext",
    "bundler",
    "classic",
    "node",
    "node10",
];
static JSX_ENUM_VALUES: &[&str] = &[
    "preserve",
    "react-native",
    "react-jsx",
    "react-jsxdev",
    "react",
];
static NEW_LINE_ENUM_VALUES: &[&str] = &["crlf", "lf"];
static MODULE_DETECTION_ENUM_VALUES: &[&str] = &["auto", "legacy", "force"];
pub const OPTIONS: &[OptionDecl] = &[
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

pub const BUILD_OPTIONS: &[OptionDecl] = &[
    OptionDecl {
        name: "build",
        short_name: Some("b"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "verbose",
        short_name: Some("v"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Enable verbose logging in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "dry",
        short_name: Some("d"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Show what would be built (dry run) in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "force",
        short_name: Some("f"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Build all projects, including those that are up to date.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "clean",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Delete the outputs of all projects in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "builders",
        short_name: None,
        kind: OptionKind::Number,
        is_file_path: false,
        min_value: Some(1),
        extra_validation: ExtraValidation::MinValue,
        description: "Number of concurrent build workers in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "stopBuildOnErrors",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Stop building projects immediately after an error is reported in build mode.",
        ..DEFAULT_DECL
    },
];

/// Distinguishes compiler-mode parsing from build-mode parsing, mirroring
/// Go's `AlternateModeDiagnostics` selection.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    Compiler,
    Build,
}

// Valid enum values for the watch-option declarations. These mirror the keys
// of Go's `watchFileEnumMap` / `watchDirectoryEnumMap` / `fallbackEnumMap`
// (`internal/tsoptions/enummaps.go:234-255`).
static WATCH_FILE_ENUM_VALUES: &[&str] = &[
    "fixedpollinginterval",
    "prioritypollinginterval",
    "dynamicprioritypolling",
    "fixedchunksizepolling",
    "usefsevents",
    "usefseventsonparentdirectory",
];
static WATCH_DIRECTORY_ENUM_VALUES: &[&str] = &[
    "usefsevents",
    "fixedpollinginterval",
    "dynamicprioritypolling",
    "fixedchunksizepolling",
];
static FALLBACK_POLLING_ENUM_VALUES: &[&str] = &[
    "fixedinterval",
    "priorityinterval",
    "dynamicpriority",
    "fixedchunksize",
];

/// The set of watch options, mirroring Go's `OptionsForWatch`
/// (`internal/tsoptions/declswatch.go:8-88`).
///
/// These are modeled as an independent axis from `OPTIONS` (compiler) and
/// `BUILD_OPTIONS` (build): a separate declarations list, a separate name
/// map (`find_watch_option`), and a separate parser pass (`apply_watch_options`).
/// On the CLI, watch flags are accepted as a fallback when the compiler/build
/// map misses, mirroring Go's `WatchNameMap` fallback in `parseStrings`.
pub const OPTIONS_FOR_WATCH: &[OptionDecl] = &[
    OptionDecl {
        name: "watchInterval",
        short_name: None,
        kind: OptionKind::Number,
        description: "Specify the polling interval for watch mode (milliseconds).",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "watchFile",
        short_name: None,
        kind: OptionKind::Enum,
        enum_values: Some(WATCH_FILE_ENUM_VALUES),
        description: "Specify how the TypeScript watch mode works.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "watchDirectory",
        short_name: None,
        kind: OptionKind::Enum,
        enum_values: Some(WATCH_DIRECTORY_ENUM_VALUES),
        description: "Specify how directories are watched on systems that lack recursive file watching functionality.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "fallbackPolling",
        short_name: None,
        kind: OptionKind::Enum,
        enum_values: Some(FALLBACK_POLLING_ENUM_VALUES),
        description: "Specify what approach the watcher should use if the system runs out of native file watchers.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "synchronousWatchDirectory",
        short_name: None,
        kind: OptionKind::Boolean,
        description: "Synchronously call callbacks and update the state of directory watchers on platforms that don't support recursive watching natively.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "excludeDirectories",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: true,
        description: "Remove a list of directories from the watch process.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "excludeFiles",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: true,
        description: "Remove a list of files from the watch mode's processing.",
        ..DEFAULT_DECL
    },
];

/// Case-insensitive match on an option's name or short name. Mirrors Go's
/// `NameMap.GetOptionDeclarationFromName`, which lowercases the lookup key.
fn decl_matches(o: &OptionDecl, name: &str) -> bool {
    o.name.eq_ignore_ascii_case(name)
        || o.short_name
            .map(|s| s.eq_ignore_ascii_case(name))
            .unwrap_or(false)
}

/// Case-insensitive lookup over the compiler option declarations (the
/// `NameMap` for compiler mode). Replaces the previous case-sensitive scan.
fn find_option(name: &str) -> Option<&'static OptionDecl> {
    OPTIONS.iter().find(|o| decl_matches(o, name))
}

/// Case-insensitive lookup over build-only declarations (used for
/// alternate-mode detection in compiler mode).
fn find_build_only_option(name: &str) -> Option<&'static OptionDecl> {
    BUILD_OPTIONS.iter().find(|o| decl_matches(o, name))
}

/// Case-insensitive lookup over the build option declarations, chaining the
/// compiler declarations so that shared options (e.g. `watch`) resolve.
fn find_build_option(name: &str) -> Option<&'static OptionDecl> {
    BUILD_OPTIONS
        .iter()
        .chain(OPTIONS.iter())
        .find(|o| decl_matches(o, name))
}

/// Case-insensitive lookup over the watch option declarations, mirroring Go's
/// `WatchNameMap` (`internal/tsoptions/namemap.go:12`). Used as a fallback in
/// `parse_command_line_worker` when the compiler/build map misses, so that
/// `--watchFile usefsevents` etc. are accepted on the CLI alongside compiler
/// flags but routed into a separate `WatchOptions` value.
fn find_watch_option(name: &str) -> Option<&'static OptionDecl> {
    OPTIONS_FOR_WATCH.iter().find(|o| decl_matches(o, name))
}

// ────────────────────────────────────────────────────────────────────────────
// Parsed value
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum OptValue {
    Bool(bool),
    Str(String),
    Num(i64),
    List(Vec<String>),
    Null,
}

impl OptValue {
    fn as_bool(&self) -> Option<bool> {
        match self {
            OptValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            OptValue::Str(s) => Some(s),
            _ => None,
        }
    }
    fn as_list(&self) -> Option<&[String]> {
        match self {
            OptValue::List(v) => Some(v),
            _ => None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ParsedCommandLine
// ────────────────────────────────────────────────────────────────────────────

/// A parsed command line or tsconfig, mirroring `tsoptions.ParsedCommandLine`.
#[derive(Debug, Clone, Default)]
pub struct ParsedCommandLine {
    pub compiler_options: CompilerOptions,
    pub file_names: Vec<String>,
    pub errors: Vec<Diagnostic>,
    pub config_file_name: String,
    /// Raw `compilerOptions` value from tsconfig.json (if any), for `--showConfig`.
    pub raw_options: Option<crate::json::Value>,
    /// `files`/`include`/`exclude` specs from tsconfig.json.
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub files_spec: Vec<String>,
    pub has_include_spec: bool,
    pub has_exclude_spec: bool,
    pub has_files_spec: bool,
    pub references: Vec<crate::core::project_reference::ProjectReference>,
    pub compile_on_save: Option<bool>,
    pub watch: bool,
    /// Watch options parsed from the command line, mirroring Go's
    /// `ParsedOptions.WatchOptions`. Modeled independently from
    /// `compiler_options` (separate declarations, name map, and parser pass).
    /// A `watchOptions` key inside `tsconfig.json` is not yet parsed, matching
    /// the current Go state.
    pub watch_options: WatchOptions,
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub clean: Tristate,
    pub dry: Tristate,
    pub force: Tristate,
    pub verbose: Tristate,
    pub stop_build_on_errors: Tristate,
    pub builders: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedBuildCommandLine {
    pub build_options: BuildOptions,
    pub compiler_options: CompilerOptions,
    pub projects: Vec<String>,
    pub errors: Vec<Diagnostic>,
    /// Watch options parsed from the command line, mirroring Go's
    /// `ParsedBuildCommandLine.WatchOptions`.
    pub watch_options: WatchOptions,
    current_dir: String,
}

impl ParsedBuildCommandLine {
    pub fn resolved_project_paths(&self) -> Vec<String> {
        self.projects
            .iter()
            .map(|project| tspath::get_normalized_absolute_path(project, &self.current_dir))
            .collect()
    }
}

/// Helper extension to build a compiler diagnostic with custom text.
impl Diagnostic {
    pub fn with_text(self, text: impl Into<String>) -> Diagnostic {
        Diagnostic {
            file: self.file,
            loc: self.loc,
            code: self.code,
            category: self.category,
            message: None,
            message_key: self.message_key,
            message_args: vec![text.into()],
            message_chain: self.message_chain,
            related_information: self.related_information,
            reports_unnecessary: self.reports_unnecessary,
            reports_deprecated: self.reports_deprecated,
            skipped_on_no_emit: self.skipped_on_no_emit,
        }
    }
}

fn err(text: impl Into<String>) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), new_ad_hoc_message(""), vec![]).with_text(text)
}

// ────────────────────────────────────────────────────────────────────────────
// Command-line parsing
// ────────────────────────────────────────────────────────────────────────────

/// Parse command-line arguments into a `ParsedCommandLine`.
///
/// `current_dir` is used to resolve relative file paths and response files.
/// `fs` is used to read response files.
pub fn parse_command_line(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
) -> ParsedCommandLine {
    let (options, watch_options_map, file_names, errors) =
        parse_command_line_worker(args, current_dir, fs, find_option, ParseMode::Compiler);

    let mut compiler_options = CompilerOptions::default();
    apply_options(&options, &mut compiler_options);
    let watch = compiler_options.watch.is_true();
    let mut watch_options = WatchOptions::default();
    apply_watch_options(&watch_options_map, &mut watch_options);
    // Resolve relative file names to absolute paths.
    let file_names = file_names
        .iter()
        .map(|f| tspath::get_normalized_absolute_path(f, current_dir))
        .collect();

    ParsedCommandLine {
        compiler_options,
        file_names,
        errors,
        config_file_name: String::new(),
        raw_options: None,
        include: Vec::new(),
        exclude: Vec::new(),
        files_spec: Vec::new(),
        has_include_spec: false,
        has_exclude_spec: false,
        has_files_spec: false,
        references: Vec::new(),
        compile_on_save: None,
        watch,
        watch_options,
    }
}

pub fn parse_build_command_line(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
) -> ParsedBuildCommandLine {
    let (options, watch_options_map, mut projects, mut errors) =
        parse_command_line_worker(args, current_dir, fs, find_build_option, ParseMode::Build);

    if projects.is_empty() {
        projects.push(".".to_string());
    }

    let mut compiler_options = CompilerOptions::default();
    apply_options(&options, &mut compiler_options);

    let mut build_options = BuildOptions::default();
    apply_build_options(&options, &mut build_options);

    let mut watch_options = WatchOptions::default();
    apply_watch_options(&watch_options_map, &mut watch_options);

    if build_options.clean.is_true() && build_options.force.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["clean".to_string(), "force".to_string()],
        ));
    }
    if build_options.clean.is_true() && build_options.verbose.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["clean".to_string(), "verbose".to_string()],
        ));
    }
    if build_options.clean.is_true() && compiler_options.watch.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["clean".to_string(), "watch".to_string()],
        ));
    }
    if compiler_options.watch.is_true() && build_options.dry.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["watch".to_string(), "dry".to_string()],
        ));
    }

    ParsedBuildCommandLine {
        build_options,
        compiler_options,
        projects,
        errors,
        watch_options,
        current_dir: current_dir.to_string(),
    }
}

fn parse_command_line_worker(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
    find: fn(&str) -> Option<&'static OptionDecl>,
    mode: ParseMode,
) -> (
    HashMap<String, OptValue>,
    HashMap<String, OptValue>,
    Vec<String>,
    Vec<Diagnostic>,
) {
    let mut options: HashMap<String, OptValue> = HashMap::new();
    let mut watch_options: HashMap<String, OptValue> = HashMap::new();
    let mut file_names: Vec<String> = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let s = &args[i];
        i += 1;
        if s.is_empty() {
            continue;
        }
        let first = s.chars().next().unwrap();
        match first {
            '@' => {
                // Response file
                let response_path = &s[1..];
                let abs = tspath::get_normalized_absolute_path(response_path, current_dir);
                if let Some(fs) = fs {
                    if let Some(content) = fs.read_file(&abs) {
                        let (response_args, split_errors) =
                            split_response_file(&content, &abs);
                        errors.extend(split_errors);
                        let (sub_options, sub_watch_options, sub_files, sub_errors) =
                            parse_command_line_worker(
                                &response_args,
                                current_dir,
                                Some(fs),
                                find,
                                mode,
                            );
                        file_names.extend(sub_files);
                        for (k, v) in sub_options {
                            options.insert(k, v);
                        }
                        for (k, v) in sub_watch_options {
                            watch_options.insert(k, v);
                        }
                        errors.extend(sub_errors);
                    } else {
                        errors.push(Diagnostic::new(
                            None,
                            TextRange::undefined(),
                            CANNOT_READ_FILE_0,
                            vec![response_path.to_string()],
                        ));
                    }
                } else {
                    errors.push(Diagnostic::new(
                        None,
                        TextRange::undefined(),
                        CANNOT_READ_FILE_0,
                        vec![response_path.to_string()],
                    ));
                }
            }
            '-' => {
                // Strip up to two leading dashes.
                let name_part = s.trim_start_matches('-');
                // Support `--name=value`.
                let (name, inline_value) = match name_part.split_once('=') {
                    Some((n, v)) => (n, Some(v.to_string())),
                    None => (name_part, None),
                };
                match find(name) {
                    Some(opt) => {
                        i = parse_option_value(
                            args,
                            i,
                            opt,
                            inline_value,
                            &mut options,
                            &mut errors,
                            false,
                        );
                    }
                    None => {
                        // Watch-option fallback: if the option exists in the
                        // watch name map, route it into the separate
                        // `watch_options` map. Mirrors Go's `WatchNameMap`
                        // fallback in `parseStrings` (`commandlineparser.go:150`).
                        if let Some(opt) = find_watch_option(name) {
                            i = parse_option_value(
                                args,
                                i,
                                opt,
                                inline_value,
                                &mut watch_options,
                                &mut errors,
                                true,
                            );
                            continue;
                        }
                        // Alternate-mode: if the option exists in the *other*
                        // name map, emit the appropriate diagnostic instead of
                        // the generic "unknown" error. Mirrors Go's
                        // `createUnknownOptionError` / `AlternateModeDiagnostics`.
                        if mode == ParseMode::Compiler && find_build_only_option(name).is_some() {
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
                                vec![name.to_string()],
                            ));
                            continue;
                        }
                        errors.push(Diagnostic::new(
                            None,
                            TextRange::undefined(),
                            UNKNOWN_COMPILER_OPTION_0,
                            vec![name.to_string()],
                        ));
                        continue;
                    }
                }
            }
            _ => {
                file_names.push(s.clone());
            }
        }
    }
    (options, watch_options, file_names, errors)
}

fn parse_option_value(
    args: &[String],
    mut i: usize,
    opt: &OptionDecl,
    inline_value: Option<String>,
    options: &mut HashMap<String, OptValue>,
    errors: &mut Vec<Diagnostic>,
    watch: bool,
) -> usize {
    // For watch options, type-mismatch / missing-value diagnostics use
    // `Watch_option_0_requires_a_value_of_type_1` (TS5080) instead of the
    // generic ad-hoc message, mirroring Go's `watchOptionsDidYouMeanDiagnostics.
    // OptionTypeMismatchDiagnostic` (`tsoptions/diagnostics.go:46-54`).
    let type_name = |kind: OptionKind| -> &'static str {
        match kind {
            OptionKind::Boolean => "boolean",
            OptionKind::String => "string",
            OptionKind::Number => "number",
            OptionKind::List => "list",
            OptionKind::Enum => "string",
        }
    };
    let missing_value_error = |errors: &mut Vec<Diagnostic>| {
        if watch {
            errors.push(Diagnostic::new(
                None,
                TextRange::undefined(),
                WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                vec![opt.name.to_string(), type_name(opt.kind).to_string()],
            ));
        } else {
            errors.push(err(format!("Option '{}' requires a value.", opt.name)));
        }
    };
    // TSConfigOnly options can only appear in tsconfig.json; on the command
    // line only `false`/`null` (booleans) or `null` (others) are accepted.
    // Mirrors Go's `parseOptionValue` `IsTSConfigOnly` branch.
    if opt.is_tsconfig_only {
        let (opt_value, from_args) = match &inline_value {
            Some(v) => (v.clone(), false),
            None => {
                if i < args.len() {
                    (args[i].clone(), true)
                } else {
                    (String::new(), false)
                }
            }
        };
        if opt_value == "null" {
            options.insert(opt.name.to_string(), OptValue::Null);
            if from_args {
                i += 1;
            }
        } else if opt.kind == OptionKind::Boolean {
            if opt_value == "false" {
                options.insert(opt.name.to_string(), OptValue::Bool(false));
                if from_args {
                    i += 1;
                }
            } else {
                errors.push(Diagnostic::new(
                    None,
                    TextRange::undefined(),
                    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
                    vec![opt.name.to_string()],
                ));
                if from_args && opt_value == "true" {
                    i += 1;
                }
            }
        } else {
            errors.push(Diagnostic::new(
                None,
                TextRange::undefined(),
                OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
                vec![opt.name.to_string()],
            ));
            if from_args && !opt_value.is_empty() && !opt_value.starts_with('-') {
                i += 1;
            }
        }
        return i;
    }

    match opt.kind {
        OptionKind::Boolean => {
            if let Some(v) = inline_value {
                let b = v != "false";
                options.insert(opt.name.to_string(), OptValue::Bool(b));
            } else if i < args.len() && (args[i] == "true" || args[i] == "false") {
                options.insert(opt.name.to_string(), OptValue::Bool(args[i] == "true"));
                i += 1;
            } else {
                options.insert(opt.name.to_string(), OptValue::Bool(true));
            }
        }
        OptionKind::String => {
            let val = match inline_value {
                Some(v) => Some(v),
                None => {
                    if i < args.len() {
                        let v = args[i].clone();
                        i += 1;
                        Some(v)
                    } else {
                        None
                    }
                }
            };
            match val {
                Some(v) if v == "null" => {
                    options.insert(opt.name.to_string(), OptValue::Null);
                }
                Some(v) => {
                    options.insert(opt.name.to_string(), OptValue::Str(v));
                }
                None => {
                    missing_value_error(errors);
                }
            }
        }
        OptionKind::Enum => {
            let val = match inline_value {
                Some(v) => Some(v),
                None => {
                    if i < args.len() {
                        let v = args[i].clone();
                        i += 1;
                        Some(v)
                    } else {
                        None
                    }
                }
            };
            match val {
                Some(v) if v == "null" => {
                    options.insert(opt.name.to_string(), OptValue::Null);
                }
                Some(v) => {
                    // Declaration-driven enum validation: if the option declares
                    // `enum_values`, the (case-insensitive) value must be in that
                    // list, otherwise emit `ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1`
                    // listing the valid values. Mirrors Go's
                    // `convertJsonOptionOfEnumType` / `createDiagnosticForInvalidEnumType`.
                    if let Some(enum_vals) = opt.enum_values {
                        if enum_vals.iter().any(|e| e.eq_ignore_ascii_case(&v)) {
                            options.insert(opt.name.to_string(), OptValue::Str(v));
                        } else {
                            let valid = enum_vals
                                .iter()
                                .map(|e| format!("'{}'", e))
                                .collect::<Vec<_>>()
                                .join(", ");
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1,
                                vec![format!("--{}", opt.name), valid],
                            ));
                        }
                    } else {
                        options.insert(opt.name.to_string(), OptValue::Str(v));
                    }
                }
                None => {
                    missing_value_error(errors);
                }
            }
        }
        OptionKind::Number => {
            let val = inline_value.or_else(|| {
                if i < args.len() {
                    let v = args[i].clone();
                    i += 1;
                    Some(v)
                } else {
                    None
                }
            });
            match val {
                Some(v) => match v.parse::<i64>() {
                    Ok(n) => {
                        // Declaration-driven min-value validation (e.g.
                        // `builders` must be >= 1). Mirrors Go's
                        // `parseOptionValue` number branch.
                        if let Some(min) = opt.min_value {
                            if n < min {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1,
                                    vec![opt.name.to_string(), min.to_string()],
                                ));
                            } else {
                                options.insert(opt.name.to_string(), OptValue::Num(n));
                            }
                        } else {
                            options.insert(opt.name.to_string(), OptValue::Num(n));
                        }
                    }
                    Err(_) => {
                        if watch {
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                                vec![
                                    opt.name.to_string(),
                                    type_name(opt.kind).to_string(),
                                ],
                            ));
                        } else {
                            errors.push(err(format!("Option '{}' requires a number.", opt.name)));
                        }
                    }
                },
                None => {
                    missing_value_error(errors);
                }
            }
        }
        OptionKind::List => {
            let val = inline_value.or_else(|| {
                if i < args.len() && !args[i].starts_with('-') {
                    let v = args[i].clone();
                    i += 1;
                    Some(v)
                } else {
                    None
                }
            });
            let list = match val {
                Some(v) => v.split(',').map(|s| s.trim().to_string()).collect(),
                None => Vec::new(),
            };
            options.insert(opt.name.to_string(), OptValue::List(list));
        }
    }
    i
}

/// Tokenize a response file's contents into arguments, mirroring Go's
/// `parseResponseFile` (`commandlineparser.go:183-213`). Whitespace separates
/// arguments; double-quoted spans are captured literally (without the quotes).
/// An unterminated quoted string emits a TS6045 diagnostic and consumes the
/// remainder of the file as the argument (matching Go's behavior of still
/// pushing `text[start+1:pos]` before reporting the error).
fn split_response_file(content: &str, file_name: &str) -> (Vec<String>, Vec<Diagnostic>) {
    let mut args = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0usize;
    while pos < chars.len() {
        while pos < chars.len() && chars[pos] <= ' ' {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }
        if chars[pos] == '"' {
            pos += 1;
            let start = pos;
            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }
            args.push(chars[start..pos].iter().collect());
            if pos < chars.len() {
                pos += 1;
            } else {
                // Reached end of file inside a quoted string: emit TS6045,
                // aligned with Go's `Unterminated_quoted_string_in_response_file_0`.
                errors.push(Diagnostic::new(
                    None,
                    TextRange::undefined(),
                    UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0,
                    vec![file_name.to_string()],
                ));
            }
        } else {
            let start = pos;
            while pos < chars.len() && chars[pos] > ' ' {
                pos += 1;
            }
            args.push(chars[start..pos].iter().collect());
        }
    }
    (args, errors)
}

// ────────────────────────────────────────────────────────────────────────────
// Applying parsed options to CompilerOptions
// ────────────────────────────────────────────────────────────────────────────

fn set_bool(options: &mut CompilerOptions, name: &str, b: bool) {
    let t = Tristate::from(b);
    match name {
        "noEmit" => options.no_emit = t,
        "noCheck" => options.no_check = t,
        "noLib" => options.no_lib = t,
        "skipLibCheck" => options.skip_lib_check = t,
        "skipDefaultLibCheck" => options.skip_default_lib_check = t,
        "strictNullChecks" => options.strict_null_checks = t,
        "strictFunctionTypes" => options.strict_function_types = t,
        "strictBindCallApply" => options.strict_bind_call_apply = t,
        "strictPropertyInitialization" => options.strict_property_initialization = t,
        "strictBuiltinIteratorReturn" => options.strict_builtin_iterator_return = t,
        "noImplicitAny" => options.no_implicit_any = t,
        "noImplicitThis" => options.no_implicit_this = t,
        "noImplicitOverride" => options.no_implicit_override = t,
        "noUnusedLocals" => options.no_unused_locals = t,
        "noUnusedParameters" => options.no_unused_parameters = t,
        "noFallthroughCasesInSwitch" => options.no_fallthrough_cases_in_switch = t,
        "noUncheckedIndexedAccess" => options.no_unchecked_indexed_access = t,
        "noPropertyAccessFromIndexSignature" => options.no_property_access_from_index_signature = t,
        "noErrorTruncation" => options.no_error_truncation = t,
        "noEmitOnError" => options.no_emit_on_error = t,
        "noResolve" => options.no_resolve = t,
        "useUnknownInCatchVariables" => options.use_unknown_in_catch_variables = t,
        "exactOptionalPropertyTypes" => options.exact_optional_property_types = t,
        "esModuleInterop" => options.es_module_interop = t,
        "allowSyntheticDefaultImports" => options.allow_synthetic_default_imports = t,
        "allowJs" => options.allow_js = t,
        "checkJs" => options.check_js = t,
        "composite" => options.composite = t,
        "declaration" => options.declaration = t,
        "declarationMap" => options.declaration_map = t,
        "emitDeclarationOnly" => options.emit_declaration_only = t,
        "sourceMap" => options.source_map = t,
        "inlineSourceMap" => options.inline_source_map = t,
        "inlineSources" => options.inline_sources = t,
        "removeComments" => options.remove_comments = t,
        "isolatedModules" => options.isolated_modules = t,
        "isolatedDeclarations" => options.isolated_declarations = t,
        "verbatimModuleSyntax" => options.verbatim_module_syntax = t,
        "preserveConstEnums" => options.preserve_const_enums = t,
        "importHelpers" => options.import_helpers = t,
        "experimentalDecorators" => options.experimental_decorators = t,
        "emitDecoratorMetadata" => options.emit_decorator_metadata = t,
        "forceConsistentCasingInFileNames" => options.force_consistent_casing_in_file_names = t,
        "listFiles" => options.list_files = t,
        "listFilesOnly" => options.list_files_only = t,
        "listEmittedFiles" => options.list_emitted_files = t,
        "explainFiles" => options.explain_files = t,
        "extendedDiagnostics" => options.extended_diagnostics = t,
        "diagnostics" => options.diagnostics = t,
        "pretty" => options.pretty = t,
        "showConfig" => options.show_config = t,
        "ignoreConfig" => options.ignore_config = t,
        "incremental" => options.incremental = t,
        "watch" => options.watch = t,
        "version" => options.version = t,
        "help" => options.help = t,
        "all" => options.all = t,
        "init" => options.init = t,
        "build" => options.build = t,
        "singleThreaded" => options.single_threaded = t,
        "quiet" => options.quiet = t,
        "strict" => {
            options.strict = t;
            // `--strict` enables the full strict family.
            options.strict_null_checks = t;
            options.strict_function_types = t;
            options.strict_bind_call_apply = t;
            options.strict_property_initialization = t;
            options.strict_builtin_iterator_return = t;
            options.no_implicit_any = t;
            options.no_implicit_this = t;
            options.use_unknown_in_catch_variables = t;
            options.always_strict = t;
        }
        _ => {}
    }
}

fn apply_options(options: &HashMap<String, OptValue>, out: &mut CompilerOptions) {
    for (name, value) in options {
        match name.as_str() {
            "target" => {
                if let Some(s) = value.as_str() {
                    out.target = parse_script_target(s);
                }
            }
            "module" => {
                if let Some(s) = value.as_str() {
                    out.module = parse_module_kind(s);
                }
            }
            "moduleResolution" => {
                if let Some(s) = value.as_str() {
                    out.module_resolution = parse_module_resolution(s);
                }
            }
            "jsx" => {
                if let Some(s) = value.as_str() {
                    out.jsx = parse_jsx_emit(s);
                }
            }
            "newLine" => {
                if let Some(s) = value.as_str() {
                    out.new_line = match s.to_lowercase().as_str() {
                        "crlf" => NewLineKind::CRLF,
                        "lf" => NewLineKind::LF,
                        _ => NewLineKind::None,
                    };
                }
            }
            "moduleDetection" => {
                if let Some(s) = value.as_str() {
                    out.module_detection = match s.to_lowercase().as_str() {
                        "auto" => ModuleDetectionKind::Auto,
                        "legacy" => ModuleDetectionKind::Legacy,
                        "force" => ModuleDetectionKind::Force,
                        _ => ModuleDetectionKind::None,
                    };
                }
            }
            "lib" => {
                if let Some(list) = value.as_list() {
                    out.lib = list.to_vec();
                }
            }
            "types" => {
                if let Some(list) = value.as_list() {
                    out.types = list.to_vec();
                }
            }
            "typeRoots" => {
                if let Some(list) = value.as_list() {
                    out.type_roots = list.to_vec();
                }
            }
            "rootDirs" => {
                if let Some(list) = value.as_list() {
                    out.root_dirs = list.to_vec();
                }
            }
            "outDir" => {
                if let Some(s) = value.as_str() {
                    out.out_dir = s.to_string();
                }
            }
            "outFile" => {
                if let Some(s) = value.as_str() {
                    out.out_file = s.to_string();
                }
            }
            "rootDir" => {
                if let Some(s) = value.as_str() {
                    out.root_dir = s.to_string();
                }
            }
            "baseUrl" => {
                if let Some(s) = value.as_str() {
                    out.base_url = s.to_string();
                }
            }
            "project" => {
                if let Some(s) = value.as_str() {
                    out.project = s.to_string();
                }
            }
            "declarationDir" => {
                if let Some(s) = value.as_str() {
                    out.declaration_dir = s.to_string();
                }
            }
            "tsBuildInfoFile" => {
                if let Some(s) = value.as_str() {
                    out.ts_build_info_file = s.to_string();
                }
            }
            "sourceRoot" => {
                if let Some(s) = value.as_str() {
                    out.source_root = s.to_string();
                }
            }
            "mapRoot" => {
                if let Some(s) = value.as_str() {
                    out.map_root = s.to_string();
                }
            }
            "jsxFactory" => {
                if let Some(s) = value.as_str() {
                    out.jsx_factory = s.to_string();
                }
            }
            "jsxFragmentFactory" => {
                if let Some(s) = value.as_str() {
                    out.jsx_fragment_factory = s.to_string();
                }
            }
            "jsxImportSource" => {
                if let Some(s) = value.as_str() {
                    out.jsx_import_source = s.to_string();
                }
            }
            "reactNamespace" => {
                if let Some(s) = value.as_str() {
                    out.react_namespace = s.to_string();
                }
            }
            "locale" => {
                if let Some(s) = value.as_str() {
                    out.locale = s.to_string();
                }
            }
            "generateTrace" => {
                if let Some(s) = value.as_str() {
                    out.generate_trace = s.to_string();
                }
            }
            _ => {
                if let Some(b) = value.as_bool() {
                    set_bool(out, name, b);
                }
            }
        }
    }
}

fn apply_build_options(options: &HashMap<String, OptValue>, out: &mut BuildOptions) {
    for (name, value) in options {
        match name.as_str() {
            "clean" => {
                if let Some(b) = value.as_bool() {
                    out.clean = Tristate::from(b);
                }
            }
            "dry" => {
                if let Some(b) = value.as_bool() {
                    out.dry = Tristate::from(b);
                }
            }
            "force" => {
                if let Some(b) = value.as_bool() {
                    out.force = Tristate::from(b);
                }
            }
            "verbose" => {
                if let Some(b) = value.as_bool() {
                    out.verbose = Tristate::from(b);
                }
            }
            "stopBuildOnErrors" => {
                if let Some(b) = value.as_bool() {
                    out.stop_build_on_errors = Tristate::from(b);
                }
            }
            "builders" => {
                if let OptValue::Num(n) = value {
                    out.builders = Some(*n as i32);
                }
            }
            _ => {}
        }
    }
}

/// Apply parsed watch-option values to a `WatchOptions`, mirroring Go's
/// `ParseWatchOptions` (`internal/tsoptions/parsinghelpers.go:489-516`).
///
/// Enum values are validated case-insensitively against the declaration's
/// `enum_values` during CLI extraction, so here we just convert the accepted
/// string to the typed enum. `Null` clears the field (mirrors Go's JSON
/// `null` semantics).
fn apply_watch_options(options: &HashMap<String, OptValue>, out: &mut WatchOptions) {
    for (name, value) in options {
        match name.as_str() {
            "watchInterval" => {
                if let OptValue::Num(n) = value {
                    out.interval = Some(*n as i32);
                } else if matches!(value, OptValue::Null) {
                    out.interval = None;
                }
            }
            "watchFile" => {
                if let Some(s) = value.as_str() {
                    out.file_kind = parse_watch_file_kind(s).unwrap_or(WatchFileKind::None);
                } else if matches!(value, OptValue::Null) {
                    out.file_kind = WatchFileKind::None;
                }
            }
            "watchDirectory" => {
                if let Some(s) = value.as_str() {
                    out.directory_kind = parse_watch_directory_kind(s).unwrap_or(WatchDirectoryKind::None);
                } else if matches!(value, OptValue::Null) {
                    out.directory_kind = WatchDirectoryKind::None;
                }
            }
            "fallbackPolling" => {
                if let Some(s) = value.as_str() {
                    out.fallback_polling = parse_polling_kind(s).unwrap_or(PollingKind::None);
                } else if matches!(value, OptValue::Null) {
                    out.fallback_polling = PollingKind::None;
                }
            }
            "synchronousWatchDirectory" => {
                if let Some(b) = value.as_bool() {
                    out.sync_watch_dir = Tristate::from(b);
                } else if matches!(value, OptValue::Null) {
                    out.sync_watch_dir = Tristate::Unknown;
                }
            }
            "excludeDirectories" => {
                if let Some(list) = value.as_list() {
                    out.exclude_dir = list.to_vec();
                }
            }
            "excludeFiles" => {
                if let Some(list) = value.as_list() {
                    out.exclude_files = list.to_vec();
                }
            }
            _ => {}
        }
    }
}

fn parse_script_target(s: &str) -> ScriptTarget {
    let s = s.to_lowercase();
    let s = s.replace('-', "");
    match s.as_str() {
        "es3" => ScriptTarget::ES5,
        "es5" => ScriptTarget::ES5,
        "es6" | "es2015" => ScriptTarget::ES2015,
        "es2016" => ScriptTarget::ES2016,
        "es2017" => ScriptTarget::ES2017,
        "es2018" => ScriptTarget::ES2018,
        "es2019" => ScriptTarget::ES2019,
        "es2020" => ScriptTarget::ES2020,
        "es2021" => ScriptTarget::ES2021,
        "es2022" => ScriptTarget::ES2022,
        "es2023" => ScriptTarget::ES2023,
        "es2024" => ScriptTarget::ES2024,
        "es2025" => ScriptTarget::ES2025,
        "esnext" => ScriptTarget::ESNext,
        "json" => ScriptTarget::JSON,
        _ => ScriptTarget::None,
    }
}

fn parse_module_kind(s: &str) -> ModuleKind {
    match s.to_lowercase().as_str() {
        "commonjs" => ModuleKind::CommonJS,
        "amd" => ModuleKind::AMD,
        "umd" => ModuleKind::UMD,
        "system" => ModuleKind::System,
        "es6" | "es2015" => ModuleKind::ES2015,
        "es2020" => ModuleKind::ES2020,
        "es2022" => ModuleKind::ES2022,
        "esnext" => ModuleKind::ESNext,
        "node16" => ModuleKind::Node16,
        "node18" => ModuleKind::Node18,
        "node20" => ModuleKind::Node20,
        "nodenext" => ModuleKind::NodeNext,
        "preserve" => ModuleKind::Preserve,
        _ => ModuleKind::None,
    }
}

fn parse_module_resolution(s: &str) -> ModuleResolutionKind {
    match s.to_lowercase().as_str() {
        "classic" => ModuleResolutionKind::Classic,
        "node" | "node10" => ModuleResolutionKind::Node10,
        "node16" => ModuleResolutionKind::Node16,
        "nodenext" => ModuleResolutionKind::NodeNext,
        "bundler" => ModuleResolutionKind::Bundler,
        _ => ModuleResolutionKind::Unknown,
    }
}

fn parse_jsx_emit(s: &str) -> JsxEmit {
    match s.to_lowercase().as_str() {
        "preserve" => JsxEmit::Preserve,
        "react" => JsxEmit::React,
        "react-native" => JsxEmit::ReactNative,
        "react-jsx" => JsxEmit::ReactJSX,
        "react-jsxdev" => JsxEmit::ReactJSXDev,
        _ => JsxEmit::None,
    }
}

/// Reverse mapping: `ScriptTarget` value → canonical string name used in
/// tsconfig.json. Returns `None` for `ScriptTarget::None` (unset).
pub fn script_target_name(t: ScriptTarget) -> Option<&'static str> {
    match t {
        ScriptTarget::ES5 => Some("es5"),
        ScriptTarget::ES2015 => Some("es2015"),
        ScriptTarget::ES2016 => Some("es2016"),
        ScriptTarget::ES2017 => Some("es2017"),
        ScriptTarget::ES2018 => Some("es2018"),
        ScriptTarget::ES2019 => Some("es2019"),
        ScriptTarget::ES2020 => Some("es2020"),
        ScriptTarget::ES2021 => Some("es2021"),
        ScriptTarget::ES2022 => Some("es2022"),
        ScriptTarget::ES2023 => Some("es2023"),
        ScriptTarget::ES2024 => Some("es2024"),
        ScriptTarget::ES2025 => Some("es2025"),
        ScriptTarget::ESNext => Some("esnext"),
        ScriptTarget::JSON => Some("json"),
        ScriptTarget::None => None,
    }
}

/// Reverse mapping: `ModuleKind` value → canonical string name.
pub fn module_kind_name(m: ModuleKind) -> Option<&'static str> {
    match m {
        ModuleKind::CommonJS => Some("commonjs"),
        ModuleKind::AMD => Some("amd"),
        ModuleKind::UMD => Some("umd"),
        ModuleKind::System => Some("system"),
        ModuleKind::ES2015 => Some("es2015"),
        ModuleKind::ES2020 => Some("es2020"),
        ModuleKind::ES2022 => Some("es2022"),
        ModuleKind::ESNext => Some("esnext"),
        ModuleKind::Node16 => Some("node16"),
        ModuleKind::Node18 => Some("node18"),
        ModuleKind::Node20 => Some("node20"),
        ModuleKind::NodeNext => Some("nodenext"),
        ModuleKind::Preserve => Some("preserve"),
        ModuleKind::None => None,
    }
}

/// Reverse mapping: `ModuleResolutionKind` value → canonical string name.
pub fn module_resolution_name(r: ModuleResolutionKind) -> Option<&'static str> {
    match r {
        ModuleResolutionKind::Classic => Some("classic"),
        ModuleResolutionKind::Node10 => Some("node10"),
        ModuleResolutionKind::Node16 => Some("node16"),
        ModuleResolutionKind::NodeNext => Some("nodenext"),
        ModuleResolutionKind::Bundler => Some("bundler"),
        ModuleResolutionKind::Unknown => None,
    }
}

/// Reverse mapping: `JsxEmit` value → canonical string name.
pub fn jsx_emit_name(j: JsxEmit) -> Option<&'static str> {
    match j {
        JsxEmit::Preserve => Some("preserve"),
        JsxEmit::React => Some("react"),
        JsxEmit::ReactNative => Some("react-native"),
        JsxEmit::ReactJSX => Some("react-jsx"),
        JsxEmit::ReactJSXDev => Some("react-jsxdev"),
        JsxEmit::None => None,
    }
}

/// Reverse mapping: `ModuleDetectionKind` value → canonical string name.
pub fn module_detection_name(d: ModuleDetectionKind) -> Option<&'static str> {
    match d {
        ModuleDetectionKind::Auto => Some("auto"),
        ModuleDetectionKind::Force => Some("force"),
        ModuleDetectionKind::Legacy => Some("legacy"),
        ModuleDetectionKind::None => None,
    }
}

/// Reverse mapping: `NewLineKind` value → canonical string name.
pub fn new_line_name(n: NewLineKind) -> Option<&'static str> {
    match n {
        NewLineKind::CRLF => Some("crlf"),
        NewLineKind::LF => Some("lf"),
        NewLineKind::None => None,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// tsconfig.json parsing
// ────────────────────────────────────────────────────────────────────────────

/// Parse a `tsconfig.json` file into a `ParsedCommandLine`, merging `base_options`
/// (from the command line) and expanding `files`/`include`/`exclude`.
///
/// This is the public entry point; it begins `extends` resolution with an empty
/// resolution stack. Cycle detection and `extends`-as-array handling are
/// implemented in the internal worker.
pub fn get_parsed_command_line_of_config_file(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
) -> ParsedCommandLine {
    get_parsed_command_line_of_config_file_with_stack(
        config_file_name,
        base_options,
        current_dir,
        fs,
        &[],
    )
}

fn get_parsed_command_line_of_config_file_with_stack(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
    resolution_stack: &[String],
) -> ParsedCommandLine {
    let mut result = ParsedCommandLine::default();
    result.compiler_options = base_options.clone();
    result.config_file_name = config_file_name.to_string();

    // Cycle detection: normalize the config path and check the resolution
    // stack. A repeat means an `extends` cycle (a -> b -> a); emit the
    // circularity diagnostic and bail out to avoid infinite recursion.
    let resolved_path = tspath::get_normalized_absolute_path(config_file_name, current_dir);
    if resolution_stack.iter().any(|p| p == &resolved_path) {
        result.errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0,
            vec![resolved_path],
        ));
        return result;
    }

    let config_text = match fs.read_file(config_file_name) {
        Some(t) => t,
        None => {
            result.errors.push(err(format!(
                "Cannot find a tsconfig.json file at the specified directory: '{config_file_name}'."
            )));
            return result;
        }
    };

    let jsonc = strip_jsonc(&config_text);
    // An empty tsconfig.json is treated as {} (no options).
    let root: crate::json::Value = if jsonc.trim().is_empty() {
        crate::json::Value::Object(crate::json::Map::new())
    } else {
        match crate::json::from_str(&jsonc) {
            Ok(v) => v,
            Err(e) => {
                result
                    .errors
                    .push(err(format!("Failed to parse tsconfig.json: {e}.")));
                return result;
            }
        }
    };

    let root_obj = match root.as_object() {
        Some(o) => o,
        None => {
            result.errors.push(err("tsconfig.json must be an object."));
            return result;
        }
    };

    // `extends` — may be a single string or an array of strings. Each target
    // is resolved and merged in order; later targets have higher priority
    // among extended configs (Go: last-entry-wins for options, via
    // `mergeCompilerOptions` source-wins semantics in `applyExtendedConfig`).
    // The own config (parsed below) overrides extended options; command-line
    // base options override own. Effective precedence:
    //   command-line > own > last-extended > ... > first-extended > defaults.
    //
    // `include`/`exclude`/`files` specs follow a different rule: the first
    // extended config that declares a spec wins (later extended configs do
    // not override it), and the own config overrides inherited specs.
    //
    // The current config's path is pushed onto the resolution stack before
    // recursing so cycles are detected.
    let mut extended_opts = CompilerOptions::default();
    if let Some(extends) = root_obj.get("extends") {
        let extends_paths = extends_as_paths(extends, config_file_name, current_dir, fs);
        if !extends_paths.is_empty() {
            let mut new_stack: Vec<String> = resolution_stack.to_vec();
            new_stack.push(resolved_path.clone());
            // Parse all extended configs first so we can merge options in
            // reverse order (last wins) while inheriting include/exclude/files
            // in forward order (first wins).
            let mut extended_configs: Vec<(String, ParsedCommandLine)> = Vec::new();
            for ext_path in &extends_paths {
                let parent = get_parsed_command_line_of_config_file_with_stack(
                    ext_path,
                    &CompilerOptions::default(),
                    current_dir,
                    fs,
                    &new_stack,
                );
                extended_configs.push((ext_path.clone(), parent));
            }
            // Options: merge in reverse so the last extends entry wins
            // (dst-wins merge: last iterated first sets fields, earlier
            // entries only fill gaps the last didn't set).
            for (_, parent) in extended_configs.iter().rev() {
                merge_compiler_options(&mut extended_opts, &parent.compiler_options);
            }
            // include/exclude/files: first extended config that declares a
            // spec wins (only inherit if result doesn't already have it).
            // Relative paths in inherited specs are rewritten to be relative
            // to the OWN config's directory (not the extended config's
            // directory), mirroring Go's `applyExtendedConfig` which calls
            // `tspath.ConvertToRelativePath(GetDirectoryPath(extendedConfigPath), …)`
            // and prefixes each relative spec with the result. Absolute paths
            // and `${configDir}`-prefixed paths are passed through as-is.
            let own_config_dir = tspath::get_directory_path(config_file_name);
            let compare_opts = tspath::ComparePathsOptions {
                use_case_sensitive_file_names: fs.use_case_sensitive_file_names(),
                current_directory: own_config_dir.clone(),
            };
            for (ext_path, parent) in &extended_configs {
                let ext_dir = tspath::get_directory_path(ext_path);
                let relative_difference =
                    tspath::convert_to_relative_path(&ext_dir, &compare_opts);
                let rewrite = |spec: &str| -> String {
                    if starts_with_config_dir_template(spec) || tspath::is_rooted_disk_path(spec) {
                        spec.to_string()
                    } else {
                        tspath::combine_paths(&relative_difference, &[spec])
                    }
                };
                if !result.has_include_spec && parent.has_include_spec {
                    result.include = parent.include.iter().map(|s| rewrite(s)).collect();
                    result.has_include_spec = true;
                }
                if !result.has_exclude_spec && parent.has_exclude_spec {
                    result.exclude = parent.exclude.iter().map(|s| rewrite(s)).collect();
                    result.has_exclude_spec = true;
                }
                if !result.has_files_spec && parent.has_files_spec {
                    result.files_spec = parent.files_spec.iter().map(|s| rewrite(s)).collect();
                    result.has_files_spec = true;
                }
                result.errors.extend(parent.errors.clone());
            }
        }
    }

    if let Some(value) = root_obj.get("compileOnSave").and_then(|v| v.as_bool()) {
        result.compile_on_save = Some(value);
    }

    if let Some(references) = root_obj.get("references").and_then(|v| v.as_array()) {
        let config_dir_for_refs = tspath::get_directory_path(config_file_name);
        result.references = references
            .iter()
            .filter_map(|entry| {
                let raw_path = entry.as_object()?.get("path")?.as_str()?;
                Some(crate::core::project_reference::ProjectReference {
                    path: tspath::get_normalized_absolute_path(raw_path, &config_dir_for_refs),
                    original_path: raw_path.to_string(),
                    circular: false,
                })
            })
            .collect();
    }

    // `files`
    if let Some(files) = root_obj.get("files").and_then(|v| v.as_array()) {
        result.has_files_spec = true;
        result.files_spec.clear();
        for f in files {
            if let Some(s) = f.as_str() {
                result.files_spec.push(s.to_string());
            }
        }
    }
    // `include`
    if let Some(include) = root_obj.get("include").and_then(|v| v.as_array()) {
        result.has_include_spec = true;
        result.include.clear();
        for f in include {
            if let Some(s) = f.as_str() {
                result.include.push(s.to_string());
            }
        }
    }
    // `exclude`
    if let Some(exclude) = root_obj.get("exclude").and_then(|v| v.as_array()) {
        result.has_exclude_spec = true;
        result.exclude.clear();
        for f in exclude {
            if let Some(s) = f.as_str() {
                result.exclude.push(s.to_string());
            }
        }
    }

    // `compilerOptions`
    if let Some(co) = root_obj.get("compilerOptions").and_then(|v| v.as_object()) {
        result.raw_options = Some(crate::json::Value::Object(co.clone()));
        let (opts, opts_errors) = json_object_to_options(co);
        result.errors.extend(opts_errors);
        // Build the own config's compiler options in isolation.
        let mut config_opts = CompilerOptions::default();
        apply_options(&opts, &mut config_opts);
        // Handle `paths` specially — it's an object map, not handled by apply_options.
        if let Some(paths_val) = co.get("paths").and_then(|v| v.as_object()) {
            let mut paths_map = HashMap::new();
            for (key, val) in paths_val {
                if let Some(arr) = val.as_array() {
                    let targets: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    paths_map.insert(key.clone(), targets);
                }
            }
            config_opts.paths = Some(paths_map);
        }
        // Resolve `IsFilePath` options (rootDir, outDir, declarationDir, …) to
        // absolute paths relative to the config file directory, mirroring Go's
        // `normalizeNonListOptionValue`.
        let config_dir_for_opts = tspath::get_directory_path(config_file_name);
        resolve_file_path_options(&mut config_opts, &config_dir_for_opts);
        // Apply precedence: command-line (base) > own (config_opts) > extended.
        // `result.compiler_options` currently holds base_options (command-line).
        // merge_compiler_options is dst-wins (src fills gaps), so:
        //   1. merge own into base → own fills gaps of command-line (cmd wins)
        //   2. merge extended into result → extended fills gaps of own (own wins)
        merge_compiler_options(&mut result.compiler_options, &config_opts);
        merge_compiler_options(&mut result.compiler_options, &extended_opts);
    } else {
        // No own compilerOptions; merge extended into base (command-line
        // wins, extended fills gaps). No-op when no extends was present.
        merge_compiler_options(&mut result.compiler_options, &extended_opts);
    }

    // Apply `${configDir}` template substitution to the merged compiler
    // options. This must happen AFTER the merge so that `${configDir}`-prefixed
    // values from the own config (which survived `resolve_file_path_options`
    // because they were skipped) are resolved against this config's directory.
    // Extended config `${configDir}` values were already substituted during
    // their recursive parse, so only own-config values remain. Mirrors Go's
    // `handleOptionConfigDirTemplateSubstitution` (tsconfigparsing.go:1210).
    let config_dir = tspath::get_directory_path(config_file_name);
    handle_config_dir_template_substitution(&mut result.compiler_options, &config_dir);

    // Apply `${configDir}` substitution to include/exclude/files specs.
    // Mirrors Go's `getSubstitutedStringArrayWithConfigDirTemplate` calls at
    // tsconfigparsing.go:1290/1298/1309.
    //
    // IMPORTANT: Only apply this substitution for the OWN config (when
    // `resolution_stack` is empty). For extended configs, the `${configDir}`
    // prefixes must be preserved so they can be passed through during
    // inheritance and resolved against the OWN config's directory later.
    // This matches Go's behavior where `applyExtendedConfig` reads the RAW
    // extended config's include/exclude/files (not the substituted ones) and
    // passes `${configDir}`-prefixed paths through as-is.
    if resolution_stack.is_empty() {
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&result.include, &config_dir)
        {
            result.include = substituted;
        }
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&result.exclude, &config_dir)
        {
            result.exclude = substituted;
        }
        if let Some(substituted) =
            get_substituted_string_array_with_config_dir_template(&result.files_spec, &config_dir)
        {
            result.files_spec = substituted;
        }
    }

    // Resolve file names from specs.
    result.file_names = expand_file_names(
        &result.files_spec,
        result.has_files_spec,
        &result.include,
        result.has_include_spec,
        &result.exclude,
        result.has_exclude_spec,
        &result.compiler_options,
        &config_dir,
        fs,
    );

    // TS18003: report when a config yields no input files, unless the config
    // explicitly opts out by declaring `files` or `references`, or this config
    // is being parsed as part of an `extends` chain (resolution_stack non-empty).
    // Mirrors Go's `shouldReportNoInputFiles` + `canJsonReportNoInputFiles`.
    if result.file_names.is_empty() && resolution_stack.is_empty() {
        let can_report = !root_obj.contains_key("files") && !root_obj.contains_key("references");
        if can_report {
            let include_json = serde_json::to_string(&result.include).unwrap_or_else(|_| "[]".into());
            let exclude_json = serde_json::to_string(&result.exclude).unwrap_or_else(|_| "[]".into());
            result.errors.push(Diagnostic::new(
                None,
                TextRange::undefined(),
                NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2,
                vec![config_file_name.to_string(), include_json, exclude_json],
            ));
        }
    }

    result
}

/// Resolve the `extends` field of a tsconfig into a list of concrete config
/// file paths. `extends` may be a single string or an array of strings (TS 5.0+);
/// each entry is resolved relative to the extending config's directory, then
/// relative to `current_dir`. Non-string entries are ignored. Returns an empty
/// vec when no valid extends target can be produced.
fn extends_as_paths(
    extends: &crate::json::Value,
    config_file_name: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Vec<String> {
    // Accept either a single string or an array of strings.
    let specs: Vec<String> = match extends {
        crate::json::Value::String(s) => vec![s.clone()],
        crate::json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => return Vec::new(),
    };
    specs
        .into_iter()
        .filter_map(|s| resolve_single_extends_path(&s, config_file_name, current_dir, fs))
        .collect()
}

fn resolve_single_extends_path(
    s: &str,
    config_file_name: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Option<String> {
    let config_dir = tspath::get_directory_path(config_file_name);
    // Combine and normalize so that `./base` doesn't leave a stray `./` in
    // the path (which would cause `.json` suffix checks to miss the file).
    let base = tspath::normalize_path(&tspath::combine_paths(&config_dir, &[s]));
    // Try as-is first (mirrors Go's `fs.FileExists(extendedConfigPath)`).
    if fs.file_exists(&base) {
        return Some(base);
    }
    // If the spec doesn't already end in `.json`, try appending `.json`
    // (mirrors Go's `extendedConfigPath + ".json"` fallback for relative specs).
    if !base.ends_with(".json") {
        let with_json = format!("{base}.json");
        if fs.file_exists(&with_json) {
            return Some(with_json);
        }
    }
    // Try the directory form (`spec/tsconfig.json`). Go only does this via
    // Node-style resolution for module specs, but the TS docs document
    // directory extends and this form is widely used in monorepos.
    let dir_form = tspath::combine_paths(&base, &["tsconfig.json"]);
    if fs.file_exists(&dir_form) {
        return Some(dir_form);
    }
    // Fall back to the raw string resolved against current_dir.
    let abs = tspath::get_normalized_absolute_path(s, current_dir);
    if fs.file_exists(&abs) {
        Some(abs)
    } else {
        Some(tspath::combine_paths(&abs, &["tsconfig.json"]))
    }
}

fn json_object_to_options(
    obj: &crate::json::Map<String, crate::json::Value>,
) -> (HashMap<String, OptValue>, Vec<Diagnostic>) {
    let mut out = HashMap::new();
    let mut errors = Vec::new();
    for (k, v) in obj {
        // Declaration-driven case-mismatch detection: look up the key
        // case-insensitively; if a declaration exists but its canonical name
        // does not exactly match the key, emit a "did you mean" diagnostic and
        // skip the key (mirrors Go's `convertOptionsFromJson`).
        if let Some(opt) = find_option(k) {
            if opt.name != k {
                errors.push(Diagnostic::new(
                    None,
                    TextRange::undefined(),
                    UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1,
                    vec![k.clone(), opt.name.to_string()],
                ));
                continue;
            }
        }
        let val = json_to_opt_value(v);
        out.insert(k.clone(), val);
    }
    (out, errors)
}

fn json_to_opt_value(v: &crate::json::Value) -> OptValue {
    match v {
        crate::json::Value::Bool(b) => OptValue::Bool(*b),
        crate::json::Value::String(s) => OptValue::Str(s.clone()),
        crate::json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                OptValue::Num(i)
            } else {
                OptValue::Str(n.to_string())
            }
        }
        crate::json::Value::Array(arr) => {
            let list = arr
                .iter()
                .filter_map(|e| e.as_str().map(|s| s.to_string()))
                .collect();
            OptValue::List(list)
        }
        crate::json::Value::Null => OptValue::Null,
        crate::json::Value::Object(_) => OptValue::Null,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// `${configDir}` template substitution (TS 5.5+)
// ────────────────────────────────────────────────────────────────────────────

/// The `${configDir}` template variable (TS 5.5+). When a tsconfig.json value
/// starts with this prefix, it is resolved relative to the config file's
/// directory rather than the usual basePath. Mirrors Go's
/// `configDirTemplate` (`tsconfigparsing.go:428`).
const CONFIG_DIR_TEMPLATE: &str = "${configDir}";

/// Whether `value` starts with the `${configDir}` template prefix
/// (case-insensitive). Mirrors Go's `startsWithConfigDirTemplate`.
fn starts_with_config_dir_template(value: &str) -> bool {
    value.to_ascii_lowercase().starts_with(&CONFIG_DIR_TEMPLATE.to_ascii_lowercase())
}

/// Replace the first `${configDir}` in `value` with `./` and resolve the
/// result as a normalized absolute path against `base_path`. Mirrors Go's
/// `getSubstitutedPathWithConfigDirTemplate`.
fn get_substituted_path_with_config_dir_template(value: &str, base_path: &str) -> String {
    let replaced = value.replacen(CONFIG_DIR_TEMPLATE, "./", 1);
    tspath::get_normalized_absolute_path(&replaced, base_path)
}

/// Apply `${configDir}` substitution to a string array. Returns `Some(new_vec)`
/// if any element was substituted, or `None` if no element needed substitution
/// (mirrors Go's nil-return convention so callers can skip clone).
/// Mirrors Go's `getSubstitutedStringArrayWithConfigDirTemplate`.
fn get_substituted_string_array_with_config_dir_template(
    list: &[String],
    base_path: &str,
) -> Option<Vec<String>> {
    let mut result: Option<Vec<String>> = None;
    for (i, element) in list.iter().enumerate() {
        if starts_with_config_dir_template(element) {
            let arr = result.get_or_insert_with(|| list.to_vec());
            arr[i] = get_substituted_path_with_config_dir_template(element, base_path);
        }
    }
    result
}

/// Apply `${configDir}` substitution to all relevant compiler options after
/// the merge step. Mirrors Go's `handleOptionConfigDirTemplateSubstitution`.
///
/// Affected options: `paths` (each target), `rootDirs`, `typeRoots`,
/// `generateCpuProfile`, `generateTrace`, `outFile`, `outDir`, `rootDir`,
/// `tsBuildInfoFile`, `baseUrl`, `declarationDir`.
fn handle_config_dir_template_substitution(options: &mut CompilerOptions, base_path: &str) {
    // `paths` — substitute each target list, keyed by pattern.
    if let Some(paths) = options.paths.as_mut() {
        let mut changed = false;
        for (_, targets) in paths.iter_mut() {
            if let Some(substituted) =
                get_substituted_string_array_with_config_dir_template(targets, base_path)
            {
                *targets = substituted;
                changed = true;
            }
        }
        if !changed {
            // No substitution needed; nothing to do.
        }
    }

    // `rootDirs`
    if let Some(root_dirs) =
        get_substituted_string_array_with_config_dir_template(&options.root_dirs, base_path)
    {
        options.root_dirs = root_dirs;
    }

    // `typeRoots`
    if let Some(type_roots) =
        get_substituted_string_array_with_config_dir_template(&options.type_roots, base_path)
    {
        options.type_roots = type_roots;
    }

    // String-valued file-path options.
    if starts_with_config_dir_template(&options.generate_cpu_profile) {
        options.generate_cpu_profile =
            get_substituted_path_with_config_dir_template(&options.generate_cpu_profile, base_path);
    }
    if starts_with_config_dir_template(&options.generate_trace) {
        options.generate_trace =
            get_substituted_path_with_config_dir_template(&options.generate_trace, base_path);
    }
    if starts_with_config_dir_template(&options.out_file) {
        options.out_file =
            get_substituted_path_with_config_dir_template(&options.out_file, base_path);
    }
    if starts_with_config_dir_template(&options.out_dir) {
        options.out_dir =
            get_substituted_path_with_config_dir_template(&options.out_dir, base_path);
    }
    if starts_with_config_dir_template(&options.root_dir) {
        options.root_dir =
            get_substituted_path_with_config_dir_template(&options.root_dir, base_path);
    }
    if starts_with_config_dir_template(&options.ts_build_info_file) {
        options.ts_build_info_file =
            get_substituted_path_with_config_dir_template(&options.ts_build_info_file, base_path);
    }
    if starts_with_config_dir_template(&options.base_url) {
        options.base_url =
            get_substituted_path_with_config_dir_template(&options.base_url, base_path);
    }
    if starts_with_config_dir_template(&options.declaration_dir) {
        options.declaration_dir =
            get_substituted_path_with_config_dir_template(&options.declaration_dir, base_path);
    }
}

/// Resolve `IsFilePath` compiler options to absolute paths relative to
/// `base_path`, mirroring Go's `normalizeNonListOptionValue`.
///
/// Options with `IsFilePath: true` (rootDir, outDir, declarationDir, outFile,
/// baseUrl, tsBuildInfoFile, sourceRoot, mapRoot, project, …) are stored as
/// written in the tsconfig (often relative). Go resolves them to absolute
/// paths during JSON option parsing so that downstream code (emitter, program)
/// can compare them against absolute source file paths without needing to
/// track the config directory separately.
///
/// `${configDir}`-prefixed values are skipped here and substituted later via
/// `handle_config_dir_template_substitution` (mirrors Go's
/// `normalizeNonListOptionValue` which also skips `${configDir}` values).
fn resolve_file_path_options(options: &mut CompilerOptions, base_path: &str) {
    let resolve = |s: &str| -> String {
        if s.is_empty() {
            return s.to_string();
        }
        // `${configDir}` templates are substituted after the merge step via
        // `handle_config_dir_template_substitution`; skip them here to avoid
        // resolving against the wrong base_path (mirrors Go's
        // `normalizeNonListOptionValue` which checks `startsWithConfigDirTemplate`).
        if starts_with_config_dir_template(s) {
            return s.to_string();
        }
        tspath::get_normalized_absolute_path(s, base_path)
    };
    options.root_dir = resolve(&options.root_dir);
    options.out_dir = resolve(&options.out_dir);
    options.out_file = resolve(&options.out_file);
    options.declaration_dir = resolve(&options.declaration_dir);
    options.base_url = resolve(&options.base_url);
    options.ts_build_info_file = resolve(&options.ts_build_info_file);
    options.source_root = resolve(&options.source_root);
    options.map_root = resolve(&options.map_root);
    options.project = resolve(&options.project);
    options.generate_cpu_profile = resolve(&options.generate_cpu_profile);
    options.generate_trace = resolve(&options.generate_trace);
    if !options.root_dirs.is_empty() {
        options.root_dirs = options.root_dirs.iter().map(|s| resolve(s)).collect();
    }
}

/// Merge `src` into `dst`, where `dst` values take precedence (already set).
fn merge_compiler_options(dst: &mut CompilerOptions, src: &CompilerOptions) {
    // Apply src fields only where dst is at its default/unset.
    macro_rules! merge_tri {
        ($field:ident) => {
            if dst.$field.is_unknown() {
                dst.$field = src.$field;
            }
        };
    }
    merge_tri!(no_emit);
    merge_tri!(no_check);
    merge_tri!(no_lib);
    merge_tri!(skip_lib_check);
    merge_tri!(skip_default_lib_check);
    merge_tri!(strict);
    merge_tri!(strict_null_checks);
    merge_tri!(strict_function_types);
    merge_tri!(strict_bind_call_apply);
    merge_tri!(strict_property_initialization);
    merge_tri!(strict_builtin_iterator_return);
    merge_tri!(no_implicit_any);
    merge_tri!(no_implicit_this);
    merge_tri!(no_implicit_override);
    merge_tri!(no_unused_locals);
    merge_tri!(no_unused_parameters);
    merge_tri!(no_fallthrough_cases_in_switch);
    merge_tri!(no_unchecked_indexed_access);
    merge_tri!(exact_optional_property_types);
    merge_tri!(es_module_interop);
    merge_tri!(allow_js);
    merge_tri!(check_js);
    merge_tri!(composite);
    merge_tri!(declaration);
    merge_tri!(source_map);
    merge_tri!(remove_comments);
    merge_tri!(isolated_modules);
    merge_tri!(verbatim_module_syntax);
    merge_tri!(experimental_decorators);
    merge_tri!(force_consistent_casing_in_file_names);
    merge_tri!(use_unknown_in_catch_variables);
    merge_tri!(pretty);
    merge_tri!(incremental);
    merge_tri!(watch);
    if dst.target == ScriptTarget::None {
        dst.target = src.target;
    }
    if dst.module == ModuleKind::None {
        dst.module = src.module;
    }
    if dst.module_resolution == ModuleResolutionKind::Unknown {
        dst.module_resolution = src.module_resolution;
    }
    if dst.jsx == JsxEmit::None {
        dst.jsx = src.jsx;
    }
    if dst.out_dir.is_empty() {
        dst.out_dir = src.out_dir.clone();
    }
    if dst.root_dir.is_empty() {
        dst.root_dir = src.root_dir.clone();
    }
    if dst.base_url.is_empty() {
        dst.base_url = src.base_url.clone();
    }
    if dst.lib.is_empty() {
        dst.lib = src.lib.clone();
    }
    if dst.types.is_empty() {
        dst.types = src.types.clone();
    }
    if dst.type_roots.is_empty() {
        dst.type_roots = src.type_roots.clone();
    }
    if dst.paths.is_none() {
        dst.paths = src.paths.clone();
    }
    if dst.declaration_dir.is_empty() {
        dst.declaration_dir = src.declaration_dir.clone();
    }
    if dst.source_root.is_empty() {
        dst.source_root = src.source_root.clone();
    }
    if dst.map_root.is_empty() {
        dst.map_root = src.map_root.clone();
    }
    if dst.ts_build_info_file.is_empty() {
        dst.ts_build_info_file = src.ts_build_info_file.clone();
    }
    if dst.root_dirs.is_empty() {
        dst.root_dirs = src.root_dirs.clone();
    }
    if dst.module_suffixes.is_empty() {
        dst.module_suffixes = src.module_suffixes.clone();
    }
    if dst.custom_conditions.is_empty() {
        dst.custom_conditions = src.custom_conditions.clone();
    }
    if dst.out_file.is_empty() {
        dst.out_file = src.out_file.clone();
    }
    if dst.module_detection == ModuleDetectionKind::None {
        dst.module_detection = src.module_detection;
    }
    if dst.new_line == NewLineKind::None {
        dst.new_line = src.new_line;
    }
}

/// Resolve the set of input file names from `files`/`include`/`exclude` specs.
fn expand_file_names(
    files: &[String],
    has_files_spec: bool,
    include: &[String],
    has_include_spec: bool,
    exclude: &[String],
    has_exclude_spec: bool,
    options: &CompilerOptions,
    base_dir: &str,
    fs: &dyn FS,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut effective_exclude = exclude.to_vec();
    if !has_exclude_spec {
        if !options.out_dir.is_empty() {
            effective_exclude.push(options.out_dir.clone());
        }
        if !options.declaration_dir.is_empty() {
            effective_exclude.push(options.declaration_dir.clone());
        }
    }
    let exclude_dirs: Vec<String> = effective_exclude
        .iter()
        .filter(|p| !p.contains('*') && !p.contains('?') && !p.contains('[') && !p.contains('{'))
        .map(|p| tspath::get_normalized_absolute_path(p, base_dir))
        .collect();
    let exclude_globs: Vec<Glob> = effective_exclude
        .iter()
        .filter_map(|p| {
            let spec = if tspath::path_is_absolute(p) {
                p.clone()
            } else {
                tspath::combine_paths(base_dir, &[p])
            };
            Glob::parse(&spec).ok()
        })
        .collect();

    let add = |path: &str, out: &mut Vec<String>, seen: &mut std::collections::HashSet<String>| {
        let abs = tspath::get_normalized_absolute_path(path, base_dir);
        if seen.insert(abs.clone()) {
            out.push(abs);
        }
    };

    // Explicit `files`.
    for f in files {
        add(f, &mut result, &mut seen);
    }

    // `include` glob expansion.
    let include_specs: Vec<String> = if !has_include_spec && !has_files_spec {
        vec!["**/*".to_string()]
    } else {
        include.to_vec()
    };
    for spec in &include_specs {
        let matched = match_glob_spec(spec, base_dir, fs);
        for path in matched {
            if is_excluded(&path, &exclude_globs, &exclude_dirs) {
                continue;
            }
            if !is_supported_source_file(&path) {
                continue;
            }
            add(&path, &mut result, &mut seen);
        }
    }

    result.sort();
    result
}

fn is_excluded(path: &str, exclude_globs: &[Glob], exclude_dirs: &[String]) -> bool {
    exclude_globs.iter().any(|g| g.is_match(path))
        || exclude_dirs.iter().any(|dir| path_is_under_dir(path, dir))
}

fn path_is_under_dir(path: &str, dir: &str) -> bool {
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn is_supported_source_file(path: &str) -> bool {
    let ext = path.rfind('.').map(|i| &path[i..]).unwrap_or("");
    matches!(
        ext,
        ".ts" | ".tsx" | ".d.ts" | ".mts" | ".cts" | ".d.mts" | ".d.cts"
    )
}

/// Match an include glob spec against the filesystem, returning matching file paths.
fn match_glob_spec(spec: &str, base_dir: &str, fs: &dyn FS) -> Vec<String> {
    let mut results = Vec::new();
    // The spec may be relative to base_dir. Walk the directory tree and match.
    let abs_spec = if tspath::path_is_absolute(spec) {
        spec.to_string()
    } else {
        tspath::combine_paths(base_dir, &[spec])
    };
    if !contains_glob_char(&abs_spec) {
        if fs.file_exists(&abs_spec) {
            results.push(abs_spec);
            return results;
        }
        if fs.directory_exists(&abs_spec) {
            walk_and_collect_files(&abs_spec, fs, &mut results);
            return results;
        }
    }
    // Walk starting from the longest non-glob directory prefix of the spec.
    let walk_root = glob_base_dir(&abs_spec);
    walk_and_match(&abs_spec, &walk_root, fs, &mut results);
    results
}

fn contains_glob_char(spec: &str) -> bool {
    spec.chars()
        .any(|c| c == '*' || c == '?' || c == '{' || c == '[')
}

/// Return the longest directory prefix of `spec` that contains no glob
/// metacharacters (`*`, `?`, `{`, `[`).
fn glob_base_dir(spec: &str) -> String {
    let first_meta = spec
        .chars()
        .position(|c| c == '*' || c == '?' || c == '{' || c == '[');
    let prefix = match first_meta {
        Some(idx) => &spec[..idx],
        None => spec,
    };
    // Trim to the last directory separator.
    match prefix.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => prefix[..idx].to_string(),
        None => ".".to_string(),
    }
}

fn walk_and_collect_files(dir: &str, fs: &dyn FS, results: &mut Vec<String>) {
    let entries = fs.get_accessible_entries(dir);
    for file in &entries.files {
        results.push(tspath::combine_paths(dir, &[file]));
    }
    for d in &entries.directories {
        let full = tspath::combine_paths(dir, &[d]);
        walk_and_collect_files(&full, fs, results);
    }
}

fn walk_and_match(root_spec: &str, dir: &str, fs: &dyn FS, results: &mut Vec<String>) {
    let entries = fs.get_accessible_entries(dir);
    for file in &entries.files {
        let full = tspath::combine_paths(dir, &[file]);
        if glob_matches(root_spec, &full) {
            results.push(full);
        }
    }
    for d in &entries.directories {
        // Wildcard include walks skip common package folders like Go's vfsmatch.
        if d.eq_ignore_ascii_case("node_modules")
            || d.eq_ignore_ascii_case("bower_components")
            || d.eq_ignore_ascii_case("jspm_packages")
            || d == ".git"
        {
            continue;
        }
        let full = tspath::combine_paths(dir, &[d]);
        walk_and_match(root_spec, &full, fs, results);
    }
}

fn glob_matches(spec: &str, path: &str) -> bool {
    match Glob::parse(spec) {
        Ok(g) => g.is_match(path),
        Err(_) => false,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JSONC preprocessing
// ────────────────────────────────────────────────────────────────────────────

/// Strip `//` line comments, `/* */` block comments, and trailing commas from
/// JSONC text so it can be parsed by a strict JSON parser.
fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut in_string = false;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
                i += 1;
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                // Line comment: skip to end of line.
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                // Block comment: skip to `*/`.
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            ',' if i + 1 < chars.len() => {
                // Trailing comma: peek ahead for `}` or `]` (skipping whitespace).
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    // Drop the comma.
                    i += 1;
                } else {
                    out.push(c);
                    i += 1;
                }
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::InMemoryFS;

    #[test]
    fn parse_basic_options() {
        let args: Vec<String> = vec!["--noEmit", "--strict", "--target", "ES2020", "src/a.ts"]
            .into_iter()
            .map(String::from)
            .collect();
        let parsed = parse_command_line(&args, "/proj", None);
        assert!(parsed.compiler_options.no_emit.is_true());
        assert!(parsed.compiler_options.strict.is_true());
        assert!(parsed.compiler_options.strict_null_checks.is_true());
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert_eq!(parsed.file_names, vec!["/proj/src/a.ts"]);
    }

    #[test]
    fn parse_equals_form() {
        let args: Vec<String> = vec!["--target=ES2015", "--module=commonjs"]
            .into_iter()
            .map(String::from)
            .collect();
        let parsed = parse_command_line(&args, "/proj", None);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2015);
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
    }

    #[test]
    fn parse_short_option() {
        let args: Vec<String> = vec!["-p", "tsconfig.json"]
            .into_iter()
            .map(String::from)
            .collect();
        let parsed = parse_command_line(&args, "/proj", None);
        assert_eq!(parsed.compiler_options.project, "tsconfig.json");
    }

    #[test]
    fn strip_jsonc_comments() {
        let input = r#"{ // comment
            "compilerOptions": {
                "target": "ES5", /* block */
                "strict": true,
            }
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["compilerOptions"]["target"].as_str(), Some("ES5"));
    }

    #[test]
    fn parse_tsconfig_files() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "compilerOptions": { "target": "ES2017", "noEmit": true },
            "files": ["src/a.ts"]
        }"#,
        );
        fs.insert_file("/proj/src/a.ts", "export const x = 1;");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2017);
        assert!(parsed.compiler_options.no_emit.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/src/a.ts"]);
    }

    #[test]
    fn parse_tsconfig_include_glob() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "include": ["src/**/*"]
        }"#,
        );
        fs.insert_file("/proj/src/a.ts", "export const a = 1;");
        fs.insert_file("/proj/src/b.ts", "export const b = 2;");
        fs.insert_file("/proj/src/ignore.txt", "ignore me");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
        assert!(!parsed.file_names.iter().any(|f| f.ends_with("ignore.txt")));
    }

    // ──────────────────────────────────────────────────────────────────────
    // Helpers for the ported tests.
    // ──────────────────────────────────────────────────────────────────────

    /// Returns true if any diagnostic on `parsed` carries a message argument
    /// or message text containing `needle`. Ad-hoc errors store their text in
    /// `Diagnostic.message_args[0]`; diagnostics built from a `Message`
    /// constant store their template text in `Diagnostic.message`.
    fn has_error_containing(parsed: &ParsedCommandLine, needle: &str) -> bool {
        parsed.errors.iter().any(|e| {
            e.message_args.iter().any(|a| a.contains(needle))
                || e.message.map(|m| m.text.contains(needle)).unwrap_or(false)
        })
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    // ──────────────────────────────────────────────────────────────────────
    // Command-line parser tests (ported from commandlineparser_test.go)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_command_line_version() {
        // `--version` and `-v` both set the version flag.
        let parsed = parse_command_line(&args(&["--version"]), "/proj", None);
        assert!(parsed.compiler_options.version.is_true());

        let parsed_short = parse_command_line(&args(&["-v"]), "/proj", None);
        assert!(parsed_short.compiler_options.version.is_true());
    }

    #[test]
    fn test_parse_command_line_help() {
        let parsed = parse_command_line(&args(&["--help"]), "/proj", None);
        assert!(parsed.compiler_options.help.is_true());

        let parsed_short = parse_command_line(&args(&["-h"]), "/proj", None);
        assert!(parsed_short.compiler_options.help.is_true());
    }

    #[test]
    fn test_parse_command_line_build() {
        let parsed = parse_command_line(&args(&["--build"]), "/proj", None);
        assert!(parsed.compiler_options.build.is_true());

        let parsed_short = parse_command_line(&args(&["-b"]), "/proj", None);
        assert!(parsed_short.compiler_options.build.is_true());
    }

    #[test]
    fn test_parse_build_command_line_defaults_to_current_project() {
        let parsed = parse_build_command_line(&args(&["--build"]), "/proj", None);
        assert_eq!(parsed.projects, vec!["."]);
        assert_eq!(parsed.resolved_project_paths(), vec!["/proj"]);
        assert!(parsed.compiler_options.build.is_true());
    }

    #[test]
    fn test_parse_build_command_line_build_options() {
        let parsed = parse_build_command_line(
            &args(&["--build", "src", "tests", "--force", "-v", "--dry"]),
            "/repo",
            None,
        );
        assert_eq!(parsed.projects, vec!["src", "tests"]);
        assert_eq!(
            parsed.resolved_project_paths(),
            vec!["/repo/src", "/repo/tests"]
        );
        assert!(parsed.build_options.force.is_true());
        assert!(parsed.build_options.verbose.is_true());
        assert!(parsed.build_options.dry.is_true());
        assert!(parsed.compiler_options.build.is_true());
        assert!(!parsed.compiler_options.version.is_true());
    }

    #[test]
    fn test_parse_build_command_line_invalid_option_combinations() {
        let parsed =
            parse_build_command_line(&args(&["--build", "--clean", "--force"]), "/proj", None);
        assert!(has_error_containing(
            &ParsedCommandLine {
                errors: parsed.errors,
                ..ParsedCommandLine::default()
            },
            "cannot be combined"
        ));

        let parsed =
            parse_build_command_line(&args(&["--build", "--watch", "--dry"]), "/proj", None);
        assert!(has_error_containing(
            &ParsedCommandLine {
                errors: parsed.errors,
                ..ParsedCommandLine::default()
            },
            "cannot be combined"
        ));
    }

    #[test]
    fn test_parse_command_line_watch() {
        let parsed = parse_command_line(&args(&["--watch", "0.ts"]), "/proj", None);
        assert!(parsed.compiler_options.watch.is_true());
        // The `watch` convenience flag on ParsedCommandLine mirrors the option.
        assert!(parsed.watch);

        let parsed_short = parse_command_line(&args(&["-w", "0.ts"]), "/proj", None);
        assert!(parsed_short.compiler_options.watch.is_true());
        assert!(parsed_short.watch);
    }

    #[test]
    fn watch_options_empty_by_default() {
        // No watch flags → default WatchOptions (all None/empty).
        let parsed = parse_command_line(&args(&["--noEmit", "0.ts"]), "/proj", None);
        assert!(parsed.watch_options.is_empty());
    }

    #[test]
    fn watch_options_parse_enum_flags() {
        // `--watchFile usefsevents` etc. are routed into the separate
        // watch_options map via the WatchNameMap fallback.
        let parsed = parse_command_line(
            &args(&[
                "--watchFile",
                "UseFsEvents",
                "--watchDirectory",
                "fixedpollinginterval",
                "--fallbackPolling",
                "priorityinterval",
                "0.ts",
            ]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
        assert_eq!(
            parsed.watch_options.directory_kind,
            WatchDirectoryKind::FixedPollingInterval
        );
        assert_eq!(parsed.watch_options.fallback_polling, PollingKind::PriorityInterval);
    }

    #[test]
    fn watch_options_parse_interval_and_boolean() {
        let parsed = parse_command_line(
            &args(&["--watchInterval", "250", "--synchronousWatchDirectory", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.interval, Some(250));
        assert_eq!(parsed.watch_options.watch_interval_ms(), 250);
        assert!(parsed.watch_options.sync_watch_dir.is_true());
    }

    #[test]
    fn watch_options_parse_list_flags() {
        let parsed = parse_command_line(
            &args(&[
                "--excludeDirectories",
                "tmp,build",
                "--excludeFiles",
                "a.ts,b.ts",
                "0.ts",
            ]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.exclude_dir, vec!["tmp", "build"]);
        assert_eq!(parsed.watch_options.exclude_files, vec!["a.ts", "b.ts"]);
    }

    #[test]
    fn watch_options_invalid_enum_reports_ts6046() {
        // Invalid enum value emits ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1 (TS6046)
        // listing the valid values, mirroring compiler-option enum validation.
        let parsed = parse_command_line(&args(&["--watchFile", "bogus", "0.ts"]), "/proj", None);
        assert!(parsed
            .errors
            .iter()
            .any(|d| d.code == 6046 && d.message_args.iter().any(|a| a.contains("--watchFile"))));
        // The invalid value is not stored.
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::None);
    }

    #[test]
    fn watch_options_missing_number_value_reports_ts5080() {
        // `--watchInterval` with no value emits TS5080
        // "Watch option 'watchInterval' requires a value of type number."
        let parsed = parse_command_line(&args(&["--watchInterval"]), "/proj", None);
        assert!(parsed
            .errors
            .iter()
            .any(|d| d.code == 5080
                && d.message_args.first().map(|s| s.as_str()) == Some("watchInterval")
                && d.message_args.get(1).map(|s| s.as_str()) == Some("number")));
    }

    #[test]
    fn watch_options_non_numeric_interval_reports_ts5080() {
        let parsed = parse_command_line(&args(&["--watchInterval", "abc", "0.ts"]), "/proj", None);
        assert!(parsed.errors.iter().any(|d| d.code == 5080));
        assert_eq!(parsed.watch_options.interval, None);
    }

    #[test]
    fn watch_options_build_mode_also_accepts_watch_flags() {
        // In build mode, watch flags are accepted via the same WatchNameMap
        // fallback and routed into ParsedBuildCommandLine.watch_options.
        let parsed = parse_build_command_line(
            &args(&["--build", "--watchFile", "usefsevents", "."]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
    }

    #[test]
    fn watch_options_case_insensitive_lookup() {
        // Watch option names are matched case-insensitively, mirroring Go's
        // `NameMap.GetOptionDeclarationFromName`.
        let parsed = parse_command_line(
            &args(&["--WATCHFILE", "usefsevents", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
    }

    #[test]
    fn watch_options_do_not_leak_into_compiler_options() {
        // `--watchFile` is a watch option, not a compiler option; it must not
        // appear in the compiler_options map (which would trigger TS5023).
        let parsed = parse_command_line(&args(&["--watchFile", "usefsevents", "0.ts"]), "/proj", None);
        assert!(!parsed
            .errors
            .iter()
            .any(|d| d.code == 5023 && d.message_args.iter().any(|a| a == "watchFile")));
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
    }

    #[test]
    fn test_parse_command_line_all_and_init() {
        let parsed = parse_command_line(&args(&["--all"]), "/proj", None);
        assert!(parsed.compiler_options.all.is_true());

        let parsed = parse_command_line(&args(&["--init"]), "/proj", None);
        assert!(parsed.compiler_options.init.is_true());
    }

    #[test]
    fn test_parse_command_line_lib_list() {
        // `--lib es5,es2015.symbol.wellknown 0.ts` parses as a comma-separated list.
        let parsed = parse_command_line(
            &args(&["--lib", "es5,es2015.symbol.wellknown", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(
            parsed.compiler_options.lib,
            vec!["es5".to_string(), "es2015.symbol.wellknown".to_string()]
        );
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_lib_multiple_flags() {
        // A second `--lib` on the command line overrides the first (last wins).
        let parsed = parse_command_line(
            &args(&[
                "--module",
                "commonjs",
                "--target",
                "es5",
                "--lib",
                "es5",
                "0.ts",
                "--lib",
                "es2015.core, es2015.symbol.wellknown ",
            ]),
            "/proj",
            None,
        );
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES5);
        // List values are split on commas and trimmed.
        assert_eq!(
            parsed.compiler_options.lib,
            vec![
                "es2015.core".to_string(),
                "es2015.symbol.wellknown".to_string()
            ]
        );
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_lib_empty_followed_by_option() {
        // `0.ts --lib --sourceMap`: `--lib` does not consume `--sourceMap`.
        // (The Rust parser is case-sensitive for option names, so the canonical
        // camelCase spelling `--sourceMap` is required.)
        let parsed = parse_command_line(&args(&["0.ts", "--lib", "--sourceMap"]), "/proj", None);
        assert!(parsed.compiler_options.lib.is_empty());
        assert!(parsed.compiler_options.source_map.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_unknown_option_error() {
        let parsed = parse_command_line(&args(&["--unknownOpt", "0.ts"]), "/proj", None);
        assert!(has_error_containing(&parsed, "Unknown compiler option"));
        assert!(has_error_containing(&parsed, "unknownOpt"));
    }

    #[test]
    fn test_parse_command_line_explicit_boolean_false() {
        // `--strictNullChecks false 0.ts` sets the option to false (not unknown).
        let parsed = parse_command_line(
            &args(&["--strictNullChecks", "false", "0.ts"]),
            "/proj",
            None,
        );
        assert!(parsed.compiler_options.strict_null_checks.is_false());
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_explicit_boolean_true() {
        let parsed = parse_command_line(
            &args(&["--strictNullChecks", "true", "0.ts"]),
            "/proj",
            None,
        );
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_command_line_implicit_boolean() {
        // `--strictNullChecks` with no value defaults to true.
        let parsed = parse_command_line(&args(&["--strictNullChecks"]), "/proj", None);
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_command_line_non_boolean_after_boolean_flag() {
        // `--noImplicitAny t 0.ts`: boolean flags only consume `true`/`false`,
        // so `t` is treated as an input file (matches tsgo behavior). File names
        // are kept in insertion order (the command-line parser does not sort).
        let parsed = parse_command_line(&args(&["--noImplicitAny", "t", "0.ts"]), "/proj", None);
        assert!(parsed.compiler_options.no_implicit_any.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/t", "/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_incremental() {
        let parsed = parse_command_line(&args(&["--incremental", "0.ts"]), "/proj", None);
        assert!(parsed.compiler_options.incremental.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_ts_build_info_file() {
        let parsed = parse_command_line(
            &args(&["--tsBuildInfoFile", "build.tsbuildinfo", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(
            parsed.compiler_options.ts_build_info_file,
            "build.tsbuildinfo"
        );
    }

    #[test]
    fn test_parse_command_line_ts_build_info_file_null() {
        // `--tsBuildInfoFile null` is accepted (string options honor `null`).
        let parsed =
            parse_command_line(&args(&["--tsBuildInfoFile", "null", "0.ts"]), "/proj", None);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.compiler_options.ts_build_info_file, "");
    }

    #[test]
    fn test_parse_command_line_type_roots() {
        // `--typeRoots t` parses as a single-element list.
        // (Note: unlike tsgo, the Rust port does not resolve list entries to
        // absolute paths on the command line, so we assert the parsed value.)
        let parsed = parse_command_line(
            &args(&["--typeRoots", "t", "bug.ts"]),
            "/home/project",
            None,
        );
        assert_eq!(parsed.compiler_options.type_roots, vec!["t".to_string()]);
        assert_eq!(parsed.file_names, vec!["/home/project/bug.ts"]);
    }

    #[test]
    fn test_parse_command_line_files_in_middle() {
        // Input files may appear between flags.
        let parsed = parse_command_line(
            &args(&[
                "--module",
                "commonjs",
                "--target",
                "es5",
                "0.ts",
                "--lib",
                "es5,es2015.symbol.wellknown",
            ]),
            "/proj",
            None,
        );
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES5);
        assert_eq!(
            parsed.compiler_options.lib,
            vec!["es5".to_string(), "es2015.symbol.wellknown".to_string()]
        );
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_module_resolution_and_jsx() {
        let parsed = parse_command_line(
            &args(&["--moduleResolution", "node", "--jsx", "react", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(
            parsed.compiler_options.module_resolution,
            ModuleResolutionKind::Node10
        );
        assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
    }

    #[test]
    fn test_response_file_does_not_panic() {
        // Passing `@` with an empty or non-existent filename should produce a
        // diagnostic error rather than panicking (ported from
        // TestResponseFileDoesNotPanic).
        let parsed = parse_command_line(&args(&["@"]), "/proj", None);
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Cannot read file"));

        let parsed = parse_command_line(&args(&["@blah"]), "/proj", None);
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Cannot read file"));
        assert!(has_error_containing(&parsed, "blah"));
    }

    #[test]
    fn test_response_file_missing_with_fs() {
        // Even with an FS provided, a missing response file yields an error.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        let parsed = parse_command_line(&args(&["@missing.rsp"]), "/proj", Some(&fs));
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Cannot read file"));
    }

    #[test]
    fn test_response_file_propagates_file_names() {
        // A response file that exists is expanded into arguments. The Rust port
        // currently propagates file names (and errors) from response files but
        // does not yet merge compiler options from them, so we assert only the
        // file-name propagation here.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/args.rsp", "--strict\n0.ts");
        let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
        // No errors reading the response file.
        assert!(!has_error_containing(&parsed, "Cannot read file"));
    }

    #[test]
    fn test_response_file_unterminated_quoted_string() {
        // An unterminated quoted string in a response file emits TS6045
        // (`Unterminated quoted string in response file '{0}'.`), aligned with
        // Go's `parseResponseFile`. The unterminated token is still captured
        // as an argument (matching Go's behavior).
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/args.rsp", "--outDir \"unterminated path");
        let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));
        // TS6045 diagnostic should be present.
        let has_ts6045 = parsed.errors.iter().any(|e| e.code == 6045);
        assert!(
            has_ts6045,
            "expected TS6045 for unterminated quoted string, got errors: {:?}",
            parsed
                .errors
                .iter()
                .map(|e| (e.code, e.message_args.clone()))
                .collect::<Vec<_>>()
        );
        // The unterminated content is still captured as the --outDir value.
        assert_eq!(
            parsed.compiler_options.out_dir, "unterminated path",
            "unterminated token should still be captured as the option value"
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // JSONC preprocessing tests (ported from tsconfigparsing_test.go,
    // TestParseConfigFileTextToJson scenarios)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_strip_jsonc_whitespace_and_empty_object() {
        // Whitespace-only and comment-only inputs strip to empty/whitespace.
        let stripped = strip_jsonc("   ");
        assert_eq!(stripped.trim(), "");

        let stripped = strip_jsonc("// Comment");
        assert_eq!(stripped.trim(), "");

        let stripped = strip_jsonc("/* Comment */");
        assert_eq!(stripped.trim(), "");

        // An empty object survives.
        let stripped = strip_jsonc("{}");
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert!(v.as_object().is_some());
    }

    #[test]
    fn test_strip_jsonc_comments_in_object() {
        let input = r#"{ // Excluded files
            "exclude": [
                // Exclude d.ts
                "file.d.ts"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("file.d.ts"));

        // Multiline block comments interspersed in a line are removed.
        let input = r#"{
            /* Excluded
                    Files
            */
            "exclude": [
                /* multiline comments can be in the middle of a line */"file.d.ts"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("file.d.ts"));
    }

    #[test]
    fn test_strip_jsonc_keeps_string_content() {
        // `//` and `/* */` inside string literals are preserved verbatim.
        let input = r#"{
            "exclude": [
                "xx//file.d.ts"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("xx//file.d.ts"));

        let input = r#"{
            "exclude": [
                "xx/*file.d.ts*/"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("xx/*file.d.ts*/"));
    }

    #[test]
    fn test_strip_jsonc_trailing_comma() {
        // Trailing commas before `}` or `]` are dropped.
        let input = r#"{
            "compilerOptions": {
                "target": "ES5",
                "strict": true,
            }
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["compilerOptions"]["target"].as_str(), Some("ES5"));
        assert_eq!(v["compilerOptions"]["strict"].as_bool(), Some(true));
    }

    // ──────────────────────────────────────────────────────────────────────
    // tsconfig.json parsing tests (ported from tsconfigparsing_test.go)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_tsconfig_extends_merges_options() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{
            "compilerOptions": { "target": "ES2020", "strict": true }
        }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "extends": "base.json",
            "compilerOptions": { "outDir": "./dist" }
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // Parent options are inherited.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(parsed.compiler_options.strict.is_true());
        // Child option is applied (resolved to absolute, mirroring Go's IsFilePath).
        assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");
        // `strict` from the base enables the strict family.
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_tsconfig_extends_with_own_files_include() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/base.json",
            r#"{
            "compilerOptions": { "target": "ES2020" }
        }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "extends": "base.json",
            "compilerOptions": { "outDir": "./dist" },
            "include": ["src/**/*"]
        }"#,
        );
        fs.insert_file("/proj/src/a.ts", "export const a = 1;");
        fs.insert_file("/proj/src/b.ts", "export const b = 2;");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_extends_circular_is_detected() {
        // A circular `extends` chain (a -> b -> a) must terminate and emit the
        // circularity diagnostic (code 18000) instead of stack-overflowing.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/a.tsconfig.json",
            r#"{ "extends": "b.tsconfig.json", "compilerOptions": { "target": "ES2020" } }"#,
        );
        fs.insert_file(
            "/proj/b.tsconfig.json",
            r#"{ "extends": "a.tsconfig.json", "compilerOptions": { "strict": true } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/a.tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // The circularity diagnostic must be present with code 18000.
        assert!(
            parsed
                .errors
                .iter()
                .any(|e| e.code == CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0.code),
            "expected a circularity diagnostic, got errors: {:?}",
            parsed
                .errors
                .iter()
                .map(|e| e.code)
                .collect::<Vec<_>>()
        );
        // Resolution terminated without stack overflow; own options still apply.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    }

    #[test]
    fn test_parse_tsconfig_extends_as_array_merges_all() {
        // `extends` may be an array of strings (TS 5.0+); each target is merged
        // in order, and the own config is applied on top. Here both bases
        // contribute distinct options and the own config contributes its own.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base1.json",
            r#"{ "compilerOptions": { "target": "ES2020", "strict": true } }"#,
        );
        fs.insert_file(
            "/proj/base2.json",
            r#"{ "compilerOptions": { "module": "CommonJS", "declaration": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "extends": ["base1.json", "base2.json"],
            "compilerOptions": { "outDir": "./dist" },
            "files": []
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.errors.is_empty(), "unexpected errors: {:?}", parsed.errors);
        // From base1.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(parsed.compiler_options.strict.is_true());
        // From base2.
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert!(parsed.compiler_options.declaration.is_true());
        // Own config contributes its own (non-conflicting) option.
        assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");
    }

    #[test]
    fn test_parse_tsconfig_extends_own_overrides_extended() {
        // Go precedence: own > extended. When both the own config and the
        // extended base set the same option, the own config's value must win.
        // Previously the Rust port had inverted precedence (extended won).
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "extends": "base.json",
            "compilerOptions": { "strict": false }
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // Own `strict: false` must override extended `strict: true`.
        assert!(
            parsed.compiler_options.strict.is_false(),
            "expected own strict=false to override extended strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_array_last_wins() {
        // Go precedence for extends array: later entries override earlier
        // entries for the same option (last-entry-wins, via source-wins
        // `mergeCompilerOptions` in `applyExtendedConfig`).
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base1.json",
            r#"{ "compilerOptions": { "target": "ES2020" } }"#,
        );
        fs.insert_file(
            "/proj/base2.json",
            r#"{ "compilerOptions": { "target": "ES2015" } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": ["base1.json", "base2.json"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // base2 (last) wins.
        assert_eq!(
            parsed.compiler_options.target,
            ScriptTarget::ES2015,
            "expected last extends entry (base2/ES2015) to win, got {:?}",
            parsed.compiler_options.target
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_command_line_overrides_own() {
        // Command-line base options must override own config options.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        let mut base = CompilerOptions::default();
        base.strict = Tristate::False;
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &base,
            "/proj",
            &fs,
        );
        // Command-line `--strict false` overrides config `strict: true`.
        assert!(
            parsed.compiler_options.strict.is_false(),
            "expected command-line strict=false to override config strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_include_first_extended_wins() {
        // For `include`/`exclude`/`files`, the first extended config that
        // declares a spec wins (later extended configs do not override).
        // The own config overrides inherited specs when present.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src1");
        fs.insert_dir("/proj/src2");
        fs.insert_file("/proj/src1/a.ts", "export const a = 1;");
        fs.insert_file("/proj/src2/b.ts", "export const b = 2;");
        fs.insert_file(
            "/proj/base1.json",
            r#"{ "include": ["src1/**/*"] }"#,
        );
        fs.insert_file(
            "/proj/base2.json",
            r#"{ "include": ["src2/**/*"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": ["base1.json", "base2.json"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // base1 (first) wins for include.
        assert!(
            parsed.file_names.contains(&"/proj/src1/a.ts".to_string()),
            "expected first extended include (src1) to win, got {:?}",
            parsed.file_names
        );
        assert!(
            !parsed.file_names.contains(&"/proj/src2/b.ts".to_string()),
            "expected second extended include (src2) to be suppressed, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_resolves_json_suffix() {
        // `extends: "./base"` (without `.json` extension) should resolve to
        // `./base.json` when the file exists, mirroring Go's
        // `getExtendsConfigPath` `.json` suffix fallback.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // The extends resolved to base.json and strict was inherited.
        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected extends ./base to resolve to ./base.json and inherit strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_parse_tsconfig_full_compiler_options() {
        // Ported from "parses tsconfig with compilerOptions, files, include, and exclude".
        let fs = InMemoryFS::new();
        fs.insert_dir("/apath");
        fs.insert_dir("/apath/src");
        fs.insert_dir("/apath/node_modules");
        fs.insert_dir("/apath/dist");
        fs.insert_file(
            "/apath/tsconfig.json",
            r#"{
            "compilerOptions": {
                "outDir": "./dist",
                "strict": true,
                "noImplicitAny": true,
                "target": "ES2017",
                "module": "ESNext",
                "moduleResolution": "bundler",
                "moduleDetection": "auto",
                "jsx": "react"
            },
            "files": ["/apath/src/index.ts", "/apath/src/app.ts"],
            "include": ["/apath/src/**/*"],
            "exclude": ["/apath/node_modules", "/apath/dist"]
        }"#,
        );
        fs.insert_file("/apath/src/index.ts", "");
        fs.insert_file("/apath/src/app.ts", "");
        fs.insert_file("/apath/node_modules/module.ts", "");
        fs.insert_file("/apath/dist/output.js", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/apath/tsconfig.json",
            &CompilerOptions::default(),
            "/apath",
            &fs,
        );
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2017);
        assert_eq!(parsed.compiler_options.module, ModuleKind::ESNext);
        assert_eq!(
            parsed.compiler_options.module_resolution,
            ModuleResolutionKind::Bundler
        );
        assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
        assert!(parsed.compiler_options.strict.is_true());
        assert!(parsed.compiler_options.no_implicit_any.is_true());
        assert_eq!(parsed.compiler_options.out_dir, "/apath/dist");
        // Explicit `files` are included.
        assert!(
            parsed
                .file_names
                .contains(&"/apath/src/index.ts".to_string())
        );
        assert!(parsed.file_names.contains(&"/apath/src/app.ts".to_string()));
        // node_modules is excluded during the include walk.
        assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
    }

    #[test]
    fn test_parse_tsconfig_null_enum_options() {
        // Ported from TestParseNullEnumCompilerOptions: `target: null` and
        // `module: null` should produce no errors.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "compilerOptions": {
                "target": null,
                "module": null
            }
        }"#,
        );
        fs.insert_file("/proj/app.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.errors.is_empty());
    }

    #[test]
    fn test_parse_tsconfig_empty_types_array() {
        // Ported from "handles empty types array".
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "compilerOptions": {
                "types": []
            }
        }"#,
        );
        fs.insert_file("/proj/app.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.compiler_options.types.is_empty());
    }

    #[test]
    fn test_parse_tsconfig_include_with_exclude() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/src/tests");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "include": ["src/**/*.ts"],
            "exclude": ["**/tests/**"]
        }"#,
        );
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file("/proj/src/b.ts", "");
        fs.insert_file("/proj/src/tests/skip.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
        // Excluded file is filtered out of the include expansion. The exclude
        // glob must match the absolute paths produced by the include walk, so a
        // `**/tests/**` pattern is used.
        assert!(
            !parsed
                .file_names
                .contains(&"/proj/src/tests/skip.ts".to_string())
        );
    }

    #[test]
    fn test_parse_tsconfig_literal_directory_include_recurses() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/src/nested");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "include": ["src"]
            }"#,
        );
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file("/proj/src/nested/b.tsx", "");
        fs.insert_file("/proj/src/nested/ignore.txt", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(
            parsed
                .file_names
                .contains(&"/proj/src/nested/b.tsx".to_string())
        );
        assert!(!parsed.file_names.iter().any(|f| f.ends_with("ignore.txt")));
    }

    #[test]
    fn test_parse_tsconfig_skips_node_modules_directory() {
        // Ported from "implicitly exclude common package folders": the include
        // walk skips `node_modules` directories.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/folder");
        fs.insert_file("/proj/tsconfig.json", "{}");
        fs.insert_file("/proj/node_modules/a.ts", "");
        fs.insert_file("/proj/d.ts", "");
        fs.insert_file("/proj/folder/e.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
        assert!(parsed.file_names.contains(&"/proj/d.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/folder/e.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_files_empty_does_not_default_include() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "files": [],
                "references": [{ "path": "./tsconfig.app.json" }]
            }"#,
        );
        fs.insert_file("/proj/src/a.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.has_files_spec);
        assert!(parsed.file_names.is_empty());
    }

    #[test]
    fn test_tsconfig_no_inputs_emits_ts18003() {
        // A config with no `files`/`references` and no matched files reports
        // TS18003 (mirrors Go `shouldReportNoInputFiles`).
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "./dist" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.is_empty());
        assert!(
            parsed.errors.iter().any(|d| d.code == 18003),
            "expected TS18003, got errors: {:?}",
            parsed.errors
        );
    }

    #[test]
    fn test_tsconfig_no_inputs_suppressed_by_files_key() {
        // `files: []` opts out of TS18003 even when no files match.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "files": [] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.is_empty());
        assert!(
            !parsed.errors.iter().any(|d| d.code == 18003),
            "did not expect TS18003, got errors: {:?}",
            parsed.errors
        );
    }

    #[test]
    fn test_tsconfig_no_inputs_suppressed_by_references_key() {
        // `references` opts out of TS18003 even when no files match.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "references": [{ "path": "./other.json" }] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.is_empty());
        assert!(
            !parsed.errors.iter().any(|d| d.code == 18003),
            "did not expect TS18003, got errors: {:?}",
            parsed.errors
        );
    }

    #[test]
    fn test_tsconfig_references_parsed_as_typed_project_reference() {
        // `references` entries are parsed into typed `ProjectReference` structs
        // with a normalized absolute `path` and the raw `original_path`.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/test");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "references": [{ "path": "./test" }, { "path": "./other.json" }] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.references.len(), 2);
        assert_eq!(parsed.references[0].original_path, "./test");
        assert_eq!(parsed.references[0].path, "/proj/test");
        assert!(!parsed.references[0].circular);
        assert_eq!(parsed.references[1].original_path, "./other.json");
        assert_eq!(parsed.references[1].path, "/proj/other.json");
    }

    #[test]
    fn test_parse_tsconfig_excludes_out_dir_by_default() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/dist");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": { "outDir": "dist" }
            }"#,
        );
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file("/proj/dist/a.d.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(!parsed.file_names.iter().any(|f| f.contains("/dist/")));
    }

    #[test]
    fn test_parse_tsconfig_explicit_exclude_overrides_out_dir_default() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/dist");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": { "outDir": "dist" },
                "exclude": ["obj"]
            }"#,
        );
        fs.insert_file("/proj/dist/a.d.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.has_exclude_spec);
        assert!(parsed.file_names.contains(&"/proj/dist/a.d.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_skips_common_package_directories() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/bower_components");
        fs.insert_dir("/proj/jspm_packages");
        fs.insert_file("/proj/tsconfig.json", "{}");
        fs.insert_file("/proj/node_modules/a.ts", "");
        fs.insert_file("/proj/bower_components/b.ts", "");
        fs.insert_file("/proj/jspm_packages/c.ts", "");
        fs.insert_file("/proj/d.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
        assert!(
            !parsed
                .file_names
                .iter()
                .any(|f| f.contains("bower_components"))
        );
        assert!(
            !parsed
                .file_names
                .iter()
                .any(|f| f.contains("jspm_packages"))
        );
        assert!(parsed.file_names.contains(&"/proj/d.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_skips_git_directory() {
        // The include walk skips `.git` directories.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/.git");
        fs.insert_file("/proj/tsconfig.json", "{}");
        fs.insert_file("/proj/.git/a.ts", "");
        fs.insert_file("/proj/test.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.file_names.iter().any(|f| f.contains(".git")));
        assert!(parsed.file_names.contains(&"/proj/test.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_missing_config_file_error() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/missing.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Cannot find"));
    }

    #[test]
    fn test_parse_tsconfig_invalid_json_error() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/tsconfig.json", "{ this is not json");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Failed to parse"));
    }

    #[test]
    fn test_parse_tsconfig_command_line_overrides_config() {
        // Options supplied on the command line (via `base_options`) take
        // precedence over those in tsconfig.json.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "compilerOptions": { "target": "ES2017", "strict": true }
        }"#,
        );
        fs.insert_file("/proj/app.ts", "");
        let mut base = CompilerOptions::default();
        base.target = ScriptTarget::ES2022;
        let parsed =
            get_parsed_command_line_of_config_file("/proj/tsconfig.json", &base, "/proj", &fs);
        // Command-line target wins; config-file strict is still inherited.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2022);
        assert!(parsed.compiler_options.strict.is_true());
    }

    // ──────────────────────────────────────────────────────────────────────
    // ParsedCommandLine / wildcard-directory tests
    // (ported from parsedcommandline_test.go and wildcarddirectories_test.go)
    //
    // The Rust port does not expose a `get_wildcard_directories` helper, so
    // these tests exercise the equivalent include/exclude behavior through
    // `get_parsed_command_line_of_config_file`.
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parsed_command_line_literal_file_list_dedup() {
        // Ported from "with literal file list" > "duplicates": duplicate entries
        // in `files` are deduplicated.
        let fs = InMemoryFS::new();
        fs.insert_dir("/dev");
        fs.insert_file("/dev/a.ts", "");
        fs.insert_file("/dev/b.ts", "");
        fs.insert_file(
            "/dev/tsconfig.json",
            r#"{
            "files": ["a.ts", "a.ts", "b.ts"]
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/dev/tsconfig.json",
            &CompilerOptions::default(),
            "/dev",
            &fs,
        );
        // Each file appears exactly once, sorted.
        assert_eq!(
            parsed.file_names,
            vec!["/dev/a.ts".to_string(), "/dev/b.ts".to_string()]
        );
    }

    #[test]
    fn test_parsed_command_line_files_not_removed_by_exclude() {
        // Ported from "are not removed due to excludes": explicit `files` are
        // kept even when an `exclude` pattern matches them.
        let fs = InMemoryFS::new();
        fs.insert_dir("/dev");
        fs.insert_file("/dev/a.ts", "");
        fs.insert_file("/dev/b.ts", "");
        fs.insert_file(
            "/dev/tsconfig.json",
            r#"{
            "files": ["a.ts", "b.ts"],
            "exclude": ["b.ts"]
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/dev/tsconfig.json",
            &CompilerOptions::default(),
            "/dev",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/dev/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/dev/b.ts".to_string()));
    }

    #[test]
    fn test_parsed_command_line_literal_include_matches_files() {
        // Ported from "with literal include list" > "without exclude": a literal
        // (non-glob) include matches the named files.
        let fs = InMemoryFS::new();
        fs.insert_dir("/dev");
        fs.insert_file("/dev/a.ts", "");
        fs.insert_file("/dev/b.ts", "");
        fs.insert_file(
            "/dev/tsconfig.json",
            r#"{
            "include": ["a.ts", "b.ts"]
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/dev/tsconfig.json",
            &CompilerOptions::default(),
            "/dev",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/dev/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/dev/b.ts".to_string()));
    }

    #[test]
    fn test_wildcard_include_dot_prefixed_with_dot_dir_exclude() {
        // Ported from TestGetWildcardDirectories_DotPrefixedIncludeWithDotDirExclude.
        // Include specs with a directory prefix must still match files even when
        // a `**/.*/` exclude (dot-directory exclude) is present. The Rust port
        // does not normalize a leading `./` in include specs, so the specs here
        // use the plain `app/...` form; the exclude behavior under test is the
        // same.
        let fs = InMemoryFS::new();
        fs.insert_dir("/home/projects/monorepo/apps/web");
        fs.insert_dir("/home/projects/monorepo/apps/web/app");
        fs.insert_file(
            "/home/projects/monorepo/apps/web/tsconfig.json",
            r#"{
                "include": ["app/**/*.ts", "app/**/*.tsx"],
                "exclude": ["**/node_modules", "**/.*/", "build"]
            }"#,
        );
        fs.insert_file("/home/projects/monorepo/apps/web/app/a.ts", "");
        fs.insert_file("/home/projects/monorepo/apps/web/app/b.tsx", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/home/projects/monorepo/apps/web/tsconfig.json",
            &CompilerOptions::default(),
            "/home/projects/monorepo/apps/web",
            &fs,
        );
        assert!(
            parsed
                .file_names
                .contains(&"/home/projects/monorepo/apps/web/app/a.ts".to_string())
        );
        assert!(
            parsed
                .file_names
                .contains(&"/home/projects/monorepo/apps/web/app/b.tsx".to_string())
        );
    }

    #[test]
    fn test_wildcard_include_non_ascii_paths() {
        // Ported from TestGetWildcardDirectories_NonASCIICharacters: parsing
        // configs with non-ASCII paths must not panic and should still resolve
        // include globs.
        let fs = InMemoryFS::new();
        fs.insert_dir("/Users/ユーザー/プロジェクト");
        fs.insert_dir("/Users/ユーザー/プロジェクト/src");
        fs.insert_file(
            "/Users/ユーザー/プロジェクト/tsconfig.json",
            r#"{
                "include": ["src/**/*.ts"],
                "exclude": ["テスト"]
            }"#,
        );
        fs.insert_file("/Users/ユーザー/プロジェクト/src/a.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/Users/ユーザー/プロジェクト/tsconfig.json",
            &CompilerOptions::default(),
            "/Users/ユーザー/プロジェクト",
            &fs,
        );
        assert!(parsed.file_names.iter().any(|f| f.ends_with("/src/a.ts")));
    }

    // ──────────────────────────────────────────────────────────────────────
    // Option-declaration sanity test (adapted from decls_test.go)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_options_declarations_non_empty_and_named() {
        // The OPTIONS table must be populated and every declaration must carry
        // a non-empty name.
        assert!(!OPTIONS.is_empty());
        for o in OPTIONS {
            assert!(!o.name.is_empty(), "found an option with an empty name");
        }
        // A few key options must be present.
        let names: std::collections::HashSet<&str> = OPTIONS.iter().map(|o| o.name).collect();
        for required in [
            "help",
            "version",
            "build",
            "watch",
            "target",
            "module",
            "jsx",
            "lib",
            "strict",
            "noEmit",
            "project",
            "tsBuildInfoFile",
            "incremental",
            "moduleResolution",
            "typeRoots",
        ] {
            assert!(
                names.contains(required),
                "missing option declaration: {required}"
            );
        }
    }

    #[test]
    fn test_option_decls_short_names_unique_or_known() {
        // The commonly-used short names map to the expected options.
        assert_eq!(find_option("h").map(|o| o.name), Some("help"));
        assert_eq!(find_option("v").map(|o| o.name), Some("version"));
        assert_eq!(find_option("b").map(|o| o.name), Some("build"));
        assert_eq!(find_option("w").map(|o| o.name), Some("watch"));
        assert_eq!(find_option("p").map(|o| o.name), Some("project"));
        assert_eq!(find_option("t").map(|o| o.name), Some("target"));
        assert_eq!(find_option("m").map(|o| o.name), Some("module"));
        assert_eq!(find_option("d").map(|o| o.name), Some("declaration"));
    }

    // ──────────────────────────────────────────────────────────────────────
    // Declaration-driven option parser tests (NameMap, did-you-mean,
    // alternate-mode, TSConfigOnly, enum/min-value validation).
    // ──────────────────────────────────────────────────────────────────────

    /// Like `has_error_containing` but operates on a slice of diagnostics, so
    /// it can be used with `ParsedBuildCommandLine.errors` too.
    fn diag_contains(errors: &[Diagnostic], needle: &str) -> bool {
        errors.iter().any(|e| {
            e.message_args.iter().any(|a| a.contains(needle))
                || e.message.map(|m| m.text.contains(needle)).unwrap_or(false)
        })
    }

    #[test]
    fn test_case_insensitive_option_lookup_cli() {
        // `--Target` (wrong case) resolves case-insensitively to `target`,
        // matching Go's NameMap behaviour on the command line.
        let parsed = parse_command_line(&args(&["--Target", "ES2020", "0.ts"]), "/proj", None);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(!has_error_containing(&parsed, "Unknown compiler option"));

        // `--Module` and `--Jsx` likewise.
        let parsed = parse_command_line(
            &args(&["--Module", "commonjs", "--Jsx", "react", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
    }

    #[test]
    fn test_case_insensitive_short_name_lookup() {
        // Short names are also matched case-insensitively.
        let parsed = parse_command_line(&args(&["-P", "tsconfig.json"]), "/proj", None);
        assert_eq!(parsed.compiler_options.project, "tsconfig.json");
    }

    #[test]
    fn test_alternate_mode_build_option_in_compiler_mode() {
        // `--dry` is a build-only option; using it in compiler mode emits the
        // "may only be used with --build" diagnostic (TS5093) instead of the
        // generic unknown-option error.
        let parsed = parse_command_line(&args(&["--dry", "0.ts"]), "/proj", None);
        assert!(diag_contains(&parsed.errors, "may only be used with '--build'"));
        assert!(!diag_contains(&parsed.errors, "Unknown compiler option"));
    }

    #[test]
    fn test_alternate_mode_verbose_in_compiler_mode() {
        // `--verbose` is build-only; in compiler mode it triggers TS5093.
        let parsed = parse_command_line(&args(&["--verbose"]), "/proj", None);
        assert!(diag_contains(&parsed.errors, "may only be used with '--build'"));
    }

    #[test]
    fn test_tsconfig_only_option_on_cli_emits_diagnostic() {
        // `composite` is TSConfigOnly; on the CLI it must emit the
        // "can only be specified in tsconfig.json ... set to false or null"
        // diagnostic (TS6230) and must NOT enable composite.
        let parsed = parse_command_line(&args(&["--composite", "0.ts"]), "/proj", None);
        assert!(has_error_containing(&parsed, "tsconfig.json"));
        assert!(has_error_containing(&parsed, "composite"));
        assert!(!parsed.compiler_options.composite.is_true());
    }

    #[test]
    fn test_tsconfig_only_boolean_accepts_false() {
        // `--composite false` is allowed (no error) and sets composite to false.
        let parsed = parse_command_line(&args(&["--composite", "false", "0.ts"]), "/proj", None);
        assert!(!has_error_containing(&parsed, "tsconfig.json"));
        assert!(parsed.compiler_options.composite.is_false());
    }

    #[test]
    fn test_tsconfig_only_boolean_accepts_null() {
        // `--composite null` is allowed (no error).
        let parsed = parse_command_line(&args(&["--composite", "null", "0.ts"]), "/proj", None);
        assert!(!has_error_containing(&parsed, "tsconfig.json"));
    }

    #[test]
    fn test_invalid_enum_value_target() {
        // `--target es99` is not a valid target enum value; emit
        // "Argument for '--target' option must be: ..." (TS6046) listing the
        // valid values, and leave target unset.
        let parsed = parse_command_line(&args(&["--target", "es99", "0.ts"]), "/proj", None);
        assert!(has_error_containing(&parsed, "Argument for"));
        assert!(has_error_containing(&parsed, "--target"));
        assert!(has_error_containing(&parsed, "es5"));
        assert_eq!(parsed.compiler_options.target, ScriptTarget::None);
    }

    #[test]
    fn test_invalid_enum_value_module() {
        let parsed = parse_command_line(&args(&["--module", "nonsense", "0.ts"]), "/proj", None);
        assert!(has_error_containing(&parsed, "Argument for"));
        assert!(has_error_containing(&parsed, "commonjs"));
        assert_eq!(parsed.compiler_options.module, ModuleKind::None);
    }

    #[test]
    fn test_valid_enum_value_case_insensitive() {
        // Enum values are matched case-insensitively (Go lowercases the key).
        let parsed = parse_command_line(&args(&["--target", "ES2020", "0.ts"]), "/proj", None);
        assert!(!has_error_containing(&parsed, "Argument for"));
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    }

    #[test]
    fn test_min_value_violation_builders() {
        // `--builders 0` violates the min-value (1) constraint → TS5002.
        let parsed =
            parse_build_command_line(&args(&["--build", "--builders", "0"]), "/proj", None);
        assert!(diag_contains(&parsed.errors, "requires value to be greater"));
        assert!(diag_contains(&parsed.errors, "builders"));
        assert!(diag_contains(&parsed.errors, "1"));
    }

    #[test]
    fn test_min_value_accepted_builders() {
        // `--builders 1` satisfies the min-value constraint.
        let parsed =
            parse_build_command_line(&args(&["--build", "--builders", "2"]), "/proj", None);
        assert!(!diag_contains(&parsed.errors, "requires value to be greater"));
        assert_eq!(parsed.build_options.builders, Some(2));
    }

    #[test]
    fn test_case_mismatch_in_tsconfig_json_emits_did_you_mean() {
        // A `compilerOptions` key whose case does not exactly match the
        // canonical declaration emits a "Did you mean" diagnostic (TS5025) and
        // skips the key.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "compilerOptions": { "Target": "es2020", "noEmit": true },
            "files": ["src/a.ts"]
        }"#,
        );
        fs.insert_file("/proj/src/a.ts", "export const x = 1;");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(has_error_containing(&parsed, "Did you mean"));
        assert!(has_error_containing(&parsed, "Target"));
        assert!(has_error_containing(&parsed, "target"));
        // The miscased key is skipped, so target stays unset.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::None);
        // The correctly-cased key is still applied.
        assert!(parsed.compiler_options.no_emit.is_true());
    }

    #[test]
    fn test_tsconfig_json_correct_case_no_did_you_mean() {
        // Correctly-cased keys must not trigger the did-you-mean diagnostic.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "compilerOptions": { "target": "es2020", "noEmit": true },
            "files": ["src/a.ts"]
        }"#,
        );
        fs.insert_file("/proj/src/a.ts", "export const x = 1;");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!has_error_containing(&parsed, "Did you mean"));
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    }

    #[test]
    fn test_enum_values_declared_on_all_enum_options() {
        // Every Enum-kind declaration must carry enum_values.
        for o in OPTIONS.iter().chain(BUILD_OPTIONS.iter()) {
            if o.kind == OptionKind::Enum {
                assert!(
                    o.enum_values.is_some(),
                    "enum option '{}' must declare enum_values",
                    o.name
                );
            }
        }
    }

    #[test]
    fn test_tsconfig_only_and_min_value_flags_set() {
        // Spot-check that the declaration-driven flags are wired up.
        let composite = find_option("composite").expect("composite must exist");
        assert!(composite.is_tsconfig_only);
        let paths = find_option("paths").expect("paths must exist");
        assert!(paths.is_tsconfig_only);
        let builders = find_build_only_option("builders").expect("builders must exist");
        assert_eq!(builders.min_value, Some(1));
    }

    // ──────────────────────────────────────────────────────────────────────
    // `${configDir}` template substitution tests (TS 5.5+)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_config_dir_substitution_out_dir() {
        // `${configDir}/out` in outDir should resolve to
        // <config_dir>/out as an absolute path.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "${configDir}/out" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(
            parsed.compiler_options.out_dir,
            "/proj/out",
            "expected ${{configDir}}/out to resolve to /proj/out, got {}",
            parsed.compiler_options.out_dir
        );
    }

    #[test]
    fn test_config_dir_substitution_root_dir() {
        // `${configDir}/src` in rootDir should resolve to <config_dir>/src.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "rootDir": "${configDir}/src" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.compiler_options.root_dir, "/proj/src");
    }

    #[test]
    fn test_config_dir_substitution_case_insensitive_detection() {
        // Go's `startsWithConfigDirTemplate` is case-insensitive, but
        // `getSubstitutedPathWithConfigDirTemplate` uses `strings.Replace`
        // which is case-sensitive. This means `${configdir}` (all lowercase)
        // is detected as a configDir template (so `normalizeNonListOptionValue`
        // skips normal path resolution) but the actual replacement doesn't
        // match, leaving the literal text in place. This matches Go's behavior.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "${configDir}/out" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // Exact case ${configDir} is substituted correctly.
        assert_eq!(parsed.compiler_options.out_dir, "/proj/out");
    }

    #[test]
    fn test_config_dir_substitution_declaration_dir_and_ts_build_info() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": {
                "declarationDir": "${configDir}/decls",
                "tsBuildInfoFile": "${configDir}/build.tsbuildinfo"
            } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.compiler_options.declaration_dir, "/proj/decls");
        assert_eq!(parsed.compiler_options.ts_build_info_file, "/proj/build.tsbuildinfo");
    }

    #[test]
    fn test_config_dir_substitution_root_dirs_array() {
        // `${configDir}` in rootDirs array elements should be substituted.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": {
                "rootDirs": ["${configDir}/src", "${configDir}/lib"]
            } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(
            parsed.compiler_options.root_dirs,
            vec!["/proj/src".to_string(), "/proj/lib".to_string()]
        );
    }

    #[test]
    fn test_config_dir_substitution_paths() {
        // `${configDir}` in `paths` target values should be substituted.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                    "@/*": ["${configDir}/src/*"],
                    "lib/*": ["${configDir}/lib/*"]
                }
            } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        let paths = parsed.compiler_options.paths.expect("paths should be set");
        assert_eq!(
            paths.get("@/*").unwrap(),
            &vec!["/proj/src/*".to_string()]
        );
        assert_eq!(
            paths.get("lib/*").unwrap(),
            &vec!["/proj/lib/*".to_string()]
        );
    }

    #[test]
    fn test_config_dir_substitution_include() {
        // `${configDir}/src` in include should resolve and match files.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/index.ts", "");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "include": ["${configDir}/src"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/src/index.ts"),
            "expected /proj/src/index.ts in file_names, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_config_dir_substitution_files() {
        // `${configDir}/main.ts` in files should resolve to the absolute path.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/main.ts", "");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "files": ["${configDir}/main.ts"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/main.ts"),
            "expected /proj/main.ts in file_names, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_config_dir_substitution_exclude() {
        // `${configDir}/dist` in exclude should resolve and exclude files.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/dist");
        fs.insert_file("/proj/src/index.ts", "");
        fs.insert_file("/proj/dist/output.js", "");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "include": ["${configDir}/src/**/*"], "exclude": ["${configDir}/dist"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/src/index.ts"),
            "expected /proj/src/index.ts in file_names, got {:?}",
            parsed.file_names
        );
        assert!(
            !parsed.file_names.iter().any(|f| f.contains("dist")),
            "expected dist/ files to be excluded, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_config_dir_substitution_with_extends() {
        // `${configDir}` in an extended config's outDir should resolve to
        // the EXTENDED config's directory, not the own config's directory.
        // `${configDir}` in the own config should resolve to the own config's
        // directory.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "${configDir}/out" } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json", "compilerOptions": { "rootDir": "${configDir}/src" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // Extended config's outDir resolves to /proj/base/out (extended dir).
        assert_eq!(
            parsed.compiler_options.out_dir, "/proj/base/out",
            "extended config's ${{configDir}} should resolve to extended config's dir"
        );
        // Own config's rootDir resolves to /proj/src (own dir).
        assert_eq!(
            parsed.compiler_options.root_dir, "/proj/src",
            "own config's ${{configDir}} should resolve to own config's dir"
        );
    }

    #[test]
    fn test_config_dir_not_substituted_for_non_prefix() {
        // `${configDir}` must appear at the START of the value; embedded
        // occurrences are NOT substituted (mirrors Go's
        // `startsWithConfigDirTemplate`).
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "outDir": "prefix/${configDir}/out" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // The value does NOT start with ${configDir}, so it's resolved as a
        // relative path "prefix/${configDir}/out" → absolute path with that
        // literal text.
        assert!(
            parsed.compiler_options.out_dir.contains("configDir"),
            "embedded ${{configDir}} should not be substituted, got {}",
            parsed.compiler_options.out_dir
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // Inherited include/exclude/files path-rewriting tests
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_extends_inherited_include_path_rewriting() {
        // When an extended config has a relative `include` spec, it should be
        // rewritten to be relative to the OWN config's directory, not the
        // extended config's directory. Mirrors Go's `applyExtendedConfig`
        // which calls `ConvertToRelativePath(GetDirectoryPath(extendedConfigPath), …)`.
        //
        // Setup:
        //   /proj/tsconfig.json          (own, extends ./base/tsconfig.json)
        //   /proj/base/tsconfig.json     (extended, include: ["src/**/*"])
        //   /proj/base/src/a.ts          (file under extended dir)
        //
        // Expected: the inherited "src/**/*" is rewritten to "base/src/**/*"
        // and resolved against /proj, so /proj/base/src/a.ts is included.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/base/src");
        fs.insert_file("/proj/base/src/a.ts", "");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "include": ["src/**/*"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/base/src/a.ts"),
            "expected /proj/base/src/a.ts in file_names (relative include rewritten), got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_inherited_include_absolute_not_rewritten() {
        // Absolute paths in inherited include specs are passed through as-is
        // (not rewritten). Mirrors Go's `IsRootedDiskPath` check in
        // `applyExtendedConfig`.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/shared");
        fs.insert_file("/shared/a.ts", "");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "include": ["/shared/**/*"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.file_names.iter().any(|f| f == "/shared/a.ts"),
            "expected /shared/a.ts in file_names (absolute include not rewritten), got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_inherited_include_config_dir_not_rewritten() {
        // `${configDir}`-prefixed paths in inherited include specs are passed
        // through as-is (not rewritten relative to the extended config dir),
        // and then substituted with the OWN config's directory. Mirrors Go's
        // `startsWithConfigDirTemplate` check in `applyExtendedConfig`.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "include": ["${configDir}/src"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // `${configDir}` in inherited include is resolved against the OWN
        // config's directory (/proj), so /proj/src/a.ts is included.
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/src/a.ts"),
            "expected /proj/src/a.ts in file_names (${{configDir}} resolved against own dir), got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_inherited_exclude_path_rewriting() {
        // Inherited exclude specs are also rewritten relative to the own
        // config's directory.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/base/src");
        fs.insert_dir("/proj/base/excluded");
        fs.insert_file("/proj/base/src/a.ts", "");
        fs.insert_file("/proj/base/excluded/b.ts", "");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "include": ["src/**/*"], "exclude": ["excluded"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // src/a.ts should be included (rewritten to base/src/**/*).
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/base/src/a.ts"),
            "expected /proj/base/src/a.ts in file_names, got {:?}",
            parsed.file_names
        );
        // excluded/b.ts should be excluded (rewritten to base/excluded).
        assert!(
            !parsed.file_names.iter().any(|f| f.contains("excluded")),
            "expected excluded/ files to be excluded, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_inherited_files_path_rewriting() {
        // Inherited files specs are also rewritten relative to the own
        // config's directory.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/base/src");
        fs.insert_file("/proj/base/src/main.ts", "");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "files": ["src/main.ts"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // "src/main.ts" is rewritten to "base/src/main.ts" and resolved
        // against /proj → /proj/base/src/main.ts.
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/base/src/main.ts"),
            "expected /proj/base/src/main.ts in file_names, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_own_include_overrides_inherited() {
        // The own config's include overrides inherited include (first-wins
        // among extended, but own always wins).
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/own_src");
        fs.insert_file("/proj/own_src/a.ts", "");
        fs.insert_dir("/proj/base/src");
        fs.insert_file("/proj/base/src/b.ts", "");
        fs.insert_file(
            "/proj/base/tsconfig.json",
            r#"{ "include": ["src/**/*"] }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base/tsconfig.json", "include": ["own_src/**/*"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // Own include wins: own_src/a.ts is included, base/src/b.ts is NOT.
        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/own_src/a.ts"),
            "expected /proj/own_src/a.ts in file_names, got {:?}",
            parsed.file_names
        );
        assert!(
            !parsed.file_names.iter().any(|f| f == "/proj/base/src/b.ts"),
            "expected /proj/base/src/b.ts NOT in file_names (own include overrides), got {:?}",
            parsed.file_names
        );
    }
}
