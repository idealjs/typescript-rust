use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::Result;

use tsox_harness::baseline::{self, KnownDiffs};

use crate::classify::classify;
use crate::guard::Deadline;
use crate::logstore::RunStore;
use crate::schedule::run_pool;
use crate::select::Selection;
use crate::spawn::{run_worker_one_shot, ResidentWorker, WorkerExit, WorkerSpec};
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    AcceptedDiff,
    Fail,
    Skip,
}

impl Status {
    pub fn label(&self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::AcceptedDiff => "DIFF",
            Status::Fail => "FAIL",
            Status::Skip => "SKIP",
        }
    }
}

pub struct CaseEvent {
    pub index: usize,
    pub key: String,
    pub name: String,
    pub status: Status,
    pub secs: f32,
    pub detail: String,
    pub timed_out: bool,
}

pub struct RunConfig {
    pub work_dir: PathBuf,
    pub resident: bool,
    pub jobs: usize,
    pub mem_bytes: u64,
    pub case_timeout_secs: u64,
    pub deadline: Deadline,
}

pub struct RunOutcome {
    pub events: Vec<CaseEvent>,
    pub interrupted: bool,
    pub elapsed_secs: u64,
}

pub fn verify(
    store: &RunStore,
    cfg: RunConfig,
    selection: &Selection,
) -> Result<RunOutcome> {
    let log = RunLog::open(&store.run_log());
    let spec = WorkerSpec {
        exe: std::env::current_exe()?,
        work_dir: cfg.work_dir.clone(),
        resident: cfg.resident,
        trace_dir: Some(store.trace_dir()),
        types_dir: Some(store.types_dir()),
        mem_bytes: cfg.mem_bytes,
        case_timeout: std::time::Duration::from_secs(cfg.case_timeout_secs),
    };
    let known_diffs = Arc::new(KnownDiffs::load());
    let accept = baseline::accept_mode();

    let cases = &selection.cases;
    let total = cases.len();
    let started = Instant::now();
    let counters = Arc::new(Counters::default());
    let events: Arc<Mutex<Vec<CaseEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::new(log);

    log.line(&format!(
        "[xtask] {total} cases selected on {jobs} workers, case timeout {ct}s",
        jobs = cfg.jobs,
        ct = cfg.case_timeout_secs,
    ));

    let ctx = Arc::new(TaskCtx {
        spec,
        known_diffs,
        accept,
        events: events.clone(),
        counters: counters.clone(),
        log: log.clone(),
        total,
    });
    let stats = run_pool(cfg.jobs, total, &cfg.deadline, move |i| {
        run_one(&ctx, i, cases[i].clone());
    });

    let elapsed_secs = started.elapsed().as_secs();
    let interrupted = stats.pending > 0;
    if interrupted {
        log.line(&format!(
            "[xtask] round deadline exceeded — {pending} case(s) not started",
            pending = stats.pending
        ));
    }
    let mut events = Arc::try_unwrap(events)
        .map_err(|_| anyhow::anyhow!("events still shared"))?
        .into_inner()?;
    events.sort_by_key(|e| e.index);

    let c = &counters;
    log.line(&format!(
        "[xtask] {done}/{total} cases done in {elapsed_secs}s: {p} pass, {a} accepted-diff, {s} skip (incl. {t} timeout, {cr} crash), {f} fail",
        done = events.len(),
        elapsed_secs = elapsed_secs,
        p = c.passed.load(Ordering::Relaxed),
        a = c.accepted.load(Ordering::Relaxed),
        s = c.skipped.load(Ordering::Relaxed),
        t = c.timeout.load(Ordering::Relaxed),
        cr = c.crash.load(Ordering::Relaxed),
        f = c.failed.load(Ordering::Relaxed),
    ));
    Ok(RunOutcome {
        events,
        interrupted,
        elapsed_secs,
    })
}

#[derive(Default)]
struct Counters {
    passed: AtomicUsize,
    accepted: AtomicUsize,
    skipped: AtomicUsize,
    timeout: AtomicUsize,
    crash: AtomicUsize,
    failed: AtomicUsize,
}

