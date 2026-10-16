use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;


fn dedup_ordered<'a>(names: &[&'a str]) -> Vec<&'a str> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for n in names {
        if seen.insert(*n) {
            out.push(*n);
        }
    }
    out
}

pub fn write(trace_dir: &Path, fail_keys: &BTreeSet<String>, out: &Path) -> Result<usize> {
    let mut stems: Vec<String> = fail_keys
        .iter()
        .filter_map(|k| {
            let name = k.strip_prefix("compiler/")?;
            Some(name.trim_end_matches(".ts").trim_end_matches(".tsx").to_string())
        })
        .collect();
    stems.sort();
    stems.dedup();

    let mut rows = String::from("case,func_count,funcs\n");
    let mut count = 0;
    for stem in &stems {
        let path = trace_dir.join(format!("{stem}.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let funcs = dedup_ordered(&text.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>());
        rows.push_str(&format!("{case},{n},{funcs_j}\n", case = stem, n = funcs.len(), funcs_j = funcs.join(";")));
        count += 1;
    }
    std::fs::write(out, rows)?;
    Ok(count)
}
