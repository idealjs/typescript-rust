use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::paths::SUITE;
use crate::sys::set_process_limits;

pub struct WorkerSpec {
    pub exe: PathBuf,
    pub work_dir: PathBuf,
    pub trace_dir: Option<PathBuf>,
    pub types_dir: Option<PathBuf>,
    pub mem_bytes: u64,
    pub case_timeout: Duration,
    pub resident: bool,
}

impl WorkerSpec {
    /// 常驻 worker 的虚拟内存下限 8GiB:进程持 bind/parse 缓存(~数百 MB)
    /// 且分配器跨例滞留,重例单例峰值已达 3.9GB,4GiB 会在池化下被推穿
    /// (SIGABRT alloc failure);RSS 不受此值影响,跑飞用例仍由超时兜底。
    pub fn effective_mem_bytes(&self) -> u64 {
        if self.resident {
            self.mem_bytes.max(8 * 1024 * 1024 * 1024)
        } else {
            self.mem_bytes
        }
    }
}

pub enum WorkerExit {
    Done(String),
    TimedOut,
    Failed(String),
}

fn apply_common(spec: &WorkerSpec, cmd: &mut Command) {
    if spec.resident {
        // 常驻 worker 单线程串行:限制 glibc arena 数量,防虚拟地址空间
        // 被 per-thread arena(各 64MB)与滞留碎片吃穿 RLIMIT_AS(SIGABRT)
        cmd.env("MALLOC_ARENA_MAX", "2");
    }
    cmd.current_dir(&spec.work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        // 常驻模式保留 stderr:worker 逐例上报 case/rss,死亡原因可见
        .stderr(if spec.resident {
            Stdio::inherit()
        } else {
            Stdio::null()
        });
    match &spec.types_dir {
        Some(dir) => {
            cmd.env("TSOX_TYPES_EMIT_DIR", dir);
        }
        None => {
            cmd.env_remove("TSOX_TYPES_EMIT_DIR");
            cmd.env_remove("TSOX_TYPES_STEM");
            cmd.env_remove("TSOX_TYPES_HEADER");
        }
    }
    let mem_bytes = spec.effective_mem_bytes();
    unsafe {
        cmd.pre_exec(move || {
            set_process_limits(mem_bytes, 10);
            Ok(())
        });
    }
}

pub fn run_worker_one_shot(
    spec: &WorkerSpec,
    case: &Path,
    idx: usize,
) -> Result<(WorkerExit, Duration)> {
    let started = Instant::now();
    let basename = case
        .file_name()
        .and_then(|n| n.to_str())
        .context("non-utf8 case name")?;
    let stem = basename.trim_end_matches(".ts").trim_end_matches(".tsx");
    let out_path = std::env::temp_dir().join(format!("tsox_worker_{idx}_{stem}.json"));
    let _ = std::fs::remove_file(&out_path);

    let mut cmd = Command::new(&spec.exe);
    cmd.arg("__worker").arg("--case").arg(case).arg("--out").arg(&out_path);
    apply_common(spec, &mut cmd);
    match &spec.trace_dir {
        Some(dir) => {
            cmd.env("TSOX_FN_TRACE", dir.join(format!("{stem}.txt")));
        }
        None => {
            cmd.env_remove("TSOX_FN_TRACE");
        }
    }
    if spec.types_dir.is_some() {
        cmd.env("TSOX_TYPES_STEM", stem.replace('/', "_"));
        cmd.env(
            "TSOX_TYPES_HEADER",
            format!("tests/cases/{SUITE}/{basename}"),
        );
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Ok((WorkerExit::Failed(e.to_string()), started.elapsed())),
    };
    let deadline = Instant::now() + spec.case_timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = std::fs::remove_file(&out_path);
                return Err(e).context("wait worker");
            }
        }
    };
    let elapsed = started.elapsed();
    let payload = std::fs::read_to_string(&out_path).unwrap_or_default();
    let _ = std::fs::remove_file(&out_path);
    let exit = match status {
        Err(()) => WorkerExit::TimedOut,
        Ok(s) if s.success() => WorkerExit::Done(payload),
        Ok(s) => {
            use std::os::unix::process::ExitStatusExt;
            let raw = s
                .signal()
                .map(|sig| format!("signal {sig}"))
                .unwrap_or_else(|| format!("code {}", s.code().unwrap_or(-1)));
            WorkerExit::Failed(raw)
        }
    };
    Ok((exit, elapsed))
}

