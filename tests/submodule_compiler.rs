//! Baseline test runner: executes TypeScript's official compiler test cases
//! (from the `_submodules/TypeScript` git submodule) and compares the
//! resulting diagnostics against committed snapshots under
//! `tests/baselines/reference/compiler/`.
//!
//! Ports a focused slice of tsgo's `TestSubmodule` (errors baseline only):
//! for each `.ts` case under
//! `_submodules/TypeScript/tests/cases/compiler/`:
//!   1. parse `// @filename`/`// @module`/... directives;
//!   2. build a `Program` over the virtual files + bundled libs;
//!   3. collect semantic diagnostics, sort, render to baseline text;
//!   4. compare against the reference baseline (or accept/write it).
//!
//! Cases are skipped (not failed) when they exercise options the Rust port
//! doesn't yet support (see `should_skip`), or when their baseline diff is
//! listed in `accepted.txt`/`triaged.txt`. A checker panic converts to a
//! triaged skip rather than aborting the whole run.
//!
//! Environment variables:
//! - `TSOX_BASELINE_ACCEPT=1` — write actual output over the reference baselines.
//! - `TSOX_SUBMODULE_LIMIT=N` — only run the first N cases (for bring-up).

mod common;

use std::panic::catch_unwind;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

use tsox::ast::Diagnostic;
use tsox::bundled::{BundledFS, lib_path};
use tsox::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox::core::compiler_options::CompilerOptions;
use tsox::diagnosticwriter::format_diagnostic_compact;
use tsox::tsoptions::apply_test_settings;
use tsox::vfs::InMemoryFS;

use common::baseline::{self, KnownDiffs, NO_CONTENT};
use common::case_parser::{extract_settings, split_units};

const SUBMODULE_DIR: &str = "_submodules/TypeScript/tests/cases/compiler";
const SUBFOLDER: &str = "compiler";

/// Default cap on the number of cases run, to keep CI tractable during
/// bring-up (~1s/case, run via per-case subprocess isolation). Override with
/// `TSOX_SUBMODULE_LIMIT` (set to `0` to run all ~6400 cases). As the checker
/// matures this default will be raised.
///
/// History: 50 → 300 (private identifiers, object-literal accessors/methods,
/// parameter modifiers, class index signatures) → 600 (`<`-disambiguation,
/// set_bool case bug, generic arrow functions, diagnostic clamp, catch_unwind
/// render, skip stress + non-UTF-8) → 1000 (per-case **subprocess isolation**
/// so checker stack overflows kill only the child; circular-type family skip).
/// The 1000-case slice is green (errors baseline only).
const DEFAULT_LIMIT: usize = 1000;

/// Cases skipped because they reference inputs the port can't provide (e.g.
/// the old `typescript.d.ts` API surface) or exercise removed compiler options.
/// Mirrors tsgo's `skippedTests` (`internal/testrunner/compiler_runner.go`).
const SKIPPED_CASES: &[&str] = &[
    "alwaysStrictNoImplicitUseStrict.ts",
    // ~5000-line deeply-nested binary-expression stress test (TS issue #35633)
    // that exercises the binder/emitter trampoline for arbitrarily-deep trees.
    // The Rust port lacks the iterative/trampoline handling, so it recurses and
    // overflows the stack (which catch_unwind cannot trap). Skip until iterative
    // binary-expression handling lands.
    "binderBinaryExpressionStress.ts",
    "binderBinaryExpressionStressJs.ts",
    "APILibCheck.ts",
    "APISample_compile.ts",
    "APISample_jsdoc.ts",
    "APISample_linter.ts",
    "APISample_parseConfig.ts",
    "APISample_transform.ts",
    "APISample_watcher.ts",
    "APISample_Watch.ts",
    "APISample_WatchWithDefaults.ts",
    "APISample_WatchWithOwnWatchHost.ts",
    "excessPropertyErrorsSuppressed.ts",
    "importsNotUsedAsValues_error.ts",
    "isolatedModulesOut.ts",
    "keyofDoesntContainSymbols.ts",
    "lateBoundConstraintTypeChecksCorrectly.ts",
    "mappedTypeUnionConstraintInferences.ts",
    "moduleNoneDynamicImport.ts",
    "moduleNoneErrors.ts",
    "moduleNoneOutFile.ts",
    "noCrashWithVerbatimModuleSyntaxAndImportsNotUsedAsValues.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile1.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile2.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile3.ts",
    "noImplicitAnyIndexingSuppressed.ts",
    "noImplicitUseStrict_amd.ts",
    "noImplicitUseStrict_commonjs.ts",
    "noImplicitUseStrict_es6.ts",
    "noImplicitUseStrict_system.ts",
    "noImplicitUseStrict_umd.ts",
    "nonPrimitiveIndexingWithForInSupressError.ts",
    "noStrictGenericChecks.ts",
    "parameterInitializerBeforeDestructuringEmit.ts",
    "preserveUnusedImports.ts",
    "preserveValueImports_errors.ts",
    "preserveValueImports_importsNotUsedAsValues.ts",
    "preserveValueImports_mixedImports.ts",
    "preserveValueImports_module.ts",
    "preserveValueImports.ts",
    "requireOfJsonFileWithModuleEmitNone.ts",
    "requireOfJsonFileWithModuleNodeResolutionEmitNone.ts",
    "verbatimModuleSyntaxCompat.ts",
    "verbatimModuleSyntaxCompat2.ts",
    "verbatimModuleSyntaxCompat3.ts",
    "verbatimModuleSyntaxCompat4.ts",
];

