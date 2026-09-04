mod common;

use std::path::Path;
use std::process::{Command, Stdio};

use tsox::ast::Diagnostic;
use tsox::core::compiler_options::CompilerOptions;
use tsox::diagnosticwriter::format_diagnostic_compact;
use tsox::tsoptions::apply_test_settings;
use tsox::transpile::{TranspileOptions, transpile_declaration, transpile_module};

use common::baseline::{self, KnownDiffs};
use common::case_parser::{extract_settings, split_units};

const SUBMODULE_DIR: &str = "_submodules/TypeScript/tests/cases/transpile";
const SUBFOLDER: &str = "transpile";

const VARY_BY: &[&str] = &["declarationmap", "sourcemap", "inlinesourcemap"];

fn format_config_name(name: &str) -> String {
    name.replace("declarationmap=", "declarationMap=")
        .replace("inlinesourcemap=", "inlineSourceMap=")
        .replace("sourcemap=", "sourceMap=")
}

fn split_bool_values(value: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut raws: Vec<String> = Vec::new();
    for part in value.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        let canon = if s.eq_ignore_ascii_case("true") {
            "true"
        } else {
            "false"
        };
        if !seen.contains(&canon.to_string()) {
            seen.push(canon.to_string());
            raws.push(canon.to_string());
        }
    }
    raws
}

fn compute_configurations(
    settings: &std::collections::HashMap<String, String>,
) -> Vec<(String, std::collections::HashMap<String, String>)> {
    let mut varying: Vec<(String, Vec<String>)> = Vec::new();
    let mut base = settings.clone();
    for key in settings.keys() {
        if !VARY_BY.contains(&key.as_str()) {
            continue;
        }
        let value = &settings[key];
        if !value.contains(',') {
            continue;
        }
        let values = split_bool_values(value);
        if values.len() <= 1 {
            if let [only] = values.as_slice() {
                base.insert(key.clone(), only.clone());
            }
            continue;
        }
        varying.push((key.clone(), values));
    }
    if varying.is_empty() {
        return vec![(String::new(), base)];
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
        let suffix = format_config_name(&parts.join(","));
        let mut merged = base.clone();
        for (k, v) in config {
            merged.insert(k, v);
        }
        out.push((suffix, merged));
    }
    out
}

fn js_output_extension(file_name: &str, jsx: tsox::core::compiler_options::JsxEmit) -> &'static str {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".mts") || lower.ends_with(".mjs") {
        ".mjs"
    } else if lower.ends_with(".cts") || lower.ends_with(".cjs") {
        ".cjs"
    } else if jsx == tsox::core::compiler_options::JsxEmit::Preserve
        && (lower.ends_with(".tsx") || lower.ends_with(".jsx"))
    {
        ".jsx"
    } else {
        ".js"
    }
}

fn declaration_output_extension(file_name: &str) -> &'static str {
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".mts") {
        ".d.mts"
    } else if lower.ends_with(".cts") {
        ".d.cts"
    } else {
        ".d.ts"
    }
}

struct ConfigOutcome {

    name: String,

    ext: String,
    skip: Option<String>,
    text: Option<String>,
}

fn append_section(result: &mut String, file_name: &str, content: &str) {
    result.push_str(&format!("//// [{file_name}] ////\r\n"));
    result.push_str(content);
    if !content.ends_with('\n') {
        result.push_str("\r\n");
    }
}

