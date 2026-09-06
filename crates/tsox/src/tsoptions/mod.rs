use std::collections::{HashMap, HashSet};

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
    COMPILER_OPTION_0_MAY_NOT_BE_USED_WITH_BUILD, COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
    NO_INPUTS_WERE_FOUND_IN_CONFIG_FILE_0_SPECIFIED_INCLUDE_PATHS_WERE_1_AND_EXCLUDE_PATHS_WERE_2,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_FALSE_OR_NULL_ON_COMMAND_LINE,
    OPTION_0_CAN_ONLY_BE_SPECIFIED_IN_TSCONFIG_JSON_FILE_OR_SET_TO_NULL_ON_COMMAND_LINE,
    OPTION_0_REQUIRES_VALUE_TO_BE_GREATER_THAN_1, OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
    UNKNOWN_BUILD_OPTION_0, UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1, UNKNOWN_COMPILER_OPTION_0,
    UNKNOWN_COMPILER_OPTION_0_DID_YOU_MEAN_1, UNTERMINATED_QUOTED_STRING_IN_RESPONSE_FILE_0,
    WATCH_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1, new_ad_hoc_message,
};
use crate::glob::Glob;
use crate::tspath;
use crate::vfs::FS;

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

static TARGET_ENUM_VALUES: &[&str] = &[
    "es3", "es5", "es6", "es2015", "es2016", "es2017", "es2018", "es2019", "es2020", "es2021",
    "es2022", "es2023", "es2024", "es2025", "esnext",
];
static MODULE_ENUM_VALUES: &[&str] = &[
    "commonjs", "amd", "system", "umd", "es6", "es2015", "es2020", "es2022", "esnext", "node16",
    "node18", "node20", "nodenext", "preserve",
];
static MODULE_RESOLUTION_ENUM_VALUES: &[&str] =
    &["node16", "nodenext", "bundler", "classic", "node", "node10"];
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    Compiler,
    Build,
}

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

fn decl_matches(o: &OptionDecl, name: &str) -> bool {
    o.name.eq_ignore_ascii_case(name)
        || o.short_name
            .map(|s| s.eq_ignore_ascii_case(name))
            .unwrap_or(false)
}

fn find_option(name: &str) -> Option<&'static OptionDecl> {
    OPTIONS.iter().find(|o| decl_matches(o, name))
}

fn find_build_only_option(name: &str) -> Option<&'static OptionDecl> {
    BUILD_OPTIONS.iter().find(|o| decl_matches(o, name))
}

fn find_build_option(name: &str) -> Option<&'static OptionDecl> {
    BUILD_OPTIONS
        .iter()
        .chain(OPTIONS.iter())
        .find(|o| decl_matches(o, name))
}

fn did_you_mean_build_option(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let mut best: Option<(usize, &str)> = None;
    for opt in BUILD_OPTIONS.iter().chain(OPTIONS.iter()) {
        let name = opt.name.to_lowercase();
        let dist = levenshtein(&input_lower, &name);

        if dist <= 3 && best.map_or(true, |(d, _)| dist < d) {
            best = Some((dist, opt.name));
        }
    }
    best.map(|(_, name)| name.to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

fn find_watch_option(name: &str) -> Option<&'static OptionDecl> {
    OPTIONS_FOR_WATCH.iter().find(|o| decl_matches(o, name))
}

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

#[derive(Debug, Clone, Default)]
pub struct ParsedCommandLine {
    pub compiler_options: CompilerOptions,
    pub file_names: Vec<String>,
    pub errors: Vec<Diagnostic>,
    pub config_file_name: String,

    pub raw_options: Option<crate::json::Value>,

    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub files_spec: Vec<String>,
    pub has_include_spec: bool,
    pub has_exclude_spec: bool,
    pub has_files_spec: bool,
    pub references: Vec<crate::core::project_reference::ProjectReference>,
    pub compile_on_save: Option<bool>,
    pub watch: bool,

    pub watch_options: WatchOptions,
}

#[derive(Default)]
pub struct ExtendedConfigCache {
    entries: HashMap<String, ParsedCommandLine>,
}

impl ExtendedConfigCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn get_or_parse(
        &mut self,
        resolved_path: &str,
        config_file_name: &str,
        current_dir: &str,
        fs: &dyn FS,
        resolution_stack: &[String],
    ) -> ParsedCommandLine {

        if resolution_stack.iter().any(|p| p == resolved_path) {
            return get_parsed_command_line_of_config_file_with_stack(
                config_file_name,
                &CompilerOptions::default(),
                current_dir,
                fs,
                resolution_stack,
                self,
            );
        }
        if let Some(cached) = self.entries.get(resolved_path) {
            return cached.clone();
        }
        let parsed = get_parsed_command_line_of_config_file_with_stack(
            config_file_name,
            &CompilerOptions::default(),
            current_dir,
            fs,
            resolution_stack,
            self,
        );
        self.entries
            .insert(resolved_path.to_string(), parsed.clone());
        parsed
    }
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

                let response_path = &s[1..];
                let abs = tspath::get_normalized_absolute_path(response_path, current_dir);
                if let Some(fs) = fs {
                    if let Some(content) = fs.read_file(&abs) {
                        let (response_args, split_errors) = split_response_file(&content, &abs);
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

                let name_part = s.trim_start_matches('-');

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

                        if mode == ParseMode::Compiler && find_build_only_option(name).is_some() {
                            errors.push(Diagnostic::new(
                                None,
                                TextRange::undefined(),
                                COMPILER_OPTION_0_MAY_ONLY_BE_USED_WITH_BUILD,
                                vec![name.to_string()],
                            ));
                            continue;
                        }

                        if mode == ParseMode::Build {

                            if find_option(name).is_some() {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    COMPILER_OPTION_0_MAY_NOT_BE_USED_WITH_BUILD,
                                    vec![name.to_string()],
                                ));
                                continue;
                            }

                            let suggestion = did_you_mean_build_option(name);
                            if let Some(s) = suggestion {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    UNKNOWN_BUILD_OPTION_0_DID_YOU_MEAN_1,
                                    vec![name.to_string(), s],
                                ));
                            } else {
                                errors.push(Diagnostic::new(
                                    None,
                                    TextRange::undefined(),
                                    UNKNOWN_BUILD_OPTION_0,
                                    vec![name.to_string()],
                                ));
                            }
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
                                vec![opt.name.to_string(), type_name(opt.kind).to_string()],
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

