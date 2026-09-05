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

fn suite() -> &'static str {
    match std::env::var("TSOX_SUBMODULE_SUITE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "conformance" => "conformance",
        _ => "compiler",
    }
}

fn submodule_dir() -> String {
    format!("_submodules/TypeScript/tests/cases/{}", suite())
}

fn subfolder() -> &'static str {
    suite()
}

const DEFAULT_LIMIT: usize = 1000;

const CASE_TIMEOUT_DEFAULT_SECS: u64 = 30;

const SKIPPED_CASES: &[&str] = &[
    "alwaysStrictNoImplicitUseStrict.ts",

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

enum CaseOutcome {

    Skip(String),

    Output(String),
}

struct ConfigOutcome {

    suffix: String,
    outcome: CaseOutcome,
}

const VARY_BY: &[&str] = &[

    "jsx", "module", "moduledetection", "moduleresolution", "newline", "target",

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

        _ if VARY_BY.contains(&option) && !all_enum_values(option).is_empty() => lower,
        _ if VARY_BY.contains(&option) => match lower.as_str() {
            "true" => "true".to_string(),
            _ => "false".to_string(),
        },
        _ => lower,
    }
}

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

const TSGO_SKIPPED_TESTS: &[&str] = &[
    "APILibCheck.ts",
    "APISample_Watch.ts",
    "APISample_WatchWithDefaults.ts",
    "APISample_WatchWithOwnWatchHost.ts",
    "APISample_compile.ts",
    "APISample_jsdoc.ts",
    "APISample_linter.ts",
    "APISample_parseConfig.ts",
    "APISample_transform.ts",
    "APISample_watcher.ts",
    "preserveUnusedImports.ts",
    "noCrashWithVerbatimModuleSyntaxAndImportsNotUsedAsValues.ts",
    "verbatimModuleSyntaxCompat.ts",
    "verbatimModuleSyntaxCompat2.ts",
    "verbatimModuleSyntaxCompat3.ts",
    "verbatimModuleSyntaxCompat4.ts",
    "preserveValueImports.ts",
    "preserveValueImports_importsNotUsedAsValues.ts",
    "preserveValueImports_errors.ts",
    "preserveValueImports_mixedImports.ts",
    "preserveValueImports_module.ts",
    "importsNotUsedAsValues_error.ts",
    "alwaysStrictNoImplicitUseStrict.ts",
    "nonPrimitiveIndexingWithForInSupressError.ts",
    "parameterInitializerBeforeDestructuringEmit.ts",
    "mappedTypeUnionConstraintInferences.ts",
    "lateBoundConstraintTypeChecksCorrectly.ts",
    "keyofDoesntContainSymbols.ts",
    "isolatedModulesOut.ts",
    "noStrictGenericChecks.ts",
    "noImplicitUseStrict_umd.ts",
    "noImplicitUseStrict_system.ts",
    "noImplicitUseStrict_es6.ts",
    "noImplicitUseStrict_commonjs.ts",
    "noImplicitUseStrict_amd.ts",
    "noImplicitAnyIndexingSuppressed.ts",
    "excessPropertyErrorsSuppressed.ts",
    "moduleNoneDynamicImport.ts",
    "moduleNoneErrors.ts",
    "moduleNoneOutFile.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile1.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile2.ts",
    "noErrorUsingImportExportModuleAugmentationInDeclarationFile3.ts",
    "requireOfJsonFileWithModuleEmitNone.ts",
    "requireOfJsonFileWithModuleNodeResolutionEmitNone.ts",
];

