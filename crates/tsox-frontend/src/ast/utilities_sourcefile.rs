use crate::ast::*;
use std::sync::Arc;

pub fn is_in_js_file(node: &Node) -> bool {
    node.flags.contains(NodeFlags::JavaScriptFile)
}

pub fn is_in_json_file(node: &Node) -> bool {
    node.flags.contains(NodeFlags::JsonFile)
}

pub fn is_source_file_js(file: &SourceFile) -> bool {
    file.script_kind == ScriptKind::Js || file.script_kind == ScriptKind::Jsx
}

pub fn is_json_source_file(file: &SourceFile) -> bool {
    file.script_kind == ScriptKind::Json
}

pub fn is_external_module(file: &SourceFile) -> bool {
    file.external_module_indicator.is_some()
}

pub fn is_external_or_common_js_module(file: &SourceFile) -> bool {
    file.external_module_indicator.is_some() || file.common_js_module_indicator.is_some()
}

pub fn get_line_and_character_of_position(file: &SourceFile, position: usize) -> (usize, usize) {
    let line = file.line_map.line_at(position);
    let character = file.line_map.utf16_column_at(&file.text, position);
    (line, character)
}

pub fn get_position_of_line_and_character(
    file: &SourceFile,
    line: usize,
    character: usize,
) -> usize {
    if line >= file.line_map.line_starts.len() {
        return file.text.len();
    }
    let line_start = file.line_map.line_starts[line] as usize;

    let mut col = 0usize;
    let bytes = file.text.as_bytes();
    let mut pos = line_start;
    while pos < file.text.len() && col < character {
        let b = bytes[pos];
        if b < 0x80 {
            pos += 1;
            col += 1;
        } else {
            let s = &file.text[pos..];
            match s.chars().next() {
                Some(ch) => {
                    pos += ch.len_utf8();
                    col += ch.len_utf16();
                }
                None => break,
            }
        }
    }
    pos
}

pub fn source_file_of_node_or_panic(node: &Arc<Node>) -> Arc<Node> {
    get_source_file_of_node(node)
        .unwrap_or_else(|| panic!("get_source_file_of_node: node is not contained in a SourceFile"))
}
