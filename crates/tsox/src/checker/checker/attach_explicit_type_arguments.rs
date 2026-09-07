#![allow(unused_imports)]

use super::*;

pub(crate) fn attach_explicit_type_arguments(t: &Arc<Type>, args: Vec<Arc<Type>>) -> Arc<Type> {
    if let TypeData::Object(o) = &t.data {
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: o.structured.members.clone(),
                    properties: o.structured.properties.clone(),
                    signatures: o.structured.signatures.clone(),
                    call_signature_count: o.structured.call_signature_count,
                    index_infos: o.structured.index_infos.clone(),
                    ..Default::default()
                },
                target: Some(match &o.target {
                    Some(tg) => Arc::clone(tg),
                    None => Arc::clone(t),
                }),
                mapper: o.mapper.clone(),
                type_arguments: args,
            }),
        );
        rebuilt.object_flags = t.object_flags | ObjectFlags::Reference;
        rebuilt.symbol = t.symbol.clone();
        return Arc::new(rebuilt);
    }
    Arc::clone(t)
}

pub(crate) fn qualified_name_text(name: &Arc<Node>) -> String {
    match &name.data {
        crate::ast::NodeData::QualifiedName(d) => {
            format!("{}.{}", qualified_name_text(&d.left), d.right.text())
        }
        _ => name.text().to_string(),
    }
}

pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

pub(crate) fn levenshtein_with_max(s1: &str, s2: &str, max: f64) -> Option<f64> {
    let s1: Vec<char> = s1.chars().collect();
    let s2: Vec<char> = s2.chars().collect();
    let big = max + 0.01;
    let mut prev: Vec<f64> = (0..=s2.len()).map(|i| i as f64).collect();
    let mut curr = vec![0.0f64; s2.len() + 1];
    for i in 1..=s1.len() {
        let c1 = s1[i - 1];
        let min_j = (((i as f64) - max).ceil().max(1.0)) as usize;
        let max_j = ((max + i as f64).floor()) as usize;
        let max_j = max_j.min(s2.len());
        curr[0] = i as f64;
        let mut col_min = i as f64;
        for j in 1..(min_j.min(s2.len() + 1)) {
            curr[j] = big;
        }
        if min_j <= max_j {
            for j in min_j..=max_j {
                let substitution = if c1.to_lowercase().eq(s2[j - 1].to_lowercase()) {
                    prev[j - 1] + 0.1
                } else {
                    prev[j - 1] + 2.0
                };
                let dist = if c1 == s2[j - 1] {
                    prev[j - 1]
                } else {
                    (prev[j] + 1.0).min(curr[j - 1] + 1.0).min(substitution)
                };
                curr[j] = dist;
                col_min = col_min.min(dist);
            }
        }
        for j in (max_j + 1)..=s2.len() {
            curr[j] = big;
        }
        if col_min > max {
            return None;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    let res = prev[s2.len()];
    if res > max {
        return None;
    }
    Some(res)
}

pub(crate) fn relative_emit_specifier(from_file: &str, symbol_file: &str) -> String {
    let from_dir = {
        let dir = from_file.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        dir.trim_end_matches('/').to_string()
    };
    let from_segs: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to_segs: Vec<&str> = symbol_file.split('/').filter(|s| !s.is_empty()).collect();

    let mut common = 0;
    while common < from_segs.len()
        && common < to_segs.len().saturating_sub(1)
        && from_segs[common] == to_segs[common]
    {
        common += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_segs.len() {
        parts.push("..".to_string());
    }
    for seg in &to_segs[common..] {
        parts.push((*seg).to_string());
    }
    let last = parts.len().saturating_sub(1);
    if let Some(name) = parts.last().cloned() {
        let mapped = if let Some(stripped) = name.strip_suffix(".d.ts") {
            format!("{stripped}.d.ts")
        } else if let Some(stripped) = name.strip_suffix(".mts") {
            format!("{stripped}.mjs")
        } else if let Some(stripped) = name.strip_suffix(".cts") {
            format!("{stripped}.cjs")
        } else if let Some(stripped) = name.strip_suffix(".tsx") {
            format!("{stripped}.jsx")
        } else if let Some(stripped) = name.strip_suffix(".ts") {
            format!("{stripped}.js")
        } else {
            name
        };
        parts[last] = mapped;
    }
    let mut spec = parts.join("/");
    if !spec.starts_with("..") {
        spec = format!("./{spec}");
    }
    spec
}

pub(crate) fn module_format_is_esm_for_require_check(
    path: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> bool {
    use crate::core::compiler_options::ModuleKind;
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".d.ts") {
        return false;
    }
    crate::compiler::implied_node_format_of_file(path, read_file) == ModuleKind::ESNext
}

pub(crate) fn importer_is_cjs_for_require_check(
    path: &str,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> bool {
    use crate::core::compiler_options::ModuleKind;
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".d.ts") {
        return true;
    }
    crate::compiler::implied_node_format_of_file(path, read_file) == ModuleKind::CommonJS
}

pub(crate) fn module_is_instantiated(node: &Arc<Node>, preserve_const_enums: bool) -> bool {
    let state = module_instance_state(node, &mut Vec::new());
    state == 2 || (preserve_const_enums && state == 1)
}

pub(crate) fn module_instance_state(node: &Arc<Node>, visited: &mut Vec<usize>) -> u8 {
    let id = Arc::as_ptr(node) as usize;

    if visited.contains(&id) {
        return 0;
    }
    visited.push(id);
    let state = module_instance_state_worker(node, visited);
    visited.pop();
    state
}

pub(crate) fn module_instance_state_worker(node: &Arc<Node>, visited: &mut Vec<usize>) -> u8 {
    match &node.data {
        crate::ast::NodeData::InterfaceDeclaration(_)
        | crate::ast::NodeData::TypeAliasDeclaration(_) => 0,
        crate::ast::NodeData::EnumDeclaration(_) => {
            if node.has_syntactic_modifier(ModifierFlags::Const) {
                1
            } else {
                2
            }
        }
        crate::ast::NodeData::ImportDeclaration(_)
        | crate::ast::NodeData::ImportEqualsDeclaration(_) => {
            if node.has_syntactic_modifier(ModifierFlags::Export) {
                2
            } else {
                0
            }
        }
        crate::ast::NodeData::ExportDeclaration(ed) => {
            if ed.module_specifier.is_none()
                && ed
                    .export_clause
                    .as_ref()
                    .is_some_and(|c| c.kind == SyntaxKind::NamedExports)
            {
                let clause = ed.export_clause.as_ref().unwrap();
                let crate::ast::NodeData::NamedExports(named) = &clause.data else {
                    return 2;
                };
                let mut state = 0u8;
                for spec in &named.elements.nodes {
                    let s = module_alias_target_state(spec, node, visited);
                    if s > state {
                        state = s;
                    }
                    if state == 2 {
                        return 2;
                    }
                }
                state
            } else {
                2
            }
        }
        crate::ast::NodeData::ModuleDeclaration(md) => match &md.body {
            Some(body) => module_instance_state(body, visited),
            None => 2,
        },
        crate::ast::NodeData::ModuleBlock(block) => {
            let mut state = 0u8;
            for stmt in &block.statements.nodes {
                let child = module_instance_state(stmt, visited);
                if child == 2 {
                    return 2;
                }
                if child == 1 {
                    state = 1;
                }
            }
            state
        }
        _ => 2,
    }
}
