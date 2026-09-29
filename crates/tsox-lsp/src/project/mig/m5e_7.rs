#![allow(dead_code, unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use tsox_core::collections::mig::x11a::SyncSet;
use tsox_core::tspath::Path;

use crate::ls::lsconv_converters as lsconv;
use crate::project::overlay_fs::{DiskFile, FileContent, FileHandle};
use crate::project::project_collection::ProjectCollection;
use crate::project::session::*;
use crate::project::snapshot_fs::{FileSource, SnapshotFS};
use crate::project::watch::{get_recursive_glob_pattern, PatternsAndIgnored, WatchedFiles};

const MIN_WATCH_LOCATION_DEPTH: usize = 2;

#[derive(Default)]
pub struct RealpathAliasSet {
    paths: Mutex<HashSet<Path>>,
}

impl RealpathAliasSet {
    pub fn add(&self, path: Path) {
        self.paths.lock().unwrap().insert(path);
    }

    pub fn locked_paths(&self) -> HashSet<Path> {
        self.paths.lock().unwrap().clone()
    }

    pub fn clone_set(&self) -> RealpathAliasSet {
        RealpathAliasSet {
            paths: Mutex::new(self.paths.lock().unwrap().clone()),
        }
    }
}

pub fn is_relevant_extension(ext: &str) -> bool {
    matches!(ext, ".js" | ".jsx" | ".mjs" | ".cjs" | ".ts" | ".tsx" | ".mts" | ".cts" | ".json")
}

pub fn is_node_modules_path(path: &Path) -> bool {
    let s = path.as_str();
    s.ends_with("/node_modules") || s.contains("/node_modules/")
}

pub fn read_directory_into_entries(
    directories: &HashMap<Path, String>,
    is_file: &dyn Fn(&Path) -> bool,
    entries: &mut tsox_tsoptions::vfs::Entries,
) {
    for (child_path, child_name) in directories {
        if is_file(child_path) {
            entries.files.push(child_name.clone());
        } else {
            entries.directories.push(child_name.clone());
        }
    }
}

pub struct SnapshotFSBuilder {
    pub fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    pub prev_overlays: HashMap<Path, Arc<Overlay>>,
    pub overlays: HashMap<Path, Arc<Overlay>>,
    pub overlay_directories: HashMap<Path, HashMap<Path, String>>,
    pub disk_files: HashMap<Path, Arc<DiskFile>>,
    pub disk_directories: HashMap<Path, HashMap<Path, String>>,
    pub node_modules_realpath_aliases: Mutex<HashMap<Path, Arc<RealpathAliasSet>>>,
    pub to_path: Arc<dyn Fn(&str) -> Path + Send + Sync>,
    pub accessible_entries: Mutex<HashMap<Path, tsox_tsoptions::vfs::Entries>>,
    pub deleted_disk_files: Mutex<HashSet<Path>>,
    pub pending_realpath_paths: Mutex<HashMap<Path, Path>>,
}

fn reloaded_disk_file(entry: &DiskFile, content: &str) -> DiskFile {
    let mut file = DiskFile::new(entry.file_name().to_string(), content.to_string());
    file.realpath_path = entry.realpath_path.clone();
    file
}

