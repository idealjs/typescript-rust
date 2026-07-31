//! CLI execution pipeline, ported from `internal/execute/`.
//!
//! This module wires together the pieces ported in `tsoptions`, `compiler`,
//! and `diagnosticwriter` into a working command-line entry point. It mirrors
//! the control flow of `execute.CommandLine` in the Go implementation:
//!
//! ```text
//! CommandLine(args)
//!   ├─ -b / --build  → build mode (stubbed)
//!   └─ tsc_compilation(args)
//!        ├─ report parse errors
//!        ├─ --version / --help / --all
//!        ├─ --project <dir|file>
//!        ├─ find tsconfig.json up the tree
//!        ├─ read tsconfig.json (extends, include, exclude, …)
//!        └─ perform_compilation()
//!             ├─ create CompilerHost + Program
//!             ├─ report diagnostics
//!             └─ return ExitStatus
//! ```

use std::collections::HashSet;
use std::io::{IsTerminal, Write};
use std::sync::Arc;
use std::time::Instant;

use crate::ast::diagnostic::Diagnostic;
use crate::bundled::{self, BundledFS};
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::core::compiler_options::CompilerOptions;
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::diagnosticwriter::{format_diagnostic, report_diagnostics};
use crate::diagnostics::{
    A_TSCONFIG_JSON_FILE_IS_ALREADY_DEFINED_AT_COLON_0,
    CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_CURRENT_DIRECTORY_COLON_0,
    CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0, CANNOT_READ_FILE_0,
    OPTION_BUILD_MUST_BE_THE_FIRST_COMMAND_LINE_ARGUMENT,
    OPTION_PROJECT_CANNOT_BE_MIXED_WITH_SOURCE_FILES_ON_A_COMMAND_LINE,
    OPTIONS_0_AND_1_CANNOT_BE_COMBINED, THE_SPECIFIED_PATH_DOES_NOT_EXIST_COLON_0,
    X_TSCONFIG_JSON_IS_PRESENT_BUT_WILL_NOT_BE_LOADED_IF_FILES_ARE_SPECIFIED_ON_COMMANDLINE_USE_IGNORECONFIG_TO_SKIP_THIS_ERROR,
};
use crate::tsoptions::{
    ParsedBuildCommandLine, ParsedCommandLine, get_parsed_command_line_of_config_file,
    parse_build_command_line, parse_command_line,
};
use crate::tspath;
use crate::vfs::{FS, OsFS};

/// Compiler version string, matching Go's `core.Version()`.
pub const VERSION: &str = "7.1.0-dev";

// ────────────────────────────────────────────────────────────────────────────
// ExitStatus
// ────────────────────────────────────────────────────────────────────────────

/// The process exit status returned by `command_line`.
///
/// Mirrors `tsc.ExitStatus` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum ExitStatus {
    Success = 0,
    DiagnosticsPresent_OutputsSkipped = 1,
    DiagnosticsPresent_OutputsGenerated = 2,
    InvalidProject_OutputsSkipped = 3,
    ProjectReferenceCycle_OutputsSkipped = 4,
    NotImplemented = 5,
}

impl ExitStatus {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// The result of running `command_line`.
#[derive(Debug)]
pub struct CommandLineResult {
    pub status: ExitStatus,
}

// ────────────────────────────────────────────────────────────────────────────
// System
// ────────────────────────────────────────────────────────────────────────────

/// Abstraction over the host environment, mirroring `tsc.System` in Go.
///
/// A `System` provides the writer for output, the file system, the current
/// working directory, and a few environment / terminal helpers.
pub trait System: Send + Sync {
    /// Return a boxed writer. Each call produces a fresh handle; callers should
    /// flush before dropping.
    fn writer(&self) -> Box<dyn Write + Send>;
    fn fs(&self) -> Arc<dyn FS>;
    fn default_library_path(&self) -> &str;
    fn current_directory(&self) -> &str;
    fn write_output_is_tty(&self) -> bool;
    fn width_of_terminal(&self) -> usize;
    fn environment_variable(&self, name: &str) -> Option<String>;
}

/// OS-backed `System` implementation using real stdout, the real file system,
/// and the real environment.
pub struct OsSystem {
    fs: Arc<BundledFS>,
    default_library_path: String,
    cwd: String,
    #[allow(dead_code)]
    start: Instant,
}

impl OsSystem {
    pub fn new() -> Self {
        let cwd = std::env::current_dir()
            .map(|p| tspath::normalize_path(&p.to_string_lossy()))
            .unwrap_or_else(|_| ".".to_string());
        Self {
            fs: Arc::new(BundledFS::new(Arc::new(OsFS))),
            default_library_path: bundled::lib_path(),
            cwd,
            start: Instant::now(),
        }
    }
}

impl Default for OsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for OsSystem {
    fn writer(&self) -> Box<dyn Write + Send> {
        Box::new(std::io::stdout())
    }
    fn fs(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs) as Arc<dyn FS>
    }
    fn default_library_path(&self) -> &str {
        &self.default_library_path
    }
    fn current_directory(&self) -> &str {
        &self.cwd
    }
    fn write_output_is_tty(&self) -> bool {
        std::io::stdout().is_terminal()
    }
    fn width_of_terminal(&self) -> usize {
        80
    }
    fn environment_variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CommandLine entry point
// ────────────────────────────────────────────────────────────────────────────

/// Build a compiler-level diagnostic (no source file) from a `Message`
/// constant and its arguments, mirroring `ast.NewCompilerDiagnostic` in Go.
fn compiler_diagnostic(message: crate::diagnostics::Message, args: Vec<String>) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), message, args)
}

