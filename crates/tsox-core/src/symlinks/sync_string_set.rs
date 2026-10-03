#![allow(unused_imports)]

use super::*;

pub type SyncStringSet = Arc<Mutex<HashSet<String>>>;

pub(crate) fn new_sync_string_set() -> SyncStringSet { crate::fntrace::enter("new_sync_string_set"); 
    Arc::new(Mutex::new(HashSet::new()))
}

#[derive(Clone, Debug)]
pub struct KnownDirectoryLink {
    pub real: String,

    pub real_path: Path,
}

pub struct KnownSymlinks {
    pub(crate) directories: SyncMap<Path, KnownDirectoryLink>,
    pub(crate) directories_by_realpath: SyncMap<Path, SyncStringSet>,
    pub(crate) files: SyncMap<Path, String>,
    pub(crate) files_by_realpath: SyncMap<Path, SyncStringSet>,
    pub cwd: String,
    pub use_case_sensitive_file_names: bool,
}

impl KnownSymlinks {
    pub fn new(current_directory: &str, use_case_sensitive_file_names: bool) -> Self { crate::fntrace::enter("new"); 
        Self {
            directories: SyncMap::new(),
            directories_by_realpath: SyncMap::new(),
            files: SyncMap::new(),
            files_by_realpath: SyncMap::new(),
            cwd: current_directory.to_string(),
            use_case_sensitive_file_names,
        }
    }

    pub fn has_directory(&self, symlink_path: &Path) -> bool { crate::fntrace::enter("has_directory"); 
        let p = symlink_path.ensure_trailing_directory_separator();
        self.directories.load(&p).is_some()
    }

    pub fn directories(&self) -> &SyncMap<Path, KnownDirectoryLink> { crate::fntrace::enter("directories"); 
        &self.directories
    }

    pub fn directories_by_realpath(&self) -> &SyncMap<Path, SyncStringSet> { crate::fntrace::enter("directories_by_realpath"); 
        &self.directories_by_realpath
    }

    pub fn files(&self) -> &SyncMap<Path, String> { crate::fntrace::enter("files"); 
        &self.files
    }

    pub fn files_by_realpath(&self) -> &SyncMap<Path, SyncStringSet> { crate::fntrace::enter("files_by_realpath"); 
        &self.files_by_realpath
    }

    pub fn set_directory(
        &self,
        symlink: &str,
        symlink_path: Path,
        real_directory: KnownDirectoryLink,
    ) { crate::fntrace::enter("set_directory"); 
        if self.directories.load(&symlink_path).is_none() {
            let (set, _) = self
                .directories_by_realpath
                .load_or_store(real_directory.real_path.clone(), new_sync_string_set());
            set.lock().unwrap().insert(symlink.to_string());
        }
        self.directories.store(symlink_path, real_directory);
    }

    pub fn set_file(&self, symlink: &str, symlink_path: Path, realpath: &str) { crate::fntrace::enter("set_file"); 
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

    pub fn process_resolution(&self, original_path: &str, resolved_file_name: &str) { crate::fntrace::enter("process_resolution"); 
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

    pub fn guess_directory_symlink(&self, a: &str, b: &str, cwd: &str) -> (String, String) { crate::fntrace::enter("guess_directory_symlink"); 
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

    pub fn is_node_modules_or_scoped_package_directory(&self, s: &str) -> bool { crate::fntrace::enter("is_node_modules_or_scoped_package_directory"); 
        !s.is_empty()
            && (tspath::get_canonical_file_name(s, self.use_case_sensitive_file_names)
                == "node_modules"
                || s.starts_with('@'))
    }

    pub fn set_symlinks_from_resolutions(
        &self,
        for_each_resolved_module: impl Fn(&dyn Fn(&str, &str)),
        for_each_resolved_type_reference_directive: impl Fn(&dyn Fn(&str, &str)),
    ) { crate::fntrace::enter("set_symlinks_from_resolutions"); 
        for_each_resolved_module(&|original_path, resolved_file_name| {
            self.process_resolution(original_path, resolved_file_name);
        });
        for_each_resolved_type_reference_directive(&|original_path, resolved_file_name| {
            self.process_resolution(original_path, resolved_file_name);
        });
    }
}