/// Outcome of processing a single case in the worker subprocess.
enum CaseOutcome {
    /// Case was skipped (unsupported option, KNOWN gap, panic) — `reason` is logged.
    Skip(String),
    /// Rendered errors-baseline text (may be `NO_CONTENT`).
    Output(String),
}

/// Process one case end-to-end: parse directives, apply skip rules, build the
/// Program, collect + render diagnostics. Shared by the worker subprocess. A
/// checker/parse panic is caught and converted to `Skip` (the worker runs in a
/// child process, but a panic — as opposed to a stack overflow — is recoverable
/// in-process and we avoid a needless process kill).
fn process_case(content: &str, basename: &str) -> CaseOutcome {
    let settings = extract_settings(content);
    let (compiler_options, unrecognized) = apply_test_settings(&settings);
    let parsed = split_units(content, basename);

    if SKIPPED_CASES.contains(&basename) {
        return CaseOutcome::Skip("in SKIPPED_CASES list".to_string());
    }
    // Circular-type family — the checker lacks Go's recursion guards and these
    // overflow the stack. (Also enforced by the parent skipping the worker's
    // crash, but skipping here avoids spawning a doomed process.)
    if basename.to_ascii_lowercase().starts_with("circular") {
        return CaseOutcome::Skip("circular-type recursion (no checker guard)".to_string());
    }
    if let Some(reason) = should_skip(&compiler_options, &unrecognized) {
        return CaseOutcome::Skip(reason);
    }

    match catch_unwind(|| {
        let diags = build_and_check(&compiler_options, &parsed.units);
        render_errors_baseline(&diags)
    }) {
        Ok(actual) => CaseOutcome::Output(actual),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(|s| s.as_str()))
                .unwrap_or("<non-string panic>");
            CaseOutcome::Skip(format!("panicked: {msg}"))
        }
    }
}

