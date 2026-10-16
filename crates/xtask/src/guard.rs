use std::time::{Duration, Instant};

use anyhow::{bail, Result};

use crate::paths::DISK_MIN_FREE_BYTES;
use crate::sys;

pub struct Deadline {
    limit: Duration,
    started: Instant,
}

impl Deadline {
    pub fn from_secs(secs: u64) -> Self {
        Self {
            limit: Duration::from_secs(secs),
            started: Instant::now(),
        }
    }

    pub fn exceeded(&self) -> bool {
        self.started.elapsed() >= self.limit
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.started.elapsed().as_secs()
    }
}

pub fn disk_guard() -> Result<()> {
    let free = sys::disk_free_bytes(std::path::Path::new("/"))?;
    if free < DISK_MIN_FREE_BYTES {
        bail!(
            "disk free {:.1}GiB < 15GiB — refusing to run; clean up and retry",
            free as f64 / (1024.0 * 1024.0 * 1024.0)
        );
    }
    Ok(())
}
