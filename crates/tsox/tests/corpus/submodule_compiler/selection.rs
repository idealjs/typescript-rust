use std::path::PathBuf;

use super::runlog::RunLog;

pub struct Selection {

    pub cases: Vec<PathBuf>,

    pub desc: String,

    pub filter: String,

    pub window_start: usize,

    pub window_end: usize,
}

pub fn select(cases: &[PathBuf], total: usize, log: &RunLog) -> Option<Selection> {
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
            super::DEFAULT_LIMIT
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
            format!("first {}", super::DEFAULT_LIMIT)
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
        return None;
    }

    let filter = std::env::var("TSOX_SUBMODULE_FILTER").unwrap_or_default();
    let filter_lc = filter.to_lowercase();
    // 修复批清单模式：TSOX_SUBMODULE_CASES_FILE 指向逐行用例基名清单，
    // 精确匹配并绕过 LIMIT/START/END 窗口（修复迭代跑指定批次用）
    let batch_file = std::env::var("TSOX_SUBMODULE_CASES_FILE").unwrap_or_default();
    let batch_set: std::collections::HashSet<String> = if batch_file.is_empty() {
        Default::default()
    } else {
        std::fs::read_to_string(&batch_file)
            .unwrap_or_else(|e| panic!("cannot read cases file {batch_file}: {e}"))
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    };
    let (scan_range, desc) = if batch_set.is_empty() {
        (start..end, desc)
    } else {
        (0..total, format!("batch file {batch_file} ({} names)", batch_set.len()))
    };
    // corpus_one.sh 单用例契约（tools/corpus_one.sh 头注）：filter 为完整用例基名
    // （含或不含扩展名）时只选该用例；仅余前缀同名兄弟时才退化为 substring 选中
    let exact_selected: Vec<&PathBuf> = if batch_set.is_empty() && !filter_lc.is_empty()
    {
        cases[scan_range.clone()]
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.to_lowercase() == filter_lc)
                    || p.file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.to_lowercase() == filter_lc)
            })
            .collect()
    } else {
        Vec::new()
    };
    let selected: Vec<&PathBuf> = if !exact_selected.is_empty() {
        exact_selected
    } else {
        cases[scan_range.clone()]
            .iter()
            .filter(|p| {
                if !batch_set.is_empty() {
                    let base = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    return batch_set.contains(base);
                }
                filter_lc.is_empty()
                    || p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.to_lowercase().contains(&filter_lc))
            })
            .collect()
    };
    if selected.is_empty() {
        log.line(&format!(
            "[submodule_compiler] filter '{filter}' matched no cases — nothing to do."
        ));
        return None;
    }

    Some(Selection {
        cases: selected.into_iter().cloned().collect(),
        desc,
        filter,
        window_start: start + 1,
        window_end: end,
    })
}
