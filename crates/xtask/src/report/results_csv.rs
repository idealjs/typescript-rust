use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};

use crate::paths::{FAILURES_BASE_CSV, FAILURES_CSV, SKIPS_BASE_CSV, SKIPS_CSV};
use crate::run::CaseEvent;

pub struct ResultSets {
    pub fails: BTreeSet<String>,
    pub skips: BTreeSet<String>,
    pub timeout_skips: BTreeSet<String>,
}

pub fn key_sets(events: &[CaseEvent]) -> ResultSets {
    let mut sets = ResultSets {
        fails: BTreeSet::new(),
        skips: BTreeSet::new(),
        timeout_skips: BTreeSet::new(),
    };
    for e in events {
        match e.status {
            crate::run::Status::Fail => {
                sets.fails.insert(e.key.clone());
            }
            crate::run::Status::Skip => {
                sets.skips.insert(e.key.clone());
                if e.timed_out {
                    sets.timeout_skips.insert(e.key.clone());
                }
            }
            crate::run::Status::Pass | crate::run::Status::AcceptedDiff => {}
        }
    }
    sets
}

fn write_key_csv(path: &Path, keys: &BTreeSet<String>) -> Result<()> {
    let mut text = String::from("key\n");
    for k in keys {
        text.push_str(k);
        text.push('\n');
    }
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
}

fn rotate_prev(path: &Path) {
    if path.is_file() {
        let mut prev = path.as_os_str().to_os_string();
        prev.push(".prev");
        let _ = std::fs::copy(path, Path::new(&prev));
    }
}

pub fn write(repo: &crate::paths::Repo, events: &[CaseEvent]) -> Result<ResultSets> {
    let sets = key_sets(events);
    std::fs::create_dir_all(repo.data_dir())?;
    let fails_path = repo.data_csv(FAILURES_CSV);
    let skips_path = repo.data_csv(SKIPS_CSV);
    rotate_prev(&fails_path);
    rotate_prev(&skips_path);
    write_key_csv(&fails_path, &sets.fails)?;
    write_key_csv(&skips_path, &sets.skips)?;
    Ok(sets)
}

/// base 快照只属于 base 检出(缺省即自身):仅 base 归属本检出时可生成/刷新,
/// 否则恒只读。
pub fn refresh_base_if_allowed(
    repo: &crate::paths::Repo,
    base_data_dir: Option<&Path>,
    force: bool,
) -> Result<bool> {
    let Some(base_dir) = base_data_dir else {
        return Ok(false);
    };
    if base_dir != repo.data_dir() {
        return Ok(false);
    }
    let fails_base = base_dir.join(FAILURES_BASE_CSV);
    let skips_base = base_dir.join(SKIPS_BASE_CSV);
    if !force && fails_base.is_file() && skips_base.is_file() {
        return Ok(false);
    }
    std::fs::copy(base_dir.join(FAILURES_CSV), &fails_base)?;
    std::fs::copy(base_dir.join(SKIPS_CSV), &skips_base)?;
    Ok(true)
}
