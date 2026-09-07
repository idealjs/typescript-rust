use super::fs::FS;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct Entries {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub symlinks: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub modified: std::time::SystemTime,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            name: String::new(),
            size: 0,
            is_dir: false,
            is_symlink: false,
            modified: std::time::SystemTime::UNIX_EPOCH,
        }
    }
}

pub type SharedFS = Arc<dyn FS>;
