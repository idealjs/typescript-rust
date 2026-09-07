use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, SourceFile, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::core::compiler_options::JsxEmit;
use crate::emitter::commonjs::*;
use crate::emitter::*;
use crate::tspath::{self};

pub(crate) fn collect_import_clause_type_cuts(
    clause: &Node,
    source: &str,
    cuts: &mut Vec<(usize, usize)>,
) {
    let cd = match &clause.data {
        NodeData::ImportClause(cd) => cd,
        _ => return,
    };

    let bindings = match &cd.named_bindings {
        Some(b) => b,
        None => return,
    };
    let named = match &bindings.data {
        NodeData::NamedImports(named) => named,
        _ => return,
    };
    if named.elements.is_empty() {
        return;
    }
    let all_type = named
        .elements
        .iter()
        .all(|spec| is_type_only_import_specifier(spec));

    if all_type {
        if cd.name.is_some() {
            let bytes = source.as_bytes();
            let mut start = bindings.pos();
            while start > 0 && (bytes[start - 1] == b' ' || bytes[start - 1] == b'\t') {
                start -= 1;
            }
            if start > 0 && bytes[start - 1] == b',' {
                start -= 1;
            }
            cuts.push((start, bindings.end()));
        }
        return;
    }

    for spec in named.elements.iter() {
        if is_type_only_import_specifier(spec) {
            cuts.push(specifier_cut_range(spec, source));
        }
    }
}

pub(crate) fn specifier_cut_range(spec: &Node, source: &str) -> (usize, usize) {
    let s = spec.pos();
    let e = spec.end();
    let bytes = source.as_bytes();

    let mut back = s;
    while back > 0 && (bytes[back - 1] == b' ' || bytes[back - 1] == b'\t') {
        back -= 1;
    }
    if back > 0 && bytes[back - 1] == b',' {
        return (back - 1, e);
    }

    let mut fwd = e;
    while fwd < bytes.len() && (bytes[fwd] == b' ' || bytes[fwd] == b'\t') {
        fwd += 1;
    }
    if fwd < bytes.len() && bytes[fwd] == b',' {
        let mut end = fwd + 1;
        while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
            end += 1;
        }
        return (s, end);
    }

    (s, e)
}

#[derive(Default)]
pub(crate) struct JsxRuntimeUsage {
    pub(crate) used_jsx: bool,
    pub(crate) used_jsxs: bool,
    pub(crate) used_fragment: bool,
}

pub(crate) fn needs_jsx_transform(options: &CompilerOptions, source_file: &SourceFile) -> bool {
    matches!(options.jsx, JsxEmit::ReactJSX | JsxEmit::ReactJSXDev)
        && tspath::file_extension_is(&source_file.file_name, ".tsx")
}

pub(crate) fn collect_jsx_replacements(
    statements: &[Arc<Node>],
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> Vec<(usize, usize, String)> {
    let mut replacements = Vec::new();
    for stmt in statements {
        collect_jsx_replacements_recursive(stmt, source, &mut replacements, usage);
    }
    replacements
}

pub(crate) fn collect_jsx_replacements_recursive(
    node: &Node,
    source: &str,
    replacements: &mut Vec<(usize, usize, String)>,
    usage: &mut JsxRuntimeUsage,
) {
    match node.kind {
        SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
            let text = generate_jsx_call(node, source, usage);
            replacements.push((node.pos(), node.end(), text));
        }
        _ => {
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collect_jsx_replacements_recursive(child, source, replacements, usage);
                false
            });
        }
    }
}

pub(crate) fn generate_jsx_call(node: &Node, source: &str, usage: &mut JsxRuntimeUsage) -> String {
    match &node.data {
        NodeData::JsxSelfClosingElement(d) => {
            generate_element_call(&d.tag_name, &d.attributes, None, source, usage)
        }
        NodeData::JsxElement(d) => {
            let opener = &d.opening_element;
            let (tag_name, attributes) = match &opener.data {
                NodeData::JsxOpeningElement(o) => (&o.tag_name, &o.attributes),
                _ => return source[node.pos()..node.end()].to_string(),
            };
            generate_element_call(tag_name, attributes, Some(&d.children), source, usage)
        }
        NodeData::JsxFragment(d) => generate_fragment_call(&d.children, source, usage),
        _ => source[node.pos()..node.end()].to_string(),
    }
}
