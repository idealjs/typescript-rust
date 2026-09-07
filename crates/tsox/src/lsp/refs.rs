use super::document_symbols::*;
use super::lsp_server::LspServer;
use super::symbol_nav::*;
use super::utils::*;

use serde_json::{Value, json};
use std::collections::HashMap;

impl LspServer {
    pub(super) fn handle_references(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!([]),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = match build_program_from_documents(&self.documents) {
            Some(p) => p,
            None => return json!([]),
        };
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!([]),
        };

        let offset = position_to_offset(params, content);
        let symbol_map = program.symbol_map();
        let target_symbol = match crate::astnav::get_token_at_position(&source_file.node, offset) {
            Some(node) => match resolve_symbol_for_node(symbol_map, &node) {
                Some(s) => s,
                None => return json!([]),
            },
            None => return json!([]),
        };

        let include_declaration = params
            .get("context")
            .and_then(|c| c.get("includeDeclaration"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let refs = find_all_references(&program, &target_symbol);
        let mut locations = Vec::with_capacity(refs.len());
        for (sf, node) in &refs {
            if !include_declaration && is_declaration_name(symbol_map, node) {
                continue;
            }
            let (sl, sc) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.pos());
            let (el, ec) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.end());
            locations.push(json!({
                "uri": path_to_uri(&sf.file_name),
                "range": {
                    "start": {"line": sl, "character": sc},
                    "end": {"line": el, "character": ec}
                }
            }));
        }
        json!(locations)
    }

    pub(super) fn handle_document_symbol(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!([]),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!([]),
        };

        let statements = match &source_file.node.data {
            crate::ast::NodeData::SourceFile(data) => data.statements.nodes.as_slice(),
            _ => return json!([]),
        };
        json!(symbols_for_statements(statements, &source_file))
    }

    pub(super) fn handle_rename(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return json!({}),
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        let new_name = params
            .get("newName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if new_name.is_empty() {
            return json!({});
        }

        let program = match build_program_from_documents(&self.documents) {
            Some(p) => p,
            None => return json!({}),
        };
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return json!({}),
        };

        let offset = position_to_offset(params, content);
        let symbol_map = program.symbol_map();
        let target_symbol = match crate::astnav::get_token_at_position(&source_file.node, offset) {
            Some(node) => match resolve_symbol_for_node(symbol_map, &node) {
                Some(s) => s,
                None => return json!({}),
            },
            None => return json!({}),
        };

        let refs = find_all_references(&program, &target_symbol);
        let mut changes: HashMap<String, Vec<Value>> = HashMap::new();
        for (sf, node) in &refs {
            let (sl, sc) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.pos());
            let (el, ec) =
                crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.end());
            changes
                .entry(path_to_uri(&sf.file_name))
                .or_default()
                .push(json!({
                    "range": {
                        "start": {"line": sl, "character": sc},
                        "end": {"line": el, "character": ec}
                    },
                    "newText": new_name
                }));
        }
        json!({ "changes": changes })
    }
}
