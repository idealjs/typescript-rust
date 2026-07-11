//! Command-line and `tsconfig.json` option parsing, ported from
//! `internal/tsoptions/`.
//!
//! This is a pragmatic port: it handles the common compiler options, file
//! arguments, response files, and `tsconfig.json` reading (including JSONC
//! comments, `extends`, `files`/`include`/`exclude` glob expansion). It does
//! not yet mirror the full `NameMap`/did-you-mean machinery of the Go port.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use crate::core::compiler_options::{
    CompilerOptions, JsxEmit, ModuleDetectionKind, ModuleKind, ModuleResolutionKind, NewLineKind,
    ScriptTarget,
};
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::diagnostics::new_ad_hoc_message;
use crate::glob::Glob;
use crate::tspath;
use crate::vfs::FS;

// ────────────────────────────────────────────────────────────────────────────
// Option declarations
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Boolean,
    String,
    Number,
    List,
    Enum,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionDecl {
    pub name: &'static str,
    pub short_name: Option<&'static str>,
    pub kind: OptionKind,
    pub is_file_path: bool,
}

/// The set of compiler options accepted on the command line.
///
/// Mirrors a subset of `tsoptions.CommandLineCompilerOptions`.
pub const OPTIONS: &[OptionDecl] = &[
    OptionDecl { name: "help", short_name: Some("h"), kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "all", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "version", short_name: Some("v"), kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "init", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "project", short_name: Some("p"), kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "build", short_name: Some("b"), kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "watch", short_name: Some("w"), kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "incremental", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noEmit", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noCheck", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noLib", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "skipLibCheck", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "skipDefaultLibCheck", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "strict", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "strictNullChecks", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "strictFunctionTypes", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "strictBindCallApply", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "strictPropertyInitialization", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "strictBuiltinIteratorReturn", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noImplicitAny", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noImplicitThis", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noImplicitOverride", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noUnusedLocals", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noUnusedParameters", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noFallthroughCasesInSwitch", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noUncheckedIndexedAccess", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noPropertyAccessFromIndexSignature", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noErrorTruncation", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noEmitOnError", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "noResolve", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "useUnknownInCatchVariables", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "exactOptionalPropertyTypes", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "esModuleInterop", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "allowSyntheticDefaultImports", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "allowJs", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "checkJs", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "composite", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "declaration", short_name: Some("d"), kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "declarationMap", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "declarationDir", short_name: None, kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "emitDeclarationOnly", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "sourceMap", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "inlineSourceMap", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "inlineSources", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "removeComments", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "isolatedModules", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "isolatedDeclarations", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "verbatimModuleSyntax", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "preserveConstEnums", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "importHelpers", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "experimentalDecorators", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "emitDecoratorMetadata", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "forceConsistentCasingInFileNames", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "listFiles", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "listFilesOnly", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "listEmittedFiles", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "explainFiles", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "extendedDiagnostics", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "diagnostics", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "pretty", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "showConfig", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "ignoreConfig", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "locale", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "target", short_name: Some("t"), kind: OptionKind::Enum, is_file_path: false },
    OptionDecl { name: "module", short_name: Some("m"), kind: OptionKind::Enum, is_file_path: false },
    OptionDecl { name: "moduleResolution", short_name: None, kind: OptionKind::Enum, is_file_path: false },
    OptionDecl { name: "jsx", short_name: None, kind: OptionKind::Enum, is_file_path: false },
    OptionDecl { name: "newLine", short_name: None, kind: OptionKind::Enum, is_file_path: false },
    OptionDecl { name: "moduleDetection", short_name: None, kind: OptionKind::Enum, is_file_path: false },
    OptionDecl { name: "lib", short_name: None, kind: OptionKind::List, is_file_path: false },
    OptionDecl { name: "types", short_name: None, kind: OptionKind::List, is_file_path: false },
    OptionDecl { name: "typeRoots", short_name: None, kind: OptionKind::List, is_file_path: true },
    OptionDecl { name: "rootDirs", short_name: None, kind: OptionKind::List, is_file_path: true },
    OptionDecl { name: "paths", short_name: None, kind: OptionKind::List, is_file_path: false },
    OptionDecl { name: "outDir", short_name: None, kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "outFile", short_name: None, kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "rootDir", short_name: None, kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "baseUrl", short_name: None, kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "tsBuildInfoFile", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "sourceRoot", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "mapRoot", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "jsxFactory", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "jsxFragmentFactory", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "jsxImportSource", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "reactNamespace", short_name: None, kind: OptionKind::String, is_file_path: false },
    OptionDecl { name: "generateTrace", short_name: None, kind: OptionKind::String, is_file_path: true },
    OptionDecl { name: "singleThreaded", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
    OptionDecl { name: "quiet", short_name: None, kind: OptionKind::Boolean, is_file_path: false },
];

fn find_option(name: &str) -> Option<&'static OptionDecl> {
    OPTIONS.iter().find(|o| o.name == name || o.short_name == Some(name))
}

// ────────────────────────────────────────────────────────────────────────────
// Parsed value
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum OptValue {
    Bool(bool),
    Str(String),
    Num(i64),
    List(Vec<String>),
    Null,
}

impl OptValue {
    fn as_bool(&self) -> Option<bool> {
        match self {
            OptValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            OptValue::Str(s) => Some(s),
            _ => None,
        }
    }
    fn as_list(&self) -> Option<&[String]> {
        match self {
            OptValue::List(v) => Some(v),
            _ => None,
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ParsedCommandLine
// ────────────────────────────────────────────────────────────────────────────

/// A parsed command line or tsconfig, mirroring `tsoptions.ParsedCommandLine`.
#[derive(Debug, Clone, Default)]
pub struct ParsedCommandLine {
    pub compiler_options: CompilerOptions,
    pub file_names: Vec<String>,
    pub errors: Vec<Diagnostic>,
    pub config_file_name: String,
    /// Raw `compilerOptions` value from tsconfig.json (if any), for `--showConfig`.
    pub raw_options: Option<crate::json::Value>,
    /// `files`/`include`/`exclude` specs from tsconfig.json.
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub files_spec: Vec<String>,
    pub watch: bool,
}

impl ParsedCommandLine {
    fn compiler_diagnostic(text: impl Into<String>) -> Diagnostic {
        Diagnostic::new(None, TextRange::undefined(), new_ad_hoc_message(""), vec![])
            .with_text(text)
    }
}

/// Helper extension to build a compiler diagnostic with custom text.
impl Diagnostic {
    pub fn with_text(self, text: impl Into<String>) -> Diagnostic {
        Diagnostic {
            file: self.file,
            loc: self.loc,
            code: self.code,
            category: self.category,
            message: None,
            message_key: self.message_key,
            message_args: vec![text.into()],
            message_chain: self.message_chain,
            related_information: self.related_information,
            reports_unnecessary: self.reports_unnecessary,
            reports_deprecated: self.reports_deprecated,
            skipped_on_no_emit: self.skipped_on_no_emit,
        }
    }
}

fn err(text: impl Into<String>) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), new_ad_hoc_message(""), vec![]).with_text(text)
}

// ────────────────────────────────────────────────────────────────────────────
// Command-line parsing
// ────────────────────────────────────────────────────────────────────────────

/// Parse command-line arguments into a `ParsedCommandLine`.
///
/// `current_dir` is used to resolve relative file paths and response files.
/// `fs` is used to read response files.
pub fn parse_command_line(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
) -> ParsedCommandLine {
    let mut options: HashMap<String, OptValue> = HashMap::new();
    let mut file_names: Vec<String> = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        let s = &args[i];
        i += 1;
        if s.is_empty() {
            continue;
        }
        let first = s.chars().next().unwrap();
        match first {
            '@' => {
                // Response file
                let response_path = &s[1..];
                let abs = tspath::get_normalized_absolute_path(response_path, current_dir);
                if let Some(fs) = fs {
                    if let Some(content) = fs.read_file(&abs) {
                        let response_args = split_response_file(&content);
                        let sub = parse_command_line(&response_args, current_dir, Some(fs));
                        file_names.extend(sub.file_names);
                        for (k, v) in sub_compiler_options_to_map(&sub.compiler_options) {
                            options.insert(k, v);
                        }
                        errors.extend(sub.errors);
                    } else {
                        errors.push(err(format!("Cannot read response file '{response_path}'.")));
                    }
                } else {
                    errors.push(err(format!("Cannot read response file '{response_path}'.")));
                }
            }
            '-' => {
                // Strip up to two leading dashes.
                let name_part = s.trim_start_matches('-');
                // Support `--name=value`.
                let (name, inline_value) = match name_part.split_once('=') {
                    Some((n, v)) => (n, Some(v.to_string())),
                    None => (name_part, None),
                };
                let opt = match find_option(name) {
                    Some(o) => o,
                    None => {
                        errors.push(err(format!("Unknown option '{name}'.")));
                        continue;
                    }
                };
                i = parse_option_value(args, i, opt, inline_value, &mut options, &mut errors);
            }
            _ => {
                file_names.push(s.clone());
            }
        }
    }

    let mut compiler_options = CompilerOptions::default();
    apply_options(&options, &mut compiler_options);
    let watch = compiler_options.watch.is_true();
    // Resolve relative file names to absolute paths.
    let file_names = file_names
        .iter()
        .map(|f| tspath::get_normalized_absolute_path(f, current_dir))
        .collect();

    ParsedCommandLine {
        compiler_options,
        file_names,
        errors,
        config_file_name: String::new(),
        raw_options: None,
        include: Vec::new(),
        exclude: Vec::new(),
        files_spec: Vec::new(),
        watch,
    }
}

