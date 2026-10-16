use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use tsox_harness::case_index::collect_ts_files;

pub struct Selection {

    pub cases: Vec<PathBuf>,

    pub unmatched: Vec<String>,
}

fn normalize_key(line: &str) -> String {
    let s = line.trim();
    let s = s.strip_prefix("compiler/").unwrap_or(s);
    s.trim_end_matches(".ts").trim_end_matches(".tsx").to_string()
}

pub fn select_cases(testdata: &Path, cases_file: Option<&Path>) -> Result<Selection> {
    let mut cases: Vec<PathBuf> = Vec::new();
    collect_ts_files(testdata, &mut cases);
    cases.sort();

    let Some(file) = cases_file else {
        return Ok(Selection {
            cases,
            unmatched: Vec::new(),
        });
    };

    let text = std::fs::read_to_string(file)
        .with_context(|| format!("read cases file {}", file.display()))?;
    let keys: Vec<String> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(normalize_key)
        .collect();

    let mut by_stem: BTreeMap<String, PathBuf> = BTreeMap::new();
    for p in &cases {
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .context("non-utf8 case name")?;
        by_stem.entry(stem.to_string()).or_insert_with(|| p.clone());
    }

    let mut selected: Vec<PathBuf> = Vec::new();
    let mut unmatched: Vec<String> = Vec::new();
    for key in &keys {
        match by_stem.get(key) {
            Some(p) => {
                if !selected.contains(p) {
                    selected.push(p.clone());
                }
            }
            None => unmatched.push(key.clone()),
        }
    }
    Ok(Selection { cases: selected, unmatched })
}
