#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Weak};

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::symlinks::{KnownDirectoryLink, KnownSymlinks};
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::module::ResolutionHost;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::compiler::ProgramOptions;
use super::m4x_2::ProjectReferenceFileMapper;

pub struct ProjectReferenceDtsFakingHost {
    pub host: Arc<dyn crate::compiler::CompilerHost>,
    pub fs: CachedVfs,
}

pub type CachedVfs = ProjectReferenceDtsFakingVfs;

impl ResolutionHost for ProjectReferenceDtsFakingHost {
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS {
        &self.fs
    }

    fn get_current_directory(&self) -> &str {
        self.host.current_directory()
    }
}

impl tsox_tsoptions::vfs::FS for ProjectReferenceDtsFakingVfs {
    fn use_case_sensitive_file_names(&self) -> bool {
        ProjectReferenceDtsFakingVfs::use_case_sensitive_file_names(self)
    }

    fn file_exists(&self, path: &str) -> bool {
        ProjectReferenceDtsFakingVfs::file_exists(self, path)
    }

    fn read_file(&self, path: &str) -> Option<String> {
        ProjectReferenceDtsFakingVfs::read_file(self, path)
    }

    fn write_file(&self, _path: &str, _data: &str) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    fn append_file(&self, _path: &str, _data: &str) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    fn remove(&self, _path: &str) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    fn chtimes(
        &self,
        _path: &str,
        _a_time: std::time::SystemTime,
        _m_time: std::time::SystemTime,
    ) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    fn directory_exists(&self, path: &str) -> bool {
        ProjectReferenceDtsFakingVfs::directory_exists(self, path)
    }

    fn get_accessible_entries(&self, _path: &str) -> tsox_tsoptions::vfs::Entries {
        panic!("should not be called by resolver")
    }

    fn stat(&self, _path: &str) -> Option<tsox_tsoptions::vfs::FileInfo> {
        panic!("should not be called by resolver")
    }

    fn realpath(&self, path: &str) -> String {
        ProjectReferenceDtsFakingVfs::realpath(self, path)
    }

    fn walk_dir(
        &self,
        _root: &str,
        _walk_fn: &mut dyn FnMut(&str, &tsox_tsoptions::vfs::FileInfo),
    ) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }
}

pub fn new_project_reference_dts_faking_host(
    host: Arc<dyn crate::compiler::CompilerHost>,
    project_reference_file_mapper: Weak<ProjectReferenceFileMapper>,
    dts_directories: Set<Path>,
) -> ProjectReferenceDtsFakingHost {
    ProjectReferenceDtsFakingHost {
        host: host.clone(),
        fs: ProjectReferenceDtsFakingVfs {
            project_reference_file_mapper,
            dts_directories,
            known_symlinks: KnownSymlinks::new(
                host.current_directory(),
                host.use_case_sensitive_file_names(),
            ),
        },
    }
}

pub struct ProjectReferenceDtsFakingVfs {
    pub project_reference_file_mapper: Weak<ProjectReferenceFileMapper>,
    pub dts_directories: Set<Path>,
    pub known_symlinks: KnownSymlinks,
}

impl ProjectReferenceDtsFakingVfs {
    fn mapper_host(&self) -> Arc<dyn crate::compiler::CompilerHost> {
        self.project_reference_file_mapper
            .upgrade()
            .expect("project reference file mapper outlives its faking host")
            .host_compiler_host()
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.mapper_host().use_case_sensitive_file_names()
    }

    pub fn file_exists(&self, path: &str) -> bool {
        if self.mapper_host().fs().file_exists(path) {
            return true;
        }
        if !tsox_core::tspath::is_declaration_file_name(path) {
            return false;
        }
        self.file_or_directory_exists_using_source(path, true)
    }

    pub fn read_file(&self, path: &str) -> Option<String> {
        self.mapper_host().fs().read_file(path)
    }

    pub fn write_file(&self, path: &str, data: &str) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    pub fn append_file(&self, path: &str, data: &str) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    pub fn remove(&self, path: &str) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    pub fn chtimes(
        &self,
        path: &str,
        a_time: std::time::SystemTime,
        m_time: std::time::SystemTime,
    ) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    pub fn directory_exists(&self, path: &str) -> bool {
        if self.mapper_host().fs().directory_exists(path) {
            self.handle_directory_could_be_symlink(path);
            return true;
        }
        self.file_or_directory_exists_using_source(path, false)
    }

    pub fn get_accessible_entries(&self, path: &str) -> tsox_tsoptions::vfs::Entries {
        panic!("should not be called by resolver")
    }

    pub fn stat(&self, path: &str) -> tsox_tsoptions::vfs::FileInfo {
        panic!("should not be called by resolver")
    }

