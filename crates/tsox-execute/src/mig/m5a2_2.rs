#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::m4y_2::{
    BuildInfo, BuildInfoDiagnostic, BuildInfoFilePendingEmit, BuildInfoRepopulateInfo,
};
use super::m5a2_3::{
    ReadableBuildInfoEmitSignature, ReadableBuildInfoFilePendingEmit,
    ReadableBuildInfoResolvedRoot, ReadableBuildInfoSemanticDiagnostic,
};
use tsox_core::core::bfs::SyncSet;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_core::diagnostics::Category;
use tsox_core::tspath::{self, EXTENSION_TS_BUILD_INFO};
use tsox_frontend::ast::mig::m3e::RepopulateDiagnosticKind;
use tsox_tsoptions::vfs::FS;

pub mod fs_baseline_util {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use tsox_tsoptions::vfs::FS;

    use super::{NameSet, TestFs, vfstest};

    pub struct FileChange {
        pub path: String,
        pub deleted: bool,
    }

    #[derive(Clone, Debug)]
    pub struct DiffEntry {
        pub content: String,
        pub m_time: SystemTime,
        pub is_written: bool,
        pub symlink_target: String,
    }

    #[derive(Clone, Default)]
    pub struct Snapshot {
        pub snap: HashMap<String, DiffEntry>,
        pub default_libs: Option<NameSet>,
    }

    pub struct FsDiffer {
        pub fs: Arc<dyn FS>,
        map_fs: Arc<vfstest::MapFS>,
        pub default_libs: Option<Arc<NameSet>>,
        pub written_files: Arc<NameSet>,
        serialized_diff: Mutex<Option<Snapshot>>,
    }

    impl FsDiffer {
        pub fn new(test_fs: &TestFs) -> Self {
            FsDiffer {
                fs: test_fs.fs.clone(),
                map_fs: test_fs
                    .map_fs
                    .clone()
                    .expect("FsDiffer requires a MapFS-backed test fs"),
                default_libs: test_fs.default_libs.clone(),
                written_files: test_fs.written_files.clone(),
                serialized_diff: Mutex::new(None),
            }
        }

        pub fn map_fs(&self) -> Arc<vfstest::MapFS> {
            Arc::clone(&self.map_fs)
        }

        pub fn serialized_diff(&self) -> Option<Snapshot> {
            self.serialized_diff.lock().unwrap().clone()
        }

        pub fn baseline_fs_with_diff(&self, baseline: &mut dyn std::io::Write) {
            let map_fs = self.map_fs();
            let mut snap: HashMap<String, DiffEntry> = HashMap::new();
            let mut diffs: HashMap<String, String> = HashMap::new();
            let previous = self.serialized_diff.lock().unwrap().clone();

            for (path, file) in map_fs.entries() {
                if file.is_symlink {
                    let target = map_fs
                        .get_target_of_symlink(&path)
                        .unwrap_or_else(|| panic!("Failed to resolve symlink target: {path}"));
                    let new_entry = DiffEntry {
                        content: String::new(),
                        m_time: SystemTime::UNIX_EPOCH,
                        is_written: false,
                        symlink_target: target,
                    };
                    snap.insert(path.clone(), new_entry.clone());
                    add_fs_entry_diff(
                        previous.as_ref(),
                        self.default_libs.as_deref(),
                        Some(&new_entry),
                        &path,
                        &mut diffs,
                    );
                } else if !file.is_dir {
                    let content = sanitize_internal_symbol_name(&file.data);
                    let new_entry = DiffEntry {
                        content,
                        m_time: file.mod_time,
                        is_written: self.written_files.has(&path),
                        symlink_target: String::new(),
                    };
                    snap.insert(path.clone(), new_entry.clone());
                    add_fs_entry_diff(
                        previous.as_ref(),
                        self.default_libs.as_deref(),
                        Some(&new_entry),
                        &path,
                        &mut diffs,
                    );
                }
            }
            if let Some(old) = &previous {
                for path in old.snap.keys() {
                    if map_fs.get_file_info(path).is_none() {
                        add_fs_entry_diff(
                            previous.as_ref(),
                            self.default_libs.as_deref(),
                            None,
                            path,
                            &mut diffs,
                        );
                    }
                }
            }

            let snapshot_default_libs = self.default_libs.as_ref().map(|libs| {
                let copy = NameSet::new();
                for key in libs.keys_vec() {
                    copy.add_if_absent(&key);
                }
                copy
            });
            *self.serialized_diff.lock().unwrap() = Some(Snapshot {
                snap,
                default_libs: snapshot_default_libs,
            });

            let mut diff_keys: Vec<String> = diffs.keys().cloned().collect();
            diff_keys.sort();
            for path in &diff_keys {
                let _ = writeln!(baseline, "//// [{}] {}", path, diffs[path]);
            }
            let _ = writeln!(baseline);
            self.written_files.clear();
        }
    }

