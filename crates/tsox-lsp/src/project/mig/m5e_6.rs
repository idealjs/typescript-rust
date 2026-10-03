#![allow(dead_code, unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use tsox_core::tspath::Path;

use crate::project::session::*;

use crate::project::config_file_registry::ConfigFileRegistry;
use crate::project::parse_cache::ParseCacheKey;
use crate::project::project::{Kind as ProjectKind, ProgramUpdateKind, Project};
use crate::project::snapshot_fs::SnapshotFS;
use crate::project::watch::WatchedFiles;
use crate::project::overlay_fs::FileHandle as _;
use tsox_core::core::compiler_options::CompilerOptions;

pub(crate) use super::m5e::ContentMapperContributions;
use super::m5d_5::{new_project_collection_builder, DirtyMapMigExt};
use super::m5e_7::{RealpathAliasSet, SnapshotFSBuilder, new_snapshot_fs_builder};
use crate::project::mig::m5d_2::{parse_cache_key_for_duplicate, parse_cache_key_for_file};

mod core {
    pub use tsox_core::core::project_reference::ProjectReference;
}

mod ast {
    pub use tsox_frontend::ast::{Diagnostic, SourceFile};
}

mod ls {
    pub use crate::ls::cross_project::Project;
}

mod lsconv {
    pub use crate::ls::lsconv_converters::Converters;
    pub use crate::ls::lsconv_converters::PositionEncodingKind;
    pub use crate::ls::lsconv_linemap::LspLineMap;
    pub use crate::mig::m5u_conv::new_converters;
}

mod sourcemap {
    pub use crate::mig::m6b::EcmaLineInfo;
}

mod autoimport {
    pub use crate::ls::autoimport_registry::*;
}

mod vfsmatch {
    pub use tsox_tsoptions::vfs::vfsmatch::read_directory;
}

trait SourceFileContentMapperExt {
    fn content_mapper(&self) -> String;
    fn is_content_mapper_failure_stub(&self) -> bool;
    fn is_content_mapper_supplemental(&self) -> bool;
}

impl SourceFileContentMapperExt for ast::SourceFile {
    fn content_mapper(&self) -> String { ::tsox_core::fntrace::enter("content_mapper"); 
        tsox_compile::mig::m3l_cm_2::content_mapper_source_file_info(&self.file_name)
            .map(|info| info.content_mapper)
            .unwrap_or_default()
    }

    fn is_content_mapper_supplemental(&self) -> bool { ::tsox_core::fntrace::enter("is_content_mapper_supplemental"); 
        tsox_compile::mig::m3l_cm_2::content_mapper_source_file_info(&self.file_name)
            .map(|info| info.canonical_source_file.is_some())
            .unwrap_or(false)
    }

    fn is_content_mapper_failure_stub(&self) -> bool { ::tsox_core::fntrace::enter("is_content_mapper_failure_stub"); 
        tsox_frontend::ast::mig::m3b_2::is_content_mapper_failure_stub(self)
    }
}

impl crate::ls::autoimport::RegistryCloneHost
    for crate::project::auto_import::AutoImportRegistryCloneHost
{
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS { ::tsox_core::fntrace::enter("fs"); 
        todo!("AutoImportRegistryCloneHost::fs requires snapshotFSBuilder integration")
    }

    fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        <crate::project::auto_import::AutoImportRegistryCloneHost as crate::project::auto_import::RegistryCloneHost>::get_current_directory(self)
    }

    fn get_default_project(
        &self,
        _path: &Path,
    ) -> (Path, Option<Arc<tsox_compile::compiler::Program>>) { ::tsox_core::fntrace::enter("get_default_project"); 
        (Path::default(), None)
    }

    fn get_program_for_project(
        &self,
        _project_path: &Path,
    ) -> Option<Arc<tsox_compile::compiler::Program>> { ::tsox_core::fntrace::enter("get_program_for_project"); 
        None
    }

    fn get_package_json(
        &self,
        _file_name: &str,
    ) -> Option<crate::project::auto_import::PackageJsonInfoCacheEntry> { ::tsox_core::fntrace::enter("get_package_json"); 
        None
    }

    fn get_source_file(&self, _file_name: &str, _path: &Path) -> Option<Arc<ast::SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        None
    }

    fn dispose(&self) { ::tsox_core::fntrace::enter("dispose"); }
}

fn position_encoding_from_protocol(encoding: &str) -> lsconv::PositionEncodingKind { ::tsox_core::fntrace::enter("position_encoding_from_protocol"); 
    if encoding == crate::lsp::lsproto::POSITION_ENCODING_UTF8 {
        lsconv::PositionEncodingKind::Utf8
    } else if encoding == crate::lsp::lsproto::POSITION_ENCODING_UTF32 {
        lsconv::PositionEncodingKind::Utf32
    } else {
        lsconv::PositionEncodingKind::Utf16
    }
}

