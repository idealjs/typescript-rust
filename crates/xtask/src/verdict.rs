use std::collections::BTreeSet;
use std::path::Path;

use serde::Serialize;

use crate::paths::{FAILURES_BASE_CSV, SKIPS_BASE_CSV};

#[derive(Serialize)]
pub struct SelectedCase {
    pub case: String,
    pub status: String,
    pub anchor_gone: Option<bool>,
    pub anchor_context: Option<String>,
}

#[derive(Serialize, Default)]
pub struct SkipBuckets {
    pub real: Vec<String>,
    pub timeout: Vec<String>,
}

#[derive(Serialize)]
pub struct Verdict {
    pub status: String,
    pub run_id: String,
    pub total: usize,
    pub completed: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub accepted_diff: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub anchor_count: usize,
    pub base_available: bool,
    pub interrupted: bool,
    pub elapsed_secs: u64,
    pub new_fails: Vec<String>,
    pub new_skips: SkipBuckets,
    pub fixed: usize,
    pub unmatched: Vec<String>,
    pub selected: Vec<SelectedCase>,
}

fn read_key_set(path: &Path) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        for line in text.lines().skip(1) {
            let k = line.trim();
            if !k.is_empty() {
                set.insert(k.to_string());
            }
        }
    }
    set
}

pub struct Attribution {
    pub base_available: bool,
    pub new_fails: Vec<String>,
    pub new_skips: SkipBuckets,
    pub fixed: usize,
}

/// 相对主仓波起点快照(base)的回归归因。
/// 语义沿用既有口径:new FAIL 不含 skip→fail 翻转;新 SKIP 按超时噪声分桶
/// (超时桶不计回归);fixed = base FAIL 中不再 FAIL 且未转为 SKIP 的数量。
pub fn attribute(
    main_data_dir: Option<&Path>,
    fails_now: &BTreeSet<String>,
    skips_now: &BTreeSet<String>,
    timeout_skips: &BTreeSet<String>,
) -> Attribution {
    let Some(main_data_dir) = main_data_dir else {
        return Attribution {
            base_available: false,
            new_fails: Vec::new(),
            new_skips: SkipBuckets::default(),
            fixed: 0,
        };
    };
    let base_fails_path = main_data_dir.join(FAILURES_BASE_CSV);
    let base_skips_path = main_data_dir.join(SKIPS_BASE_CSV);
    if !base_fails_path.is_file() || !base_skips_path.is_file() {
        return Attribution {
            base_available: false,
            new_fails: Vec::new(),
            new_skips: SkipBuckets::default(),
            fixed: 0,
        };
    }
    let base_fails = read_key_set(&base_fails_path);
    let base_skips = read_key_set(&base_skips_path);

    let new_fails: Vec<String> = fails_now
        .iter()
        .filter(|k| !base_fails.contains(*k) && !base_skips.contains(*k))
        .cloned()
        .collect();

    let mut new_skips = SkipBuckets::default();
    for k in skips_now {
        if base_skips.contains(k) {
            continue;
        }
        if timeout_skips.contains(k) {
            new_skips.timeout.push(k.clone());
        } else {
            new_skips.real.push(k.clone());
        }
    }

    let all_new_skips: BTreeSet<String> = new_skips
        .real
        .iter()
        .chain(new_skips.timeout.iter())
        .cloned()
        .collect();
    let fixed = base_fails
        .iter()
        .filter(|k| !fails_now.contains(*k) && !all_new_skips.contains(*k))
        .count();

    Attribution {
        base_available: true,
        new_fails,
        new_skips,
        fixed,
    }
}
