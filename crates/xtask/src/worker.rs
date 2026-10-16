use std::path::PathBuf;

use anyhow::{anyhow, Result};

pub fn run(case: Option<PathBuf>, out: Option<PathBuf>, resident: bool) -> Result<()> {
    if resident {
        tsox_harness::worker::run_resident();
        return Ok(());
    }
    let (Some(case), Some(out)) = (case, out) else {
        return Err(anyhow!("worker requires --resident or --case/--out"));
    };
    if !case.is_absolute() {
        return Err(anyhow!("worker case path must be absolute: {}", case.display()));
    }
    tsox_harness::worker::run_worker(&case, &out);
    Ok(())
}
