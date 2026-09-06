use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

pub const NO_CONTENT: &str = "<no content>";

#[derive(Default)]
pub struct BaselineOptions {

    pub subfolder: String,

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

fn baseline_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/baselines")
}

pub fn local_root() -> PathBuf {
    baseline_root().join("local")
}

pub fn reference_root() -> PathBuf {
    baseline_root().join("reference")
}

pub fn run(file_name: &str, actual: &str, opts: &BaselineOptions) -> Result<(), String> {
    let subfolder = if opts.is_submodule {
        format!("submodule/{}", opts.subfolder)
    } else {
        opts.subfolder.clone()
    };

    let local_path = local_root().join(&subfolder).join(file_name);
    let reference_path = reference_root().join(&subfolder).join(file_name);

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

fn summarize(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= 3 {
        s.to_string()
    } else {
        format!("{}... ({} lines total)", lines[..3].join("\n"), lines.len())
    }
}

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