pub fn new_snapshot_fs_builder(
    fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    prev_overlays: &HashMap<Path, Arc<Overlay>>,
    overlays: &HashMap<Path, Arc<Overlay>>,
    disk_files: &HashMap<Path, Arc<DiskFile>>,
    disk_directories: &HashMap<Path, HashMap<Path, String>>,
    node_modules_realpath_aliases: &HashMap<Path, Arc<RealpathAliasSet>>,
    _position_encoding: lsproto::PositionEncodingKind,
    to_path: Arc<dyn Fn(&str) -> Path + Send + Sync>,
) -> SnapshotFSBuilder {
    let mut overlay_directories: HashMap<Path, HashMap<Path, String>> = HashMap::new();
    for (path, overlay) in overlays {
        let mut child_path = path.clone();
        let mut child = overlay.file_name().to_string();
        loop {
            let parent_path = child_path.get_directory_path();
            let parent = tsox_core::tspath::get_directory_path(&child);
            if child_path == parent_path {
                break;
            }
            let base_name = tsox_core::tspath::get_base_file_name(&child);
            overlay_directories
                .entry(parent_path.clone())
                .or_default()
                .insert(child_path.clone(), base_name);
            child_path = parent_path;
            child = parent;
        }
    }
    SnapshotFSBuilder {
        fs,
        prev_overlays: prev_overlays.clone(),
        overlays: overlays.clone(),
        overlay_directories,
        disk_files: disk_files.clone(),
        disk_directories: disk_directories.clone(),
        node_modules_realpath_aliases: Mutex::new(node_modules_realpath_aliases.clone()),
        to_path,
        accessible_entries: Mutex::new(HashMap::new()),
        deleted_disk_files: Mutex::new(HashSet::new()),
        pending_realpath_paths: Mutex::new(HashMap::new()),
    }
}

impl SnapshotFSBuilder {
    pub fn finalize(&mut self) -> SnapshotFS {
        self.apply_pending_realpath_paths();
        let deleted: Vec<Path> = self.deleted_disk_files.lock().unwrap().drain().collect();
        let deleted_with_realpath: Vec<(Path, Path)> = deleted
            .iter()
            .filter_map(|path| {
                self.disk_files
                    .get(path)
                    .map(|file| (path.clone(), file.realpath_path.clone()))
            })
            .collect();
        for path in &deleted {
            self.disk_files.remove(path);
        }
        let mut on_added_file = |builder: &mut Self, path: &Path, file_name: &str| {
            let mut child_path = path.clone();
            let mut child = file_name.to_string();
            loop {
                let parent_path = child_path.get_directory_path();
                let parent = tsox_core::tspath::get_directory_path(&child);
                if child_path == parent_path {
                    break;
                }
                let base_name = tsox_core::tspath::get_base_file_name(&child);
                builder
                    .disk_directories
                    .entry(parent_path.clone())
                    .or_default()
                    .insert(child_path.clone(), base_name);
                child_path = parent_path;
                child = parent;
            }
        };
        let mut on_deleted_file_or_directory = |builder: &mut Self, path: &Path| {
            let parent_path = path.get_directory_path();
            let Some(dir) = builder.disk_directories.get_mut(&parent_path) else {
                return;
            };
            dir.remove(path);
            if dir.is_empty() {
                builder.disk_directories.remove(&parent_path);
            }
        };
        self.disk_files.retain(|path, _| !deleted.contains(path));
        for (path, realpath_path) in &deleted_with_realpath {
            if !realpath_path.as_str().is_empty() {
                let aliases = self
                    .node_modules_realpath_aliases
                    .lock()
                    .unwrap()
                    .get(realpath_path)
                    .cloned();
                if let Some(aliases) = aliases {
                    aliases.add(path.clone());
                }
            }
        }
        let disk_files_snapshot: Vec<(Path, Arc<DiskFile>)> = self
            .disk_files
            .iter()
            .map(|(p, f)| (p.clone(), Arc::clone(f)))
            .collect();
        for (path, file) in &disk_files_snapshot {
            on_added_file(self, path, &file.file_name().to_string());
        }
        for path in &deleted {
            on_deleted_file_or_directory(self, path);
        }
        let to_path = Arc::clone(&self.to_path);
        SnapshotFS {
            fs: Arc::clone(&self.fs),
            overlays: self.overlays.clone(),
            disk_files: self.disk_files.clone(),
            disk_directories: self.disk_directories.clone(),
            node_modules_realpath_aliases: self.node_modules_realpath_aliases.lock().unwrap().clone(),
            to_path: Box::new(move |f: &str| to_path(f)),
        }
    }

    fn to_path_ref(&self) -> Arc<dyn Fn(&str) -> Path + Send + Sync> {
        Arc::clone(&self.to_path)
    }

    pub fn is_open_file(&self, path: &Path) -> bool {
        self.overlays.contains_key(path)
    }

