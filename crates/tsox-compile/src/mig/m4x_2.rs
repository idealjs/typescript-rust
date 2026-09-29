#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::compiler::ProgramOptions;
use super::m4x::new_project_reference_dts_faking_host;

pub struct FileLoader {
    pub opts: ProgramOptions,
    pub project_reference_file_mapper: Arc<ProjectReferenceFileMapper>,
    pub dts_directories: Set<Path>,
}

impl FileLoader {
    pub fn to_path(&self, path: &str) -> Path {
        tspath::to_path(
            path,
            self.opts.host.current_directory(),
            self.opts.host.use_case_sensitive_file_names(),
        )
    }
}

pub struct ProjectReferenceFileMapper {
    pub opts: ProgramOptions,
    pub host: Option<Box<dyn tsox_tsoptions::module::ResolutionHost + Send + Sync>>,
    pub loader: Option<Arc<Mutex<FileLoader>>>,

    pub config_to_project_reference: HashMap<Path, Option<Arc<ParsedCommandLine>>>,
    pub references_in_config_file: HashMap<Path, Vec<Path>>,
    pub source_to_project_reference: HashMap<Path, Option<SourceOutputAndProjectReference>>,
    pub output_dts_to_project_reference: HashMap<Path, Option<SourceOutputAndProjectReference>>,

    pub realpath_dts_to_source: SyncMap<Path, Option<SourceOutputAndProjectReference>>,
}

impl ProjectReferenceFileMapper {
    pub fn new(opts: &ProgramOptions) -> Self {
        Self {
            opts: opts.clone(),
            host: None,
            loader: None,
            config_to_project_reference: HashMap::new(),
            references_in_config_file: HashMap::new(),
            source_to_project_reference: HashMap::new(),
            output_dts_to_project_reference: HashMap::new(),
            realpath_dts_to_source: SyncMap::new(),
        }
    }

    pub fn host_compiler_host(&self) -> Arc<dyn crate::compiler::CompilerHost> {
        self.opts.host.clone()
    }

    pub fn can_use_project_reference_source(&self) -> bool {
        self.opts.can_use_project_reference_source()
    }

    pub fn root_config_path(&self) -> Path {
        match self.opts.config.config_file.as_ref() {
            None => Path(String::new()),
            Some(config_file) => Path(tsox_frontend::ast::mig::m3b_2::path(
                &config_file.source_file,
            )),
        }
    }

    pub fn get_parse_file_redirect(&self, file: &dyn HasFileName) -> String {
        if self.can_use_project_reference_source() {
            let mut source = self.get_project_reference_from_output_dts(&Path(file.path()));
            if source.is_none() {
                source = self.get_source_to_dts_if_symlink(file);
            }
            if let Some(source) = source {
                return source.source;
            }
        } else {
            let output = self.get_project_reference_from_source(&Path(file.path()));
            if let Some(output) = output {
                if !output.output_dts.is_empty() {
                    return output.output_dts.clone();
                }
            }
        }
        String::new()
    }

    pub fn get_resolved_project_references(&self) -> Vec<Option<Arc<ParsedCommandLine>>> {
        let mut result = Vec::new();
        if let Some(refs) = self.references_in_config_file.get(&self.root_config_path()) {
            result.reserve(refs.len());
            for ref_path in refs {
                let ref_config = self
                    .config_to_project_reference
                    .get(ref_path)
                    .cloned()
                    .flatten();
                result.push(ref_config);
            }
        }
        result
    }

    pub fn get_project_reference_from_source(
        &self,
        path: &Path,
    ) -> Option<SourceOutputAndProjectReference> {
        self.source_to_project_reference
            .get(path)
            .cloned()
            .flatten()
    }

    pub fn get_project_reference_from_output_dts(
        &self,
        path: &Path,
    ) -> Option<SourceOutputAndProjectReference> {
        self.output_dts_to_project_reference
            .get(path)
            .cloned()
            .flatten()
    }

    pub fn is_source_from_project_reference(&self, path: &Path) -> bool {
        self.can_use_project_reference_source()
            && self.get_project_reference_from_source(path).is_some()
    }

    pub fn get_compiler_options_for_file(
        &self,
        file: &dyn HasFileName,
    ) -> tsox_core::core::compiler_options::CompilerOptions {
        let redirect = self.get_redirect_parsed_command_line_for_resolution(file);
        get_compiler_options_with_redirect(
            &self.opts.config.compiler_options(),
            redirect.as_deref(),
        )
        .clone()
    }

    pub fn get_redirect_parsed_command_line_for_resolution(
        &self,
        file: &dyn HasFileName,
    ) -> Option<Arc<ParsedCommandLine>> {
        self.get_redirect_for_resolution(file).0
    }

    pub fn get_redirect_for_resolution(
        &self,
        file: &dyn HasFileName,
    ) -> (Option<Arc<ParsedCommandLine>>, String) {
        let path = Path(file.path());
        let output = self.get_project_reference_from_source(&path);
        if let Some(output) = output {
            return (output.resolved, output.source.clone());
        }

        let result_from_dts = self.get_project_reference_from_output_dts(&path);
        if let Some(result_from_dts) = result_from_dts {
            return (result_from_dts.resolved, result_from_dts.source.clone());
        }

        if let Some(realpath_dts_to_source) = self.get_source_to_dts_if_symlink(file) {
            return (
                realpath_dts_to_source.resolved,
                realpath_dts_to_source.source.clone(),
            );
        }
        (None, file.file_name())
    }

