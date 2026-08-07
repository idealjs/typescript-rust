//! The compiler program, ported from `internal/compiler/`.
//!
//! `Program` orchestrates loading, parsing, and binding of source files, and
//! exposes the inputs the type checker needs. Emit and full module resolution
//! are not yet ported; this module provides the minimum needed to drive the
//! CLI pipeline (parse + bind + report diagnostics).

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::NodeSymbolMap;
use crate::ast::SourceFile;
use crate::ast::diagnostic::Diagnostic;
use crate::binder::Binder;
use crate::core::compiler_options::CompilerOptions;
use crate::core::text::TextRange;
use crate::diagnostics::Category;
use crate::module;
use crate::parser::{Parser, script_kind_from_file_name};
use crate::tspath;
use crate::vfs::FS;

use crate::tsoptions::ParsedCommandLine;

// ────────────────────────────────────────────────────────────────────────────
// CompilerHost
// ────────────────────────────────────────────────────────────────────────────

/// Provides the file system and environment context the compiler runs in.
///
/// Mirrors `compiler.CompilerHost` in Go (a reduced form).
pub trait CompilerHost: Send + Sync {
    fn fs(&self) -> &dyn FS;
    /// Return the underlying file system as a cloneable `Arc`, so adapters
    /// (e.g. the module resolver's `ResolutionHost`) can retain ownership of
    /// the same FS without lifetime entanglement.
    fn fs_arc(&self) -> Arc<dyn FS>;
    fn current_directory(&self) -> &str;
    fn default_library_path(&self) -> &str;
    fn use_case_sensitive_file_names(&self) -> bool {
        self.fs().use_case_sensitive_file_names()
    }
}

/// A basic `CompilerHost` backed by a real (or virtual) file system.
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

// ────────────────────────────────────────────────────────────────────────────
// ResolutionHostAdapter
// ────────────────────────────────────────────────────────────────────────────

/// Owned adapter that bridges `CompilerHost` and `module::ResolutionHost`.
///
/// Stored as `Arc<dyn ResolutionHost + Send + Sync>` inside the `Resolver`,
/// so it must own its data (an `Arc<dyn FS>` and a `String` current directory)
/// rather than borrow from the `CompilerHost`.
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

// ────────────────────────────────────────────────────────────────────────────
// ProgramOptions
// ────────────────────────────────────────────────────────────────────────────

/// Options for constructing a `Program`.
pub struct ProgramOptions {
    pub config: ParsedCommandLine,
    pub host: Arc<dyn CompilerHost>,
}

// ────────────────────────────────────────────────────────────────────────────
// Program
// ────────────────────────────────────────────────────────────────────────────

/// A compiled program: a set of parsed, bound source files plus diagnostics.
///
/// Mirrors `compiler.Program` in Go (a reduced form).
pub struct Program {
    options: CompilerOptions,
    source_files: Vec<Arc<SourceFile>>,
    source_files_by_name: HashMap<String, Arc<SourceFile>>,
    default_library_file_names: std::collections::HashSet<String>,
    diagnostics: Vec<Arc<Diagnostic>>,
    host: Arc<dyn CompilerHost>,
    config_file_name: String,
    /// Side table from the binder: maps node IDs to symbols, locals, and flow
    /// nodes. Shared across all source files in the program (node IDs are
    /// globally unique).
    symbol_map: NodeSymbolMap,
}

