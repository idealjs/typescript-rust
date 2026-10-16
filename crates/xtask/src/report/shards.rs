use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::run::{CaseEvent, Status};

const RUST_ONLY_LIMIT: usize = 60;

fn baseline_file(dir: &Path, name: &str, stem: &str) -> Option<std::path::PathBuf> {
    for candidate in [
        format!("{name}.errors.txt"),
        format!("{stem}.errors.txt"),
        format!("{stem}.ts.errors.txt"),
        format!("{stem}.tsx.errors.txt"),
    ] {
        let p = dir.join(&candidate);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn unified_diff(reference: &Path, local: &Path) -> Option<String> {
    let out = std::process::Command::new("diff")
        .arg("-u")
        .arg("--label")
        .arg("参考")
        .arg("--label")
        .arg("本地")
        .arg(reference)
        .arg(local)
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn write(
    shards_dir: &Path,
    events: &[CaseEvent],
    stack_diff_csv: &Path,
    reference_dir: &Path,
    local_dir: &Path,
) -> Result<usize> {
    let mut stacks: BTreeMap<String, (String, String)> = BTreeMap::new();
    if let Ok(text) = std::fs::read_to_string(stack_diff_csv) {
        for line in text.lines().skip(1) {
            let mut parts = line.splitn(3, ',');
            let (Some(case), Some(go_only), Some(rust_only)) = (parts.next(), parts.next(), parts.next()) else {
                continue;
            };
            stacks.insert(case.to_string(), (go_only.to_string(), rust_only.to_string()));
        }
    }

    let mut count = 0;
    for e in events {
        if e.status != Status::Fail {
            continue;
        }
        let stem = e.name.trim_end_matches(".ts").trim_end_matches(".tsx");
        let ref_file = baseline_file(reference_dir, &e.name, stem);
        let loc_file = baseline_file(local_dir, &e.name, stem);
        let (go_only, rust_only) = stacks
            .get(stem)
            .cloned()
            .unwrap_or_else(|| (String::new(), String::new()));

        let mut md = format!("# 用例 {}\n\n", e.name);
        md.push_str(&format!(
            "全量日志行：#{}/? {} {} ({:.2}s){}\n\n",
            e.index + 1,
            e.status.label(),
            e.name,
            e.secs,
            if e.detail.is_empty() { String::new() } else { format!(" — {}", e.detail) }
        ));
        md.push_str("## 执行栈对照（两侧全函数 trace 差集，见 stack-diff.csv）\n");
        md.push_str(&format!(
            "go_only（Go 执行而 Rust 未执行）：{}\n",
            if go_only.is_empty() { "（无）" } else { &go_only }
        ));
        let rust_shown: Vec<&str> = rust_only.split(';').take(RUST_ONLY_LIMIT).collect();
        md.push_str(&format!(
            "rust_only（Rust 独有执行，前 {} 个）：{}\n\n",
            RUST_ONLY_LIMIT,
            if rust_only.is_empty() { "（无）".to_string() } else { rust_shown.join(";") }
        ));
        md.push_str(&format!(
            "参考基线文件：{}\n",
            ref_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "无（Go 期望零错误，或超时/挂起类）".into())
        ));
        md.push_str(&format!(
            "本地基线文件：{}\n\n",
            loc_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "无（本地无输出：挂起/超时/信号嫌疑）".into())
        ));
        md.push_str("## 参考(期望)\n\n");
        md.push_str(&ref_file
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|t| format!("{}\n", t.trim_end()))
            .unwrap_or_else(|| "（无参考基线文件）\n".into()));
        md.push_str("\n## 本地(实际)\n\n");
        md.push_str(&loc_file
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|t| format!("{}\n", t.trim_end()))
            .unwrap_or_else(|| "（本地未产出 baseline 文件）\n".into()));
        md.push_str("\n## 机械 diff（- 参考 + 本地）\n\n");
        md.push_str(&match (ref_file.as_ref(), loc_file.as_ref()) {
            (Some(r), Some(l)) => unified_diff(r, l)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "（无差异，需排查判定路径）".into()),
            (Some(_), None) => "（本地未产出 baseline 文件）".into(),
            (None, Some(_)) => "（无参考基线文件）".into(),
            (None, None) => "（无法生成 diff：两侧基线文件均缺失）".into(),
        });
        md.push('\n');

        std::fs::write(shards_dir.join(&e.name), md)?;
        count += 1;
    }
    Ok(count)
}