fn empty_lsp_line_map() -> lsconv::LspLineMap { ::tsox_core::fntrace::enter("empty_lsp_line_map"); 
    crate::ls::lsconv_linemap::compute_lsp_line_starts("")
}

fn new_snapshot_line_map_closure() -> Box<dyn Fn(&str) -> lsconv::LspLineMap + Send + Sync> { ::tsox_core::fntrace::enter("new_snapshot_line_map_closure"); 
    let snapshot_cell: OnceLock<Arc<Snapshot>> = OnceLock::new();
    Box::new(move |file_name: &str| {
        snapshot_cell
            .get()
            .and_then(|s| s.lsp_line_map(file_name))
            .unwrap_or_else(empty_lsp_line_map)
    })
}

fn wire_snapshot_line_map_closure(new_snapshot: &mut Arc<Snapshot>) { ::tsox_core::fntrace::enter("wire_snapshot_line_map_closure"); 
    let cell = Arc::downgrade(new_snapshot);
    let converters = Arc::get_mut(new_snapshot)
        .expect("freshly created snapshot is uniquely held")
        .converters
        .as_mut();
    if let Some(converters) = converters {
        converters.get_line_map = Box::new(move |file_name: &str| {
            cell.upgrade()
                .and_then(|s| s.lsp_line_map(file_name))
                .unwrap_or_else(empty_lsp_line_map)
        });
    }
}

pub fn new_snapshot(
    id: u64,
    fs: Arc<SnapshotFS>,
    session_options: SessionOptions,
    config_file_registry: Box<ConfigFileRegistry>,
    compiler_options_for_inferred_projects: Option<CompilerOptions>,
    user_preferences: UserPreferences,
    auto_imports: Option<Arc<autoimport::Registry>>,
    auto_imports_watch: Option<Arc<WatchedFiles<HashMap<Path, String>>>>,
) -> Snapshot { ::tsox_core::fntrace::enter("new_snapshot"); 
    let mut s = Snapshot::new(id);
    s.fs = Some(fs);
    s.config_file_registry = Some(config_file_registry);
    s.compiler_options_for_inferred_projects = compiler_options_for_inferred_projects;
    s.user_preferences = user_preferences;
    s.auto_imports = auto_imports;
    s.auto_imports_watch = auto_imports_watch;
    s.converters = Some(lsconv::new_converters(
        position_encoding_from_protocol(&session_options.position_encoding),
        new_snapshot_line_map_closure(),
    ));
    s
}

impl Snapshot {
    pub fn content_mapper_watch_state(&self) -> (&Vec<String>, &HashSet<Path>) { ::tsox_core::fntrace::enter("content_mapper_watch_state"); 
        let (extensions, watched_files) = self
            .content_mapper_watch_state_once
            .get_or_init(|| {
                let configured = self.config_file_registry().content_mappers();
                let mut extensions = configured.extensions.clone();
                extensions.extend(self.inferred_project_content_mapper_extensions.iter().cloned());
                extensions.sort();
                extensions.dedup();
                let mut watched_files = HashSet::new();
                for project in self.projects() {
                    if let Some(files) = project.content_mapper_watched_files() {
                        watched_files.extend(files.iter().cloned());
                    }
                }
                (extensions, watched_files)
            });
        (extensions, watched_files)
    }

