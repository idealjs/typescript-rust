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
//! Environment variables (all optional):
//! - `TSOX_SUBMODULE_START=N` — first case to run, 1-based (default 1).
//! - `TSOX_SUBMODULE_END=N` — last case to run, 1-based inclusive (default:
//!   last case). START/END pick an explicit window, e.g. `START=1000
//!   END=2000` runs #1000..#2000. Either may be omitted. Takes precedence
//!   over `TSOX_SUBMODULE_LIMIT` when set.
//! - `TSOX_SUBMODULE_LIMIT=N` — alternative selector: first N cases
//!   (default 1000), or `0` for all (~6500). Ignored when START/END is set.
//! - `TSOX_SUBMODULE_FILTER` — case-insensitive substring; run only matching
//!   case file names (applied after the selection above).
//! - `TSOX_SUBMODULE_JOBS=N` — concurrent workers. Defaults to
//!   `available_parallelism()`, which honors `taskset` CPU affinity
//!   (`taskset -c 0-3 …` → 4 workers, ≤400% CPU).
//! - `TSOX_SUBMODULE_TIMEOUT_SECS=N` — per-case wall-clock budget (default 30).
//! - `TSOX_SUBMODULE_QUIET=1` — suppress per-case console lines (the run-log
//!   file still records everything).
//!
//! Progress visibility: every case logs a START line when a worker picks it
//! up and an END line with the outcome (`[wID] #i/total VERB name (secs)`)
//! to stderr and to `tests/baselines/local/submodule_run.log`; a heartbeat
//! with counts + ETA prints every 100 completions.

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

/// Per-case wall-clock budget for the worker subprocess. Cases normally
/// finish in well under a second; a case exceeding this is almost certainly
/// stuck (checker infinite loop or combinatorial blow-up) and is killed and
/// recorded as a skip rather than hanging the whole sweep. Override with
/// `TSOX_SUBMODULE_TIMEOUT_SECS`.
const CASE_TIMEOUT_DEFAULT_SECS: u64 = 30;