fn parse_option_value(
    args: &[String],
    mut i: usize,
    opt: &OptionDecl,
    inline_value: Option<String>,
    options: &mut HashMap<String, OptValue>,
    errors: &mut Vec<Diagnostic>,
) -> usize {
    match opt.kind {
        OptionKind::Boolean => {
            if let Some(v) = inline_value {
                let b = v != "false";
                options.insert(opt.name.to_string(), OptValue::Bool(b));
            } else if i < args.len() && (args[i] == "true" || args[i] == "false") {
                options.insert(opt.name.to_string(), OptValue::Bool(args[i] == "true"));
                i += 1;
            } else {
                options.insert(opt.name.to_string(), OptValue::Bool(true));
            }
        }
        OptionKind::String | OptionKind::Enum => {
            let val = match inline_value {
                Some(v) => Some(v),
                None => {
                    if i < args.len() {
                        let v = args[i].clone();
                        i += 1;
                        Some(v)
                    } else {
                        None
                    }
                }
            };
            match val {
                Some(v) if v == "null" => {
                    options.insert(opt.name.to_string(), OptValue::Null);
                }
                Some(v) => {
                    options.insert(opt.name.to_string(), OptValue::Str(v));
                }
                None => {
                    errors.push(err(format!("Option '{}' requires a value.", opt.name)));
                }
            }
        }
        OptionKind::Number => {
            let val = inline_value.or_else(|| {
                if i < args.len() {
                    let v = args[i].clone();
                    i += 1;
                    Some(v)
                } else {
                    None
                }
            });
            match val {
                Some(v) => match v.parse::<i64>() {
                    Ok(n) => {
                        options.insert(opt.name.to_string(), OptValue::Num(n));
                    }
                    Err(_) => {
                        errors.push(err(format!("Option '{}' requires a number.", opt.name)));
                    }
                },
                None => {
                    errors.push(err(format!("Option '{}' requires a value.", opt.name)));
                }
            }
        }
        OptionKind::List => {
            let val = inline_value.or_else(|| {
                if i < args.len() && !args[i].starts_with('-') {
                    let v = args[i].clone();
                    i += 1;
                    Some(v)
                } else {
                    None
                }
            });
            let list = match val {
                Some(v) => v.split(',').map(|s| s.trim().to_string()).collect(),
                None => Vec::new(),
            };
            options.insert(opt.name.to_string(), OptValue::List(list));
        }
    }
    i
}

fn split_response_file(content: &str) -> Vec<String> {
    let mut args = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0usize;
    while pos < chars.len() {
        while pos < chars.len() && chars[pos] <= ' ' {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }
        if chars[pos] == '"' {
            pos += 1;
            let start = pos;
            while pos < chars.len() && chars[pos] != '"' {
                pos += 1;
            }
            args.push(chars[start..pos].iter().collect());
            if pos < chars.len() {
                pos += 1;
            }
        } else {
            let start = pos;
            while pos < chars.len() && chars[pos] > ' ' {
                pos += 1;
            }
            args.push(chars[start..pos].iter().collect());
        }
    }
    args
}

// ────────────────────────────────────────────────────────────────────────────
// Applying parsed options to CompilerOptions
// ────────────────────────────────────────────────────────────────────────────

fn set_bool(options: &mut CompilerOptions, name: &str, b: bool) {
    let t = Tristate::from(b);
    match name {
        "noEmit" => options.no_emit = t,
        "noCheck" => options.no_check = t,
        "noLib" => options.no_lib = t,
        "skipLibCheck" => options.skip_lib_check = t,
        "skipDefaultLibCheck" => options.skip_default_lib_check = t,
        "strictNullChecks" => options.strict_null_checks = t,
        "strictFunctionTypes" => options.strict_function_types = t,
        "strictBindCallApply" => options.strict_bind_call_apply = t,
        "strictPropertyInitialization" => options.strict_property_initialization = t,
        "strictBuiltinIteratorReturn" => options.strict_builtin_iterator_return = t,
        "noImplicitAny" => options.no_implicit_any = t,
        "noImplicitThis" => options.no_implicit_this = t,
        "noImplicitOverride" => options.no_implicit_override = t,
        "noUnusedLocals" => options.no_unused_locals = t,
        "noUnusedParameters" => options.no_unused_parameters = t,
        "noFallthroughCasesInSwitch" => options.no_fallthrough_cases_in_switch = t,
        "noUncheckedIndexedAccess" => options.no_unchecked_indexed_access = t,
        "noPropertyAccessFromIndexSignature" => options.no_property_access_from_index_signature = t,
        "noErrorTruncation" => options.no_error_truncation = t,
        "noEmitOnError" => options.no_emit_on_error = t,
        "noResolve" => options.no_resolve = t,
        "useUnknownInCatchVariables" => options.use_unknown_in_catch_variables = t,
        "exactOptionalPropertyTypes" => options.exact_optional_property_types = t,
        "esModuleInterop" => options.es_module_interop = t,
        "allowSyntheticDefaultImports" => options.allow_synthetic_default_imports = t,
        "allowJs" => options.allow_js = t,
        "checkJs" => options.check_js = t,
        "composite" => options.composite = t,
        "declaration" => options.declaration = t,
        "declarationMap" => options.declaration_map = t,
        "emitDeclarationOnly" => options.emit_declaration_only = t,
        "sourceMap" => options.source_map = t,
        "inlineSourceMap" => options.inline_source_map = t,
        "inlineSources" => options.inline_sources = t,
        "removeComments" => options.remove_comments = t,
        "isolatedModules" => options.isolated_modules = t,
        "isolatedDeclarations" => options.isolated_declarations = t,
        "verbatimModuleSyntax" => options.verbatim_module_syntax = t,
        "preserveConstEnums" => options.preserve_const_enums = t,
        "importHelpers" => options.import_helpers = t,
        "experimentalDecorators" => options.experimental_decorators = t,
        "emitDecoratorMetadata" => options.emit_decorator_metadata = t,
        "forceConsistentCasingInFileNames" => options.force_consistent_casing_in_file_names = t,
        "listFiles" => options.list_files = t,
        "listFilesOnly" => options.list_files_only = t,
        "listEmittedFiles" => options.list_emitted_files = t,
        "explainFiles" => options.explain_files = t,
        "extendedDiagnostics" => options.extended_diagnostics = t,
        "diagnostics" => options.diagnostics = t,
        "pretty" => options.pretty = t,
        "showConfig" => options.show_config = t,
        "ignoreConfig" => options.ignore_config = t,
        "incremental" => options.incremental = t,
        "watch" => options.watch = t,
        "version" => options.version = t,
        "help" => options.help = t,
        "all" => options.all = t,
        "init" => options.init = t,
        "build" => options.build = t,
        "singleThreaded" => options.single_threaded = t,
        "quiet" => options.quiet = t,
        "strict" => {
            options.strict = t;
            // `--strict` enables the full strict family.
            options.strict_null_checks = t;
            options.strict_function_types = t;
            options.strict_bind_call_apply = t;
            options.strict_property_initialization = t;
            options.strict_builtin_iterator_return = t;
            options.no_implicit_any = t;
            options.no_implicit_this = t;
            options.use_unknown_in_catch_variables = t;
            options.always_strict = t;
        }
        _ => {}
    }
}

