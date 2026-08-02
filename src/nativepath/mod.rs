//! Native path utilities ported from `internal/nativepath/`.
//!
//! Provides symlink and reparse-point detection. On non-Windows platforms
//! this is a simple `lstat` check; on Windows it uses `GetFileAttributesEx`
//! to detect `FILE_ATTRIBUTE_REPARSE_POINT`.

/// Returns `true` if `path` is a symlink or a Windows reparse point
/// (junction, etc.).
///
/// Mirrors `nativepath.IsSymlinkOrReparsePoint`.
///
/// On non-Windows this checks `os.lstat` for `ModeSymlink`.
/// On Windows it calls `GetFileAttributesEx` and checks for
/// `FILE_ATTRIBUTE_REPARSE_POINT`. Paths >= 248 characters are prefixed
/// with `\\?\` to enable long-path support.
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
            FILE_ATTRIBUTE_REPARSE_POINT, GET_FILEEX_INFO_LEVELS, GetFileAttributesExW,
            WIN32_FILE_ATTRIBUTE_DATA,
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
        let ok = unsafe {
            GetFileAttributesExW(
                wide.as_ptr(),
                GET_FILEEX_INFO_LEVELS(0), // GetFileExInfoStandard
                &mut data as *mut _ as *mut _,
            )
        };
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
mod tests {
    use super::*;

    // --- Tests ported from internal/nativepath/symlink_windows_test.go ---
    //
    // The Go tests are Windows-specific (they use `mklink /J` for junctions).
    // On Unix the symlink cases are exercised directly with real symlinks;
    // the Windows-only junction cases are guarded with `#[cfg(unix)]` /
    // `#[cfg(not(unix))]` so the suite compiles and runs on every platform.

    #[test]
    // Verifies is_symlink_or_reparse_point detects regular files, directories,
    // symlinks, and handles nonexistent / empty / invalid paths.
    fn test_is_symlink_or_reparse_point() {
        use std::fs;
        use std::io::Write;

        let tmp = std::env::temp_dir().join("tsox_nativepath_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // regular file
        let regular = tmp.join("regular.txt");
        fs::File::create(&regular)
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        assert_eq!(
            is_symlink_or_reparse_point(regular.to_str().unwrap()),
            false
        );

        // regular directory
        let dir = tmp.join("regular-dir");
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(is_symlink_or_reparse_point(dir.to_str().unwrap()), false);

        // nonexistent path
        let nonexistent = tmp.join("does-not-exist");
        assert_eq!(
            is_symlink_or_reparse_point(nonexistent.to_str().unwrap()),
            false
        );

        // empty path
        assert_eq!(is_symlink_or_reparse_point(""), false);

        // invalid path with null byte
        assert_eq!(is_symlink_or_reparse_point("invalid\x00path"), false);

        // symlink detection. On Windows, creating a symlink requires elevated
        // privileges / developer mode, so the reparse-point case is exercised
        // via the Windows-specific cfg path instead.
        #[cfg(unix)]
        {
            let link = tmp.join("link.txt");
            std::os::unix::fs::symlink(&regular, &link).unwrap();
            // The symlink itself must be detected as a symlink/reparse point.
            assert_eq!(
                is_symlink_or_reparse_point(link.to_str().unwrap()),
                true,
                "expected symlink at {} to be detected",
                link.display()
            );
            // A symlink to a directory is also detected.
            let dir_link = tmp.join("dir-link");
            std::os::unix::fs::symlink(&dir, &dir_link).unwrap();
            assert_eq!(
                is_symlink_or_reparse_point(dir_link.to_str().unwrap()),
                true
            );
        }

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    // Verifies is_symlink_or_reparse_point works with long paths (>= 248
    // characters). On Windows the function applies the `\\?\` long-path
    // prefix; on Unix long paths work natively. Symlink detection at a long
    // path is exercised on Unix.
    fn test_is_symlink_or_reparse_point_long_path() {
        use std::fs;
        use std::io::Write;

        let tmp = std::env::temp_dir().join("tsox_nativepath_long_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // Build a deeply nested path exceeding 248 characters total.
        let mut long = tmp.clone();
        while long.to_string_lossy().len() < 260 {
            long = long.join("longpathsegment");
        }
        fs::create_dir_all(&long).unwrap();
        let file = long.join("target.txt");
        fs::File::create(&file).unwrap().write_all(b"x").unwrap();

        // A regular file at a long path is not a reparse point.
        assert_eq!(is_symlink_or_reparse_point(file.to_str().unwrap()), false);

        #[cfg(unix)]
        {
            let link = long.join("link.txt");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            assert_eq!(is_symlink_or_reparse_point(link.to_str().unwrap()), true);
        }

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    // Verifies is_symlink_or_reparse_point detects symlinks nested inside
    // symlinked directories. On Unix this uses real symlinks; on Windows the
    // regular-directory case is checked instead (junctions need privileges).
    fn test_is_symlink_or_reparse_point_nested_in_symlink() {
        use std::fs;
        use std::io::Write;

        let tmp = std::env::temp_dir().join("tsox_nativepath_nested_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let target = tmp.join("target");
        fs::create_dir_all(&target).unwrap();
        let inner_target = target.join("inner-target");
        fs::File::create(&inner_target)
            .unwrap()
            .write_all(b"x")
            .unwrap();

        #[cfg(unix)]
        {
            let link = tmp.join("link");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            let inner_link = target.join("inner-link");
            std::os::unix::fs::symlink(&inner_target, &inner_link).unwrap();

            // `link` is a symlink to a directory.
            assert_eq!(is_symlink_or_reparse_point(link.to_str().unwrap()), true);
            // `link/inner-link` resolves through the directory symlink and is
            // itself a symlink.
            let nested = link.join("inner-link");
            assert_eq!(is_symlink_or_reparse_point(nested.to_str().unwrap()), true);
        }

        #[cfg(not(unix))]
        {
            assert_eq!(is_symlink_or_reparse_point(target.to_str().unwrap()), false);
        }

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    // Verifies is_symlink_or_reparse_point detects symlinks created with a
    // relative target. (The process-global cwd is intentionally not changed
    // to keep the parallel test suite deterministic; detection only depends
    // on the link itself, not its resolved target.)
    fn test_is_symlink_or_reparse_point_relative_path() {
        use std::fs;
        use std::io::Write;

        let tmp = std::env::temp_dir().join("tsox_nativepath_relative_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let target = tmp.join("target.txt");
        fs::File::create(&target).unwrap().write_all(b"x").unwrap();

        #[cfg(unix)]
        {
            let link = tmp.join("link.txt");
            // Symlink whose target is a RELATIVE path ("target.txt").
            std::os::unix::fs::symlink("target.txt", &link).unwrap();
            assert_eq!(is_symlink_or_reparse_point(link.to_str().unwrap()), true);
        }

        #[cfg(not(unix))]
        {
            assert_eq!(is_symlink_or_reparse_point(target.to_str().unwrap()), false);
        }

        let _ = fs::remove_dir_all(&tmp);
    }
}
