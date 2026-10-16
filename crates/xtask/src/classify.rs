use tsox_harness::baseline::{self, KnownDiffs, Outcome};

use crate::paths::SUITE;
use crate::run::Status;

pub fn classify(payload: &str, basename: &str, known_diffs: &KnownDiffs, accept: bool) -> (Status, String, bool) {
    use Status::*;
    let entries: Vec<serde_json::Value> = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(_) => return (Skip, "worker produced no parseable output".to_string(), false),
    };
    if entries.is_empty() {
        return (Skip, "worker produced no configurations".to_string(), false);
    }
    let stem = basename.trim_end_matches(".ts").trim_end_matches(".tsx");
    let ext = ".errors.txt";
    let mut overall = Skip;
    let mut failed_names: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    for entry in &entries {
        let suffix = entry["suffix"].as_str().unwrap_or("");
        let display = if suffix.is_empty() {
            basename.to_string()
        } else {
            format!("{basename}({suffix})")
        };
        let name = if suffix.is_empty() {
            stem.to_string()
        } else {
            format!("{stem}({suffix})")
        };
        let label = if suffix.is_empty() { "default" } else { suffix };

        if let Some(reason) = entry["skip"].as_str() {
            notes.push(format!("{label}: skip ({reason})"));
            continue;
        }
        let Some(actual) = entry["text"].as_str() else {
            notes.push(format!("{label}: malformed entry"));
            failed_names.push(display);
            overall = Fail;
            continue;
        };
        match baseline::compare(SUITE, &name, ext, actual) {
            Outcome::Passed => {
                notes.push(format!("{label}: pass"));
                if !matches!(overall, Fail | AcceptedDiff) {
                    overall = Pass;
                }
            }
            Outcome::Failed { .. } => {
                if known_diffs.contains(SUITE, &name, ext) {
                    notes.push(format!("{label}: known diff (triaged/accepted)"));
                    if !matches!(overall, Fail) {
                        overall = AcceptedDiff;
                    }
                } else if accept {
                    notes.push(format!("{label}: pass"));
                    if !matches!(overall, Fail | AcceptedDiff) {
                        overall = Pass;
                    }
                } else {
                    notes.push(format!("{label}: baseline mismatch"));
                    failed_names.push(display);
                    overall = Fail;
                }
            }
        }
    }
    let detail = if entries.len() == 1 {
        notes
            .into_iter()
            .next()
            .unwrap_or_default()
            .strip_prefix("default: ")
            .map(str::to_string)
            .filter(|s| s != "pass")
            .unwrap_or_default()
    } else {
        notes.join("; ")
    };
    (overall, detail, false)
}
