use std::sync::Arc;

use crate::ast::node_data_generated::{
    for_each_child, is_meta_property, is_token_kind, NodeData,
};
use crate::ast::node_node::Node;
use crate::ast::node_source_file::{ScriptKind, SourceFile};
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_expressions::is_string_literal_like;
use crate::ast::utilities_misc::is_import_call;

fn node_contains_position(node: &Node, position: usize) -> bool {
    !is_token_kind(node.kind)
        && node.loc.pos() <= position
        && (position < node.loc.end()
            || (position == node.loc.end() && node.kind == SyntaxKind::EndOfFile))
}

pub fn get_node_at_position(file: &SourceFile, position: usize, include_jsdoc: bool) -> Arc<Node> {
    let mut current = Arc::clone(&file.node);
    loop {
        let mut child: Option<Arc<Node>> = None;
        if include_jsdoc {
            for jsdoc in file.resolve_jsdoc(&current) {
                if node_contains_position(&jsdoc, position) {
                    child = Some(Arc::clone(&jsdoc));
                    break;
                }
            }
        }
        if child.is_none() {
            for_each_child(&current, |n| {
                if node_contains_position(n, position) {
                    child = Some(Arc::clone(n));
                    return true;
                }
                false
            });
        }
        match child {
            None => return current,
            Some(c) => {
                if is_meta_property(&c) {
                    return current;
                }
                current = c;
            }
        }
    }
}

fn find_import_or_require(text: &str, start: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut index = start.max(0);
    while index < bytes.len() {
        let rel = bytes[index..]
            .iter()
            .position(|&b| b == b'i' || b == b'r')?;
        index += rel;
        let (size, expected): (usize, &[u8]) = if bytes[index] == b'i' {
            (6, b"import")
        } else {
            (7, b"require")
        };
        if index + size <= bytes.len() && &bytes[index..index + size] == expected {
            return Some((index, size));
        }
        index += 1;
    }
    None
}

fn is_require_call(node: &Node, require_string_literal_like_argument: bool) -> bool {
    if node.kind != SyntaxKind::CallExpression {
        return false;
    }
    let NodeData::CallExpression(call) = &node.data else {
        return false;
    };
    if call.expression.kind != SyntaxKind::Identifier || call.expression.text() != "require" {
        return false;
    }
    if call.arguments.nodes.len() != 1 {
        return false;
    }
    !require_string_literal_like_argument || is_string_literal_like(&call.arguments.nodes[0])
}

fn is_literal_import_type_node(node: &Node) -> bool {
    if node.kind != SyntaxKind::ImportType {
        return false;
    }
    let NodeData::ImportTypeNode(d) = &node.data else {
        return false;
    };
    d.argument.kind == SyntaxKind::LiteralType
        && matches!(
            &d.argument.data,
            NodeData::LiteralTypeNode(lt) if lt.literal.kind == SyntaxKind::StringLiteral
        )
}

/// Go ForEachDynamicImportOrRequireCall：文本扫描 import/require 命中点，
/// 取该位置最深节点判定动态导入调用或类型空间 ImportType
pub fn for_each_dynamic_import_or_require_call(
    file: &SourceFile,
    include_type_space_imports: bool,
    require_string_literal_like_argument: bool,
    mut cb: impl FnMut(&Arc<Node>, &Arc<Node>) -> bool,
) -> bool {
    let is_javascript_file = matches!(file.script_kind, ScriptKind::Js | ScriptKind::Jsx);
    let text = file.text.as_str();
    let mut cursor = find_import_or_require(text, 0);
    while let Some((index, size)) = cursor {
        let node = get_node_at_position(file, index, is_javascript_file && include_type_space_imports);
        if is_javascript_file && is_require_call(&node, require_string_literal_like_argument) {
            let NodeData::CallExpression(call) = &node.data else {
                cursor = find_import_or_require(text, index + size);
                continue;
            };
            if cb(&node, &call.arguments.nodes[0]) {
                return true;
            }
        } else if is_import_call(&node) {
            let NodeData::CallExpression(call) = &node.data else {
                cursor = find_import_or_require(text, index + size);
                continue;
            };
            if let Some(arg0) = call.arguments.nodes.first() {
                if !require_string_literal_like_argument || is_string_literal_like(arg0) {
                    if cb(&node, arg0) {
                        return true;
                    }
                }
            }        } else if include_type_space_imports && is_literal_import_type_node(&node) {
            let NodeData::ImportTypeNode(d) = &node.data else {
                cursor = find_import_or_require(text, index + size);
                continue;
            };
            let NodeData::LiteralTypeNode(lt) = &d.argument.data else {
                cursor = find_import_or_require(text, index + size);
                continue;
            };
            if cb(&node, &lt.literal) {
                return true;
            }
        }
        cursor = find_import_or_require(text, index + size);
    }
    false
}