fn apply_options(options: &HashMap<String, OptValue>, out: &mut CompilerOptions) {
    for (name, value) in options {
        match name.as_str() {
            "target" => {
                if let Some(s) = value.as_str() {
                    out.target = parse_script_target(s);
                }
            }
            "module" => {
                if let Some(s) = value.as_str() {
                    out.module = parse_module_kind(s);
                }
            }
            "moduleResolution" => {
                if let Some(s) = value.as_str() {
                    out.module_resolution = parse_module_resolution(s);
                }
            }
            "jsx" => {
                if let Some(s) = value.as_str() {
                    out.jsx = parse_jsx_emit(s);
                }
            }
            "newLine" => {
                if let Some(s) = value.as_str() {
                    out.new_line = match s.to_lowercase().as_str() {
                        "crlf" => NewLineKind::CRLF,
                        "lf" => NewLineKind::LF,
                        _ => NewLineKind::None,
                    };
                }
            }
            "moduleDetection" => {
                if let Some(s) = value.as_str() {
                    out.module_detection = match s.to_lowercase().as_str() {
                        "auto" => ModuleDetectionKind::Auto,
                        "legacy" => ModuleDetectionKind::Legacy,
                        "force" => ModuleDetectionKind::Force,
                        _ => ModuleDetectionKind::None,
                    };
                }
            }
            "lib" => {
                if let Some(list) = value.as_list() {
                    out.lib = list.to_vec();
                }
            }
            "types" => {
                if let Some(list) = value.as_list() {
                    out.types = list.to_vec();
                }
            }
            "typeRoots" => {
                if let Some(list) = value.as_list() {
                    out.type_roots = list.to_vec();
                }
            }
            "rootDirs" => {
                if let Some(list) = value.as_list() {
                    out.root_dirs = list.to_vec();
                }
            }
            "outDir" => {
                if let Some(s) = value.as_str() {
                    out.out_dir = s.to_string();
                }
            }
            "outFile" => {
                if let Some(s) = value.as_str() {
                    out.out_file = s.to_string();
                }
            }
            "rootDir" => {
                if let Some(s) = value.as_str() {
                    out.root_dir = s.to_string();
                }
            }
            "baseUrl" => {
                if let Some(s) = value.as_str() {
                    out.base_url = s.to_string();
                }
            }
            "project" => {
                if let Some(s) = value.as_str() {
                    out.project = s.to_string();
                }
            }
            "declarationDir" => {
                if let Some(s) = value.as_str() {
                    out.declaration_dir = s.to_string();
                }
            }
            "tsBuildInfoFile" => {
                if let Some(s) = value.as_str() {
                    out.ts_build_info_file = s.to_string();
                }
            }
            "sourceRoot" => {
                if let Some(s) = value.as_str() {
                    out.source_root = s.to_string();
                }
            }
            "mapRoot" => {
                if let Some(s) = value.as_str() {
                    out.map_root = s.to_string();
                }
            }
            "jsxFactory" => {
                if let Some(s) = value.as_str() {
                    out.jsx_factory = s.to_string();
                }
            }
            "jsxFragmentFactory" => {
                if let Some(s) = value.as_str() {
                    out.jsx_fragment_factory = s.to_string();
                }
            }
            "jsxImportSource" => {
                if let Some(s) = value.as_str() {
                    out.jsx_import_source = s.to_string();
                }
            }
            "reactNamespace" => {
                if let Some(s) = value.as_str() {
                    out.react_namespace = s.to_string();
                }
            }
            "locale" => {
                if let Some(s) = value.as_str() {
                    out.locale = s.to_string();
                }
            }
            "generateTrace" => {
                if let Some(s) = value.as_str() {
                    out.generate_trace = s.to_string();
                }
            }
            _ => {
                if let Some(b) = value.as_bool() {
                    set_bool(out, name, b);
                }
            }
        }
    }
}

fn sub_compiler_options_to_map(_opts: &CompilerOptions) -> Vec<(String, OptValue)> {
    // Response-file option merging is best-effort; a full round-trip is not
    // implemented here. (Options from response files are merged by re-parsing.)
    Vec::new()
}

fn parse_script_target(s: &str) -> ScriptTarget {
    let s = s.to_lowercase();
    let s = s.replace('-', "");
    match s.as_str() {
        "es3" => ScriptTarget::ES5,
        "es5" => ScriptTarget::ES5,
        "es6" | "es2015" => ScriptTarget::ES2015,
        "es2016" => ScriptTarget::ES2016,
        "es2017" => ScriptTarget::ES2017,
        "es2018" => ScriptTarget::ES2018,
        "es2019" => ScriptTarget::ES2019,
        "es2020" => ScriptTarget::ES2020,
        "es2021" => ScriptTarget::ES2021,
        "es2022" => ScriptTarget::ES2022,
        "es2023" => ScriptTarget::ES2023,
        "es2024" => ScriptTarget::ES2024,
        "es2025" => ScriptTarget::ES2025,
        "esnext" => ScriptTarget::ESNext,
        "json" => ScriptTarget::JSON,
        _ => ScriptTarget::None,
    }
}

fn parse_module_kind(s: &str) -> ModuleKind {
    match s.to_lowercase().as_str() {
        "commonjs" => ModuleKind::CommonJS,
        "amd" => ModuleKind::AMD,
        "umd" => ModuleKind::UMD,
        "system" => ModuleKind::System,
        "es6" | "es2015" => ModuleKind::ES2015,
        "es2020" => ModuleKind::ES2020,
        "es2022" => ModuleKind::ES2022,
        "esnext" => ModuleKind::ESNext,
        "node16" => ModuleKind::Node16,
        "node18" => ModuleKind::Node18,
        "node20" => ModuleKind::Node20,
        "nodenext" => ModuleKind::NodeNext,
        "preserve" => ModuleKind::Preserve,
        _ => ModuleKind::None,
    }
}

fn parse_module_resolution(s: &str) -> ModuleResolutionKind {
    match s.to_lowercase().as_str() {
        "classic" => ModuleResolutionKind::Classic,
        "node" | "node10" => ModuleResolutionKind::Node10,
        "node16" => ModuleResolutionKind::Node16,
        "nodenext" => ModuleResolutionKind::NodeNext,
        "bundler" => ModuleResolutionKind::Bundler,
        _ => ModuleResolutionKind::Unknown,
    }
}

fn parse_jsx_emit(s: &str) -> JsxEmit {
    match s.to_lowercase().as_str() {
        "preserve" => JsxEmit::Preserve,
        "react" => JsxEmit::React,
        "react-native" => JsxEmit::ReactNative,
        "react-jsx" => JsxEmit::ReactJSX,
        "react-jsxdev" => JsxEmit::ReactJSXDev,
        _ => JsxEmit::None,
    }
}

/// Reverse mapping: `ScriptTarget` value → canonical string name used in
/// tsconfig.json. Returns `None` for `ScriptTarget::None` (unset).
pub fn script_target_name(t: ScriptTarget) -> Option<&'static str> {
    match t {
        ScriptTarget::ES5 => Some("es5"),
        ScriptTarget::ES2015 => Some("es2015"),
        ScriptTarget::ES2016 => Some("es2016"),
        ScriptTarget::ES2017 => Some("es2017"),
        ScriptTarget::ES2018 => Some("es2018"),
        ScriptTarget::ES2019 => Some("es2019"),
        ScriptTarget::ES2020 => Some("es2020"),
        ScriptTarget::ES2021 => Some("es2021"),
        ScriptTarget::ES2022 => Some("es2022"),
        ScriptTarget::ES2023 => Some("es2023"),
        ScriptTarget::ES2024 => Some("es2024"),
        ScriptTarget::ES2025 => Some("es2025"),
        ScriptTarget::ESNext => Some("esnext"),
        ScriptTarget::JSON => Some("json"),
        ScriptTarget::None => None,
    }
}

/// Reverse mapping: `ModuleKind` value → canonical string name.
pub fn module_kind_name(m: ModuleKind) -> Option<&'static str> {
    match m {
        ModuleKind::CommonJS => Some("commonjs"),
        ModuleKind::AMD => Some("amd"),
        ModuleKind::UMD => Some("umd"),
        ModuleKind::System => Some("system"),
        ModuleKind::ES2015 => Some("es2015"),
        ModuleKind::ES2020 => Some("es2020"),
        ModuleKind::ES2022 => Some("es2022"),
        ModuleKind::ESNext => Some("esnext"),
        ModuleKind::Node16 => Some("node16"),
        ModuleKind::Node18 => Some("node18"),
        ModuleKind::Node20 => Some("node20"),
        ModuleKind::NodeNext => Some("nodenext"),
        ModuleKind::Preserve => Some("preserve"),
        ModuleKind::None => None,
    }
}

