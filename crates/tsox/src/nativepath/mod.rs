pub fn is_symlink_or_reparse_point(path: &str) -> bool {
    #[cfg(unix)]
    {
        match std::fs::symlink_metadata(path) {
            Ok(meta) => meta.file_type().is_symlink(),
            Err(_) => false,
        }
    }
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_REPARSE_POINT, GetFileAttributesExW, WIN32_FILE_ATTRIBUTE_DATA,
        };

        let path = if path.len() >= 248 {
            format!("\\\\?\\{path}")
        } else {
            path.to_string()
        };

        let wide: Vec<u16> = OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut data: WIN32_FILE_ATTRIBUTE_DATA = unsafe { std::mem::zeroed() };

        let ok = unsafe { GetFileAttributesExW(wide.as_ptr(), 0, &mut data as *mut _ as *mut _) };
        if ok == 0 {
            return false;
        }
        data.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        false
    }
}

#[cfg(test)]
mod tests;