struct TaskCtx {
    spec: WorkerSpec,
    known_diffs: Arc<KnownDiffs>,
    accept: bool,
    events: Arc<Mutex<Vec<CaseEvent>>>,
    counters: Arc<Counters>,
    log: Arc<RunLog>,
    total: usize,
}

thread_local! {
    static SLOT: RefCell<Option<ResidentWorker>> = const { RefCell::new(None) };
}

fn run_one(ctx: &TaskCtx, i: usize, case: PathBuf) {
    let basename = case
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<bad-name>")
        .to_string();
    let (exit, secs) = if ctx.spec.resident {
        SLOT.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_none() {
                match ResidentWorker::spawn(&ctx.spec) {
                    Ok(w) => *slot = Some(w),
                    Err(e) => {
                        return (
                            WorkerExit::Failed(format!("spawn error: {e}")),
                            std::time::Duration::ZERO,
                        )
                    }
                }
            }
            let w = slot.as_mut().unwrap();
            match w.run_case(&ctx.spec, &case, ctx.spec.case_timeout) {
                Ok(r) => r,
                Err(e) => (
                    WorkerExit::Failed(format!("spawn error: {e}")),
                    std::time::Duration::ZERO,
                ),
            }
        })
    } else {
        match run_worker_one_shot(&ctx.spec, &case, i) {
            Ok(r) => r,
            Err(e) => (
                WorkerExit::Failed(format!("spawn error: {e}")),
                std::time::Duration::ZERO,
            ),
        }
    };
    // 超时/崩溃后槽位已不可信(进程被杀),丢弃让下一例重建
    if matches!(exit, WorkerExit::TimedOut | WorkerExit::Failed(_)) {
        SLOT.with(|s| s.borrow_mut().take());
    }
    // 失败详情在此归一为最终展示形态:常驻路径已带 "worker crashed (...)"
    // 前缀原样保留,单例路径的裸 "signal/code" 在下方统一包装,避免双层嵌套
    let exit = match exit {
        WorkerExit::Failed(raw) if raw.starts_with("worker crashed") => WorkerExit::Failed(raw),
        other => other,
    };
    let (status, detail, timed_out) = match exit {
        WorkerExit::Done(payload) => classify(&payload, &basename, &ctx.known_diffs, ctx.accept),
        WorkerExit::TimedOut => (Status::Skip, "timed out".to_string(), true),
        WorkerExit::Failed(raw) => (Status::Skip, raw, false),
    };
    let event = CaseEvent {
        index: i,
        key: format!("compiler/{basename}"),
        name: basename,
        status,
        secs: secs.as_secs_f32(),
        detail,
        timed_out,
    };
    match status {
        Status::Pass => {
            ctx.counters.passed.fetch_add(1, Ordering::Relaxed);
        }
        Status::AcceptedDiff => {
            ctx.counters.accepted.fetch_add(1, Ordering::Relaxed);
        }
        Status::Fail => {
            ctx.counters.failed.fetch_add(1, Ordering::Relaxed);
        }
        Status::Skip => {
            ctx.counters.skipped.fetch_add(1, Ordering::Relaxed);
            if timed_out {
                ctx.counters.timeout.fetch_add(1, Ordering::Relaxed);
            } else if event.detail.starts_with("worker crashed") {
                ctx.counters.crash.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    ctx.log.line(&format!(
        "[w] #{}/{} {} {} ({:.2}s){note}",
        i + 1,
        ctx.total,
        event.status.label(),
        event.name,
        event.secs,
        note = if event.detail.is_empty() {
            String::new()
        } else {
            format!(" — {}", event.detail)
        },
    ));
    ctx.events.lock().unwrap().push(event);
}

pub struct RunLog {
    file: Option<std::fs::File>,
}

impl RunLog {
    fn open(path: &std::path::Path) -> Self {
        let file = std::fs::File::create(path).ok();
        Self { file }
    }

    pub fn line(&self, msg: &str) {
        println!("{msg}");
        if let Some(f) = &self.file {
            let mut f = f;
            let _ = writeln!(f, "{msg}");
        }
    }
}
