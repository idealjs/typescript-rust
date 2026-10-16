use std::path::Path;
use std::process::{Command, Stdio};

use tsox_harness::baseline::{self, KnownDiffs, Outcome};

pub enum StepOutcome {
    Passed,
    AcceptedDiff,
    Failed,
    Skipped,
}

pub fn run_case(
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
    let fn_trace_path = std::env::var_os("TSOX_FN_TRACE_DIR")
        .map(|d| std::path::Path::new(&d).join(format!("{stem}.txt")).to_string_lossy().into_owned())
        .unwrap_or_default();

    let types_dir = std::env::var_os("TSOX_TYPES_EMIT_DIR");
    let types_stem = types_dir.as_ref().map(|_| stem.replace('/', "_")).unwrap_or_default();
    let types_header = types_dir.as_ref().map(|_| format!("tests/cases/compiler/{basename}")).unwrap_or_default();
    let _ = std::fs::remove_file(&out_path);
    let worker = Command::new(exe)
        .arg("--exact")
        .arg("submodule_compiler::submodule_compiler_cases")
        .args(if std::env::var_os("TSOX_PROBE_PHASES").is_some() {
            vec!["--nocapture".to_string()]
        } else {
            Vec::new()
        })
        .env("TSOX_SUBMODULE_WORKER", case_path)
        .env("TSOX_SUBMODULE_OUT", &out_path)
        .env("TSOX_FN_TRACE", &fn_trace_path)
        .env("TSOX_TYPES_EMIT_DIR", std::env::var_os("TSOX_TYPES_EMIT_DIR").unwrap_or_default())
        .env("TSOX_TYPES_HEADER", &types_header)
        .env("TSOX_TYPES_STEM", &types_stem)
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
        match baseline::compare(super::subfolder(), &name, ext, actual) {
            Outcome::Passed => {
                notes.push(format!("{label}: pass"));
                if !matches!(overall, StepOutcome::Failed | StepOutcome::AcceptedDiff) {
                    overall = StepOutcome::Passed;
                }
            }
            Outcome::Failed { .. } => {
                if known_diffs.contains(super::subfolder(), &name, ext) {
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
