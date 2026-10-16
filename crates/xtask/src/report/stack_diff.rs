use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use anyhow::Result;

use crate::paths::GO_TRACE_CSV;

fn norm(name: &str) -> String {
    name.replace('_', "").to_lowercase()
}

fn load_set(path: &Path) -> BTreeSet<String> {
    std::fs::read_to_string(path)
        .map(|t| t.lines().map(|l| norm(l.trim())).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

fn strip_case_ext(stem: &str) -> &str {
    for ext in [".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"] {
        if let Some(s) = stem.strip_suffix(ext) {
            return s;
        }
    }
    stem
}

/// Go 侧取一次性快照 GO_TRACE_CSV(case,func_count,funcs),Rust 侧取本轮 trace 目录,
/// 归一函数名后做集合差,高频共现函数视为两侧框架命名噪音剔除。
pub fn write(trace_dir: &Path, repo: &crate::paths::Repo, out: &Path) -> Result<usize> {
    let go_csv = repo.data_dir().join(GO_TRACE_CSV);
    let mut go_by_case: HashMap<String, BTreeSet<String>> = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(&go_csv) {
        for line in text.lines().skip(1) {
            let mut parts = line.splitn(3, ',');
            let (Some(case), Some(_count), Some(funcs)) = (parts.next(), parts.next(), parts.next()) else {
                continue;
            };
            go_by_case.insert(
                case.to_string(),
                funcs.split(';').map(norm).filter(|f| !f.is_empty()).collect(),
            );
        }
    }

    let mut rust_by_case: BTreeMap<String, std::path::PathBuf> = BTreeMap::new();
    for entry in std::fs::read_dir(trace_dir)?.flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".txt") else {
            continue;
        };
        rust_by_case.insert(stem.to_string(), p.clone());
    }

    let mut pairs: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();
    for (case, go) in &go_by_case {
        let Some(rust_path) = rust_by_case.get(case) else {
            continue;
        };
        let rust = load_set(rust_path);
        if go.is_empty() || rust.is_empty() {
            continue;
        }
        let go_only: Vec<String> = go.difference(&rust).cloned().collect();
        let rust_only: Vec<String> = rust.difference(go).cloned().collect();
        pairs.push((case.clone(), go_only, rust_only));
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let noise_threshold = std::cmp::max(1, pairs.len() / 2);
    let mut freq: HashMap<String, usize> = HashMap::new();
    for (_, go_only, rust_only) in &pairs {
        for f in go_only.iter().chain(rust_only.iter()) {
            *freq.entry(f.clone()).or_default() += 1;
        }
    }
    let noise: BTreeSet<String> = freq
        .into_iter()
        .filter(|(_, c)| *c > noise_threshold)
        .map(|(f, _)| f)
        .collect();

    let mut rows = String::from("case,go_only,rust_only\n");
    for (case, go_only, rust_only) in &pairs {
        let go_clean: Vec<&String> = go_only.iter().filter(|f| !noise.contains(*f)).collect();
        let rust_clean: Vec<&String> = rust_only.iter().filter(|f| !noise.contains(*f)).collect();
        rows.push_str(&format!(
            "{},{},{}\n",
            strip_case_ext(case),
            go_clean.into_iter().map(String::as_str).collect::<Vec<_>>().join(";"),
            rust_clean.into_iter().map(String::as_str).collect::<Vec<_>>().join(";"),
        ));
    }
    std::fs::write(out, rows)?;
    Ok(pairs.len())
}
