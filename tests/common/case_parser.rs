//! Parse a single TypeScript compiler test case file into virtual files and
//! compiler-option directives.
//!
//! Ports the core of tsgo's `internal/testrunner/test_case_parser.go`:
//! - `// @FileName: path.ts` splits one on-disk file into many virtual files.
//! - Other `// @Name: value` lines are compiler-option / harness directives.
//!
//! Unlike `fourslash::parse_test_content`, this does NOT strip fourslash
//! cursor/range markers (compiler baselines want the source unchanged) and it
//! ALSO extracts the `// @module`/`// @strict`/... directives.

use std::collections::HashMap;

/// One virtual file produced by splitting a test case on `// @filename:`.
#[derive(Debug, Clone)]
pub struct TestUnit {
    /// The filename as written in the directive (e.g. `a.ts`, `b.d.ts`).
    /// For a case with no `@filename` directive, this is the case's own basename.
    pub name: String,
    /// The verbatim source text of this file (directives stripped, markers kept).
    pub content: String,
}

/// Result of parsing a test case.
#[derive(Debug, Clone)]
pub struct ParsedCase {
    /// The virtual files in the case, in order of appearance.
    pub units: Vec<TestUnit>,
    /// Lowercased directive name → raw value, for every non-structural directive
    /// (everything except `@filename`/`@currentdirectory`/`@symlink`/`@link`).
    pub settings: HashMap<String, String>,
    /// Value of `// @currentdirectory:` if present.
    pub current_directory: Option<String>,
}

/// Regex matching `// @<word>: <value>` at the start of a line.
/// Captures (name, value). Mirrors Go's `optionRegex`.
fn parse_directive_line(line: &str) -> Option<(&str, &str)> {
    // Must start with `//` (optionally preceded by leading whitespace).
    let line = line.trim_start();
    let after_slashes = line.strip_prefix("//")?;
    // Optional whitespace between `//` and `@`.
    let after_at = after_slashes.trim_start().strip_prefix('@')?;
    // `@<word>` — word is [A-Za-z0-9_]+ terminated by optional ws then `:`.
    let colon = after_at.find(':')?;
    let name = after_at[..colon].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let value = after_at[colon + 1..].trim();
    Some((name, value))
}

/// Extract every `// @Name: value` directive in `content` into a lowercased map.
///
/// Structural directives (`filename`, `currentdirectory`, `symlink`, `link`)
/// and pure **harness** directives (which don't map to compiler options and
/// only affect baseline generation the runner doesn't do yet) are excluded.
/// The trailing `;` is trimmed from values (Go does the same).
pub fn extract_settings(content: &str) -> HashMap<String, String> {
    // Directives that affect the test harness, not the compiler. We drop them
    // rather than feeding them to `apply_test_settings` (which would otherwise
    // flag them as "unrecognized" and cause the case to be skipped).
    const HARNESS_DIRECTIVES: &[&str] = &[
        "notypesandsymbols",
        "noimplicitreferences",
        "fullemitpaths",
        "traceresolution",
        "baselinefile",
        "libfiles",
        "reportdiagnostics",
        "capturesuggestions",
        "typescriptversion",
        "emitthisfile",
        "currentdirectory",
        "symlink",
        "link",
        "filename",
    ];
    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some((name, value)) = parse_directive_line(line) {
            let lower = name.to_ascii_lowercase();
            if HARNESS_DIRECTIVES.contains(&lower.as_str()) {
                continue;
            }
            let v = value.trim_end_matches(';').trim().to_string();
            map.entry(lower).or_insert(v);
        }
    }
    map
}

