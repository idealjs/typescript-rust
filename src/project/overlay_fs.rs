#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::lsp::lsproto;
use crate::tspath::Path;
use crate::vfs::FS;

use super::file_change::{FileChange, FileChangeKind, FileChangeSummary};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Hash128 {
    pub lo: u64,
    pub hi: u64,
}

pub trait FileContent: Send + Sync {
    fn content(&self) -> &str;
    fn hash(&self) -> Hash128;
}

pub trait FileHandle: FileContent + Send + Sync {
    fn file_name(&self) -> &str;
    fn version(&self) -> i32;
    fn matches_disk_text(&self) -> bool;
    fn is_overlay(&self) -> bool;
    fn kind(&self) -> i32;
}

pub struct FileBase {
    file_name: String,
    content: String,
    hash: Hash128,
}

impl FileBase {
    pub fn new(file_name: String, content: String, hash: Hash128) -> Self {
        FileBase {
            file_name,
            content,
            hash,
        }
    }
}

impl FileContent for FileBase {
    fn content(&self) -> &str {
        &self.content
    }
    fn hash(&self) -> Hash128 {
        self.hash
    }
}

pub struct DiskFile {
    base: FileBase,
    pub needs_reload: bool,
    pub realpath_path: Path,
}

impl DiskFile {
    pub fn new(file_name: String, content: String) -> Self {
        let hash = hash_string_128(&content);
        DiskFile {
            base: FileBase::new(file_name, content, hash),
            needs_reload: false,
            realpath_path: Path::default(),
        }
    }
}

impl FileContent for DiskFile {
    fn content(&self) -> &str {
        self.base.content()
    }
    fn hash(&self) -> Hash128 {
        self.base.hash()
    }
}

impl FileHandle for DiskFile {
    fn file_name(&self) -> &str {
        &self.base.file_name
    }
    fn version(&self) -> i32 {
        0
    }
    fn matches_disk_text(&self) -> bool {
        !self.needs_reload
    }
    fn is_overlay(&self) -> bool {
        false
    }
    fn kind(&self) -> i32 {
        script_kind_from_file_name(&self.base.file_name)
    }
}

pub struct Overlay {
    base: FileBase,
    version: i32,
    kind: i32,
    pub matches_disk_text: bool,
}

impl Overlay {
    pub fn new(file_name: String, content: String, version: i32, kind: i32) -> Self {
        let hash = hash_string_128(&content);
        Overlay {
            base: FileBase::new(file_name, content, hash),
            version,
            kind,
            matches_disk_text: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.base.content
    }
}

impl FileContent for Overlay {
    fn content(&self) -> &str {
        self.base.content()
    }
    fn hash(&self) -> Hash128 {
        self.base.hash()
    }
}

impl FileHandle for Overlay {
    fn file_name(&self) -> &str {
        &self.base.file_name
    }
    fn version(&self) -> i32 {
        self.version
    }
    fn matches_disk_text(&self) -> bool {
        self.matches_disk_text
    }
    fn is_overlay(&self) -> bool {
        true
    }
    fn kind(&self) -> i32 {
        self.kind
    }
}

pub struct OverlayFS {
    pub fs: Arc<dyn FS>,
    pub position_encoding: lsproto::PositionEncodingKind,
    overlays: RwLock<HashMap<Path, Arc<Overlay>>>,
    to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
}

impl OverlayFS {
    pub fn new(
        fs: Arc<dyn FS>,
        overlays: HashMap<Path, Arc<Overlay>>,
        position_encoding: lsproto::PositionEncodingKind,
        to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
    ) -> Self {
        OverlayFS {
            fs,
            position_encoding,
            overlays: RwLock::new(overlays),
            to_path,
        }
    }

    pub fn overlays(&self) -> HashMap<Path, Arc<Overlay>> {
        self.overlays.read().unwrap().clone()
    }

    pub fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        let overlays = self.overlays.read().unwrap();
        let path = (self.to_path)(file_name);
        if let Some(overlay) = overlays.get(&path) {
            let cloned: Arc<dyn FileHandle> = Arc::clone(overlay) as Arc<dyn FileHandle>;
            return Some(cloned);
        }
        drop(overlays);
        match self.fs.read_file(file_name) {
            Some(content) => Some(Arc::new(DiskFile::new(file_name.to_string(), content))),
            None => None,
        }
    }

    pub fn process_changes(
        &self,
        changes: &[FileChange],
    ) -> (FileChangeSummary, HashMap<Path, Arc<Overlay>>) {
        let mut result = FileChangeSummary::default();
        let mut new_overlays = self.overlays.read().unwrap().clone();

        for change in changes {
            let uri = &change.uri;
            if !result.includes_watch_change_outside_node_modules
                && change.kind.is_watch_kind()
                && !uri.0.contains("/node_modules/")
            {
                result.includes_watch_change_outside_node_modules = true;
            }

            let path = (self.to_path)(&uri.file_name());
            match change.kind {
                FileChangeKind::Open => {
                    if result.opened.0.is_empty() && result.reopened.0.is_empty() {
                        result.opened = uri.clone();
                    }
                    let overlay = Overlay::new(
                        uri.file_name(),
                        change.content.clone(),
                        change.version,
                        script_kind_from_file_name(&uri.file_name()),
                    );
                    new_overlays.insert(path, Arc::new(overlay));
                }
                FileChangeKind::Close => {
                    result.closed.insert(uri.clone());
                    new_overlays.remove(&path);
                }
                FileChangeKind::Change => {
                    result.changed.insert(uri.clone());
                }
                FileChangeKind::Save => {}
                FileChangeKind::WatchCreate => {
                    result.created.insert(uri.clone());
                }
                FileChangeKind::WatchChange => {
                    result.changed.insert(uri.clone());
                }
                FileChangeKind::WatchDelete => {
                    result.deleted.insert(uri.clone());
                }
            }
        }

        *self.overlays.write().unwrap() = new_overlays.clone();
        (result, new_overlays)
    }
}

pub fn hash_string_128(s: &str) -> Hash128 {
    use std::hash::Hasher;
    use xxhash_rust::xxh3::Xxh3;
    let mut hasher = Xxh3::new();
    hasher.write(s.as_bytes());

    let lo = hasher.finish();
    let mut hasher2 = Xxh3::new();
    hasher2.write(s.as_bytes());
    hasher2.write(&[0x42]);
    let hi = hasher2.finish();
    Hash128 { lo, hi }
}

pub fn script_kind_from_file_name(file_name: &str) -> i32 {
    let ext = file_name.rsplit('.').next().unwrap_or("");
    match ext {
        "ts" => 3,
        "tsx" => 4,
        "js" | "jsx" | "mjs" | "cjs" => 1,
        "json" => 5,
        _ => 0,
    }
}
