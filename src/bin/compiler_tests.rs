//! Compiler baseline test runner — ported from typescript-go's
//! `testrunner/compiler_runner.go`.
//!
//! Runs multi-file test cases, produces error baselines, and compares
//! against reference outputs.
//!
//! Test cases live in `tests/cases/compiler/` (local) or
//! `../typescript-go/_submodules/TypeScript/tests/cases/conformance/` (submodule).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::Regex;

use tsox::bundled::BundledFS;
use tsox::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox::core::compiler_options::{
    CompilerOptions, JsxEmit, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use tsox::core::tristate::Tristate;
use tsox::diagnosticwriter::format_diagnostic;
use tsox::testutil::baseline::{self, BaselineOptions};
use tsox::testutil::test_case_parser::{TestCaseContent, parse_test_files};
use tsox::tsoptions::ParsedCommandLine;
use tsox::vfs::{FS, InMemoryFS};

/// The virtual current directory used by all compiler tests.
const SRC_FOLDER: &str = "/.src";

/// A compiled test result containing diagnostics.
struct CompilationOutput {
    diagnostics: Vec<String>,
    emitted_files: BTreeMap<String, String>,
}

/// Compile a single test case and produce diagnostics + emit output.
fn compile_test_case(content: &TestCaseContent) -> CompilationOutput {
    let mut fs = InMemoryFS::new();

    // Write all test units to the virtual FS.
    let mut file_names: Vec<String> = Vec::new();
    for unit in &content.units {
        let abs_path = normalize_abs_path(&unit.name, &content.current_directory);
        let _ = fs.write_file(&abs_path, &unit.content);
        file_names.push(abs_path);
    }

    // Build compiler options from test settings.
    let mut options = CompilerOptions::default();
    apply_test_settings(&mut options, &content.settings);

    // Set test defaults matching Go's harness.
    options.skip_default_lib_check = Tristate::True;
    options.no_error_truncation = Tristate::True;

    // Create program.
    let config = ParsedCommandLine {
        compiler_options: options,
        file_names,
        errors: Vec::new(),
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
        watch: false,
        watch_options: Default::default(),
    };

    let host = CompilerHostImpl::new(
        Arc::new(BundledFS::new(Arc::new(fs))),
        String::new(),
        content.current_directory.clone(),
    );

    let program = Arc::new(Program::new(ProgramOptions {
        config,
        host: Arc::new(host),
    }));

    // Collect diagnostics.
    let pretty = false;
    let locale = None;
    let mut diagnostics: Vec<String> = Vec::new();
    for d in program.get_diagnostics_to_report() {
        diagnostics.push(format_diagnostic(&d, pretty, locale));
    }
    for d in &program.get_semantic_diagnostics() {
        diagnostics.push(format_diagnostic(d, pretty, locale));
    }

    // Collect emitted files.
    let mut emitted: BTreeMap<String, String> = BTreeMap::new();
    let emit_result = program.emit(&|_path, _data| Ok(()));
    for path in &emit_result.emitted_files {
        emitted.insert(path.clone(), String::new());
    }

    CompilationOutput {
        diagnostics,
        emitted_files: emitted,
    }
}

/// Format diagnostics into the baseline error format.
fn format_error_baseline(output: &CompilationOutput) -> String {
    if output.diagnostics.is_empty() {
        return String::new();
    }
    let mut result = String::new();
    for diag in &output.diagnostics {
        result.push_str(diag);
        result.push('\n');
    }
    result
}

/// Apply `// @Option: value` settings to CompilerOptions.
fn apply_test_settings(
    options: &mut CompilerOptions,
    settings: &std::collections::HashMap<String, String>,
) {
    for (key, value) in settings {
        let v = value.trim();
        match key.as_str() {
            "strict" => options.strict = parse_tristate(v),
            "noimplicitany" => options.no_implicit_any = parse_tristate(v),
            "strictnullchecks" => options.strict_null_checks = parse_tristate(v),
            "strictfunctiontypes" => options.strict_function_types = parse_tristate(v),
            "strictbindcallapply" => options.strict_bind_call_apply = parse_tristate(v),
            "strictpropertyinitialization" => {
                options.strict_property_initialization = parse_tristate(v)
            }
            "noimplicitthis" => options.no_implicit_this = parse_tristate(v),
            "alwaysstrict" => options.always_strict = parse_tristate(v),
            "noimplicitreturns" => options.no_implicit_returns = parse_tristate(v),
            "nofallthroughcasesinswitch" => {
                options.no_fallthrough_cases_in_switch = parse_tristate(v)
            }
            "skiplibcheck" => options.skip_lib_check = parse_tristate(v),
            "skipdefaultlibcheck" => options.skip_default_lib_check = parse_tristate(v),
            "allowjs" => options.allow_js = parse_tristate(v),
            "checkjs" => options.check_js = parse_tristate(v),
            "declaration" => options.declaration = parse_tristate(v),
            "sourcemap" => options.source_map = parse_tristate(v),
            "noemit" => options.no_emit = parse_tristate(v),
            "isolatedmodules" => options.isolated_modules = parse_tristate(v),
            "verbatimmodulesyntax" => options.verbatim_module_syntax = parse_tristate(v),
            "target" => {
                if let Some(kind) = parse_target(v) {
                    options.target = kind;
                }
            }
            "module" => {
                if let Some(kind) = parse_module(v) {
                    options.module = kind;
                }
            }
            "moduleresolution" => {
                if let Some(kind) = parse_module_resolution(v) {
                    options.module_resolution = kind;
                }
            }
            "jsx" => {
                if let Some(kind) = parse_jsx(v) {
                    options.jsx = kind;
                }
            }
            "lib" => {
                options.lib = v.split(',').map(|s| s.trim().to_string()).collect();
            }
            "outdir" => options.out_dir = normalize_option_path(v),
            "rootdir" => options.root_dir = normalize_option_path(v),
            "baseurl" => options.base_url = normalize_option_path(v),
            "experimentaldecorators" => options.experimental_decorators = parse_tristate(v),
            "emitdecoratormetadata" => options.emit_decorator_metadata = parse_tristate(v),
            "usesdefineforclassfields" => options.use_define_for_class_fields = parse_tristate(v),
            _ => {} // Unknown settings silently ignored
        }
    }
}

fn parse_tristate(v: &str) -> Tristate {
    match v.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Tristate::True,
        "false" | "0" | "no" => Tristate::False,
        _ => Tristate::Unknown,
    }
}