    pub fn clone_snapshot(
        &self,
        change: SnapshotChange,
        overlays: Option<&HashMap<Path, Arc<Overlay>>>,
        session: &Session,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("clone_snapshot"); 
        let start = std::time::Instant::now();
        let mut inferred_content_mappers = self.inferred_project_content_mappers.clone();
        let mut inferred_content_mapper_extensions = self.inferred_project_content_mapper_extensions.clone();
        if let Some(contributions) = &change.content_mapper_contributions {
            inferred_content_mappers = contributions
                .mappers
                .iter()
                .map(|m| (**m).clone())
                .collect();
            inferred_content_mapper_extensions = contributions.extensions.clone();
        }
        let overlays = overlays.cloned().unwrap_or_else(|| {
            self.fs
                .as_ref()
                .expect("fs")
                .overlays
                .clone()
        });
        let fs_snapshot = self.fs.as_ref().expect("fs");
        let mut fs = new_snapshot_fs_builder(
            session.fs_fs(),
            &fs_snapshot.overlays,
            &overlays,
            &fs_snapshot.disk_files,
            &self.fs_disk_directories(),
            &self.fs_node_modules_realpath_aliases(),
            session.options.position_encoding.clone(),
            self.to_path_fn(),
        );
        let mut change = change;
        change.file_changes = self.process_file_changes(&mut fs, change.file_changes, change.content_mapper_contributions.as_ref());
        let compiler_options_for_inferred_projects = change
            .compiler_options_for_inferred_projects
            .clone()
            .or_else(|| self.compiler_options_for_inferred_projects.clone());
        let custom_config_file_name = change
            .new_config
            .as_ref()
            .map(|c| c.custom_config_file_name.clone())
            .unwrap_or_else(|| self.config_file_registry().custom_config_file_name.clone());
        let new_snapshot_id = session.next_snapshot_id();
        let mut builder = new_project_collection_builder(
            crate::mig::m5m::ResolvedClientCapabilitiesContext {
                capabilities: None,
            },
            new_snapshot_id,
            &mut fs,
            self.project_collection(),
            self.config_file_registry(),
            &self.project_collection().api_state,
            compiler_options_for_inferred_projects.clone(),
            inferred_content_mappers.clone(),
            inferred_content_mapper_extensions.clone(),
            session.options.clone(),
            custom_config_file_name,
            session.parse_cache.clone().expect("parse cache"),
            session
                .content_mapped_parse_cache
                .clone()
                .expect("content mapped parse cache"),
            session
                .extended_config_cache
                .clone()
                .expect("extended config cache"),
        );
        if !change.ata_changes.is_empty() {
            builder.did_update_ata_state(&change.ata_changes);
        }
        builder.did_change_custom_config_file_name();
        if !change.file_changes.is_empty() {
            builder.did_change_files(&change.file_changes);
        }
        for uri in &change.resource_request.documents {
            builder.did_request_file(uri, false);
        }
        for uri in &change.resource_request.configured_project_documents {
            builder.did_request_file(uri, true);
        }
        for project_id in &change.resource_request.projects {
            builder.did_request_project(project_id);
        }
        if let Some(project_tree) = &change.resource_request.project_tree {
            builder.did_request_project_trees(project_tree, None);
        }
        let (project_collection, config_file_registry) = builder.finalize();
        let mut projects_with_new_program_structure: HashMap<Path, bool> = HashMap::new();
        for project in project_collection.projects() {
            if project.program_last_update == new_snapshot_id
                && project.program_update_kind != ProgramUpdateKind::Cloned
            {
                projects_with_new_program_structure
                    .insert(project.config_file_path.clone(), project.program_update_kind == ProgramUpdateKind::NewFiles);
            }
        }
        let should_clean_disk_cache = change.clean_disk_cache
            || !change.file_changes.opened.0.is_empty()
            || !change.file_changes.reopened.0.is_empty()
            || !change.file_changes.closed.is_empty()
            || !change.file_changes.deleted.is_empty();
        if should_clean_disk_cache
            && (!projects_with_new_program_structure.is_empty() || change.clean_disk_cache)
        {
            fs.clean_disk_files_not_seen_by(&project_collection);
        }
        let config = change.new_config.clone().unwrap_or_else(|| self.user_preferences());
        let mut open_files: HashMap<Path, String> = HashMap::with_capacity(overlays.len());
        for (path, overlay) in &overlays {
            open_files.insert(path.clone(), overlay.file_name().to_string());
        }
        let prepare_auto_imports = if !change.resource_request.auto_imports.0.is_empty() {
            (session.to_path)(&change.resource_request.auto_imports.file_name())
        } else {
            Path::default()
        };
        let old_auto_imports = self
            .auto_imports
            .clone()
            .unwrap_or_else(|| {
                let to_path = self.to_path_fn();
                Arc::new(autoimport::Registry::new(
                    Box::new(move |file_name: &str| (to_path)(file_name)),
                    self.user_preferences(),
                ))
            });
        let registry_change = autoimport::RegistryChange {
            requested_file: prepare_auto_imports,
            open_files,
            changed: uri_set(change.file_changes.changed.iter().cloned()),
            created: uri_set(change.file_changes.created.iter().cloned()),
            deleted: uri_set(change.file_changes.deleted.iter().cloned()),
            rebuilt_programs: projects_with_new_program_structure,
            user_preferences: change.new_config.clone(),
        };
        let clone_host = crate::project::auto_import::AutoImportRegistryCloneHost::new(
            session.options.current_directory.clone(),
        );
        let (auto_imports, auto_imports_watch) =
            match old_auto_imports.clone_registry(&registry_change, &clone_host, None) {
            Ok(auto_imports) => {
                let watch = self
                    .auto_imports_watch
                    .clone()
                    .map(|w| Arc::new(w.clone_with_input(auto_imports.node_modules_directories())));
                (Some(Arc::new(auto_imports)), watch)
            }
            Err(_) => (None, None),
        };
        let snapshot_fs = Arc::new(fs.finalize());
        let builder_logs = if session.options.logging_enabled {
            Some(crate::project::logging_log_tree::new_log_tree(&format!(
                "Cloning snapshot {}",
                self.id
            )))
        } else {
            None
        };
        let mut new_snapshot = Arc::new(new_snapshot(
            new_snapshot_id,
            Arc::clone(&snapshot_fs),
            session.options.clone(),
            Box::new(config_file_registry),
            compiler_options_for_inferred_projects,
            config,
            auto_imports,
            auto_imports_watch.map(|w| w as Arc<WatchedFiles<HashMap<Path, String>>>),
        ));
        {
            let new_snapshot_ref = Arc::get_mut(&mut new_snapshot).unwrap();
            new_snapshot_ref.parent_id = self.id;
            new_snapshot_ref.project_collection = Some(Box::new(project_collection));
            new_snapshot_ref.builder_logs = builder_logs;
        }
        wire_snapshot_line_map_closure(&mut new_snapshot);
        for project in new_snapshot.projects() {
            if let Some(program) = project.get_program() {
                session.program_counter_ref(&program);
                if project.program_last_update == new_snapshot_id {
                    project.freeze_host(&snapshot_fs, new_snapshot.config_file_registry());
                }
            }
        }
        for file in new_snapshot.extended_source_files() {
            let path = (new_snapshot.fs.as_ref().expect("fs").to_path)(&file);
            session.extended_config_cache_add_owner(&path, new_snapshot.id);
        }
        if session.options.logging_enabled {
            session.logger.as_ref().expect("logger").log(&format!(
                "Finished cloning snapshot {} into snapshot {} in {:?}",
                self.id,
                new_snapshot.id,
                start.elapsed()
            ));
        }
        new_snapshot
    }