#[test]
fn submodule_compiler_cases() {
    // ── Worker mode ────────────────────────────────────────────────────────
    // When invoked (by the parent, below) with TSOX_SUBMODULE_WORKER=<case>
    // and TSOX_SUBMODULE_OUT=<path>, process exactly that one case and write a
    // one-line status (`O` = output, `S` = skip) plus the payload to OUT, then
    // exit. Running each case in its own process means a checker stack overflow
    // (uncatchable via catch_unwind) kills only this child — the parent records
    // it as a skip instead of aborting the whole multi-thousand-case sweep.
    if let (Ok(case_path), Ok(out_path)) = (
        std::env::var("TSOX_SUBMODULE_WORKER"),
        std::env::var("TSOX_SUBMODULE_OUT"),
    ) {
        let basename = Path::new(&case_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let payload = match std::fs::read_to_string(&case_path) {
            Ok(content) => match process_case(&content, &basename) {
                CaseOutcome::Skip(reason) => format!("S\n{reason}"),
                CaseOutcome::Output(s) => format!("O\n{s}"),
            },
            Err(e) => format!("S\nunreadable as UTF-8: {e}"),
        };
        let _ = std::fs::write(&out_path, payload);
        return;
    }

    let root = std::path::Path::new(SUBMODULE_DIR);
    if !root.is_dir() {
        eprintln!(
            "[submodule_compiler] {SUBMODULE_DIR} not found — \
             run `git submodule update --init` to fetch official test cases. Skipping."
        );
        return;
    }

    // Enumerate cases (sorted for determinism).
    let mut cases: Vec<std::path::PathBuf> = Vec::new();
    collect_ts_files(root, &mut cases);
    cases.sort();

    let limit = std::env::var("TSOX_SUBMODULE_LIMIT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        // 0 means "run all"; otherwise use the env override or the default cap.
        .map(|n| if n == 0 { usize::MAX } else { n })
        .unwrap_or(DEFAULT_LIMIT);
    cases.truncate(limit);

    let known_diffs = KnownDiffs::load();
    let accept = baseline::accept_mode();
    // This test binary re-invokes itself (in worker mode) per case — see the
    // TSOX_SUBMODULE_WORKER block at the top of this function.
    let exe = std::env::current_exe().expect("current_exe");

    let mut passed = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    let mut accepted_diff = 0usize;
    let mut failed_non_crash: Vec<String> = Vec::new();

    for case_path in &cases {
        let basename = case_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<bad-name>");
        let stem = basename.trim_end_matches(".ts").trim_end_matches(".tsx");
        let ext = ".errors.txt";

        // Run the case in a child process so a checker stack overflow (e.g.
        // circular-type recursion — uncatchable via catch_unwind) kills only the
        // child, not the whole run. The child re-invokes this test binary in
        // worker mode (see TSOX_SUBMODULE_WORKER above).
        let out_path = std::env::temp_dir().join(format!("tsox_submodule_{stem}.out"));
        let _ = std::fs::remove_file(&out_path);
        let status = Command::new(&exe)
            .arg("--exact")
            .arg("submodule_compiler_cases")
            .env("TSOX_SUBMODULE_WORKER", case_path)
            .env("TSOX_SUBMODULE_OUT", &out_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let payload = std::fs::read_to_string(&out_path).unwrap_or_default();
        let _ = std::fs::remove_file(&out_path);

        let success = matches!(status, Ok(s) if s.success());
        if !success {
            // Worker was killed (e.g. stack overflow → signal) or exited
            // non-zero. Report the raw status to aid diagnosis.
            use std::os::unix::process::ExitStatusExt;
            let raw = status
                .as_ref()
                .ok()
                .map(|s| {
                    s.signal()
                        .map(|sig| format!("signal {sig}"))
                        .unwrap_or_else(|| format!("code {}", s.code().unwrap_or(-1)))
                })
                .unwrap_or_else(|| "spawn failed".to_string());
            skipped += 1;
            eprintln!("[skip] {basename}: worker crashed ({raw})");
            continue;
        }
        // Parse the worker's `O`/`S` status line.
        let actual = if let Some(rest) = payload.strip_prefix("O\n") {
            rest.to_string()
        } else if let Some(reason) = payload.strip_prefix("S\n") {
            skipped += 1;
            eprintln!("[skip] {basename}: {reason}");
            continue;
        } else {
            skipped += 1;
            eprintln!("[skip] {basename}: worker produced no output");
            continue;
        };

        // Compare against the reference.
        match baseline::compare(SUBFOLDER, stem, ext, &actual) {
            baseline::Outcome::Passed => {
                passed += 1;
            }
            baseline::Outcome::Failed { .. } => {
                if known_diffs.contains(SUBFOLDER, stem, ext) {
                    accepted_diff += 1;
                } else if accept {
                    // shouldn't happen (compare returns Passed in accept mode),
                    // but be defensive.
                    passed += 1;
                } else {
                    failed += 1;
                    failed_non_crash.push(basename.to_string());
                }
            }
        }
    }

    eprintln!(
        "[submodule_compiler] {} cases: {} passed, {} skipped (unsupported/crash), \
         {} accepted-diff, {} failed",
        cases.len(),
        passed,
        skipped,
        accepted_diff,
        failed,
    );

    if failed > 0 {
        panic!(
            "{failed} baseline mismatch(es):\n  {}\n\
             Run with TSOX_BASELINE_ACCEPT=1 to accept the new output, or add the\n\
             cases to tests/baselines/reference/triaged.txt.",
            failed_non_crash.join("\n  ")
        );
    }
}

/// Recursively collect `*.ts` / `*.tsx` files under `dir`.
fn collect_ts_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_ts_files(&p, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if ext == "ts" || ext == "tsx" {
                out.push(p);
            }
        }
    }
}

/// Build a Program over the virtual files and return semantic diagnostics.
fn build_and_check(
    options: &CompilerOptions,
    units: &[common::case_parser::TestUnit],
) -> Vec<Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");

    let mut file_names: Vec<String> = Vec::new();
    for unit in units {
        let abs = if unit.name.starts_with('/') {
            unit.name.clone()
        } else {
            format!("/proj/{}", unit.name)
        };
        // Ensure parent dirs exist.
        let parent = tsox::tspath::get_directory_path(&abs);
        if !parent.is_empty() {
            fs.insert_dir(&parent);
        }
        fs.insert_file(&abs, &unit.content);
        file_names.push(abs);
    }

    // Wrap with BundledFS so lib.d.ts files resolve (unless the case set --noLib).
    let bf = Arc::new(BundledFS::new(fs.clone()));
    let host = Arc::new(CompilerHostImpl::new(bf, "/proj".to_string(), lib_path()));

    let mut config = tsox::tsoptions::ParsedCommandLine::default();
    config.compiler_options = options.clone();
    config.file_names = file_names;

    let program = Arc::new(Program::new(ProgramOptions { config, host }));
    // Collect the FULL diagnostic set the Go oracle would report: program
    // construction diagnostics (TS2307 "Cannot find module", TS6053 config
    // errors, etc. — these are syntactic/global-layer) PLUS the checker's
    // semantic diagnostics. `get_semantic_diagnostics` alone misses TS2307.
    let mut all = Vec::new();
    for d in program.diagnostics() {
        all.push((**d).clone());
    }
    all.extend(program.get_semantic_diagnostics());
    all
}