    fn add_fs_entry_diff(
        serialized: Option<&Snapshot>,
        default_libs: Option<&NameSet>,
        new_content: Option<&DiffEntry>,
        path: &str,
        diffs: &mut HashMap<String, String>,
    ) {
        let (old_content, snapshot_default_libs) = match serialized {
            Some(snapshot) => (snapshot.snap.get(path), snapshot.default_libs.as_ref()),
            None => (None, None),
        };
        match (old_content, new_content) {
            (None, Some(new)) => {
                let is_default_lib = default_libs.is_some_and(|libs| libs.has(path));
                if !is_default_lib {
                    if !new.symlink_target.is_empty() {
                        diffs.insert(path.to_string(), format!("-> {} *new*", new.symlink_target));
                    } else {
                        diffs.insert(path.to_string(), format!("*new* \n{}", new.content));
                    }
                }
            }
            (Some(_), None) => {
                diffs.insert(path.to_string(), "*deleted*".to_string());
            }
            (Some(old), Some(new)) => {
                if new.content != old.content {
                    diffs.insert(path.to_string(), format!("*modified* \n{}", new.content));
                } else if new.is_written {
                    diffs.insert(path.to_string(), "*rewrite with same content*".to_string());
                } else if new.m_time != old.m_time {
                    diffs.insert(path.to_string(), "*mTime changed*".to_string());
                } else if snapshot_default_libs.is_some_and(|libs| libs.has(path))
                    && !default_libs.is_some_and(|libs| libs.has(path))
                {
                    diffs.insert(path.to_string(), format!("*Lib*\n{}", new.content));
                }
            }
            (None, None) => {}
        }
    }

    pub fn sanitize_internal_symbol_name(s: &str) -> String {
        if !s.contains("\u{FFFD}@") {
            return s.to_string();
        }
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        while let Some(pos) = rest.find("\u{FFFD}@") {
            let marker_end = pos + "\u{FFFD}@".len();
            let after_marker = &rest[marker_end..];
            let name_end = match after_marker.find('@') {
                Some(index) if index > 0 => index,
                _ => {
                    out.push_str(&rest[..marker_end]);
                    rest = after_marker;
                    continue;
                }
            };
            let after_name = &after_marker[name_end + 1..];
            let digits_end = after_name
                .bytes()
                .take_while(|b| b.is_ascii_digit())
                .count();
            if digits_end == 0 {
                out.push_str(&rest[..marker_end]);
                rest = after_marker;
                continue;
            }
            out.push_str(&rest[..pos]);
            out.push_str("\u{FFFD}@");
            out.push_str(&after_marker[..name_end]);
            out.push_str("@<symbolId>");
            rest = &after_name[digits_end..];
        }
        out.push_str(rest);
        out
    }
}

pub mod harness_util {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use tsox_core::tspath::{self, ComparePathsOptions};

    pub const FAKE_TS_VERSION: &str = "FakeTSVersion";

    pub struct TracerForBaselining {
        opts: ComparePathsOptions,
        package_json_cache: Mutex<HashMap<tspath::Path, bool>>,
    }

    impl TracerForBaselining {
        pub fn trace_with_writer(
            &self,
            w: &mut dyn std::io::Write,
            msg: &str,
            use_package_json_cache: bool,
        ) {
            let _ = writeln!(w, "{}", self.sanitize_trace(msg, use_package_json_cache));
        }

        pub fn reset(&self) {
            self.package_json_cache.lock().unwrap().clear();
        }

        fn cache_path(&self, file: &str) -> tspath::Path {
            tspath::to_path(
                file,
                &self.opts.current_directory,
                self.opts.use_case_sensitive_file_names,
            )
        }

