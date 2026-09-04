use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const NO_CONTENT: &str = "<no content>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    Go,
    Upstream,
}

pub fn flavor() -> Flavor {
    match std::env::var("TSOX_BASELINE_FLAVOR").as_deref() {
        Ok("upstream") => Flavor::Upstream,
        _ => Flavor::Go,
    }
}

pub fn reference_root() -> &'static str {
    match flavor() {
        Flavor::Go => "tests/baselines/reference-go",
        Flavor::Upstream => "tests/baselines/reference",
    }
}

pub const LOCAL_ROOT: &str = "tests/baselines/local";

pub fn flat_segment(text: &str) -> &str {
    match text.find("\n==== ") {
        Some(i) => &text[..=i],
        None => text,
    }
}

pub fn accept_mode() -> bool {
    std::env::var("TSOX_BASELINE_ACCEPT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[derive(Debug)]
pub enum Outcome {

    Passed,

    Failed {
        #[allow(dead_code)]
        local_path: PathBuf,
        #[allow(dead_code)]
        reference_path: PathBuf,
        #[allow(dead_code)]
        message: String,
    },
}

pub fn compare(subfolder: &str, name: &str, ext: &str, actual: &str) -> Outcome {
    let reference_path = Path::new(reference_root())
        .join(subfolder)
        .join(format!("{name}{ext}"));
    let local_path = Path::new(LOCAL_ROOT)
        .join(subfolder)
        .join(format!("{name}{ext}"));

    if accept_mode() {

        fs::create_dir_all(reference_path.parent().unwrap()).ok();
        if actual == NO_CONTENT {

            fs::remove_file(&reference_path).ok();
        } else {
            fs::write(&reference_path, &actual).ok();
        }
        return Outcome::Passed;
    }

    let expected = fs::read_to_string(&reference_path)
        .map(|t| flat_segment(&t).replace("\r\n", "\n").trim_end().to_string())
        .unwrap_or_else(|_| NO_CONTENT.to_string());
    let reference_existed = reference_path.is_file();
    let actual = actual.trim_end();
    let actual = actual.to_string();

    if actual == expected {
        return Outcome::Passed;
    }

    fs::create_dir_all(local_path.parent().unwrap()).ok();
    if actual == NO_CONTENT {

        let delete_marker = local_path.with_extension(format!(
            "{}.delete",
            local_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
        ));
        fs::write(&delete_marker, "").ok();
    } else {
        fs::write(&local_path, &actual).ok();
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

pub struct KnownDiffs {
    entries: HashSet<String>,
}

impl KnownDiffs {

    pub fn load() -> Self {
        let mut entries = HashSet::new();

        let ledger = match flavor() {
            Flavor::Go => "triaged-go.txt",
            Flavor::Upstream => "triaged.txt",
        };
        for fname in ["accepted.txt", ledger] {
            let p = Path::new(reference_root()).join(fname);
            for e in load_list(&p) {
                entries.insert(e);
            }
        }
        Self { entries }
    }

    pub fn contains(&self, subfolder: &str, name: &str, ext: &str) -> bool {
        self.entries.contains(&format!("{subfolder}/{name}{ext}"))
    }
}
