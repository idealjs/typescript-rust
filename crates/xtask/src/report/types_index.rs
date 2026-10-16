use std::path::Path;

use anyhow::{Context, Result};


fn escape_content(content: &str) -> String {
    let cleaned: String = content
        .chars()
        .map(|c| match c {
            '\u{2028}' | '\u{2029}' | '\u{85}' | '\u{b}' | '\u{c}' => ' ',
            _ => c,
        })
        .collect();
    cleaned.replace('\\', "\\\\").replace('\n', "\\n").replace('"', "\"\"")
}

pub fn write(types_dir: &Path, out: &Path) -> Result<usize> {
    let mut names: Vec<String> = std::fs::read_dir(types_dir)?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            name.ends_with(".types").then_some(name)
        })
        .collect();
    names.sort();

    let mut rows = String::from("case,line_count,content\n");
    for name in &names {
        let stem = &name[..name.len() - ".types".len()];
        let content = std::fs::read_to_string(types_dir.join(name))
            .with_context(|| format!("read {}", name))?;
        let line_count = content.matches('\n').count();
        rows.push_str(&format!(
            "{},{},\"{}\"\n",
            stem,
            line_count,
            escape_content(&content)
        ));
    }
    std::fs::write(out, rows).with_context(|| format!("write {}", out.display()))?;
    Ok(names.len())
}