        fn sanitize_trace(&self, msg: &str, use_package_json_cache: bool) -> String {
            let version_token = format!("'{}'", tsox_core::core::mig::m3k::version());
            if msg.contains(&version_token) {
                return msg.replacen(&version_token, &format!("'{}'", FAKE_TS_VERSION), 1);
            }
            let not_exist_cached = "' does not exist according to earlier cached lookups.";
            if let Some(rest) = msg.strip_suffix(not_exist_cached) {
                let file = rest.strip_prefix("File '").unwrap_or(rest);
                if use_package_json_cache {
                    let file_path = self.cache_path(file);
                    let mut cache = self.package_json_cache.lock().unwrap();
                    if cache.contains_key(&file_path) {
                        return msg.to_string();
                    }
                    cache.insert(file_path, false);
                }
                return format!("File '{file}' does not exist.");
            }
            let exists_cached = "' exists according to earlier cached lookups.";
            if let Some(rest) = msg.strip_suffix(exists_cached) {
                let file = rest.strip_prefix("File '").unwrap_or(rest);
                if use_package_json_cache {
                    let file_path = self.cache_path(file);
                    let mut cache = self.package_json_cache.lock().unwrap();
                    if cache.contains_key(&file_path) {
                        return msg.to_string();
                    }
                    cache.insert(file_path, true);
                }
                return format!("Found 'package.json' at '{file}'.");
            }
            if use_package_json_cache {
                if let Some(rest) = msg.strip_suffix("' does not exist.") {
                    let file = rest.strip_prefix("File '").unwrap_or(rest);
                    let file_path = self.cache_path(file);
                    let mut cache = self.package_json_cache.lock().unwrap();
                    if !cache.contains_key(&file_path) {
                        cache.insert(file_path, false);
                        return msg.to_string();
                    }
                    return format!("File '{file}' does not exist according to earlier cached lookups.");
                }
                if let Some(rest) = msg.strip_prefix("Found 'package.json' at '") {
                    let file = rest.strip_suffix("'.").unwrap_or(rest);
                    let file_path = self.cache_path(file);
                    let mut cache = self.package_json_cache.lock().unwrap();
                    if !cache.contains_key(&file_path) {
                        cache.insert(file_path, true);
                        return msg.to_string();
                    }
                    return format!("File '{file}' exists according to earlier cached lookups.");
                }
            }
            msg.to_string()
        }
    }

    pub fn new_tracer_for_baselining(opts: ComparePathsOptions) -> TracerForBaselining {
        TracerForBaselining {
            opts,
            package_json_cache: Mutex::new(HashMap::new()),
        }
    }
}

pub mod vfstest {
    use std::collections::{BTreeMap, HashMap};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use tsox_core::tspath;
    use tsox_tsoptions::vfs::{Entries, FileInfo};
    use tsox_tsoptions::vfs::FS;

    use super::{NameSet, TestFs};

    pub trait Clock: Send + Sync {
        fn now(&self) -> SystemTime;
    }

    #[derive(Clone, Debug)]
    pub struct MapFile {
        pub path: String,
        pub data: String,
        pub mod_time: SystemTime,
        pub is_dir: bool,
        pub is_symlink: bool,
    }

    pub struct MapFS {
        entries: Mutex<BTreeMap<String, MapFile>>,
        use_case_sensitive_file_names: bool,
        clock: Arc<dyn Clock>,
    }

    impl MapFS {
        fn canonical(&self, path: &str) -> String {
            tspath::get_canonical_file_name(path, self.use_case_sensitive_file_names)
        }

        pub fn entries(&self) -> Vec<(String, MapFile)> {
            self.entries
                .lock()
                .unwrap()
                .iter()
                .map(|(key, file)| (key.clone(), file.clone()))
                .collect()
        }

        pub fn get_mod_time(&self, path: &str) -> SystemTime {
            let canonical = self.canonical(path);
            self.entries
                .lock()
                .unwrap()
                .get(&canonical)
                .map(|file| file.mod_time)
                .unwrap_or(SystemTime::UNIX_EPOCH)
        }

        pub fn get_target_of_symlink(&self, path: &str) -> Option<String> {
            let canonical = self.canonical(path);
            self.entries
                .lock()
                .unwrap()
                .get(&canonical)
                .filter(|file| file.is_symlink)
                .map(|file| format!("/{}", file.data))
        }