    pub fn walk_dir(
        &self,
        root: &str,
        walk_fn: &mut dyn FnMut(&str, &tsox_tsoptions::vfs::Entries) -> Result<(), std::io::Error>,
    ) -> Result<(), std::io::Error> {
        panic!("should not be called by resolver")
    }

    pub fn realpath(&self, path: &str) -> String {
        if let Some(result) = self.known_symlinks.files().load(&self.to_path(path)) {
            return result;
        }
        self.mapper_host().fs().realpath(path)
    }

    pub fn to_path(&self, path: &str) -> Path {
        tspath::to_path(
            path,
            self.mapper_host().current_directory(),
            self.use_case_sensitive_file_names(),
        )
    }

    pub fn handle_directory_could_be_symlink(&self, directory: &str) {
        if tspath::contains_ignored_path(directory) {
            return;
        }
        if !directory.contains("/node_modules/") {
            return;
        }

        let directory_path = Path(tspath::ensure_trailing_directory_separator(
            &self.to_path(directory).0,
        ));
        if self
            .known_symlinks
            .directories()
            .load(&directory_path)
            .is_some()
        {
            return;
        }

        let real_directory = self.realpath(directory);
        if real_directory == directory {
            return;
        }
        let real_path = Path(tspath::ensure_trailing_directory_separator(
            &self.to_path(&real_directory).0,
        ));
        if real_path == directory_path {
            return;
        }
        self.known_symlinks.set_directory(
            directory,
            directory_path,
            KnownDirectoryLink {
                real: tspath::ensure_trailing_directory_separator(&real_directory),
                real_path,
            },
        );
    }

    pub fn file_or_directory_exists_using_source(
        &self,
        file_or_directory: &str,
        is_file: bool,
    ) -> bool {
        let mut result = if is_file {
            self.file_exists_if_project_reference_dts(file_or_directory)
        } else {
            self.directory_exists_if_project_reference_decl_dir(file_or_directory)
        };
        if result != Tristate::Unknown {
            return result == Tristate::True;
        }

        let file_or_directory_path = self.to_path(file_or_directory);
        if !file_or_directory_path.0.contains("/node_modules/") {
            return false;
        }
        let package_root =
            tsox_tsoptions::module::parse_node_module_from_path(file_or_directory, true);
        if !package_root.is_empty() {
            self.handle_directory_could_be_symlink(&package_root);
        }
        if self.known_symlinks.directories().is_empty() {
            return false;
        }
        if is_file && self.known_symlinks.files().load(&file_or_directory_path).is_some() {
            return true;
        }

        let mut exists = false;
        let current_directory = self.mapper_host().current_directory().to_string();
        let entries = self.known_symlinks.directories().to_hash_map();
        for (directory_path, known_directory_link) in entries {
            let relative = match file_or_directory_path
                .0
                .strip_prefix(&directory_path.0)
            {
                Some(relative) => relative.to_string(),
                None => continue,
            };
            let candidate = format!("{}{}", known_directory_link.real_path.0, relative);
            exists = if is_file {
                self.file_exists_if_project_reference_dts(&candidate) == Tristate::True
            } else {
                self.directory_exists_if_project_reference_decl_dir(&candidate) == Tristate::True
            };
            if exists {
                if is_file {
                    let absolute_path = tspath::get_normalized_absolute_path(
                        file_or_directory,
                        &current_directory,
                    );
                    let mapped = format!(
                        "{}{}",
                        known_directory_link.real,
                        &absolute_path[directory_path.0.len()..]
                    );
                    self.known_symlinks.set_file(
                        &absolute_path,
                        file_or_directory_path.clone(),
                        &mapped,
                    );
                }
                break;
            }
        }
        exists
    }

    pub fn file_exists_if_project_reference_dts(&self, file: &str) -> Tristate {
        let Some(mapper) = self.project_reference_file_mapper.upgrade() else {
            return Tristate::Unknown;
        };
        if let Some(source) = mapper.get_project_reference_from_output_dts(&self.to_path(file))
        {
            return if self
                .mapper_host()
                .fs()
                .file_exists(&source.source)
            {
                Tristate::True
            } else {
                Tristate::False
            };
        }
        Tristate::Unknown
    }

    pub fn directory_exists_if_project_reference_decl_dir(&self, dir: &str) -> Tristate {
        let dir_path = self.to_path(dir);
        for decl_dir_path in self.dts_directories.iter() {
            if dir_path.contains_path(decl_dir_path) || decl_dir_path.contains_path(&dir_path) {
                return Tristate::True;
            }
        }
        Tristate::Unknown
    }
}