fn parse_target(v: &str) -> Option<ScriptTarget> {
    match v.trim().to_lowercase().as_str() {
        "es5" => Some(ScriptTarget::ES5),
        "es6" | "es2015" => Some(ScriptTarget::ES2015),
        "es2016" => Some(ScriptTarget::ES2016),
        "es2017" => Some(ScriptTarget::ES2017),
        "es2018" => Some(ScriptTarget::ES2018),
        "es2019" => Some(ScriptTarget::ES2019),
        "es2020" => Some(ScriptTarget::ES2020),
        "es2021" => Some(ScriptTarget::ES2021),
        "es2022" => Some(ScriptTarget::ES2022),
        "esnext" => Some(ScriptTarget::ESNext),
        _ => None,
    }
}

fn parse_module(v: &str) -> Option<ModuleKind> {
    match v.trim().to_lowercase().as_str() {
        "none" => Some(ModuleKind::None),
        "commonjs" => Some(ModuleKind::CommonJS),
        "amd" => Some(ModuleKind::AMD),
        "system" => Some(ModuleKind::System),
        "umd" => Some(ModuleKind::UMD),
        "es6" | "es2015" => Some(ModuleKind::ES2015),
        "es2020" => Some(ModuleKind::ES2020),
        "es2022" => Some(ModuleKind::ES2022),
        "esnext" => Some(ModuleKind::ESNext),
        "node16" => Some(ModuleKind::Node16),
        "nodenext" => Some(ModuleKind::NodeNext),
        "preserve" => Some(ModuleKind::Preserve),
        _ => None,
    }
}

fn parse_module_resolution(v: &str) -> Option<ModuleResolutionKind> {
    match v.trim().to_lowercase().as_str() {
        "classic" => Some(ModuleResolutionKind::Classic),
        "node" | "node10" => Some(ModuleResolutionKind::Node10),
        "node16" => Some(ModuleResolutionKind::Node16),
        "nodenext" => Some(ModuleResolutionKind::NodeNext),
        "bundler" => Some(ModuleResolutionKind::Bundler),
        _ => None,
    }
}

fn parse_jsx(v: &str) -> Option<JsxEmit> {
    match v.trim().to_lowercase().as_str() {
        "preserve" => Some(JsxEmit::Preserve),
        "react" => Some(JsxEmit::React),
        "react-native" => Some(JsxEmit::ReactNative),
        "react-jsx" => Some(JsxEmit::ReactJSX),
        "react-jsxdev" => Some(JsxEmit::ReactJSXDev),
        _ => None,
    }
}

fn normalize_option_path(v: &str) -> String {
    let v = v.trim();
    if v.starts_with('/') || v.starts_with('\\') {
        v.to_string()
    } else {
        format!("{SRC_FOLDER}/{v}")
    }
}

fn normalize_abs_path(name: &str, current_dir: &str) -> String {
    if name.starts_with('/') {
        name.to_string()
    } else if name.starts_with("./") || name.starts_with("../") {
        format!("{current_dir}/{name}")
    } else {
        format!("{current_dir}/{name}")
    }
}

/// Run all compiler baseline tests from a directory.
pub fn run_compiler_baselines(test_dir: &Path, suite_name: &str) {
    let ts_file_re = Regex::new(r"\.tsx?$").unwrap();
    let files = baseline::enumerate_test_files(test_dir, &ts_file_re);

    for rel_path in &files {
        let full_path = test_dir.join(rel_path);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let test_case = parse_test_files(&content, rel_path);
        let output = compile_test_case(&test_case);
        let error_baseline = format_error_baseline(&output);

        let baseline_name = format!(
            "{}.errors.txt",
            Path::new(rel_path).with_extension("").to_string_lossy()
        );

        let opts = BaselineOptions::new(suite_name);
        if let Err(e) = baseline::run(&baseline_name, &error_baseline, &opts) {
            eprintln!("FAIL: {baseline_name}\n{e}");
        } else {
            eprintln!("PASS: {baseline_name}");
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Local regression tests.
    let local_dir = manifest_dir.join("tests/cases/compiler");
    if local_dir.exists() {
        println!("=== Running local compiler tests ===");
        run_compiler_baselines(&local_dir, "compiler");
    }

    // TypeScript submodule conformance tests (if available).
    let submodule_dir =
        manifest_dir.join("../typescript-go/_submodules/TypeScript/tests/cases/conformance");
    if submodule_dir.exists() {
        println!("=== Running submodule conformance tests ===");
        run_compiler_baselines(&submodule_dir, "conformance");
    }
}
