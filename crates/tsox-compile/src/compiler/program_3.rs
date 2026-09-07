#![allow(unused_imports)]

use super::*;

impl Program {
    pub fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.source_files
    }

    pub fn get_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>> {
        self.source_files_by_name.get(file_name).cloned()
    }

    pub fn diagnostics(&self) -> &[Arc<Diagnostic>] {
        &self.diagnostics
    }

    pub fn get_diagnostics_to_report(&self) -> Vec<Arc<Diagnostic>> {
        let skip_lib = self.options.skip_lib_check.is_true();
        let skip_default_lib = self.options.skip_default_lib_check.is_true();

        if !skip_lib && !skip_default_lib {
            return self.diagnostics.clone();
        }

        self.diagnostics
            .iter()
            .filter(|d| {
                let Some(file) = &d.file else {
                    return true;
                };
                if skip_lib {
                    !file.is_declaration_file && !is_external_library_file(&file.file_name)
                } else {
                    !self.default_library_file_names.contains(&file.file_name)
                }
            })
            .cloned()
            .collect()
    }

    pub fn get_semantic_diagnostics(self: &Arc<Self>) -> Vec<Diagnostic> {
        let skip_lib = self.options.skip_lib_check.is_true();
        let skip_default_lib = self.options.skip_default_lib_check.is_true();

        let checker = self.build_checker_internal(skip_lib, skip_default_lib);
        let check_diagnostics = checker.get_semantic_diagnostics();

        let mut diagnostics: Vec<Diagnostic> = if skip_lib {
            self.symbol_map
                .binder_diagnostics
                .iter()
                .filter(|d| {
                    d.file
                        .as_ref()
                        .map(|f| !f.is_declaration_file && !is_external_library_file(&f.file_name))
                        .unwrap_or(true)
                })
                .cloned()
                .collect()
        } else if skip_default_lib {
            self.symbol_map
                .binder_diagnostics
                .iter()
                .filter(|d| {
                    d.file
                        .as_ref()
                        .map(|f| !self.default_library_file_names.contains(&f.file_name))
                        .unwrap_or(true)
                })
                .cloned()
                .collect()
        } else {
            self.symbol_map.binder_diagnostics.iter().cloned().collect()
        };
        diagnostics.extend(check_diagnostics);

        diagnostics.retain(|d| self.includes_semantic_diagnostic(d));
        diagnostics
    }

    pub(crate) fn can_include_bind_and_check_diagnostics(&self, file: &SourceFile) -> bool {
        match file.script_kind {
            ScriptKind::Ts | ScriptKind::Tsx | ScriptKind::External | ScriptKind::Deferred => true,
            ScriptKind::Js | ScriptKind::Jsx => !self.options.check_js.is_false(),
            ScriptKind::Json | ScriptKind::Unknown => false,
        }
    }

    pub(crate) fn includes_semantic_diagnostic(&self, d: &Diagnostic) -> bool {
        let Some(file) = &d.file else {
            return true;
        };
        if !self.can_include_bind_and_check_diagnostics(file) {
            return false;
        }
        if is_plain_js_file(file, self.options.check_js) && !PLAIN_JS_ERROR_CODES.contains(&d.code)
        {
            return false;
        }
        true
    }

    pub fn build_checker(self: &Arc<Self>) -> tsox_checker::checker::Checker {
        self.build_checker_internal(false, false)
    }

    pub(crate) fn build_checker_internal(
        self: &Arc<Self>,
        skip_lib: bool,
        skip_default_lib: bool,
    ) -> tsox_checker::checker::Checker {
        let tracer = Arc::new(tsox_checker::checker::Tracer::new());
        let program: Arc<dyn tsox_checker::checker::Program> = Arc::clone(self) as _;
        let mut checker = tsox_checker::checker::Checker::new(program, tracer);
        for file in &self.source_files {
            if skip_lib && (file.is_declaration_file || is_external_library_file(&file.file_name)) {
                continue;
            }

            if skip_default_lib && self.default_library_file_names.contains(&file.file_name) {
                continue;
            }
            checker.check_source_file(file);
        }
        checker
    }

    pub fn config_file_name(&self) -> &str {
        &self.config_file_name
    }

    pub fn symbol_map(&self) -> &NodeSymbolMap {
        &self.symbol_map
    }

    pub fn host(&self) -> &dyn CompilerHost {
        self.host.as_ref()
    }

    pub fn is_source_file_default_library(&self, file_name: &str) -> bool {
        self.default_library_file_names.contains(file_name)
    }

    pub fn file_exists(&self, file_name: &str) -> bool {
        self.host.fs().file_exists(file_name)
    }

    pub fn emit(
        &self,
        write_file: &dyn Fn(&str, &str) -> std::io::Result<()>,
    ) -> tsox_emit::emitter::EmitResult {
        let fs = self.host.fs();

        let source_files: Vec<_> = self
            .source_files
            .iter()
            .filter(|sf| {
                !self.default_library_file_names.contains(&sf.file_name)
                    && !is_external_library_file(&sf.file_name)
            })
            .cloned()
            .collect();
        tsox_emit::emitter::emit_program(&source_files, &self.options, fs, write_file)
    }
}