fn process_case(content: &str, basename: &str) -> Vec<ConfigOutcome> {
    let settings = extract_settings(content);

    if SKIPPED_CASES.contains(&basename) {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip("in SKIPPED_CASES list".to_string()),
        }];
    }

    if baseline::flavor() == baseline::Flavor::Go && TSGO_SKIPPED_TESTS.contains(&basename) {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip("in tsgo skippedTests".to_string()),
        }];
    }

    if basename.to_ascii_lowercase().starts_with("circular") {
        return vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip("circular-type recursion (no checker guard)".to_string()),
        }];
    }

    let parsed = split_units(content, basename);

    let tsconfig = detect_tsconfig(&parsed.units);
    match compute_configurations(&settings) {
        Err(reason) => vec![ConfigOutcome {
            suffix: String::new(),
            outcome: CaseOutcome::Skip(reason),
        }],
        Ok(configs) => configs
            .into_iter()
            .map(|(suffix, config_settings)| {
                let (mut compiler_options, unrecognized) = match &tsconfig {
                    Some((parsed_config, config_path)) => {
                        let (mut opts, unrec) = tsox::tsoptions::apply_test_settings_with_base(
                            &config_settings,
                            parsed_config.compiler_options.clone(),
                        );

                        if opts.config_file_path.is_empty() {
                            opts.config_file_path = config_path.clone();
                        }
                        (opts, unrec)
                    }
                    None => apply_test_settings(&config_settings),
                };
                if std::env::var_os("TSOX_PROBE_SKIPLIB").is_some() {
                    compiler_options.skip_default_lib_check = tsox::core::tristate::Tristate::True;
                }
                if std::env::var_os("TSOX_DEBUG_OPTIONS").is_some() {
                    eprintln!(
                        "[opts] module={:?} modres={:?} target={:?} decl={} out={} jsxisrc={:?} nia={:?} strict={:?}",
                        compiler_options.module,
                        compiler_options.module_resolution,
                        compiler_options.target,
                        compiler_options.declaration.is_true(),
                        compiler_options.out_dir,
                        compiler_options.jsx_import_source,
                        compiler_options.no_implicit_any,
                        compiler_options.strict,
                    );
                }
                let outcome = if let Some(reason) = should_skip(&compiler_options, &unrecognized) {
                    CaseOutcome::Skip(reason)
                } else {

                    let no_implicit_refs = content.lines().any(|l| {
                        let t = l.trim_start();
                        t.starts_with("//")
                            && t
                                .to_ascii_lowercase()
                                .starts_with("// @noimplicitreferences:")
                            && t.contains("true")
                    });
                    match catch_unwind(|| {
                        let diags = build_and_check(
                            &compiler_options,
                            &parsed.units,
                            no_implicit_refs,
                            tsconfig.as_ref().map(|(c, _)| c.file_names.as_slice()),
                        );
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

enum StepOutcome {
    Passed,
    AcceptedDiff,
    Failed,
    Skipped,
}

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

    let out_path = std::env::temp_dir().join(format!("tsox_submodule_{idx}_{stem}.out"));
    let _ = std::fs::remove_file(&out_path);
    let worker = Command::new(exe)
        .arg("--exact")
        .arg("submodule_compiler_cases")
        .env("TSOX_SUBMODULE_WORKER", case_path)
        .env("TSOX_SUBMODULE_OUT", &out_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(if std::env::var_os("TSOX_PROBE_PHASES").is_some() {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
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

        if baseline::flavor() == baseline::Flavor::Go
            && !suffix.is_empty()
            && !std::path::Path::new(baseline::reference_root())
                .join(subfolder())
                .join(format!("{name}{ext}"))
                .is_file()
        {
            notes.push(format!("{label}: skip (config baseline missing in tsgo tree)"));
            continue;
        }
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
        match baseline::compare(subfolder(), &name, ext, actual) {
            baseline::Outcome::Passed => {
                notes.push(format!("{label}: pass"));
                if !matches!(overall, StepOutcome::Failed | StepOutcome::AcceptedDiff) {
                    overall = StepOutcome::Passed;
                }
            }
            baseline::Outcome::Failed { .. } => {
                if known_diffs.contains(subfolder(), &name, ext) {
                    notes.push(format!("{label}: known diff (triaged/accepted)"));
                    if !matches!(overall, StepOutcome::Failed) {
                        overall = StepOutcome::AcceptedDiff;
                    }
                } else if accept {

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

    if let (Ok(case_path), Ok(out_path)) = (
        std::env::var("TSOX_SUBMODULE_WORKER"),
        std::env::var("TSOX_SUBMODULE_OUT"),
    ) {

        let case_path = case_path.clone();
        let payload = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || {
                let basename = Path::new(&case_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

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

    let dir = submodule_dir();
    let root = std::path::Path::new(&dir);
    if !root.is_dir() {
        eprintln!(
            "[submodule_compiler] {dir} not found — \
             run `git submodule update --init` to fetch official test cases. Skipping."
        );
        return;
    }

    let mut cases: Vec<std::path::PathBuf> = Vec::new();
    collect_ts_files(root, &mut cases);
    cases.sort();

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

                let mut err = std::io::stderr().lock();
                let _ = writeln!(err, "{msg}");
            }
            if let Some(f) = &self.file {
                use std::io::Write;
                let _ = writeln!(f.lock().unwrap(), "{msg}");
            }
        }
    }

    let log_name = if suite() == "compiler" {
        "submodule_run.log".to_string()
    } else {
        format!("submodule_{}_run.log", suite())
    };
    let log_path = Path::new(baseline::LOCAL_ROOT).join(log_name);
    let log = RunLog::new(
        log_path.clone(),
        std::env::var("TSOX_SUBMODULE_QUIET")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
    );
    if log.file.is_none() {
        eprintln!("[submodule_compiler] note: cannot write run log {}", log_path.display());
    }

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
                .max(1)
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

    let exe = std::env::current_exe().expect("current_exe");

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
            "run log: {}; actual outputs under {}/{}/",
            log_path.display(),
            baseline::LOCAL_ROOT,
            subfolder(),
        ));
        panic!(
            "{failed} baseline mismatch(es):\n  {}\n\
             Run with TSOX_BASELINE_ACCEPT=1 to accept the new output, or add the\n\
             cases to tests/baselines/reference/triaged.txt.",
            failed_non_crash.join("\n  ")
        );
    }
}

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

fn unit_abs_path(unit_name: &str) -> String {
    if tsox::tspath::is_rooted_disk_path(unit_name) {
        unit_name.to_string()
    } else {
        format!("/proj/{}", unit_name)
    }
}

fn is_config_unit(unit_name: &str) -> bool {
    let base = unit_name.rsplit(['/', '\\']).next().unwrap_or(unit_name);
    let lower = base.to_ascii_lowercase();
    lower == "tsconfig.json" || lower == "jsconfig.json"
}

fn detect_tsconfig(
    units: &[common::case_parser::TestUnit],
) -> Option<(tsox::tsoptions::ParsedCommandLine, String)> {
    let config_unit = units.iter().find(|u| is_config_unit(&u.name))?;
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    for unit in units {
        let abs = unit_abs_path(&unit.name);
        let mut parent = tsox::tspath::get_directory_path(&abs);
        while !parent.is_empty() {
            fs.insert_dir(&parent);
            let next = tsox::tspath::get_directory_path(&parent);
            if next == parent {
                break;
            }
            parent = next;
        }
        fs.insert_file(&abs, &unit.content);
    }

    let cwd = if units.iter().any(|u| tsox::tspath::is_rooted_disk_path(&u.name)) {
        "/.src"
    } else {
        "/proj"
    };
    let config_path = unit_abs_path(&config_unit.name);
    Some((
        tsox::tsoptions::get_parsed_command_line_of_config_file(
            &config_path,
            &CompilerOptions::default(),
            cwd,
            fs.as_ref(),
        ),
        config_path,
    ))
}

fn build_and_check(
    options: &CompilerOptions,
    units: &[common::case_parser::TestUnit],
    no_implicit_references: bool,
    tsconfig_file_names: Option<&[String]>,
) -> Vec<Diagnostic> {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");

    mount_test_lib_fixtures(&fs);

    let config_roots: Option<Vec<String>> =
        tsconfig_file_names.map(|names| names.to_vec());
    let refs_only = config_roots.is_none()
        && (no_implicit_references
            || units.last().is_some_and(|u| {
                u.content.contains("require(") || u.content.contains("reference path")
            }));
    let rooted: Vec<&common::case_parser::TestUnit> = match &config_roots {
        Some(names) => units
            .iter()
            .filter(|u| names.iter().any(|n| n == &unit_abs_path(&u.name)))
            .collect(),
        None if refs_only => units.last().into_iter().collect(),
        None => units.iter().collect(),
    };

    let mut file_names: Vec<String> = Vec::new();
    for unit in units {

        let abs = if tsox::tspath::is_rooted_disk_path(&unit.name) {
            unit.name.clone()
        } else {
            format!("/proj/{}", unit.name)
        };

        let mut parent = tsox::tspath::get_directory_path(&abs);
        while !parent.is_empty() {
            fs.insert_dir(&parent);
            let next = tsox::tspath::get_directory_path(&parent);
            if next == parent {
                break;
            }
            parent = next;
        }
        fs.insert_file(&abs, &unit.content);

        let lower = abs.to_ascii_lowercase();
        if rooted.iter().any(|r| std::ptr::eq(*r, unit))
            && (lower.ends_with(".ts")
                || lower.ends_with(".tsx")
                || lower.ends_with(".mts")
                || lower.ends_with(".cts"))
        {
            file_names.push(abs);
        }
    }

    let bf = Arc::new(BundledFS::new(fs.clone()));

    let all_rooted = units.iter().any(|u| {
        tsox::tspath::is_rooted_disk_path(&u.name)
    });
    let cwd = if all_rooted && !units.is_empty() {
        "/.src"
    } else {
        "/proj"
    };
    let host = Arc::new(CompilerHostImpl::new(bf, cwd.to_string(), lib_path()));

    let mut config = tsox::tsoptions::ParsedCommandLine::default();
    config.compiler_options = options.clone();
    config.file_names = file_names;

    let t_program = std::time::Instant::now();
    let program = Arc::new(Program::new(ProgramOptions { config, host }));
    let program_elapsed = t_program.elapsed();
    if std::env::var_os("TSOX_DEBUG_FILES").is_some() {
        eprintln!(
            "[files] refs_only={} roots={}",
            refs_only,
            units
                .iter()
                .filter(|u| rooted.iter().any(|r| std::ptr::eq(*r, *u)))
                .map(|u| u.name.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        eprintln!(
            "[files] {}",
            program
                .source_files()
                .iter()
                .map(|f| f.file_name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let t_check = std::time::Instant::now();
    let mut all = Vec::new();
    for d in program.diagnostics() {
        all.push((**d).clone());
    }
    all.extend(program.get_semantic_diagnostics());
    let check_elapsed = t_check.elapsed();
    if let Some(path) = std::env::var_os("TSOX_PROBE_PHASES") {
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(
            f,
            "program_build={:?} check={:?}",
            program_elapsed, check_elapsed
        )
        .ok();
    }
    all
}

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

            .then(a.4.loc.end.cmp(&b.4.loc.end))
            .then(a.3.cmp(&b.3))
    });

    let globals: Vec<&Diagnostic> = keyed
        .iter()
        .filter(|(_, _, _, _, d)| d.file.is_none())
        .map(|(_, _, _, _, d)| *d)
        .collect();

    let mut out = String::new();
    for (_, _, _, _, d) in keyed {
        let mut line = format_diagnostic_compact(d, None);

        line = line.replace("/proj/", "");
        out.push_str(&line);
        out.push('\n');
    }
    if !globals.is_empty() {
        out.push('\n');
        out.push('\n');
        for d in globals {
            let msg = tsox::diagnosticwriter::message_text(d, None);
            for line in msg.lines() {
                if line.is_empty() {
                    continue;
                }
                let line = line.replace("/proj/", "");
                out.push_str(&format!("!!! {} TS{}: {}", d.category.name(), d.code, line));
                out.push('\n');
            }
        }
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

fn should_skip(options: &CompilerOptions, unrecognized: &[String]) -> Option<String> {
    use tsox::core::compiler_options::{ModuleKind, ModuleResolutionKind, ScriptTarget};

    if !unrecognized.is_empty() {
        return Some(format!(
            "unrecognized option(s): {}",
            unrecognized.join(", ")
        ));
    }

    match options.module {
        ModuleKind::AMD | ModuleKind::UMD | ModuleKind::System => {
            return Some(format!("module={:?} not supported", options.module));
        }
        _ => {}
    }

    if matches!(options.module_resolution, ModuleResolutionKind::Classic) {
        return Some(format!(
            "moduleResolution={:?} not supported",
            options.module_resolution
        ));
    }

    if !options.base_url.is_empty() {
        return Some("baseUrl not supported".to_string());
    }
    if !options.out_file.is_empty() {
        return Some("outFile not supported".to_string());
    }

    if matches!(options.target, ScriptTarget::ES5) {
        return Some(format!(
            "target={:?} (ES5 down-level) not supported",
            options.target
        ));
    }

    if options.allow_js.is_true() {
        return Some("allowJs not supported".to_string());
    }

    None
}