    pub fn clone_for_program(
        &self,
        root_file_names: Vec<String>,
        compiler_options: CompilerOptions,
        project_references: Vec<core::ProjectReference>,
        config_file_parsing_diagnostics: Vec<ast::Diagnostic>,
        old_project: Option<&Project>,
        mut file_changes: FileChangeSummary,
        session: &Session,
    ) -> Arc<Snapshot> { ::tsox_core::fntrace::enter("clone_for_program"); 
        let start = std::time::Instant::now();
        let fs_snapshot = self.fs.as_ref().expect("fs");
        let mut fs = new_snapshot_fs_builder(
            session.fs_fs(),
            &fs_snapshot.overlays,
            &fs_snapshot.overlays,
            &fs_snapshot.disk_files,
            &self.fs_disk_directories(),
            &self.fs_node_modules_realpath_aliases(),
            session.options.position_encoding.clone(),
            self.to_path_fn(),
        );
        file_changes = self.process_file_changes(&mut fs, file_changes, None);
        let new_snapshot_id = session.next_snapshot_id();
        let mut builder = new_project_collection_builder(
            crate::mig::m5m::ResolvedClientCapabilitiesContext {
                capabilities: None,
            },
            new_snapshot_id,
            &mut fs,
            self.project_collection(),
            self.config_file_registry(),
            &self.project_collection().api_state,
            Some(compiler_options.clone()),
            self.inferred_project_content_mappers.clone(),
            self.inferred_project_content_mapper_extensions.clone(),
            session.options.clone(),
            self.config_file_registry().custom_config_file_name.clone(),
            session.parse_cache.clone().expect("parse cache"),
            session
                .content_mapped_parse_cache
                .clone()
                .expect("content mapped parse cache"),
            session
                .extended_config_cache
                .clone()
                .expect("extended config cache"),
        );
        if let Some(old_project) = old_project {
            builder.seed_inferred_project_for_program(old_project, None);
        }
        if !file_changes.is_empty() {
            builder.did_change_files(&file_changes);
        }
        builder.update_or_create_inferred_project(
            root_file_names.clone(),
            Some(compiler_options.clone()),
            project_references.clone(),
            config_file_parsing_diagnostics.clone(),
            self.inferred_project_content_mappers.clone(),
            None,
        );
        if builder
            .inferred_project
            .value()
            .as_ref()
            .is_some_and(|p| p.dirty)
        {
            let entry = builder.inferred_project_entry().clone();
            builder.update_program(&entry, None);
        }
        builder.cleanup_all_configured_projects(None);
        let (new_project_collection, new_config_file_registry) = builder.finalize();
        fs.clean_disk_files_not_seen_by(&new_project_collection);
        let snapshot_fs = fs.finalize();
        let builder_logs = if session.options.logging_enabled {
            Some(crate::project::logging_log_tree::new_log_tree(&format!(
                "Cloning snapshot {}",
                self.id
            )))
        } else {
            None
        };
        let mut new_snapshot = Arc::new(new_snapshot(
            new_snapshot_id,
            Arc::new(snapshot_fs),
            session.options.clone(),
            Box::new(new_config_file_registry),
            Some(compiler_options),
            self.user_preferences(),
            None,
            None,
        ));
        {
            let new_snapshot_ref = Arc::get_mut(&mut new_snapshot).unwrap();
            new_snapshot_ref.parent_id = self.id;
            new_snapshot_ref.project_collection = Some(Box::new(new_project_collection));
            new_snapshot_ref.inferred_project_content_mappers =
                self.inferred_project_content_mappers.clone();
            new_snapshot_ref.inferred_project_content_mapper_extensions =
                self.inferred_project_content_mapper_extensions.clone();
            new_snapshot_ref.builder_logs = builder_logs;
        }
        wire_snapshot_line_map_closure(&mut new_snapshot);
        for project in new_snapshot.projects() {
            if let Some(program) = project.get_program() {
                session.program_counter_ref(&program);
                if project.program_last_update == new_snapshot_id {
                    project.freeze_host(new_snapshot.fs.as_ref().unwrap(), new_snapshot.config_file_registry());
                }
            }
        }
        for file in new_snapshot.extended_source_files() {
            let path = (new_snapshot.fs.as_ref().expect("fs").to_path)(&file);
            session.extended_config_cache_add_owner(&path, new_snapshot.id);
        }
        new_snapshot
    }