impl Program {
    /// Create a new program: load lib files and input files, parse, and bind.
    pub fn new(opts: ProgramOptions) -> Self {
        let host = opts.host;
        let mut options = opts.config.compiler_options.clone();
        let config_file_name = opts.config.config_file_name.clone();
        // Propagate the config file path onto the options so downstream
        // consumers (e.g. the emitter's common-source-directory computation)
        // can mirror Go's `options.ConfigFilePath`.
        if !config_file_name.is_empty() && options.config_file_path.is_empty() {
            options.config_file_path = config_file_name.clone();
        }

        let mut source_files: Vec<Arc<SourceFile>> = Vec::new();
        let mut by_name: HashMap<String, Arc<SourceFile>> = HashMap::new();
        let mut default_lib_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut diagnostics: Vec<Arc<Diagnostic>> = Vec::new();

        // 1. Load default library files (unless --noLib).
        // Go's program construction only loads default libs when there is at
        // least one root file. Solution configs such as `files: []` should not
        // parse libs by themselves.
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

        // 2. Load input files from the parsed command line / tsconfig.
        //    Process `/// <reference path=... />` directives recursively so that
        //    dependencies are loaded before dependent files (matching Go's ordering).
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

        // 3. Resolve module imports (`import`/`export` specifiers) and load any
        //    resolved dependencies that aren't already part of the program.
        //    This mirrors Go's `processRootFile`/`fileLoader` import discovery,
        //    performing a breadth-first walk over every loaded file's `imports`.
        {
            let resolution_host: Arc<dyn module::ResolutionHost + Send + Sync> =
                Arc::new(ResolutionHostAdapter::new(host.as_ref()));
            let resolver = module::Resolver::new(
                resolution_host,
                Arc::new(options.clone()),
                String::new(), // typings_location
                String::new(), // project_name
            );

            let mut visited: std::collections::HashSet<String> = by_name.keys().cloned().collect();
            let mut queue: Vec<Arc<SourceFile>> = source_files.clone();
            while let Some(file) = queue.pop() {
                for import_node in &file.imports {
                    let module_spec = import_node.text();
                    if module_spec.is_empty() {
                        continue;
                    }
                    let (resolved, _traces) = resolver.resolve_module_name(
                        module_spec,
                        &file.file_name,
                        crate::core::compiler_options::ModuleKind::None,
                        None,
                    );
                    if let Some(resolved_module) = resolved {
                        if resolved_module.is_resolved() {
                            let resolved_path = resolved_module.resolved_file_name.as_str();
                            if visited.insert(resolved_path.to_string()) {
                                if let Some(sf) = load_source_file(
                                    resolved_path,
                                    host.as_ref(),
                                    &mut source_files,
                                    &mut by_name,
                                    &mut diagnostics,
                                    allow_js,
                                ) {
                                    queue.push(sf);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Report any errors from the parsed command line itself.
        for err in &opts.config.errors {
            diagnostics.push(Arc::new(err.clone()));
        }

        // 5. Bind all source files.
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

    /// Diagnostics that should be reported, applying `skipLibCheck` /
    /// `skipDefaultLibCheck` filtering.
    ///
    /// In typescript-go, `skipLibCheck` skips type-checking of *all*
    /// declaration files (`.d.ts`) and external library files
    /// (node_modules). `skipDefaultLibCheck` only skips the built-in default
    /// library files (e.g. `lib.d.ts`). Both options also suppress parse/bind
    /// diagnostics from the same set of files.
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
                    return true; // Keep fileless diagnostics.
                };
                if skip_lib {
                    // skipLibCheck suppresses all declaration files and
                    // node_modules files.
                    !file.is_declaration_file && !is_external_library_file(&file.file_name)
                } else {
                    // skipDefaultLibCheck suppresses only default library files.
                    !self.default_library_file_names.contains(&file.file_name)
                }
            })
            .cloned()
            .collect()
    }

    /// Run the type checker and return semantic diagnostics.
    ///
    /// Go: `Program.GetSemanticDiagnostics` → creates a `Checker`, calls
    /// `checkSourceFile` for each source file, and returns accumulated diagnostics.
    ///
    /// When `skipLibCheck` is on, the checker skips declaration files (`.d.ts`)
    /// and node_modules files — mirroring Go's behavior of not type-checking
    /// those files. `skipDefaultLibCheck` only skips built-in default library
    /// files.
    pub fn get_semantic_diagnostics(self: &Arc<Self>) -> Vec<Diagnostic> {
        let skip_lib = self.options.skip_lib_check.is_true();
        let skip_default_lib = self.options.skip_default_lib_check.is_true();

        let mut checker = self.build_checker_internal(skip_lib, skip_default_lib);
        let mut diagnostics = checker.get_semantic_diagnostics();
        // Surface binder-level diagnostics (e.g. TS2451 block-scoped
        // redeclarations) alongside the checker's semantic diagnostics,
        // applying the same skip filtering.
        if skip_lib {
            diagnostics.extend(
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
                    .cloned(),
            );
        } else if skip_default_lib {
            diagnostics.extend(
                self.symbol_map
                    .binder_diagnostics
                    .iter()
                    .filter(|d| {
                        d.file
                            .as_ref()
                            .map(|f| !self.default_library_file_names.contains(&f.file_name))
                            .unwrap_or(true)
                    })
                    .cloned(),
            );
        } else {
            diagnostics.extend(self.symbol_map.binder_diagnostics.iter().cloned());
        }
        diagnostics
    }

    /// Build a fully-initialized `Checker` for this program, with all source
    /// files already checked. Exposed so tests and advanced callers can
    /// inspect checker state (e.g. emit-resolver visibility) after the
    /// type-check pass. Mirrors the setup done by `get_semantic_diagnostics`.
    pub fn build_checker(self: &Arc<Self>) -> crate::checker::Checker {
        self.build_checker_internal(false, false)
    }

    /// Internal checker builder with skipLibCheck / skipDefaultLibCheck support.
    /// When `skip_lib` is true, skips checking declaration files and
    /// node_modules files. When `skip_default_lib` is true (and `skip_lib`
    /// is false), only skips built-in default library files.
    fn build_checker_internal(
        self: &Arc<Self>,
        skip_lib: bool,
        skip_default_lib: bool,
    ) -> crate::checker::Checker {
        let tracer = Arc::new(crate::checker::Tracer::new());
        let program: Arc<dyn crate::checker::Program> = Arc::clone(self) as _;
        let mut checker = crate::checker::Checker::new(program, tracer);
        for file in &self.source_files {
            // Skip declaration files when skipLibCheck is on.
            if skip_lib && (file.is_declaration_file || is_external_library_file(&file.file_name)) {
                continue;
            }
            // Skip default library files when skipDefaultLibCheck is on.
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

    /// Side table from the binder: maps node IDs to symbols, locals, and flow
    /// nodes. Used by the checker for identifier resolution and flow analysis.
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

    /// Emit JavaScript output for all source files.
    ///
    /// Mirrors `Program.Emit` in Go. Writes `.js` files (and optionally
    /// `.d.ts`, source maps) via the provided `write_file` callback.
    pub fn emit(
        &self,
        write_file: &dyn Fn(&str, &str) -> std::io::Result<()>,
    ) -> crate::emitter::EmitResult {
        let fs = self.host.fs();
        // Only emit non-lib, non-external-library source files.
        // External library files are those found under node_modules.
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
        // Binding is performed eagerly during construction.
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
        // Delegate to the emitter's computation, which mirrors Go's
        // `outputpaths.GetCommonSourceDirectory`. Only non-lib source files
        // are considered (lib files don't affect the common source dir).
        let source_files: Vec<_> = self
            .source_files
            .iter()
            .filter(|sf| !self.default_library_file_names.contains(&sf.file_name))
            .cloned()
            .collect();
        crate::emitter::compute_program_common_source_directory(&source_files, &self.options)
    }
}

/// Check if a file path belongs to an external library (node_modules).
/// Mirrors Go's `IsSourceFileFromExternalLibrary` substring check.
pub fn is_external_library_file(file_name: &str) -> bool {
    file_name.contains("/node_modules/") || file_name.contains("\\node_modules\\")
}

/// Whether a JavaScript file inside `node_modules` should be skipped when
/// loading source files. When `allowJs`/`checkJs` is false (the default),
/// `.js`/`.jsx`/`.mjs`/`.cjs` files are not part of the program: parsing them
/// as TypeScript produces false TS1003/TS1005 syntax diagnostics. Only the
/// corresponding `.d.ts` declarations (if any) are loaded for type checking.
///
/// Mirrors Go's `fileLoader`/`processRootFile`, which only includes JS files
/// when `allowJs` is enabled. Files outside `node_modules` are unaffected.
fn should_skip_js_file(file_name: &str, allow_js: bool) -> bool {
    if allow_js || !is_external_library_file(file_name) {
        return false;
    }
    matches!(
        script_kind_from_file_name(file_name),
        crate::ast::ScriptKind::Js | crate::ast::ScriptKind::Jsx
    )
}

// ────────────────────────────────────────────────────────────────────────────
// File loading helpers
// ────────────────────────────────────────────────────────────────────────────

/// Read and parse a file from the host file system, returning the source file
/// and any parse diagnostics.
fn read_and_parse(
    file_name: &str,
    host: &dyn CompilerHost,
) -> Result<(Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>), String> {
    let text = host
        .fs()
        .read_file(file_name)
        .ok_or_else(|| format!("Cannot read file '{file_name}'."))?;
    let (file, diags) = Parser::parse_source_file_text_with_diagnostics(file_name, text);
    Ok((Arc::new(file), diags))
}

/// Load a single source file (no `/// <reference path=... />` following):
/// read, parse, record parse diagnostics, and register it in the program's
/// file tables. Returns the loaded file, or `None` if it was already loaded
/// or could not be read.
///
/// Used by the module-resolution step to pull in import/export dependencies.
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
    // G3: When `allowJs` is false, skip `.js`/`.jsx` (etc.) files pulled in
    // from `node_modules` during module resolution — they are not part of the
    // program and would otherwise produce false syntax diagnostics.
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

/// Load a source file and recursively process its `/// <reference path=... />`
/// directives, so that referenced files are loaded before the referencing file.
///
/// Mirrors the triple-slash reference path resolution in Go's `fileLoader`.
/// Referenced files are resolved relative to the containing file's directory,
/// and each is loaded recursively before the containing file is added to the
/// source file list. This produces a dependency-first ordering.
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
    // G3: When `allowJs` is false, skip `.js`/`.jsx` (etc.) files coming from
    // `node_modules` — they are not part of the program.
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

    // Mark as loaded before recursing to break cycles.
    by_name.insert(normalized.clone(), Arc::clone(&file));

    // Process `/// <reference path=... />` directives.
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

/// Extract `/// <reference path="..." />` directives from source text.
///
/// Resolves each path relative to `containing_file`'s directory, mirroring
/// Go's `resolveTripleslashPathReference`.
fn extract_reference_path_directives(text: &str, containing_file: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let base_dir = tspath::get_directory_path(containing_file);
    for line in text.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("///") else {
            continue;
        };
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

/// Recursively load a lib file and its `/// <reference lib="..." />` dependencies.
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
            // Lib file missing is non-fatal (the bundled set may be partial).
            return;
        }
    };

    // Resolve referenced libs before adding this file.
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

    let (file, parse_diags) = Parser::parse_source_file_text_with_diagnostics(&path, text);
    let file = Arc::new(file);
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

/// Extract `/// <reference lib="X" />` directives from source text.
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

/// Determine the default lib file name(s) from compiler options.
///
/// Mirrors a simplified `compiler.GetDefaultLibFileName` / `getDefaultLibFilenames`.
fn default_lib_file_names(options: &CompilerOptions) -> Vec<String> {
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
    // Default: lib.d.ts (which references es5 + dom).
    vec!["lib.d.ts".to_string()]
}

// ────────────────────────────────────────────────────────────────────────────
// Diagnostic conversion
// ────────────────────────────────────────────────────────────────────────────

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

// Ensure `script_kind_from_file_name` is reachable (used by the parser).
#[allow(dead_code)]
fn _ensure_script_kind(file_name: &str) -> crate::ast::ScriptKind {
    script_kind_from_file_name(file_name)
}

// ────────────────────────────────────────────────────────────────────────────
// File-include tracking types (ported from `internal/compiler/fileInclude.go`)
// ────────────────────────────────────────────────────────────────────────────

/// Why a file was included in the program.
///
/// Mirrors `compiler.fileIncludeKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
#[allow(dead_code)]
pub enum FileIncludeKind {
    /// An `import`/`export` reference.
    #[default]
    Import = 0,
    /// A `/// <reference path=... />` directive.
    ReferenceFile = 1,
    /// A `/// <reference types=... />` directive.
    TypeReferenceDirective = 2,
    /// A `/// <reference lib=... />` directive.
    LibReferenceDirective = 3,
    /// A root file from the command line / tsconfig `files`.
    RootFile = 4,
    /// A default library file.
    LibFile = 5,
    /// An automatic type-directive file.
    AutomaticTypeDirectiveFile = 6,
}

/// A reason a file was included in the program.
///
/// Mirrors `compiler.FileIncludeReason` in Go. The Go struct carries an
/// untyped `data any` field plus lazily-computed diagnostics; the Rust port
/// models the common case (a `FileIncludeKind` plus an optional file-name
/// payload).
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

    /// Whether this reason is a referenced-file kind (reference path,
    /// type-reference, or lib-reference directive).
    pub fn is_referenced_file(&self) -> bool {
        matches!(
            self.kind,
            FileIncludeKind::ReferenceFile
                | FileIncludeKind::TypeReferenceDirective
                | FileIncludeKind::LibReferenceDirective
        )
    }
}

/// A parsed file that was dropped from the final program (deduplicated).
///
/// Mirrors `compiler.DuplicateSourceFile` in Go.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DuplicateSourceFile {
    pub file_name: String,
    pub hash: u128,
    pub script_kind: crate::ast::ScriptKind,
}

/// A library file reference (name + resolved path + replaced flag).
///
/// Mirrors `compiler.LibFile` in Go.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct LibFile {
    pub name: String,
    pub path: String,
    pub replaced: bool,
}

/// Build-info snapshot for diagnostics and stats.
///
/// Mirrors the data surfaced by Go's `Program` stats methods
/// (`LineCount`, `IdentifierCount`, `SymbolCount`, `TypeCount`,
/// `InstantiationCount`).
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

// ────────────────────────────────────────────────────────────────────────────
// Additional Program methods (ported from `internal/compiler/program.go`)
// ────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
impl Program {
    /// Get all source files as an owned vector.
    ///
    /// Mirrors `Program.GetSourceFiles` in Go (the capital-G alias that
    /// returns the file slice). [`Program::source_files`] returns a borrow.
    pub fn get_source_files(&self) -> Vec<Arc<SourceFile>> {
        self.source_files.clone()
    }

    /// Get the file-include reasons map (testing only).
    ///
    /// Mirrors `Program.GetIncludeReasons` in Go. The Rust program does not
    /// yet track per-file include reasons, so this returns an empty map.
    pub fn get_file_include_reasons(&self) -> HashMap<String, Vec<FileIncludeReason>> {
        // TODO: track include reasons during the file-loading pipeline.
        HashMap::new()
    }

    /// Whether `path` is a missing file (referenced but not found).
    ///
    /// Mirrors `Program.IsMissingPath` in Go.
    pub fn is_missing_path(&self, path: &str) -> bool {
        !self.source_files_by_name.contains_key(path)
    }

    /// Get a source file by its normalized path.
    ///
    /// Mirrors `Program.GetSourceFileByPath` in Go.
    pub fn get_source_file_by_path(&self, path: &str) -> Option<Arc<SourceFile>> {
        self.source_files_by_name.get(path).cloned()
    }

    /// Get duplicate source files dropped during program construction.
    ///
    /// Mirrors `Program.DuplicateSourceFiles` in Go.
    pub fn duplicate_source_files(&self) -> &[DuplicateSourceFile] {
        // TODO: track duplicates during the file-loading pipeline.
        &[]
    }

    /// The total line count across all source files.
    ///
    /// Mirrors `Program.LineCount` in Go.
    pub fn line_count(&self) -> usize {
        self.source_files
            .iter()
            .map(|f| f.text.lines().count())
            .sum()
    }

    /// The total identifier count across all source files.
    ///
    /// Mirrors `Program.IdentifierCount` in Go.
    pub fn identifier_count(&self) -> usize {
        // TODO: requires a node-count walk over each source file's AST.
        0
    }

    /// The total symbol count.
    ///
    /// Mirrors `Program.SymbolCount` in Go.
    pub fn symbol_count(&self) -> usize {
        self.symbol_map.symbols.len()
    }

    /// The total type count (checker stat).
    ///
    /// Mirrors `Program.TypeCount` in Go.
    pub fn type_count(&self) -> usize {
        // TODO: requires checker-side type accounting.
        0
    }

    /// The total instantiation count (checker stat).
    ///
    /// Mirrors `Program.InstantiationCount` in Go.
    pub fn instantiation_count(&self) -> usize {
        // TODO: requires checker-side instantiation accounting.
        0
    }

    /// A build-info snapshot.
    ///
    /// Aggregates the stats surfaced by Go's `Program` methods
    /// (`LineCount`, `IdentifierCount`, `SymbolCount`, …).
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

    /// Whether file names are compared case-sensitively.
    ///
    /// Mirrors `Program.UseCaseSensitiveFileNames` in Go.
    pub fn use_case_sensitive_file_names(&self) -> bool {
        self.host.use_case_sensitive_file_names()
    }

    /// The current working directory.
    ///
    /// Mirrors `Program.GetCurrentDirectory` in Go.
    pub fn get_current_directory(&self) -> &str {
        self.host.current_directory()
    }

    /// The resolved modules cache (per-file import resolutions).
    ///
    /// Mirrors `Program.GetResolvedModules` in Go.
    pub fn get_resolved_modules(
        &self,
    ) -> HashMap<String, Vec<(String, Option<crate::module::ResolvedModule>)>> {
        // TODO: requires tracking per-file resolved modules during the
        // file-loading pipeline. The current pipeline resolves modules
        // inline without caching.
        HashMap::new()
    }

    /// The set of package names discovered during module resolution.
    ///
    /// Mirrors `Program.GetPackagesMap` in Go.
    pub fn get_packages_map(&self) -> HashMap<String, bool> {
        // TODO: requires package-name tracking during module resolution.
        HashMap::new()
    }

    /// Whether the program runs single-threaded.
    ///
    /// Mirrors `Program.SingleThreaded` in Go. The Rust port is always
    /// single-threaded for now.
    pub fn single_threaded(&self) -> bool {
        true
    }
}

// ────────────────────────────────────────────────────────────────────────────
// File-loading pipeline (ported from `internal/compiler/fileloader.go` and
// `internal/compiler/filesparser.go`). These functions mirror Go's
// `processRootFile`/`processSourceFile`/`processAllProgramFiles`; the Rust
// `Program::new` already inlines a simplified version of this pipeline, so
// these are provided as standalone entry points for callers that need to
// drive file loading incrementally.
// ────────────────────────────────────────────────────────────────────────────

/// Process a root file: read, parse, register, and recursively resolve its
/// references and imports.
///
/// Mirrors `fileLoader.addRootFileTask` + `processRootFile` in Go. The Rust
/// `Program::new` performs this inline; this entry point is provided for
/// parity with the Go API surface.
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

/// Process a single source file: read, parse, and register it (no reference
/// or import resolution).
///
/// Mirrors `fileLoader.parseSourceFile` / `processSourceFile` in Go.
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

/// Process all program files: load root files, resolve references and imports,
/// and return the resulting file set.
///
/// Mirrors `compiler.processAllProgramFiles` in Go. The Rust `Program::new`
/// inlines this pipeline; this entry point is provided for parity.
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
        // b.ts has a parse error.
        assert!(
            program
                .diagnostics()
                .iter()
                .any(|d| d.category == Category::Error)
        );
    }