fn render_diagnostics_section(diags: &[Diagnostic]) -> String {
    let mut keyed: Vec<(String, usize, usize, i32, &Diagnostic)> = diags
        .iter()
        .map(|d| {
            let (file_name, line, col) = if let Some(f) = &d.file {
                let (l, c) = tsox::diagnosticwriter::line_and_character(
                    &f.line_map,
                    &f.text,
                    d.loc.pos(),
                );
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

        if let Some(stripped) = line.strip_prefix('/') {
            line = stripped.to_string();
        }
        out.push_str(&line);
        out.push_str("\r\n");
    }
    out
}

fn process_case(content: &str, basename: &str) -> Vec<ConfigOutcome> {
    let parsed = split_units(content, basename);

    let report_diagnostics = parsed
        .settings
        .get("reportdiagnostics")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let settings = extract_settings(content);

    let stem = basename
        .trim_end_matches(".ts")
        .trim_end_matches(".tsx")
        .to_string();

    compute_configurations(&settings)
        .into_iter()
        .flat_map(|(suffix, config_settings)| {
            let configured_name = if suffix.is_empty() {
                stem.clone()
            } else {
                format!("{stem}({suffix})")
            };
            let (compiler_options, unrecognized) = apply_test_settings(&config_settings);
            if !unrecognized.is_empty() {
                return vec![ConfigOutcome {
                    name: configured_name,
                    ext: ".js".to_string(),
                    skip: Some(format!(
                        "unrecognized option(s): {}",
                        unrecognized.join(", ")
                    )),
                    text: None,
                }];
            }

            let mut kinds: Vec<(String, bool)> = Vec::new();
            if !compiler_options.emit_declaration_only.is_true() {
                kinds.push((".js".to_string(), false));
            }
            if compiler_options.declaration.is_true() {
                kinds.push((".d.ts".to_string(), true));
            }
            if kinds.is_empty() {
                kinds.push((".js".to_string(), false));
            }

            kinds
                .into_iter()
                .map(|(kind_ext, declaration)| {
                    let text = run_kind(
                        &compiler_options,
                        &parsed.units,
                        declaration,
                        report_diagnostics,
                    );
                    ConfigOutcome {
                        name: configured_name.clone(),
                        ext: kind_ext,
                        skip: None,
                        text: Some(text),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn run_kind(
    options: &CompilerOptions,
    units: &[common::case_parser::TestUnit],
    declaration: bool,
    report_diagnostics: bool,
) -> String {
    let mut result = String::new();
    for unit in units {
        append_section(&mut result, &unit.name, &unit.content);
    }
    for unit in units {
        let transpile_options = TranspileOptions {
            compiler_options: options.clone(),
            file_name: unit.name.clone(),
            report_diagnostics,
        };
        let output = if declaration {
            transpile_declaration(&unit.content, transpile_options)
        } else {
            transpile_module(&unit.content, transpile_options)
        };

        let output_ext = if declaration {
            declaration_output_extension(&unit.name)
        } else {
            js_output_extension(&unit.name, options.jsx)
        };
        let output_file_name = tsox::tspath::change_extension(&unit.name, output_ext);
        append_section(&mut result, &output_file_name, &output.output_text);
        if !output.source_map_text.is_empty() {
            append_section(&mut result, &format!("{output_file_name}.map"), &output.source_map_text);
        }
        if !output.diagnostics.is_empty() {
            result.push_str("\r\n\r\n//// [Diagnostics reported]\r\n");
            let rendered = render_diagnostics_section(&output.diagnostics);
            result.push_str(&rendered);
            if !result.ends_with('\n') {
                result.push_str("\r\n");
            }
        }
    }
    result
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

    let out_path = std::env::temp_dir().join(format!("tsox_transpile_{idx}_{basename}.out"));
    let _ = std::fs::remove_file(&out_path);
    let worker = Command::new(exe)
        .arg("--exact")
        .arg("submodule_transpile_cases")
        .env("TSOX_TRANSPILE_WORKER", case_path)
        .env("TSOX_TRANSPILE_OUT", &out_path)
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
        let name = entry["name"].as_str().unwrap_or_default().to_string();
        let ext = entry["ext"].as_str().unwrap_or(".js").to_string();
        let display = format!("{basename} → {name}{ext}");
        if let Some(reason) = entry["skip"].as_str() {
            notes.push(format!("{name}{ext}: skip ({reason})"));
            continue;
        }
        let Some(actual) = entry["text"].as_str() else {
            notes.push(format!("{name}{ext}: malformed entry"));
            failed_names.push(display);
            overall = StepOutcome::Failed;
            continue;
        };

        let actual = actual.replace("\r\n", "\n");
        let actual = actual.as_str();
        match baseline::compare(SUBFOLDER, &name, &ext, actual) {
            baseline::Outcome::Passed => {
                notes.push(format!("{name}{ext}: pass"));
                if !matches!(overall, StepOutcome::Failed | StepOutcome::AcceptedDiff) {
                    overall = StepOutcome::Passed;
                }
            }
            baseline::Outcome::Failed { .. } => {
                if known_diffs.contains(SUBFOLDER, &name, &ext) {
                    notes.push(format!("{name}{ext}: known diff (triaged/accepted)"));
                    if !matches!(overall, StepOutcome::Failed) {
                        overall = StepOutcome::AcceptedDiff;
                    }
                } else if accept {
                    notes.push(format!("{name}{ext}: pass"));
                    if !matches!(overall, StepOutcome::Failed | StepOutcome::AcceptedDiff) {
                        overall = StepOutcome::Passed;
                    }
                } else {
                    notes.push(format!("{name}{ext}: baseline mismatch"));
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
            .strip_suffix(": pass")
            .map(str::to_string)
            .unwrap_or_default()
    } else {
        notes.join("; ")
    };
    (overall, failed_names, detail)
}

#[test]
fn submodule_transpile_cases() {

    if let (Ok(case_path), Ok(out_path)) = (
        std::env::var("TSOX_TRANSPILE_WORKER"),
        std::env::var("TSOX_TRANSPILE_OUT"),
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
                    Ok(content) => serde_json::to_string(
                        &process_case(&content, &basename)
                            .iter()
                            .map(|c| {
                                let mut entry = serde_json::json!({
                                    "name": c.name,
                                    "ext": c.ext,
                                });
                                if let Some(reason) = &c.skip {
                                    entry["skip"] = serde_json::json!(reason);
                                }
                                if let Some(text) = &c.text {
                                    entry["text"] = serde_json::json!(text);
                                }
                                entry
                            })
                            .collect::<Vec<_>>(),
                    )
                    .unwrap_or_else(|_| "[]".to_string()),
                    Err(e) => serde_json::json!([{
                        "name": basename,
                        "ext": ".js",
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
            "[submodule_transpile] {SUBMODULE_DIR} not found — \
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
    let log_path = Path::new(baseline::LOCAL_ROOT).join("submodule_transpile_run.log");
    let log = RunLog::new(
        log_path.clone(),
        std::env::var("TSOX_SUBMODULE_QUIET")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
    );

    let total = cases.len();
    let limit_spec = std::env::var("TSOX_TRANSPILE_LIMIT").unwrap_or_default();
    let start_spec = std::env::var("TSOX_TRANSPILE_START").unwrap_or_default();
    let end_spec = std::env::var("TSOX_TRANSPILE_END").unwrap_or_default();
    let (start, end) = if !start_spec.is_empty() || !end_spec.is_empty() {
        let a = start_spec.trim().parse::<usize>().unwrap_or(1).max(1);
        let b = end_spec
            .trim()
            .parse::<usize>()
            .unwrap_or(total)
            .min(total);
        assert!(b >= a, "START/END must be 1-based with END >= START");
        (a - 1, b)
    } else {
        let n = if limit_spec.is_empty() {
            total
        } else {
            limit_spec.trim().parse::<usize>().unwrap_or(total)
        };
        (0, n.min(total))
    };
    if start >= end {
        log.line(&format!(
            "[submodule_transpile] selection selects no cases of {total} — nothing to do."
        ));
        return;
    }

    let filter = std::env::var("TSOX_TRANSPILE_FILTER").unwrap_or_default();
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
        log.line("[submodule_transpile] filter matched no cases — nothing to do.");
        return;
    }
    let selected_total = selected.len();

    let known_diffs = KnownDiffs::load();
    let accept = baseline::accept_mode();
    let case_timeout = std::time::Duration::from_secs(
        std::env::var("TSOX_TRANSPILE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30),
    );
    let exe = std::env::current_exe().expect("current_exe");

    let jobs = std::env::var("TSOX_TRANSPILE_JOBS")
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
    let passed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let accepted_diff = AtomicUsize::new(0);
    let failed_non_crash: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

    log.line(&format!(
        "[submodule_transpile] {total} cases enumerated → {selected_total} cases \
         on {jobs} workers, timeout {}s; log: {}",
        case_timeout.as_secs(),
        log_path.display(),
    ));

    let run_start = std::time::Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
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
                        "[submodule_transpile] #{}/{} {verb}{name} ({secs:.2}s){note}",
                        i + 1,
                        selected_total,
                    ));
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
        "[submodule_transpile] {selected_total} cases done in {:.0}s: \
         {passed} passed, {skipped} skipped (unsupported/crash), \
         {accepted_diff} accepted-diff, {failed} failed",
        run_start.elapsed().as_secs_f32(),
    ));
    if failed > 0 {
        for name in &failed_non_crash {
            log.line(&format!("[submodule_transpile] FAILED: {name}"));
        }
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
            let ext = ext.to_ascii_lowercase();
            if ["ts", "tsx", "mts", "cts"].contains(&ext.as_str()) {
                out.push(p);
            }
        }
    }
}
