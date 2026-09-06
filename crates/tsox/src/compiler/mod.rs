use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ast::NodeSymbolMap;
use crate::ast::ScriptKind;
use crate::ast::SourceFile;
use crate::ast::diagnostic::Diagnostic;
use crate::ast::{self};
use crate::binder::Binder;
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::diagnostics::Category;
use crate::module;
use crate::parser::{Parser, script_kind_from_file_name};
use crate::tspath;
use crate::vfs::FS;

use crate::tsoptions::ParsedCommandLine;

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
    fs: Arc<dyn FS>,
    current_directory: String,
    default_library_path: String,
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

struct ResolutionHostAdapter {
    fs: Arc<dyn FS>,
    current_directory: String,
}

impl ResolutionHostAdapter {
    fn new(host: &dyn CompilerHost) -> Self {
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
    options: CompilerOptions,
    source_files: Vec<Arc<SourceFile>>,
    source_files_by_name: HashMap<String, Arc<SourceFile>>,
    default_library_file_names: std::collections::HashSet<String>,
    diagnostics: Vec<Arc<Diagnostic>>,
    host: Arc<dyn CompilerHost>,
    config_file_name: String,

    symbol_map: NodeSymbolMap,
}

impl Program {

    pub fn new(opts: ProgramOptions) -> Self {
        let host = opts.host;
        let mut options = opts.config.compiler_options.clone();
        let config_file_name = opts.config.config_file_name.clone();

        if !config_file_name.is_empty() && options.config_file_path.is_empty() {
            options.config_file_path = config_file_name.clone();
        }

        let mut source_files: Vec<Arc<SourceFile>> = Vec::new();
        let mut by_name: HashMap<String, Arc<SourceFile>> = HashMap::new();
        let mut default_lib_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut diagnostics: Vec<Arc<Diagnostic>> = Vec::new();

        if options.module_resolution == ModuleResolutionKind::Node10 {
            let mut deprecation = Diagnostic::new(
                None,
                TextRange::default(),
                crate::diagnostics::messages_generated::
                    OPTION_0_1_IS_DEPRECATED_AND_WILL_STOP_FUNCTIONING_IN_TYPESCRIPT_2_SPECIFY_COMPILEROPTION_IGNOREDEPRECATIONS_COLON_3_TO_SILENCE_THIS_ERROR,
                vec![
                    "moduleResolution".to_string(),
                    "node10".to_string(),
                    "7.0".to_string(),
                    "6.0".to_string(),
                ],
            );
            deprecation.message_chain = vec![Diagnostic::new(
                None,
                TextRange::default(),
                crate::diagnostics::messages_generated::
                    VISIT_HTTPS_COLON_SLASH_SLASHAKA_MS_SLASHTS6_FOR_MIGRATION_INFORMATION,
                Vec::new(),
            )];
            diagnostics.push(Arc::new(deprecation));
        }

        if !options.lib.is_empty() && options.no_lib.is_true() {
            diagnostics.push(Arc::new(Diagnostic::new(
                None,
                TextRange::default(),
                crate::diagnostics::messages_generated::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                vec!["lib".to_string(), "noLib".to_string()],
            )));
        }

        if !opts.config.file_names.is_empty() && !options.no_lib.is_true() {
            let lib_names = default_lib_file_names(&options);
            let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
            for lib_name in &lib_names {
                load_lib_recursive(
                    lib_name,
                    host.as_ref(),
                    &mut source_files,
                    &mut by_name,
                    &mut default_lib_names,
                    &mut visited,
                    &mut diagnostics,
                );
            }
        }

        let allow_js = options.get_allow_js();
        for file_name in &opts.config.file_names {
            load_source_file_with_references(
                file_name,
                host.as_ref(),
                &mut source_files,
                &mut by_name,
                &mut diagnostics,
                allow_js,
            );
        }

        {
            let resolution_host: Arc<dyn module::ResolutionHost + Send + Sync> =
                Arc::new(ResolutionHostAdapter::new(host.as_ref()));
            let resolver = module::Resolver::new(
                resolution_host,
                Arc::new(options.clone()),
                String::new(),
                String::new(),
            );

            let mut visited: std::collections::HashSet<String> = by_name.keys().cloned().collect();
            let mut stack: Vec<Arc<SourceFile>> = Vec::new();

            let expanded_types: Vec<String> = if options.types.iter().any(|t| t == "*") {
                let (type_roots, _from_config) =
                    module::resolver::get_effective_type_roots(&options, host.current_directory());
                let mut names: Vec<String> = Vec::new();
                for root in &type_roots {
                    for entry in host.fs().get_accessible_entries(root).directories {
                        names.push(entry);
                    }
                }
                names
            } else {
                options.types.clone()
            };
            for type_name in &expanded_types {
                let (resolved, _traces) = resolver.resolve_type_reference_directive(
                    type_name,
                    &config_file_name,
                    crate::core::compiler_options::ModuleKind::None,
                    None,
                );
                if let Some(resolved_tr) = resolved {
                    if resolved_tr.is_resolved() {
                        let resolved_path = resolved_tr.resolved_file_name.as_str();
                        if visited.insert(resolved_path.to_string()) {
                            let pre = source_files.len();
                            load_source_file_with_references(
                                resolved_path,
                                host.as_ref(),
                                &mut source_files,
                                &mut by_name,
                                &mut diagnostics,
                                allow_js,
                            );
                            stack.extend(source_files[pre..].iter().cloned());
                        }
                    }
                }
            }

            stack.extend(source_files.iter().cloned());
            while let Some(file) = stack.pop() {

                let type_refs = extract_reference_types_directives(&file.text);
                for type_ref in &type_refs {

                    let mut mode = crate::core::compiler_options::ModuleKind::None;
                    let mut bad_mode_value = false;
                    match type_ref.mode_value.as_deref() {
                        Some("import") => mode = ModuleKind::ESNext,
                        Some("require") => mode = ModuleKind::CommonJS,
                        Some(_) => bad_mode_value = true,
                        None => {}
                    }
                    if bad_mode_value {
                        diagnostics.push(Arc::new(crate::ast::Diagnostic::new(
                            Some(Arc::clone(&file)),

                            TextRange::new(
                                type_ref.types_value_range.0,
                                type_ref.types_value_range.1,
                            ),
                            crate::diagnostics::messages_generated::
                                X_RESOLUTION_MODE_SHOULD_BE_EITHER_REQUIRE_OR_IMPORT,
                            Vec::new(),
                        )));
                    }
                    let (resolved, _traces) = resolver.resolve_type_reference_directive(
                        &type_ref.name,
                        &file.file_name,
                        mode,
                        None,
                    );
                    if let Some(resolved_tr) = resolved {
                        if resolved_tr.is_resolved() {
                            let resolved_path = resolved_tr.resolved_file_name.as_str();
                            if visited.insert(resolved_path.to_string()) {
                                let pre = source_files.len();
                                load_source_file_with_references(
                                    resolved_path,
                                    host.as_ref(),
                                    &mut source_files,
                                    &mut by_name,
                                    &mut diagnostics,
                                    allow_js,
                                );
                                stack.extend(source_files[pre..].iter().cloned());
                            }
                        }
                    }
                }

                for import_node in &file.imports {
                    let module_spec = import_node.text();
                    if module_spec.is_empty() {
                        continue;
                    }                    let (resolved, _traces) = resolver.resolve_module_name(
                        module_spec,
                        &file.file_name,
                        import_resolution_mode_override(import_node),
                        None,
                    );
                    let is_resolved = resolved.as_ref().map(|m| m.is_resolved()).unwrap_or(false);
                    if is_resolved {
                        let resolved_module = resolved.unwrap();
                        let resolved_path = resolved_module.resolved_file_name.as_str();
                        if visited.insert(resolved_path.to_string()) {
                            let pre = source_files.len();
                            load_source_file_with_references(
                                resolved_path,
                                host.as_ref(),
                                &mut source_files,
                                &mut by_name,
                                &mut diagnostics,
                                allow_js,
                            );
                            stack.extend(source_files[pre..].iter().cloned());
                        }
                    } else if module_spec.starts_with('.')
                        || !ambient_module_exists(&source_files, module_spec)
                    {

                        let mut module_not_found = Diagnostic::new(
                            Some(file.clone()),
                            import_node.loc,
                            crate::diagnostics::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
                            vec![module_spec.to_string()],
                        );

                        if let Some(alt) = resolved
                            .as_ref()
                            .and_then(|m| m.alternate_result.clone())
                        {
                            module_not_found.message_chain = vec![Diagnostic::new(
                                Some(file.clone()),
                                import_node.loc,
                                crate::diagnostics::messages_generated::
                                    THERE_ARE_TYPES_AT_0_BUT_THIS_RESULT_COULD_NOT_BE_RESOLVED_UNDER_YOUR_CURRENT_MODULERESOLUTION_SETTING_CONSIDER_UPDATING_TO_NODE16_NODENEXT_OR_BUNDLER,
                                vec![alt],
                            )];
                        }
                        diagnostics.push(Arc::new(module_not_found));
                    }
                }

                use crate::core::compiler_options::JsxEmit;
                if matches!(options.jsx, JsxEmit::ReactJSX | JsxEmit::ReactJSXDev)
                    && (file.file_name.ends_with(".tsx")
                        || file.file_name.ends_with(".jsx"))
                {
                    let source = if options.jsx_import_source.is_empty() {
                        "react"
                    } else {
                        options.jsx_import_source.as_str()
                    };
                    let module_ref = if options.jsx == JsxEmit::ReactJSXDev {
                        format!("{source}/jsx-dev-runtime")
                    } else {
                        format!("{source}/jsx-runtime")
                    };
                    let mode = implied_node_format_of_file(&file.file_name, &|p| {
                        host.fs().read_file(p)
                    });
                    let (resolved, _traces) = resolver.resolve_module_name(
                        &module_ref,
                        &file.file_name,
                        mode,
                        None,
                    );
                    if resolved.as_ref().is_some_and(|m| m.is_resolved()) {
                        let resolved_path =
                            resolved.as_ref().unwrap().resolved_file_name.as_str();
                        if visited.insert(resolved_path.to_string()) {
                            load_source_file_with_references(
                                resolved_path,
                                host.as_ref(),
                                &mut source_files,
                                &mut by_name,
                                &mut diagnostics,
                                allow_js,
                            );
                        }
                    }
                }
            }
        }

        for err in &opts.config.errors {
            diagnostics.push(Arc::new(err.clone()));
        }

        let mut binder = Binder::new();
        for file in &source_files {
            binder.bind_source_file(file);
        }
        let symbol_map = std::mem::take(&mut binder.symbol_map);

        Program {
            options,
            source_files,
            source_files_by_name: by_name,
            default_library_file_names: default_lib_names,
            diagnostics,
            host,
            config_file_name,
            symbol_map,
        }
    }