    pub fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        let path = (self.to_path)(file_name);
        self.get_file_by_path(file_name, &path)
    }

    pub fn file_exists(&self, file_name: &str, path: &Path) -> bool {
        if self.overlays.contains_key(path) {
            return true;
        }
        if let Some(entry) = self.disk_files.get(path) {
            return !entry.needs_reload || self.reload_entry_if_needed(path, entry).is_some();
        }
        self.fs.file_exists(file_name)
    }

    pub fn get_file_by_path(&self, file_name: &str, path: &Path) -> Option<Arc<dyn FileHandle>> {
        if let Some(file) = self.overlays.get(path) {
            return Some(Arc::clone(file) as Arc<dyn FileHandle>);
        }
        self.get_disk_file(file_name, path, false)
    }

    pub fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries {
        let mut entries = self.fs.get_accessible_entries(path);
        let p = (self.to_path)(path);
        let Some(overlay_directories) = self.overlay_directories.get(&p) else {
            return entries;
        };
        if let Some(merged) = self.accessible_entries.lock().unwrap().get(&p) {
            return merged.clone();
        }
        read_directory_into_entries(overlay_directories, &|p: &Path| self.is_open_file(p), &mut entries);
        self.accessible_entries.lock().unwrap().insert(p, entries.clone());
        entries
    }

    pub fn get_disk_file(
        &self,
        file_name: &str,
        path: &Path,
        force_reload: bool,
    ) -> Option<Arc<dyn FileHandle>> {
        let entry = self.disk_files.get(path).cloned().unwrap_or_else(|| {
            let mut file = DiskFile::new(file_name.to_string(), String::new());
            file.needs_reload = true;
            Arc::new(file)
        });
        if path.as_str().contains("/node_modules/") {
            self.record_realpath_alias(path, &entry, file_name);
        }
        if force_reload {
            return self.reload_entry(path, &entry);
        }
        self.reload_entry_if_needed(path, &entry)
    }

    fn apply_pending_realpath_paths(&mut self) {
        let pending: Vec<(Path, Path)> = self
            .pending_realpath_paths
            .lock()
            .unwrap()
            .drain()
            .collect();
        for (symlink_path, realpath_path) in pending {
            if let Some(file) = self.disk_files.get(&symlink_path) {
                if file.realpath_path == realpath_path {
                    continue;
                }
                let mut updated = (**file).clone();
                updated.realpath_path = realpath_path;
                self.disk_files.insert(symlink_path, Arc::new(updated));
            }
        }
    }

    pub fn record_realpath_alias(
        &self,
        symlink_path: &Path,
        disk_file_entry: &Arc<DiskFile>,
        symlink_file_name: &str,
    ) {
        let realpath = self.fs.realpath(symlink_file_name);
        let realpath_path = (self.to_path)(&realpath);
        if realpath_path != *symlink_path {
            self.pending_realpath_paths
                .lock()
                .unwrap()
                .insert(symlink_path.clone(), realpath_path.clone());
            let mut aliases_map = self.node_modules_realpath_aliases.lock().unwrap();
            let aliases = aliases_map
                .entry(realpath_path)
                .or_insert_with(|| Arc::new(RealpathAliasSet::default()));
            aliases.add(symlink_path.clone());
            let _ = disk_file_entry;
        }
    }

    pub fn reload_entry(&self, path: &Path, entry: &Arc<DiskFile>) -> Option<Arc<dyn FileHandle>> {
        let file_name = entry.file_name().to_string();
        match self.fs.read_file(&file_name) {
            Some(content) => Some(Arc::new(reloaded_disk_file(entry, &content)) as Arc<dyn FileHandle>),
            None => {
                self.deleted_disk_files.lock().unwrap().insert(path.clone());
                None
            }
        }
    }

    pub fn reload_entry_if_needed(
        &self,
        path: &Path,
        entry: &Arc<DiskFile>,
    ) -> Option<Arc<dyn FileHandle>> {
        if entry.matches_disk_text() {
            return Some(Arc::clone(entry) as Arc<dyn FileHandle>);
        }
        self.reload_entry(path, entry)
    }

    pub fn watch_changes_overlap_cache(&self, change: &FileChangeSummary) -> bool {
        for uri in change.changed.iter().chain(change.deleted.iter()) {
            let path = (self.to_path)(&uri.file_name());
            if self.disk_files.contains_key(&path) || self.node_modules_realpath_aliases.lock().unwrap().contains_key(&path) {
                return true;
            }
        }
        false
    }

    pub fn invalidate_cache(&mut self) {
        let paths: Vec<Path> = self.disk_files.keys().cloned().collect();
        for path in paths {
            if let Some(file) = self.disk_files.get(&path) {
                let mut updated = (**file).clone();
                updated.needs_reload = true;
                self.disk_files.insert(path, Arc::new(updated));
            }
        }
    }

    pub fn invalidate_node_modules_cache(&mut self) {
        let paths: Vec<Path> = self
            .disk_files
            .keys()
            .filter(|path| path.as_str().contains("/node_modules/"))
            .cloned()
            .collect();
        for path in paths {
            if let Some(file) = self.disk_files.get(&path) {
                let mut updated = (**file).clone();
                updated.needs_reload = true;
                self.disk_files.insert(path, Arc::new(updated));
            }
        }
    }

    pub fn mark_dirty_files(&mut self, mut change: FileChangeSummary) -> FileChangeSummary {
        self.apply_pending_realpath_paths();
        if !change.changed.is_empty() {
            let mut filtered_changed = HashSet::new();
            for uri in change.changed.iter() {
                let path = (self.to_path)(&uri.file_name());
                if self.overlays.contains_key(&path) {
                    filtered_changed.insert(uri.clone());
                    continue;
                }
                let Some(entry) = self.disk_files.get(&path).cloned() else {
                    filtered_changed.insert(uri.clone());
                    continue;
                };
                if self.reload_entry_if_content_changed(&path, &entry) {
                    filtered_changed.insert(uri.clone());
                }
            }
            change.changed = filtered_changed;
        }
        for uri in change.deleted.iter() {
            let path = (self.to_path)(&uri.file_name());
            if self.disk_files.remove(&path).is_some() {
                self.deleted_disk_files.lock().unwrap().insert(path.clone());
            }
        }
        change
    }

    pub fn reload_entry_if_content_changed(
        &mut self,
        path: &Path,
        entry: &Arc<DiskFile>,
    ) -> bool {
        match self.fs.read_file(&entry.file_name().to_string()) {
            None => {
                self.disk_files.remove(path);
                self.deleted_disk_files.lock().unwrap().insert(path.clone());
                true
            }
            Some(content) => {
                if content == entry.content() {
                    false
                } else {
                    self.disk_files
                        .insert(path.clone(), Arc::new(reloaded_disk_file(entry, &content)));
                    true
                }
            }
        }
    }

    pub fn is_relevant_file_name(
        &self,
        uri: &lsproto::DocumentUri,
        content_mapper_extensions: &[String],
        content_mapper_watched_files: &HashSet<Path>,
    ) -> bool {
        let file_name = uri.file_name();
        if content_mapper_watched_files.contains(&(self.to_path)(&file_name)) {
            return true;
        }
        let extensions: Vec<&str> = content_mapper_extensions.iter().map(|s| s.as_str()).collect();
        if tsox_core::tspath::file_extension_is_one_of(&file_name, &extensions) {
            return true;
        }
        if tsox_core::tspath::is_dynamic_file_name(&file_name) {
            return true;
        }
        let path = (self.to_path)(&file_name);
        if self.overlays.contains_key(&path) {
            return true;
        }
        let s = path.as_str();
        match s.rfind('.') {
            Some(i) if s[i..].len() > 0 && s.rfind('/').map_or(true, |sl| sl < i) => {
                is_relevant_extension(&s[i..])
            }
            _ => false,
        }
    }

    pub fn expand_and_filter_watch_events(
        &self,
        mut change: FileChangeSummary,
        content_mapper_extensions: &[String],
        content_mapper_watched_files: &HashSet<Path>,
    ) -> FileChangeSummary {
        if !change.deleted.is_empty() {
            let mut filtered_deleted = HashSet::new();
            for uri in change.deleted.iter() {
                let path = (self.to_path)(&uri.file_name());
                if self.disk_directories.contains_key(&path) {
                    self.collect_files_recursive(&path, &mut filtered_deleted);
                } else if self.is_relevant_file_name(uri, content_mapper_extensions, content_mapper_watched_files)
                    || is_node_modules_path(&path)
                {
                    filtered_deleted.insert(uri.clone());
                }
            }
            change.deleted = filtered_deleted;
        }
        if !change.changed.is_empty() {
            let mut filtered_changed = HashSet::new();
            for uri in change.changed.iter() {
                if self.is_relevant_file_name(uri, content_mapper_extensions, content_mapper_watched_files) {
                    filtered_changed.insert(uri.clone());
                }
            }
            change.changed = filtered_changed;
        }
        change
    }

    pub fn collect_files_recursive(&self, dir_path: &Path, files: &mut HashSet<lsproto::DocumentUri>) {
        let Some(dir_entry) = self.disk_directories.get(dir_path) else {
            return;
        };
        for child_path in dir_entry.keys() {
            if let Some(file) = self.disk_files.get(child_path) {
                files.insert(lsproto::DocumentUri(lsconv::file_name_to_document_uri(
                    &file.file_name().to_string(),
                )));
            }
            self.collect_files_recursive(child_path, files);
        }
    }

    pub fn convert_open_and_close_to_changes(&mut self, mut change: FileChangeSummary) -> FileChangeSummary {
        if !change.opened.0.is_empty() && !tsox_core::tspath::is_dynamic_file_name(&change.opened.file_name()) {
            let path = (self.to_path)(&change.opened.file_name());
            match self.disk_files.get(&path) {
                None => {
                    change.created.insert(change.opened.clone());
                }
                Some(entry) => {
                    if let Some(overlay) = self.overlays.get(&path)
                        && overlay.hash() != entry.hash()
                    {
                        change.changed.insert(change.opened.clone());
                    }
                }
            }
        }
        let closed: Vec<lsproto::DocumentUri> = change.closed.iter().cloned().collect();
        for uri in closed {
            let file_name = uri.file_name();
            if tsox_core::tspath::is_dynamic_file_name(&file_name) {
                continue;
            }
            let path = (self.to_path)(&file_name);
            if let Some(fh) = self.get_disk_file(&file_name, &path, true) {
                if let Some(prev_overlay) = self.prev_overlays.get(&path)
                    && fh.hash() != prev_overlay.hash()
                {
                    change.changed.insert(uri);
                }
                continue;
            }
            change.deleted.insert(uri);
        }
        change
    }

    pub fn clean_disk_files_not_seen_by(&mut self, collection: &ProjectCollection) {
        let _ = collection;
    }
}