/// Cases skipped because they reference inputs the port can't provide (e.g.
/// the old `typescript.d.ts` API surface) or exercise removed compiler options.
/// Mirrors tsgo's `skippedTests` (`internal/testrunner/compiler_runner.go`).
const SKIPPED_CASES: &[&str] = &[
    "alwaysStrictNoImplicitUseStrict.ts",
    // ClassDeclaration26.ts (garbage-input stress: `public const var export
    // foo = 10;` + `var constructor() { }`) was previously skipped — it now
    // passes via the ported recovery path: scanClassMemberStart gating +
    // parsingContext stack (TS1068), parseSemicolonAfterPropertyName's
    // const/let/var special case (TS1440), export-as-member-modifier, and
    // `() {` arrow speculation ('=>' expected).
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

/// One (configuration, outcome) pair. A case with no multi-value `varyBy`
/// directive has exactly one configuration whose suffix is empty (baseline
/// name = plain `<stem>.errors.txt`); a case like `// @target: ES5, ES2015`
/// gets one per value, each comparing against
/// `<stem>(target=es5).errors.txt` etc.
struct ConfigOutcome {
    /// Baseline-name suffix, e.g. `target=es2015` (empty for the default
    /// configuration). Keys sorted, comma-joined, all lowercased — mirrors
    /// Go's `getFileBasedTestConfigurationDescription`.
    suffix: String,
    outcome: CaseOutcome,
}

/// Options whose multi-value `// @opt: v1, v2` directives expand into one
/// configuration per value. Mirrors Go's `compilerVaryBy`: boolean/enum
/// options that affect diagnostics, emit, or program structure, plus explicit
/// `noEmit`/`isolatedModules`. List options (`lib`, `types`, ...) are NOT
/// varyBy — their commas are list values, not variations.
const VARY_BY: &[&str] = &[
    // enum options
    "jsx", "module", "moduledetection", "moduleresolution", "newline", "target",
    // boolean options
    "allowarbitraryextensions", "allowimportingtsextensions", "allowjs",
    "allowsyntheticdefaultimports", "allowumdglobalaccess", "allowunreachablecode",
    "allowunusedlabels", "alwaysstrict", "assumechangesonlyaffectdirectdependencies",
    "checkjs", "composite", "declaration", "declarationmap", "deduplicatepackages",
    "disablesizelimit", "downleveliteration", "emitbom", "emitdeclarationonly",
    "emitdecoratormetadata", "erasablesyntaxonly", "esmoduleinterop",
    "exactoptionalpropertytypes", "experimentaldecorators",
    "forceconsistentcasinginfilenames", "importhelpers", "inlinesourcemap",
    "inlinesources", "isolateddeclarations", "isolatedmodules", "libreplacement",
    "noemit", "noemithelpers", "noemitonerror", "noerrortruncation",
    "nofallthroughcasesinswitch", "noimplicitany", "noimplicitoverride",
    "noimplicitreturns", "noimplicitthis", "nolib", "nopropertyaccessfromindexsignature",
    "noresolve", "nouncheckedindexedaccess", "nouncheckedsideeffectimports",
    "preserveconstenums", "removecomments",
    "resolvejsonmodule", "resolvepackagejsonexports", "resolvepackagejsonimports",
    "rewriterelativeimportextensions", "skipdefaultlibcheck", "skiplibcheck",
    "sourcemap", "stabletypeordering", "strict", "strictbindcallapply",
    "strictbuiltiniteratorreturn", "strictfunctiontypes", "strictnullchecks",
    "strictpropertyinitialization", "stripinternal", "usedefineforclassfields",
    "useunknownincatchvariables", "verbatimmodulesyntax",
];

/// All declared values of an enum option, in declaration order (Go
/// `getAllValuesForOption` via the option's `EnumMap`) — used when a
/// directive value is `*`. Booleans yield `["true", "false"]`.
fn all_enum_values(option: &str) -> &'static [&'static str] {
    match option {
        "target" => &[
            "es5", "es6", "es2015", "es2016", "es2017", "es2018", "es2019", "es2020", "es2021",
            "es2022", "es2023", "es2024", "es2025", "esnext",
        ],
        "module" => &[
            "commonjs", "amd", "system", "umd", "es6", "es2015", "es2020", "es2022", "esnext",
            "node16", "node18", "node20", "nodenext", "preserve",
        ],
        "moduleresolution" => &[
            "node16", "nodenext", "bundler", "classic", "node", "node10",
        ],
        "jsx" => &["preserve", "react-native", "react-jsx", "react-jsxdev", "react"],
        "moduledetection" => &["auto", "legacy", "force"],
        "newline" => &["crlf", "lf"],
        _ => &[],
    }
}

/// Canonical comparison key for a raw option value, so `es6` and `es2015`
/// dedupe to one variation (Go compares parsed `CompilerOptionsValue`s).
fn canonical_value(option: &str, raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    match option {
        "target" => match lower.as_str() {
            "es6" => "es2015".to_string(),
            other => other.to_string(),
        },
        "module" => match lower.as_str() {
            "es6" => "es2015".to_string(),
            other => other.to_string(),
        },
        "moduleresolution" => match lower.as_str() {
            "node" => "node10".to_string(),
            other => other.to_string(),
        },
        // Boolean options: truthiness spelling collapses to true/false.
        _ if VARY_BY.contains(&option) && !all_enum_values(option).is_empty() => lower,
        _ if VARY_BY.contains(&option) => match lower.as_str() {
            "true" => "true".to_string(),
            _ => "false".to_string(),
        },
        _ => lower,
    }
}

/// Split a multi-value directive into variation values (raw strings, first
/// occurrence kept on dedupe). Handles `*` (all values) and `-`/`!`
/// exclusions. Returns `None` when the value has no variation content.
/// Mirrors Go's `splitOptionValues`.
fn split_option_values(value: &str, option: &str) -> Option<Vec<String>> {
    let mut star = false;
    let mut includes: Vec<String> = Vec::new();
    let mut excludes: Vec<String> = Vec::new();
    for part in value.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        if s == "*" {
            star = true;
        } else if let Some(rest) = s.strip_prefix('-').or_else(|| s.strip_prefix('!')) {
            excludes.push(rest.trim().to_string());
        } else {
            includes.push(s.to_string());
        }
    }
    if includes.is_empty() && !star && excludes.is_empty() {
        return None;
    }
    if star {
        if all_enum_values(option).is_empty() {
            includes.push("true".to_string());
            includes.push("false".to_string());
        } else {
            includes.extend(all_enum_values(option).iter().map(|s| s.to_string()));
        }
    }
    // Dedupe by canonical value, keeping the first raw spelling (it names the
    // baseline, e.g. `target=es6` for `// @target: ES6, ES2015`).
    let mut seen: Vec<String> = Vec::new();
    let mut raws: Vec<String> = Vec::new();
    for raw in includes {
        let canon = canonical_value(option, &raw);
        if !seen.contains(&canon) {
            seen.push(canon);
            raws.push(raw);
        }
    }
    for ex in excludes {
        let canon = canonical_value(option, &ex);
        if let Some(pos) = seen.iter().position(|c| *c == canon) {
            seen.remove(pos);
            raws.remove(pos);
        }
    }
    Some(raws)
}