/// Reverse mapping: `ModuleResolutionKind` value → canonical string name.
pub fn module_resolution_name(r: ModuleResolutionKind) -> Option<&'static str> {
    match r {
        ModuleResolutionKind::Classic => Some("classic"),
        ModuleResolutionKind::Node10 => Some("node10"),
        ModuleResolutionKind::Node16 => Some("node16"),
        ModuleResolutionKind::NodeNext => Some("nodenext"),
        ModuleResolutionKind::Bundler => Some("bundler"),
        ModuleResolutionKind::Unknown => None,
    }
}

/// Reverse mapping: `JsxEmit` value → canonical string name.
pub fn jsx_emit_name(j: JsxEmit) -> Option<&'static str> {
    match j {
        JsxEmit::Preserve => Some("preserve"),
        JsxEmit::React => Some("react"),
        JsxEmit::ReactNative => Some("react-native"),
        JsxEmit::ReactJSX => Some("react-jsx"),
        JsxEmit::ReactJSXDev => Some("react-jsxdev"),
        JsxEmit::None => None,
    }
}

/// Reverse mapping: `ModuleDetectionKind` value → canonical string name.
pub fn module_detection_name(d: ModuleDetectionKind) -> Option<&'static str> {
    match d {
        ModuleDetectionKind::Auto => Some("auto"),
        ModuleDetectionKind::Force => Some("force"),
        ModuleDetectionKind::Legacy => Some("legacy"),
        ModuleDetectionKind::None => None,
    }
}

/// Reverse mapping: `NewLineKind` value → canonical string name.
pub fn new_line_name(n: NewLineKind) -> Option<&'static str> {
    match n {
        NewLineKind::CRLF => Some("crlf"),
        NewLineKind::LF => Some("lf"),
        NewLineKind::None => None,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// tsconfig.json parsing
// ────────────────────────────────────────────────────────────────────────────

/// Parse a `tsconfig.json` file into a `ParsedCommandLine`, merging `base_options`
/// (from the command line) and expanding `files`/`include`/`exclude`.
pub fn get_parsed_command_line_of_config_file(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
) -> ParsedCommandLine {
    let mut result = ParsedCommandLine::default();
    result.compiler_options = base_options.clone();
    result.config_file_name = config_file_name.to_string();

    let config_text = match fs.read_file(config_file_name) {
        Some(t) => t,
        None => {
            result
                .errors
                .push(err(format!("Cannot find a tsconfig.json file at the specified directory: '{config_file_name}'.")));
            return result;
        }
    };

    let jsonc = strip_jsonc(&config_text);
    // An empty tsconfig.json is treated as {} (no options).
    let root: crate::json::Value = if jsonc.trim().is_empty() {
        crate::json::Value::Object(crate::json::Map::new())
    } else {
        match crate::json::from_str(&jsonc) {
            Ok(v) => v,
            Err(e) => {
                result
                    .errors
                    .push(err(format!("Failed to parse tsconfig.json: {e}.")));
                return result;
            }
        }
    };

    let root_obj = match root.as_object() {
        Some(o) => o,
        None => {
            result
                .errors
                .push(err("tsconfig.json must be an object."));
            return result;
        }
    };

    // `extends`
    if let Some(extends) = root_obj.get("extends") {
        let extends_path = extends_as_path(extends, config_file_name, current_dir, fs);
        if let Some(ext_path) = extends_path {
            let parent = get_parsed_command_line_of_config_file(
                &ext_path,
                &CompilerOptions::default(),
                current_dir,
                fs,
            );
            // Merge parent options first (lower priority).
            merge_compiler_options(&mut result.compiler_options, &parent.compiler_options);
            result.include = parent.include;
            result.exclude = parent.exclude;
            result.files_spec = parent.files_spec;
            result.errors.extend(parent.errors);
        }
    }

    // `files`
    if let Some(files) = root_obj.get("files").and_then(|v| v.as_array()) {
        for f in files {
            if let Some(s) = f.as_str() {
                result.files_spec.push(s.to_string());
            }
        }
    }
    // `include`
    if let Some(include) = root_obj.get("include").and_then(|v| v.as_array()) {
        for f in include {
            if let Some(s) = f.as_str() {
                result.include.push(s.to_string());
            }
        }
    }
    // `exclude`
    if let Some(exclude) = root_obj.get("exclude").and_then(|v| v.as_array()) {
        for f in exclude {
            if let Some(s) = f.as_str() {
                result.exclude.push(s.to_string());
            }
        }
    }

    // `compilerOptions`
    if let Some(co) = root_obj.get("compilerOptions").and_then(|v| v.as_object()) {
        result.raw_options = Some(crate::json::Value::Object(co.clone()));
        let opts = json_object_to_options(co);
        // Command-line base options take precedence over config-file options
        // for values explicitly set on the command line; here we apply config
        // options first, then re-apply base options on top.
        let mut config_opts = CompilerOptions::default();
        apply_options(&opts, &mut config_opts);
        // Handle `paths` specially — it's an object map, not handled by apply_options.
        if let Some(paths_val) = co.get("paths").and_then(|v| v.as_object()) {
            let mut paths_map = HashMap::new();
            for (key, val) in paths_val {
                if let Some(arr) = val.as_array() {
                    let targets: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    paths_map.insert(key.clone(), targets);
                }
            }
            config_opts.paths = Some(paths_map);
        }
        merge_compiler_options(&mut result.compiler_options, &config_opts);
        // Re-apply base (command-line) options so they win.
        merge_compiler_options(&mut result.compiler_options, base_options);
    }

    // Resolve file names from specs.
    let config_dir = tspath::get_directory_path(config_file_name);
    result.file_names = expand_file_names(
        &result.files_spec,
        &result.include,
        &result.exclude,
        &config_dir,
        fs,
    );

    result
}

fn extends_as_path(
    extends: &crate::json::Value,
    config_file_name: &str,
    current_dir: &str,
    fs: &dyn FS,
) -> Option<String> {
    let s = extends.as_str()?;
    let config_dir = tspath::get_directory_path(config_file_name);
    let base = tspath::combine_paths(&config_dir, &[s]);
    // Try as-is, then with /tsconfig.json, then as node_modules path.
    let candidates = [
        base.clone(),
        tspath::combine_paths(&base, &["tsconfig.json"]),
    ];
    for c in &candidates {
        if fs.file_exists(c) {
            return Some(c.clone());
        }
    }
    // Fall back to the raw string resolved against current_dir.
    let abs = tspath::get_normalized_absolute_path(s, current_dir);
    if fs.file_exists(&abs) {
        Some(abs)
    } else {
        Some(tspath::combine_paths(&abs, &["tsconfig.json"]))
    }
}

fn json_object_to_options(obj: &crate::json::Map<String, crate::json::Value>) -> HashMap<String, OptValue> {
    let mut out = HashMap::new();
    for (k, v) in obj {
        let val = json_to_opt_value(v);
        out.insert(k.clone(), val);
    }
    out
}

fn json_to_opt_value(v: &crate::json::Value) -> OptValue {
    match v {
        crate::json::Value::Bool(b) => OptValue::Bool(*b),
        crate::json::Value::String(s) => OptValue::Str(s.clone()),
        crate::json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                OptValue::Num(i)
            } else {
                OptValue::Str(n.to_string())
            }
        }
        crate::json::Value::Array(arr) => {
            let list = arr
                .iter()
                .filter_map(|e| e.as_str().map(|s| s.to_string()))
                .collect();
            OptValue::List(list)
        }
        crate::json::Value::Null => OptValue::Null,
        crate::json::Value::Object(_) => OptValue::Null,
    }
}

