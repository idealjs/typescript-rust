#![allow(unused_imports)]

use super::*;

pub(crate) fn extract_reference_types_directives(text: &str) -> Vec<ReferenceTypesDirective> {
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

pub(crate) fn load_lib_recursive(
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

pub(crate) fn extract_reference_lib_directives(text: &str) -> Vec<String> {
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

pub(crate) fn parser_diagnostic_to_diagnostic(
    file: Arc<SourceFile>,
    pd: &crate::parser::ParserDiagnostic,
) -> Diagnostic {
    Diagnostic::new(Some(file), pd.range, pd.message, pd.message_args.clone())
}

pub(crate) fn file_error_diagnostic(file_name: &str, _message: &str) -> Diagnostic {
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
pub(crate) fn _ensure_script_kind(file_name: &str) -> crate::ast::ScriptKind {
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
