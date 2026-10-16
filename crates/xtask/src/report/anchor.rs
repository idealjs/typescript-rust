use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;


fn read_lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim_end_matches(['\r', '\n']).to_string())
        .collect()
}

// types_anchor.py 的两个噪声模式:^>(\S+) : typeof \1$(且 ref 行以 > 开头)、^>\S+ : any$
fn split_type_line(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix('>')?;
    let (name, tail) = rest.split_once(" : ")?;
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    Some((name, tail))
}

fn is_typeof_noise(local: &str, reference: &str) -> bool {
    match split_type_line(local) {
        Some((name, tail)) => tail == format!("typeof {name}") && reference.starts_with('>'),
        None => false,
    }
}

fn is_any_noise(local: &str) -> bool {
    matches!(split_type_line(local), Some((_, "any")))
}

fn anchor_case(stem: &str) -> &str {
    for ext in [".ts", ".tsx"] {
        if let Some(s) = stem.strip_suffix(ext) {
            return s;
        }
    }
    stem
}

pub struct AnchorRow {
    pub line_no: usize,
    pub context: String,
}

pub fn write(
    local_dir: &Path,
    go_ref_dir: &Path,
    out: &Path,
) -> Result<BTreeMap<String, AnchorRow>> {
    let mut anchors = BTreeMap::new();
    let mut rows = String::from("case,line_no,context\n");
    let mut names: Vec<String> = std::fs::read_dir(local_dir)?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            name.ends_with(".types").then_some(name)
        })
        .collect();
    names.sort();

    for name in names {
        let ref_path = go_ref_dir.join(&name);
        if !ref_path.is_file() {
            continue;
        }
        let local = read_lines(&local_dir.join(&name));
        let expect = read_lines(&ref_path);
        let stem = &name[..name.len() - ".types".len()];
        let (mut i, mut j) = (0usize, 0usize);
        let mut found: Option<(usize, String, String)> = None;
        while i < local.len() || j < expect.len() {
            let a = local.get(i).map(String::as_str).unwrap_or("<EOF>");
            let b = expect.get(j).map(String::as_str).unwrap_or("<EOF>");
            if a == b {
                i += 1;
                j += 1;
                continue;
            }
            if is_typeof_noise(a, b) || is_any_noise(a) {
                i += 1;
                j += 1;
                continue;
            }
            if a.starts_with('>') && b.starts_with('>') {
                found = Some((i + 1, a.to_string(), b.to_string()));
                break;
            }
            if i + 1 < local.len() && local[i + 1] == b {
                i += 1;
                continue;
            }
            if j + 1 < expect.len() && expect[j + 1] == a {
                j += 1;
                continue;
            }
            found = Some((i + 1, a.to_string(), b.to_string()));
            break;
        }
        if let Some((line_no, l, r)) = found {
            let case = anchor_case(stem);
            let ctx = format!("L:{l} || R:{r}").replace('"', "'");
            rows.push_str(&format!("{case},{line_no},\"{ctx}\"\n"));
            anchors.insert(
                case.to_string(),
                AnchorRow {
                    line_no,
                    context: ctx,
                },
            );
        }
    }
    std::fs::write(out, rows)?;
    Ok(anchors)
}
