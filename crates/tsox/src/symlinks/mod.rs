use crate::collections::syncmap::SyncMap;
use crate::tspath::{self, Path};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub type SyncStringSet = Arc<Mutex<HashSet<String>>>;

fn new_sync_string_set() -> SyncStringSet {
    Arc::new(Mutex::new(HashSet::new()))
}

#[derive(Clone, Debug)]
pub struct KnownDirectoryLink {

    pub real: String,

    pub real_path: Path,
}

pub struct KnownSymlinks {
    directories: SyncMap<Path, KnownDirectoryLink>,
    directories_by_realpath: SyncMap<Path, SyncStringSet>,
    files: SyncMap<Path, String>,
    files_by_realpath: SyncMap<Path, SyncStringSet>,
    pub cwd: String,
    pub use_case_sensitive_file_names: bool,
}

impl KnownSymlinks {

    pub fn new(current_directory: &str, use_case_sensitive_file_names: bool) -> Self {
        Self {
            directories: SyncMap::new(),
            directories_by_realpath: SyncMap::new(),
            files: SyncMap::new(),
            files_by_realpath: SyncMap::new(),
            cwd: current_directory.to_string(),
            use_case_sensitive_file_names,
        }
    }

    pub fn has_directory(&self, symlink_path: &Path) -> bool {
        let p = symlink_path.ensure_trailing_directory_separator();
        self.directories.load(&p).is_some()
    }

    pub fn directories(&self) -> &SyncMap<Path, KnownDirectoryLink> {
        &self.directories
    }

    pub fn directories_by_realpath(&self) -> &SyncMap<Path, SyncStringSet> {
        &self.directories_by_realpath
    }

    pub fn files(&self) -> &SyncMap<Path, String> {
        &self.files
    }

    pub fn files_by_realpath(&self) -> &SyncMap<Path, SyncStringSet> {
        &self.files_by_realpath
    }

    pub fn set_directory(
        &self,
        symlink: &str,
        symlink_path: Path,
        real_directory: KnownDirectoryLink,
    ) {
        if self.directories.load(&symlink_path).is_none() {
            let (set, _) = self
                .directories_by_realpath
                .load_or_store(real_directory.real_path.clone(), new_sync_string_set());
            set.lock().unwrap().insert(symlink.to_string());
        }
        self.directories.store(symlink_path, real_directory);
    }

    pub fn set_file(&self, symlink: &str, symlink_path: Path, realpath: &str) {
        if self.files.load(&symlink_path).is_none() {
            let realpath_path =
                tspath::to_path(realpath, &self.cwd, self.use_case_sensitive_file_names);
            let (set, _) = self
                .files_by_realpath
                .load_or_store(realpath_path, new_sync_string_set());
            set.lock().unwrap().insert(symlink.to_string());
        }
        self.files.store(symlink_path, realpath.to_string());
    }

    pub fn process_resolution(&self, original_path: &str, resolved_file_name: &str) {
        if original_path.is_empty() || resolved_file_name.is_empty() {
            return;
        }
        self.set_file(
            original_path,
            tspath::to_path(original_path, &self.cwd, self.use_case_sensitive_file_names),
            resolved_file_name,
        );
        let (common_resolved, common_original) =
            self.guess_directory_symlink(resolved_file_name, original_path, &self.cwd.clone());
        if !common_resolved.is_empty() && !common_original.is_empty() {
            let symlink_path = tspath::to_path(
                &common_original,
                &self.cwd,
                self.use_case_sensitive_file_names,
            );
            if !tspath::contains_ignored_path(symlink_path.as_str()) {
                self.set_directory(
                    &common_original,
                    symlink_path.ensure_trailing_directory_separator(),
                    KnownDirectoryLink {
                        real: tspath::ensure_trailing_directory_separator(&common_resolved),
                        real_path: tspath::to_path(
                            &common_resolved,
                            &self.cwd,
                            self.use_case_sensitive_file_names,
                        )
                        .ensure_trailing_directory_separator(),
                    },
                );
            }
        }
    }

