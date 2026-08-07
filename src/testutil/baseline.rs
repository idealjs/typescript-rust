//! Baseline comparison — ported from typescript-go's `testutil/baseline/baseline.go`.
//!
//! Compares actual compiler output against reference baseline files. If the
//! output differs, generates a diff file for manual inspection.
//!
//! Usage:
//! ```text
//! baseline::run("test_name.errors.txt", actual_output, "compiler")
//! ```
//!
//! Workflow:
//! 1. Tests produce output → written to `testdata/baselines/local/`
//! 2. Compared against `testdata/baselines/reference/`
//! 3. If mismatch → test fails with diff
//! 4. `cargo run --bin baseline-accept` copies local → reference

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// The "no content" placeholder used when a baseline file would be empty.
pub const NO_CONTENT: &str = "<no content>";

/// Options controlling baseline comparison.
#[derive(Default)]
pub struct BaselineOptions {
    /// Subfolder under `local/` and `reference/` (e.g. "compiler", "conformance").
    pub subfolder: String,
    /// If true, output goes under `submodule/` prefix (for TypeScript submodule tests).
    pub is_submodule: bool,
}

impl BaselineOptions {
    pub fn new(subfolder: &str) -> Self {
        Self {
            subfolder: subfolder.to_string(),
            is_submodule: false,
        }
    }
}

/// Get the root directory for baselines, derived from CARGO_MANIFEST_DIR.
fn baseline_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/baselines")
}

/// Get the local output directory (where test results are written).
pub fn local_root() -> PathBuf {
    baseline_root().join("local")
}

/// Get the reference directory (the "golden" expected outputs).
pub fn reference_root() -> PathBuf {
    baseline_root().join("reference")
}

/// Run a baseline comparison: compare `actual` output against the reference
/// baseline file. If they differ, write the actual output to the local
/// directory and return an error message describing the diff.
///
/// In test mode (cfg(test)), this panics on mismatch. In accept mode
/// (feature `accept`), it always writes to reference and succeeds.
pub fn run(file_name: &str, actual: &str, opts: &BaselineOptions) -> Result<(), String> {
    let subfolder = if opts.is_submodule {
        format!("submodule/{}", opts.subfolder)
    } else {
        opts.subfolder.clone()
    };

    let local_path = local_root().join(&subfolder).join(file_name);
    let reference_path = reference_root().join(&subfolder).join(file_name);

    // Always write to local (for inspection and accept workflow).
    if let Some(parent) = local_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&local_path, actual);

    #[cfg(feature = "accept")]
    {
        if let Some(parent) = reference_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&reference_path, actual);
        return Ok(());
    }

    #[cfg(not(feature = "accept"))]
    {
        let expected =
            fs::read_to_string(&reference_path).unwrap_or_else(|_| NO_CONTENT.to_string());

        let actual_normalized = if actual.is_empty() {
            NO_CONTENT.to_string()
        } else {
            actual.to_string()
        };

        if expected.trim_end() == actual_normalized.trim_end() {
            Ok(())
        } else {
            Err(format!(
                "Baseline mismatch: {}\n\
                 Expected: {}\n\
                 Actual:   {}\n\
                 Reference: {}\n\
                 Local:     {}\n\
                 Run `cargo run --bin baseline-accept` to accept the new output.",
                file_name,
                summarize(&expected),
                summarize(&actual_normalized),
                reference_path.display(),
                local_path.display(),
            ))
        }
    }
}

/// A short summary of a string for error messages.
fn summarize(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= 3 {
        s.to_string()
    } else {
        format!("{}... ({} lines total)", lines[..3].join("\n"), lines.len())
    }
}

/// Enumerate all test files matching a pattern in a directory.
pub fn enumerate_test_files(dir: &Path, pattern: &regex::Regex) -> Vec<String> {
    let mut files = Vec::new();
    enumerate_recursive(dir, dir, pattern, &mut files);
    files.sort();
    files
}

fn enumerate_recursive(base: &Path, current: &Path, pattern: &Regex, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                enumerate_recursive(base, &path, pattern, files);
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if pattern.is_match(name) {
                    if let Ok(rel) = path.strip_prefix(base) {
                        files.push(rel.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
}