fn set_bool(options: &mut CompilerOptions, name: &str, b: bool) {
    let t = Tristate::from(b);

    let name = name.to_ascii_lowercase();
    match name.as_str() {
        "noemit" => options.no_emit = t,
        "nocheck" => options.no_check = t,
        "nolib" => options.no_lib = t,
        "skiplibcheck" => options.skip_lib_check = t,
        "skipdefaultlibcheck" => options.skip_default_lib_check = t,
        "strictnullchecks" => options.strict_null_checks = t,
        "strictfunctiontypes" => options.strict_function_types = t,
        "strictbindcallapply" => options.strict_bind_call_apply = t,
        "strictpropertyinitialization" => options.strict_property_initialization = t,
        "strictbuiltiniteratorreturn" => options.strict_builtin_iterator_return = t,
        "noimplicitany" => options.no_implicit_any = t,
        "noimplicitthis" => options.no_implicit_this = t,
        "noimplicitoverride" => options.no_implicit_override = t,
        "nounusedlocals" => options.no_unused_locals = t,
        "nounusedparameters" => options.no_unused_parameters = t,
        "nofallthroughcasesinswitch" => options.no_fallthrough_cases_in_switch = t,
        "nouncheckedindexedaccess" => options.no_unchecked_indexed_access = t,
        "nopropertyaccessfromindexsignature" => options.no_property_access_from_index_signature = t,
        "noerrortruncation" => options.no_error_truncation = t,
        "noemitonerror" => options.no_emit_on_error = t,
        "noresolve" => options.no_resolve = t,
        "useunknownincatchvariables" => options.use_unknown_in_catch_variables = t,
        "exactoptionalpropertytypes" => options.exact_optional_property_types = t,
        "esmoduleinterop" => options.es_module_interop = t,
        "allowsyntheticdefaultimports" => options.allow_synthetic_default_imports = t,
        "allowjs" => options.allow_js = t,
        "alwaysstrict" => options.always_strict = t,
        "checkjs" => options.check_js = t,
        "composite" => options.composite = t,
        "declaration" => options.declaration = t,
        "declarationmap" => options.declaration_map = t,
        "emitdeclarationonly" => options.emit_declaration_only = t,
        "sourcemap" => options.source_map = t,
        "inlinesourcemap" => options.inline_source_map = t,
        "inlinesources" => options.inline_sources = t,
        "removecomments" => options.remove_comments = t,
        "isolatedmodules" => options.isolated_modules = t,
        "isolateddeclarations" => options.isolated_declarations = t,
        "verbatimmodulesyntax" => options.verbatim_module_syntax = t,
        "preserveconstenums" => options.preserve_const_enums = t,
        "importhelpers" => options.import_helpers = t,
        "experimentaldecorators" => options.experimental_decorators = t,
        "emitdecoratormetadata" => options.emit_decorator_metadata = t,
        "forceconsistentcasinginfilenames" => options.force_consistent_casing_in_file_names = t,
        "listfiles" => options.list_files = t,
        "listfilesonly" => options.list_files_only = t,
        "listemittedfiles" => options.list_emitted_files = t,
        "explainfiles" => options.explain_files = t,
        "extendeddiagnostics" => options.extended_diagnostics = t,
        "diagnostics" => options.diagnostics = t,
        "pretty" => options.pretty = t,
        "showconfig" => options.show_config = t,
        "ignoreconfig" => options.ignore_config = t,
        "incremental" => options.incremental = t,
        "watch" => options.watch = t,
        "version" => options.version = t,
        "help" => options.help = t,
        "all" => options.all = t,
        "init" => options.init = t,
        "build" => options.build = t,
        "singlethreaded" => options.single_threaded = t,
        "quiet" => options.quiet = t,
        "strict" => {
            options.strict = t;

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

pub fn apply_test_settings(settings: &HashMap<String, String>) -> (CompilerOptions, Vec<String>) {
    apply_test_settings_with_base(settings, CompilerOptions::default())
}

pub fn apply_test_settings_with_base(
    settings: &HashMap<String, String>,
    base: CompilerOptions,
) -> (CompilerOptions, Vec<String>) {

    const KNOWN_BOOL_OPTIONS: &[&str] = &[
        "noemit",
        "nocheck",
        "nolib",
        "skiplibcheck",
        "skipdefaultlibcheck",
        "strictnullchecks",
        "strictfunctiontypes",
        "strictbindcallapply",
        "strictpropertyinitialization",
        "strictbuiltiniteratorreturn",
        "noimplicitany",
        "noimplicitthis",
        "noimplicitoverride",
        "nounsusedlocals",
        "nounsusedparameters",
        "nofallthroughcasesinswitch",
        "nouncheckedindexedaccess",
        "nopropertyaccessfromindexsignature",
        "noerrortruncation",
        "noemitonerror",
        "noresolve",
        "useunknownincatchvariables",
        "exactoptionalpropertytypes",
        "esmoduleinterop",
        "allowsyntheticdefaultimports",
        "allowjs",
        "checkjs",
        "composite",
        "declaration",
        "declarationmap",
        "emitdeclarationonly",
        "sourcemap",
        "inlinesourcemap",
        "inlinesources",
        "removecomments",
        "isolatedmodules",
        "isolateddeclarations",
        "verbatimmodulesyntax",
        "preserveconstenums",
        "importhelpers",
        "experimentaldecorators",
        "emitdecoratormetadata",
        "forceconsistencingcasingfilenames",
        "listfiles",
        "listfilesonly",
        "listemittedfiles",
        "explainfiles",
        "extendeddiagnostics",
        "diagnostics",
        "pretty",
        "showconfig",
        "ignoreconfig",
        "incremental",
        "watch",
        "version",
        "help",
        "all",
        "init",
        "build",
        "singlethreaded",
        "quiet",
        "strict",
        "alwaysstrict",
    ];
    const KNOWN_STR_OPTIONS: &[&str] = &[
        "target",
        "module",
        "moduleresolution",
        "jsx",
        "newline",
        "moduledetection",
        "outdir",
        "outfile",
        "rootdir",
        "declarationdir",
        "tsbuildinfofile",
        "sourceroot",
        "maproot",
        "jsxfactory",
        "jsxfragmentfactory",
        "jsximportsource",
        "reactnamespace",
        "locale",
        "baseurl",
        "modulosuffixes",
        "customconditions",
        "jsxmode",
    ];
    const KNOWN_LIST_OPTIONS: &[&str] = &["lib", "types", "typeroots", "rootdirs"];

    let mut options = base;
    let mut unrecognized: Vec<String> = Vec::new();

    let has_strict_directive = settings
        .keys()
        .any(|k| k.eq_ignore_ascii_case("strict"));
    let has_nia_directive = settings
        .keys()
        .any(|k| k.eq_ignore_ascii_case("noimplicitany"));
    if !has_strict_directive && !has_nia_directive && options.no_implicit_any.is_unknown() {
        options.no_implicit_any = crate::core::tristate::Tristate::True;
    }

    for (name, raw_value) in settings {
        let lower = name.to_lowercase();
        let trimmed = raw_value.trim().trim_end_matches(';').to_string();

        let known = KNOWN_BOOL_OPTIONS.contains(&lower.as_str())
            || KNOWN_STR_OPTIONS.contains(&lower.as_str())
            || KNOWN_LIST_OPTIONS.contains(&lower.as_str());

        if !known {
            unrecognized.push(name.clone());
            continue;
        }

        let is_bool_val = matches!(trimmed.as_str(), "true" | "false")
            && KNOWN_BOOL_OPTIONS.contains(&lower.as_str());

        let canonical = find_option(&lower)
            .map(|o| o.name.to_string())
            .unwrap_or_else(|| lower.clone());
        if is_bool_val {
            set_bool(&mut options, &lower, trimmed.eq_ignore_ascii_case("true"));
        } else if KNOWN_LIST_OPTIONS.contains(&lower.as_str()) {
            let list: Vec<String> = trimmed
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let mut map = HashMap::new();
            map.insert(canonical, OptValue::List(list));
            apply_options(&map, &mut options);
        } else {
            let mut map = HashMap::new();
            map.insert(canonical, OptValue::Str(trimmed.clone()));
            apply_options(&map, &mut options);
        }
    }

    (options, unrecognized)
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
                    out.directory_kind =
                        parse_watch_directory_kind(s).unwrap_or(WatchDirectoryKind::None);
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

pub fn module_detection_name(d: ModuleDetectionKind) -> Option<&'static str> {
    match d {
        ModuleDetectionKind::Auto => Some("auto"),
        ModuleDetectionKind::Force => Some("force"),
        ModuleDetectionKind::Legacy => Some("legacy"),
        ModuleDetectionKind::None => None,
    }
}

pub fn new_line_name(n: NewLineKind) -> Option<&'static str> {
    match n {
        NewLineKind::CRLF => Some("crlf"),
        NewLineKind::LF => Some("lf"),
        NewLineKind::None => None,
    }
}

pub fn get_parsed_command_line_of_config_file(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
) -> ParsedCommandLine {
    let mut cache = ExtendedConfigCache::new();
    get_parsed_command_line_of_config_file_with_stack(
        config_file_name,
        base_options,
        current_dir,
        fs,
        &[],
        &mut cache,
    )
}

fn get_parsed_command_line_of_config_file_with_stack(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
    resolution_stack: &[String],
    cache: &mut ExtendedConfigCache,
) -> ParsedCommandLine {
    let mut result = ParsedCommandLine::default();
    result.compiler_options = base_options.clone();
    result.config_file_name = config_file_name.to_string();

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

    let mut extended_opts = CompilerOptions::default();
    if let Some(extends) = root_obj.get("extends") {
        let extends_paths = extends_as_paths(extends, config_file_name, current_dir, fs);
        if !extends_paths.is_empty() {
            let mut new_stack: Vec<String> = resolution_stack.to_vec();
            new_stack.push(resolved_path.clone());

            let mut extended_configs: Vec<(String, ParsedCommandLine)> = Vec::new();
            for ext_path in &extends_paths {

                let ext_resolved = tspath::get_normalized_absolute_path(ext_path, current_dir);
                let parent =
                    cache.get_or_parse(&ext_resolved, ext_path, current_dir, fs, &new_stack);
                extended_configs.push((ext_path.clone(), parent));
            }

            for (_, parent) in extended_configs.iter().rev() {
                merge_compiler_options(&mut extended_opts, &parent.compiler_options);
            }

            let own_config_dir = tspath::get_directory_path(config_file_name);
            let compare_opts = tspath::ComparePathsOptions {
                use_case_sensitive_file_names: fs.use_case_sensitive_file_names(),
                current_directory: own_config_dir.clone(),
            };
            for (ext_path, parent) in &extended_configs {
                let ext_dir = tspath::get_directory_path(ext_path);
                let relative_difference = tspath::convert_to_relative_path(&ext_dir, &compare_opts);
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

    if let Some(files) = root_obj.get("files").and_then(|v| v.as_array()) {
        result.has_files_spec = true;
        result.files_spec.clear();
        for f in files {
            if let Some(s) = f.as_str() {
                result.files_spec.push(s.to_string());
            }
        }
    }

    if let Some(include) = root_obj.get("include").and_then(|v| v.as_array()) {
        result.has_include_spec = true;
        result.include.clear();
        for f in include {
            if let Some(s) = f.as_str() {
                result.include.push(s.to_string());
            }
        }
    }

    if let Some(exclude) = root_obj.get("exclude").and_then(|v| v.as_array()) {
        result.has_exclude_spec = true;
        result.exclude.clear();
        for f in exclude {
            if let Some(s) = f.as_str() {
                result.exclude.push(s.to_string());
            }
        }
    }

    let mut explicit_null_fields: HashSet<String> = HashSet::new();
    if let Some(co) = root_obj.get("compilerOptions").and_then(|v| v.as_object()) {
        for (key, value) in co {
            if value.is_null() {
                explicit_null_fields.insert(key.clone());
            }
        }
    }

    if let Some(co) = root_obj.get("compilerOptions").and_then(|v| v.as_object()) {
        result.raw_options = Some(crate::json::Value::Object(co.clone()));
        let (opts, opts_errors) = json_object_to_options(co);
        result.errors.extend(opts_errors);

        let mut config_opts = CompilerOptions::default();
        apply_options(&opts, &mut config_opts);

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

        let config_dir_for_opts = tspath::get_directory_path(config_file_name);
        resolve_file_path_options(&mut config_opts, &config_dir_for_opts);

        merge_compiler_options(&mut result.compiler_options, &config_opts);
        merge_compiler_options_with_skip(
            &mut result.compiler_options,
            &extended_opts,
            &explicit_null_fields,
        );
    } else {

        merge_compiler_options(&mut result.compiler_options, &extended_opts);
    }

    let config_dir = tspath::get_directory_path(config_file_name);
    handle_config_dir_template_substitution(&mut result.compiler_options, &config_dir);

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

    if result.file_names.is_empty() && resolution_stack.is_empty() {
        let can_report = !root_obj.contains_key("files") && !root_obj.contains_key("references");
        if can_report {
            let include_json =
                serde_json::to_string(&result.include).unwrap_or_else(|_| "[]".into());
            let exclude_json =
                serde_json::to_string(&result.exclude).unwrap_or_else(|_| "[]".into());
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

fn extends_as_paths(
    extends: &crate::json::Value,
    config_file_name: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Vec<String> {

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

    if tspath::is_external_module_name_relative(s) {
        resolve_relative_extends_path(s, &config_dir, current_dir, fs)
    } else {
        resolve_config_via_node_modules(s, &config_dir, fs)
    }
}

fn resolve_relative_extends_path(
    s: &str,
    config_dir: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Option<String> {

    let base = tspath::normalize_path(&tspath::combine_paths(&config_dir, &[s]));

    if fs.file_exists(&base) {
        return Some(base);
    }

    if !base.ends_with(".json") {
        let with_json = format!("{base}.json");
        if fs.file_exists(&with_json) {
            return Some(with_json);
        }
    }

    let dir_form = tspath::combine_paths(&base, &["tsconfig.json"]);
    if fs.file_exists(&dir_form) {
        return Some(dir_form);
    }

    let abs = tspath::get_normalized_absolute_path(s, current_dir);
    if fs.file_exists(&abs) {
        Some(abs)
    } else {
        Some(tspath::combine_paths(&abs, &["tsconfig.json"]))
    }
}

fn resolve_config_via_node_modules(
    module_name: &str,
    containing_directory: &str,
    fs: &dyn FS,
) -> Option<String> {
    let mut result: Option<String> = None;
    tspath::for_each_ancestor_directory(containing_directory, |ancestor| {

        if tspath::get_base_file_name(ancestor) == "node_modules" {
            return false;
        }
        let node_modules = tspath::combine_paths(ancestor, &["node_modules"]);
        if !fs.directory_exists(&node_modules) {
            return false;
        }
        if let Some(resolved) = load_config_from_node_modules(module_name, &node_modules, fs) {
            result = Some(resolved);
            return true;
        }
        false
    });
    result
}

fn load_config_from_node_modules(
    module_name: &str,
    node_modules_dir: &str,
    fs: &dyn FS,
) -> Option<String> {

    let (package_name, _rest) = crate::module::parse_package_name(module_name);

    let candidate =
        tspath::normalize_path(&tspath::combine_paths(node_modules_dir, &[module_name]));

    if candidate.ends_with(".json") {
        if fs.file_exists(&candidate) {
            return Some(candidate);
        }
    } else {
        let with_json = format!("{candidate}.json");
        if fs.file_exists(&with_json) {
            return Some(with_json);
        }
    }

    let tsconfig_in_dir = tspath::combine_paths(&candidate, &["tsconfig.json"]);
    if fs.file_exists(&tsconfig_in_dir) {
        return Some(tsconfig_in_dir);
    }

    let package_dir = tspath::combine_paths(node_modules_dir, &[&package_name]);
    let package_json_path = tspath::combine_paths(&package_dir, &["package.json"]);
    if fs.file_exists(&package_json_path) {
        if let Some(content) = fs.read_file(&package_json_path) {
            if let Ok(fields) = crate::packagejson::parse(&content) {
                if let Some(tsconfig_field) = fields.path_fields.tsconfig.get_value() {
                    let resolved =
                        tspath::get_normalized_absolute_path(tsconfig_field, &package_dir);
                    if fs.file_exists(&resolved) {
                        return Some(resolved);
                    }
                }
            }
        }
    }

    None
}

fn json_object_to_options(
    obj: &crate::json::Map<String, crate::json::Value>,
) -> (HashMap<String, OptValue>, Vec<Diagnostic>) {
    let mut out = HashMap::new();
    let mut errors = Vec::new();
    for (k, v) in obj {

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

const CONFIG_DIR_TEMPLATE: &str = "${configDir}";

fn starts_with_config_dir_template(value: &str) -> bool {
    value
        .to_ascii_lowercase()
        .starts_with(&CONFIG_DIR_TEMPLATE.to_ascii_lowercase())
}

fn get_substituted_path_with_config_dir_template(value: &str, base_path: &str) -> String {
    let replaced = value.replacen(CONFIG_DIR_TEMPLATE, "./", 1);
    tspath::get_normalized_absolute_path(&replaced, base_path)
}

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

fn handle_config_dir_template_substitution(options: &mut CompilerOptions, base_path: &str) {

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

        }
    }

    if let Some(root_dirs) =
        get_substituted_string_array_with_config_dir_template(&options.root_dirs, base_path)
    {
        options.root_dirs = root_dirs;
    }

    if let Some(type_roots) =
        get_substituted_string_array_with_config_dir_template(&options.type_roots, base_path)
    {
        options.type_roots = type_roots;
    }

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

fn resolve_file_path_options(options: &mut CompilerOptions, base_path: &str) {
    let resolve = |s: &str| -> String {
        if s.is_empty() {
            return s.to_string();
        }

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

fn merge_compiler_options(dst: &mut CompilerOptions, src: &CompilerOptions) {
    let empty = HashSet::new();
    merge_compiler_options_with_skip(dst, src, &empty);
}

fn merge_compiler_options_with_skip(
    dst: &mut CompilerOptions,
    src: &CompilerOptions,
    skip_fields: &HashSet<String>,
) {

    macro_rules! merge_tri {
        ($field:ident, $json_name:literal) => {
            if dst.$field.is_unknown() && !skip_fields.contains($json_name) {
                dst.$field = src.$field;
            }
        };
    }
    merge_tri!(no_emit, "noEmit");
    merge_tri!(no_check, "noCheck");
    merge_tri!(no_lib, "noLib");
    merge_tri!(skip_lib_check, "skipLibCheck");
    merge_tri!(skip_default_lib_check, "skipDefaultLibCheck");
    merge_tri!(strict, "strict");
    merge_tri!(strict_null_checks, "strictNullChecks");
    merge_tri!(strict_function_types, "strictFunctionTypes");
    merge_tri!(strict_bind_call_apply, "strictBindCallApply");
    merge_tri!(
        strict_property_initialization,
        "strictPropertyInitialization"
    );
    merge_tri!(
        strict_builtin_iterator_return,
        "strictBuiltinIteratorReturn"
    );
    merge_tri!(no_implicit_any, "noImplicitAny");
    merge_tri!(no_implicit_this, "noImplicitThis");
    merge_tri!(no_implicit_override, "noImplicitOverride");
    merge_tri!(no_unused_locals, "noUnusedLocals");
    merge_tri!(no_unused_parameters, "noUnusedParameters");
    merge_tri!(no_fallthrough_cases_in_switch, "noFallthroughCasesInSwitch");
    merge_tri!(no_unchecked_indexed_access, "noUncheckedIndexedAccess");
    merge_tri!(exact_optional_property_types, "exactOptionalPropertyTypes");
    merge_tri!(es_module_interop, "esModuleInterop");
    merge_tri!(allow_js, "allowJs");
    merge_tri!(check_js, "checkJs");
    merge_tri!(composite, "composite");
    merge_tri!(declaration, "declaration");
    merge_tri!(source_map, "sourceMap");
    merge_tri!(remove_comments, "removeComments");
    merge_tri!(isolated_modules, "isolatedModules");
    merge_tri!(verbatim_module_syntax, "verbatimModuleSyntax");
    merge_tri!(experimental_decorators, "experimentalDecorators");
    merge_tri!(
        force_consistent_casing_in_file_names,
        "forceConsistentCasingInFileNames"
    );
    merge_tri!(use_unknown_in_catch_variables, "useUnknownInCatchVariables");
    merge_tri!(pretty, "pretty");
    merge_tri!(incremental, "incremental");
    merge_tri!(watch, "watch");
    if dst.target == ScriptTarget::None && !skip_fields.contains("target") {
        dst.target = src.target;
    }
    if dst.module == ModuleKind::None && !skip_fields.contains("module") {
        dst.module = src.module;
    }
    if dst.module_resolution == ModuleResolutionKind::Unknown
        && !skip_fields.contains("moduleResolution")
    {
        dst.module_resolution = src.module_resolution;
    }
    if dst.jsx == JsxEmit::None && !skip_fields.contains("jsx") {
        dst.jsx = src.jsx;
    }
    if dst.out_dir.is_empty() && !skip_fields.contains("outDir") {
        dst.out_dir = src.out_dir.clone();
    }
    if dst.root_dir.is_empty() && !skip_fields.contains("rootDir") {
        dst.root_dir = src.root_dir.clone();
    }
    if dst.base_url.is_empty() && !skip_fields.contains("baseUrl") {
        dst.base_url = src.base_url.clone();
    }
    if dst.lib.is_empty() && !skip_fields.contains("lib") {
        dst.lib = src.lib.clone();
    }
    if dst.types.is_empty() && !skip_fields.contains("types") {
        dst.types = src.types.clone();
    }
    if dst.type_roots.is_empty() && !skip_fields.contains("typeRoots") {
        dst.type_roots = src.type_roots.clone();
    }
    if dst.paths.is_none() && !skip_fields.contains("paths") {
        dst.paths = src.paths.clone();
    }
    if dst.declaration_dir.is_empty() && !skip_fields.contains("declarationDir") {
        dst.declaration_dir = src.declaration_dir.clone();
    }
    if dst.source_root.is_empty() && !skip_fields.contains("sourceRoot") {
        dst.source_root = src.source_root.clone();
    }
    if dst.map_root.is_empty() && !skip_fields.contains("mapRoot") {
        dst.map_root = src.map_root.clone();
    }
    if dst.ts_build_info_file.is_empty() && !skip_fields.contains("tsBuildInfoFile") {
        dst.ts_build_info_file = src.ts_build_info_file.clone();
    }
    if dst.root_dirs.is_empty() && !skip_fields.contains("rootDirs") {
        dst.root_dirs = src.root_dirs.clone();
    }
    if dst.module_suffixes.is_empty() && !skip_fields.contains("moduleSuffixes") {
        dst.module_suffixes = src.module_suffixes.clone();
    }
    if dst.custom_conditions.is_empty() && !skip_fields.contains("customConditions") {
        dst.custom_conditions = src.custom_conditions.clone();
    }
    if dst.out_file.is_empty() && !skip_fields.contains("outFile") {
        dst.out_file = src.out_file.clone();
    }
    if dst.module_detection == ModuleDetectionKind::None && !skip_fields.contains("moduleDetection")
    {
        dst.module_detection = src.module_detection;
    }
    if dst.new_line == NewLineKind::None && !skip_fields.contains("newLine") {
        dst.new_line = src.new_line;
    }
}

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

    for f in files {
        add(f, &mut result, &mut seen);
    }

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
            if !is_supported_source_file_ex(&path, options.allow_js.is_true()) {
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

#[allow(dead_code)]
fn is_supported_source_file(path: &str) -> bool {
    is_supported_source_file_ex(path, false)
}

fn is_supported_source_file_ex(path: &str, allow_js: bool) -> bool {
    let ext = path.rfind('.').map(|i| &path[i..]).unwrap_or("");
    if matches!(
        ext,
        ".ts" | ".tsx" | ".d.ts" | ".mts" | ".cts" | ".d.mts" | ".d.cts"
    ) {
        return true;
    }
    if allow_js && matches!(ext, ".js" | ".jsx" | ".mjs" | ".cjs") {
        return true;
    }
    false
}

fn match_glob_spec(spec: &str, base_dir: &str, fs: &dyn FS) -> Vec<String> {
    let mut results = Vec::new();

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

    let walk_root = glob_base_dir(&abs_spec);
    walk_and_match(&abs_spec, &walk_root, fs, &mut results);
    results
}

fn contains_glob_char(spec: &str) -> bool {
    spec.chars()
        .any(|c| c == '*' || c == '?' || c == '{' || c == '[')
}

fn glob_base_dir(spec: &str) -> String {
    let first_meta = spec
        .chars()
        .position(|c| c == '*' || c == '?' || c == '{' || c == '[');
    let prefix = match first_meta {
        Some(idx) => &spec[..idx],
        None => spec,
    };

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

                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '*' => {

                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            ',' if i + 1 < chars.len() => {

                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {

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

    fn has_error_containing(parsed: &ParsedCommandLine, needle: &str) -> bool {
        parsed.errors.iter().any(|e| {
            e.message_args.iter().any(|a| a.contains(needle))
                || e.message.map(|m| m.text.contains(needle)).unwrap_or(false)
        })
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_command_line_version() {

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

        assert!(parsed.watch);

        let parsed_short = parse_command_line(&args(&["-w", "0.ts"]), "/proj", None);
        assert!(parsed_short.compiler_options.watch.is_true());
        assert!(parsed_short.watch);
    }

    #[test]
    fn watch_options_empty_by_default() {

        let parsed = parse_command_line(&args(&["--noEmit", "0.ts"]), "/proj", None);
        assert!(parsed.watch_options.is_empty());
    }

    #[test]
    fn watch_options_parse_enum_flags() {

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
        assert_eq!(
            parsed.watch_options.fallback_polling,
            PollingKind::PriorityInterval
        );
    }

    #[test]
    fn watch_options_parse_interval_and_boolean() {
        let parsed = parse_command_line(
            &args(&[
                "--watchInterval",
                "250",
                "--synchronousWatchDirectory",
                "0.ts",
            ]),
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

        let parsed = parse_command_line(&args(&["--watchFile", "bogus", "0.ts"]), "/proj", None);
        assert!(parsed
            .errors
            .iter()
            .any(|d| d.code == 6046 && d.message_args.iter().any(|a| a.contains("--watchFile"))));

        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::None);
    }

    #[test]
    fn watch_options_missing_number_value_reports_ts5080() {

        let parsed = parse_command_line(&args(&["--watchInterval"]), "/proj", None);
        assert!(parsed.errors.iter().any(|d| d.code == 5080
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

        let parsed = parse_build_command_line(
            &args(&["--build", "--watchFile", "usefsevents", "."]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
    }

    #[test]
    fn watch_options_case_insensitive_lookup() {

        let parsed = parse_command_line(
            &args(&["--WATCHFILE", "usefsevents", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(parsed.watch_options.file_kind, WatchFileKind::UseFsEvents);
    }

    #[test]
    fn watch_options_do_not_leak_into_compiler_options() {

        let parsed = parse_command_line(
            &args(&["--watchFile", "usefsevents", "0.ts"]),
            "/proj",
            None,
        );
        assert!(
            !parsed
                .errors
                .iter()
                .any(|d| d.code == 5023 && d.message_args.iter().any(|a| a == "watchFile"))
        );
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

        let parsed = parse_command_line(&args(&["--strictNullChecks"]), "/proj", None);
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_command_line_non_boolean_after_boolean_flag() {

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

        let parsed =
            parse_command_line(&args(&["--tsBuildInfoFile", "null", "0.ts"]), "/proj", None);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.compiler_options.ts_build_info_file, "");
    }

    #[test]
    fn test_parse_command_line_type_roots() {

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

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        let parsed = parse_command_line(&args(&["@missing.rsp"]), "/proj", Some(&fs));
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Cannot read file"));
    }

    #[test]
    fn test_response_file_propagates_file_names() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/args.rsp", "--strict\n0.ts");
        let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);

        assert!(!has_error_containing(&parsed, "Cannot read file"));
    }

    #[test]
    fn test_response_file_unterminated_quoted_string() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/args.rsp", "--outDir \"unterminated path");
        let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));

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

        assert_eq!(
            parsed.compiler_options.out_dir, "unterminated path",
            "unterminated token should still be captured as the option value"
        );
    }

    #[test]
    fn test_strip_jsonc_whitespace_and_empty_object() {

        let stripped = strip_jsonc("   ");
        assert_eq!(stripped.trim(), "");

        let stripped = strip_jsonc("// Comment");
        assert_eq!(stripped.trim(), "");

        let stripped = strip_jsonc("/* Comment */");
        assert_eq!(stripped.trim(), "");

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
            "extends": "./base.json",
            "compilerOptions": { "outDir": "./dist" }
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(parsed.compiler_options.strict.is_true());

        assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");

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
            "extends": "./base.json",
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

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/a.tsconfig.json",
            r#"{ "extends": "./b.tsconfig.json", "compilerOptions": { "target": "ES2020" } }"#,
        );
        fs.insert_file(
            "/proj/b.tsconfig.json",
            r#"{ "extends": "./a.tsconfig.json", "compilerOptions": { "strict": true } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/a.tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert!(
            parsed
                .errors
                .iter()
                .any(|e| e.code == CIRCULARITY_DETECTED_WHILE_RESOLVING_CONFIGURATION_COLON_0.code),
            "expected a circularity diagnostic, got errors: {:?}",
            parsed.errors.iter().map(|e| e.code).collect::<Vec<_>>()
        );

        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    }

    #[test]
    fn test_parse_tsconfig_extends_as_array_merges_all() {

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
            "extends": ["./base1.json", "./base2.json"],
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
        assert!(
            parsed.errors.is_empty(),
            "unexpected errors: {:?}",
            parsed.errors
        );

        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(parsed.compiler_options.strict.is_true());

        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert!(parsed.compiler_options.declaration.is_true());

        assert_eq!(parsed.compiler_options.out_dir, "/proj/dist");
    }

    #[test]
    fn test_parse_tsconfig_extends_own_overrides_extended() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
            "extends": "./base.json",
            "compilerOptions": { "strict": false }
        }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert!(
            parsed.compiler_options.strict.is_false(),
            "expected own strict=false to override extended strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_array_last_wins() {

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
            r#"{ "extends": ["./base1.json", "./base2.json"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert_eq!(
            parsed.compiler_options.target,
            ScriptTarget::ES2015,
            "expected last extends entry (base2/ES2015) to win, got {:?}",
            parsed.compiler_options.target
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_command_line_overrides_own() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        let mut base = CompilerOptions::default();
        base.strict = Tristate::False;
        let parsed =
            get_parsed_command_line_of_config_file("/proj/tsconfig.json", &base, "/proj", &fs);

        assert!(
            parsed.compiler_options.strict.is_false(),
            "expected command-line strict=false to override config strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_parse_tsconfig_extends_include_first_extended_wins() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src1");
        fs.insert_dir("/proj/src2");
        fs.insert_file("/proj/src1/a.ts", "export const a = 1;");
        fs.insert_file("/proj/src2/b.ts", "export const b = 2;");
        fs.insert_file("/proj/base1.json", r#"{ "include": ["src1/**/*"] }"#);
        fs.insert_file("/proj/base2.json", r#"{ "include": ["src2/**/*"] }"#);
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": ["./base1.json", "./base2.json"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

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

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "./base" }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected extends ./base to resolve to ./base.json and inherit strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_parse_tsconfig_full_compiler_options() {

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

        assert!(
            parsed
                .file_names
                .contains(&"/apath/src/index.ts".to_string())
        );
        assert!(parsed.file_names.contains(&"/apath/src/app.ts".to_string()));

        assert!(!parsed.file_names.iter().any(|f| f.contains("node_modules")));
    }

    #[test]
    fn test_parse_tsconfig_null_enum_options() {

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

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file("/proj/tsconfig.json", r#"{ "files": [] }"#);
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

        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2022);
        assert!(parsed.compiler_options.strict.is_true());
    }

    #[test]
    fn test_parsed_command_line_literal_file_list_dedup() {

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

        assert_eq!(
            parsed.file_names,
            vec!["/dev/a.ts".to_string(), "/dev/b.ts".to_string()]
        );
    }

    #[test]
    fn test_parsed_command_line_files_not_removed_by_exclude() {

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

    #[test]
    fn test_options_declarations_non_empty_and_named() {

        assert!(!OPTIONS.is_empty());
        for o in OPTIONS {
            assert!(!o.name.is_empty(), "found an option with an empty name");
        }

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

        assert_eq!(find_option("h").map(|o| o.name), Some("help"));
        assert_eq!(find_option("v").map(|o| o.name), Some("version"));
        assert_eq!(find_option("b").map(|o| o.name), Some("build"));
        assert_eq!(find_option("w").map(|o| o.name), Some("watch"));
        assert_eq!(find_option("p").map(|o| o.name), Some("project"));
        assert_eq!(find_option("t").map(|o| o.name), Some("target"));
        assert_eq!(find_option("m").map(|o| o.name), Some("module"));
        assert_eq!(find_option("d").map(|o| o.name), Some("declaration"));
    }

    fn diag_contains(errors: &[Diagnostic], needle: &str) -> bool {
        errors.iter().any(|e| {
            e.message_args.iter().any(|a| a.contains(needle))
                || e.message.map(|m| m.text.contains(needle)).unwrap_or(false)
        })
    }

    #[test]
    fn test_case_insensitive_option_lookup_cli() {

        let parsed = parse_command_line(&args(&["--Target", "ES2020", "0.ts"]), "/proj", None);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(!has_error_containing(&parsed, "Unknown compiler option"));

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

        let parsed = parse_command_line(&args(&["-P", "tsconfig.json"]), "/proj", None);
        assert_eq!(parsed.compiler_options.project, "tsconfig.json");
    }

    #[test]
    fn test_alternate_mode_build_option_in_compiler_mode() {

        let parsed = parse_command_line(&args(&["--dry", "0.ts"]), "/proj", None);
        assert!(diag_contains(
            &parsed.errors,
            "may only be used with '--build'"
        ));
        assert!(!diag_contains(&parsed.errors, "Unknown compiler option"));
    }

    #[test]
    fn test_alternate_mode_verbose_in_compiler_mode() {

        let parsed = parse_command_line(&args(&["--verbose"]), "/proj", None);
        assert!(diag_contains(
            &parsed.errors,
            "may only be used with '--build'"
        ));
    }

    #[test]
    fn test_tsconfig_only_option_on_cli_emits_diagnostic() {

        let parsed = parse_command_line(&args(&["--composite", "0.ts"]), "/proj", None);
        assert!(has_error_containing(&parsed, "tsconfig.json"));
        assert!(has_error_containing(&parsed, "composite"));
        assert!(!parsed.compiler_options.composite.is_true());
    }

    #[test]
    fn test_tsconfig_only_boolean_accepts_false() {

        let parsed = parse_command_line(&args(&["--composite", "false", "0.ts"]), "/proj", None);
        assert!(!has_error_containing(&parsed, "tsconfig.json"));
        assert!(parsed.compiler_options.composite.is_false());
    }

    #[test]
    fn test_tsconfig_only_boolean_accepts_null() {

        let parsed = parse_command_line(&args(&["--composite", "null", "0.ts"]), "/proj", None);
        assert!(!has_error_containing(&parsed, "tsconfig.json"));
    }

    #[test]
    fn test_invalid_enum_value_target() {

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

        let parsed = parse_command_line(&args(&["--target", "ES2020", "0.ts"]), "/proj", None);
        assert!(!has_error_containing(&parsed, "Argument for"));
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
    }

    #[test]
    fn test_min_value_violation_builders() {

        let parsed =
            parse_build_command_line(&args(&["--build", "--builders", "0"]), "/proj", None);
        assert!(diag_contains(
            &parsed.errors,
            "requires value to be greater"
        ));
        assert!(diag_contains(&parsed.errors, "builders"));
        assert!(diag_contains(&parsed.errors, "1"));
    }

    #[test]
    fn test_min_value_accepted_builders() {

        let parsed =
            parse_build_command_line(&args(&["--build", "--builders", "2"]), "/proj", None);
        assert!(!diag_contains(
            &parsed.errors,
            "requires value to be greater"
        ));
        assert_eq!(parsed.build_options.builders, Some(2));
    }

    #[test]
    fn test_case_mismatch_in_tsconfig_json_emits_did_you_mean() {

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

        assert_eq!(parsed.compiler_options.target, ScriptTarget::None);

        assert!(parsed.compiler_options.no_emit.is_true());
    }

    #[test]
    fn test_tsconfig_json_correct_case_no_did_you_mean() {

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

        let composite = find_option("composite").expect("composite must exist");
        assert!(composite.is_tsconfig_only);
        let paths = find_option("paths").expect("paths must exist");
        assert!(paths.is_tsconfig_only);
        let builders = find_build_only_option("builders").expect("builders must exist");
        assert_eq!(builders.min_value, Some(1));
    }

    #[test]
    fn test_config_dir_substitution_out_dir() {

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
            parsed.compiler_options.out_dir, "/proj/out",
            "expected ${{configDir}}/out to resolve to /proj/out, got {}",
            parsed.compiler_options.out_dir
        );
    }

    #[test]
    fn test_config_dir_substitution_root_dir() {

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
        assert_eq!(
            parsed.compiler_options.ts_build_info_file,
            "/proj/build.tsbuildinfo"
        );
    }

    #[test]
    fn test_config_dir_substitution_root_dirs_array() {

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
        assert_eq!(paths.get("@/*").unwrap(), &vec!["/proj/src/*".to_string()]);
        assert_eq!(
            paths.get("lib/*").unwrap(),
            &vec!["/proj/lib/*".to_string()]
        );
    }

    #[test]
    fn test_config_dir_substitution_include() {

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

        assert_eq!(
            parsed.compiler_options.out_dir, "/proj/base/out",
            "extended config's ${{configDir}} should resolve to extended config's dir"
        );

        assert_eq!(
            parsed.compiler_options.root_dir, "/proj/src",
            "own config's ${{configDir}} should resolve to own config's dir"
        );
    }

    #[test]
    fn test_config_dir_not_substituted_for_non_prefix() {

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

        assert!(
            parsed.compiler_options.out_dir.contains("configDir"),
            "embedded ${{configDir}} should not be substituted, got {}",
            parsed.compiler_options.out_dir
        );
    }

    #[test]
    fn test_extends_inherited_include_path_rewriting() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/base/src");
        fs.insert_file("/proj/base/src/a.ts", "");
        fs.insert_file("/proj/base/tsconfig.json", r#"{ "include": ["src/**/*"] }"#);
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

        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/src/a.ts"),
            "expected /proj/src/a.ts in file_names (${{configDir}} resolved against own dir), got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_inherited_exclude_path_rewriting() {

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

        assert!(
            parsed.file_names.iter().any(|f| f == "/proj/base/src/a.ts"),
            "expected /proj/base/src/a.ts in file_names, got {:?}",
            parsed.file_names
        );

        assert!(
            !parsed.file_names.iter().any(|f| f.contains("excluded")),
            "expected excluded/ files to be excluded, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_inherited_files_path_rewriting() {

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

        assert!(
            parsed
                .file_names
                .iter()
                .any(|f| f == "/proj/base/src/main.ts"),
            "expected /proj/base/src/main.ts in file_names, got {:?}",
            parsed.file_names
        );
    }

    #[test]
    fn test_extends_own_include_overrides_inherited() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/base");
        fs.insert_dir("/proj/own_src");
        fs.insert_file("/proj/own_src/a.ts", "");
        fs.insert_dir("/proj/base/src");
        fs.insert_file("/proj/base/src/b.ts", "");
        fs.insert_file("/proj/base/tsconfig.json", r#"{ "include": ["src/**/*"] }"#);
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

    #[test]
    fn test_extends_null_clears_inherited_tristate() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "strict": null } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict.is_unknown(),
            "expected strict=null to clear inherited strict=true, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_extends_null_clears_inherited_string_field() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "outDir": "./dist" } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "outDir": null } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.out_dir.is_empty(),
            "expected outDir=null to clear inherited outDir, got {:?}",
            parsed.compiler_options.out_dir
        );
    }

    #[test]
    fn test_extends_null_clears_inherited_enum_field() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "target": "ES2020" } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "target": null } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(
            parsed.compiler_options.target,
            ScriptTarget::None,
            "expected target=null to clear inherited target=ES2020, got {:?}",
            parsed.compiler_options.target
        );
    }

    #[test]
    fn test_extends_null_does_not_override_command_line() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "strict": null } }"#,
        );
        let mut base = CompilerOptions::default();
        base.strict = crate::core::tristate::Tristate::True;
        let parsed =
            get_parsed_command_line_of_config_file("/proj/tsconfig.json", &base, "/proj", &fs);
        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected command-line strict=true to survive own strict=null, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_extends_null_only_clears_specified_field() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true, "noImplicitAny": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "strict": null } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict.is_unknown(),
            "expected strict=null to clear inherited strict, got {:?}",
            parsed.compiler_options.strict
        );
        assert!(
            parsed.compiler_options.no_implicit_any.is_true(),
            "expected noImplicitAny to be inherited (not nulled), got {:?}",
            parsed.compiler_options.no_implicit_any
        );
    }

    #[test]
    fn test_extends_null_with_multiple_fields() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/base.json",
            r#"{ "compilerOptions": { "strict": true, "outDir": "./dist", "target": "ES2020" } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "./base.json", "compilerOptions": { "strict": null, "outDir": null, "target": null } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict.is_unknown(),
            "expected strict=null to clear, got {:?}",
            parsed.compiler_options.strict
        );
        assert!(
            parsed.compiler_options.out_dir.is_empty(),
            "expected outDir=null to clear, got {:?}",
            parsed.compiler_options.out_dir
        );
        assert_eq!(
            parsed.compiler_options.target,
            ScriptTarget::None,
            "expected target=null to clear, got {:?}",
            parsed.compiler_options.target
        );
    }

    #[test]
    fn test_extends_diamond_inheritance() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/d.json",
            r#"{ "compilerOptions": { "strict": true, "noImplicitAny": true } }"#,
        );
        fs.insert_file("/proj/b.json", r#"{ "extends": "./d.json" }"#);
        fs.insert_file("/proj/c.json", r#"{ "extends": "./d.json" }"#);
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": ["./b.json", "./c.json"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected strict=true from diamond D, got {:?}",
            parsed.compiler_options.strict
        );
        assert!(
            parsed.compiler_options.no_implicit_any.is_true(),
            "expected noImplicitAny=true from diamond D, got {:?}",
            parsed.compiler_options.no_implicit_any
        );
    }

    #[test]
    fn test_extends_diamond_no_duplicate_errors() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");

        fs.insert_file(
            "/proj/d.json",
            r#"{ "compilerOptions": { "strict": true, "Strict": true } }"#,
        );
        fs.insert_file("/proj/b.json", r#"{ "extends": "./d.json" }"#);
        fs.insert_file("/proj/c.json", r#"{ "extends": "./d.json" }"#);
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": ["./b.json", "./c.json"] }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        let ts5025_count = parsed.errors.iter().filter(|d| d.code == 5025).count();
        assert_eq!(
            ts5025_count, 2,
            "expected exactly 2 TS5025 errors (D via B and C), got {}: {:?}",
            ts5025_count, parsed.errors
        );
    }

    #[test]
    fn test_extends_cache_cycle_not_cached() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.json", r#"{ "extends": "./b.json" }"#);
        fs.insert_file("/proj/b.json", r#"{ "extends": "./a.json" }"#);
        fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "./a.json" }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        let has_cycle = parsed.errors.iter().any(|d| d.code == 18000);
        assert!(
            has_cycle,
            "expected TS18000 circularity error, got errors: {:?}",
            parsed.errors
        );
    }

    #[test]
    fn test_extends_bare_specifier_file_form() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_file(
            "/proj/node_modules/tsconfig-base.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "tsconfig-base" }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected strict=true from node_modules/tsconfig-base.json, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_extends_bare_specifier_directory_form() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/tsconfig-base");
        fs.insert_file(
            "/proj/node_modules/tsconfig-base/tsconfig.json",
            r#"{ "compilerOptions": { "noImplicitAny": true } }"#,
        );
        fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "tsconfig-base" }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.no_implicit_any.is_true(),
            "expected noImplicitAny=true from node_modules/tsconfig-base/tsconfig.json, got {:?}",
            parsed.compiler_options.no_implicit_any
        );
    }

    #[test]
    fn test_extends_bare_specifier_package_json_tsconfig_field() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/tsconfig-base");
        fs.insert_file(
            "/proj/node_modules/tsconfig-base/package.json",
            r#"{ "name": "tsconfig-base", "tsconfig": "my-base.json" }"#,
        );
        fs.insert_file(
            "/proj/node_modules/tsconfig-base/my-base.json",
            r#"{ "compilerOptions": { "strict": true, "noImplicitThis": true } }"#,
        );
        fs.insert_file("/proj/tsconfig.json", r#"{ "extends": "tsconfig-base" }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected strict=true from package.json tsconfig field, got {:?}",
            parsed.compiler_options.strict
        );
        assert!(
            parsed.compiler_options.no_implicit_this.is_true(),
            "expected noImplicitThis=true from package.json tsconfig field, got {:?}",
            parsed.compiler_options.no_implicit_this
        );
    }

    #[test]
    fn test_extends_bare_specifier_scoped_package() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/@scope");
        fs.insert_dir("/proj/node_modules/@scope/tsconfig-base");
        fs.insert_file(
            "/proj/node_modules/@scope/tsconfig-base/tsconfig.json",
            r#"{ "compilerOptions": { "strictNullChecks": true } }"#,
        );
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "@scope/tsconfig-base" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict_null_checks.is_true(),
            "expected strictNullChecks=true from @scope/tsconfig-base, got {:?}",
            parsed.compiler_options.strict_null_checks
        );
    }

    #[test]
    fn test_extends_bare_specifier_ancestor_walk() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/tsconfig-base");
        fs.insert_file(
            "/proj/node_modules/tsconfig-base/tsconfig.json",
            r#"{ "compilerOptions": { "strict": true } }"#,
        );
        fs.insert_dir("/proj/packages");
        fs.insert_dir("/proj/packages/foo");
        fs.insert_file(
            "/proj/packages/foo/tsconfig.json",
            r#"{ "extends": "tsconfig-base" }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/packages/foo/tsconfig.json",
            &CompilerOptions::default(),
            "/proj/packages/foo",
            &fs,
        );
        assert!(
            parsed.compiler_options.strict.is_true(),
            "expected strict=true from ancestor node_modules, got {:?}",
            parsed.compiler_options.strict
        );
    }

    #[test]
    fn test_extends_bare_specifier_not_found() {

        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{ "extends": "nonexistent-config", "compilerOptions": { "target": "ES2020" } }"#,
        );
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );

        assert_eq!(
            parsed.compiler_options.target,
            ScriptTarget::ES2020,
            "expected own config target to be applied"
        );

        assert!(
            !parsed.compiler_options.strict.is_true(),
            "expected strict=false (no extended config found), got {:?}",
            parsed.compiler_options.strict
        );
    }
}
