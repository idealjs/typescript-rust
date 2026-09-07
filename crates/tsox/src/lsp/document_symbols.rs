use serde_json::{Value, json};
use std::sync::Arc;

pub(super) fn symbols_for_statements(
    statements: &[Arc<crate::ast::Node>],
    sf: &Arc<crate::ast::SourceFile>,
) -> Vec<Value> {
    use crate::ast::{NodeData, SyntaxKind};
    let mut result = Vec::new();
    for stmt in statements {
        match stmt.kind {
            SyntaxKind::VariableStatement => {
                if let NodeData::VariableStatement(vs) = &stmt.data {
                    if let NodeData::VariableDeclarationList(list) = &vs.declaration_list.data {
                        for decl in &list.declarations.nodes {
                            if let Some(sym) = document_symbol_for_node(decl, sf) {
                                result.push(sym);
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(sym) = document_symbol_for_node(stmt, sf) {
                    result.push(sym);
                }
            }
        }
    }
    result
}

fn document_symbol_for_node(
    node: &Arc<crate::ast::Node>,
    sf: &Arc<crate::ast::SourceFile>,
) -> Option<Value> {
    let name_node = node.name()?;
    let name = identifier_text(name_node)?;
    let kind = symbol_kind_for(node);

    let (rl, rc) = crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.pos());
    let (rel, rec) =
        crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, node.end());
    let (sl, sc) =
        crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, name_node.pos());
    let (sel, sec) =
        crate::diagnosticwriter::line_and_character(&sf.line_map, &sf.text, name_node.end());

    let mut sym = json!({
        "name": name,
        "kind": kind,
        "range": {
            "start": {"line": rl, "character": rc},
            "end": {"line": rel, "character": rec}
        },
        "selectionRange": {
            "start": {"line": sl, "character": sc},
            "end": {"line": sel, "character": sec}
        }
    });
    let children = child_symbols(node, sf);
    if !children.is_empty() {
        sym["children"] = json!(children);
    }
    Some(sym)
}

fn identifier_text(node: &Arc<crate::ast::Node>) -> Option<String> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::Identifier(data) => Some(data.text.clone()),
        NodeData::StringLiteral(data) => Some(data.text.clone()),
        NodeData::NumericLiteral(data) => Some(data.text.clone()),
        _ => Some(node.text().to_string()),
    }
}

fn symbol_kind_for(node: &Arc<crate::ast::Node>) -> i32 {
    use crate::ast::{NodeFlags, SyntaxKind as K};
    match node.kind {
        K::FunctionDeclaration => 12,
        K::ClassDeclaration => 5,
        K::InterfaceDeclaration => 11,
        K::TypeAliasDeclaration => 23,
        K::EnumDeclaration => 10,
        K::ModuleDeclaration => 3,
        K::VariableDeclaration => {
            let is_const = node
                .parent
                .as_ref()
                .map_or(false, |p| p.flags.contains(NodeFlags::Const));
            if is_const { 14 } else { 13 }
        }
        K::MethodDeclaration | K::MethodSignature => 6,
        K::GetAccessor | K::SetAccessor => 6,
        K::Constructor => 9,
        K::PropertyDeclaration | K::PropertySignature => 7,
        K::EnumMember => 22,
        _ => 13,
    }
}

fn child_symbols(node: &Arc<crate::ast::Node>, sf: &Arc<crate::ast::SourceFile>) -> Vec<Value> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::ClassDeclaration(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::ClassExpression(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::InterfaceDeclaration(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::EnumDeclaration(d) => d
            .members
            .nodes
            .iter()
            .filter_map(|m| document_symbol_for_node(m, sf))
            .collect(),
        NodeData::ModuleDeclaration(d) => {
            if let Some(body) = &d.body {
                if let NodeData::ModuleBlock(mb) = &body.data {
                    return symbols_for_statements(&mb.statements.nodes, sf);
                }
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}