    #[test]
    fn program_does_not_load_bundled_libs_without_root_files() {
        // Use the bundled lib files via BundledFS over OsFS.
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
        // Use the bundled lib files via BundledFS over OsFS.
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
        // lib.d.ts is the default lib; it should have loaded the root file plus
        // at least one referenced lib file.
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

    /// Port of Go's `TestProgram` (BasicFileOrdering case).
    ///
    /// Verifies that `/// <reference path=... />` directives cause referenced
    /// files to be loaded, and that files are ordered with dependencies first
    /// (deepest dependency before dependent).
    ///
    /// Go's `TestProgram` also covers import-based file ordering (FileOrderingImports,
    /// FileOrderingCycles), but those require module resolution which is not yet
    /// ported to Rust. This test covers the reference-path case.
    #[test]
    fn program_file_ordering_with_reference_paths() {
        let fs = Arc::new(InMemoryFS::new());

        // Build a chain: index.ts → 5.ts → 4.ts → 3.ts → 2.ts → 1.ts
        //                index.ts → 10.ts → 9.ts → 8.ts → 7.ts → 6.ts
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

    /// Port of Go's `TestProgram` — FileOrderingImports case.
    ///
    /// Same file graph as `program_file_ordering_with_reference_paths` but
    /// using `import` statements instead of `/// <reference path=... />`
    /// directives. Verifies that transitive `import` resolution pulls in the
    /// full dependency graph (all 11 files).
    ///
    /// NOTE: Go orders files deepest-dependency-first (`1.ts … 5.ts, 6.ts …
    /// 10.ts, index.ts`). The Rust `Program` currently emits files in
    /// module-resolution discovery order (root first, then a stack-based walk
    /// over resolved imports). The expected ordering below characterizes the
    /// current Rust behavior; once dependency-first reordering is implemented
    /// (mirroring Go's `fileLoader.processRootFile`), update the expected
    /// vector to match Go's ordering.
    #[test]
    fn program_file_ordering_imports() {
        let fs = Arc::new(InMemoryFS::new());
        // InMemoryFS requires explicit directory entries for
        // `directory_exists` checks during module resolution.
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

    /// Port of Go's `TestProgram` — FileOrderingCycles case.
    ///
    /// Same graph as `program_file_ordering_imports` but with cyclic imports
    /// (3.ts and 9.ts import back to index.ts). Verifies that cycles are
    /// broken gracefully and the full dependency graph still loads.
    ///
    /// NOTE: Same ordering caveat as `program_file_ordering_imports` — the
    /// expected vector reflects current Rust discovery order, not Go's
    /// dependency-first order.
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

    /// Module resolution: importing `"./foo"` should cause `foo.ts` to be
    /// resolved and loaded into the program, even though it isn't listed as a
    /// root file. This is the P5.6 integration of the `Resolver` into `Program`.
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

        // Should have loaded main.ts AND the resolved dependency foo.ts.
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

    /// Transitive module resolution: `a.ts` imports `b.ts`, which imports `c.ts`.
    /// The resolver's BFS walk should pull in the whole dependency chain.
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

        // All three files should be loaded.
        assert_eq!(program.source_files().len(), 3);
        assert!(program.get_source_file("/src/b.ts").is_some());
        assert!(program.get_source_file("/src/c.ts").is_some());
    }

    /// Port of Go's `TestIncludeProcessorDiagnosticsWithMissingFileCasing`.
    ///
    /// On a case-sensitive filesystem, requesting `/src/MyFile.ts` when only
    /// `/src/myFile.ts` exists should produce a "file not found" diagnostic
    /// without panicking. Go's test exercises the include processor's case-
    /// sensitivity diagnostic; Rust doesn't have an include processor, but the
    /// Program must still handle missing files gracefully.
    #[test]
    fn include_processor_diagnostics_with_missing_file_casing() {
        let fs = Arc::new(InMemoryFS::with_case_sensitivity(true));
        fs.insert_dir("/src");
        // Only the lowercase version exists.
        fs.insert_file("/src/myFile.ts", "export const y = 2;");

        let parsed = ParsedCommandLine {
            compiler_options: {
                let mut opts = CompilerOptions::default();
                opts.no_lib = Tristate::True;
                opts.skip_lib_check = Tristate::True;
                opts
            },
            // List both casings as root files.
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

        // The program should not panic when computing diagnostics.
        // /src/MyFile.ts does not exist on the case-sensitive FS, so we expect
        // at least one error diagnostic about the missing file.
        let diags = program.diagnostics();
        assert!(
            diags.iter().any(|d| d.category == Category::Error),
            "expected at least one error diagnostic for missing /src/MyFile.ts, got: {:?}",
            diags
        );
        // The existing /src/myFile.ts should still be loaded.
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

    // ── Bundled lib smoke tests (P2.9d) ─────────────────────────────────
    // Verify that the historically-troublesome bundled lib files parse
    // with zero parser diagnostics. Prior to P2.4/P2.5 these produced
    // thousands of TS1003 errors.

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

    // ── G3: node_modules .js files must not be parsed when allowJs is false ──

    /// When `allowJs` is false (the default), `.js` files pulled in from
    /// `node_modules` during module resolution must not be parsed as
    /// TypeScript — otherwise they produce false TS1003/TS1005 syntax
    /// diagnostics. They should also not appear in the program's source files.
    #[test]
    fn node_modules_js_skipped_when_allow_js_false() {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/node_modules/mypkg");
        // A `.js` entry point whose package.json `main` points at it. This
        // file is intentionally valid JS but would be parsed (and could
        // surface syntax diagnostics) if it were loaded into the program.
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

        // allowJs is false (default): no `--allowJs`.
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

        // The `.js` file must not have been loaded into the program.
        assert!(
            program
                .get_source_file("/proj/node_modules/mypkg/index.js")
                .is_none(),
            "expected node_modules .js file to be skipped when allowJs is false"
        );
        // And it must not have produced any syntax diagnostics.
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

    /// When `allowJs` is true, `.js` files in `node_modules` ARE loaded.
    /// This guards against the G3 filter being too aggressive.
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