impl SnapshotFSBuilder {
    pub fn expand_realpath_aliases(&self, mut change: FileChangeSummary) -> FileChangeSummary {
        if self.node_modules_realpath_aliases.lock().unwrap().is_empty() {
            return change;
        }
        let changed_uris: Vec<lsproto::DocumentUri> = change.changed.iter().cloned().collect();
        for uri in changed_uris {
            let path = (self.to_path)(&uri.file_name());
            let aliases = self.node_modules_realpath_aliases.lock().unwrap().get(&path).cloned();
            if let Some(aliases) = aliases {
                for alias_path in aliases.locked_paths() {
                    change.changed.insert(lsproto::DocumentUri(lsconv::file_name_to_document_uri(
                        alias_path.as_str(),
                    )));
                }
            }
        }
        let deleted_uris: Vec<lsproto::DocumentUri> = change.deleted.iter().cloned().collect();
        for uri in deleted_uris {
            let path = (self.to_path)(&uri.file_name());
            let aliases = self
                .node_modules_realpath_aliases
                .lock()
                .unwrap()
                .get(&path)
                .cloned();
            if let Some(aliases) = aliases {
                for alias_path in aliases.locked_paths() {
                    change.deleted.insert(lsproto::DocumentUri(lsconv::file_name_to_document_uri(
                        alias_path.as_str(),
                    )));
                }
            }
        }
        change
    }
}

