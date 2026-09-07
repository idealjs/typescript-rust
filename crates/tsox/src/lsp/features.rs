use super::lsp_server::LspServer;
use super::utils::*;

use serde_json::{Value, json};
use std::sync::Arc;

impl LspServer {
    pub(super) fn handle_hover(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return Value::Null,
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let Some(source_file) = program.get_source_file(path) else {
            return Value::Null;
        };

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
        let offset = source_file
            .line_map
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(0) as usize
            + character;

        let node = find_deepest_node(&source_file.node, offset);
        let mut checker = program.build_checker();

        let parts = checker.get_quick_info_display_parts(&node);
        let type_str = if parts.is_empty() {
            checker.get_quick_info_text(&node)
        } else {
            display_parts_to_string(&parts)
        };
        if type_str.is_empty() {
            return Value::Null;
        }

        json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```typescript\n{}\n```", type_str)
            }
        })
    }

    pub(super) fn handle_definition(&self, params: &Value) -> Value {
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
        let offset = line_map.line_starts.get(line).copied().unwrap_or(0) as usize + character;

        let node = find_deepest_node(&source_file.node, offset);
        let checker = program.build_checker();

        let symbol = checker.resolve_identifier(&node).or_else(|| {
            let symbol_map = checker.program.symbol_map();
            let mut current: Option<&Arc<crate::ast::Node>> = Some(&node);
            while let Some(n) = current {
                if let Some(sym) = symbol_map.symbol_of(n) {
                    return Some(Arc::clone(sym));
                }
                current = n.parent.as_ref();
            }
            None
        });

        if let Some(symbol) = symbol {
            if let Some(decl) = symbol.value_declaration.as_ref() {
                let decl_file = checker
                    .program
                    .source_files()
                    .iter()
                    .find(|sf| decl.loc.pos() < sf.text.len())
                    .cloned();

                if let Some(sf) = decl_file {
                    let (dl, dc) = crate::diagnosticwriter::line_and_character(
                        &sf.line_map,
                        &sf.text,
                        decl.loc.pos(),
                    );
                    let decl_uri = if sf.file_name.starts_with('/') {
                        format!("file://{}", sf.file_name)
                    } else {
                        format!("file:///{}", sf.file_name)
                    };
                    return json!([{
                        "uri": decl_uri,
                        "range": {
                            "start": {"line": dl, "character": dc},
                            "end": {"line": dl, "character": dc + decl.loc.len().max(1)}
                        }
                    }]);
                }
            }
        }

        json!([])
    }

    pub(super) fn handle_completion(&self, params: &Value) -> Value {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = match self.documents.get(uri) {
            Some(c) => c.as_str(),
            None => return Value::Null,
        };
        let path = uri.strip_prefix("file://").unwrap_or(uri);

        let program = build_program(path, content);
        let source_file = match program.get_source_file(path) {
            Some(sf) => sf,
            None => return Value::Null,
        };

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
        let offset = line_map.line_starts.get(line).copied().unwrap_or(0) as usize + character;

        let _node = find_deepest_node(&source_file.node, offset);
        let checker = program.build_checker();

        let mut items: Vec<Value> = Vec::new();

        for (name, sym) in checker.globals.iter() {
            if name.starts_with("__") {
                continue;
            }
            let kind = completion_item_kind(&sym.flags);
            items.push(json!({
                "label": name,
                "kind": kind
            }));
        }

        if items.len() < 50 {
            for kw in &[
                "const",
                "let",
                "var",
                "function",
                "class",
                "interface",
                "type",
                "enum",
                "if",
                "else",
                "for",
                "while",
                "do",
                "switch",
                "case",
                "break",
                "continue",
                "return",
                "try",
                "catch",
                "finally",
                "throw",
                "new",
                "delete",
                "typeof",
                "instanceof",
                "in",
                "of",
                "import",
                "export",
                "from",
                "default",
                "async",
                "await",
                "public",
                "private",
                "protected",
                "readonly",
                "static",
                "abstract",
                "declare",
                "namespace",
                "module",
                "as",
                "is",
                "satisfies",
                "true",
                "false",
                "null",
                "undefined",
                "void",
                "this",
                "super",
                "extends",
                "implements",
                "get",
                "set",
            ] {
                items.push(json!({"label": kw, "kind": 14}));
            }
        }

        if items.is_empty() {
            Value::Null
        } else {
            json!({
                "isIncomplete": true,
                "items": items
            })
        }
    }
}

fn completion_item_kind(flags: &crate::ast::SymbolFlags) -> i32 {
    use crate::ast::SymbolFlags as F;
    if flags.contains(F::FunctionScopedVariable | F::BlockScopedVariable)
        || flags.contains(F::Function)
    {
        3
    } else if flags.contains(F::Class) {
        7
    } else if flags.contains(F::Interface) {
        8
    } else if flags.contains(F::RegularEnum | F::ConstEnum) {
        13
    } else if flags.contains(F::TypeAlias | F::TypeParameter) {
        25
    } else if flags.contains(F::ValueModule | F::NamespaceModule) {
        9
    } else if flags.contains(F::Property) {
        5
    } else if flags.contains(F::Method) {
        2
    } else if flags.contains(F::ConstEnum | F::RegularEnum | F::EnumMember) {
        21
    } else {
        6
    }
}
