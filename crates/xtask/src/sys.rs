use std::os::unix::io::AsRawFd;

use anyhow::{Context, Result};

pub struct ExclusiveLock {
    // 持有 fd 即持有 flock,析构(close)即放锁:字段本身无需被读
    #[allow(dead_code)]
    file: std::fs::File,
}

impl ExclusiveLock {
    pub fn acquire(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)
            .with_context(|| format!("open lock {}", path.display()))?;
        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("flock {}", path.display()));
        }
        Ok(Self { file })
    }
}

pub fn disk_free_bytes(path: &std::path::Path) -> Result<u64> {
    let c = std::ffi::CString::new(path.to_str().context("non-utf8 path")?)?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c.as_ptr(), &mut st) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("statvfs");
    }
    Ok(st.f_bavail as u64 * st.f_frsize as u64)
}

pub fn set_process_limits(rlimit_as_bytes: u64, nice: i32) {
    unsafe {
        let rl = libc::rlimit {
            rlim_cur: rlimit_as_bytes,
            rlim_max: rlimit_as_bytes,
        };
        libc::setrlimit(libc::RLIMIT_AS, &rl);
        libc::nice(nice);
    }
}