    pub fn process_file_changes(
        &self,
        fs: &mut SnapshotFSBuilder,
        file_changes: FileChangeSummary,
        content_mapper_contributions: Option<&ContentMapperContributions>,
    ) -> FileChangeSummary { ::tsox_core::fntrace::enter("process_file_changes"); 
        let mut file_changes = file_changes;
        if file_changes.has_excessive_watch_events() {
            if file_changes.invalidate_all {
                fs.invalidate_cache();
            } else if !fs.watch_changes_overlap_cache(&file_changes) {
                file_changes.changed.clear();
                file_changes.deleted.clear();
            } else if file_changes.includes_watch_change_outside_node_modules {
                fs.invalidate_cache();
            } else {
                fs.invalidate_node_modules_cache();
            }
            return file_changes;
        }
        let content_mapper_extensions = match content_mapper_contributions {
            None => self.content_mapper_watch_state().0.clone(),
            Some(contributions) => {
                let mut extensions = self.config_file_registry().content_mappers().extensions.clone();
                extensions.extend(contributions.extensions.iter().cloned());
                extensions
            }
        };
        let content_mapper_watched_files = self.content_mapper_watch_state().1.clone();
        file_changes = fs.expand_and_filter_watch_events(
            file_changes,
            &content_mapper_extensions,
            &content_mapper_watched_files,
        );
        file_changes = fs.expand_realpath_aliases(file_changes);
        file_changes = fs.mark_dirty_files(file_changes);
        file_changes = fs.convert_open_and_close_to_changes(file_changes);
        file_changes
    }

    pub fn get_projects_containing_file(&self, uri: &lsproto::DocumentUri) -> Vec<&Project> { ::tsox_core::fntrace::enter("get_projects_containing_file"); 
        let file_name = uri.file_name();
        let path = (self.fs.as_ref().expect("fs").to_path)(&file_name);
        self.project_collection
            .as_deref()
            .expect("project collection")
            .get_projects_containing_file(&path)
    }

    pub fn lsp_line_map(&self, file_name: &str) -> Option<lsconv::LspLineMap> { ::tsox_core::fntrace::enter("lsp_line_map"); 
        self.get_file(file_name)
            .map(|f| crate::ls::lsconv_linemap::compute_lsp_line_starts(f.content()))
    }

    pub fn get_ecma_line_info(&self, _file_name: &str) -> Option<sourcemap::EcmaLineInfo> { ::tsox_core::fntrace::enter("get_ecma_line_info"); 
        todo!("ECMALineInfo 未移植：按 Go lineinfo.go 移植至 tsox-frontend sourcemap（跨 crate 交接）")
    }

    pub fn get_preferences(&self, _active_file: &str) -> UserPreferences { ::tsox_core::fntrace::enter("get_preferences"); 
        self.user_preferences()
    }

    pub fn auto_import_registry(&self) -> Option<Arc<autoimport::Registry>> { ::tsox_core::fntrace::enter("auto_import_registry"); 
        self.auto_imports.clone()
    }