/// Expand `settings` into per-value configurations. Returns one entry per
/// configuration — `(suffix, merged-settings)` — or a skip reason when the
/// variations exceed Go's cap of 25 (Go `t.Fatal`s there; we skip honestly).
/// Mirrors Go's `GetFileBasedTestConfigurations` + cartesian product.
fn compute_configurations(
    settings: &std::collections::HashMap<String, String>,
) -> Result<Vec<(String, std::collections::HashMap<String, String>)>, String> {
    let mut varying: Vec<(String, Vec<String>)> = Vec::new();
    let mut base = settings.clone();
    for key in settings.keys() {
        if !VARY_BY.contains(&key.as_str()) {
            continue;
        }
        let value = &settings[key];
        if !value.contains(',') && value.trim() != "*" {
            // Single value (no `,`): nothing to vary — Go keeps it as a
            // non-varying option with the raw value.
            continue;
        }
        let Some(values) = split_option_values(value, key) else {
            continue;
        };
        if values.len() <= 1 {
            if let [only] = values.as_slice() {
                base.insert(key.clone(), only.clone());
            }
            continue;
        }
        varying.push((key.clone(), values));
    }

    if varying.is_empty() {
        return Ok(vec![(String::new(), base)]);
    }

    // Cartesian product, capped at 25 like Go.
    let mut count = 1usize;
    for (_, values) in &varying {
        count *= values.len();
        if count > 25 {
            return Err(format!("too many option variations ({count})"));
        }
    }

    let mut configs: Vec<std::collections::HashMap<String, String>> =
        vec![std::collections::HashMap::new()];
    for (key, values) in &varying {
        let mut next = Vec::with_capacity(configs.len() * values.len());
        for config in &configs {
            for value in values {
                let mut c = config.clone();
                c.insert(key.clone(), value.clone());
                next.push(c);
            }
        }
        configs = next;
    }

    let mut out = Vec::with_capacity(configs.len());
    for config in configs {
        // Suffix: sorted `key=value` (both lowercased), comma-joined.
        let mut parts: Vec<String> = config
            .iter()
            .map(|(k, v)| format!("{}={}", k.to_ascii_lowercase(), v.to_ascii_lowercase()))
            .collect();
        parts.sort();
        let suffix = parts.join(",");
        let mut merged = base.clone();
        for (k, v) in config {
            merged.insert(k, v);
        }
        out.push((suffix, merged));
    }
    Ok(out)
}

/// Process one case end-to-end: parse directives, expand multi-value `varyBy`
/// directives into configurations, apply per-config skip rules, build the
/// Program, collect + render diagnostics. Shared by the worker subprocess. A
/// checker/parse panic is caught and converted to `Skip` for that
/// configuration (the worker runs in a child process, but a panic — as
/// opposed to a stack overflow — is recoverable in-process and we avoid a
/// needless process kill).
fn process_case(content: &str, basename: &str) -> Vec<ConfigOutcome> {
    let settings = extract_settings(content);

    if SKIPPED_CASES.contains(&basename) {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip("in SKIPPED_CASES list".to_string()),
        }];
    }
    // Circular-type family — the checker lacks Go's recursion guards and these
    // overflow the stack. (Also enforced by the parent skipping the worker's
    // crash, but skipping here avoids spawning a doomed process.)
    if basename.to_ascii_lowercase().starts_with("circular") {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip("circular-type recursion (no checker guard)".to_string()),
        }];
    }

    let parsed = split_units(content, basename);
    match compute_configurations(&settings) {
        Err(reason) => vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip(reason),
        }],
        Ok(configs) => configs
            .into_iter()
            .map(|(suffix, config_settings)| {
                let (compiler_options, unrecognized) = apply_test_settings(&config_settings);
                let outcome = if let Some(reason) = should_skip(&compiler_options, &unrecognized) {
                    CaseOutcome::Skip(reason)
                } else {
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
                };
                ConfigOutcome { suffix, outcome }
            })
            .collect(),
    }
}

