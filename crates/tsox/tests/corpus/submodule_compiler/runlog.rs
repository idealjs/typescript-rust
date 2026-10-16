use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tsox_harness::baseline;

pub struct RunLog {
    path: PathBuf,
    file: Option<Mutex<fs::File>>,
    quiet: bool,
}

impl RunLog {
    pub fn for_suite(suite: &str) -> Self {
        let log_name = if suite == "compiler" {
            "submodule_run.log".to_string()
        } else {
            format!("submodule_{suite}_run.log")
        };
        let path = PathBuf::from(baseline::LOCAL_ROOT).join(log_name);
        let quiet = std::env::var("TSOX_SUBMODULE_QUIET")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let file = fs::create_dir_all(path.parent().unwrap())
            .ok()
            .and_then(|_| fs::File::create(&path).ok())
            .map(Mutex::new);
        if file.is_none() {
            eprintln!(
                "[submodule_compiler] note: cannot write run log {}",
                path.display()
            );
        }
        Self { path, file, quiet }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn line(&self, msg: &str) {
        use std::io::Write;
        if !self.quiet {
            let mut err = std::io::stderr().lock();
            let _ = writeln!(err, "{msg}");
        }
        if let Some(f) = &self.file {
            let _ = writeln!(f.lock().unwrap(), "{msg}");
        }
    }
}