        pub fn get_file_info(&self, path: &str) -> Option<MapFile> {
            let canonical = self.canonical(path);
            self.entries.lock().unwrap().get(&canonical).cloned()
        }

        fn set_entry(&self, path: &str, data: String, is_dir: bool, is_symlink: bool) {
            let canonical = self.canonical(path);
            self.entries.lock().unwrap().insert(
                canonical,
                MapFile {
                    path: path.to_string(),
                    data,
                    mod_time: self.clock.now(),
                    is_dir,
                    is_symlink,
                },
            );
        }
    }

    impl FS for MapFS {
        fn use_case_sensitive_file_names(&self) -> bool {
            self.use_case_sensitive_file_names
        }

        fn file_exists(&self, path: &str) -> bool {
            self.get_file_info(path).is_some_and(|file| !file.is_dir)
        }

        fn read_file(&self, path: &str) -> Option<String> {
            self.get_file_info(path)
                .filter(|file| !file.is_dir)
                .map(|file| file.data)
        }

        fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
            self.set_entry(path, data.to_string(), false, false);
            Ok(())
        }

        fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
            let existing = self.read_file(path).unwrap_or_default();
            self.set_entry(path, format!("{existing}{data}"), false, false);
            Ok(())
        }

        fn remove(&self, path: &str) -> std::io::Result<()> {
            let canonical = self.canonical(path);
            let mut entries = self.entries.lock().unwrap();
            let prefix = format!("{canonical}/");
            let children: Vec<String> = entries
                .keys()
                .filter(|key| key.starts_with(&prefix))
                .cloned()
                .collect();
            for child in children {
                entries.remove(&child);
            }
            entries.remove(&canonical);
            Ok(())
        }

        fn chtimes(
            &self,
            path: &str,
            _atime: SystemTime,
            mtime: SystemTime,
        ) -> std::io::Result<()> {
            let canonical = self.canonical(path);
            let mut entries = self.entries.lock().unwrap();
            match entries.get_mut(&canonical) {
                Some(file) => {
                    file.mod_time = mtime;
                    Ok(())
                }
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("chtimes: file does not exist: {path}"),
                )),
            }
        }

        fn directory_exists(&self, path: &str) -> bool {
            self.get_file_info(path).is_some_and(|file| file.is_dir)
        }

        fn get_accessible_entries(&self, path: &str) -> Entries {
            let prefix = format!("{}/", self.canonical(path));
            let mut result = Entries::default();
            let entries = self.entries.lock().unwrap();
            for (key, file) in entries.iter() {
                if let Some(rest) = key.strip_prefix(&prefix) {
                    if rest.is_empty() || rest.contains('/') {
                        continue;
                    }
                    if file.is_dir {
                        result.directories.push(file.path.clone());
                    } else {
                        result.files.push(file.path.clone());
                    }
                }
            }
            result
        }

        fn stat(&self, path: &str) -> Option<FileInfo> {
            let file = self.get_file_info(path)?;
            let name = file.path.rsplit('/').next().unwrap_or(&file.path).to_string();
            Some(FileInfo {
                name,
                size: file.data.len() as u64,
                is_dir: file.is_dir,
                is_symlink: file.is_symlink,
                modified: file.mod_time,
            })
        }

        fn realpath(&self, path: &str) -> String {
            self.get_file_info(path)
                .map(|file| file.path)
                .unwrap_or_else(|| path.to_string())
        }
    }

    fn ensure_parent_dirs(
        map: &mut BTreeMap<String, MapFile>,
        path: &str,
        clock: &Arc<dyn Clock>,
        use_case_sensitive_file_names: bool,
    ) {
        let mut dir = path;
        while let Some(index) = dir.rfind('/') {
            dir = &dir[..index];
            if dir.is_empty() {
                break;
            }
            let canonical = tspath::get_canonical_file_name(dir, use_case_sensitive_file_names);
            if map.contains_key(&canonical) {
                break;
            }
            map.insert(
                canonical,
                MapFile {
                    path: dir.to_string(),
                    data: String::new(),
                    mod_time: clock.now(),
                    is_dir: true,
                    is_symlink: false,
                },
            );
        }
    }

    pub fn from_map_with_clock(
        files: HashMap<String, String>,
        use_case_sensitive_file_names: bool,
        clock: Arc<dyn Clock>,
    ) -> TestFs {
        let mut map: BTreeMap<String, MapFile> = BTreeMap::new();
        let mut keys: Vec<&String> = files.keys().collect();
        keys.sort();
        for path in keys {
            map.insert(
                tspath::get_canonical_file_name(path, use_case_sensitive_file_names),
                MapFile {
                    path: path.clone(),
                    data: files[path].clone(),
                    mod_time: clock.now(),
                    is_dir: false,
                    is_symlink: false,
                },
            );
            ensure_parent_dirs(&mut map, path, &clock, use_case_sensitive_file_names);
        }
        let map_fs = Arc::new(MapFS {
            entries: Mutex::new(map),
            use_case_sensitive_file_names,
            clock,
        });
        TestFs {
            fs: map_fs.clone(),
            map_fs: Some(map_fs),
            default_libs: Some(Arc::new(NameSet::new())),
            written_files: Arc::new(NameSet::new()),
        }
    }
}

