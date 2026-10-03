pub(crate) use std::sync::Arc;

pub(crate) use tsox_tsoptions::vfs::Entries;
pub(crate) use tsox_tsoptions::vfs::FS;
pub(crate) use tsox_tsoptions::vfs::FileInfo;

pub use tsox_ts_libs::bundled_libs;

pub const SCHEME: &str = "bundled:///";

pub fn lib_path() -> String { ::tsox_core::fntrace::enter("lib_path"); 
    format!("{SCHEME}libs")
}

pub fn is_bundled(path: &str) -> bool { ::tsox_core::fntrace::enter("is_bundled"); 
    path.starts_with(SCHEME)
}

pub(crate) fn split_path(path: &str) -> Option<&str> { ::tsox_core::fntrace::enter("split_path"); 
    path.strip_prefix(SCHEME)
}

pub fn lib_names() -> Vec<&'static str> { ::tsox_core::fntrace::enter("lib_names"); 
    bundled_libs().iter().map(|(n, _)| *n).collect()
}

pub fn lib_contents(name: &str) -> Option<&'static str> { ::tsox_core::fntrace::enter("lib_contents"); 
    bundled_libs()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
}

pub struct BundledFS {
    inner: Arc<dyn FS>,
}

impl BundledFS {
    pub fn new(inner: Arc<dyn FS>) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self { inner }
    }
}

impl FS for BundledFS {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.inner.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        if let Some(rest) = split_path(path) {
            return bundled_lib_name(rest).is_some();
        }
        self.inner.file_exists(path)
    }

    fn read_file(&self, path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        if let Some(rest) = split_path(path) {
            if let Some(name) = bundled_lib_name(rest) {
                return lib_contents(name).map(|s| s.to_string());
            }
            return None;
        }
        self.inner.read_file(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_file"); 
        if is_bundled(path) {
            panic!("cannot write to embedded file system: {path}");
        }
        self.inner.write_file(path, data)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("append_file"); 
        if is_bundled(path) {
            panic!("cannot write to embedded file system: {path}");
        }
        self.inner.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("remove"); 
        if is_bundled(path) {
            panic!("cannot remove from embedded file system: {path}");
        }
        self.inner.remove(path)
    }

    fn directory_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        if let Some(rest) = split_path(path) {
            return rest == "libs" || rest.is_empty();
        }
        self.inner.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> Entries { ::tsox_core::fntrace::enter("get_accessible_entries"); 
        if let Some(rest) = split_path(path) {
            let mut entries = Entries::default();
            if rest.is_empty() {
                entries.directories.push("libs".to_string());
            } else if rest == "libs" {
                entries.files = lib_names().iter().map(|s| s.to_string()).collect();
            }
            return entries;
        }
        self.inner.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<FileInfo> { ::tsox_core::fntrace::enter("stat"); 
        if let Some(rest) = split_path(path) {
            if rest.is_empty() || rest == "libs" {
                return Some(FileInfo {
                    name: if rest.is_empty() {
                        String::new()
                    } else {
                        "libs".to_string()
                    },
                    is_dir: true,
                    ..FileInfo::default()
                });
            }
            if let Some(name) = bundled_lib_name(rest) {
                let size = lib_contents(name).map(|c| c.len()).unwrap_or(0) as u64;
                return Some(FileInfo {
                    name: name.to_string(),
                    size,
                    is_dir: false,
                    ..FileInfo::default()
                });
            }
            return None;
        }
        self.inner.stat(path)
    }

    fn realpath(&self, path: &str) -> String { ::tsox_core::fntrace::enter("realpath"); 
        if is_bundled(path) {
            return path.to_string();
        }
        self.inner.realpath(path)
    }
}

pub(crate) fn bundled_lib_name(rest: &str) -> Option<&'static str> { ::tsox_core::fntrace::enter("bundled_lib_name"); 
    let name = rest.strip_prefix("libs/").unwrap_or(rest);
    bundled_libs()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, _)| *n)
}

#[cfg(test)]
pub(crate) mod tests;

// r 轮接线:迁移批次模块
pub mod mig;