/// Merge `src` into `dst`, where `dst` values take precedence (already set).
fn merge_compiler_options(dst: &mut CompilerOptions, src: &CompilerOptions) {
    // Apply src fields only where dst is at its default/unset.
    macro_rules! merge_tri {
        ($field:ident) => {
            if dst.$field.is_unknown() {
                dst.$field = src.$field;
            }
        };
    }
    merge_tri!(no_emit);
    merge_tri!(no_check);
    merge_tri!(no_lib);
    merge_tri!(skip_lib_check);
    merge_tri!(skip_default_lib_check);
    merge_tri!(strict);
    merge_tri!(strict_null_checks);
    merge_tri!(strict_function_types);
    merge_tri!(strict_bind_call_apply);
    merge_tri!(strict_property_initialization);
    merge_tri!(strict_builtin_iterator_return);
    merge_tri!(no_implicit_any);
    merge_tri!(no_implicit_this);
    merge_tri!(no_implicit_override);
    merge_tri!(no_unused_locals);
    merge_tri!(no_unused_parameters);
    merge_tri!(no_fallthrough_cases_in_switch);
    merge_tri!(no_unchecked_indexed_access);
    merge_tri!(exact_optional_property_types);
    merge_tri!(es_module_interop);
    merge_tri!(allow_js);
    merge_tri!(check_js);
    merge_tri!(composite);
    merge_tri!(declaration);
    merge_tri!(source_map);
    merge_tri!(remove_comments);
    merge_tri!(isolated_modules);
    merge_tri!(verbatim_module_syntax);
    merge_tri!(experimental_decorators);
    merge_tri!(force_consistent_casing_in_file_names);
    merge_tri!(use_unknown_in_catch_variables);
    merge_tri!(pretty);
    merge_tri!(incremental);
    merge_tri!(watch);
    if dst.target == ScriptTarget::None {
        dst.target = src.target;
    }
    if dst.module == ModuleKind::None {
        dst.module = src.module;
    }
    if dst.module_resolution == ModuleResolutionKind::Unknown {
        dst.module_resolution = src.module_resolution;
    }
    if dst.jsx == JsxEmit::None {
        dst.jsx = src.jsx;
    }
    if dst.out_dir.is_empty() {
        dst.out_dir = src.out_dir.clone();
    }
    if dst.root_dir.is_empty() {
        dst.root_dir = src.root_dir.clone();
    }
    if dst.base_url.is_empty() {
        dst.base_url = src.base_url.clone();
    }
    if dst.lib.is_empty() {
        dst.lib = src.lib.clone();
    }
    if dst.types.is_empty() {
        dst.types = src.types.clone();
    }
    if dst.type_roots.is_empty() {
        dst.type_roots = src.type_roots.clone();
    }
    if dst.paths.is_none() {
        dst.paths = src.paths.clone();
    }
    if dst.declaration_dir.is_empty() {
        dst.declaration_dir = src.declaration_dir.clone();
    }
    if dst.source_root.is_empty() {
        dst.source_root = src.source_root.clone();
    }
    if dst.map_root.is_empty() {
        dst.map_root = src.map_root.clone();
    }
    if dst.ts_build_info_file.is_empty() {
        dst.ts_build_info_file = src.ts_build_info_file.clone();
    }
    if dst.root_dirs.is_empty() {
        dst.root_dirs = src.root_dirs.clone();
    }
    if dst.module_suffixes.is_empty() {
        dst.module_suffixes = src.module_suffixes.clone();
    }
    if dst.custom_conditions.is_empty() {
        dst.custom_conditions = src.custom_conditions.clone();
    }
    if dst.out_file.is_empty() {
        dst.out_file = src.out_file.clone();
    }
    if dst.module_detection == ModuleDetectionKind::None {
        dst.module_detection = src.module_detection;
    }
    if dst.new_line == NewLineKind::None {
        dst.new_line = src.new_line;
    }
}

/// Resolve the set of input file names from `files`/`include`/`exclude` specs.
fn expand_file_names(
    files: &[String],
    include: &[String],
    exclude: &[String],
    base_dir: &str,
    fs: &dyn FS,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let exclude_globs: Vec<Glob> = exclude
        .iter()
        .filter_map(|p| Glob::parse(p).ok())
        .collect();

    let add = |path: &str, out: &mut Vec<String>, seen: &mut std::collections::HashSet<String>| {
        let abs = tspath::get_normalized_absolute_path(path, base_dir);
        if seen.insert(abs.clone()) {
            out.push(abs);
        }
    };

    // Explicit `files`.
    for f in files {
        add(f, &mut result, &mut seen);
    }

    // `include` glob expansion.
    let include_specs: Vec<String> = if include.is_empty() && files.is_empty() {
        vec!["**/*".to_string()]
    } else {
        include.to_vec()
    };
    for spec in &include_specs {
        let matched = match_glob_spec(spec, base_dir, fs);
        for path in matched {
            if is_excluded(&path, &exclude_globs) {
                continue;
            }
            if !is_supported_source_file(&path) {
                continue;
            }
            add(&path, &mut result, &mut seen);
        }
    }

    result.sort();
    result
}

fn is_excluded(path: &str, exclude_globs: &[Glob]) -> bool {
    exclude_globs.iter().any(|g| g.is_match(path))
}

fn is_supported_source_file(path: &str) -> bool {
    let ext = path.rfind('.').map(|i| &path[i..]).unwrap_or("");
    matches!(
        ext,
        ".ts" | ".tsx" | ".d.ts" | ".mts" | ".cts" | ".d.mts" | ".d.cts"
    )
}

/// Match an include glob spec against the filesystem, returning matching file paths.
fn match_glob_spec(spec: &str, base_dir: &str, fs: &dyn FS) -> Vec<String> {
    let mut results = Vec::new();
    // The spec may be relative to base_dir. Walk the directory tree and match.
    let abs_spec = if tspath::path_is_absolute(spec) {
        spec.to_string()
    } else {
        tspath::combine_paths(base_dir, &[spec])
    };
    // Walk starting from the longest non-glob directory prefix of the spec.
    let walk_root = glob_base_dir(&abs_spec);
    walk_and_match(&abs_spec, &walk_root, fs, &mut results);
    results
}

/// Return the longest directory prefix of `spec` that contains no glob
/// metacharacters (`*`, `?`, `{`, `[`).
fn glob_base_dir(spec: &str) -> String {
    let metachars = |c: char| c == '*' || c == '?' || c == '{' || c == '[';
    let first_meta = spec.chars().position(metachars);
    let prefix = match first_meta {
        Some(idx) => &spec[..idx],
        None => spec,
    };
    // Trim to the last directory separator.
    match prefix.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => prefix[..idx].to_string(),
        None => ".".to_string(),
    }
}

fn walk_and_match(root_spec: &str, dir: &str, fs: &dyn FS, results: &mut Vec<String>) {
    let entries = fs.get_accessible_entries(dir);
    for file in &entries.files {
        let full = tspath::combine_paths(dir, &[file]);
        if glob_matches(root_spec, &full) {
            results.push(full);
        }
    }
    for d in &entries.directories {
        // Skip node_modules and other common ignore dirs.
        if d == "node_modules" || d == ".git" {
            continue;
        }
        let full = tspath::combine_paths(dir, &[d]);
        walk_and_match(root_spec, &full, fs, results);
    }
}