pub mod contentmapper_test {
    pub use tsox_compile::mig::m3l_cm_2::ReadWriteCloser;
    use tsox_compile::mig::m3l_cm_2::Spawner;

    #[derive(Debug, Default)]
    pub struct MemPipe {
        buf: Vec<u8>,
        read_pos: usize,
    }

    impl std::io::Read for MemPipe {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            let available = self.buf.len() - self.read_pos;
            if available == 0 {
                return Ok(0);
            }
            let count = available.min(out.len());
            out[..count].copy_from_slice(&self.buf[self.read_pos..self.read_pos + count]);
            self.read_pos += count;
            Ok(count)
        }
    }

    impl std::io::Write for MemPipe {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.buf.extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl ReadWriteCloser for MemPipe {
        fn close(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    pub struct TestSpawner;

    pub fn new_spawner() -> TestSpawner {
        TestSpawner
    }

    impl Spawner for TestSpawner {
        fn spawn(
            &self,
            command: &[String],
            dir: &str,
            stderr: &mut dyn std::io::Write,
        ) -> std::io::Result<Box<dyn ReadWriteCloser>> {
            if command.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "spawn: empty command",
                ));
            }
            let _ = (dir, stderr);
            Ok(Box::new(MemPipe::default()))
        }
    }
}

#[derive(Debug, Default)]
pub struct NameSet {
    inner: Mutex<HashSet<String>>,
}

impl NameSet {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_if_absent(&self, key: &str) -> bool {
        self.inner.lock().unwrap().insert(key.to_string())
    }

    pub fn has(&self, key: &str) -> bool {
        self.inner.lock().unwrap().contains(key)
    }

    pub fn delete(&self, key: &str) {
        self.inner.lock().unwrap().remove(key);
    }

    pub fn keys_vec(&self) -> Vec<String> {
        self.inner.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap().clear();
    }
}

impl Clone for NameSet {
    fn clone(&self) -> Self {
        let copy = Self::new();
        for key in self.keys_vec() {
            copy.add_if_absent(&key);
        }
        copy
    }
}

#[derive(Clone)]
pub struct TestFs {
    pub fs: Arc<dyn FS>,
    pub map_fs: Option<Arc<vfstest::MapFS>>,
    pub default_libs: Option<Arc<NameSet>>,
    pub written_files: Arc<NameSet>,
}


impl TestFs {
    pub fn written_files_keys(&self) -> Vec<String> {
        self.written_files.keys_vec()
    }

    pub fn default_libs_add(&self, path: &str) {
        if let Some(default_libs) = &self.default_libs {
            default_libs.add_if_absent(path);
        }
    }

    pub fn remove_ignore_lib_path(&self, path: &str) {
        if let Some(default_libs) = &self.default_libs {
            if default_libs.has(path) {
                default_libs.delete(path);
            }
        }
    }

    pub fn read_file_handling_build_info(&self, path: &str) -> Option<String> {
        let contents = self.fs.read_file(path)?;
        if tspath::file_extension_is(path, EXTENSION_TS_BUILD_INFO) {
            if let Ok(mut build_info) = serde_json::from_str::<BuildInfo>(&contents) {
                if build_info.version == harness_util::FAKE_TS_VERSION {
                    build_info.version = tsox_core::core::mig::m3k::version().to_string();
                    let new_contents = serde_json::to_string(&build_info).expect(
                        "testFs.ReadFile: failed to marshal build info after fixing version",
                    );
                    return Some(new_contents);
                }
            }
        }
        Some(contents)
    }

