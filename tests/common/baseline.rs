//! Baseline (snapshot) comparison workflow.
//!
//! Ports a simplified subset of tsgo's `internal/testutil/baseline`:
//! - `reference/` holds committed "standard answer" snapshots.
//! - `local/` holds the current run's actual output (gitignored).
//! - On mismatch the actual is written to `local/` and the test fails (unless
//!   the mismatching baseline is listed in `accepted.txt` / `triaged.txt`).
//! - `TSOX_BASELINE_ACCEPT=1` writes the actual over the reference instead of
//!   comparing (mirrors `hereby baseline-accept`).
//!
//! Format note: unlike tsgo, this first cut does NOT reproduce the CRLF /
//! `==== file (N errors) ====` / squiggle format. Each errors baseline is one
//! diagnostic per line via `format_diagnostic_compact`. Aligning with tsgo's
//! exact byte format is deferred to a later phase.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical "no baseline content" marker, matching tsgo's `baseline.NoContent`.
pub const NO_CONTENT: &str = "<no content>";

/// Root directory of committed reference baselines.
pub const REFERENCE_ROOT: &str = "tests/baselines/reference";
/// Root directory of per-run actual output (gitignored).
pub const LOCAL_ROOT: &str = "tests/baselines/local";

/// Whether the runner is in "accept" mode (write actual over reference).
pub fn accept_mode() -> bool {
    std::env::var("TSOX_BASELINE_ACCEPT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Outcome of a single baseline comparison.
#[derive(Debug)]
pub enum Outcome {
    /// Actual matched the reference (or was newly accepted).
    Passed,
    /// Mismatch; `local_path` has the actual written, `message` is the diff.
    Failed {
        #[allow(dead_code)]
        local_path: PathBuf,
        #[allow(dead_code)]
        reference_path: PathBuf,
        #[allow(dead_code)]
        message: String,
    },
}

/// Compare `actual` against the reference baseline named by `subfolder`/`name`,
/// or — in accept mode — overwrite the reference with `actual`.
///
/// `name` is the baseline filename without extension, e.g. `"foo"`; `ext` is
/// `".errors.txt"`. The reference is read from
/// `REFERENCE_ROOT/<subfolder>/<name><ext>`; on mismatch the actual is written
/// to `LOCAL_ROOT/<subfolder>/<name><ext>`.
pub fn compare(subfolder: &str, name: &str, ext: &str, actual: &str) -> Outcome {
    let reference_path = Path::new(REFERENCE_ROOT)
        .join(subfolder)
        .join(format!("{name}{ext}"));
    let local_path = Path::new(LOCAL_ROOT)
        .join(subfolder)
        .join(format!("{name}{ext}"));

    if accept_mode() {
        // Write actual over the reference (creating parent dirs as needed).
        fs::create_dir_all(reference_path.parent().unwrap()).ok();
        if actual == NO_CONTENT {
            // Accepting a deletion: remove the reference file if present.
            fs::remove_file(&reference_path).ok();
        } else {
            fs::write(&reference_path, actual).ok();
        }
        return Outcome::Passed;
    }

    // Normal compare mode.
    let expected = fs::read_to_string(&reference_path).unwrap_or_else(|_| NO_CONTENT.to_string());
    let reference_existed = reference_path.is_file();

    if actual == expected {
        return Outcome::Passed;
    }
    // Mismatch (includes "reference existed but actual is NO_CONTENT" deletion
    // case, and "reference absent but actual has content" new-baseline case).
    fs::create_dir_all(local_path.parent().unwrap()).ok();
    if actual == NO_CONTENT {
        // Write a `.delete` marker instead of an empty file, mirroring tsgo.
        let delete_marker = local_path.with_extension(format!(
            "{}.delete",
            local_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
        ));
        fs::write(&delete_marker, "").ok();
    } else {
        fs::write(&local_path, actual).ok();
    }

    let kind = if !reference_existed {
        "new baseline created"
    } else if actual == NO_CONTENT {
        "baseline deleted"
    } else {
        "baseline changed"
    };
    let message = format!(
        "Baseline {kind}: {name}{ext} ({subfolder}).\n\
         Run with TSOX_BASELINE_ACCEPT=1 to accept the new output.\n\
         --- reference ({}) ---\n{}\n\
         --- actual ---\n{}",
        if reference_existed {
            "exists"
        } else {
            "missing"
        },
        expected,
        actual,
    );
    Outcome::Failed {
        local_path,
        reference_path,
        message,
    }
}

/// Load an `accepted.txt` / `triaged.txt`-style list.
///
/// Format: one entry per line; lines starting with `#` (incl. `## group ##`
/// headers) and blank lines are ignored. Entries are paths relative to
/// `REFERENCE_ROOT`, e.g. `compiler/foo.errors.txt`. Returns the set of entries.
pub fn load_list(path: &Path) -> HashSet<String> {
    let mut set = HashSet::new();
    let Ok(text) = fs::read_to_string(path) else {
        return set;
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        set.insert(trimmed.to_string());
    }
    set
}

/// A combined accepted+triaged lookup, keyed by `"<subfolder>/<name><ext>"`.
pub struct KnownDiffs {
    entries: HashSet<String>,
}

impl KnownDiffs {
    /// Load from `accepted.txt` and `triaged.txt` under `REFERENCE_ROOT`.
    pub fn load() -> Self {
        let mut entries = HashSet::new();
        for fname in ["accepted.txt", "triaged.txt"] {
            let p = Path::new(REFERENCE_ROOT).join(fname);
            for e in load_list(&p) {
                entries.insert(e);
            }
        }
        Self { entries }
    }

    /// Returns true if `<subfolder>/<name><ext>` is a known/accepted diff.
    pub fn contains(&self, subfolder: &str, name: &str, ext: &str) -> bool {
        self.entries.contains(&format!("{subfolder}/{name}{ext}"))
    }
}
