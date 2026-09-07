use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

pub(super) fn build_program(path: &str, content: &str) -> Arc<crate::compiler::Program> {
    let fs = Arc::new(crate::vfs::InMemoryFS::new());
    let parent = std::path::Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());
    fs.insert_dir(&parent);
    fs.insert_file(path, content);

    let host = crate::compiler::CompilerHostImpl::new(fs, parent, crate::bundled::lib_path());
    let host: Arc<dyn crate::compiler::CompilerHost> = Arc::new(host);

    let mut config = crate::tsoptions::ParsedCommandLine::default();
    config.file_names = vec![path.to_string()];
    config.compiler_options.no_lib = crate::core::tristate::Tristate::True;

    Arc::new(crate::compiler::Program::new(
        crate::compiler::ProgramOptions { config, host },
    ))
}

pub(super) fn display_parts_to_string(
    parts: &[crate::checker::nodebuilder::SymbolDisplayPart],
) -> String {
    parts.iter().map(|p| p.text.as_str()).collect()
}

pub(super) fn find_deepest_node(
    node: &Arc<crate::ast::Node>,
    offset: usize,
) -> Arc<crate::ast::Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<crate::ast::Node>> = None;
        crate::ast::for_each_child(&current, |child| {
            if child.loc.pos() <= offset && offset < child.loc.end() {
                next = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        match next {
            Some(child) => deepest = child,
            None => break,
        }
    }
    deepest
}

pub(super) fn position_to_offset(params: &Value, content: &str) -> usize {
    let line = params
        .get("position")
        .and_then(|p| p.get("line"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let character = params
        .get("position")
        .and_then(|p| p.get("character"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let line_map = crate::ast::LineMap::from_text(content);
    line_map.line_starts.get(line).copied().unwrap_or(0) as usize + character
}

pub(super) fn build_program_from_documents(
    documents: &HashMap<String, String>,
) -> Option<Arc<crate::compiler::Program>> {
    if documents.is_empty() {
        return None;
    }
    let fs = Arc::new(crate::vfs::InMemoryFS::new());
    fs.insert_dir("/");
    let mut file_names = Vec::with_capacity(documents.len());
    for (uri, content) in documents {
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        fs.insert_file(path, content);
        file_names.push(path.to_string());
    }
    let host =
        crate::compiler::CompilerHostImpl::new(fs, "/".to_string(), crate::bundled::lib_path());
    let host: Arc<dyn crate::compiler::CompilerHost> = Arc::new(host);

    let mut config = crate::tsoptions::ParsedCommandLine::default();
    config.file_names = file_names;
    config.compiler_options.no_lib = crate::core::tristate::Tristate::True;

    Some(Arc::new(crate::compiler::Program::new(
        crate::compiler::ProgramOptions { config, host },
    )))
}

pub(super) fn path_to_uri(path: &str) -> String {
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

pub(super) fn diagnostic_to_lsp(diag: &crate::ast::diagnostic::Diagnostic, content: &str) -> Value {
    let (line, col) = if let Some(file) = &diag.file {
        crate::diagnosticwriter::line_and_character(&file.line_map, &file.text, diag.loc.pos())
    } else {
        let line_map = crate::ast::LineMap::from_text(content);
        let pos = diag.loc.pos().min(content.len());
        crate::diagnosticwriter::line_and_character(&line_map, content, pos)
    };

    let severity = match diag.category {
        crate::diagnostics::Category::Error => 1,
        crate::diagnostics::Category::Warning => 2,
        crate::diagnostics::Category::Suggestion => 3,
        crate::diagnostics::Category::Message => 3,
    };

    let message = crate::diagnosticwriter::message_text(diag, None);

    json!({
        "range": {
            "start": {"line": line, "character": col},
            "end": {"line": line, "character": col + diag.loc.len().max(1)}
        },
        "severity": severity,
        "code": diag.code,
        "source": "tsox",
        "message": message
    })
}

pub(super) fn make_response(id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

pub(super) fn make_error_response(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}
