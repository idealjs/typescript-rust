use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

use super::super::fs::FS;
use super::super::os_fs::OsFS;
use super::super::types::{FileInfo, SharedFS};

pub fn fs() -> SharedFS {
    static OS_VFS: OnceLock<SharedFS> = OnceLock::new();
    Arc::clone(OS_VFS.get_or_init(|| Arc::new(OsFS)))
}

pub fn get_global_typings_cache_location() -> String {
    let cache_dir = user_cache_dir().unwrap_or_else(std::env::temp_dir);
    let subdir = if cfg!(windows) {
        "Microsoft/TypeScript"
    } else {
        "typescript"
    };
    tsox_core::tspath::combine_paths(
        &cache_dir.to_string_lossy(),
        &[subdir, tsox_core::core::mig::m3k::version_major_minor()],
    )
}

fn user_cache_dir() -> Option<std::path::PathBuf> {
    if cfg!(windows) {
        return std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .map(|p| p.join("cache"));
    }
    if let Some(dir) = std::env::var_os("XDG_CACHE_HOME") {
        if !dir.is_empty() {
            return Some(std::path::PathBuf::from(dir));
        }
    }
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(|h| std::path::PathBuf::from(h).join(".cache"))
}

pub fn swap_case(s: &str) -> String {
    s.chars()
        .map(|r| {
            let upper = r.to_uppercase().next().unwrap_or(r);
            if upper == r {
                r.to_lowercase().next().unwrap_or(r)
            } else {
                upper
            }
        })
        .collect()
}

pub fn is_reparse_point(path: &str) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

static LIMITED_WALK_DIR_FUNC_POOL: Mutex<Vec<LimitedWalkDirFunc<'static>>> = Mutex::new(Vec::new());

pub struct LimitedWalkDirFunc<'a> {
    inner: Option<Box<dyn FnMut(&str, &FileInfo) + Send + 'a>>,
}

pub fn get_limited_walk_dir_func(
    walk_fn: Box<dyn FnMut(&str, &FileInfo) + Send>,
) -> LimitedWalkDirFunc<'static> {
    let mut w = LIMITED_WALK_DIR_FUNC_POOL
        .lock()
        .unwrap()
        .pop()
        .unwrap_or_else(|| LimitedWalkDirFunc { inner: None });
    w.inner = Some(walk_fn);
    w
}

pub fn put_limited_walk_dir_func(w: LimitedWalkDirFunc<'static>) {
    let mut w = w;
    w.inner = None;
    LIMITED_WALK_DIR_FUNC_POOL.lock().unwrap().push(w);
}

impl<'a> LimitedWalkDirFunc<'a> {
    pub fn walker(&mut self, path: &str, info: &FileInfo) {
        if let Some(inner) = self.inner.as_mut() {
            inner(path, info);
        }
    }
}

impl OsFS {
    pub fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut (dyn FnMut(&str, &FileInfo) + Send),
    ) -> std::io::Result<()> {
        let mut walker = LimitedWalkDirFunc {
            inner: Some(Box::new(move |path: &str, info: &FileInfo| {
                walk_fn(path, info)
            })),
        };
        walk_tree(root, &mut walker)
    }

    pub fn chtimes(
        &self,
        path: &str,
        atime: SystemTime,
        mtime: SystemTime,
    ) -> std::io::Result<()> {
        let file = std::fs::OpenOptions::new().write(true).open(path)?;
        file.set_times(
            std::fs::FileTimes::new().set_accessed(atime).set_modified(mtime),
        )
    }

    fn write_file_with_flag(
        &self,
        path: &str,
        content: &str,
        append: bool,
    ) -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(!append)
            .append(append)
            .open(path)?;
        file.write_all(content.as_bytes())
    }

    fn ensure_directory_exists(&self, directory_path: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(directory_path)
    }

    fn write_file_ensuring_dir(&self, path: &str, content: &str, append: bool) -> std::io::Result<()> {
        if self.write_file_with_flag(path, content, append).is_ok() {
            return Ok(());
        }
        let dir = tsox_core::tspath::get_directory_path(&tsox_core::tspath::normalize_path(path));
        self.ensure_directory_exists(&dir)?;
        self.write_file_with_flag(path, content, append)
    }

    pub fn write_file_ensuring_dir_truncate(&self, path: &str, content: &str) -> std::io::Result<()> {
        self.write_file_ensuring_dir(path, content, false)
    }

    pub fn write_file_ensuring_dir_append(&self, path: &str, content: &str) -> std::io::Result<()> {
        self.write_file_ensuring_dir(path, content, true)
    }
}

fn walk_tree(root: &str, walker: &mut LimitedWalkDirFunc<'_>) -> std::io::Result<()> {
    let mut stack = vec![root.to_string()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let full = format!("{}/{}", dir.trim_end_matches('/'), name);
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            walker.walker(&full, &FileInfo {
                name,
                size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                is_dir,
                is_symlink: entry.file_type().map(|t| t.is_symlink()).unwrap_or(false),
                modified: entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH),
            });
            if is_dir {
                stack.push(full);
            }
        }
    }
    Ok(())
}
