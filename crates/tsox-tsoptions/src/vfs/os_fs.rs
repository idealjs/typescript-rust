use super::fs::FS;
use super::types::{Entries, FileInfo};
use std::path::Path;

pub struct OsFS;

impl FS for OsFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        cfg!(not(target_os = "windows"))
    }

    fn file_exists(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }

    fn read_file(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok().map(|s| {
            s.strip_prefix('\u{FEFF}')
                .map(|t| t.to_string())
                .unwrap_or(s)
        })
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        std::fs::write(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(data.as_bytes())
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        if Path::new(path).is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

    fn directory_exists(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        let mut entries = Entries::default();
        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = entry.file_type();
                if file_type.as_ref().map(|t| t.is_dir()).unwrap_or(false) {
                    entries.directories.push(name);
                } else if file_type.as_ref().map(|t| t.is_symlink()).unwrap_or(false) {
                    entries.symlinks.push(name.clone());

                    if let Ok(meta) = std::fs::metadata(entry.path()) {
                        if meta.is_dir() {
                            entries.directories.push(name);
                        } else {
                            entries.files.push(name);
                        }
                    }
                } else {
                    entries.files.push(name);
                }
            }
        }
        entries.files.sort();
        entries.directories.sort();
        entries
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        let p = Path::new(path);
        let meta = std::fs::metadata(p).ok()?;
        let symlink_meta = std::fs::symlink_metadata(p).ok()?;
        Some(FileInfo {
            name: p.file_name()?.to_string_lossy().to_string(),
            size: meta.len(),
            is_dir: meta.is_dir(),
            is_symlink: symlink_meta.file_type().is_symlink(),
            modified: meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
        })
    }

    fn realpath(&self, path: &str) -> String {
        std::fs::canonicalize(path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string())
    }
}