    pub fn guess_directory_symlink(&self, a: &str, b: &str, cwd: &str) -> (String, String) {
        let mut a_parts =
            tspath::get_path_components(&tspath::get_normalized_absolute_path(a, cwd), "");
        let mut b_parts =
            tspath::get_path_components(&tspath::get_normalized_absolute_path(b, cwd), "");
        let mut is_directory = false;
        while a_parts.len() >= 2
            && b_parts.len() >= 2
            && !self.is_node_modules_or_scoped_package_directory(&a_parts[a_parts.len() - 2])
            && !self.is_node_modules_or_scoped_package_directory(&b_parts[b_parts.len() - 2])
            && tspath::get_canonical_file_name(
                &a_parts[a_parts.len() - 1],
                self.use_case_sensitive_file_names,
            ) == tspath::get_canonical_file_name(
                &b_parts[b_parts.len() - 1],
                self.use_case_sensitive_file_names,
            )
        {
            a_parts.pop();
            b_parts.pop();
            is_directory = true;
        }
        if is_directory {
            (
                tspath::get_path_from_path_components(&a_parts),
                tspath::get_path_from_path_components(&b_parts),
            )
        } else {
            (String::new(), String::new())
        }
    }

    pub fn is_node_modules_or_scoped_package_directory(&self, s: &str) -> bool {
        !s.is_empty()
            && (tspath::get_canonical_file_name(s, self.use_case_sensitive_file_names)
                == "node_modules"
                || s.starts_with('@'))
    }

