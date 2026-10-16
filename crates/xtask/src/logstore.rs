use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::paths::RUNS_DIR;

pub struct RunStore {
    pub run_id: String,
    pub dir: PathBuf,
}

impl RunStore {
    pub fn create(keep_days: u32, keep_runs: u32) -> Result<Self> {
        std::fs::create_dir_all(RUNS_DIR).context("create runs dir")?;
        sweep(keep_days, keep_runs)?;
        let run_id = chrono_now_compact();
        let dir = Path::new(RUNS_DIR).join(&run_id);
        std::fs::create_dir_all(dir.join("shards"))?;
        for sub in ["fn-trace", "types"] {
            std::fs::create_dir_all(dir.join(sub))?;
        }
        let link = Path::new(RUNS_DIR).join("latest");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&dir, &link).ok();
        Ok(Self { run_id, dir })
    }

    pub fn trace_dir(&self) -> PathBuf {
        self.dir.join("fn-trace")
    }

    pub fn types_dir(&self) -> PathBuf {
        self.dir.join("types")
    }

    pub fn run_log(&self) -> PathBuf {
        self.dir.join("run.log")
    }

    pub fn verdict_json(&self) -> PathBuf {
        self.dir.join("verdict.json")
    }

    pub fn shards_dir(&self) -> PathBuf {
        self.dir.join("shards")
    }
}

fn chrono_now_compact() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs() as i64;
    let (y, mo, d, h, mi, s) = civil_from_unix(now);
    format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
}

fn civil_from_unix(secs: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d, h, mi, s)
}

fn run_id_timestamp(name: &str) -> Option<i64> {
    if name.len() != 15 || name.as_bytes().get(8) != Some(&b'-') {
        return None;
    }
    let digits: Result<Vec<i64>, _> = name
        .chars()
        .filter(|c| *c != '-')
        .map(|c| c.to_digit(10).map(|v| v as i64).ok_or(()))
        .collect();
    let d = digits.ok()?;
    let (y, mo, da) = (d[0] * 1000 + d[1] * 100 + d[2] * 10 + d[3], d[4] * 10 + d[5], d[6] * 10 + d[7]);
    let (h, mi, s) = (d[8] * 10 + d[9], d[10] * 10 + d[11], d[12] * 10 + d[13]);
    Some(unix_from_civil(y, mo, da, h, mi, s))
}

fn unix_from_civil(y: i64, m: i64, d: i64, h: i64, mi: i64, s: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) * 86_400 + h * 3600 + mi * 60 + s
}

pub fn sweep(keep_days: u32, keep_runs: u32) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let cutoff = now - keep_days as i64 * 86_400;
    let mut runs: Vec<(String, i64)> = std::fs::read_dir(RUNS_DIR)?
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            run_id_timestamp(&name).map(|ts| (name, ts))
        })
        .collect();
    runs.sort_by(|a, b| b.1.cmp(&a.1));
    for (i, (name, ts)) in runs.iter().enumerate() {
        if *ts < cutoff || i >= keep_runs as usize {
            let _ = std::fs::remove_dir_all(Path::new(RUNS_DIR).join(name));
        }
    }
    Ok(())
}

pub fn clean_all() -> Result<usize> {
    let mut removed = 0;
    for e in std::fs::read_dir(RUNS_DIR)?.flatten() {
        let p = e.path();
        let is_link = std::fs::symlink_metadata(&p)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        let r = if is_link {
            std::fs::remove_file(&p)
        } else {
            std::fs::remove_dir_all(&p)
        };
        if r.is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}
