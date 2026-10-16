use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use tsox_harness::baseline::{self, KnownDiffs};
use tsox_harness::case_index::collect_ts_files;

mod runlog;
mod selection;
mod worker_spawn;

use runlog::RunLog;
use worker_spawn::{run_case, StepOutcome};

const DEFAULT_LIMIT: usize = 1000;

const CASE_TIMEOUT_DEFAULT_SECS: u64 = 30;

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

fn subfolder() -> &'static str {
    suite()
}

fn submodule_dir() -> String {
    format!("tests/corpus/testdata/tests/cases/{}", suite())
}

#[test]
fn submodule_compiler_cases() {

    if let (Ok(case_path), Ok(out_path)) = (
        std::env::var("TSOX_SUBMODULE_WORKER"),
        std::env::var("TSOX_SUBMODULE_OUT"),
    ) {

        tsox_harness::worker::run_worker(Path::new(&case_path), Path::new(&out_path));
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

    let log = RunLog::for_suite(suite());
    let total = cases.len();
    let Some(selection) = selection::select(&cases, total, &log) else {
        return;
    };
    let selected = &selection.cases;
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

    use std::sync::atomic::AtomicUsize;
    let next_case = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let passed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let accepted_diff = AtomicUsize::new(0);
    let failed_non_crash: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    const HEARTBEAT_EVERY: usize = 100;

    let first_no = selection.window_start;
    let last_no = selection.window_end;
    log.line(&format!(
        "[submodule_compiler] {total} cases enumerated; selection '{}' \
         (+filter '{}') → #{first_no}..#{last_no} = {selected_total} cases \
         [{first} … {last}] on {jobs} workers, timeout {}s; log: {}",
        selection.desc,
        selection.filter,
        case_timeout.as_secs(),
        log.path().display(),
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
                let case_path = &selected[i];
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
            log.path().display(),
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
