use crate::ast::*;
use std::sync::Arc;

fn strip_quotes(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2
        && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn literal_attributes(node: Option<&Arc<Node>>) -> Option<Vec<(String, String)>> {
    let NodeData::ImportAttributes(ImportAttributesData { attributes, .. }) = &node?.data else {
        return None;
    };
    let mut pairs = Vec::new();
    for element in attributes.nodes.iter() {
        let NodeData::ImportAttribute(attr) = &element.data else {
            return None;
        };
        let value = match &attr.value.data {
            NodeData::StringLiteral(s) => s.text.trim_matches(['"', '\'', '`']).to_string(),
            _ => return None,
        };
        pairs.push((attr.name.text().to_string(), value));
    }
    Some(pairs)
}

fn attributes_subset_of(module: &[(String, String)], import: &[(String, String)]) -> bool {
    module
        .iter()
        .all(|(k, v)| import.iter().any(|(ik, iv)| ik == k && iv == v))
}

pub fn pattern_ambient_module_with_attributes_exists(
    source_files: &[Arc<SourceFile>],
    name: &str,
    import_attributes: Option<&Arc<Node>>,
) -> bool {
    let import_pairs = literal_attributes(import_attributes).unwrap_or_default();
    for file in source_files {
        if file.external_module_indicator.is_some() {
            continue;
        }
        let NodeData::SourceFile(sf) = &file.node.data else {
            continue;
        };
        for stmt in sf.statements.iter() {
            let NodeData::ModuleDeclaration(md) = &stmt.data else {
                continue;
            };
            if md.name.kind != SyntaxKind::StringLiteral {
                continue;
            }
            let pattern = strip_quotes(md.name.text());
            let Some(star) = pattern.find('*') else {
                continue;
            };
            if pattern[star + 1..].contains('*') {
                continue;
            }
            let (prefix, suffix) = (&pattern[..star], &pattern[star + 1..]);
            if !(name.len() >= prefix.len() + suffix.len()
                && name.starts_with(prefix)
                && name.ends_with(suffix))
            {
                continue;
            }
            match literal_attributes(md.attributes.as_ref()) {
                None => return true,
                Some(module_pairs) if module_pairs.is_empty() => return true,
                Some(module_pairs) if attributes_subset_of(&module_pairs, &import_pairs) => {
                    return true
                }
                Some(_) => {}
            }
        }
    }
    false
}

pub fn import_attributes_of_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => d.attributes.clone(),
        NodeData::ExportDeclaration(d) => d.attributes.clone(),
        _ => None,
    }
}