    pub fn write_file_handling_build_info(&self, path: &str, data: &str) -> Result<(), String> {
        if tspath::file_extension_is(path, EXTENSION_TS_BUILD_INFO) {
            match serde_json::from_str::<BuildInfo>(data) {
                Ok(mut build_info) => {
                    let mut data = data.to_string();
                    if build_info.version == tsox_core::core::mig::m3k::version() {
                        build_info.version = harness_util::FAKE_TS_VERSION.to_string();
                        data = serde_json::to_string(&build_info).map_err(|e| {
                            format!(
                                "testFs.WriteFile: failed to marshal build info after fixing version: {e}"
                            )
                        })?;
                    }
                    self.write_file(
                        &format!("{path}.readable.baseline.txt"),
                        &super::m5a2_3::to_readable_build_info(
                            &build_info,
                            &fs_baseline_util::sanitize_internal_symbol_name(&data),
                        ),
                    )?;
                }
                Err(e) => panic!(
                    "testFs.WriteFile: failed to unmarshal build info: - use underlying FS's write method if this is intended use for testcase{e}"
                ),
            }
        }
        self.fs
            .write_file(path, data)
            .map_err(|e| e.to_string())
    }

    pub fn write_file(&self, path: &str, data: &str) -> Result<(), String> {
        self.remove_ignore_lib_path(path);
        self.written_files.add_if_absent(path);
        self.write_file_handling_build_info(path, data)
    }
}

impl FS for TestFs {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.fs.use_case_sensitive_file_names()
    }

    fn file_exists(&self, path: &str) -> bool {
        self.fs.file_exists(path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        self.remove_ignore_lib_path(path);
        self.read_file_handling_build_info(path)
    }

    fn write_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        Self::write_file(self, path, data).map_err(std::io::Error::other)
    }

    fn append_file(&self, path: &str, data: &str) -> std::io::Result<()> {
        self.fs.append_file(path, data)
    }

    fn remove(&self, path: &str) -> std::io::Result<()> {
        self.fs.remove(path)
    }

    fn chtimes(
        &self,
        path: &str,
        atime: std::time::SystemTime,
        mtime: std::time::SystemTime,
    ) -> std::io::Result<()> {
        self.fs.chtimes(path, atime, mtime)
    }

    fn directory_exists(&self, path: &str) -> bool {
        self.fs.directory_exists(path)
    }

    fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries {
        self.fs.get_accessible_entries(path)
    }

    fn stat(&self, path: &str) -> Option<tsox_tsoptions::vfs::FileInfo> {
        self.fs.stat(path)
    }

    fn realpath(&self, path: &str) -> String {
        self.fs.realpath(path)
    }

    fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &tsox_tsoptions::vfs::FileInfo),
    ) -> std::io::Result<()> {
        self.fs.walk_dir(root, walk_fn)
    }
}

pub fn is_zero(value: &usize) -> bool {
    *value == 0
}

#[derive(Serialize)]
pub struct ReadableBuildInfo<'a> {
    #[serde(skip)]
    pub build_info: &'a BuildInfo,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub errors: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub check_pending: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root: Vec<ReadableBuildInfoRoot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub package_jsons: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_package_jsons: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_infos: Vec<ReadableBuildInfoFileInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_ids_list: Vec<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<tsox_core::collections::ordered_map::OrderedMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referenced_map: Option<tsox_core::collections::ordered_map::OrderedMap<String, Vec<String>>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_diagnostics_per_file: Vec<ReadableBuildInfoSemanticDiagnostic>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emit_diagnostics_per_file: Vec<ReadableBuildInfoDiagnosticsOfFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub change_file_set: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_files_pending_emit: Vec<ReadableBuildInfoFilePendingEmit>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub latest_changed_dts_file: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emit_signatures: Vec<ReadableBuildInfoEmitSignature>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolved_root: Vec<ReadableBuildInfoResolvedRoot>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub size: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub semantic_errors: bool,
}

#[derive(Serialize)]
pub struct ReadableBuildInfoRoot {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    pub original: super::m4y_2::BuildInfoRoot,
}

