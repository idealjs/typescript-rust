//! Diagnostic formatting and reporting, ported from
//! `internal/diagnosticwriter/diagnosticwriter.go`.
//!
//! Produces human-readable diagnostic output in either the compact
//! `file(line,col): error TS1234: message` form or the pretty
//! `file:line:col - error TS1234: message` form with a code snippet.

use std::io::Write;
use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use crate::ast::{LineMap, SourceFile, utf16_len};
use crate::diagnostics::Category;

/// Convert a byte offset into a 0-based (line, character) pair within a source
/// file, using its line map. The character is a UTF-16 code unit offset from
/// the start of the line, matching Go's `GetECMALineAndUTF16CharacterOfPosition`.
pub fn line_and_character(line_map: &LineMap, text: &str, offset: usize) -> (usize, usize) {
    let starts = &line_map.line_starts;
    if starts.is_empty() {
        return (0, 0);
    }
    // Binary search for the last line start <= offset.
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
    let col = utf16_len(&text[line_start..offset]);
    (line, col)
}

/// Format a single diagnostic into a compact single-line string.
///
/// Mirrors `WriteFormatDiagnostic`: `file(line,col): category TScode: message`.
pub fn format_diagnostic_compact(diag: &Diagnostic) -> String {
    let mut out = String::new();
    if let Some(file) = &diag.file {
        let (line, col) = line_and_character(&file.line_map, &file.text, diag.loc.pos());
        out.push_str(&format!("{}({},{}): ", file.file_name, line + 1, col + 1));
    }
    out.push_str(&format!("{} TS{}: ", diag.category.name(), diag.code));
    out.push_str(&message_text(diag));
    out
}

/// Format a single diagnostic into a pretty string with a code snippet.
///
/// Mirrors `FormatDiagnosticWithColorAndContext` (without ANSI colors).
pub fn format_diagnostic_pretty(diag: &Diagnostic) -> String {
    let mut out = String::new();
    if let Some(file) = &diag.file {
        let (line, col) = line_and_character(&file.line_map, &file.text, diag.loc.pos());
        out.push_str(&format!("{}:{}:{} - ", file.file_name, line + 1, col + 1));
    }
    out.push_str(&format!("{} TS{}: ", diag.category.name(), diag.code));
    out.push_str(&message_text(diag));
    if let Some(file) = &diag.file {
        out.push('\n');
        out.push_str(&code_snippet(file, diag.loc.pos(), diag.loc.len()));
    }
    out
}

/// Resolve the flattened message text of a diagnostic (including its chain).
pub fn message_text(diag: &Diagnostic) -> String {
    let mut out = match &diag.message {
        Some(msg) => {
            let args: Vec<&str> = diag.message_args.iter().map(|s| s.as_str()).collect();
            msg.format(&args)
        }
        None => diag.message_args.join(""),
    };
    for chain in &diag.message_chain {
        out.push('\n');
        out.push_str(&message_text(chain));
    }
    out
}

/// Render the source line containing `pos` with a `~~~` squiggle of `len`.
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

/// Format a diagnostic using the chosen style.
pub fn format_diagnostic(diag: &Diagnostic, pretty: bool) -> String {
    if pretty {
        format_diagnostic_pretty(diag)
    } else {
        format_diagnostic_compact(diag)
    }
}

/// Write a slice of diagnostics to a writer, one per line.
pub fn write_diagnostics<W: Write>(
    writer: &mut W,
    diags: &[Diagnostic],
    pretty: bool,
) -> std::io::Result<()> {
    for diag in diags {
        writeln!(writer, "{}", format_diagnostic(diag, pretty))?;
    }
    Ok(())
}

/// Write a slice of diagnostics to a writer, returning the number of errors.
pub fn report_diagnostics<W: Write>(
    writer: &mut W,
    diags: &[Arc<Diagnostic>],
    pretty: bool,
) -> std::io::Result<usize> {
    let mut error_count = 0usize;
    for diag in diags {
        if diag.category == Category::Error {
            error_count += 1;
        }
        writeln!(writer, "{}", format_diagnostic(diag, pretty))?;
    }
    Ok(error_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::SourceFile;
    use crate::core::text::TextRange;
    use crate::diagnostics::new_ad_hoc_message;

    fn make_file(text: &str) -> Arc<SourceFile> {
        use crate::ast::{Node, NodeData, NodeList, SyntaxKind};
        let line_map = LineMap::from_text(text);
        Arc::new(SourceFile {
            node: Arc::new(Node::with_loc(
                SyntaxKind::SourceFile,
                NodeData::SourceFile(crate::ast::node_data_generated::SourceFileData {
                    statements: Arc::new(NodeList::default()),
                    end_of_file_token: Arc::new(Node::with_loc(
                        SyntaxKind::EndOfFile,
                        NodeData::Token,
                        TextRange::new(text.len(), text.len()),
                    )),
                }),
                TextRange::new(0, text.len()),
            )),
            file_name: "test.ts".to_string(),
            text: text.to_string(),
            line_map,
            language_variant: crate::ast::LanguageVariant::Standard,
            script_kind: crate::ast::ScriptKind::Ts,
            comment_directives: Vec::new(),
            jsdoc_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
            has_lazy_jsdoc: true,
        })
    }

    #[test]
    fn line_and_character_basic() {
        let file = make_file("abc\ndef\nghi");
        let (line, col) = line_and_character(&file.line_map, &file.text, 5);
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn compact_format() {
        let file = make_file("abc\ndef");
        let diag = Diagnostic::new(
            Some(file),
            TextRange::new(5, 6),
            new_ad_hoc_message("Cannot find name 'x'."),
            vec![],
        );
        let s = format_diagnostic_compact(&diag);
        assert_eq!(s, "test.ts(2,2): error TS-1: Cannot find name 'x'.");
    }

    #[test]
    fn pretty_format_has_squiggle() {
        let file = make_file("let x = 1");
        let diag = Diagnostic::new(
            Some(file),
            TextRange::new(4, 5),
            new_ad_hoc_message("oops"),
            vec![],
        );
        let s = format_diagnostic_pretty(&diag);
        assert!(s.contains("test.ts:1:5 - error TS-1: oops"));
        assert!(s.contains("~"));
    }
}