impl SnapshotFS {
    pub fn is_file(&self, path: &Path) -> bool {
        self.disk_files.contains_key(path) || self.overlays.contains_key(path)
    }
}

pub struct SourceFS {
    pub tracking: bool,
    pub to_path: Arc<dyn Fn(&str) -> Path + Send + Sync>,
    pub missing_directories: Mutex<HashSet<Path>>,
    pub seen_files: Mutex<HashSet<Path>>,
    pub source: Arc<dyn FileSource>,
}

pub fn new_source_fs(
    tracking: bool,
    source: Arc<dyn FileSource>,
    to_path: Arc<dyn Fn(&str) -> Path + Send + Sync>,
) -> SourceFS {
    SourceFS {
        tracking,
        to_path,
        missing_directories: Mutex::new(HashSet::new()),
        seen_files: Mutex::new(HashSet::new()),
        source,
    }
}

impl SourceFS {
    pub fn disable_tracking(&mut self) {
        self.tracking = false;
    }

    pub fn track(&self, file_name: &str) {
        if !self.tracking {
            return;
        }
        self.seen_files.lock().unwrap().insert((self.to_path)(file_name));
    }

    pub fn seen_file(&self, path: &Path) -> bool {
        self.seen_files.lock().unwrap().contains(path)
    }

