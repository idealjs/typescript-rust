// 分配密度观测与分配器选型:计数器(mimalloc 之上)逐例上报,验证分配路径占比
#[global_allocator]
static GLOBAL_ALLOC: tsox_harness::alloc::CountingAlloc<tsox_harness::alloc::WrappedMiMalloc> =
    tsox_harness::alloc::CountingAlloc::with_inner(tsox_harness::alloc::WrappedMiMalloc::INSTANCE);

use std::time::Instant;

use anyhow::{bail, Context, Result};

use clap::Parser;
use xtask::cli::{self, Cmd, VerifyArgs};
use xtask::paths::Repo;
use xtask::run::{CaseEvent, RunConfig, Status};

fn main() {
    let cmd = cli::Cli::parse().cmd;
    let result = match cmd {
        Cmd::Verify(args) => verify(args),
        Cmd::Clean => clean(),
        Cmd::Snapshot { force } => snapshot(force),
        Cmd::Worker(args) => xtask::worker::run(args.case, args.out, args.resident),
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn snapshot(force: bool) -> Result<()> {
    let repo = Repo::locate()?;
    if !repo.is_main(None)? {
        bail!("snapshot 只允许在 base 检出内执行(当前检出无 base 归属)");
    }
    xtask::report::results_csv::refresh_base_if_allowed(&repo, Some(&repo.data_dir()), force)?;
    let fails = std::fs::read_to_string(repo.data_csv(xtask::paths::FAILURES_BASE_CSV))?
        .lines()
        .count()
        .saturating_sub(1);
    let skips = std::fs::read_to_string(repo.data_csv(xtask::paths::SKIPS_BASE_CSV))?
        .lines()
        .count()
        .saturating_sub(1);
    println!("snapshot: {fails} FAIL / {skips} SKIP @ {}", repo.root.display());
    Ok(())
}

fn clean() -> Result<()> {
    let removed = xtask::logstore::clean_all()?;
    println!("cleaned {removed} run(s) under {}", xtask::paths::RUNS_DIR);
    Ok(())
}

fn verify(args: VerifyArgs) -> Result<()> {
    let started = Instant::now();
    let repo = Repo::locate()?;
    let main_data_dir = repo.main_data_dir(args.base_dir.as_deref());

    let store = xtask::logstore::RunStore::create(args.keep_days, args.keep_runs)?;
    let _lock = xtask::sys::ExclusiveLock::acquire(std::path::Path::new(xtask::paths::LOCK_PATH))?;
    xtask::guard::disk_guard()?;

    // harness 基线路径以 tsox crate 根为 CWD,验证期间统一切过去
    std::env::set_current_dir(repo.tsox_crate_dir())
        .with_context(|| format!("cd {}", repo.tsox_crate_dir().display()))?;

    let selection = xtask::select::select_cases(&repo.testdata_dir(), args.cases.as_deref())?;
    if selection.cases.is_empty() {
        bail!("selection is empty — nothing to do");
    }

    let outcome = xtask::run::verify(
        &store,
        RunConfig {
            work_dir: repo.tsox_crate_dir(),
            resident: std::env::var("TSOX_POOL").unwrap_or_default() == "1",
            jobs: args.jobs,
            mem_bytes: args.mem_mb * 1024 * 1024,
            case_timeout_secs: args.case_timeout_secs,
            deadline: xtask::guard::Deadline::from_secs(args.round_timeout_secs),
        },
        &selection,
    )?;

    // data/ 水位 CSV 只允许全量跑刷新:抽样/单例跑的事件集不是全语料,
    // 覆盖写入会把水位清空(也污染 base 快照)。抽样跑的报告只落运行归档。
    let full_run = args.cases.is_none();
    let sets = if full_run {
        xtask::report::results_csv::write(&repo, &outcome.events)?
    } else {
        xtask::report::results_csv::key_sets(&outcome.events)
    };
    let base_refreshed = if full_run {
        xtask::report::results_csv::refresh_base_if_allowed(
            &repo,
            main_data_dir.as_deref(),
            args.refresh_baseline,
        )?
    } else {
        false
    };

    let types_n = if full_run {
        xtask::report::types_index::write(
            &store.types_dir(),
            &repo.data_csv(xtask::paths::RUST_TYPES_CSV),
        )?
    } else {
        0
    };
    let anchor_out = if full_run {
        repo.data_csv(xtask::paths::TYPES_ANCHOR_CSV)
    } else {
        store.dir.join("types-anchor.csv")
    };
    let anchors = xtask::report::anchor::write(&store.types_dir(), &args.go_ref, &anchor_out)?;
    let traced = if full_run {
        xtask::report::trace_index::write(
            &store.trace_dir(),
            &sets.fails,
            &repo.data_csv(xtask::paths::RUST_TRACE_CSV),
        )?
    } else {
        0
    };
    if full_run {
        xtask::report::stack_diff::write(
            &store.trace_dir(),
            &repo,
            &repo.data_csv(xtask::paths::STACK_DIFF_CSV),
        )?;
    }
    let reference_dir = repo
        .tsox_crate_dir()
        .join("tests/corpus/testdata/baselines/reference/compiler");
    let local_dir = repo
        .tsox_crate_dir()
        .join("tests/corpus/baselines/local/compiler");
    let shard_n = xtask::report::shards::write(
        &store.shards_dir(),
        &outcome.events,
        &repo.data_csv(xtask::paths::STACK_DIFF_CSV),
        &reference_dir,
        &local_dir,
    )?;

    let attribution = xtask::verdict::attribute(
        main_data_dir.as_deref(),
        &sets.fails,
        &sets.skips,
        &sets.timeout_skips,
    );
    if !attribution.base_available {
        eprintln!(
            "note: base snapshot unavailable (no --base-dir / TSOX_BASE_DIR / main-branch worktree) — attribution degraded to absolute water level"
        );
    }
    let anchor_count = anchors.len();
    let target_stem = args.target.as_ref().map(|k| {
        k.trim_start_matches("compiler/")
            .trim_end_matches(".ts")
            .trim_end_matches(".tsx")
            .to_string()
    });
    let selected: Vec<xtask::verdict::SelectedCase> = if args.cases.is_some() || target_stem.is_some() {
        outcome
            .events
            .iter()
            // 全量跑带 --target 时只保留目标例,避免 6820 例塞爆 verdict
            .filter(|e| {
                args.cases.is_some()
                    || target_stem
                        .as_deref()
                        .is_some_and(|t| e.key == format!("compiler/{t}.ts") || e.key == format!("compiler/{t}.tsx"))
            })
            .map(|e: &CaseEvent| {
                let stem = e.name.trim_end_matches(".ts").trim_end_matches(".tsx");
                let anchor = anchors.get(stem);
                xtask::verdict::SelectedCase {
                    case: e.key.clone(),
                    status: e.status.label().to_string(),
                    anchor_gone: Some(anchor.is_none()),
                    anchor_context: anchor.map(|a| a.context.clone()),
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let has_real_regression =
        !attribution.new_fails.is_empty() || !attribution.new_skips.real.is_empty();
    let selected_fail = selection_has_fail(&selected);
    let status = if outcome.interrupted {
        "timeout"
    } else if has_real_regression || selected_fail {
        "fail"
    } else {
        "pass"
    };
    let counts = count_statuses(&outcome.events);
    let v = xtask::verdict::Verdict {
        status: status.to_string(),
        run_id: store.run_id.clone(),
        total: selection.cases.len(),
        completed: outcome.events.len(),
        passed: counts.0,
        failed: counts.1,
        skipped: counts.2,
        accepted_diff: counts.3,
        fail_count: sets.fails.len(),
        skip_count: sets.skips.len(),
        anchor_count,
        base_available: attribution.base_available,
        interrupted: outcome.interrupted,
        elapsed_secs: outcome.elapsed_secs,
        new_fails: attribution.new_fails,
        new_skips: attribution.new_skips,
        fixed: attribution.fixed,
        unmatched: selection.unmatched,
        selected,
    };
    let json = serde_json::to_string(&v)?;
    std::fs::write(store.verdict_json(), serde_json::to_string_pretty(&v)?)?;
    println!("VERDICT_JSON={json}");
    eprintln!(
        "done in {:.1}s — run archive: {} (base_refreshed={base_refreshed}, types={types_n}, traced={traced}, shards={shard_n}, anchor={anchor_count})",
        started.elapsed().as_secs_f32(),
        store.dir.display(),
    );
    Ok(())
}

fn selection_has_fail(selected: &[xtask::verdict::SelectedCase]) -> bool {
    selected.iter().any(|s| s.status == Status::Fail.label())
}

fn count_statuses(events: &[CaseEvent]) -> (usize, usize, usize, usize) {
    let mut p = 0;
    let mut f = 0;
    let mut s = 0;
    let mut a = 0;
    for e in events {
        match e.status {
            Status::Pass => p += 1,
            Status::Fail => f += 1,
            Status::Skip => s += 1,
            Status::AcceptedDiff => a += 1,
        }
    }
    (p, f, s, a)
}
