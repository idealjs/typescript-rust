#![allow(unused_imports)]

use super::*;

pub trait CompilerHost: Send + Sync {
    fn fs(&self) -> &dyn FS;

    fn fs_arc(&self) -> Arc<dyn FS>;
    fn current_directory(&self) -> &str;
    fn default_library_path(&self) -> &str;
    fn use_case_sensitive_file_names(&self) -> bool {
        self.fs().use_case_sensitive_file_names()
    }
}

pub struct CompilerHostImpl {
    pub(crate) fs: Arc<dyn FS>,
    pub(crate) current_directory: String,
    pub(crate) default_library_path: String,
}

impl CompilerHostImpl {
    pub fn new(fs: Arc<dyn FS>, current_directory: String, default_library_path: String) -> Self {
        Self {
            fs,
            current_directory,
            default_library_path,
        }
    }
}

impl CompilerHost for CompilerHostImpl {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn fs_arc(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs)
    }
    fn current_directory(&self) -> &str {
        &self.current_directory
    }
    fn default_library_path(&self) -> &str {
        &self.default_library_path
    }
}

pub(crate) struct ResolutionHostAdapter {
    pub(crate) fs: Arc<dyn FS>,
    pub(crate) current_directory: String,
}

impl ResolutionHostAdapter {
    pub(crate) fn new(host: &dyn CompilerHost) -> Self {
        Self {
            fs: host.fs_arc(),
            current_directory: host.current_directory().to_string(),
        }
    }
}

impl module::ResolutionHost for ResolutionHostAdapter {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
}

pub struct ProgramOptions {
    pub config: ParsedCommandLine,
    pub host: Arc<dyn CompilerHost>,
}

pub struct Program {
    pub(crate) options: CompilerOptions,
    pub(crate) source_files: Vec<Arc<SourceFile>>,
    pub(crate) source_files_by_name: HashMap<String, Arc<SourceFile>>,
    pub(crate) default_library_file_names: std::collections::HashSet<String>,
    pub(crate) diagnostics: Vec<Arc<Diagnostic>>,
    pub(crate) host: Arc<dyn CompilerHost>,
    pub(crate) config_file_name: String,

    pub(crate) symbol_map: NodeSymbolMap,
}
