use std::io::Write;
use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use crate::ast::{LineMap, SourceFile, utf16_len};
use crate::diagnostics::Category;
use crate::locale::Locale;

pub fn line_and_character(line_map: &LineMap, text: &str, offset: usize) -> (usize, usize) {
    let starts = &line_map.line_starts;
    if starts.is_empty() {
        return (0, 0);
    }

    let mut offset = offset.min(text.len());

    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }

    let mut lo = 0usize;
    let mut hi = starts.len();
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if starts[mid] as usize <= offset {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let line = lo;
    let line_start = starts[line] as usize;
    let col = if line_start <= offset {
        utf16_len(&text[line_start..offset])
    } else {
        0
    };
    (line, col)
}

pub fn format_diagnostic_compact(diag: &Diagnostic, locale: Option<&Locale>) -> String {
    let mut out = String::new();
    if let Some(file) = &diag.file {
        let (line, col) = line_and_character(&file.line_map, &file.text, diag.loc.pos());
        out.push_str(&format!("{}({},{}): ", file.file_name, line + 1, col + 1));
    }
    out.push_str(&format!("{} TS{}: ", diag.category.name(), diag.code));
    out.push_str(&message_text(diag, locale));
    out
}

pub fn format_diagnostic_pretty(diag: &Diagnostic, locale: Option<&Locale>) -> String {
    let mut out = String::new();
    if let Some(file) = &diag.file {
        let (line, col) = line_and_character(&file.line_map, &file.text, diag.loc.pos());
        out.push_str(&format!("{}:{}:{} - ", file.file_name, line + 1, col + 1));
    }
    out.push_str(&format!("{} TS{}: ", diag.category.name(), diag.code));
    out.push_str(&message_text(diag, locale));
    if let Some(file) = &diag.file {
        out.push('\n');
        out.push_str(&code_snippet(file, diag.loc.pos(), diag.loc.len()));
    }
    out
}

pub fn message_text(diag: &Diagnostic, locale: Option<&Locale>) -> String {
    message_text_ex(diag, locale, 0)
}

fn message_text_ex(diag: &Diagnostic, locale: Option<&Locale>, depth: usize) -> String {
    let mut out = match &diag.message {
        Some(msg) => {
            let args: Vec<&str> = diag.message_args.iter().map(|s| s.as_str()).collect();
            match locale {
                Some(loc) => msg.localize(loc, &args),
                None => msg.format(&args),
            }
        }
        None => diag.message_args.join(""),
    };
    for chain in &diag.message_chain {
        out.push('\n');
        out.push_str(&"  ".repeat(depth + 1));
        out.push_str(&message_text_ex(chain, locale, depth + 1));
    }
    out
}

fn code_snippet(file: &SourceFile, pos: usize, len: usize) -> String {
    let text = &file.text;
    let (line, col) = line_and_character(&file.line_map, text, pos);
    let line_start = file.line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    let line_end = text[line_start..]
        .find('\n')
        .map(|i| line_start + i)
        .unwrap_or(text.len());
    let line_content = &text[line_start..line_end];
    let squiggle_len = if len == 0 { 1 } else { len }.max(1);
    let mut out = String::new();
    out.push_str(&format!("{} | {}\n", line + 1, line_content));
    out.push_str(&format!("{} | ", line + 1));
    out.push_str(&" ".repeat(col));
    out.push_str(&"~".repeat(squiggle_len));
    out
}

pub fn format_diagnostic(diag: &Diagnostic, pretty: bool, locale: Option<&Locale>) -> String {
    if pretty {
        format_diagnostic_pretty(diag, locale)
    } else {
        format_diagnostic_compact(diag, locale)
    }
}

pub fn write_diagnostics<W: Write>(
    writer: &mut W,
    diags: &[Diagnostic],
    pretty: bool,
    locale: Option<&Locale>,
) -> std::io::Result<()> {
    for diag in diags {
        writeln!(writer, "{}", format_diagnostic(diag, pretty, locale))?;
    }
    Ok(())
}

pub fn report_diagnostics<W: Write>(
    writer: &mut W,
    diags: &[Arc<Diagnostic>],
    pretty: bool,
    locale: Option<&Locale>,
) -> std::io::Result<usize> {
    let mut error_count = 0usize;
    for diag in diags {
        if diag.category == Category::Error {
            error_count += 1;
        }
        writeln!(writer, "{}", format_diagnostic(diag, pretty, locale))?;
    }
    Ok(error_count)
}

#[cfg(test)]
mod tests;
