#![allow(unused_imports)]

use super::*;

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

pub(crate) fn ambient_module_exists(source_files: &[Arc<crate::ast::SourceFile>], name: &str) -> bool {
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

pub(crate) fn strip_quotes(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2
        && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