    pub fn directory_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        self.fs.as_ref().unwrap().fs.directory_exists(path)
    }

    pub fn get_directories(&self, path: &str) -> Vec<String> { ::tsox_core::fntrace::enter("get_directories"); 
        self.fs.as_ref().unwrap().fs.get_accessible_entries(path).directories
    }

    pub fn read_directory(
        &self,
        current_dir: &str,
        path: &str,
        extensions: &[String],
        excludes: &[String],
        includes: &[String],
        depth: i32,
    ) -> Vec<String> { ::tsox_core::fntrace::enter("read_directory"); 
        let extensions_ref: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
        let excludes_ref: Vec<&str> = excludes.iter().map(|s| s.as_str()).collect();
        let includes_ref: Vec<&str> = includes.iter().map(|s| s.as_str()).collect();
        vfsmatch::read_directory(
            self.fs.as_ref().unwrap().fs.as_ref(),
            current_dir,
            path,
            &extensions_ref,
            &excludes_ref,
            &includes_ref,
            depth,
        )
    }

    pub fn deref(&self, session: &Session) { ::tsox_core::fntrace::enter("deref"); 
        let rc = self.ref_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) - 1;
        if rc < 0 {
            panic!("snapshot {}: ref count below zero, parentId={}", self.id, self.parent_id);
        }
        if rc == 0 {
            self.dispose_with_session(session);
        }
    }

    fn dispose_with_session(&self, session: &Session) { ::tsox_core::fntrace::enter("dispose_with_session"); 
        for project in self.projects() {
            if let Some(program) = project.get_program()
                && session.program_counter_deref(&program)
            {
                if let Some(content_mapper_project) = program.content_mapper_project() {
                    let _ = content_mapper_project.close();
                }
                if let Some(checker_pool) = project.checker_pool() {
                    checker_pool.discard();
                }
                for file in program.source_files() {
                    if !file.is_content_mapper_failure_stub() && !file.is_content_mapper_supplemental() {
                        if !file.content_mapper().is_empty() {
                            session.content_mapped_parse_cache_deref(&parse_cache_key_for_file(file));
                        } else {
                            session.parse_cache_deref(&parse_cache_key_for_file(file));
                        }
                    }
                }
                for file in program.duplicate_source_files() {
                    if !file.is_content_mapper_failure_stub {
                        if !file.content_mapper.is_empty() {
                            session.content_mapped_parse_cache_deref(&parse_cache_key_for_duplicate(file));
                        } else {
                            session.parse_cache_deref(&parse_cache_key_for_duplicate(file));
                        }
                    }
                }
            }
        }
        let to_path = &self.fs.as_ref().expect("fs").to_path;
        for config in self.config_file_registry_configs().values() {
            if let Some(command_line) = &config.command_line {
                for file in command_line.extended_source_files() {
                    session.extended_config_cache_release(&to_path(&file), self.id);
                }
            }
        }
    }
}

impl Project {
    pub fn content_mapper_watched_files(&self) -> Option<&HashSet<Path>> { ::tsox_core::fntrace::enter("content_mapper_watched_files"); 
        self.content_mapper_watched_files.as_ref()
    }

    pub fn freeze_host(&self, _fs: &SnapshotFS, _registry: &ConfigFileRegistry) { ::tsox_core::fntrace::enter("freeze_host"); 
    }
}

impl Session {
    pub fn fs_fs(&self) -> Arc<dyn tsox_tsoptions::vfs::FS> { ::tsox_core::fntrace::enter("fs_fs"); 
        self.fs.as_ref().expect("fs").fs.clone()
    }

    pub fn next_snapshot_id(&self) -> u64 { ::tsox_core::fntrace::enter("next_snapshot_id"); 
        self.snapshot_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn program_counter_ref(&self, program: &Arc<tsox_compile::compiler::Program>) { ::tsox_core::fntrace::enter("program_counter_ref"); 
        self.program_counter
            .as_ref()
            .expect("program counter")
            .r#ref(program);
    }

    pub fn program_counter_deref(&self, program: &Arc<tsox_compile::compiler::Program>) -> bool { ::tsox_core::fntrace::enter("program_counter_deref"); 
        self.program_counter
            .as_ref()
            .expect("program counter")
            .deref(program)
    }

    pub fn extended_config_cache_add_owner(&self, path: &Path, owner: u64) { ::tsox_core::fntrace::enter("extended_config_cache_add_owner"); 
        self.extended_config_cache
            .as_ref()
            .expect("extended config cache")
            .add_owner(path, owner);
    }

    pub fn extended_config_cache_release(&self, path: &Path, owner: u64) { ::tsox_core::fntrace::enter("extended_config_cache_release"); 
        self.extended_config_cache
            .as_ref()
            .expect("extended config cache")
            .release(path, owner);
    }

    pub fn parse_cache_deref(&self, key: &ParseCacheKey) { ::tsox_core::fntrace::enter("parse_cache_deref"); 
        self.parse_cache
            .as_ref()
            .expect("parse cache")
            .deref(key);
    }

    pub fn content_mapped_parse_cache_deref(&self, key: &ParseCacheKey) { ::tsox_core::fntrace::enter("content_mapped_parse_cache_deref"); 
        self.content_mapped_parse_cache
            .as_ref()
            .expect("content mapped parse cache")
            .deref(key);
    }
}

impl Snapshot {
    pub fn user_preferences(&self) -> UserPreferences { ::tsox_core::fntrace::enter("user_preferences"); 
        self.user_preferences.clone()
    }

    pub fn config_file_registry(&self) -> &ConfigFileRegistry { ::tsox_core::fntrace::enter("config_file_registry"); 
        self.config_file_registry
            .as_deref()
            .expect("config file registry")
    }