/// 常驻 worker 槽位:一个进程串行吃多个 case,stdin 下发请求,stdout 收 DONE。
/// 超时/崩溃即杀进程,由调用方重建槽位。
pub struct ResidentWorker {
    child: Child,
    stdin: ChildStdin,
    done_rx: Receiver<std::io::Result<String>>,
    out_path: PathBuf,
}

impl ResidentWorker {
    pub fn spawn(spec: &WorkerSpec) -> Result<Self> {
        let mut cmd = Command::new(&spec.exe);
        cmd.arg("__worker").arg("--resident");
        apply_common(spec, &mut cmd);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
        let mut child = cmd.spawn().context("spawn resident worker")?;
        let stdin = child.stdin.take().context("resident stdin")?;
        let stdout: ChildStdout = child.stdout.take().context("resident stdout")?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if tx.send(Ok(line.clone())).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        break;
                    }
                }
            }
        });
        let out_path = std::env::temp_dir().join(format!(
            "tsox_resident_{}_{}.json",
            std::process::id(),
            unix_nanos()
        ));
        Ok(Self {
            child,
            stdin,
            done_rx: rx,
            out_path,
        })
    }

    pub fn run_case(
        &mut self,
        spec: &WorkerSpec,
        case: &Path,
        timeout: Duration,
    ) -> Result<(WorkerExit, Duration)> {
        let started = Instant::now();
        let basename = case
            .file_name()
            .and_then(|n| n.to_str())
            .context("non-utf8 case name")?;
        let stem = basename.trim_end_matches(".ts").trim_end_matches(".tsx");
        let out_path = self.out_path.clone();
        let _ = std::fs::remove_file(&out_path);
        let fn_trace = spec
            .trace_dir
            .as_ref()
            .map(|d| d.join(format!("{stem}.txt")))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let types_stem = stem.replace('/', "_");
        let types_header = format!("tests/cases/{SUITE}/{basename}");

        let request = format!(
            "{}\t{}\t{}\t{}\t{}\n",
            case.display(),
            out_path.display(),
            fn_trace,
            types_stem,
            types_header
        );
        if let Err(e) = self.stdin.write_all(request.as_bytes()) {
            return Ok((
                WorkerExit::Failed(format!("worker crashed ({e})")),
                started.elapsed(),
            ));
        }
        self.stdin.flush().context("flush resident stdin")?;

        let deadline = started + timeout;
        let done = loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.done_rx.recv_timeout(remaining) {
                Ok(Ok(line)) if line.trim() == "DONE" => break Ok(()),
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => break Err(format!("worker crashed ({e})")),
                Err(mpsc::RecvTimeoutError::Timeout) => break Err("__TIMEOUT__".into()),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break Err("worker crashed (stdout closed)".into())
                }
            }
        };
        let elapsed = started.elapsed();
        let exit = match done {
            Ok(()) => {
                let payload = std::fs::read_to_string(&out_path).unwrap_or_default();
                WorkerExit::Done(payload)
            }
            Err(msg) if msg == "__TIMEOUT__" => {
                let _ = self.child.kill();
                let _ = self.child.wait();
                WorkerExit::TimedOut
            }
            Err(msg) => {
                let _ = self.child.kill();
                let status = self.child.wait();
                let detail = match status {
                    Ok(st) => {
                        use std::os::unix::process::ExitStatusExt;
                        st.signal()
                            .map(|sig| format!("{msg} [signal {sig}]"))
                            .unwrap_or_else(|| format!("{msg} [code {}]", st.code().unwrap_or(-1)))
                    }
                    Err(e) => format!("{msg} [wait error: {e}]"),
                };
                WorkerExit::Failed(detail)
            }
        };
        Ok((exit, elapsed))
    }
}

impl Drop for ResidentWorker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn unix_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}
