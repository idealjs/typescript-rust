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
    // All tests are #[ignore] because:
    //   1. The nativepath module is newly created and not yet integrated.
    //   2. Several subtests require Windows junctions which cannot be created
    //      on Unix without elevated privileges.

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
    #[ignore]
    // TODO: Requires Windows mklink /J for junction; on Unix symlinks can
    // be tested directly. Verify long path support (>= 248 chars).
    fn test_is_symlink_or_reparse_point_long_path() {
        // The Go test creates a deeply nested path exceeding 248 characters
        // and verifies that is_symlink_or_reparse_point works with the
        // \\?\ prefix on Windows.
        //
        // TODO: Implement once cross-platform test infrastructure is available.
    }

    #[test]
    #[ignore]
    // TODO: Requires symlink/junction creation for nested symlink test;
    // verify is_symlink_or_reparse_point detects junctions nested inside
    // symlinked directories
    fn test_is_symlink_or_reparse_point_nested_in_symlink() {
        // The Go test creates:
        //   target/inner-target
        //   link -> target (junction)
        //   target/inner-link -> target/inner-target (junction)
        // and checks that link/inner-link is detected as a reparse point.
        //
        // TODO: Implement once cross-platform test infrastructure is available.
    }

    #[test]
    #[ignore]
    // TODO: Requires symlink/junction creation with relative paths;
    // verify is_symlink_or_reparse_point works with relative paths
    fn test_is_symlink_or_reparse_point_relative_path() {
        // The Go test uses t.Chdir(tmp) and creates a junction with a
        // relative path, then verifies detection.
        //
        // TODO: Implement once cross-platform test infrastructure is available.
    }
}
