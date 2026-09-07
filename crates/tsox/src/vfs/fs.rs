use super::types::{Entries, FileInfo};

pub trait FS: Send + Sync {
    fn use_case_sensitive_file_names(&self) -> bool;
    fn file_exists(&self, path: &str) -> bool;
    fn read_file(&self, path: &str) -> Option<String>;
    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()>;
    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()>;
    fn remove(&self, path: &str) -> std::io::Result<()>;
    fn directory_exists(&self, path: &str) -> bool;
    fn get_accessible_entries(&self, path: &str) -> Entries;
    fn stat(&self, path: &str) -> Option<FileInfo>;
    fn realpath(&self, path: &str) -> String;

    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &FileInfo),
    ) -> std::io::Result<()> {
        let _ = (root, walk_fn);
        Ok(())
    }
}