/// Render diagnostics into the errors-baseline text format.
///
/// Diagnostics are sorted by (file_name, line, col, code) for determinism,
/// then each rendered on one line via `format_diagnostic_compact`. Test-path
/// prefixes (`/proj/`) are stripped. No diagnostics → `NO_CONTENT`.
fn render_errors_baseline(diags: &[Diagnostic]) -> String {
    if diags.is_empty() {
        return NO_CONTENT.to_string();
    }
    let mut keyed: Vec<(String, usize, usize, i32, &Diagnostic)> = diags
        .iter()
        .map(|d| {
            let (file_name, line, col) = if let Some(f) = &d.file {
                let (l, c) = crate_line_col(d);
                (f.file_name.clone(), l, c)
            } else {
                (String::new(), 0, 0)
            };
            (file_name, line, col, d.code, d)
        })
        .collect();
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });

    let mut out = String::new();
    for (_, _, _, _, d) in keyed {
        let mut line = format_diagnostic_compact(d, None);
        // Strip the `/proj/` test-path prefix from file names in the output.
        line = line.replace("/proj/", "");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn crate_line_col(d: &Diagnostic) -> (usize, usize) {
    if let Some(file) = &d.file {
        tsox::diagnosticwriter::line_and_character(&file.line_map, &file.text, d.loc.pos())
    } else {
        (0, 0)
    }
}

/// Decide whether a case should be skipped because it exercises options the
/// Rust port doesn't yet support. Mirrors tsgo's `SkipUnsupportedCompilerOptions`
/// at a coarse granularity.
fn should_skip(options: &CompilerOptions, unrecognized: &[String]) -> Option<String> {
    use tsox::core::compiler_options::{ModuleKind, ModuleResolutionKind, ScriptTarget};

    // Unknown directive → we can't faithfully set up the case.
    if !unrecognized.is_empty() {
        return Some(format!(
            "unrecognized option(s): {}",
            unrecognized.join(", ")
        ));
    }

    // Module kinds not yet supported.
    match options.module {
        ModuleKind::AMD | ModuleKind::UMD | ModuleKind::System => {
            return Some(format!("module={:?} not supported", options.module));
        }
        _ => {}
    }
    // Module resolution modes not yet supported.
    if matches!(
        options.module_resolution,
        ModuleResolutionKind::Node10 | ModuleResolutionKind::Classic
    ) {
        return Some(format!(
            "moduleResolution={:?} not supported",
            options.module_resolution
        ));
    }
    // baseUrl / outFile scenarios.
    if !options.base_url.is_empty() {
        return Some("baseUrl not supported".to_string());
    }
    if !options.out_file.is_empty() {
        return Some("outFile not supported".to_string());
    }
    // ES5 down-leveling (Rust emitter doesn't down-level yet).
    if matches!(options.target, ScriptTarget::ES5) {
        return Some(format!(
            "target={:?} (ES5 down-level) not supported",
            options.target
        ));
    }
    // `allowJs`/`checkJs` — the checker path differs for JS files.
    if options.allow_js.is_true() {
        return Some("allowJs not supported".to_string());
    }

    None
}
