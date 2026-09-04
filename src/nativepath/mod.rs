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
mod tests {
    use super::*;

    #[test]

    fn test_is_symlink_or_reparse_point() {
        use std::fs;
        use std::io::Write;

        let tmp = std::env::temp_dir().join("tsox_nativepath_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let regular = tmp.join("regular.txt");
        fs::File::create(&regular)
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        assert_eq!(
            is_symlink_or_reparse_point(regular.to_str().unwrap()),
            false
        );

        let dir = tmp.join("regular-dir");
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(is_symlink_or_reparse_point(dir.to_str().unwrap()), false);

        let nonexistent = tmp.join("does-not-exist");
        assert_eq!(
            is_symlink_or_reparse_point(nonexistent.to_str().unwrap()),
            false
        );

        assert_eq!(is_symlink_or_reparse_point(""), false);

        assert_eq!(is_symlink_or_reparse_point("invalid\x00path"), false);

        #[cfg(unix)]
        {
            let link = tmp.join("link.txt");
            std::os::unix::fs::symlink(&regular, &link).unwrap();

            assert_eq!(
                is_symlink_or_reparse_point(link.to_str().unwrap()),
                true,
                "expected symlink at {} to be detected",
                link.display()
            );

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

    fn test_is_symlink_or_reparse_point_long_path() {
        use std::fs;
        use std::io::Write;

        let tmp = std::env::temp_dir().join("tsox_nativepath_long_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let mut long = tmp.clone();
        while long.to_string_lossy().len() < 260 {
            long = long.join("longpathsegment");
        }
        fs::create_dir_all(&long).unwrap();
        let file = long.join("target.txt");
        fs::File::create(&file).unwrap().write_all(b"x").unwrap();

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

            assert_eq!(is_symlink_or_reparse_point(link.to_str().unwrap()), true);

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