    pub fn project_collection(&self) -> &crate::project::project_collection::ProjectCollection { ::tsox_core::fntrace::enter("project_collection"); 
        self.project_collection
            .as_deref()
            .expect("project collection")
    }

    pub fn to_path_fn(&self) -> Arc<dyn Fn(&str) -> Path + Send + Sync> { ::tsox_core::fntrace::enter("to_path_fn"); 
        let fs = Arc::clone(self.fs.as_ref().expect("fs"));
        Arc::new(move |file_name: &str| (fs.to_path)(file_name))
    }

    pub fn get_file(
        &self,
        file_name: &str,
    ) -> Option<std::sync::Arc<dyn crate::project::overlay_fs::FileHandle>> { ::tsox_core::fntrace::enter("get_file"); 
        self.fs.as_ref().expect("fs").get_file(file_name)
    }

    pub fn fs_disk_directories(&self) -> &HashMap<Path, HashMap<Path, String>> { ::tsox_core::fntrace::enter("fs_disk_directories"); 
        &self.fs.as_ref().expect("fs").disk_directories
    }

    pub fn fs_node_modules_realpath_aliases(&self) -> &HashMap<Path, Arc<RealpathAliasSet>> { ::tsox_core::fntrace::enter("fs_node_modules_realpath_aliases"); 
        &self.fs.as_ref().expect("fs").node_modules_realpath_aliases
    }

    pub fn extended_source_files(&self) -> Vec<String> { ::tsox_core::fntrace::enter("extended_source_files"); 
        let mut files = Vec::new();
        if let Some(registry) = &self.config_file_registry {
            for config in registry.configs.values() {
                if let Some(command_line) = &config.command_line {
                    files.extend(command_line.extended_source_files().iter().cloned());
                }
            }
        }
        files
    }
}

impl super::m5d_5::ProjectCollectionBuilderMig<'_> {
    pub fn did_update_ata_state(
        &mut self,
        ata_changes: &HashMap<Path, crate::project::snapshot::ATAStateChange>,
    ) { ::tsox_core::fntrace::enter("did_update_ata_state"); 
        for (project_path, ata_change) in ata_changes {
            let change = ata_change.clone();
            self.configured_projects.change(project_path, |p| {
                p.typings_files = change.typings_files.clone();
                p.dirty = true;
                p.dirty_file_path = Path::default();
            });
        }
    }

    pub fn did_request_project(&mut self, project_id: &Path) { ::tsox_core::fntrace::enter("did_request_project"); 
        if let Some(entry) = self.configured_projects.load(project_id) {
            self.update_program(&entry, None);
        }
    }

    pub fn did_request_file(
        &mut self,
        uri: &lsproto::DocumentUri,
        configured_projects_only: bool,
    ) { ::tsox_core::fntrace::enter("did_request_file"); 
        let file_name = uri.file_name();
        let path = (self.to_path)(&file_name);
        if self.default_projects_invalidated {
            self.ensure_configured_project_and_ancestors_for_file(&file_name, &path, None);
            if !self.fs.is_open_file(&path) {
                return;
            }
        }
        if self.fs.is_open_file(&path) {
            let mut has_changes = self.program_structure_changed;
            if let Some(result) = self.find_default_project(&file_name, &path) {
                has_changes = self.update_program(&result, None) || has_changes;
                let contains = result
                    .value()
                    .as_ref()
                    .contains_file_in_program(&path);
                if contains {
                    if has_changes {
                        self.cleanup_inferred_project(None);
                        if self.inferred_project.value().is_some() {
                            let entry = self.inferred_project_entry().clone();
                            self.update_program(&entry, None);
                        }
                    }
                    return;
                }
            }
            let mut entries: Vec<
                crate::project::dirty_map_::MapEntry<Path, Box<Project>>,
            > = Vec::new();
            self.for_each_project(|entry| {
                entries.push(entry.clone());
                true
            });
            for entry in &entries {
                has_changes = self.update_program(entry, None) || has_changes;
            }
            if has_changes {
                self.cleanup_inferred_project(None);
            }
            if self.inferred_project.value().is_some() {
                let entry = self.inferred_project_entry().clone();
                self.update_program(&entry, None);
            }
        } else {
            let result =
                self.ensure_configured_project_and_ancestors_for_file(&file_name, &path, None);
            if result.project.is_none() && !configured_projects_only {
                self.ensure_inferred_project_includes_closed_file(&file_name, None);
            }
        }
    }

    pub fn did_change_custom_config_file_name(&mut self) { ::tsox_core::fntrace::enter("did_change_custom_config_file_name"); 
        if !self
            .config_file_registry_builder
            .did_change_custom_config_file_name()
        {
            return;
        }
        self.file_default_projects = HashMap::new();
        self.default_projects_invalidated = true;
        self.program_structure_changed = true;
    }