impl tsox_checker::checker::Program for Program {
    fn options(&self) -> &CompilerOptions {
        &self.options
    }
    fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.source_files
    }
    fn bind_source_files(&self) {}
    fn file_exists(&self, file_name: &str) -> bool {
        Program::file_exists(self, file_name)
    }
    fn get_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>> {
        Program::get_source_file(self, file_name)
    }
    fn is_source_file_default_library(&self, path: &str) -> bool {
        Program::is_source_file_default_library(self, path)
    }
    fn resolve_external_module_path(
        &self,
        specifier: &str,
        containing_file: &str,
        resolution_mode: tsox_core::core::compiler_options::ModuleKind,
    ) -> Option<String> {
        let resolution_host: Arc<dyn tsox_tsoptions::module::ResolutionHost + Send + Sync> =
            Arc::new(ResolutionHostAdapter::new(self.host.as_ref()));
        let resolver = tsox_tsoptions::module::Resolver::new(
            resolution_host,
            Arc::new(self.options.clone()),
            String::new(),
            String::new(),
        );
        let (resolved, _traces) =
            resolver.resolve_module_name(specifier, containing_file, resolution_mode, None);
        resolved
            .filter(|m| m.is_resolved())
            .map(|m| m.resolved_file_name)
    }
    fn symbol_map(&self) -> &NodeSymbolMap {
        Program::symbol_map(self)
    }
    fn current_directory(&self) -> &str {
        self.host.current_directory()
    }
    fn use_case_sensitive_file_names(&self) -> bool {
        self.host.use_case_sensitive_file_names()
    }
    fn common_source_directory(&self) -> String {
        let source_files: Vec<_> = self
            .source_files
            .iter()
            .filter(|sf| !self.default_library_file_names.contains(&sf.file_name))
            .cloned()
            .collect();
        tsox_emit::emitter::compute_program_common_source_directory(&source_files, &self.options)
    }
    fn read_file(&self, file_name: &str) -> Option<String> {
        self.host.fs().read_file(file_name)
    }
    fn get_emit_module_format_of_file(
        &self,
        file_name: &str,
    ) -> tsox_core::core::compiler_options::ModuleKind {
        use tsox_core::core::compiler_options::ModuleKind;
        match self.options.module {
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext => {
                if tsox_tsoptions::tsoptions::implied_node_format_of_file(file_name, &|p| {
                    self.host.fs().read_file(p)
                }) == ModuleKind::ESNext
                {
                    ModuleKind::ES2020
                } else {
                    ModuleKind::CommonJS
                }
            }
            ModuleKind::None => {
                if self.options.get_emit_script_target()
                    >= tsox_core::core::compiler_options::ScriptTarget::ES2015
                {
                    ModuleKind::ES2015
                } else {
                    ModuleKind::CommonJS
                }
            }
            other => other,
        }
    }
}