/// The main entry point for the `tsc` command line.
///
/// Mirrors `execute.CommandLine` in Go. Dispatches build mode (`-b`) or runs a
/// regular compilation.
pub fn command_line(sys: &dyn System, args: &[String]) -> CommandLineResult {
    if let Some(first) = args.first() {
        if is_build_mode_arg(first) {
            let parsed =
                parse_build_command_line(args, sys.current_directory(), Some(sys.fs().as_ref()));
            return tsc_build_compilation(sys, parsed);
        }
    }

    if args.iter().skip(1).any(|arg| is_build_mode_arg(arg)) {
        let mut writer = sys.writer();
        let diag = compiler_diagnostic(
            OPTION_BUILD_MUST_BE_THE_FIRST_COMMAND_LINE_ARGUMENT,
            vec![],
        );
        let _ = writeln!(writer, "{}", format_diagnostic(&diag, false));
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let parsed = parse_command_line(args, sys.current_directory(), Some(sys.fs().as_ref()));
    tsc_compilation(sys, parsed)
}

fn is_build_mode_arg(arg: &str) -> bool {
    matches!(
        arg.to_lowercase().as_str(),
        "-b" | "--b" | "-build" | "--build"
    )
}

fn tsc_build_compilation(
    sys: &dyn System,
    command_line: ParsedBuildCommandLine,
) -> CommandLineResult {
    let pretty = should_be_pretty(sys, &command_line.compiler_options);

    if !command_line.errors.is_empty() {
        let mut writer = sys.writer();
        for e in &command_line.errors {
            let _ = writeln!(writer, "{}", format_diagnostic(e, pretty));
        }
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    if command_line.compiler_options.help.is_true() || command_line.compiler_options.all.is_true() {
        print_help(sys);
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    let projects = command_line.resolved_project_paths();

    let mut status = ExitStatus::Success;
    let mut seen_projects = HashSet::new();
    for project in projects {
        let result = build_project(
            sys,
            &project,
            &command_line.compiler_options,
            pretty,
            &mut seen_projects,
        );
        status = status.max(result.status);
    }

    CommandLineResult { status }
}

fn build_project(
    sys: &dyn System,
    project: &str,
    compiler_options: &CompilerOptions,
    pretty: bool,
    seen_projects: &mut HashSet<String>,
) -> CommandLineResult {
    let config_file_name = match resolve_project_config(sys, project) {
        Ok(config) => config,
        Err(diag) => {
            let mut writer = sys.writer();
            let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
            };
        }
    };

    let normalized_config = tspath::normalize_path(&config_file_name);
    if !seen_projects.insert(normalized_config.clone()) {
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    let config = get_parsed_command_line_of_config_file(
        &normalized_config,
        compiler_options,
        sys.current_directory(),
        sys.fs().as_ref(),
    );
    if !config.errors.is_empty() {
        let mut writer = sys.writer();
        for e in &config.errors {
            let _ = writeln!(writer, "{}", format_diagnostic(e, pretty));
        }
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let mut status = ExitStatus::Success;
    for reference in resolve_project_references(&config) {
        let result = build_project(sys, &reference, compiler_options, pretty, seen_projects);
        status = status.max(result.status);
    }

    if !config.file_names.is_empty() {
        let result = perform_compilation(sys, config, pretty);
        status = status.max(result.status);
    }

    CommandLineResult { status }
}

fn resolve_project_config(sys: &dyn System, project: &str) -> Result<String, Diagnostic> {
    if sys.fs().directory_exists(project) {
        let config = tspath::combine_paths(project, &["tsconfig.json"]);
        if !sys.fs().file_exists(&config) {
            return Err(compiler_diagnostic(
                CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0,
                vec![config],
            ));
        }
        Ok(config)
    } else if sys.fs().file_exists(project) {
        Ok(project.to_string())
    } else {
        Err(compiler_diagnostic(
            CANNOT_READ_FILE_0,
            vec![project.to_string()],
        ))
    }
}

fn resolve_project_references(config: &ParsedCommandLine) -> Vec<String> {
    let config_dir = tspath::get_directory_path(&config.config_file_name);
    config
        .references
        .iter()
        .map(|reference| {
            resolve_config_file_name_of_project_reference(&config_dir, &reference.path)
        })
        .collect()
}

fn resolve_config_file_name_of_project_reference(config_dir: &str, path: &str) -> String {
    let resolved = tspath::get_normalized_absolute_path(path, config_dir);
    if tspath::file_extension_is(&resolved, ".json") {
        resolved
    } else {
        tspath::combine_paths(&resolved, &["tsconfig.json"])
    }
}

// ────────────────────────────────────────────────────────────────────────────
// tsc_compilation
// ────────────────────────────────────────────────────────────────────────────

fn tsc_compilation(sys: &dyn System, command_line: ParsedCommandLine) -> CommandLineResult {
    let pretty = should_be_pretty(sys, &command_line.compiler_options);

    // Report parse errors from the command line itself.
    if !command_line.errors.is_empty() {
        let mut writer = sys.writer();
        for e in &command_line.errors {
            let _ = writeln!(writer, "{}", format_diagnostic(e, pretty));
        }
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let options = &command_line.compiler_options;

    // --init
    if options.init.is_true() {
        return write_config_file(sys, options);
    }

    // --version
    if options.version.is_true() {
        let mut writer = sys.writer();
        let _ = writeln!(writer, "Version {}", VERSION);
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    // --help / --all
    if options.help.is_true() || options.all.is_true() {
        print_help(sys);
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    // --watch + --listFilesOnly is invalid.
    if options.watch.is_true() && options.list_files_only.is_true() {
        let mut writer = sys.writer();
        let diag = compiler_diagnostic(
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["watch".to_string(), "listFilesOnly".to_string()],
        );
        let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    // Determine the config file name.
    let mut config_file_name = String::new();

    if !options.project.is_empty() {
        if !command_line.file_names.is_empty() {
            let mut writer = sys.writer();
            let diag = compiler_diagnostic(
                OPTION_PROJECT_CANNOT_BE_MIXED_WITH_SOURCE_FILES_ON_A_COMMAND_LINE,
                vec![],
            );
            let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
            };
        }
        let file_or_directory = tspath::normalize_path(&tspath::combine_paths(
            sys.current_directory(),
            &[&options.project],
        ));
        if sys.fs().directory_exists(&file_or_directory) {
            config_file_name = tspath::combine_paths(&file_or_directory, &["tsconfig.json"]);
            if !sys.fs().file_exists(&config_file_name) {
                let mut writer = sys.writer();
                let diag = compiler_diagnostic(
                    CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_SPECIFIED_DIRECTORY_COLON_0,
                    vec![config_file_name.clone()],
                );
                let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
                return CommandLineResult {
                    status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
                };
            }
        } else {
            config_file_name = file_or_directory.clone();
            if !sys.fs().file_exists(&config_file_name) {
                let mut writer = sys.writer();
                let diag = compiler_diagnostic(
                    THE_SPECIFIED_PATH_DOES_NOT_EXIST_COLON_0,
                    vec![file_or_directory.clone()],
                );
                let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
                return CommandLineResult {
                    status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
                };
            }
        }
    } else if !options.ignore_config.is_true() || command_line.file_names.is_empty() {
        let search_path = tspath::normalize_path(sys.current_directory());
        config_file_name =
            find_config_file(&search_path, &|p| sys.fs().file_exists(p), "tsconfig.json");
        if !command_line.file_names.is_empty() {
            if !config_file_name.is_empty() {
                let mut writer = sys.writer();
                let diag = compiler_diagnostic(
                    X_TSCONFIG_JSON_IS_PRESENT_BUT_WILL_NOT_BE_LOADED_IF_FILES_ARE_SPECIFIED_ON_COMMANDLINE_USE_IGNORECONFIG_TO_SKIP_THIS_ERROR,
                    vec![],
                );
                let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
                return CommandLineResult {
                    status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
                };
            }
        } else if config_file_name.is_empty() {
            let mut writer = sys.writer();
            let diag = compiler_diagnostic(
                CANNOT_FIND_A_TSCONFIG_JSON_FILE_AT_THE_CURRENT_DIRECTORY_COLON_0,
                vec![search_path],
            );
            let _ = writeln!(writer, "{}", format_diagnostic(&diag, pretty));
            let _ = writeln!(writer, "  Searching for: tsconfig.json");
            print_help(sys);
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
            };
        }
    }

    // Save the show_config flag before command_line is moved into config_for_compilation.
    let show_config_requested = command_line.compiler_options.show_config.is_true();

    // Read the config file (if found) and merge with command-line options.
    let config_for_compilation: ParsedCommandLine = if !config_file_name.is_empty() {
        let config_parsed = get_parsed_command_line_of_config_file(
            &config_file_name,
            &command_line.compiler_options,
            sys.current_directory(),
            sys.fs().as_ref(),
        );
        if !config_parsed.errors.is_empty() {
            let mut writer = sys.writer();
            for e in &config_parsed.errors {
                let _ = writeln!(writer, "{}", format_diagnostic(e, pretty));
            }
            return CommandLineResult {
                status: ExitStatus::DiagnosticsPresent_OutputsGenerated,
            };
        }
        config_parsed
    } else {
        command_line
    };

    // --showConfig: print the effective configuration as JSON and exit.
    if show_config_requested {
        show_config(sys, &config_for_compilation);
        return CommandLineResult {
            status: ExitStatus::Success,
        };
    }

    perform_compilation(sys, config_for_compilation, pretty)
}

// ────────────────────────────────────────────────────────────────────────────
// --showConfig
// ────────────────────────────────────────────────────────────────────────────

/// Print the effective compiler configuration as indented JSON.
///
/// Mirrors `execute.showConfig` in Go, but uses a simplified serialization
/// that covers the commonly-used options rather than reflecting over all
/// fields. Enum values are converted to their string names; paths are
/// emitted as-is (not relativized).
fn show_config(sys: &dyn System, config: &ParsedCommandLine) {
    use crate::json::Value;
    use crate::tsoptions as opts;

    let options = &config.compiler_options;
    let mut map = crate::json::Map::new();

    // Enum-valued options
    if let Some(s) = opts::script_target_name(options.target) {
        map.insert("target".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::module_kind_name(options.module) {
        map.insert("module".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::module_resolution_name(options.module_resolution) {
        map.insert("moduleResolution".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::jsx_emit_name(options.jsx) {
        map.insert("jsx".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::module_detection_name(options.module_detection) {
        map.insert("moduleDetection".to_string(), Value::String(s.to_string()));
    }
    if let Some(s) = opts::new_line_name(options.new_line) {
        map.insert("newLine".to_string(), Value::String(s.to_string()));
    }

    // String-valued options
    // `IsFilePath` options (outDir, rootDir, …) are stored as absolute paths
    // internally (mirroring Go's `normalizeNonListOptionValue`); for
    // `--showConfig` output we convert them back to paths relative to the
    // config file directory, matching Go's `serializeCompilerOptions`.
    let config_dir = tspath::get_directory_path(&config.config_file_name);
    let to_relative = |val: &str| -> String {
        if val.is_empty() {
            return val.to_string();
        }
        // If the value is already relative, keep it as-is (e.g. CLI-provided).
        if !tspath::path_is_absolute(val) {
            return val.to_string();
        }
        // Convert absolute → relative from config dir.
        let abs_val = tspath::get_normalized_absolute_path(val, "");
        let abs_config_dir = tspath::get_normalized_absolute_path(&config_dir, "");
        let abs_config_dir_with_sep = tspath::ensure_trailing_directory_separator(&abs_config_dir);
        if abs_val == abs_config_dir {
            // Value is the config directory itself.
            return ".".to_string();
        }
        if let Some(stripped) = abs_val.strip_prefix(&abs_config_dir_with_sep) {
            return stripped.to_string();
        }
        val.to_string()
    };
    for (name, val, is_path) in [
        ("outDir", &options.out_dir, true),
        ("outFile", &options.out_file, true),
        ("rootDir", &options.root_dir, true),
        ("declarationDir", &options.declaration_dir, true),
        ("sourceRoot", &options.source_root, true),
        ("mapRoot", &options.map_root, true),
        ("tsBuildInfoFile", &options.ts_build_info_file, true),
        ("jsxFactory", &options.jsx_factory, false),
        ("jsxFragmentFactory", &options.jsx_fragment_factory, false),
        ("jsxImportSource", &options.jsx_import_source, false),
        ("baseUrl", &options.base_url, true),
        ("locale", &options.locale, false),
    ] {
        if !val.is_empty() {
            let display = if is_path { to_relative(val) } else { val.clone() };
            map.insert(name.to_string(), Value::String(display));
        }
    }

    // List-valued options
    if !options.lib.is_empty() {
        map.insert(
            "lib".to_string(),
            Value::Array(
                options
                    .lib
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.types.is_empty() {
        map.insert(
            "types".to_string(),
            Value::Array(
                options
                    .types
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.type_roots.is_empty() {
        map.insert(
            "typeRoots".to_string(),
            Value::Array(
                options
                    .type_roots
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.root_dirs.is_empty() {
        map.insert(
            "rootDirs".to_string(),
            Value::Array(
                options
                    .root_dirs
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.module_suffixes.is_empty() {
        map.insert(
            "moduleSuffixes".to_string(),
            Value::Array(
                options
                    .module_suffixes
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !options.custom_conditions.is_empty() {
        map.insert(
            "customConditions".to_string(),
            Value::Array(
                options
                    .custom_conditions
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }

    // paths
    if let Some(paths) = &options.paths {
        let mut paths_map = crate::json::Map::new();
        for (k, v) in paths {
            paths_map.insert(
                k.clone(),
                Value::Array(v.iter().map(|s| Value::String(s.clone())).collect()),
            );
        }
        map.insert("paths".to_string(), Value::Object(paths_map));
    }

    // Boolean (Tristate) options — emit true/false when explicitly set
    let bool_opts: &[(&str, Tristate)] = &[
        ("allowJs", options.allow_js),
        (
            "allowImportingTsExtensions",
            options.allow_importing_ts_extensions,
        ),
        ("allowUmdGlobalAccess", options.allow_umd_global_access),
        ("allowUnreachableCode", options.allow_unreachable_code),
        ("allowUnusedLabels", options.allow_unused_labels),
        ("alwaysStrict", options.always_strict),
        ("checkJs", options.check_js),
        ("composite", options.composite),
        ("declaration", options.declaration),
        ("declarationMap", options.declaration_map),
        ("downlevelIteration", options.downlevel_iteration),
        ("emitBOM", options.emit_bom),
        ("emitDeclarationOnly", options.emit_declaration_only),
        ("emitDecoratorMetadata", options.emit_decorator_metadata),
        ("esModuleInterop", options.es_module_interop),
        (
            "exactOptionalPropertyTypes",
            options.exact_optional_property_types,
        ),
        ("experimentalDecorators", options.experimental_decorators),
        (
            "forceConsistentCasingInFileNames",
            options.force_consistent_casing_in_file_names,
        ),
        ("importHelpers", options.import_helpers),
        ("incremental", options.incremental),
        ("inlineSourceMap", options.inline_source_map),
        ("inlineSources", options.inline_sources),
        ("isolatedModules", options.isolated_modules),
        ("isolatedDeclarations", options.isolated_declarations),
        ("noCheck", options.no_check),
        ("noEmit", options.no_emit),
        ("noEmitOnError", options.no_emit_on_error),
        ("noErrorTruncation", options.no_error_truncation),
        (
            "noFallthroughCasesInSwitch",
            options.no_fallthrough_cases_in_switch,
        ),
        ("noImplicitAny", options.no_implicit_any),
        ("noImplicitOverride", options.no_implicit_override),
        ("noImplicitReturns", options.no_implicit_returns),
        ("noImplicitThis", options.no_implicit_this),
        ("noLib", options.no_lib),
        (
            "noPropertyAccessFromIndexSignature",
            options.no_property_access_from_index_signature,
        ),
        ("noResolve", options.no_resolve),
        (
            "noUncheckedIndexedAccess",
            options.no_unchecked_indexed_access,
        ),
        (
            "noUncheckedSideEffectImports",
            options.no_unchecked_side_effect_imports,
        ),
        ("noUnusedLocals", options.no_unused_locals),
        ("noUnusedParameters", options.no_unused_parameters),
        ("preserveConstEnums", options.preserve_const_enums),
        ("removeComments", options.remove_comments),
        ("resolveJsonModule", options.resolve_json_module),
        (
            "resolvePackageJsonExports",
            options.resolve_package_json_exports,
        ),
        (
            "resolvePackageJsonImports",
            options.resolve_package_json_imports,
        ),
        (
            "rewriteRelativeImportExtensions",
            options.rewrite_relative_import_extensions,
        ),
        ("skipLibCheck", options.skip_lib_check),
        ("strict", options.strict),
        ("strictBindCallApply", options.strict_bind_call_apply),
        (
            "strictBuiltinIteratorReturn",
            options.strict_builtin_iterator_return,
        ),
        ("strictFunctionTypes", options.strict_function_types),
        ("strictNullChecks", options.strict_null_checks),
        (
            "strictPropertyInitialization",
            options.strict_property_initialization,
        ),
        ("stripInternal", options.strip_internal),
        (
            "useDefineForClassFields",
            options.use_define_for_class_fields,
        ),
        (
            "useUnknownInCatchVariables",
            options.use_unknown_in_catch_variables,
        ),
        ("verbatimModuleSyntax", options.verbatim_module_syntax),
    ];
    for (name, t) in bool_opts {
        match *t {
            Tristate::True => {
                map.insert(name.to_string(), Value::Bool(true));
            }
            Tristate::False => {
                map.insert(name.to_string(), Value::Bool(false));
            }
            Tristate::Unknown => {}
        }
    }

    // Build the top-level TSConfig object
    let mut top = crate::json::Map::new();
    if !map.is_empty() {
        top.insert("compilerOptions".to_string(), Value::Object(map));
    }
    if config.has_files_spec {
        top.insert(
            "files".to_string(),
            Value::Array(
                config
                    .files_spec
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if config.has_include_spec {
        top.insert(
            "include".to_string(),
            Value::Array(
                config
                    .include
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if config.has_exclude_spec {
        top.insert(
            "exclude".to_string(),
            Value::Array(
                config
                    .exclude
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !config.references.is_empty() {
        let refs: Vec<Value> = config
            .references
            .iter()
            .map(|r| {
                let mut obj = crate::json::Map::new();
                obj.insert("path".to_string(), Value::String(r.original_path.clone()));
                if r.circular {
                    obj.insert("circular".to_string(), Value::Bool(true));
                }
                Value::Object(obj)
            })
            .collect();
        top.insert("references".to_string(), Value::Array(refs));
    }
    if config.compile_on_save == Some(true) {
        top.insert("compileOnSave".to_string(), Value::Bool(true));
    }

    let json = Value::Object(top);
    let mut writer = sys.writer();
    let _ = crate::json::marshal_indent_write(&mut writer, &json, "    ");
    let _ = writeln!(writer);
}

// ────────────────────────────────────────────────────────────────────────────
// perform_compilation
// ────────────────────────────────────────────────────────────────────────────

fn perform_compilation(
    sys: &dyn System,
    config: ParsedCommandLine,
    pretty: bool,
) -> CommandLineResult {
    let host: Arc<dyn CompilerHost> = Arc::new(CompilerHostImpl::new(
        sys.fs(),
        sys.current_directory().to_string(),
        sys.default_library_path().to_string(),
    ));

    let program = Arc::new(Program::new(ProgramOptions {
        config,
        host: Arc::clone(&host),
    }));

    let diags = program.get_diagnostics_to_report();
    let mut writer = sys.writer();
    let error_count = report_diagnostics(&mut writer, &diags, pretty).unwrap_or(0);

    // Run the type checker and merge semantic diagnostics.
    let semantic_diags: Vec<Arc<Diagnostic>> = program
        .get_semantic_diagnostics()
        .into_iter()
        .map(Arc::new)
        .collect();
    let semantic_error_count = if !semantic_diags.is_empty() {
        let mut writer = sys.writer();
        report_diagnostics(&mut writer, &semantic_diags, pretty).unwrap_or(0)
    } else {
        0
    };
    let error_count = error_count + semantic_error_count;

    let options = program.options();

    // Determine whether to emit output files.
    let should_emit = !options.no_emit.is_true()
        && !options.list_files_only.is_true()
        && (error_count == 0 || !options.no_emit_on_error.is_true());

    let mut emitted_any = false;
    if should_emit {
        let fs = sys.fs();
        let emit_result = program.emit(&|path, data| {
            // Ensure parent directory exists for real FS.
            if let Some(parent) = std::path::Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            fs.write_file(path, data)
        });
        emitted_any = !emit_result.emitted_files.is_empty();
        for diag in &emit_result.diagnostics {
            let _ = writeln!(writer, "{diag}");
        }
    }

    let status = if error_count > 0 {
        if options.no_emit.is_true() || !emitted_any {
            ExitStatus::DiagnosticsPresent_OutputsSkipped
        } else {
            ExitStatus::DiagnosticsPresent_OutputsGenerated
        }
    } else {
        ExitStatus::Success
    };

    // --listFiles / --listFilesOnly
    if options.list_files.is_true() || options.list_files_only.is_true() {
        for file in program.source_files() {
            let _ = writeln!(writer, "{}", file.file_name);
        }
    }

    CommandLineResult { status }
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Search ancestor directories for a config file named `config_name`.
///
/// Mirrors `findConfigFile` / `tspath.ForEachAncestorDirectory` in Go.
fn find_config_file(
    search_path: &str,
    file_exists: &dyn Fn(&str) -> bool,
    config_name: &str,
) -> String {
    let mut current = search_path.to_string();
    loop {
        let candidate = tspath::combine_paths(&current, &[config_name]);
        if file_exists(&candidate) {
            return candidate;
        }
        let parent = tspath::get_directory_path(&current);
        if parent == current {
            break;
        }
        current = parent;
    }
    String::new()
}

/// Whether diagnostics should be pretty (colored, with context).
fn should_be_pretty(sys: &dyn System, options: &CompilerOptions) -> bool {
    match options.pretty {
        Tristate::True => true,
        Tristate::False => false,
        Tristate::Unknown => default_is_pretty(sys),
    }
}

fn default_is_pretty(sys: &dyn System) -> bool {
    if sys.environment_variable("NO_COLOR").is_some() {
        return false;
    }
    if sys.environment_variable("FORCE_COLOR").is_some() {
        return true;
    }
    sys.write_output_is_tty()
}

fn print_help(sys: &dyn System) {
    let mut writer = sys.writer();
    let _ = writeln!(
        writer,
        "tsc: The TypeScript Compiler - Version {}",
        VERSION
    );
    let _ = writeln!(writer);
    let _ = writeln!(writer, "Common options:");
    let _ = writeln!(
        writer,
        "  -p, --project <file>   Compile the project given the path to its configuration file,"
    );
    let _ = writeln!(
        writer,
        "                         or to a folder with a 'tsconfig.json'."
    );
    let _ = writeln!(
        writer,
        "  -b, --build            Build one or more projects and their dependencies, if out of date."
    );
    let _ = writeln!(writer, "  -w, --watch            Watch input files.");
    let _ = writeln!(
        writer,
        "  -t, --target <ver>     Specify ECMAScript target version."
    );
    let _ = writeln!(
        writer,
        "  -m, --module <kind>    Specify module code generation."
    );
    let _ = writeln!(
        writer,
        "      --moduleResolution <kind>  Specify module resolution strategy."
    );
    let _ = writeln!(
        writer,
        "      --lib <lib,...>    Specify library files to be included in the compilation."
    );
    let _ = writeln!(
        writer,
        "      --outDir <dir>     Redirect output structure to the directory."
    );
    let _ = writeln!(
        writer,
        "      --outFile <file>   Concatenate and emit output to single file."
    );
    let _ = writeln!(
        writer,
        "      --sourceMap        Generates corresponding '.map' file."
    );
    let _ = writeln!(
        writer,
        "      --declaration, -d  Generates corresponding '.d.ts' file."
    );
    let _ = writeln!(
        writer,
        "      --strict           Enable all strict type-checking options."
    );
    let _ = writeln!(writer, "      --noEmit           Do not emit outputs.");
    let _ = writeln!(
        writer,
        "      --skipLibCheck     Skip type checking of declaration files."
    );
    let _ = writeln!(writer, "      --help, -h         Print this message.");
    let _ = writeln!(
        writer,
        "      --version, -v      Print the compiler's version."
    );
    let _ = writeln!(
        writer,
        "      --all              Show all compiler options."
    );
    let _ = writeln!(writer);
    let _ = writeln!(
        writer,
        "For more information, see https://www.typescriptlang.org/tsconfig"
    );
}

fn write_config_file(sys: &dyn System, options: &CompilerOptions) -> CommandLineResult {
    let config_file_name = tspath::combine_paths(sys.current_directory(), &["tsconfig.json"]);
    if sys.fs().file_exists(&config_file_name) {
        let mut writer = sys.writer();
        let diag = compiler_diagnostic(
            A_TSCONFIG_JSON_FILE_IS_ALREADY_DEFINED_AT_COLON_0,
            vec![config_file_name.clone()],
        );
        let _ = writeln!(writer, "{}", format_diagnostic(&diag, false));
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let config_text = generate_tsconfig(options);
    if let Err(err) = sys.fs().write_file(&config_file_name, &config_text) {
        let mut writer = sys.writer();
        let _ = writeln!(
            writer,
            "error TS5033: Could not write file '{config_file_name}': {err}."
        );
        return CommandLineResult {
            status: ExitStatus::DiagnosticsPresent_OutputsSkipped,
        };
    }

    let mut writer = sys.writer();
    let _ = writeln!(writer);
    let _ = writeln!(writer, "Created a new tsconfig.json");
    let _ = writeln!(writer, "You can learn more at https://aka.ms/tsconfig");
    CommandLineResult {
        status: ExitStatus::Success,
    }
}

fn generate_tsconfig(options: &CompilerOptions) -> String {
    let target = crate::tsoptions::script_target_name(options.target).unwrap_or("esnext");
    let module = crate::tsoptions::module_kind_name(options.module).unwrap_or("nodenext");
    let jsx = crate::tsoptions::jsx_emit_name(options.jsx).unwrap_or("react-jsx");
    let module_detection =
        crate::tsoptions::module_detection_name(options.module_detection).unwrap_or("force");

    format!(
        concat!(
            "{{\n",
            "  // Visit https://aka.ms/tsconfig to read more about this file\n",
            "  \"compilerOptions\": {{\n",
            "    // File Layout\n",
            "    //\"rootDir\": \"./src\",\n",
            "    //\"outDir\": \"./dist\",\n",
            "\n",
            "    // Environment Settings\n",
            "    // See also https://aka.ms/tsconfig/module\n",
            "    \"module\": \"{module}\",\n",
            "    \"target\": \"{target}\",\n",
            "    \"types\": [],\n",
            "    // For nodejs:\n",
            "    // \"lib\": [\"esnext\"],\n",
            "    // \"types\": [\"node\"],\n",
            "    // and npm install -D @types/node\n",
            "\n",
            "    // Other Outputs\n",
            "    \"sourceMap\": true,\n",
            "    \"declaration\": true,\n",
            "    \"declarationMap\": true,\n",
            "\n",
            "    // Stricter Typechecking Options\n",
            "    \"noUncheckedIndexedAccess\": true,\n",
            "    \"exactOptionalPropertyTypes\": true,\n",
            "\n",
            "    // Style Options\n",
            "    //\"noImplicitReturns\": true,\n",
            "    //\"noImplicitOverride\": true,\n",
            "    //\"noUnusedLocals\": true,\n",
            "    //\"noUnusedParameters\": true,\n",
            "    //\"noFallthroughCasesInSwitch\": true,\n",
            "    //\"noPropertyAccessFromIndexSignature\": true,\n",
            "\n",
            "    // Recommended Options\n",
            "    \"strict\": true,\n",
            "    \"jsx\": \"{jsx}\",\n",
            "    \"verbatimModuleSyntax\": true,\n",
            "    \"isolatedModules\": true,\n",
            "    \"noUncheckedSideEffectImports\": true,\n",
            "    \"moduleDetection\": \"{module_detection}\",\n",
            "    \"skipLibCheck\": true\n",
            "  }}\n",
            "}}\n"
        ),
        module = module,
        target = target,
        jsx = jsx,
        module_detection = module_detection
    )
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundled::BundledFS;
    use crate::vfs::InMemoryFS;
    use std::sync::Mutex;

    /// A test System backed by an in-memory FS and a captured output buffer.
    struct TestSystem {
        fs: Arc<BundledFS>,
        cwd: String,
        output: Arc<Mutex<Vec<u8>>>,
    }

    impl TestSystem {
        fn new(inner_fs: Arc<InMemoryFS>, cwd: &str) -> Self {
            Self {
                fs: Arc::new(BundledFS::new(inner_fs as Arc<dyn FS>)),
                cwd: cwd.to_string(),
                output: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn output_string(&self) -> String {
            String::from_utf8_lossy(&self.output.lock().unwrap()).to_string()
        }
    }

    impl System for TestSystem {
        fn writer(&self) -> Box<dyn Write + Send> {
            Box::new(BufferWriter {
                buf: Arc::clone(&self.output),
            })
        }
        fn fs(&self) -> Arc<dyn FS> {
            Arc::clone(&self.fs) as Arc<dyn FS>
        }
        fn default_library_path(&self) -> &str {
            "bundled:///libs"
        }
        fn current_directory(&self) -> &str {
            &self.cwd
        }
        fn write_output_is_tty(&self) -> bool {
            false
        }
        fn width_of_terminal(&self) -> usize {
            80
        }
        fn environment_variable(&self, _name: &str) -> Option<String> {
            None
        }
    }

    /// A writer that appends to a shared, mutex-protected buffer.
    struct BufferWriter {
        buf: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for BufferWriter {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.buf.lock().unwrap().extend_from_slice(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn version_flag_prints_version() {
        let fs = Arc::new(InMemoryFS::new());
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["--version".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.output_string().contains("Version 7.1.0-dev"));
    }

    #[test]
    fn help_flag_prints_help() {
        let fs = Arc::new(InMemoryFS::new());
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["--help".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.output_string().contains("Common options"));
    }

    #[test]
    fn init_flag_writes_tsconfig() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        let sys = TestSystem::new(Arc::clone(&fs), "/proj");
        let args = vec!["--init".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let config = fs.read_file("/proj/tsconfig.json").unwrap();
        assert!(config.contains("\"compilerOptions\""));
        assert!(config.contains("\"strict\": true"));
        assert!(sys.output_string().contains("Created a new tsconfig.json"));
    }

    #[test]
    fn init_flag_errors_when_tsconfig_exists() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/index.ts", "");
        fs.insert_file("/proj/tsconfig.json", "{}");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["--init".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("already defined"));
    }

    #[test]
    fn no_config_no_files_shows_help_and_errors() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        let sys = TestSystem::new(fs, "/proj");
        let args: Vec<String> = vec![];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("tsconfig.json"));
    }

    #[test]
    fn compiles_simple_file() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x: number = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        // No type checking yet, so parse-only → success.
        if result.status != ExitStatus::Success {
            panic!(
                "Expected Success but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
    }

    #[test]
    fn non_ascii_invalid_character_does_not_panic_in_command_line() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/middle-dot.ts", "·");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--noEmitOnError".to_string(),
            "/proj/middle-dot.ts".to_string(),
        ];

        let result = command_line(&sys, &args);

        // The `·` (U+00B7) is an invalid character. Scanner errors are now
        // wired into the parser diagnostics pipeline, so the CLI should report
        // diagnostics present. This test primarily guards against the UTF-8
        // slicing panic that `·` originally triggered.
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
    }

    #[test]
    fn finds_config_in_ancestor_directory() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/root");
        fs.insert_dir("/root/sub");
        fs.insert_file(
            "/root/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true},"files":["sub/a.ts"]}"#,
        );
        fs.insert_file("/root/sub/a.ts", "let x = 1;");
        let sys = TestSystem::new(fs, "/root/sub");
        let args: Vec<String> = vec![];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
    }

    #[test]
    fn build_mode_produces_output() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x: number = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true},"files":["a.ts"]}"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-b".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(
            result.status,
            ExitStatus::Success,
            "output:\n{}",
            sys.output_string()
        );
        // The -b flag should produce a .js output file.
        assert!(sys.fs().file_exists("/proj/a.js"));
        let js = sys.fs().read_file("/proj/a.js").unwrap();
        assert_eq!(js.trim(), "let x = 1;");
    }

    #[test]
    fn regular_compilation_produces_output() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/b.ts",
            "function foo(a: number): number { return a; }",
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/b.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.fs().file_exists("/proj/b.js"));
        let js = sys.fs().read_file("/proj/b.js").unwrap();
        assert!(js.contains("function foo(a)"));
        assert!(!js.contains(": number"));
    }

    #[test]
    fn no_emit_flag_skips_output() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/c.ts", "let y = 2;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--noEmit".to_string(),
            "/proj/c.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(!sys.fs().file_exists("/proj/c.js"));
    }

    #[test]
    fn out_dir_redirects_output() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/d.ts", "let z: string = \"hi\";");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--outDir".to_string(),
            "/proj/dist".to_string(),
            "/proj/src/d.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.fs().file_exists("/proj/dist/d.js"));
        let js = sys.fs().read_file("/proj/dist/d.js").unwrap();
        assert_eq!(js.trim(), "let z = \"hi\";");
    }

    #[test]
    fn no_emit_on_error_skips_output_when_errors() {
        // File with a syntax error: `interface` without a name triggers
        // "Expected identifier" in parse_interface_declaration.
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/e.ts", "interface { x: number }\nlet y = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--noEmitOnError".to_string(),
            "/proj/e.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        // Errors present, no output emitted → DiagnosticsPresent_OutputsSkipped.
        if result.status != ExitStatus::DiagnosticsPresent_OutputsSkipped {
            panic!(
                "Expected DiagnosticsPresent_OutputsSkipped but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
        assert!(!sys.fs().file_exists("/proj/e.js"));
    }

    #[test]
    fn no_emit_on_error_emits_when_no_errors() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/f.ts", "let x: number = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--noEmitOnError".to_string(),
            "/proj/f.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.fs().file_exists("/proj/f.js"));
        let js = sys.fs().read_file("/proj/f.js").unwrap();
        assert_eq!(js.trim(), "let x = 1;");
    }

    #[test]
    fn errors_without_no_emit_on_error_still_emits() {
        // File with a syntax error (interface without name) but WITHOUT --noEmitOnError.
        // The emitter should still produce output.
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/g.ts", "interface { x: number }\nlet y = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/g.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        // Errors present, but output was emitted → DiagnosticsPresent_OutputsGenerated.
        if result.status != ExitStatus::DiagnosticsPresent_OutputsGenerated {
            panic!(
                "Expected DiagnosticsPresent_OutputsGenerated but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
        assert!(sys.fs().file_exists("/proj/g.js"));
    }

    #[test]
    fn list_files_only_skips_output() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/h.ts", "let x: number = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--listFilesOnly".to_string(),
            "/proj/h.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(!sys.fs().file_exists("/proj/h.js"));
        // --listFilesOnly should print the file name.
        assert!(sys.output_string().contains("/proj/h.ts"));
    }

    #[test]
    fn build_mode_with_out_dir() {
        // Verify -b flag works in combination with --outDir.
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/i.ts", "let value: number = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true,"outDir":"/proj/dist"},"files":["src/i.ts"]}"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-b".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(
            result.status,
            ExitStatus::Success,
            "output:\n{}",
            sys.output_string()
        );
        assert!(sys.fs().file_exists("/proj/dist/src/i.js"));
        let js = sys.fs().read_file("/proj/dist/src/i.js").unwrap();
        assert!(js.contains("let value = 1;"));
        assert!(!js.contains(": number"));
    }

    #[test]
    fn build_mode_builds_referenced_solution_project() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
        );
        fs.insert_file(
            "/proj/tsconfig.app.json",
            r#"{"compilerOptions":{"noLib":true,"outDir":"/proj/dist"},"include":["src"]}"#,
        );
        fs.insert_file("/proj/src/app.ts", "export const app: number = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-b".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(
            result.status,
            ExitStatus::Success,
            "output:\n{}",
            sys.output_string()
        );
        assert!(sys.fs().file_exists("/proj/dist/src/app.js"));
    }

    // ───────────────────────────────────────────────────────────────────────
    // --showConfig tests (simplified ports of tsctests/showconfig_test.go)
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn show_config_with_boolean_option() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/index.ts", "");
        fs.insert_file("/proj/tsconfig.json", "{}");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["--showConfig".to_string(), "--noUnusedLocals".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"compilerOptions\""), "output: {out}");
        assert!(out.contains("\"noUnusedLocals\": true"), "output: {out}");
    }

    #[test]
    fn show_config_with_enum_options() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/index.ts", "");
        fs.insert_file("/proj/tsconfig.json", "{}");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--showConfig".to_string(),
            "--target".to_string(),
            "es5".to_string(),
            "--jsx".to_string(),
            "react".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"target\": \"es5\""), "output: {out}");
        assert!(out.contains("\"jsx\": \"react\""), "output: {out}");
    }

    #[test]
    fn show_config_with_list_options() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/index.ts", "");
        fs.insert_file("/proj/tsconfig.json", "{}");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--showConfig".to_string(),
            "--types".to_string(),
            "jquery,mocha".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"types\""), "output: {out}");
        assert!(out.contains("jquery"), "output: {out}");
        assert!(out.contains("mocha"), "output: {out}");
    }

    #[test]
    fn show_config_with_tsconfig_file() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/index.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": {
                    "esModuleInterop": true,
                    "target": "es5",
                    "module": "commonjs",
                    "strict": true
                },
                "include": ["src/*"]
            }"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "-p".to_string(),
            "tsconfig.json".to_string(),
            "--showConfig".to_string(),
        ];
        let result = command_line(&sys, &args);
        if result.status != ExitStatus::Success {
            panic!(
                "Expected Success but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
        let out = sys.output_string();
        assert!(out.contains("\"target\": \"es5\""), "output: {out}");
        assert!(out.contains("\"module\": \"commonjs\""), "output: {out}");
        assert!(out.contains("\"strict\": true"), "output: {out}");
        assert!(out.contains("\"include\""), "output: {out}");
        assert!(out.contains("src/*"), "output: {out}");
    }

    #[test]
    fn show_config_with_paths() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/index.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": {
                    "baseUrl": ".",
                    "paths": {
                        "@root/*": ["./*"],
                        "@common/*": ["src/common/*"]
                    }
                }
            }"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "-p".to_string(),
            "tsconfig.json".to_string(),
            "--showConfig".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"paths\""), "output: {out}");
        assert!(out.contains("@root/*"), "output: {out}");
        assert!(out.contains("@common/*"), "output: {out}");
        assert!(out.contains("\"baseUrl\": \".\""), "output: {out}");
    }

    #[test]
    fn show_config_with_exclude() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/index.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": { "strict": true },
                "exclude": ["test"]
            }"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "-p".to_string(),
            "tsconfig.json".to_string(),
            "--showConfig".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"exclude\""), "output: {out}");
        assert!(out.contains("test"), "output: {out}");
    }

    #[test]
    fn show_config_with_advanced_options() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/index.ts", "");
        fs.insert_file("/proj/tsconfig.json", "{}");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--showConfig".to_string(),
            "--declaration".to_string(),
            "--declarationDir".to_string(),
            "lib".to_string(),
            "--skipLibCheck".to_string(),
            "--noErrorTruncation".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"declaration\": true"), "output: {out}");
        assert!(out.contains("\"declarationDir\": \"lib\""), "output: {out}");
        assert!(out.contains("\"skipLibCheck\": true"), "output: {out}");
        assert!(out.contains("\"noErrorTruncation\": true"), "output: {out}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // -p / --project tests (simplified ports of tsctests/tsc_test.go)
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn project_with_file_path() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/first.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true,"noEmit":true},"files":["first.ts"]}"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), "/proj/tsconfig.json".to_string()];
        let result = command_line(&sys, &args);
        if result.status != ExitStatus::Success {
            panic!(
                "Expected Success but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
    }

    #[test]
    fn project_with_folder_path() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/first.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true,"noEmit":true},"files":["first.ts"]}"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), "/proj".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
    }

    #[test]
    fn project_with_dot_folder() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/first.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true,"noEmit":true},"files":["first.ts"]}"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), ".".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
    }

    #[test]
    fn project_with_nonexistent_path() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), "/proj/nonexistent.json".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("does not exist"));
    }

    #[test]
    fn project_with_nonexistent_directory() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), "/proj/nonexistent".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("does not exist"));
    }

    #[test]
    fn project_mixed_with_files_errors() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{"compilerOptions":{"noLib":true}}"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "-p".to_string(),
            "/proj/tsconfig.json".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("cannot be mixed"));
    }

    #[test]
    fn empty_tsconfig_file() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/first.ts", "export const a = 1;");
        fs.insert_file("/proj/tsconfig.json", "");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), ".".to_string(), "--noLib".to_string()];
        let result = command_line(&sys, &args);
        if result.status != ExitStatus::Success {
            panic!(
                "Expected Success but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    // --watch + --listFilesOnly error
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn watch_and_list_files_only_errors() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--watch".to_string(),
            "--listFilesOnly".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("cannot be combined"));
    }

    // ───────────────────────────────────────────────────────────────────────
    // --build not first argument
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn build_not_first_argument() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x: number = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--build".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::DiagnosticsPresent_OutputsSkipped);
        assert!(sys.output_string().contains("must be the first"));
    }

    // ───────────────────────────────────────────────────────────────────────
    // Multiple files compilation
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn compiles_multiple_files() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "export const x = 1;");
        fs.insert_file("/proj/b.ts", "export const y = 2;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/a.ts".to_string(),
            "/proj/b.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.fs().file_exists("/proj/a.js"));
        assert!(sys.fs().file_exists("/proj/b.js"));
    }

    // ───────────────────────────────────────────────────────────────────────
    // --declaration emit
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn declaration_option_compiles_with_flag() {
        // --declaration is parsed and accepted; .d.ts emit is not yet implemented
        // but the compilation should still succeed and produce .js output.
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file(
            "/proj/a.ts",
            "export const x: number = 1;\nexport function foo(a: number): number { return a; }",
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--declaration".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        if result.status != ExitStatus::Success {
            panic!(
                "Expected Success but got {:?}. Output:\n{}",
                result.status,
                sys.output_string()
            );
        }
        assert!(sys.fs().file_exists("/proj/a.js"));
    }

    // ───────────────────────────────────────────────────────────────────────
    // --sourceMap emit
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn source_map_option_compiles_with_flag() {
        // --sourceMap is parsed and accepted; .map emit is not yet wired into
        // the emitter, but the compilation should still succeed and produce .js.
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x: number = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--sourceMap".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.fs().file_exists("/proj/a.js"));
    }

    // ───────────────────────────────────────────────────────────────────────
    // Parse enum options
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn parse_enum_options_module_target() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x = 1;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--module".to_string(),
            "commonjs".to_string(),
            "--target".to_string(),
            "es5".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.fs().file_exists("/proj/a.js"));
    }

    #[test]
    fn show_config_with_module_and_target() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/index.ts", "");
        fs.insert_file("/proj/tsconfig.json", "{}");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--showConfig".to_string(),
            "--module".to_string(),
            "nodenext".to_string(),
            "--target".to_string(),
            "esnext".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"module\": \"nodenext\""), "output: {out}");
        assert!(out.contains("\"target\": \"esnext\""), "output: {out}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // Color / environment variable tests
    // ───────────────────────────────────────────────────────────────────────

    /// A test system that allows setting environment variables.
    struct EnvTestSystem {
        fs: Arc<BundledFS>,
        cwd: String,
        output: Arc<Mutex<Vec<u8>>>,
        env: std::collections::HashMap<String, String>,
    }

    impl EnvTestSystem {
        fn new(inner_fs: Arc<InMemoryFS>, cwd: &str) -> Self {
            Self {
                fs: Arc::new(BundledFS::new(inner_fs as Arc<dyn FS>)),
                cwd: cwd.to_string(),
                output: Arc::new(Mutex::new(Vec::new())),
                env: std::collections::HashMap::new(),
            }
        }

        fn with_env(mut self, key: &str, val: &str) -> Self {
            self.env.insert(key.to_string(), val.to_string());
            self
        }

        fn output_string(&self) -> String {
            String::from_utf8_lossy(&self.output.lock().unwrap()).to_string()
        }
    }

    impl System for EnvTestSystem {
        fn writer(&self) -> Box<dyn Write + Send> {
            Box::new(BufferWriter {
                buf: Arc::clone(&self.output),
            })
        }
        fn fs(&self) -> Arc<dyn FS> {
            Arc::clone(&self.fs) as Arc<dyn FS>
        }
        fn default_library_path(&self) -> &str {
            "bundled:///libs"
        }
        fn current_directory(&self) -> &str {
            &self.cwd
        }
        fn write_output_is_tty(&self) -> bool {
            false
        }
        fn width_of_terminal(&self) -> usize {
            80
        }
        fn environment_variable(&self, name: &str) -> Option<String> {
            self.env.get(name).cloned()
        }
    }

    #[test]
    fn no_color_env_disables_pretty() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "interface { x: number }");
        let sys = EnvTestSystem::new(fs, "/proj").with_env("NO_COLOR", "true");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        // Should have errors but output should not contain color codes.
        let out = sys.output_string();
        assert!(
            !out.contains("\x1b["),
            "output should not contain ANSI codes: {out}"
        );
        // Status indicates errors present.
        assert!(result.status != ExitStatus::Success);
    }

    #[test]
    fn force_color_enables_pretty() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "interface { x: number }");
        // FORCE_COLOR only takes effect when write_output_is_tty returns true.
        // Since our test system returns false, pretty is still disabled.
        // We just verify the command runs without crashing.
        let sys = EnvTestSystem::new(fs, "/proj").with_env("FORCE_COLOR", "true");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "/proj/a.ts".to_string(),
        ];
        let _result = command_line(&sys, &args);
        // Just verify it doesn't crash.
    }

    // ───────────────────────────────────────────────────────────────────────
    // --listFiles
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn list_files_prints_source_files() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x = 1;");
        fs.insert_file("/proj/b.ts", "let y = 2;");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "--noLib".to_string(),
            "--ignoreConfig".to_string(),
            "--listFiles".to_string(),
            "/proj/a.ts".to_string(),
            "/proj/b.ts".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("/proj/a.ts"), "output: {out}");
        assert!(out.contains("/proj/b.ts"), "output: {out}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // showConfig with compileOnSave
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn show_config_with_compile_on_save() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/index.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": { "strict": true },
                "compileOnSave": true,
                "include": ["src/*"]
            }"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "-p".to_string(),
            "tsconfig.json".to_string(),
            "--showConfig".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"compileOnSave\": true"), "output: {out}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // showConfig with references
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn show_config_with_references() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/src/index.ts", "export const a = 1;");
        fs.insert_file(
            "/proj/tsconfig.json",
            r#"{
                "compilerOptions": { "composite": true, "strict": true },
                "references": [{ "path": "./packages/a" }]
            }"#,
        );
        let sys = TestSystem::new(fs, "/proj");
        let args = vec![
            "-p".to_string(),
            "tsconfig.json".to_string(),
            "--showConfig".to_string(),
        ];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        let out = sys.output_string();
        assert!(out.contains("\"composite\": true"), "output: {out}");
        assert!(out.contains("\"references\""), "output: {out}");
        assert!(out.contains("\"path\": \"./packages/a\""), "output: {out}");
    }

    // ───────────────────────────────────────────────────────────────────────
    // Missing file in tsconfig
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn missing_file_in_tsconfig_reports_error() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/tsconfig.json", r#"{"files":["./doesNotExist.ts"]}"#);
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["-p".to_string(), "./tsconfig.json".to_string()];
        let result = command_line(&sys, &args);
        // Missing files should result in diagnostics.
        assert_ne!(result.status, ExitStatus::Success);
    }

    // ───────────────────────────────────────────────────────────────────────
    // --all flag
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn all_flag_prints_help() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        let sys = TestSystem::new(fs, "/proj");
        let args = vec!["--all".to_string()];
        let result = command_line(&sys, &args);
        assert_eq!(result.status, ExitStatus::Success);
        assert!(sys.output_string().contains("Common options"));
    }
}
