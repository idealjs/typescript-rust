use std::sync::Arc;

use crate::ast::node_flags::ModifierFlags;
use crate::ast::{Node, NodeFlags, SyntaxKind};
use crate::core::compiler_options::{CompilerOptions, ScriptTarget};

pub(crate) fn needs_es5_downlevel(options: &CompilerOptions) -> bool {
    options.target == ScriptTarget::ES5
}

pub(crate) fn collect_es5_replacements(
    statements: &[Arc<Node>],
) -> Vec<(usize, usize, &'static str)> {
    let mut replacements = Vec::new();
    for stmt in statements {
        collect_es5_replacements_recursive(stmt, &mut replacements);
    }
    replacements
}

pub(crate) fn collect_es5_replacements_recursive(
    node: &Node,
    replacements: &mut Vec<(usize, usize, &'static str)>,
) {
    if node.kind == crate::ast::SyntaxKind::VariableDeclarationList {
        let flags = node.flags;
        if flags.contains(NodeFlags::Const) {
            let pos = node.pos();
            replacements.push((pos, pos + 5, "var"));
        } else if flags.contains(NodeFlags::Let) {
            let pos = node.pos();
            replacements.push((pos, pos + 3, "var"));
        }
    }

    crate::ast::node_data_generated::for_each_child(node, |child| {
        collect_es5_replacements_recursive(child, replacements);
        false
    });
}

pub(crate) fn collect_export_modifier_cuts(stmt: &Node, source: &str) -> Vec<(usize, usize)> {
    let modifiers = match stmt.modifiers() {
        Some(m) => m,
        None => return Vec::new(),
    };
    if !modifiers.modifier_flags.contains(ModifierFlags::Export) {
        return Vec::new();
    }

    let mut cuts = Vec::new();
    let bytes = source.as_bytes();
    for mod_node in modifiers.list.iter() {
        if mod_node.kind == SyntaxKind::ExportKeyword || mod_node.kind == SyntaxKind::DefaultKeyword
        {
            let start = mod_node.pos();
            let mut end = mod_node.end();

            while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
                end += 1;
            }
            cuts.push((start, end));
        }
    }
    cuts
}