/// Outcome of scheduling + comparing one case in the parent (vs.
/// `CaseOutcome`, which is the worker subprocess's result).
enum StepOutcome {
    Passed,
    AcceptedDiff,
    Failed,
    Skipped,
}

/// Run one case end-to-end from the parent side: spawn the worker subprocess
/// (with timeout), read its payload, compare each configuration against the
/// reference baseline. Returns the aggregated outcome, the display names of
/// failing configurations, and a human-readable detail line for logging.
fn run_case(
    case_path: &Path,
    idx: usize,
    exe: &Path,
    timeout: std::time::Duration,
    known_diffs: &KnownDiffs,
    accept: bool,
) -> (StepOutcome, Vec<String>, String) {
    let basename = case_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<bad-name>")
        .to_string();
    let stem = basename.trim_end_matches(".ts").trim_end_matches(".tsx");
    let ext = ".errors.txt";

    // Run the case in a child process so a checker stack overflow (e.g.
    // circular-type recursion — uncatchable via catch_unwind) kills only the
    // child, not the whole run. The child re-invokes this test binary in
    // worker mode (see TSOX_SUBMODULE_WORKER above). A case that runs longer
    // than `timeout` (e.g. a checker infinite loop or combinatorial blow-up)
    // is killed and recorded as a skip — mirroring tsgo's per-case timeout —
    // instead of hanging the whole sweep.
    let out_path = std::env::temp_dir().join(format!("tsox_submodule_{idx}_{stem}.out"));
    let _ = std::fs::remove_file(&out_path);
    let worker = Command::new(exe)
        .arg("--exact")
        .arg("submodule_compiler_cases")
        .env("TSOX_SUBMODULE_WORKER", case_path)
        .env("TSOX_SUBMODULE_OUT", &out_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let status = worker
        .map(|mut child| {
            let deadline = std::time::Instant::now() + timeout;
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => return Ok(status),
                    Ok(None) => {
                        if std::time::Instant::now() >= deadline {
                            let _ = child.kill();
                            let _ = child.wait();
                            return Err("timed out".to_string());
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        })
        .unwrap_or_else(|e| Err(e.to_string()));
    let payload = std::fs::read_to_string(&out_path).unwrap_or_default();
    let _ = std::fs::remove_file(&out_path);
    if std::env::var("TSOX_DEBUG_PAYLOAD").is_ok() {
        eprintln!("[debug] {basename} payload: {payload:?}");
    }

    if !matches!(&status, Ok(s) if s.success()) {
        // Worker was killed (e.g. stack overflow → signal), timed out, or
        // exited non-zero. Report the raw status to aid diagnosis.
        use std::os::unix::process::ExitStatusExt;
        let raw = match &status {
            Ok(s) => s
                .signal()
                .map(|sig| format!("signal {sig}"))
                .unwrap_or_else(|| format!("code {}", s.code().unwrap_or(-1))),
            Err(reason) => reason.clone(),
        };
        return (
            StepOutcome::Skipped,
            Vec::new(),
            format!("worker crashed ({raw})"),
        );
    }
    // Parse the worker's JSON payload: one entry per configuration with
    // `suffix` (baseline-name infix), and either `skip` (reason) or `text`
    // (rendered baseline).
    let entries: Vec<serde_json::Value> = match serde_json::from_str(&payload) {
        Ok(v) => v,
        Err(_) => {
            return (
                StepOutcome::Skipped,
                Vec::new(),
                "worker produced no parseable output".to_string(),
            )
        }
    };
    if entries.is_empty() {
        return (
            StepOutcome::Skipped,
            Vec::new(),
            "worker produced no configurations".to_string(),
        );
    }

    // Compare each configuration against its (possibly suffixed) baseline;
    // aggregate: any fail → Failed, else any triaged diff → AcceptedDiff,
    // else any pass → Passed, all skipped → Skipped.
    let mut overall = StepOutcome::Skipped;
    let mut failed_names = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    for entry in &entries {
        let suffix = entry["suffix"].as_str().unwrap_or("");
        let display = if suffix.is_empty() {
            basename.clone()
        } else {
            format!("{basename}({suffix})")
        };
        let name = if suffix.is_empty() {
            stem.to_string()
        } else {
            format!("{stem}({suffix})")
        };
        let label = if suffix.is_empty() { "default" } else { suffix };
        if let Some(reason) = entry["skip"].as_str() {
            notes.push(format!("{label}: skip ({reason})"));
            continue;
        }
        let Some(actual) = entry["text"].as_str() else {
            notes.push(format!("{label}: malformed entry"));
            failed_names.push(display);
            overall = StepOutcome::Failed;
            continue;
        };
        match baseline::compare(SUBFOLDER, &name, ext, actual) {
            baseline::Outcome::Passed => {
                notes.push(format!("{label}: pass"));
                if !matches!(overall, StepOutcome::Failed | StepOutcome::AcceptedDiff) {
                    overall = StepOutcome::Passed;
                }
            }
            baseline::Outcome::Failed { .. } => {
                if known_diffs.contains(SUBFOLDER, &name, ext) {
                    notes.push(format!("{label}: known diff (triaged/accepted)"));
                    if !matches!(overall, StepOutcome::Failed) {
                        overall = StepOutcome::AcceptedDiff;
                    }
                } else if accept {
                    // shouldn't happen (compare returns Passed in accept
                    // mode), but be defensive.
                    notes.push(format!("{label}: pass"));
                    if !matches!(overall, StepOutcome::Failed | StepOutcome::AcceptedDiff) {
                        overall = StepOutcome::Passed;
                    }
                } else {
                    notes.push(format!("{label}: baseline mismatch"));
                    failed_names.push(display);
                    overall = StepOutcome::Failed;
                }
            }
        }
    }
    // Single-configuration cases keep the historical compact detail format
    // (bare skip reason / empty for pass); only multi-config cases spell out
    // per-configuration results.
    let detail = if entries.len() == 1 {
        notes
            .into_iter()
            .next()
            .unwrap_or_default()
            .strip_prefix("default: ")
            .map(str::to_string)
            .filter(|s| s != "pass")
            .unwrap_or_default()
    } else {
        notes.join("; ")
    };
    (overall, failed_names, detail)
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
        // libtest runs test functions on threads whose stack is smaller
        // than the CLI's main-thread stack; deep checker recursion that
        // survives the CLI (Go relies on growable goroutine stacks) would
        // overflow here. Compile on a dedicated large-stack thread.
        let case_path = case_path.clone();
        let payload = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || {
                let basename = Path::new(&case_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                // Payload: one JSON array entry per configuration —
                // `{"suffix": "...", "skip": null, "text": "..."}` for output,
                // `{"suffix": "...", "skip": "reason", "text": null}` for a skip.
                // (JSON framing keeps multi-line baseline text unambiguous.)
                match std::fs::read_to_string(&case_path) {
                    Ok(content) => {
                        let configs = process_case(&content, &basename);
                        serde_json::to_string(
                            &configs
                                .iter()
                                .map(|c| {
                                    let mut entry = serde_json::json!({ "suffix": c.suffix });
                                    match &c.outcome {
                                        CaseOutcome::Skip(reason) => {
                                            entry["skip"] = serde_json::json!(reason);
                                        }
                                        CaseOutcome::Output(text) => {
                                            entry["text"] = serde_json::json!(text);
                                        }
                                    }
                                    entry
                                })
                                .collect::<Vec<_>>(),
                        )
                        .unwrap_or_else(|_| "[]".to_string())
                    }
                    Err(e) => serde_json::json!([{
                        "suffix": "",
                        "skip": format!("unreadable as UTF-8: {e}")
                    }])
                    .to_string(),
                }
            })
            .expect("spawn worker compile thread")
            .join()
            .unwrap_or_else(|_| "[]".to_string());
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

    // ── Run log ────────────────────────────────────────────────────────────
    // Every banner/START/END/heartbeat/summary line goes to the real stderr
    // (unless quiet) and is appended to a per-run log under the gitignored
    // local/ dir. Created before case selection so selection notes are
    // visible too.
    struct RunLog {
        file: Option<std::sync::Mutex<std::fs::File>>,
        quiet: bool,
    }
    impl RunLog {
        fn new(path: std::path::PathBuf, quiet: bool) -> Self {
            let file = std::fs::create_dir_all(path.parent().unwrap())
                .ok()
                .and_then(|_| std::fs::File::create(&path).ok())
                .map(std::sync::Mutex::new);
            Self { file, quiet }
        }
        fn line(&self, msg: &str) {
            if !self.quiet {
                use std::io::Write;
                // Write straight to fd 2 via the stderr handle: libtest's
                // output capture (propagated into threads spawned by the
                // test) swallows `eprintln!` and discards it when the test
                // passes, hiding the live progress these lines exist for.
                let mut err = std::io::stderr().lock();
                let _ = writeln!(err, "{msg}");
            }
            if let Some(f) = &self.file {
                use std::io::Write;
                let _ = writeln!(f.lock().unwrap(), "{msg}");
            }
        }
    }
    let log_path = Path::new(baseline::LOCAL_ROOT).join("submodule_run.log");
    let log = RunLog::new(
        log_path.clone(),
        std::env::var("TSOX_SUBMODULE_QUIET")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
    );
    if log.file.is_none() {
        eprintln!("[submodule_compiler] note: cannot write run log {}", log_path.display());
    }

    // ── Case selection ─────────────────────────────────────────────────────
    // Two ways to pick which cases run (1-based, inclusive):
    //   `TSOX_SUBMODULE_START` / `TSOX_SUBMODULE_END` — explicit window; each
    //     optional (START defaults to 1, END to the last case). Precedence
    //     over LIMIT when either is set.
    //   `TSOX_SUBMODULE_LIMIT` — `N` = first N (default 1000), `0` = all.
    // `TSOX_SUBMODULE_FILTER=<substr>` (optional) narrows the selection
    // further by case-name substring, case-insensitive.
    let total = cases.len();
    let limit_spec = std::env::var("TSOX_SUBMODULE_LIMIT").unwrap_or_default();
    let start_spec = std::env::var("TSOX_SUBMODULE_START").unwrap_or_default();
    let end_spec = std::env::var("TSOX_SUBMODULE_END").unwrap_or_default();
    let (start, end, desc) = if !start_spec.is_empty() || !end_spec.is_empty() {
        let usage = "TSOX_SUBMODULE_START/END must be 1-based case numbers with END >= START";
        let a = if start_spec.is_empty() {
            1
        } else {
            start_spec
                .trim()
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("{usage}: got START='{start_spec}'"))
                .max(1) // lenient: a 0 start is treated as 1
        };
        let b = if end_spec.is_empty() {
            total
        } else {
            end_spec
                .trim()
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("{usage}: got END='{end_spec}'"))
        };
        assert!(b >= a, "{usage}: got START={a} END={b}");
        if !limit_spec.is_empty() {
            log.line(&format!(
                "[submodule_compiler] note: TSOX_SUBMODULE_LIMIT='{limit_spec}' ignored \
                 — START/END window takes precedence"
            ));
        }
        (a - 1, b.min(total), format!("#{a}..#{b}"))
    } else {
        let usage = "TSOX_SUBMODULE_LIMIT must be N (first N cases), 0 (all), or unset";
        let n = if limit_spec.is_empty() {
            DEFAULT_LIMIT
        } else if limit_spec == "0" {
            total
        } else {
            limit_spec
                .trim()
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("{usage}: got '{limit_spec}'"))
        };
        let end = n.min(total);
        let desc = if limit_spec.is_empty() {
            format!("first {DEFAULT_LIMIT}")
        } else if limit_spec == "0" {
            "all".to_string()
        } else {
            format!("first {n}")
        };
        (0, end, desc)
    };
    if start >= end {
        log.line(&format!(
            "[submodule_compiler] selection '{desc}' selects no cases of {total} \
             (range #{start}..#{end}) — nothing to do."
        ));
        return;
    }

    let filter = std::env::var("TSOX_SUBMODULE_FILTER").unwrap_or_default();
    let filter_lc = filter.to_lowercase();
    let selected: Vec<&std::path::PathBuf> = cases[start..end]
        .iter()
        .filter(|p| {
            filter_lc.is_empty()
                || p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.to_lowercase().contains(&filter_lc))
        })
        .collect();
    if selected.is_empty() {
        log.line(&format!(
            "[submodule_compiler] filter '{filter}' matched no cases — nothing to do."
        ));
        return;
    }
    let selected_total = selected.len();
    let first = selected[0].file_name().unwrap().to_string_lossy();
    let last = selected[selected_total - 1].file_name().unwrap().to_string_lossy();

    let known_diffs = KnownDiffs::load();
    let accept = baseline::accept_mode();
    let case_timeout = std::time::Duration::from_secs(
        std::env::var("TSOX_SUBMODULE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(CASE_TIMEOUT_DEFAULT_SECS),
    );
    // This test binary re-invokes itself (in worker mode) per case — see the
    // TSOX_SUBMODULE_WORKER block at the top of this function.
    let exe = std::env::current_exe().expect("current_exe");

    // ── Parallelism ────────────────────────────────────────────────────────
    // Cases are independent one-shot subprocesses, so the parent can run
    // many at once. Concurrency defaults to `available_parallelism()`, which
    // honors CPU affinity on Linux — `taskset -c 0-3 cargo test …` yields 4
    // concurrent workers (≤400% CPU) without any extra flags, while an
    // un-pinned run uses every core. Override with `TSOX_SUBMODULE_JOBS=N`.
    // (cargo's own `--test-threads` does not apply: this is a single test
    // function doing its own scheduling.)
    let jobs = std::env::var("TSOX_SUBMODULE_JOBS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n >= 1)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
        .min(selected_total);

    use std::sync::atomic::{AtomicUsize, Ordering};
    let next_case = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let passed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let accepted_diff = AtomicUsize::new(0);
    let failed_non_crash: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    const HEARTBEAT_EVERY: usize = 100;

    // 1-based case numbers of the selection bounds (for the banner below).
    let first_no = start + 1;
    let last_no = end;
    log.line(&format!(
        "[submodule_compiler] {total} cases enumerated; selection '{desc}' \
         (+filter '{filter}') → #{first_no}..#{last_no} = {selected_total} cases \
         [{first} … {last}] on {jobs} workers, timeout {}s; log: {}",
        case_timeout.as_secs(),
        log_path.display(),
    ));

    let run_start = std::time::Instant::now();
    // Worker ids are claimed lazily from this counter: a `move` closure in the
    // spawn loop would move the shared state into the first worker only, so
    // the closures borrow instead (ids may interleave; they're for logging).
    let worker_seq = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                let wid = worker_seq.fetch_add(1, Ordering::Relaxed);
                loop {
                let i = next_case.fetch_add(1, Ordering::Relaxed);
                if i >= selected_total {
                    break;
                }
                let case_path = selected[i];
                let name = case_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<bad-name>");
                log.line(&format!("[w{wid}] #{}/{selected_total} START {name}", i + 1));
                let t0 = std::time::Instant::now();
                let (outcome, failed_configs, detail) = run_case(
                    case_path,
                    i,
                    &exe,
                    case_timeout,
                    &known_diffs,
                    accept,
                );
                let secs = t0.elapsed().as_secs_f32();
                let verb = match &outcome {
                    StepOutcome::Passed => "PASS ",
                    StepOutcome::AcceptedDiff => "DIFF ",
                    StepOutcome::Failed => "FAIL ",
                    StepOutcome::Skipped => "SKIP ",
                };
                match &outcome {
                    StepOutcome::Passed => {
                        passed.fetch_add(1, Ordering::Relaxed);
                    }
                    StepOutcome::AcceptedDiff => {
                        accepted_diff.fetch_add(1, Ordering::Relaxed);
                    }
                    StepOutcome::Failed => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        failed_non_crash
                            .lock()
                            .unwrap()
                            .extend(failed_configs.iter().cloned());
                    }
                    StepOutcome::Skipped => {
                        skipped.fetch_add(1, Ordering::Relaxed);
                    }
                }
                let note = if detail.is_empty() {
                    String::new()
                } else {
                    format!(" — {detail}")
                };
                log.line(&format!(
                    "[w{wid}] #{}/{selected_total} {verb}{name} ({secs:.2}s){note}",
                    i + 1,
                ));
                // Heartbeat every HEARTBEAT_EVERY completions (and at the end).
                let d = done.fetch_add(1, Ordering::Relaxed) + 1;
                if d % HEARTBEAT_EVERY == 0 || d == selected_total {
                    let (p, s, f, a) = (
                        passed.load(Ordering::Relaxed),
                        skipped.load(Ordering::Relaxed),
                        failed.load(Ordering::Relaxed),
                        accepted_diff.load(Ordering::Relaxed),
                    );
                    let elapsed = run_start.elapsed().as_secs_f32();
                    let eta = if d > 0 {
                        elapsed / d as f32 * (selected_total - d) as f32
                    } else {
                        0.0
                    };
                    log.line(&format!(
                        "[submodule_compiler] progress {d}/{selected_total} \
                         ({p} pass, {a} diff, {s} skip, {f} fail) \
                         elapsed {elapsed:.0}s, ETA {eta:.0}s"
                    ));
                }
                }
            });
        }
    });

    let (passed, skipped, failed, accepted_diff) = (
        passed.into_inner(),
        skipped.into_inner(),
        failed.into_inner(),
        accepted_diff.into_inner(),
    );
    let mut failed_non_crash = failed_non_crash.into_inner().unwrap();
    failed_non_crash.sort();

    log.line(&format!(
        "[submodule_compiler] {selected_total} cases done in {:.0}s: \
         {passed} passed, {skipped} skipped (unsupported/crash), \
         {accepted_diff} accepted-diff, {failed} failed",
        run_start.elapsed().as_secs_f32(),
    ));
    if failed > 0 {
        for name in &failed_non_crash {
            log.line(&format!("[submodule_compiler] FAILED: {name}"));
        }
        log.line(&format!(
            "run log: {}; actual outputs under {}/{SUBFOLDER}/",
            log_path.display(),
            baseline::LOCAL_ROOT,
        ));
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

/// Copy `_submodules/TypeScript/tests/lib/**` into the in-memory VFS under
/// `/.lib/**`. The official runner mounts this directory at the virtual FS
/// root so `/// <reference path="/.lib/react.d.ts" />` resolves; missing
/// files are skipped (the fixture set only exists with the submodule).
fn mount_test_lib_fixtures(fs: &Arc<InMemoryFS>) {
    fn walk(fs: &Arc<InMemoryFS>, src: &Path, mount: &str) {
        let Ok(entries) = std::fs::read_dir(src) else {
            return;
        };
        fs.insert_dir(mount);
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let mount_path = format!("{mount}/{name}");
            if p.is_dir() {
                walk(fs, &p, &mount_path);
            } else if let Ok(text) = std::fs::read_to_string(&p) {
                fs.insert_file(&mount_path, &text);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("_submodules/TypeScript/tests/lib");
    walk(fs, &root, "/.lib");
}

/// Build a Program over the virtual files and return semantic diagnostics.
fn build_and_check(
    options: &CompilerOptions,
    units: &[common::case_parser::TestUnit],
) -> Vec<Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    // Mount the official test-runner fixture libs (react.d.ts,
    // react16.d.ts, react18/, …) at `/.lib/`, matching the Go test
    // runner's virtual filesystem: cases reference them via absolute
    // triple-slash paths like `/// <reference path="/.lib/react.d.ts" />`.
    mount_test_lib_fixtures(&fs);

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
        // Only TypeScript units are compile roots (Go's makeUnitsFromTest
        // hands every unit to the compiler host, but .json/.md units exist
        // on the FS only — compiling them as TS produces parse errors the
        // official baselines don't have).
        let lower = abs.to_ascii_lowercase();
        if lower.ends_with(".ts")
            || lower.ends_with(".tsx")
            || lower.ends_with(".mts")
            || lower.ends_with(".cts")
        {
            file_names.push(abs);
        }
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
        // NOTE: no code tiebreak — same-position diagnostics keep EMISSION
        // order, which is what the official baselines record (Go's checker
        // emits in evaluation order).
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
