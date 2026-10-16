use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub const RUNS_DIR: &str = "/tmp/tsox-verify";
pub const LOCK_PATH: &str = "/tmp/tsox-verify.lock";
pub const DISK_MIN_FREE_BYTES: u64 = 15 * 1024 * 1024 * 1024;
pub const DEFAULT_GO_REF: &str = "/home/cqh/workspace/typescript-go/tsc/testdata/baselines/reference/compiler";

pub const SUITE: &str = "compiler";
pub const DATA_DIR: &str = "data";

pub const FAILURES_CSV: &str = "corpus-failures.csv";
pub const SKIPS_CSV: &str = "corpus-skips.csv";
pub const FAILURES_BASE_CSV: &str = "corpus-failures.base.csv";
pub const SKIPS_BASE_CSV: &str = "corpus-skips.base.csv";
pub const TYPES_ANCHOR_CSV: &str = "types-anchor.csv";
pub const STACK_DIFF_CSV: &str = "stack-diff.csv";
pub const RUST_TRACE_CSV: &str = "rust-trace.csv";
pub const RUST_TYPES_CSV: &str = "rust-types.csv";
pub const GO_TRACE_CSV: &str = "corpus_go_trace.csv";

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    pub fn locate() -> Result<Self> {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("run git rev-parse --show-toplevel")?;
        if !out.status.success() {
            anyhow::bail!("not inside a git repository (run from a worktree)");
        }
        let root = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        Ok(Self { root })
    }

    /// 波起点 base 快照的归属检出:--base-dir 显式指定 > TSOX_BASE_DIR 环境
    /// 变量 > 检出在 main 分支上的 worktree。主 worktree 不一定是数据主仓
    /// (本仓与 oracle 共享对象库,git 视角的主 worktree 是 oracle),探测
    /// 失败时由调用方退化为 base_available=false。
    pub fn base_dir(&self, explicit: Option<&Path>) -> Option<PathBuf> {
        if let Some(dir) = explicit {
            return Some(dir.to_path_buf());
        }
        if let Ok(dir) = std::env::var("TSOX_BASE_DIR") {
            return Some(PathBuf::from(dir));
        }
        self.main_branch_worktree().filter(|p| p != &self.root)
    }

    fn main_branch_worktree(&self) -> Option<PathBuf> {
        let out = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.root)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let mut current: Option<PathBuf> = None;
        for line in text.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                current = Some(PathBuf::from(p));
            } else if let Some(b) = line.strip_prefix("branch ") {
                if b == "refs/heads/main" {
                    let path = current.take()?;
                    if path.join("Cargo.toml").is_file() {
                        return Some(path);
                    }
                    return None;
                }
            }
        }
        None
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join(DATA_DIR)
    }

    pub fn main_data_dir(&self, explicit: Option<&Path>) -> Option<PathBuf> {
        self.base_dir(explicit).map(|p| p.join(DATA_DIR))
    }

    pub fn tsox_crate_dir(&self) -> PathBuf {
        self.root.join("crates/tsox")
    }

    pub fn testdata_dir(&self) -> PathBuf {
        self.tsox_crate_dir()
            .join(format!("tests/corpus/testdata/tests/cases/{SUITE}"))
    }

    pub fn data_csv(&self, name: &str) -> PathBuf {
        self.data_dir().join(name)
    }

    pub fn is_main(&self, explicit: Option<&Path>) -> Result<bool> {
        Ok(self.base_dir(explicit).is_none())
    }
}

pub fn path_str(p: &Path) -> Result<String> {
    Ok(p.to_str().context("non-utf8 path")?.to_string())
}