    pub fn options(&self) -> &CompilerOptions {
        &self.options
    }

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
                        .map(|f| {
                            !f.is_declaration_file && !is_external_library_file(&f.file_name)
                        })
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

    fn can_include_bind_and_check_diagnostics(&self, file: &SourceFile) -> bool {
        match file.script_kind {
            ScriptKind::Ts | ScriptKind::Tsx | ScriptKind::External | ScriptKind::Deferred => true,
            ScriptKind::Js | ScriptKind::Jsx => !self.options.check_js.is_false(),
            ScriptKind::Json | ScriptKind::Unknown => false,
        }
    }

    fn includes_semantic_diagnostic(&self, d: &Diagnostic) -> bool {
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

    pub fn build_checker(self: &Arc<Self>) -> crate::checker::Checker {
        self.build_checker_internal(false, false)
    }

    fn build_checker_internal(
        self: &Arc<Self>,
        skip_lib: bool,
        skip_default_lib: bool,
    ) -> crate::checker::Checker {
        let tracer = Arc::new(crate::checker::Tracer::new());
        let program: Arc<dyn crate::checker::Program> = Arc::clone(self) as _;
        let mut checker = crate::checker::Checker::new(program, tracer);
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
    ) -> crate::emitter::EmitResult {
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
        crate::emitter::emit_program(&source_files, &self.options, fs, write_file)
    }
}

impl crate::checker::Program for Program {
    fn options(&self) -> &CompilerOptions {
        &self.options
    }
    fn source_files(&self) -> &[Arc<SourceFile>] {
        &self.source_files
    }
    fn bind_source_files(&self) {

    }
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
        resolution_mode: crate::core::compiler_options::ModuleKind,
    ) -> Option<String> {

        let resolution_host: Arc<dyn module::ResolutionHost + Send + Sync> =
            Arc::new(ResolutionHostAdapter::new(self.host.as_ref()));
        let resolver = module::Resolver::new(
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
        crate::emitter::compute_program_common_source_directory(&source_files, &self.options)
    }
    fn read_file(&self, file_name: &str) -> Option<String> {
        self.host.fs().read_file(file_name)
    }
    fn get_emit_module_format_of_file(
        &self,
        file_name: &str,
    ) -> crate::core::compiler_options::ModuleKind {

        use crate::core::compiler_options::ModuleKind;
        match self.options.module {
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext => {
                if implied_node_format_of_file(file_name, &|p| self.host.fs().read_file(p))
                    == ModuleKind::ESNext
                {
                    ModuleKind::ES2020
                } else {
                    ModuleKind::CommonJS
                }
            }
            ModuleKind::None => {
                if self.options.get_emit_script_target() >= crate::core::compiler_options::ScriptTarget::ES2015
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

pub fn implied_node_format_of_file(
    file_name: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> crate::core::compiler_options::ModuleKind {
    use crate::core::compiler_options::ModuleKind;
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".mts") || lower.ends_with(".mjs") || lower.ends_with(".mjsx") {
        return ModuleKind::ESNext;
    }
    if lower.ends_with(".cts") || lower.ends_with(".cjs") || lower.ends_with(".cjsx") {
        return ModuleKind::CommonJS;
    }

    let mut dir = tspath::get_directory_path(file_name);
    loop {
        let pkg = tspath::combine_paths(&dir, &["package.json"]);
        if let Some(text) = read_file(&pkg)
            && let Ok(fields) = crate::packagejson::parse(&text)
            && let Some(ty) = fields.header_fields.r#type.get_value()
        {
            return if ty == "module" {
                ModuleKind::ESNext
            } else {
                ModuleKind::CommonJS
            };
        }        let parent = tspath::get_directory_path(&dir);
        if parent == dir {
            return ModuleKind::CommonJS;
        }
        dir = parent;
    }
}

fn import_resolution_mode_override(
    import_node: &Arc<ast::Node>,
) -> crate::core::compiler_options::ModuleKind {
    use crate::core::compiler_options::ModuleKind;
    let Some(decl) = import_node.parent.as_ref() else {
        return ModuleKind::None;
    };
    let (attributes, type_only) = match &decl.data {
        ast::NodeData::ImportDeclaration(d) => {
            let type_only = d.import_clause.as_ref().is_some_and(|c| {
                matches!(&c.data, ast::NodeData::ImportClause(ic)
                    if ic.phase_modifier == Some(ast::SyntaxKind::TypeKeyword))
            });
            (d.attributes.as_ref(), type_only)
        }
        ast::NodeData::ExportDeclaration(d) => (d.attributes.as_ref(), d.is_type_only),
        _ => return ModuleKind::None,
    };
    let Some(attrs) = attributes else {
        return ModuleKind::None;
    };
    if !type_only {
        return ModuleKind::None;
    }
    let ast::NodeData::ImportAttributes(data) = &attrs.data else {
        return ModuleKind::None;
    };
    if data.attributes.len() != 1 {
        return ModuleKind::None;
    }
    let ast::NodeData::ImportAttribute(attr) = &data.attributes.nodes[0].data else {
        return ModuleKind::None;
    };
    if attr.name.text() != "resolution-mode" {
        return ModuleKind::None;
    }
    match attr.value.text() {
        "import" => ModuleKind::ESNext,
        "require" => ModuleKind::CommonJS,
        _ => ModuleKind::None,
    }
}

pub fn is_external_library_file(file_name: &str) -> bool {
    file_name.contains("/node_modules/") || file_name.contains("\\node_modules\\")
}

fn is_plain_js_file(file: &SourceFile, check_js: Tristate) -> bool {
    matches!(file.script_kind, ScriptKind::Js | ScriptKind::Jsx) && check_js.is_unknown()
}

const PLAIN_JS_ERROR_CODES: &[i32] = &[

    2451,
    2528,
    2753,
    2752,
    1262,
    1214,
    1359,
    18012,
    1102,
    1210,
    1215,
    1100,
    1344,
    1101,

    1105,
    1116,
    1211,
    1248,
    1171,
    1104,
    1115,
    1113,
    1258,
    1255,
    1182,
    1054,
    2501,
    2566,
    1186,
    2462,
    1048,
    1014,
    1013,
    18041,
    1053,
    1049,
    1474,
    1193,
    1473,
    1191,
    1162,
    1325,
    2803,
    2492,
    1197,
    18036,
    1174,
    18006,
    1312,
    1114,
    1450,
    18038,
    17000,
    17001,
    18007,
    2633,
    1107,
    1200,
    1184,
    1091,
    1188,
    18016,
    1451,
    18013,
    1358,
    1106,
    1189,
    1190,
    1009,
    1123,
    5076,
    1005,
    17012,
    1097,
    1030,
    1089,
    1044,
    1090,
    1031,
    1042,
    1029,
    1156,
    1155,
    1172,
    2480,
    1341,
    1368,
    1308,
    2852,
    1111,

    2839,
];

fn should_skip_js_file(file_name: &str, allow_js: bool) -> bool {
    if allow_js || !is_external_library_file(file_name) {
        return false;
    }
    matches!(
        script_kind_from_file_name(file_name),
        crate::ast::ScriptKind::Js | crate::ast::ScriptKind::Jsx
    )
}

fn read_and_parse(
    file_name: &str,
    host: &dyn CompilerHost,
) -> Result<(Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>), String> {
    let text = host
        .fs()
        .read_file(file_name)
        .ok_or_else(|| format!("Cannot read file '{file_name}'."))?;
    read_and_parse_text(file_name, text)
}

fn cached_parse(
    file_name: &str,
    text: &str,
) -> (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>) {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<(String, u64), (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>)>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let key = (file_name.to_string(), hasher.finish());
    if let Some(hit) = cache.lock().unwrap().get(&key) {
        return (Arc::clone(&hit.0), hit.1.clone());
    }
    let (file, diags) = Parser::parse_source_file_text_with_diagnostics(file_name, text.to_string());
    let file = Arc::new(file);
    cache
        .lock()
        .unwrap()
        .insert(key, (Arc::clone(&file), diags.clone()));
    (file, diags)
}

fn read_and_parse_text(
    file_name: &str,
    text: String,
) -> Result<(Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>), String> {
    let (file, diags) = cached_parse(file_name, &text);
    Ok((file, diags))
}

fn load_source_file(
    file_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
    allow_js: bool,
) -> Option<Arc<SourceFile>> {
    let normalized = tspath::normalize_path(file_name);
    if let Some(existing) = by_name.get(&normalized) {
        return Some(Arc::clone(existing));
    }

    if should_skip_js_file(&normalized, allow_js) {
        return None;
    }

    let (file, parse_diags) = match read_and_parse(&normalized, host) {
        Ok(result) => result,
        Err(msg) => {
            diagnostics.push(Arc::new(file_error_diagnostic(&normalized, &msg)));
            return None;
        }
    };

    for pd in &parse_diags {
        diagnostics.push(Arc::new(parser_diagnostic_to_diagnostic(
            Arc::clone(&file),
            pd,
        )));
    }

    by_name.insert(normalized.clone(), Arc::clone(&file));
    source_files.push(Arc::clone(&file));
    Some(file)
}

fn load_source_file_with_references(
    file_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
    allow_js: bool,
) {
    let normalized = tspath::normalize_path(file_name);
    if by_name.contains_key(&normalized) {
        return;
    }

    if should_skip_js_file(&normalized, allow_js) {
        return;
    }

    let (file, parse_diags) = match read_and_parse(&normalized, host) {
        Ok(result) => result,
        Err(msg) => {
            diagnostics.push(Arc::new(file_error_diagnostic(&normalized, &msg)));
            return;
        }
    };

    for pd in &parse_diags {
        diagnostics.push(Arc::new(parser_diagnostic_to_diagnostic(
            Arc::clone(&file),
            pd,
        )));
    }

    by_name.insert(normalized.clone(), Arc::clone(&file));

    let text = file.text.as_str();
    let refs = extract_reference_path_directives(text, &normalized);
    for ref_path in &refs {
        load_source_file_with_references(
            ref_path,
            host,
            source_files,
            by_name,
            diagnostics,
            allow_js,
        );
    }

    source_files.push(file);
}

fn extract_reference_path_directives(text: &str, containing_file: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let base_dir = tspath::get_directory_path(containing_file);
    for line in text.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("///") else {
            continue;
        };

        if !rest.trim_start().starts_with("<reference") {
            continue;
        }
        if let Some(start) = rest.find("path=\"") {
            let after = &rest[start + 6..];
            if let Some(end) = after.find('"') {
                let path = &after[..end];
                let resolved = if tspath::is_rooted_disk_path(path) {
                    tspath::normalize_path(path)
                } else {
                    tspath::normalize_path(&tspath::combine_paths(&base_dir, &[path]))
                };
                refs.push(resolved);
            }
        } else if let Some(start) = rest.find("path='") {
            let after = &rest[start + 6..];
            if let Some(end) = after.find('\'') {
                let path = &after[..end];
                let resolved = if tspath::is_rooted_disk_path(path) {
                    tspath::normalize_path(path)
                } else {
                    tspath::normalize_path(&tspath::combine_paths(&base_dir, &[path]))
                };
                refs.push(resolved);
            }
        }
    }
    refs
}

struct ReferenceTypesDirective {
    name: String,
    mode_value: Option<String>,
    #[allow(dead_code)]
    mode_value_range: (usize, usize),

    types_value_range: (usize, usize),
}

fn extract_reference_types_directives(text: &str) -> Vec<ReferenceTypesDirective> {
    let mut types = Vec::new();
    let mut line_start = 0usize;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let Some(rest) = trimmed.strip_prefix("///") else {
            line_start += line.len() + 1;
            continue;
        };

        for quote in ['"', '\''] {
            let marker = format!("types={quote}");
            if let Some(start) = rest.find(&marker) {
                let after = &rest[start + marker.len()..];
                if let Some(end) = after.find(quote) {
                    let name = &after[..end];
                    if name.is_empty() {
                        continue;
                    }

                    let mut mode_value = None;
                    let mut mode_value_range = (0usize, 0usize);
                    let attr_marker = "resolution-mode=";
                    if let Some(attr_pos) = rest.find(attr_marker) {
                        let val_area = &rest[attr_pos + attr_marker.len()..];
                        if let Some(q) = val_area.chars().next()
                            && (q == '"' || q == '\'')
                            && let Some(rel_end) = val_area[1..].find(q)
                        {
                            let val = &val_area[1..1 + rel_end];
                            mode_value = Some(val.to_string());
                            mode_value_range = (
                                line_start + leading + attr_pos + attr_marker.len() + 1,
                                line_start + leading + attr_pos + attr_marker.len() + 1 + rel_end,
                            );
                        }
                    }
                    types.push(ReferenceTypesDirective {
                        name: name.to_string(),
                        mode_value,
                        mode_value_range,

                        types_value_range: (
                            line_start + leading + 3 + start + marker.len(),
                            line_start + leading + 3 + start + marker.len() + end,
                        ),
                    });
                }
            }
        }
        line_start += line.len() + 1;
    }
    types
}

fn load_lib_recursive(
    lib_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    default_lib_names: &mut std::collections::HashSet<String>,
    visited: &mut std::collections::HashSet<String>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
) {
    if !visited.insert(lib_name.to_string()) {
        return;
    }
    let path = tspath::combine_paths(host.default_library_path(), &[lib_name]);
    let text = match host.fs().read_file(&path) {
        Some(t) => t,
        None => {

            return;
        }
    };

    let references = extract_reference_lib_directives(&text);
    for ref_lib in &references {
        let ref_name = format!("lib.{ref_lib}.d.ts");
        load_lib_recursive(
            &ref_name,
            host,
            source_files,
            by_name,
            default_lib_names,
            visited,
            diagnostics,
        );
    }

    let (file, parse_diags) = cached_parse(&path, &text);
    for pd in &parse_diags {
        diagnostics.push(Arc::new(parser_diagnostic_to_diagnostic(
            Arc::clone(&file),
            pd,
        )));
    }
    default_lib_names.insert(path.clone());
    by_name.insert(path.clone(), Arc::clone(&file));
    source_files.push(file);
}

fn extract_reference_lib_directives(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("///") {
            if let Some(start) = rest.find("lib=\"") {
                let after = &rest[start + 5..];
                if let Some(end) = after.find('"') {
                    refs.push(after[..end].to_string());
                }
            }
        }
    }
    refs
}

pub fn default_lib_file_names(options: &CompilerOptions) -> Vec<String> {
    if !options.lib.is_empty() {
        return options
            .lib
            .iter()
            .map(|l| {
                if l.starts_with("lib.") {
                    l.clone()
                } else {
                    format!("lib.{l}.d.ts")
                }
            })
            .collect();
    }
    let entry = match options.get_emit_script_target() {
        ScriptTarget::ESNext => "lib.esnext.full.d.ts",
        ScriptTarget::ES2025 => "lib.es2025.full.d.ts",
        ScriptTarget::ES2024 => "lib.es2024.full.d.ts",
        ScriptTarget::ES2023 => "lib.es2023.full.d.ts",
        ScriptTarget::ES2022 => "lib.es2022.full.d.ts",
        ScriptTarget::ES2021 => "lib.es2021.full.d.ts",
        ScriptTarget::ES2020 => "lib.es2020.full.d.ts",
        ScriptTarget::ES2019 => "lib.es2019.full.d.ts",
        ScriptTarget::ES2018 => "lib.es2018.full.d.ts",
        ScriptTarget::ES2017 => "lib.es2017.full.d.ts",
        ScriptTarget::ES2016 => "lib.es2016.full.d.ts",
        ScriptTarget::ES2015 => "lib.es6.d.ts",
        _ => "lib.d.ts",
    };
    vec![entry.to_string()]
}

fn parser_diagnostic_to_diagnostic(
    file: Arc<SourceFile>,
    pd: &crate::parser::ParserDiagnostic,
) -> Diagnostic {
    Diagnostic::new(Some(file), pd.range, pd.message, pd.message_args.clone())
}

fn file_error_diagnostic(file_name: &str, _message: &str) -> Diagnostic {
    use crate::diagnostics::FILE_0_NOT_FOUND;
    Diagnostic {
        file: None,
        loc: TextRange::undefined(),
        code: FILE_0_NOT_FOUND.code,
        category: Category::Error,
        message: Some(FILE_0_NOT_FOUND),
        message_key: FILE_0_NOT_FOUND.key,
        message_args: vec![file_name.to_string()],
        message_chain: Vec::new(),
        related_information: Vec::new(),
        reports_unnecessary: false,
        reports_deprecated: false,
        skipped_on_no_emit: false,
    }
}

#[allow(dead_code)]
fn _ensure_script_kind(file_name: &str) -> crate::ast::ScriptKind {
    script_kind_from_file_name(file_name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
#[allow(dead_code)]
pub enum FileIncludeKind {

    #[default]
    Import = 0,

    ReferenceFile = 1,

    TypeReferenceDirective = 2,

    LibReferenceDirective = 3,

    RootFile = 4,

    LibFile = 5,

    AutomaticTypeDirectiveFile = 6,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileIncludeReason {
    pub kind: FileIncludeKind,
    pub file_name: String,
}

impl FileIncludeReason {
    pub fn new(kind: FileIncludeKind, file_name: impl Into<String>) -> Self {
        Self {
            kind,
            file_name: file_name.into(),
        }
    }

    pub fn is_referenced_file(&self) -> bool {
        matches!(
            self.kind,
            FileIncludeKind::ReferenceFile
                | FileIncludeKind::TypeReferenceDirective
                | FileIncludeKind::LibReferenceDirective
        )
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DuplicateSourceFile {
    pub file_name: String,
    pub hash: u128,
    pub script_kind: crate::ast::ScriptKind,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct LibFile {
    pub name: String,
    pub path: String,
    pub replaced: bool,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ProgramBuildInfo {
    pub file_count: usize,
    pub line_count: usize,
    pub identifier_count: usize,
    pub symbol_count: usize,
    pub type_count: usize,
    pub instantiation_count: usize,
}

#[allow(dead_code)]
impl Program {

    pub fn get_source_files(&self) -> Vec<Arc<SourceFile>> {
        self.source_files.clone()
    }

    pub fn get_file_include_reasons(&self) -> HashMap<String, Vec<FileIncludeReason>> {

        HashMap::new()
    }

    pub fn is_missing_path(&self, path: &str) -> bool {
        !self.source_files_by_name.contains_key(path)
    }

    pub fn get_source_file_by_path(&self, path: &str) -> Option<Arc<SourceFile>> {
        self.source_files_by_name.get(path).cloned()
    }

    pub fn duplicate_source_files(&self) -> &[DuplicateSourceFile] {

        &[]
    }

    pub fn line_count(&self) -> usize {
        self.source_files
            .iter()
            .map(|f| f.text.lines().count())
            .sum()
    }

    pub fn identifier_count(&self) -> usize {

        0
    }

    pub fn symbol_count(&self) -> usize {
        self.symbol_map.symbols.len()
    }

    pub fn type_count(&self) -> usize {

        0
    }

    pub fn instantiation_count(&self) -> usize {

        0
    }

    pub fn get_program_build_info(&self) -> ProgramBuildInfo {
        ProgramBuildInfo {
            file_count: self.source_files.len(),
            line_count: self.line_count(),
            identifier_count: self.identifier_count(),
            symbol_count: self.symbol_count(),
            type_count: self.type_count(),
            instantiation_count: self.instantiation_count(),
        }
    }

    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.host.use_case_sensitive_file_names()
    }

    pub fn get_current_directory(&self) -> &str {
        self.host.current_directory()
    }

    pub fn get_resolved_modules(
        &self,
    ) -> HashMap<String, Vec<(String, Option<crate::module::ResolvedModule>)>> {

        HashMap::new()
    }

    pub fn get_packages_map(&self) -> HashMap<String, bool> {

        HashMap::new()
    }

    pub fn single_threaded(&self) -> bool {
        true
    }
}

#[allow(dead_code)]
pub fn process_root_file(
    file_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
    allow_js: bool,
) {
    load_source_file_with_references(
        file_name,
        host,
        source_files,
        by_name,
        diagnostics,
        allow_js,
    );
}

#[allow(dead_code)]
pub fn process_source_file(
    file_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
    allow_js: bool,
) -> Option<Arc<SourceFile>> {
    load_source_file(
        file_name,
        host,
        source_files,
        by_name,
        diagnostics,
        allow_js,
    )
}

#[allow(dead_code)]
pub fn process_all_program_files(
    root_file_names: &[String],
    host: &dyn CompilerHost,
    options: &CompilerOptions,
) -> (
    Vec<Arc<SourceFile>>,
    HashMap<String, Arc<SourceFile>>,
    Vec<Arc<Diagnostic>>,
) {
    let mut source_files: Vec<Arc<SourceFile>> = Vec::new();
    let mut by_name: HashMap<String, Arc<SourceFile>> = HashMap::new();
    let mut diagnostics: Vec<Arc<Diagnostic>> = Vec::new();
    let allow_js = options.get_allow_js();

    for file_name in root_file_names {
        process_root_file(
            file_name,
            host,
            &mut source_files,
            &mut by_name,
            &mut diagnostics,
            allow_js,
        );
    }

    (source_files, by_name, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundled::{BundledFS, lib_path};
    use crate::core::compiler_options::CompilerOptions;
    use crate::core::tristate::Tristate;
    use crate::tsoptions::parse_command_line;
    use crate::vfs::{InMemoryFS, OsFS};

    #[test]
    fn program_parses_input_files() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/a.ts", "let x = 1;");
        fs.insert_file("/proj/b.ts", "let y = ;");

        let args: Vec<String> = vec![
            "--noLib".to_string(),
            "/proj/a.ts".to_string(),
            "/proj/b.ts".to_string(),
        ];
        let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
        let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });
        assert_eq!(program.source_files().len(), 2);

        assert!(
            program
                .diagnostics()
                .iter()
                .any(|d| d.category == Category::Error)
        );
    }

    #[test]
    fn program_does_not_load_bundled_libs_without_root_files() {

        let fs = Arc::new(BundledFS::new(Arc::new(OsFS)));
        let args: Vec<String> = vec![];
        let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
        let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });
        assert!(program.source_files().is_empty());
    }

    #[test]
    fn program_loads_bundled_libs_with_root_files() {

        let inner = Arc::new(InMemoryFS::new());
        inner.insert_dir("/proj");
        inner.insert_file("/proj/a.ts", "let x = 1;");
        let fs = Arc::new(BundledFS::new(inner));
        let args: Vec<String> = vec!["/proj/a.ts".to_string()];
        let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
        let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        assert!(program.source_files().len() > 1);
        assert!(
            program
                .source_files()
                .iter()
                .any(|file| file.file_name == "/proj/a.ts")
        );
    }

    #[test]
    fn extract_reference_libs() {
        let text = "/// <reference lib=\"es5\" />\n/// <reference lib=\"dom\" />\ninterface X {}";
        let refs = extract_reference_lib_directives(text);
        assert_eq!(refs, vec!["es5", "dom"]);
    }

    #[test]
    fn program_file_ordering_with_reference_paths() {
        let fs = Arc::new(InMemoryFS::new());

        let files = [
            (
                "/dev/src/index.ts",
                "/// <reference path='/dev/src2/a/5.ts' />\n/// <reference path='/dev/src2/a/10.ts' />",
            ),
            ("/dev/src2/a/5.ts", "/// <reference path='4.ts' />"),
            ("/dev/src2/a/4.ts", "/// <reference path='b/3.ts' />"),
            ("/dev/src2/a/b/3.ts", "/// <reference path='2.ts' />"),
            ("/dev/src2/a/b/2.ts", "/// <reference path='c/1.ts' />"),
            ("/dev/src2/a/b/c/1.ts", "console.log('hello');"),
            ("/dev/src2/a/10.ts", "/// <reference path='b/c/d/9.ts' />"),
            ("/dev/src2/a/b/c/d/9.ts", "/// <reference path='e/8.ts' />"),
            ("/dev/src2/a/b/c/d/e/8.ts", "/// <reference path='7.ts' />"),
            (
                "/dev/src2/a/b/c/d/e/7.ts",
                "/// <reference path='f/6.ts' />",
            ),
            ("/dev/src2/a/b/c/d/e/f/6.ts", "console.log('world!');"),
        ];
        for (name, content) in &files {
            fs.insert_file(name, content);
        }

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts
            },
            file_names: vec!["/dev/src/index.ts".to_string()],
            errors: vec![],
            config_file_name: String::new(),
            raw_options: None,
            include: vec![],
            exclude: vec![],
            files_spec: vec![],
            has_include_spec: false,
            has_exclude_spec: false,
            has_files_spec: false,
            references: vec![],
            compile_on_save: None,
            watch: false,
            watch_options: Default::default(),
        };
        let host = Arc::new(CompilerHostImpl::new(
            fs,
            "/dev/src".to_string(),
            lib_path(),
        ));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        let actual: Vec<&str> = program
            .source_files()
            .iter()
            .map(|f| f.file_name.as_str())
            .collect();

        let expected = vec![
            "/dev/src2/a/b/c/1.ts",
            "/dev/src2/a/b/2.ts",
            "/dev/src2/a/b/3.ts",
            "/dev/src2/a/4.ts",
            "/dev/src2/a/5.ts",
            "/dev/src2/a/b/c/d/e/f/6.ts",
            "/dev/src2/a/b/c/d/e/7.ts",
            "/dev/src2/a/b/c/d/e/8.ts",
            "/dev/src2/a/b/c/d/9.ts",
            "/dev/src2/a/10.ts",
            "/dev/src/index.ts",
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn program_file_ordering_imports() {
        let fs = Arc::new(InMemoryFS::new());

        for dir in [
            "/dev/src",
            "/dev/src2/a",
            "/dev/src2/a/b",
            "/dev/src2/a/b/c",
            "/dev/src2/a/b/c/d",
            "/dev/src2/a/b/c/d/e",
            "/dev/src2/a/b/c/d/e/f",
        ] {
            fs.insert_dir(dir);
        }
        let files = [
            (
                "/dev/src/index.ts",
                "import * as five from '../src2/a/5.ts';\nimport * as ten from '../src2/a/10.ts';",
            ),
            ("/dev/src2/a/5.ts", "import * as four from './4.ts';"),
            ("/dev/src2/a/4.ts", "import * as three from './b/3.ts';"),
            ("/dev/src2/a/b/3.ts", "import * as two from './2.ts';"),
            ("/dev/src2/a/b/2.ts", "import * as one from './c/1.ts';"),
            ("/dev/src2/a/b/c/1.ts", "console.log('hello');"),
            ("/dev/src2/a/10.ts", "import * as nine from './b/c/d/9.ts';"),
            (
                "/dev/src2/a/b/c/d/9.ts",
                "import * as eight from './e/8.ts';",
            ),
            (
                "/dev/src2/a/b/c/d/e/8.ts",
                "import * as seven from './7.ts';",
            ),
            (
                "/dev/src2/a/b/c/d/e/7.ts",
                "import * as six from './f/6.ts';",
            ),
            ("/dev/src2/a/b/c/d/e/f/6.ts", "console.log('world!');"),
        ];
        for (name, content) in &files {
            fs.insert_file(name, content);
        }

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts
            },
            file_names: vec!["/dev/src/index.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(
            fs,
            "/dev/src".to_string(),
            lib_path(),
        ));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        let actual: Vec<&str> = program
            .source_files()
            .iter()
            .map(|f| f.file_name.as_str())
            .collect();
        let expected = vec![
            "/dev/src/index.ts",
            "/dev/src2/a/5.ts",
            "/dev/src2/a/10.ts",
            "/dev/src2/a/b/c/d/9.ts",
            "/dev/src2/a/b/c/d/e/8.ts",
            "/dev/src2/a/b/c/d/e/7.ts",
            "/dev/src2/a/b/c/d/e/f/6.ts",
            "/dev/src2/a/4.ts",
            "/dev/src2/a/b/3.ts",
            "/dev/src2/a/b/2.ts",
            "/dev/src2/a/b/c/1.ts",
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn program_file_ordering_cycles() {
        let fs = Arc::new(InMemoryFS::new());
        for dir in [
            "/dev/src",
            "/dev/src2/a",
            "/dev/src2/a/b",
            "/dev/src2/a/b/c",
            "/dev/src2/a/b/c/d",
            "/dev/src2/a/b/c/d/e",
            "/dev/src2/a/b/c/d/e/f",
        ] {
            fs.insert_dir(dir);
        }
        let files = [
            (
                "/dev/src/index.ts",
                "import * as five from '../src2/a/5.ts';\nimport * as ten from '../src2/a/10.ts';",
            ),
            ("/dev/src2/a/5.ts", "import * as four from './4.ts';"),
            ("/dev/src2/a/4.ts", "import * as three from './b/3.ts';"),
            (
                "/dev/src2/a/b/3.ts",
                "import * as two from './2.ts';\nimport * as cycle from '/dev/src/index.ts';",
            ),
            ("/dev/src2/a/b/2.ts", "import * as one from './c/1.ts';"),
            ("/dev/src2/a/b/c/1.ts", "console.log('hello');"),
            ("/dev/src2/a/10.ts", "import * as nine from './b/c/d/9.ts';"),
            (
                "/dev/src2/a/b/c/d/9.ts",
                "import * as eight from './e/8.ts';\nimport * as cycle from '/dev/src/index.ts';",
            ),
            (
                "/dev/src2/a/b/c/d/e/8.ts",
                "import * as seven from './7.ts';",
            ),
            (
                "/dev/src2/a/b/c/d/e/7.ts",
                "import * as six from './f/6.ts';",
            ),
            ("/dev/src2/a/b/c/d/e/f/6.ts", "console.log('world!');"),
        ];
        for (name, content) in &files {
            fs.insert_file(name, content);
        }

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts
            },
            file_names: vec!["/dev/src/index.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(
            fs,
            "/dev/src".to_string(),
            lib_path(),
        ));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        let actual: Vec<&str> = program
            .source_files()
            .iter()
            .map(|f| f.file_name.as_str())
            .collect();
        let expected = vec![
            "/dev/src/index.ts",
            "/dev/src2/a/5.ts",
            "/dev/src2/a/10.ts",
            "/dev/src2/a/b/c/d/9.ts",
            "/dev/src2/a/b/c/d/e/8.ts",
            "/dev/src2/a/b/c/d/e/7.ts",
            "/dev/src2/a/b/c/d/e/f/6.ts",
            "/dev/src2/a/4.ts",
            "/dev/src2/a/b/3.ts",
            "/dev/src2/a/b/2.ts",
            "/dev/src2/a/b/c/1.ts",
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn program_resolves_module_imports() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/src");
        fs.insert_file(
            "/src/main.ts",
            "import { foo } from \"./foo\"; export const x = foo;",
        );
        fs.insert_file("/src/foo.ts", "export const foo: number = 42;");

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts
            },
            file_names: vec!["/src/main.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(
            fs,
            "/src".to_string(),
            "lib.d.ts".to_string(),
        ));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        assert_eq!(program.source_files().len(), 2);
        assert!(
            program.get_source_file("/src/foo.ts").is_some(),
            "expected /src/foo.ts to be loaded via import resolution"
        );
        assert!(
            program.get_source_file("/src/main.ts").is_some(),
            "expected /src/main.ts to be loaded as a root file"
        );
    }

    #[test]
    fn program_resolves_transitive_module_imports() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/src");
        fs.insert_file(
            "/src/a.ts",
            "import { b } from \"./b\"; export const a = b;",
        );
        fs.insert_file(
            "/src/b.ts",
            "import { c } from \"./c\"; export const b = c;",
        );
        fs.insert_file("/src/c.ts", "export const c: number = 3;");

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts
            },
            file_names: vec!["/src/a.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(
            fs,
            "/src".to_string(),
            "lib.d.ts".to_string(),
        ));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        assert_eq!(program.source_files().len(), 3);
        assert!(program.get_source_file("/src/b.ts").is_some());
        assert!(program.get_source_file("/src/c.ts").is_some());
    }

    #[test]
    fn include_processor_diagnostics_with_missing_file_casing() {
        let fs = Arc::new(InMemoryFS::with_case_sensitivity(true));
        fs.insert_dir("/src");

        fs.insert_file("/src/myFile.ts", "export const y = 2;");

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts.skip_lib_check = Tristate::True;
                opts
            },

            file_names: vec!["/src/MyFile.ts".to_string(), "/src/myFile.ts".to_string()],
            errors: vec![],
            config_file_name: String::new(),
            raw_options: None,
            include: vec![],
            exclude: vec![],
            files_spec: vec![],
            has_include_spec: false,
            has_exclude_spec: false,
            has_files_spec: false,
            references: vec![],
            compile_on_save: None,
            watch: false,
            watch_options: Default::default(),
        };
        let host = Arc::new(CompilerHostImpl::new(fs, "/".to_string(), lib_path()));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        let diags = program.diagnostics();
        assert!(
            diags.iter().any(|d| d.category == Category::Error),
            "expected at least one error diagnostic for missing /src/MyFile.ts, got: {:?}",
            diags
        );

        assert!(
            program.get_source_file("/src/myFile.ts").is_some(),
            "expected /src/myFile.ts to be loaded"
        );
    }

    #[test]
    fn extract_reference_path_directives_resolves_relative() {
        let text = "/// <reference path='./b/3.ts' />\n/// <reference path='/abs/4.ts' />";
        let refs = extract_reference_path_directives(text, "/dev/src2/a/5.ts");
        assert_eq!(refs, vec!["/dev/src2/a/b/3.ts", "/abs/4.ts"]);
    }

    #[test]
    fn extract_reference_path_directives_single_quotes() {
        let text = "/// <reference path='b/3.ts' />";
        let refs = extract_reference_path_directives(text, "/dev/src2/a/5.ts");
        assert_eq!(refs, vec!["/dev/src2/a/b/3.ts"]);
    }

    fn parse_bundled_lib(lib_name: &str) -> Vec<crate::parser::ParserDiagnostic> {
        let content = crate::bundled::lib_contents(lib_name)
            .unwrap_or_else(|| panic!("bundled lib '{lib_name}' not found"));
        let (_file, diags) = crate::parser::Parser::parse_source_file_text_with_diagnostics(
            &format!("/bundled/{lib_name}"),
            content.to_string(),
        );
        diags
    }

    fn assert_no_parser_errors(lib_name: &str, diags: &[crate::parser::ParserDiagnostic]) {
        let errors: Vec<_> = diags
            .iter()
            .filter(|d| d.message.category == crate::diagnostics::Category::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "{lib_name} should parse with zero errors, got {}:\n{}",
            errors.len(),
            errors
                .iter()
                .map(|d| format!("  {:?}: {}", d.message.code, d.message.text))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn bundled_lib_es2015_iterable_parses_without_errors() {
        let diags = parse_bundled_lib("lib.es2015.iterable.d.ts");
        assert_no_parser_errors("lib.es2015.iterable.d.ts", &diags);
    }

    #[test]
    fn bundled_lib_dom_parses_without_errors() {
        let diags = parse_bundled_lib("lib.dom.d.ts");
        assert_no_parser_errors("lib.dom.d.ts", &diags);
    }

    #[test]
    fn bundled_lib_es5_parses_without_errors() {
        let diags = parse_bundled_lib("lib.es5.d.ts");
        assert_no_parser_errors("lib.es5.d.ts", &diags);
    }

    #[test]
    fn bundled_lib_es2015_collection_parses_without_errors() {
        let diags = parse_bundled_lib("lib.es2015.collection.d.ts");
        assert_no_parser_errors("lib.es2015.collection.d.ts", &diags);
    }

    #[test]
    fn bundled_lib_decorators_parses_without_errors() {
        let diags = parse_bundled_lib("lib.decorators.d.ts");
        assert_no_parser_errors("lib.decorators.d.ts", &diags);
    }

    #[test]
    fn node_modules_js_skipped_when_allow_js_false() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/mypkg");

        fs.insert_file(
            "/proj/node_modules/mypkg/index.js",
            "module.exports = { x: 1 };\nfunction f(a, b) { return a + b; }\n",
        );
        fs.insert_file(
            "/proj/node_modules/mypkg/package.json",
            r#"{"name": "mypkg", "version": "1.0.0", "main": "index.js"}"#,
        );
        fs.insert_file(
            "/proj/src/main.ts",
            "import * as pkg from 'mypkg';\nexport const v = pkg;",
        );

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts
            },
            file_names: vec!["/proj/src/main.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        assert!(
            program
                .get_source_file("/proj/node_modules/mypkg/index.js")
                .is_none(),
            "expected node_modules .js file to be skipped when allowJs is false"
        );

        let has_syntax_error = program
            .diagnostics()
            .iter()
            .any(|d| d.code == 1003 || d.code == 1005);
        assert!(
            !has_syntax_error,
            "expected no TS1003/TS1005 syntax diagnostics from node_modules .js, got: {:?}",
            program
                .diagnostics()
                .iter()
                .map(|d| (d.code, d.message_args.clone()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn node_modules_js_loaded_when_allow_js_true() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/mypkg");
        fs.insert_file("/proj/node_modules/mypkg/index.js", "export const x = 1;\n");
        fs.insert_file(
            "/proj/node_modules/mypkg/package.json",
            r#"{"name": "mypkg", "version": "1.0.0", "main": "index.js"}"#,
        );
        fs.insert_file(
            "/proj/src/main.ts",
            "import { x } from 'mypkg';\nexport const v = x;",
        );

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts.allow_js = Tristate::True;
                opts
            },
            file_names: vec!["/proj/src/main.ts".to_string()],
            ..Default::default()
        };
        let host = Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Program::new(ProgramOptions {
            config: parsed,
            host,
        });

        assert!(
            program
                .get_source_file("/proj/node_modules/mypkg/index.js")
                .is_some(),
            "expected node_modules .js file to be loaded when allowJs is true; files: {:?}",
            program
                .source_files()
                .iter()
                .map(|f| f.file_name.as_str())
                .collect::<Vec<_>>()
        );
    }
}

fn ambient_module_exists(
    source_files: &[Arc<crate::ast::SourceFile>],
    name: &str,
) -> bool {
    for file in source_files {
        if let crate::ast::NodeData::SourceFile(sf) = &file.node.data {
            for stmt in sf.statements.iter() {

                let file_is_external = file.external_module_indicator.is_some();
                if let crate::ast::NodeData::ModuleDeclaration(md) = &stmt.data
                    && md.name.kind == crate::ast::SyntaxKind::StringLiteral
                    && strip_quotes(md.name.text()) == name
                    && !file_is_external
                {
                    return true;
                }
            }
        }
    }
    false
}

fn strip_quotes(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2
        && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
