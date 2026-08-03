//! Stack trace sanitizer (1:1 port of Go's `internal/lsp/stack_sanitizer.go`).

#![allow(dead_code)]

use regex::Regex;
use std::sync::OnceLock;

static GENERIC_SECRET_REGEX: OnceLock<Regex> = OnceLock::new();

fn generic_secret_regex() -> &'static Regex {
    GENERIC_SECRET_REGEX
        .get_or_init(|| Regex::new(r"(?i)(key|token|signature|sig|pwd)([(\[.|])").unwrap())
}

/// Inserts `X_X` after trigger keywords that VS Code's telemetry pipeline
/// would redact.
///
/// Go: `func defeatGenericSecretRegex(s string) string`.
pub fn defeat_generic_secret_regex(s: &str) -> String {
    generic_secret_regex()
        .replace_all(s, "${1}X_X${2}")
        .to_string()
}

/// Sanitizes a Go panic stack trace, redacting frames from external modules
/// and trimming module paths to be relative to `typescript-go/internal`.
///
/// Go: `func sanitizeStackTrace(stack string) string`.
pub fn sanitize_stack_trace(stack: &str) -> String {
    let start_marker = "runtime/debug.Stack()";
    let start_index = match stack.find(start_marker) {
        Some(idx) => idx,
        None => return String::new(),
    };

    let stack = &stack[start_index..];
    let mut result = String::new();

    for (line_num, line) in stack.lines().enumerate() {
        if line_num > 0 {
            result.push('\n');
        }

        // Preserve leading whitespace.
        let trimmed_start = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
        let whitespace = &line[..line.len() - trimmed_start.len()];
        result.push_str(whitespace);

        let line = trimmed_start;

        if let Some(our_module_index) = line.find("typescript-go/internal") {
            let line = &line[our_module_index..];
            write_sanitized_module_or_path(line, &mut result);
        } else {
            result.push_str("(REDACTED FRAME)");
        }
    }

    defeat_generic_secret_regex(&result)
}

/// Writes a sanitized module path or function call to `result`.
///
/// Go: `func writeSanitizedModuleOrPath(line string, result *strings.Builder)`.
fn write_sanitized_module_or_path(line: &str, result: &mut String) {
    let line = line.trim();

    // Strip ` +0x...` or ` in goroutine ...` suffixes.
    let line = if let Some(idx) = line.find(" +0x") {
        &line[..idx]
    } else if let Some(idx) = line.rfind(" in goroutine ") {
        &line[..idx]
    } else {
        line
    };

    for (segment_index, segment) in line.split('/').enumerate() {
        if segment_index > 0 {
            result.push_str("|>");
        }

        if segment.ends_with(')') {
            if let Some(open_paren) = segment.rfind('(') {
                result.push_str(&segment[..open_paren]);
                result.push_str("()");
                continue;
            } else {
                result.push_str("???");
                continue;
            }
        }

        result.push_str(segment);
    }
}