    pub fn get_resolved_reference_for(
        &self,
        path: &Path,
    ) -> (Option<Arc<ParsedCommandLine>>, bool) {
        match self.config_to_project_reference.get(path) {
            Some(config) => (config.clone(), true),
            None => (None, false),
        }
    }

    pub fn range_resolved_project_reference(
        &self,
        f: &mut dyn FnMut(&Path, Option<&Arc<ParsedCommandLine>>, Option<&Arc<ParsedCommandLine>>, usize) -> bool,
    ) -> bool {
        if self.opts.config.project_references().is_empty() {
            return false;
        }
        let mut seen_ref = Set::with_capacity(self.references_in_config_file.len());
        let root_config_path = self.root_config_path();
        seen_ref.add(root_config_path.clone());
        let refs = self
            .references_in_config_file
            .get(&root_config_path)
            .cloned()
            .unwrap_or_default();
        self.range_resolved_reference_worker(&refs, f, None, &mut seen_ref)
    }

    pub fn range_resolved_reference_worker(
        &self,
        references: &[Path],
        f: &mut dyn FnMut(&Path, Option<&Arc<ParsedCommandLine>>, Option<&Arc<ParsedCommandLine>>, usize) -> bool,
        parent: Option<&Arc<ParsedCommandLine>>,
        seen_ref: &mut Set<Path>,
    ) -> bool {
        for (index, path) in references.iter().enumerate() {
            if !seen_ref.add_if_absent(path.clone()) {
                continue;
            }
            let config = self
                .config_to_project_reference
                .get(path)
                .cloned()
                .flatten();
            if !f(path, config.as_ref(), parent, index) {
                return false;
            }
            let child_refs = self
                .references_in_config_file
                .get(path)
                .cloned()
                .unwrap_or_default();
            if !self.range_resolved_reference_worker(&child_refs, f, config.as_ref(), seen_ref) {
                return false;
            }
        }
        true
    }

    pub fn range_resolved_project_reference_in_child_config(
        &self,
        child_config: Option<&Arc<ParsedCommandLine>>,
        f: &mut dyn FnMut(&Path, Option<&Arc<ParsedCommandLine>>, Option<&Arc<ParsedCommandLine>>, usize) -> bool,
    ) -> bool {
        let child_config = match child_config {
            None => return false,
            Some(child_config) => child_config,
        };
        let child_config_path = match child_config.config_file.as_ref() {
            None => return false,
            Some(config_file) => Path(tsox_frontend::ast::mig::m3b_2::path(
                &config_file.source_file,
            )),
        };
        let mut seen_ref = Set::with_capacity(self.references_in_config_file.len());
        seen_ref.add(child_config_path.clone());
        let refs = self
            .references_in_config_file
            .get(&child_config_path)
            .cloned()
            .unwrap_or_default();
        self.range_resolved_reference_worker(&refs, f, Some(&self.config()), &mut seen_ref)
    }

    fn config(&self) -> Arc<ParsedCommandLine> {
        Arc::new(self.opts.config.clone())
    }

    pub fn get_source_to_dts_if_symlink(
        &self,
        file: &dyn HasFileName,
    ) -> Option<SourceOutputAndProjectReference> {
        let path = Path(file.path());
        if let Some(realpath_dts_to_source) = self.realpath_dts_to_source.load(&path) {
            return realpath_dts_to_source;
        }
        if self.loader.is_some()
            && self.opts.config.compiler_options().preserve_symlinks == Tristate::True
        {
            let file_name = file.file_name();
            if !file_name.contains("/node_modules/") {
                self.realpath_dts_to_source.store(path, None);
            } else {
                let real_declaration_path = self.loader_to_path(
                    self.host_fs().realpath(&file_name).as_str(),
                );
                if real_declaration_path == path {
                    self.realpath_dts_to_source.store(path, None);
                } else {
                    let realpath_dts_to_source =
                        self.get_project_reference_from_output_dts(&real_declaration_path);
                    if let Some(realpath_dts_to_source) = realpath_dts_to_source {
                        self.realpath_dts_to_source
                            .store(path, Some(realpath_dts_to_source.clone()));
                        return Some(realpath_dts_to_source);
                    }
                    self.realpath_dts_to_source.store(path, None);
                }
            }
        }
        None
    }

    fn host_fs(&self) -> Arc<dyn tsox_tsoptions::vfs::FS> {
        self.opts.host.fs_arc()
    }

    fn loader_to_path(&self, path: &str) -> Path {
        tspath::to_path(path, self.opts.host.current_directory(), true)
    }
}

pub trait HasFileName {
    fn path(&self) -> String;
    fn file_name(&self) -> String;
}

impl HasFileName for tsox_frontend::ast::SourceFile {
    fn path(&self) -> String {
        tsox_frontend::ast::mig::m3b_2::path(self)
    }

    fn file_name(&self) -> String {
        self.file_name.clone()
    }
}

pub fn get_compiler_options_with_redirect<'a>(
    compiler_options: &'a tsox_core::core::compiler_options::CompilerOptions,
    redirect: Option<&'a ParsedCommandLine>,
) -> &'a tsox_core::core::compiler_options::CompilerOptions {
    match redirect {
        None => compiler_options,
        Some(parsed) => parsed.compiler_options(),
    }
}
