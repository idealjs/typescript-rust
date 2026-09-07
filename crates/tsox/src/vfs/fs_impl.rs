use super::fs::FS;
use super::in_memory::{InMemoryFS, decode_with_bom, parent_dir, strip_path_prefix};
use super::types::{Entries, FileInfo};

impl FS for InMemoryFS {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.case_sensitive
    }

    fn file_exists(&self, path: &str) -> bool {
        let resolved = self.resolve_symlinks(path);
        self.lookup_file_key(&resolved).is_some()
    }

    fn read_file(&self, path: &str) -> Option<String> {
        let resolved = self.resolve_symlinks(path);
        let files = self.files.read().unwrap();
        let content = if let Some(c) = files.get(&resolved) {
            c
        } else if !self.case_sensitive {
            let target = resolved.to_ascii_lowercase();
            files
                .iter()
                .find(|(k, _)| k.to_ascii_lowercase() == target)
                .map(|(_, v)| v.as_str())?
        } else {
            return None;
        };
        Some(decode_with_bom(content))
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        let resolved = self.resolve_symlinks(path);

        if let Some(parent) = parent_dir(&resolved) {
            if self.is_file_path(&parent) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("mkdir {parent:?}: path exists but is not a directory",),
                ));
            }
        }
        let mut files = self.files.write().unwrap();
        let key = if self.case_sensitive {
            resolved.clone()
        } else {
            let target = resolved.to_ascii_lowercase();
            files
                .keys()
                .find(|k| k.to_ascii_lowercase() == target)
                .cloned()
                .unwrap_or_else(|| resolved.clone())
        };
        files.insert(key, data.to_string());
        Ok(())
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        let resolved = self.resolve_symlinks(path);
        let mut files = self.files.write().unwrap();
        let key = if self.case_sensitive {
            resolved.clone()
        } else {
            let target = resolved.to_ascii_lowercase();
            files
                .keys()
                .find(|k| k.to_ascii_lowercase() == target)
                .cloned()
                .unwrap_or_else(|| resolved.clone())
        };
        let entry = files.entry(key).or_default();
        entry.push_str(data);
        Ok(())
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        if let Some(key) = self.lookup_symlink_stored_key(path) {
            self.symlinks.write().unwrap().remove(&key);
            return Ok(());
        }

        if let Some(key) = self.lookup_file_key(path) {
            self.files.write().unwrap().remove(&key);
            return Ok(());
        }

        if let Some(key) = self.lookup_dir_key(path) {
            let prefix = format!("{key}/");
            let mut files = self.files.write().unwrap();
            let mut dirs = self.dirs.write().unwrap();
            let mut symlinks = self.symlinks.write().unwrap();
            dirs.remove(&key);
            let desc_files: Vec<String> = files
                .keys()
                .filter(|p| p.starts_with(&prefix))
                .cloned()
                .collect();
            for p in desc_files {
                files.remove(&p);
            }
            let desc_dirs: Vec<String> = dirs
                .iter()
                .filter(|p| p.starts_with(&prefix))
                .cloned()
                .collect();
            for p in desc_dirs {
                dirs.remove(&p);
            }
            let desc_syms: Vec<String> = symlinks
                .keys()
                .filter(|p| p.starts_with(&prefix))
                .cloned()
                .collect();
            for p in desc_syms {
                symlinks.remove(&p);
            }
            return Ok(());
        }
        Ok(())
    }

    fn directory_exists(&self, path: &str) -> bool {
        let resolved = self.resolve_symlinks(path);
        self.lookup_dir_key(&resolved).is_some()
    }

    fn get_accessible_entries(&self, path: &str) -> Entries {
        let mut entries = Entries::default();
        let resolved = self.resolve_symlinks(path);
        let prefix = if resolved.ends_with('/') {
            resolved
        } else {
            format!("{}/", resolved)
        };

        for key in self.files.read().unwrap().keys() {
            if let Some(rest) = strip_path_prefix(key, &prefix, self.case_sensitive) {
                if !rest.is_empty() && !rest.contains('/') {
                    entries.files.push(rest.to_string());
                }
            }
        }

        for dir in self.dirs.read().unwrap().iter() {
            if let Some(rest) = strip_path_prefix(dir, &prefix, self.case_sensitive) {
                if !rest.is_empty() && !rest.contains('/') {
                    entries.directories.push(rest.to_string());
                }
            }
        }

        entries.files.sort();
        entries.directories.sort();
        entries
    }

    fn stat(&self, path: &str) -> Option<FileInfo> {
        let is_symlink = self.lookup_symlink_key(path).is_some();
        let resolved = self.resolve_symlinks(path);
        if let Some(key) = self.lookup_file_key(&resolved) {
            let files = self.files.read().unwrap();
            let content = files.get(&key)?;
            return Some(FileInfo {
                name: key.rsplit('/').next()?.to_string(),
                size: content.len() as u64,
                is_dir: false,
                is_symlink,
                modified: std::time::SystemTime::now(),
            });
        }
        let key = self.lookup_dir_key(&resolved)?;
        Some(FileInfo {
            name: key.rsplit('/').next()?.to_string(),
            size: 0,
            is_dir: true,
            is_symlink,
            modified: std::time::SystemTime::now(),
        })
    }

    fn realpath(&self, path: &str) -> String {
        let resolved = self.resolve_symlinks(path);
        if let Some(key) = self.lookup_file_key(&resolved) {
            return key;
        }
        if let Some(key) = self.lookup_dir_key(&resolved) {
            return key;
        }

        if resolved != path {
            return resolved;
        }
        path.to_string()
    }
}