/// Split `content` into virtual files on `// @filename:` boundaries.
///
/// - Lines before the first `@filename:` that are not comment/directive lines
///   cause a panic (matches Go's guard), UNLESS they're all comments/blank.
/// - `// @currentdirectory:` is surfaced separately (not part of `settings`).
/// - Source text is otherwise verbatim (no marker stripping).
pub fn split_units(content: &str, default_name: &str) -> ParsedCase {
    // A leading BOM is scanner trivia, not content — strip it so it never
    // occupies a line (Go's scanner skips it).
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut units: Vec<TestUnit> = Vec::new();
    let mut settings: HashMap<String, String> = HashMap::new();
    let mut current_directory: Option<String> = None;

    let mut current_name: Option<String> = None;
    let mut current_body = String::new();

    let flush = |units: &mut Vec<TestUnit>, name: &mut Option<String>, body: &mut String| {
        if let Some(n) = name.take() {
            units.push(TestUnit {
                name: n,
                content: std::mem::take(body),
            });
        }
    };

    for line in content.split_inclusive('\n') {
        let line_no_nl = line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some((name, value)) = parse_directive_line(line_no_nl) {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "filename" => {
                    // Flush the accumulated file.
                    if current_name.is_some() {
                        flush(&mut units, &mut current_name, &mut current_body);
                    }
                    // Content before the first @filename is normally comments
                    // / option directives / blank lines (we skip all of those
                    // above). Any stray bytes (e.g. a leading BOM) are also
                    // silently discarded rather than panicking — official cases
                    // rely on this, and panicking would abort the whole run.
                    current_name = Some(value.to_string());
                    current_body.clear();
                }
                "currentdirectory" => {
                    current_directory = Some(value.to_string());
                }
                _ => {
                    // A regular compiler-option directive. Don't emit it into
                    // file bodies; record in settings (lowercased).
                    settings
                        .entry(lower)
                        .or_insert_with(|| value.trim_end_matches(';').trim().to_string());
                }
            }
            continue;
        }

        // A non-directive line — it belongs to the current file body.
        // Lines seen before the first @filename (comments, blank lines, a
        // leading BOM) are simply discarded. Leading blank lines of each unit
        // are dropped too: Go's compiler-test parser inserts separators only
        // between already-accumulated lines (`Len() != 0`), so a blank line
        // before the unit's first content line never enters the virtual file
        // and official baselines number lines from the first content line.
        // Blank lines BETWEEN content lines are preserved.
        if current_name.is_some() {
            // Go's compiler-test parser drops LEADING truly-empty lines
            // (`Len() != 0` separator rule); a whitespace-only line is
            // content and keeps its line number.
            if current_body.is_empty() && line_no_nl.is_empty() {
                continue;
            }
            current_body.push_str(line);
        }
    }

    // Flush the last file, or — if no @filename was ever seen — use the whole
    // accumulated body as a single file under `default_name`.
    if current_name.is_some() {
        flush(&mut units, &mut current_name, &mut current_body);
    } else {
        // No @filename directive: the entire content (minus directives, which
        // were skipped above) is one file. We must re-extract the body without
        // directive lines, dropping leading blank lines like the multi-file
        // path (official baselines number from the first content line).
        let mut body = String::new();
        for line in content.split_inclusive('\n') {
            let line_no_nl = line.trim_end_matches('\n').trim_end_matches('\r');
            if parse_directive_line(line_no_nl).is_some() {
                continue;
            }
            if body.is_empty() && line_no_nl.is_empty() {
                continue;
            }
            body.push_str(line);
        }
        units.push(TestUnit {
            name: default_name.to_string(),
            content: body,
        });
    }

    ParsedCase {
        units,
        settings,
        current_directory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_module_and_strict_directives() {
        let src = "// @module: commonjs\n// @strict: true\n// @noImplicitAny: false\nlet x = 1;\n";
        let s = extract_settings(src);
        assert_eq!(s.get("module"), Some(&"commonjs".to_string()));
        assert_eq!(s.get("strict"), Some(&"true".to_string()));
        assert_eq!(s.get("noimplicitany"), Some(&"false".to_string()));
    }

    #[test]
    fn split_single_file_no_directive() {
        let src = "// @module: commonjs\nlet x = 1;\n";
        let parsed = split_units(src, "a.ts");
        assert_eq!(parsed.units.len(), 1);
        assert_eq!(parsed.units[0].name, "a.ts");
        assert_eq!(parsed.units[0].content, "let x = 1;\n");
        assert_eq!(parsed.settings.get("module"), Some(&"commonjs".to_string()));
    }

    #[test]
    fn split_multi_file() {
        let src = "\
// @filename: a.ts
export const a = 1;
// @filename: b.ts
import { a } from './a';
console.log(a);
";
        let parsed = split_units(src, "main.ts");
        assert_eq!(parsed.units.len(), 2);
        assert_eq!(parsed.units[0].name, "a.ts");
        assert_eq!(parsed.units[1].name, "b.ts");
        assert!(parsed.units[0].content.contains("export const a"));
        assert!(parsed.units[1].content.contains("import"));
    }

    #[test]
    fn current_directory_directive_captured() {
        let src = "// @currentdirectory: /proj/sub\n// @filename: a.ts\nlet x = 1;\n";
        let parsed = split_units(src, "a.ts");
        assert_eq!(parsed.current_directory.as_deref(), Some("/proj/sub"));
    }
}
