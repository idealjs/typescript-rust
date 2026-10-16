use tsox_core::diagnostics::Category;
use tsox_frontend::ast::Diagnostic;
use tsox_frontend::diagnosticwriter::{
    format_diagnostic_compact, format_diagnostic_pretty, line_and_character, message_text,
};

use crate::baseline::NO_CONTENT;

pub fn render_errors_baseline(diags: &[Diagnostic], program_len: usize, pretty: bool) -> String {
    // tsc .errors.txt 基线不含纯 suggestion（allowUnreachableCode 未设置时
    // 的 TS7027 等仅编辑器提示）
    let owned: Vec<Diagnostic> = diags
        .iter()
        .filter(|d| d.category != Category::Suggestion)
        .cloned()
        .collect();
    let diags: &[Diagnostic] = &owned;
    if diags.is_empty() {
        return NO_CONTENT.to_string();
    }
    let mut keyed: Vec<(String, usize, usize, usize, i32, &Diagnostic)> = diags
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let (file_name, line, col) = if let Some(f) = &d.file {
                let (l, c) = crate_line_col(d);
                (f.file_name.clone(), l, c)
            } else {
                (String::new(), 0, 0)
            };
            let bucket = if i < program_len { 0 } else { 1 };
            (file_name, line, col, bucket, d.code, d)
        })
        .collect();
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))

            .then(a.5.loc.end.cmp(&b.5.loc.end))
            .then(a.3.cmp(&b.3))
            .then(a.4.cmp(&b.4))
    });

    let globals: Vec<&Diagnostic> = keyed
        .iter()
        .filter(|(_, _, _, _, _, d)| d.file.is_none())
        .map(|(_, _, _, _, _, d)| *d)
        .collect();

    let mut out = String::new();
    for (_, _, _, _, _, d) in keyed {
        let mut line = if pretty {
            format_diagnostic_pretty(d, None)
        } else {
            format_diagnostic_compact(d, None)
        };

        line = line.replace("/proj/", "");
        out.push_str(&line);
        if !pretty {
            out.push('\n');
        }
    }
    if !globals.is_empty() {
        out.push('\n');
        out.push('\n');
        for d in globals {
            let msg = message_text(d, None);
            for line in msg.lines() {
                if line.is_empty() {
                    continue;
                }
                let line = line.replace("/proj/", "");
                out.push_str(&format!("!!! {} TS{}: {}", d.category.name(), d.code, line));
                out.push('\n');
            }
        }
    }
    out
}

fn crate_line_col(d: &Diagnostic) -> (usize, usize) {
    if let Some(file) = &d.file {
        line_and_character(&file.line_map, &file.text, d.loc.pos())
    } else {
        (0, 0)
    }
}