    pub fn did_change_files(&mut self, summary: &FileChangeSummary) { ::tsox_core::fntrace::enter("did_change_files"); 
        self.open_files_changed = self.open_files_changed
            || !summary.opened.0.is_empty()
            || !summary.closed.is_empty();

        let to_paths = |to_path: &Arc<dyn Fn(&str) -> Path + Send + Sync>,
                        uris: &HashSet<lsproto::DocumentUri>|
         -> Vec<Path> {
            uris.iter().map(|uri| to_path(&uri.file_name())).collect()
        };
        let changed_files = to_paths(&self.to_path, &summary.changed);
        let deleted_files = to_paths(&self.to_path, &summary.deleted);
        let created_files = to_paths(&self.to_path, &summary.created);

        let raw_config_change_result = self.config_file_registry_builder.did_change_files(summary);
        let config_change_result = super::m5d_5::ChangeFileResult {
            affected_projects: raw_config_change_result.affected_projects,
            affected_files: raw_config_change_result.affected_files,
        };
        self.program_structure_changed =
            self.mark_projects_affected_by_config_changes(&config_change_result, None);

        let mut entries: Vec<
            crate::project::dirty_map_::MapEntry<Path, Box<Project>>,
        > = Vec::new();
        self.for_each_project(|entry| {
            entries.push(entry.clone());
            true
        });
        for entry in &entries {
            if summary.has_excessive_non_create_watch_events() {
                self.change_entry(entry, |p| {
                    p.dirty = true;
                    p.dirty_file_path = Path::default();
                });
                continue;
            }

            self.mark_files_changed(entry, &changed_files, lsproto::FILE_CHANGE_TYPE_CHANGED, None);

            let project = entry.value();
            if project.kind == ProjectKind::Inferred && !summary.closed.is_empty() {
                if let Some(command_line) = &project.command_line {
                    let mut new_root_files = command_line.file_names().to_vec();
                    for uri in &summary.closed {
                        let file_name = uri.file_name();
                        let path = (self.to_path)(&file_name);
                        if command_line
                            .file_names()
                            .iter()
                            .any(|f| (self.to_path)(f) == path)
                        {
                            new_root_files.retain(|f| *f != file_name);
                        }
                    }
                    self.update_inferred_project_roots(new_root_files, None);
                }
            }

            if !summary.deleted.is_empty() {
                self.mark_files_changed(entry, &deleted_files, lsproto::FILE_CHANGE_TYPE_DELETED, None);
            }

            if !summary.created.is_empty() {
                self.mark_files_changed(entry, &created_files, lsproto::FILE_CHANGE_TYPE_CREATED, None);
            }
        }

        if !summary.opened.0.is_empty() || !summary.reopened.0.is_empty() {
            let file_name = if !summary.opened.0.is_empty() {
                summary.opened.file_name()
            } else {
                summary.reopened.file_name()
            };
            let path = (self.to_path)(&file_name);
            let open_file_result =
                self.ensure_configured_project_and_ancestors_for_file(&file_name, &path, None);
            self.cleanup_configured_projects(&open_file_result.retain, None);
        }
    }

    pub fn finalize(
        &mut self,
    ) -> (
        crate::project::project_collection::ProjectCollection,
        ConfigFileRegistry,
    ) { ::tsox_core::fntrace::enter("finalize"); 
        let (configured_projects, configured_projects_changed) =
            self.configured_projects.finalize();
        let inferred_project_changed = self.inferred_project.dirty();
        let inferred_project = self.inferred_project.value().clone();
        let config_file_registry = self.config_file_registry_builder.finalize();

        let mut open_files: HashSet<Path> = HashSet::new();
        for path in self.fs.overlays.keys() {
            open_files.insert(path.clone());
        }

        let to_path = Arc::clone(&self.to_path);
        let to_path_box: Box<dyn Fn(&str) -> Path + Send + Sync> =
            Box::new(move |file_name: &str| (to_path)(file_name));

        let collection = crate::project::project_collection::ProjectCollection {
            to_path: to_path_box,
            config_file_registry: Some(config_file_registry.clone_shallow()),
            file_default_projects: self.file_default_projects.clone(),
            configured_projects: if configured_projects_changed {
                configured_projects
            } else {
                self.base.configured_projects.clone()
            },
            open_files,
            inferred_project: if inferred_project_changed {
                inferred_project
            } else {
                self.base.inferred_project.clone()
            },
            api_state: self.api_state.clone(),
        };
        (collection, config_file_registry)
    }
}

fn uri_set(
    uris: impl Iterator<Item = lsproto::DocumentUri>,
) -> tsox_core::collections::set::Set<lsproto::DocumentUri> { ::tsox_core::fntrace::enter("uri_set"); 
    let mut set = tsox_core::collections::set::Set::new();
    for uri in uris {
        set.insert(uri);
    }
    set
}
