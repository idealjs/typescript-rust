use std::io;

#[cfg(unix)]
pub fn ignoring_eintr<T, F: FnMut() -> io::Result<T>>(mut f: F) -> io::Result<T> {
    loop {
        match f() {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            other => return other,
        }
    }
}

#[cfg(target_os = "linux")]
pub fn realpath(path: &str) -> io::Result<String> {
    use std::os::fd::AsRawFd;

    if !has_proc_self_fd() {
        return eval_symlinks(path);
    }

    let file = ignoring_eintr(|| std::fs::File::open(path))?;
    let fd = file.as_raw_fd();
    let proc_path = format!("/proc/self/fd/{fd}");
    let resolved = ignoring_eintr(|| std::fs::read_link(&proc_path))?;
    drop(file);
    return Ok(resolved.into_os_string().into_string().unwrap_or_default());

    fn has_proc_self_fd() -> bool {
        !std::fs::metadata("/proc/self/fd/").is_err()
    }

    fn eval_symlinks(path: &str) -> io::Result<String> {
        std::fs::canonicalize(path).map(|p| p.into_os_string().into_string().unwrap_or_default())
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
pub fn realpath(path: &str) -> io::Result<String> {
    std::fs::canonicalize(path).map(|p| p.into_os_string().into_string().unwrap_or_default())
}

#[cfg(windows)]
pub fn realpath(path: &str) -> io::Result<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::Storage::FileSystem::GetFinalPathNameByHandleW;

    let handle: *mut core::ffi::c_void = if path.len() < 248 {
        open_metadata(path)?
    } else {
        let f = std::fs::File::open(path)?;
        f.as_raw_handle() as *mut core::ffi::c_void
    };

    const VOLUME_NAME_DOS: u32 = 0;
    let mut buf: Vec<u16> = vec![0u16; 310];
    loop {
        let n = unsafe {
            GetFinalPathNameByHandleW(handle, buf.as_mut_ptr(), buf.len() as u32, VOLUME_NAME_DOS)
        };
        if n == 0 {
            return Err(io::Error::last_os_error());
        }
        if (n as usize) < buf.len() {
            buf.truncate(n as usize);
            break;
        }
        buf.resize(n as usize, 0);
    }

    let mut s = String::from_utf16_lossy(&buf);
    if s.len() > 4 && s.starts_with("\\\\?\\") {
        s = s[4..].to_string();
        if s.len() > 3 && s.starts_with("UNC") {
            return Ok(format!("\\{}", &s[3..]));
        }
        return Ok(s);
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("GetFinalPathNameByHandle returned unexpected path: {s}"),
    ))
}

#[cfg(windows)]
fn open_metadata(path: &str) -> io::Result<*mut core::ffi::c_void> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    let wide: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut() as _,
        )
    };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok(handle)
}