#[derive(Serialize)]
pub struct ReadableBuildInfoFileInfo {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub signature: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub affects_global_scope: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub implied_node_format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<super::m4y_2::BuildInfoFileInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadableBuildInfoDiagnostic {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_file: bool,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub pos: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub end: i32,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub code: i32,
    #[serde(
        default,
        skip_serializing_if = "category_is_zero",
        serialize_with = "serialize_category",
        deserialize_with = "deserialize_category"
    )]
    pub category: Category,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub message_chain: Vec<Box<ReadableBuildInfoDiagnostic>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_information: Vec<Box<ReadableBuildInfoDiagnostic>>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reports_unnecessary: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reports_deprecated: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub skipped_on_no_emit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repopulate_info: Option<ReadableBuildInfoRepopulateInfo>,
}

pub fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

fn category_is_zero(category: &Category) -> bool {
    *category as i32 == 0
}

fn serialize_category<S: serde::Serializer>(
    category: &Category,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_i32(*category as i32)
}

fn deserialize_category<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Category, D::Error> {
    Ok(match i32::deserialize(deserializer)? {
        1 => Category::Error,
        2 => Category::Suggestion,
        3 => Category::Message,
        _ => Category::Warning,
    })
}

fn resolution_mode_is_none(mode: &ResolutionMode) -> bool {
    *mode as i32 == 0
}

fn serialize_resolution_mode<S: serde::Serializer>(
    mode: &ResolutionMode,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_i32(*mode as i32)
}

fn deserialize_resolution_mode<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ResolutionMode, D::Error> {
    Ok(match i32::deserialize(deserializer)? {
        1 => ResolutionMode::CommonJS,
        99 => ResolutionMode::ESNext,
        _ => ResolutionMode::None,
    })
}

fn serialize_repopulate_diagnostic_kind<S: serde::Serializer>(
    kind: &RepopulateDiagnosticKind,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_i32(*kind as i32)
}

fn deserialize_repopulate_diagnostic_kind<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<RepopulateDiagnosticKind, D::Error> {
    Ok(match i32::deserialize(deserializer)? {
        2 => RepopulateDiagnosticKind::ModuleNotFound,
        _ => RepopulateDiagnosticKind::ModeMismatch,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadableBuildInfoRepopulateInfo {
    #[serde(
        serialize_with = "serialize_repopulate_diagnostic_kind",
        deserialize_with = "deserialize_repopulate_diagnostic_kind"
    )]
    pub kind: RepopulateDiagnosticKind,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub module_reference: String,
    #[serde(
        default,
        skip_serializing_if = "resolution_mode_is_none",
        serialize_with = "serialize_resolution_mode",
        deserialize_with = "deserialize_resolution_mode"
    )]
    pub mode: ResolutionMode,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub package_name: String,
}

pub struct ReadableBuildInfoDiagnosticsOfFile {
    pub file: String,
    pub diagnostics: Vec<Box<ReadableBuildInfoDiagnostic>>,
}

impl Serialize for ReadableBuildInfoDiagnosticsOfFile {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (self.file.clone(), self.diagnostics.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReadableBuildInfoDiagnosticsOfFile {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let file_id_and_diagnostics = Vec::<serde_json::Value>::deserialize(deserializer)
            .map_err(|_| serde::de::Error::custom("invalid readableBuildInfoDiagnosticsOfFile"))?;
        if file_id_and_diagnostics.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "invalid readableBuildInfoDiagnosticsOfFile: expected 2 elements, got {}",
                file_id_and_diagnostics.len()
            )));
        }
        let file: String = serde_json::from_value(file_id_and_diagnostics[0].clone())
            .map_err(|_| {
                serde::de::Error::custom("invalid fileId in readableBuildInfoDiagnosticsOfFile: expected string")
            })?;
        let diagnostics: Vec<Box<ReadableBuildInfoDiagnostic>> =
            serde_json::from_value(file_id_and_diagnostics[1].clone()).map_err(|_| {
                serde::de::Error::custom(
                    "invalid diagnostics in readableBuildInfoDiagnosticsOfFile: expected []*readableBuildInfoDiagnostic",
                )
            })?;
        Ok(ReadableBuildInfoDiagnosticsOfFile { file, diagnostics })
    }
}