    pub fn seen_file_or_missing_parent_directory(&self, path: &Path) -> bool {
        if self.seen_files.lock().unwrap().contains(path) {
            return true;
        }
        let missing = self.missing_directories.lock().unwrap();
        if !missing.is_empty() {
            let mut current = path.clone();
            loop {
                if missing.contains(&current) {
                    return true;
                }
                let parent = current.get_directory_path();
                if parent == current {
                    break;
                }
                current = parent;
            }
        }
        false
    }

    pub fn get_file(&self, file_name: &str) -> Option<Arc<dyn FileHandle>> {
        self.track(file_name);
        self.source.get_file(file_name)
    }

    pub fn get_file_by_path(&self, file_name: &str, path: &Path) -> Option<Arc<dyn FileHandle>> {
        self.track(file_name);
        self.source.get_file_by_path(file_name, path)
    }

    pub fn directory_exists(&self, path: &str) -> bool {
        let exists = self.source.fs().directory_exists(path);
        if !exists && self.tracking {
            self.missing_directories.lock().unwrap().insert((self.to_path)(path));
        }
        exists
    }

    pub fn file_exists(&self, path: &str) -> bool {
        self.track(path);
        self.source.file_exists(path, &(self.to_path)(path))
    }

    pub fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries {
        self.source.get_accessible_entries(path)
    }

    pub fn read_file(&self, path: &str) -> Option<String> {
        self.get_file(path).map(|fh| fh.content().to_string())
    }

    pub fn realpath(&self, path: &str) -> String {
        self.source.fs().realpath(path)
    }

    pub fn stat(&self, path: &str) -> Option<tsox_tsoptions::vfs::FileInfo> {
        self.source.fs().stat(path)
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.source.fs().use_case_sensitive_file_names()
    }

    pub fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &tsox_tsoptions::vfs::FileInfo),
    ) -> std::io::Result<()> {
        self.source.fs().walk_dir(root, walk_fn)
    }

    pub fn write_file(&self, _path: &str, _data: &str) -> Result<(), String> {
        panic!("unimplemented")
    }

    pub fn append_file(&self, _path: &str, _data: &str) -> Result<(), String> {
        panic!("unimplemented")
    }

    pub fn remove(&self, _path: &str) -> Result<(), String> {
        panic!("unimplemented")
    }

    pub fn chtimes(&self, _path: &str, _atime: std::time::SystemTime, _mtime: std::time::SystemTime) -> Result<(), String> {
        panic!("unimplemented")
    }
}