    pub fn set_symlinks_from_resolutions(
        &self,
        for_each_resolved_module: impl Fn(&dyn Fn(&str, &str)),
        for_each_resolved_type_reference_directive: impl Fn(&dyn Fn(&str, &str)),
    ) {
        for_each_resolved_module(&|original_path, resolved_file_name| {
            self.process_resolution(original_path, resolved_file_name);
        });
        for_each_resolved_type_reference_directive(&|original_path, resolved_file_name| {
            self.process_resolution(original_path, resolved_file_name);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_known_symlink() {
        let cache = KnownSymlinks::new("/test/dir", true);
        assert_eq!(cache.cwd, "/test/dir");
        assert!(cache.use_case_sensitive_file_names);
    }

    #[test]
    fn test_set_directory() {
        let cache = KnownSymlinks::new("/test/dir", true);
        let symlink_path = tspath::to_path("/test/symlink", "/test/dir", true)
            .ensure_trailing_directory_separator();
        let real_directory = KnownDirectoryLink {
            real: "/real/path/".to_string(),
            real_path: tspath::to_path("/real/path", "/test/dir", true)
                .ensure_trailing_directory_separator(),
        };

        cache.set_directory(
            "/test/symlink",
            symlink_path.clone(),
            real_directory.clone(),
        );

        let stored = cache.directories().load(&symlink_path);
        assert!(stored.is_some(), "Expected directory to be stored");
        let stored = stored.unwrap();
        assert_eq!(stored.real, real_directory.real);
        assert_eq!(stored.real_path, real_directory.real_path);

        let set = cache
            .directories_by_realpath()
            .load(&real_directory.real_path);
        assert!(
            set.is_some() && !set.as_ref().unwrap().lock().unwrap().is_empty(),
            "Expected realpath mapping to be created"
        );
        assert!(
            set.unwrap().lock().unwrap().contains("/test/symlink"),
            "Expected symlink '/test/symlink' to be in set"
        );
    }

    #[test]
    fn test_set_file() {
        let cache = KnownSymlinks::new("/test/dir", true);
        let symlink = "/test/symlink/file.ts";
        let symlink_path = tspath::to_path(symlink, "/test/dir", true);
        let realpath = "/real/path/file.ts";

        cache.set_file(symlink, symlink_path.clone(), realpath);

        let stored = cache.files().load(&symlink_path);
        assert!(stored.is_some(), "Expected file to be stored");
        assert_eq!(stored.unwrap(), realpath);
    }

    #[test]
    fn test_process_resolution() {
        let cache = KnownSymlinks::new("/test/dir", true);

        cache.process_resolution("", "");
        cache.process_resolution("original", "");
        cache.process_resolution("", "resolved");

        let original_path = "/test/original/file.ts";
        let resolved_path = "/test/resolved/file.ts";
        cache.process_resolution(original_path, resolved_path);

        let symlink_path = tspath::to_path(original_path, "/test/dir", true);
        let stored = cache.files().load(&symlink_path);
        assert!(stored.is_some(), "Expected file to be stored");
        assert_eq!(stored.unwrap(), resolved_path);
    }

    #[test]
    fn test_guess_directory_symlink() {
        let cache = KnownSymlinks::new("/test/dir", true);

        let cases: &[(&str, &str, &str, &str, &str, &str)] = &[

            (
                "identical paths",
                "/test/path/file.ts",
                "/test/path/file.ts",
                "/test/dir",
                "/",
                "/",
            ),
            (
                "different files same directory",
                "/test/path/file1.ts",
                "/test/path/file2.ts",
                "/test/dir",
                "",
                "",
            ),
            (
                "different directories",
                "/test/path1/file.ts",
                "/test/path2/file.ts",
                "/test/dir",
                "/test/path1",
                "/test/path2",
            ),
            (
                "node_modules paths",
                "/test/node_modules/pkg/file.ts",
                "/test/node_modules/pkg/file.ts",
                "/test/dir",
                "/test/node_modules/pkg",
                "/test/node_modules/pkg",
            ),
            (
                "scoped package paths",
                "/test/node_modules/@scope/pkg/file.ts",
                "/test/node_modules/@scope/pkg/file.ts",
                "/test/dir",
                "/test/node_modules/@scope/pkg",
                "/test/node_modules/@scope/pkg",
            ),
        ];

        for (name, a, b, cwd, expected_resolved, expected_original) in cases {
            let (common_resolved, common_original) = cache.guess_directory_symlink(a, b, cwd);
            assert_eq!(
                common_resolved, *expected_resolved,
                "{name}: expected common_resolved to be '{expected_resolved}', got '{common_resolved}'"
            );
            assert_eq!(
                common_original, *expected_original,
                "{name}: expected common_original to be '{expected_original}', got '{common_original}'"
            );
        }
    }

    #[test]
    fn test_is_node_modules_or_scoped_package_directory() {
        let cache = KnownSymlinks::new("/test/dir", true);

        let cases: &[(&str, &str, bool)] = &[
            ("node_modules", "node_modules", true),
            ("scoped package", "@scope", true),
            ("regular directory", "src", false),
            ("empty string", "", false),
            ("case insensitive node_modules", "NODE_MODULES", false),
            ("case insensitive scoped", "@SCOPE", true),
        ];

        for (name, dir, expected) in cases {
            let result = cache.is_node_modules_or_scoped_package_directory(dir);
            assert_eq!(
                result, *expected,
                "{name}: expected {expected}, got {result} for directory '{dir}'"
            );
        }
    }

    #[test]
    fn test_set_symlinks_from_resolutions() {
        let cache = KnownSymlinks::new("/test/dir", true);

        let resolved_modules: &[(&str, &str)] = &[
            ("/test/original/file1.ts", "/test/resolved/file1.ts"),
            ("/test/original/file2.ts", "/test/resolved/file2.ts"),
        ];

        cache.set_symlinks_from_resolutions(
            |cb| {
                for &(original, resolved) in resolved_modules {
                    cb(original, resolved);
                }
            },
            |_| {},
        );

        for &(original, resolved) in resolved_modules {
            let symlink_path = tspath::to_path(original, "/test/dir", true);
            let stored = cache.files().load(&symlink_path);
            assert!(stored.is_some(), "Expected file '{original}' to be stored");
            assert_eq!(stored.unwrap(), resolved);
        }
    }

    #[test]
    fn test_known_symlinks_thread_safety() {
        use std::thread;

        let cache = KnownSymlinks::new("/test/dir", true);

        thread::scope(|s| {
            for id in 0..10u32 {
                let cache_ref = &cache;
                s.spawn(move || {
                    let symlink = format!("/test/symlink{id}");
                    let symlink_path = tspath::to_path(&symlink, "/test/dir", true)
                        .ensure_trailing_directory_separator();
                    let real_directory = KnownDirectoryLink {
                        real: format!("/real/path{id}/"),
                        real_path: tspath::to_path(&format!("/real/path{id}"), "/test/dir", true)
                            .ensure_trailing_directory_separator(),
                    };

                    cache_ref.set_directory(&symlink, symlink_path.clone(), real_directory.clone());

                    let stored = cache_ref.directories().load(&symlink_path);
                    assert!(
                        stored.is_some(),
                        "Goroutine {id}: Expected directory to be stored"
                    );
                    assert_eq!(
                        stored.unwrap().real,
                        real_directory.real,
                        "Goroutine {id}: Expected Real to be '{}'",
                        real_directory.real
                    );
                });
            }
        });

        assert_eq!(
            cache.directories().len(),
            10,
            "Expected 10 directories to be stored"
        );
    }
}