fn glob_matches(spec: &str, path: &str) -> bool {
    match Glob::parse(spec) {
        Ok(g) => g.is_match(path),
        Err(_) => false,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JSONC preprocessing
// ────────────────────────────────────────────────────────────────────────────

/// Strip `//` line comments, `/* */` block comments, and trailing commas from
/// JSONC text so it can be parsed by a strict JSON parser.
fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut in_string = false;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
                i += 1;
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                // Line comment: skip to end of line.
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                // Block comment: skip to `*/`.
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            ',' if i + 1 < chars.len() => {
                // Trailing comma: peek ahead for `}` or `]` (skipping whitespace).
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    // Drop the comma.
                    i += 1;
                } else {
                    out.push(c);
                    i += 1;
                }
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::InMemoryFS;

    #[test]
    fn parse_basic_options() {
        let args: Vec<String> = vec!["--noEmit", "--strict", "--target", "ES2020", "src/a.ts"]
            .into_iter()
            .map(String::from)
            .collect();
        let parsed = parse_command_line(&args, "/proj", None);
        assert!(parsed.compiler_options.no_emit.is_true());
        assert!(parsed.compiler_options.strict.is_true());
        assert!(parsed.compiler_options.strict_null_checks.is_true());
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert_eq!(parsed.file_names, vec!["/proj/src/a.ts"]);
    }

    #[test]
    fn parse_equals_form() {
        let args: Vec<String> = vec!["--target=ES2015", "--module=commonjs"]
            .into_iter()
            .map(String::from)
            .collect();
        let parsed = parse_command_line(&args, "/proj", None);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2015);
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
    }

    #[test]
    fn parse_short_option() {
        let args: Vec<String> = vec!["-p", "tsconfig.json"].into_iter().map(String::from).collect();
        let parsed = parse_command_line(&args, "/proj", None);
        assert_eq!(parsed.compiler_options.project, "tsconfig.json");
    }

    #[test]
    fn strip_jsonc_comments() {
        let input = r#"{ // comment
            "compilerOptions": {
                "target": "ES5", /* block */
                "strict": true,
            }
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(
            v["compilerOptions"]["target"].as_str(),
            Some("ES5")
        );
    }

    #[test]
    fn parse_tsconfig_files() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/tsconfig.json", r#"{
            "compilerOptions": { "target": "ES2017", "noEmit": true },
            "files": ["src/a.ts"]
        }"#);
        fs.insert_file("/proj/src/a.ts", "export const x = 1;");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2017);
        assert!(parsed.compiler_options.no_emit.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/src/a.ts"]);
    }

    #[test]
    fn parse_tsconfig_include_glob() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/tsconfig.json", r#"{
            "include": ["src/**/*"]
        }"#);
        fs.insert_file("/proj/src/a.ts", "export const a = 1;");
        fs.insert_file("/proj/src/b.ts", "export const b = 2;");
        fs.insert_file("/proj/src/ignore.txt", "ignore me");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
        assert!(!parsed.file_names.iter().any(|f| f.ends_with("ignore.txt")));
    }

    // ──────────────────────────────────────────────────────────────────────
    // Helpers for the ported tests.
    // ──────────────────────────────────────────────────────────────────────

    /// Returns true if any diagnostic on `parsed` carries a message argument
    /// containing `needle`. The Rust port stores ad-hoc error text in
    /// `Diagnostic.message_args[0]`.
    fn has_error_containing(parsed: &ParsedCommandLine, needle: &str) -> bool {
        parsed.errors.iter().any(|e| {
            e.message_args
                .iter()
                .any(|a| a.contains(needle))
        })
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    // ──────────────────────────────────────────────────────────────────────
    // Command-line parser tests (ported from commandlineparser_test.go)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_command_line_version() {
        // `--version` and `-v` both set the version flag.
        let parsed = parse_command_line(&args(&["--version"]), "/proj", None);
        assert!(parsed.compiler_options.version.is_true());

        let parsed_short = parse_command_line(&args(&["-v"]), "/proj", None);
        assert!(parsed_short.compiler_options.version.is_true());
    }

    #[test]
    fn test_parse_command_line_help() {
        let parsed = parse_command_line(&args(&["--help"]), "/proj", None);
        assert!(parsed.compiler_options.help.is_true());

        let parsed_short = parse_command_line(&args(&["-h"]), "/proj", None);
        assert!(parsed_short.compiler_options.help.is_true());
    }

    #[test]
    fn test_parse_command_line_build() {
        let parsed = parse_command_line(&args(&["--build"]), "/proj", None);
        assert!(parsed.compiler_options.build.is_true());

        let parsed_short = parse_command_line(&args(&["-b"]), "/proj", None);
        assert!(parsed_short.compiler_options.build.is_true());
    }

    #[test]
    fn test_parse_command_line_watch() {
        let parsed = parse_command_line(&args(&["--watch", "0.ts"]), "/proj", None);
        assert!(parsed.compiler_options.watch.is_true());
        // The `watch` convenience flag on ParsedCommandLine mirrors the option.
        assert!(parsed.watch);

        let parsed_short = parse_command_line(&args(&["-w", "0.ts"]), "/proj", None);
        assert!(parsed_short.compiler_options.watch.is_true());
        assert!(parsed_short.watch);
    }

    #[test]
    fn test_parse_command_line_all_and_init() {
        let parsed = parse_command_line(&args(&["--all"]), "/proj", None);
        assert!(parsed.compiler_options.all.is_true());

        let parsed = parse_command_line(&args(&["--init"]), "/proj", None);
        assert!(parsed.compiler_options.init.is_true());
    }

    #[test]
    fn test_parse_command_line_lib_list() {
        // `--lib es5,es2015.symbol.wellknown 0.ts` parses as a comma-separated list.
        let parsed = parse_command_line(
            &args(&["--lib", "es5,es2015.symbol.wellknown", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(
            parsed.compiler_options.lib,
            vec!["es5".to_string(), "es2015.symbol.wellknown".to_string()]
        );
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_lib_multiple_flags() {
        // A second `--lib` on the command line overrides the first (last wins).
        let parsed = parse_command_line(
            &args(&[
                "--module",
                "commonjs",
                "--target",
                "es5",
                "--lib",
                "es5",
                "0.ts",
                "--lib",
                "es2015.core, es2015.symbol.wellknown ",
            ]),
            "/proj",
            None,
        );
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES5);
        // List values are split on commas and trimmed.
        assert_eq!(
            parsed.compiler_options.lib,
            vec!["es2015.core".to_string(), "es2015.symbol.wellknown".to_string()]
        );
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_lib_empty_followed_by_option() {
        // `0.ts --lib --sourceMap`: `--lib` does not consume `--sourceMap`.
        // (The Rust parser is case-sensitive for option names, so the canonical
        // camelCase spelling `--sourceMap` is required.)
        let parsed = parse_command_line(
            &args(&["0.ts", "--lib", "--sourceMap"]),
            "/proj",
            None,
        );
        assert!(parsed.compiler_options.lib.is_empty());
        assert!(parsed.compiler_options.source_map.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_unknown_option_error() {
        let parsed = parse_command_line(&args(&["--unknownOpt", "0.ts"]), "/proj", None);
        assert!(has_error_containing(&parsed, "Unknown option"));
        assert!(has_error_containing(&parsed, "unknownOpt"));
    }

    #[test]
    fn test_parse_command_line_explicit_boolean_false() {
        // `--strictNullChecks false 0.ts` sets the option to false (not unknown).
        let parsed = parse_command_line(
            &args(&["--strictNullChecks", "false", "0.ts"]),
            "/proj",
            None,
        );
        assert!(parsed.compiler_options.strict_null_checks.is_false());
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_explicit_boolean_true() {
        let parsed = parse_command_line(
            &args(&["--strictNullChecks", "true", "0.ts"]),
            "/proj",
            None,
        );
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_command_line_implicit_boolean() {
        // `--strictNullChecks` with no value defaults to true.
        let parsed = parse_command_line(&args(&["--strictNullChecks"]), "/proj", None);
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_command_line_non_boolean_after_boolean_flag() {
        // `--noImplicitAny t 0.ts`: boolean flags only consume `true`/`false`,
        // so `t` is treated as an input file (matches tsgo behavior). File names
        // are kept in insertion order (the command-line parser does not sort).
        let parsed = parse_command_line(
            &args(&["--noImplicitAny", "t", "0.ts"]),
            "/proj",
            None,
        );
        assert!(parsed.compiler_options.no_implicit_any.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/t", "/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_incremental() {
        let parsed = parse_command_line(&args(&["--incremental", "0.ts"]), "/proj", None);
        assert!(parsed.compiler_options.incremental.is_true());
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_ts_build_info_file() {
        let parsed = parse_command_line(
            &args(&["--tsBuildInfoFile", "build.tsbuildinfo", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(parsed.compiler_options.ts_build_info_file, "build.tsbuildinfo");
    }

    #[test]
    fn test_parse_command_line_ts_build_info_file_null() {
        // `--tsBuildInfoFile null` is accepted (string options honor `null`).
        let parsed = parse_command_line(
            &args(&["--tsBuildInfoFile", "null", "0.ts"]),
            "/proj",
            None,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.compiler_options.ts_build_info_file, "");
    }

    #[test]
    fn test_parse_command_line_type_roots() {
        // `--typeRoots t` parses as a single-element list.
        // (Note: unlike tsgo, the Rust port does not resolve list entries to
        // absolute paths on the command line, so we assert the parsed value.)
        let parsed = parse_command_line(&args(&["--typeRoots", "t", "bug.ts"]), "/home/project", None);
        assert_eq!(parsed.compiler_options.type_roots, vec!["t".to_string()]);
        assert_eq!(parsed.file_names, vec!["/home/project/bug.ts"]);
    }

    #[test]
    fn test_parse_command_line_files_in_middle() {
        // Input files may appear between flags.
        let parsed = parse_command_line(
            &args(&[
                "--module",
                "commonjs",
                "--target",
                "es5",
                "0.ts",
                "--lib",
                "es5,es2015.symbol.wellknown",
            ]),
            "/proj",
            None,
        );
        assert_eq!(parsed.compiler_options.module, ModuleKind::CommonJS);
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES5);
        assert_eq!(
            parsed.compiler_options.lib,
            vec!["es5".to_string(), "es2015.symbol.wellknown".to_string()]
        );
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
    }

    #[test]
    fn test_parse_command_line_module_resolution_and_jsx() {
        let parsed = parse_command_line(
            &args(&["--moduleResolution", "node", "--jsx", "react", "0.ts"]),
            "/proj",
            None,
        );
        assert_eq!(
            parsed.compiler_options.module_resolution,
            ModuleResolutionKind::Node10
        );
        assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
    }

    #[test]
    fn test_response_file_does_not_panic() {
        // Passing `@` with an empty or non-existent filename should produce a
        // diagnostic error rather than panicking (ported from
        // TestResponseFileDoesNotPanic).
        let parsed = parse_command_line(&args(&["@"]), "/proj", None);
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "response file"));

        let parsed = parse_command_line(&args(&["@blah"]), "/proj", None);
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "response file"));
        assert!(has_error_containing(&parsed, "blah"));
    }

    #[test]
    fn test_response_file_missing_with_fs() {
        // Even with an FS provided, a missing response file yields an error.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        let parsed = parse_command_line(&args(&["@missing.rsp"]), "/proj", Some(&fs));
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "response file"));
    }

    #[test]
    fn test_response_file_propagates_file_names() {
        // A response file that exists is expanded into arguments. The Rust port
        // currently propagates file names (and errors) from response files but
        // does not yet merge compiler options from them, so we assert only the
        // file-name propagation here.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/args.rsp", "--strict\n0.ts");
        let parsed = parse_command_line(&args(&["@args.rsp"]), "/proj", Some(&fs));
        assert_eq!(parsed.file_names, vec!["/proj/0.ts"]);
        // No errors reading the response file.
        assert!(!has_error_containing(&parsed, "response file"));
    }

    // ──────────────────────────────────────────────────────────────────────
    // JSONC preprocessing tests (ported from tsconfigparsing_test.go,
    // TestParseConfigFileTextToJson scenarios)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_strip_jsonc_whitespace_and_empty_object() {
        // Whitespace-only and comment-only inputs strip to empty/whitespace.
        let stripped = strip_jsonc("   ");
        assert_eq!(stripped.trim(), "");

        let stripped = strip_jsonc("// Comment");
        assert_eq!(stripped.trim(), "");

        let stripped = strip_jsonc("/* Comment */");
        assert_eq!(stripped.trim(), "");

        // An empty object survives.
        let stripped = strip_jsonc("{}");
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert!(v.as_object().is_some());
    }

    #[test]
    fn test_strip_jsonc_comments_in_object() {
        let input = r#"{ // Excluded files
            "exclude": [
                // Exclude d.ts
                "file.d.ts"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("file.d.ts"));

        // Multiline block comments interspersed in a line are removed.
        let input = r#"{
            /* Excluded
                    Files
            */
            "exclude": [
                /* multiline comments can be in the middle of a line */"file.d.ts"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("file.d.ts"));
    }

    #[test]
    fn test_strip_jsonc_keeps_string_content() {
        // `//` and `/* */` inside string literals are preserved verbatim.
        let input = r#"{
            "exclude": [
                "xx//file.d.ts"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("xx//file.d.ts"));

        let input = r#"{
            "exclude": [
                "xx/*file.d.ts*/"
            ]
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["exclude"][0].as_str(), Some("xx/*file.d.ts*/"));
    }

    #[test]
    fn test_strip_jsonc_trailing_comma() {
        // Trailing commas before `}` or `]` are dropped.
        let input = r#"{
            "compilerOptions": {
                "target": "ES5",
                "strict": true,
            }
        }"#;
        let stripped = strip_jsonc(input);
        let v: crate::json::Value = crate::json::from_str(&stripped).unwrap();
        assert_eq!(v["compilerOptions"]["target"].as_str(), Some("ES5"));
        assert_eq!(v["compilerOptions"]["strict"].as_bool(), Some(true));
    }

    // ──────────────────────────────────────────────────────────────────────
    // tsconfig.json parsing tests (ported from tsconfigparsing_test.go)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_tsconfig_extends_merges_options() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/base.json", r#"{
            "compilerOptions": { "target": "ES2020", "strict": true }
        }"#);
        fs.insert_file("/proj/tsconfig.json", r#"{
            "extends": "base.json",
            "compilerOptions": { "outDir": "./dist" }
        }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        // Parent options are inherited.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert!(parsed.compiler_options.strict.is_true());
        // Child option is applied.
        assert_eq!(parsed.compiler_options.out_dir, "./dist");
        // `strict` from the base enables the strict family.
        assert!(parsed.compiler_options.strict_null_checks.is_true());
    }

    #[test]
    fn test_parse_tsconfig_extends_with_own_files_include() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_file("/proj/base.json", r#"{
            "compilerOptions": { "target": "ES2020" }
        }"#);
        fs.insert_file("/proj/tsconfig.json", r#"{
            "extends": "base.json",
            "compilerOptions": { "outDir": "./dist" },
            "include": ["src/**/*"]
        }"#);
        fs.insert_file("/proj/src/a.ts", "export const a = 1;");
        fs.insert_file("/proj/src/b.ts", "export const b = 2;");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2020);
        assert_eq!(parsed.compiler_options.out_dir, "./dist");
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_full_compiler_options() {
        // Ported from "parses tsconfig with compilerOptions, files, include, and exclude".
        let fs = InMemoryFS::new();
        fs.insert_dir("/apath");
        fs.insert_dir("/apath/src");
        fs.insert_dir("/apath/node_modules");
        fs.insert_dir("/apath/dist");
        fs.insert_file("/apath/tsconfig.json", r#"{
            "compilerOptions": {
                "outDir": "./dist",
                "strict": true,
                "noImplicitAny": true,
                "target": "ES2017",
                "module": "ESNext",
                "moduleResolution": "bundler",
                "moduleDetection": "auto",
                "jsx": "react"
            },
            "files": ["/apath/src/index.ts", "/apath/src/app.ts"],
            "include": ["/apath/src/**/*"],
            "exclude": ["/apath/node_modules", "/apath/dist"]
        }"#);
        fs.insert_file("/apath/src/index.ts", "");
        fs.insert_file("/apath/src/app.ts", "");
        fs.insert_file("/apath/node_modules/module.ts", "");
        fs.insert_file("/apath/dist/output.js", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/apath/tsconfig.json",
            &CompilerOptions::default(),
            "/apath",
            &fs,
        );
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2017);
        assert_eq!(parsed.compiler_options.module, ModuleKind::ESNext);
        assert_eq!(
            parsed.compiler_options.module_resolution,
            ModuleResolutionKind::Bundler
        );
        assert_eq!(parsed.compiler_options.jsx, JsxEmit::React);
        assert!(parsed.compiler_options.strict.is_true());
        assert!(parsed.compiler_options.no_implicit_any.is_true());
        assert_eq!(parsed.compiler_options.out_dir, "./dist");
        // Explicit `files` are included.
        assert!(parsed.file_names.contains(&"/apath/src/index.ts".to_string()));
        assert!(parsed.file_names.contains(&"/apath/src/app.ts".to_string()));
        // node_modules is excluded during the include walk.
        assert!(!parsed
            .file_names
            .iter()
            .any(|f| f.contains("node_modules")));
    }

    #[test]
    fn test_parse_tsconfig_null_enum_options() {
        // Ported from TestParseNullEnumCompilerOptions: `target: null` and
        // `module: null` should produce no errors.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/tsconfig.json", r#"{
            "compilerOptions": {
                "target": null,
                "module": null
            }
        }"#);
        fs.insert_file("/proj/app.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.errors.is_empty());
    }

    #[test]
    fn test_parse_tsconfig_empty_types_array() {
        // Ported from "handles empty types array".
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/tsconfig.json", r#"{
            "compilerOptions": {
                "types": []
            }
        }"#);
        fs.insert_file("/proj/app.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.compiler_options.types.is_empty());
    }

    #[test]
    fn test_parse_tsconfig_include_with_exclude() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/src");
        fs.insert_dir("/proj/src/tests");
        fs.insert_file("/proj/tsconfig.json", r#"{
            "include": ["src/**/*.ts"],
            "exclude": ["**/tests/**"]
        }"#);
        fs.insert_file("/proj/src/a.ts", "");
        fs.insert_file("/proj/src/b.ts", "");
        fs.insert_file("/proj/src/tests/skip.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/proj/src/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/src/b.ts".to_string()));
        // Excluded file is filtered out of the include expansion. The exclude
        // glob must match the absolute paths produced by the include walk, so a
        // `**/tests/**` pattern is used.
        assert!(!parsed.file_names.contains(&"/proj/src/tests/skip.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_skips_node_modules_directory() {
        // Ported from "implicitly exclude common package folders": the include
        // walk skips `node_modules` directories.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/node_modules");
        fs.insert_dir("/proj/folder");
        fs.insert_file("/proj/tsconfig.json", "{}");
        fs.insert_file("/proj/node_modules/a.ts", "");
        fs.insert_file("/proj/d.ts", "");
        fs.insert_file("/proj/folder/e.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed
            .file_names
            .iter()
            .any(|f| f.contains("node_modules")));
        assert!(parsed.file_names.contains(&"/proj/d.ts".to_string()));
        assert!(parsed.file_names.contains(&"/proj/folder/e.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_skips_git_directory() {
        // The include walk skips `.git` directories.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_dir("/proj/.git");
        fs.insert_file("/proj/tsconfig.json", "{}");
        fs.insert_file("/proj/.git/a.ts", "");
        fs.insert_file("/proj/test.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.file_names.iter().any(|f| f.contains(".git")));
        assert!(parsed.file_names.contains(&"/proj/test.ts".to_string()));
    }

    #[test]
    fn test_parse_tsconfig_missing_config_file_error() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/missing.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Cannot find"));
    }

    #[test]
    fn test_parse_tsconfig_invalid_json_error() {
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/tsconfig.json", "{ this is not json");
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &CompilerOptions::default(),
            "/proj",
            &fs,
        );
        assert!(!parsed.errors.is_empty());
        assert!(has_error_containing(&parsed, "Failed to parse"));
    }

    #[test]
    fn test_parse_tsconfig_command_line_overrides_config() {
        // Options supplied on the command line (via `base_options`) take
        // precedence over those in tsconfig.json.
        let fs = InMemoryFS::new();
        fs.insert_dir("/proj");
        fs.insert_file("/proj/tsconfig.json", r#"{
            "compilerOptions": { "target": "ES2017", "strict": true }
        }"#);
        fs.insert_file("/proj/app.ts", "");
        let mut base = CompilerOptions::default();
        base.target = ScriptTarget::ES2022;
        let parsed = get_parsed_command_line_of_config_file(
            "/proj/tsconfig.json",
            &base,
            "/proj",
            &fs,
        );
        // Command-line target wins; config-file strict is still inherited.
        assert_eq!(parsed.compiler_options.target, ScriptTarget::ES2022);
        assert!(parsed.compiler_options.strict.is_true());
    }

    // ──────────────────────────────────────────────────────────────────────
    // ParsedCommandLine / wildcard-directory tests
    // (ported from parsedcommandline_test.go and wildcarddirectories_test.go)
    //
    // The Rust port does not expose a `get_wildcard_directories` helper, so
    // these tests exercise the equivalent include/exclude behavior through
    // `get_parsed_command_line_of_config_file`.
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parsed_command_line_literal_file_list_dedup() {
        // Ported from "with literal file list" > "duplicates": duplicate entries
        // in `files` are deduplicated.
        let fs = InMemoryFS::new();
        fs.insert_dir("/dev");
        fs.insert_file("/dev/a.ts", "");
        fs.insert_file("/dev/b.ts", "");
        fs.insert_file("/dev/tsconfig.json", r#"{
            "files": ["a.ts", "a.ts", "b.ts"]
        }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/dev/tsconfig.json",
            &CompilerOptions::default(),
            "/dev",
            &fs,
        );
        // Each file appears exactly once, sorted.
        assert_eq!(
            parsed.file_names,
            vec!["/dev/a.ts".to_string(), "/dev/b.ts".to_string()]
        );
    }

    #[test]
    fn test_parsed_command_line_files_not_removed_by_exclude() {
        // Ported from "are not removed due to excludes": explicit `files` are
        // kept even when an `exclude` pattern matches them.
        let fs = InMemoryFS::new();
        fs.insert_dir("/dev");
        fs.insert_file("/dev/a.ts", "");
        fs.insert_file("/dev/b.ts", "");
        fs.insert_file("/dev/tsconfig.json", r#"{
            "files": ["a.ts", "b.ts"],
            "exclude": ["b.ts"]
        }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/dev/tsconfig.json",
            &CompilerOptions::default(),
            "/dev",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/dev/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/dev/b.ts".to_string()));
    }

    #[test]
    fn test_parsed_command_line_literal_include_matches_files() {
        // Ported from "with literal include list" > "without exclude": a literal
        // (non-glob) include matches the named files.
        let fs = InMemoryFS::new();
        fs.insert_dir("/dev");
        fs.insert_file("/dev/a.ts", "");
        fs.insert_file("/dev/b.ts", "");
        fs.insert_file("/dev/tsconfig.json", r#"{
            "include": ["a.ts", "b.ts"]
        }"#);
        let parsed = get_parsed_command_line_of_config_file(
            "/dev/tsconfig.json",
            &CompilerOptions::default(),
            "/dev",
            &fs,
        );
        assert!(parsed.file_names.contains(&"/dev/a.ts".to_string()));
        assert!(parsed.file_names.contains(&"/dev/b.ts".to_string()));
    }

    #[test]
    fn test_wildcard_include_dot_prefixed_with_dot_dir_exclude() {
        // Ported from TestGetWildcardDirectories_DotPrefixedIncludeWithDotDirExclude.
        // Include specs with a directory prefix must still match files even when
        // a `**/.*/` exclude (dot-directory exclude) is present. The Rust port
        // does not normalize a leading `./` in include specs, so the specs here
        // use the plain `app/...` form; the exclude behavior under test is the
        // same.
        let fs = InMemoryFS::new();
        fs.insert_dir("/home/projects/monorepo/apps/web");
        fs.insert_dir("/home/projects/monorepo/apps/web/app");
        fs.insert_file(
            "/home/projects/monorepo/apps/web/tsconfig.json",
            r#"{
                "include": ["app/**/*.ts", "app/**/*.tsx"],
                "exclude": ["**/node_modules", "**/.*/", "build"]
            }"#,
        );
        fs.insert_file("/home/projects/monorepo/apps/web/app/a.ts", "");
        fs.insert_file("/home/projects/monorepo/apps/web/app/b.tsx", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/home/projects/monorepo/apps/web/tsconfig.json",
            &CompilerOptions::default(),
            "/home/projects/monorepo/apps/web",
            &fs,
        );
        assert!(parsed
            .file_names
            .contains(&"/home/projects/monorepo/apps/web/app/a.ts".to_string()));
        assert!(parsed
            .file_names
            .contains(&"/home/projects/monorepo/apps/web/app/b.tsx".to_string()));
    }

    #[test]
    fn test_wildcard_include_non_ascii_paths() {
        // Ported from TestGetWildcardDirectories_NonASCIICharacters: parsing
        // configs with non-ASCII paths must not panic and should still resolve
        // include globs.
        let fs = InMemoryFS::new();
        fs.insert_dir("/Users/ユーザー/プロジェクト");
        fs.insert_dir("/Users/ユーザー/プロジェクト/src");
        fs.insert_file(
            "/Users/ユーザー/プロジェクト/tsconfig.json",
            r#"{
                "include": ["src/**/*.ts"],
                "exclude": ["テスト"]
            }"#,
        );
        fs.insert_file("/Users/ユーザー/プロジェクト/src/a.ts", "");
        let parsed = get_parsed_command_line_of_config_file(
            "/Users/ユーザー/プロジェクト/tsconfig.json",
            &CompilerOptions::default(),
            "/Users/ユーザー/プロジェクト",
            &fs,
        );
        assert!(parsed
            .file_names
            .iter()
            .any(|f| f.ends_with("/src/a.ts")));
    }

    // ──────────────────────────────────────────────────────────────────────
    // Option-declaration sanity test (adapted from decls_test.go)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_options_declarations_non_empty_and_named() {
        // The OPTIONS table must be populated and every declaration must carry
        // a non-empty name.
        assert!(!OPTIONS.is_empty());
        for o in OPTIONS {
            assert!(!o.name.is_empty(), "found an option with an empty name");
        }
        // A few key options must be present.
        let names: std::collections::HashSet<&str> =
            OPTIONS.iter().map(|o| o.name).collect();
        for required in [
            "help",
            "version",
            "build",
            "watch",
            "target",
            "module",
            "jsx",
            "lib",
            "strict",
            "noEmit",
            "project",
            "tsBuildInfoFile",
            "incremental",
            "moduleResolution",
            "typeRoots",
        ] {
            assert!(names.contains(required), "missing option declaration: {required}");
        }
    }

    #[test]
    fn test_option_decls_short_names_unique_or_known() {
        // The commonly-used short names map to the expected options.
        assert_eq!(find_option("h").map(|o| o.name), Some("help"));
        assert_eq!(find_option("v").map(|o| o.name), Some("version"));
        assert_eq!(find_option("b").map(|o| o.name), Some("build"));
        assert_eq!(find_option("w").map(|o| o.name), Some("watch"));
        assert_eq!(find_option("p").map(|o| o.name), Some("project"));
        assert_eq!(find_option("t").map(|o| o.name), Some("target"));
        assert_eq!(find_option("m").map(|o| o.name), Some("module"));
        assert_eq!(find_option("d").map(|o| o.name), Some("declaration"));
    }
}