pub fn new_watched_files_for_paths(
    name: &str,
    watch_kind: lsproto::WatchKind,
    has_relative_pattern_capability: bool,
    workspace_directory: &str,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> WatchedFiles<Vec<String>> {
    let workspace_directory = workspace_directory.to_string();
    let current_directory = current_directory.to_string();
    WatchedFiles::new(name, watch_kind, has_relative_pattern_capability, move |files: &Vec<String>| {
        let mut result = PatternsAndIgnored::default();
        for file in files {
            if tsox_core::tspath::to_path(&workspace_directory, &current_directory, use_case_sensitive_file_names)
                .contains_path(&tsox_core::tspath::to_path(file, &current_directory, use_case_sensitive_file_names))
            {
                result.patterns_inside_workspace.push(file.clone());
            } else {
                result
                    .directories_outside_workspace
                    .push(tsox_core::tspath::get_directory_path(file));
            }
        }
        result
    })
}

pub fn create_resolution_lookup_glob_mapper(
    workspace_directory: &str,
    lib_directory: &str,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> impl Fn(&SyncSet<Path>) -> PatternsAndIgnored + Send + Sync {
    let workspace_directory_path = tsox_core::tspath::to_path(workspace_directory, current_directory, use_case_sensitive_file_names);
    let current_directory_path = tsox_core::tspath::to_path(current_directory, current_directory, use_case_sensitive_file_names);
    let lib_directory_path = tsox_core::tspath::to_path(lib_directory, current_directory, use_case_sensitive_file_names);
    move |data: &SyncSet<Path>| {
        let mut ignored: HashSet<String> = HashSet::new();
        let mut seen_dirs: HashSet<Path> = HashSet::new();
        let mut include_workspace = false;
        let mut include_root = false;
        let mut include_lib = false;
        let mut node_modules_directories: HashSet<Path> = HashSet::new();
        let mut external_directories: HashSet<Path> = HashSet::new();
        data.for_each(|path| {
            if tsox_core::tspath::is_dynamic_file_name(path.as_str()) {
                return true;
            }
            if !seen_dirs.insert(path.get_directory_path()) {
                return true;
            }
            if workspace_directory_path.contains_path(path) {
                include_workspace = true;
            } else if current_directory_path.contains_path(path) {
                include_root = true;
            } else if lib_directory_path.contains_path(path) {
                include_lib = true;
            } else if let Some(idx) = path.as_str().find("/node_modules/") {
                node_modules_directories.insert(Path::from(&path.as_str()[..idx + "/node_modules".len()]));
            } else {
                external_directories.insert(path.get_directory_path());
            }
            true
        });
        let mut globs = Vec::new();
        if include_workspace {
            globs.push(get_recursive_glob_pattern(workspace_directory_path.as_str()));
        }
        if include_root {
            globs.push(get_recursive_glob_pattern(current_directory_path.as_str()));
        }
        if include_lib {
            globs.push(get_recursive_glob_pattern(lib_directory_path.as_str()));
        }
        if !node_modules_directories.is_empty() {
            let mut node_modules_globs: Vec<String> = node_modules_directories
                .iter()
                .map(|dir| get_recursive_glob_pattern(dir.as_str()))
                .collect();
            node_modules_globs.sort();
            globs.extend(node_modules_globs);
        }
            let mut outside_dirs = Vec::new();
            if !external_directories.is_empty() {
                let mut external_dir_strings: Vec<String> =
                    external_directories.iter().map(|d| d.as_str().to_string()).collect();
                external_dir_strings.sort();
                let compare_options = tsox_core::tspath::ComparePathsOptions {
                    use_case_sensitive_file_names,
                    current_directory: current_directory.to_string(),
                };
                let (external_directory_parents, ignored_external_dirs) = tsox_core::tspath::get_common_parents(
                    &external_dir_strings,
                    MIN_WATCH_LOCATION_DEPTH,
                    &compare_options,
                );
                outside_dirs = external_directory_parents;
                ignored = ignored_external_dirs;
            }
        PatternsAndIgnored {
            directories_outside_workspace: outside_dirs,
            patterns_inside_workspace: globs,
            ignored,
        }
    }
}

pub fn get_typings_locations_globs(
    typings_files: &[String],
    typings_location: &str,
    workspace_directory: &str,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> PatternsAndIgnored {
    let mut include_typings_location = false;
    let mut include_workspace = false;
    let mut external_directories: HashMap<Path, String> = HashMap::new();
    for file in typings_files {
        if tsox_core::tspath::to_path(typings_location, current_directory, use_case_sensitive_file_names)
            .contains_path(&tsox_core::tspath::to_path(file, current_directory, use_case_sensitive_file_names))
        {
            include_typings_location = true;
        } else if !tsox_core::tspath::to_path(workspace_directory, current_directory, use_case_sensitive_file_names)
            .contains_path(&tsox_core::tspath::to_path(file, current_directory, use_case_sensitive_file_names))
        {
            let directory = tsox_core::tspath::get_directory_path(file);
            external_directories.insert(
                tsox_core::tspath::to_path(&directory, current_directory, use_case_sensitive_file_names),
                directory,
            );
        } else {
            include_workspace = true;
        }
    }
    let mut external_dir_values: Vec<String> = external_directories.values().cloned().collect();
    external_dir_values.sort();
    let compare_options = tsox_core::tspath::ComparePathsOptions {
        use_case_sensitive_file_names,
        current_directory: current_directory.to_string(),
    };
    let (external_directory_parents, ignored) = tsox_core::tspath::get_common_parents(
        &external_dir_values,
        MIN_WATCH_LOCATION_DEPTH,
        &compare_options,
    );
    let mut globs = Vec::new();
    if include_workspace {
        globs.push(get_recursive_glob_pattern(workspace_directory));
    }
    if include_typings_location {
        globs.push(get_recursive_glob_pattern(typings_location));
    }
    PatternsAndIgnored {
        directories_outside_workspace: external_directory_parents,
        patterns_inside_workspace: globs,
        ignored,
    }
}

pub fn get_path_components_for_watching(path: &str, current_directory: &str) -> Vec<String> {
    let components = tsox_core::tspath::get_path_components(path, current_directory);
    let root_length = perceived_os_root_length_for_watching(&components);
    if root_length <= 1 {
        return components;
    }
    let rest: Vec<&str> = components[1..root_length].iter().map(|s| s.as_str()).collect();
    let new_root = tsox_core::tspath::combine_paths(&components[0], &rest);
    let mut result = vec![new_root];
    result.extend(components[root_length..].iter().cloned());
    result
}

pub fn perceived_os_root_length_for_watching(path_components: &[String]) -> usize {
    let length = path_components.len();
    if length <= 1 {
        return length;
    }
    if path_components[0].starts_with("//") {
        return 2;
    }
    if path_components[0].len() == 3
        && tsox_core::tspath::is_volume_character(path_components[0].as_bytes()[0])
        && path_components[0].as_bytes()[1] == b':'
        && path_components[0].as_bytes()[2] == b'/'
    {
        if path_components[1].eq_ignore_ascii_case("users") {
            return 3.min(length);
        }
        return 1;
    }
    if path_components[1] == "home" {
        return 3.min(length);
    }
    1
}

pub fn new_recursive_directory_watcher(
    directory: &str,
    kind: lsproto::WatchKind,
    use_relative_pattern: bool,
) -> lsproto::FileSystemWatcher {
    if use_relative_pattern {
        let base_uri = lsproto::DocumentUri(lsconv::file_name_to_document_uri(directory));
        return lsproto::FileSystemWatcher {
            glob_pattern: lsproto::PatternOrRelativePattern {
                pattern: None,
                relative_pattern: Some(lsproto::RelativePattern {
                    base_uri: lsproto::WorkspaceFolderOrURI {
                        uri: Some(base_uri.0),
                        workspace_folder: None,
                    },
                    pattern: "**/*".to_string(),
                }),
            },
            kind: Some(kind),
        };
    }
    let glob = get_recursive_glob_pattern(directory);
    lsproto::FileSystemWatcher {
        glob_pattern: lsproto::PatternOrRelativePattern {
            pattern: Some(glob),
            relative_pattern: None,
        },
        kind: Some(kind),
    }
}
