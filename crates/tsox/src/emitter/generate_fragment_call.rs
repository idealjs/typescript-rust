#![allow(unused_imports)]

use super::*;

pub(crate) fn generate_fragment_call(
    children: &Arc<NodeList>,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> String {
    usage.used_fragment = true;

    let children_prop = convert_children(children, source, usage);
    let is_static = is_static_children(children);

    let props_str = match children_prop {
        Some(c) => format!("{{ children: {} }}", c),
        None => "{}".to_string(),
    };

    let callee = if is_static {
        usage.used_jsxs = true;
        "_jsxs"
    } else {
        usage.used_jsx = true;
        "_jsx"
    };

    format!("{}(_Fragment, {})", callee, props_str)
}

pub(crate) fn tag_name_to_string(tag_name: &Node, source: &str) -> String {
    if let NodeData::Identifier(d) = &tag_name.data {
        if is_intrinsic_jsx_name(&d.text) {
            return format!("\"{}\"", d.text);
        }
    }
    if let NodeData::JsxNamespacedName(d) = &tag_name.data {
        return format!("\"{}:{}\"", d.namespace.text(), d.name.text());
    }
    source[tag_name.pos()..tag_name.end()].to_string()
}

pub(crate) fn attributes_to_props(
    attributes: &Node,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> (Vec<String>, Option<String>) {
    let mut props = Vec::new();
    let mut key_arg = None;

    let properties = match &attributes.data {
        NodeData::JsxAttributes(d) => &d.properties,
        _ => return (props, key_arg),
    };

    for attr in properties.iter() {
        match &attr.data {
            NodeData::JsxAttribute(d) => {
                let name = attribute_name_to_string(&d.name, source);

                if name == "key" {
                    key_arg = Some(match &d.initializer {
                        Some(init) => attribute_value_to_string(init, source, usage),
                        None => "true".to_string(),
                    });
                    continue;
                }
                let value = match &d.initializer {
                    None => "true".to_string(),
                    Some(init) => attribute_value_to_string(init, source, usage),
                };
                props.push(format!("{}: {}", name, value));
            }
            NodeData::JsxSpreadAttribute(d) => {
                let expr_text = emit_expr_with_jsx(&d.expression, source, usage);
                props.push(format!("...{}", expr_text));
            }
            _ => {}
        }
    }

    (props, key_arg)
}

pub(crate) fn attribute_name_to_string(name: &Node, source: &str) -> String {
    if let NodeData::Identifier(d) = &name.data {
        return if is_valid_identifier(&d.text) {
            d.text.clone()
        } else {
            format!("\"{}\"", d.text)
        };
    }
    if let NodeData::JsxNamespacedName(d) = &name.data {
        return format!("\"{}:{}\"", d.namespace.text(), d.name.text());
    }
    source[name.pos()..name.end()].to_string()
}

pub(crate) fn attribute_value_to_string(
    init: &Node,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> String {
    match init.kind {
        SyntaxKind::StringLiteral => source[init.pos()..init.end()].to_string(),
        SyntaxKind::JsxExpression => {
            if let NodeData::JsxExpression(d) = &init.data {
                match &d.expression {
                    Some(expr) => emit_expr_with_jsx(expr, source, usage),
                    None => "true".to_string(),
                }
            } else {
                "true".to_string()
            }
        }
        SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
            generate_jsx_call(init, source, usage)
        }
        _ => source[init.pos()..init.end()].to_string(),
    }
}

pub(crate) fn convert_children(
    children: &Arc<NodeList>,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> Option<String> {
    let semantic: Vec<Arc<Node>> = children
        .iter()
        .filter(|c| !is_whitespace_only_jsx_text(c))
        .cloned()
        .collect();

    if semantic.is_empty() {
        return None;
    }

    if semantic.len() == 1 && !is_spread_jsx_expression(&semantic[0]) {
        return Some(transform_jsx_child(&semantic[0], source, usage));
    }

    let parts: Vec<String> = semantic
        .iter()
        .map(|c| transform_jsx_child(c, source, usage))
        .collect();
    Some(format!("[{}]", parts.join(", ")))
}

pub(crate) fn is_static_children(children: &Arc<NodeList>) -> bool {
    let semantic: Vec<&Arc<Node>> = children
        .iter()
        .filter(|c| !is_whitespace_only_jsx_text(c))
        .collect();
    if semantic.len() > 1 {
        return true;
    }
    if semantic.len() == 1 {
        return is_spread_jsx_expression(semantic[0]);
    }
    false
}

pub(crate) fn transform_jsx_child(
    child: &Node,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> String {
    match child.kind {
        SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => {
            let fixed = fixup_jsx_text(child.text());
            format!("\"{}\"", escape_js_string(&fixed))
        }
        SyntaxKind::JsxExpression => {
            if let NodeData::JsxExpression(d) = &child.data {
                match &d.expression {
                    Some(expr) => emit_expr_with_jsx(expr, source, usage),
                    None => String::new(),
                }
            } else {
                String::new()
            }
        }
        SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
            generate_jsx_call(child, source, usage)
        }
        _ => source[child.pos()..child.end()].to_string(),
    }
}

pub(crate) fn emit_expr_with_jsx(node: &Node, source: &str, usage: &mut JsxRuntimeUsage) -> String {
    let start = node.pos();
    let end = node.end();

    let mut cuts: Vec<(usize, usize)> = Vec::new();
    collect_type_cuts(node, source, &mut cuts);

    let mut jsx_repls: Vec<(usize, usize, String)> = Vec::new();
    collect_nested_jsx_in_expr(node, source, &mut jsx_repls, usage);

    let cuts: Vec<(usize, usize)> = cuts
        .iter()
        .filter(|(cs, ce)| !jsx_repls.iter().any(|(js, je, _)| *cs >= *js && *ce <= *je))
        .copied()
        .collect();

    let mut ops: Vec<(usize, usize, Option<String>)> = Vec::new();
    for &(cs, ce) in &cuts {
        if ce > start && cs < end {
            ops.push((cs.max(start), ce.min(end), None));
        }
    }
    for (rs, re, text) in &jsx_repls {
        if *re > start && *rs < end {
            ops.push(((*rs).max(start), (*re).min(end), Some(text.clone())));
        }
    }

    if ops.is_empty() {
        return source[start..end].to_string();
    }

    ops.sort_by_key(|(s, _, _)| *s);

    let mut result = String::new();
    let mut pos = start;
    for (s, e, repl) in &ops {
        if *s > pos {
            result.push_str(&source[pos..*s]);
        }
        if let Some(r) = repl {
            result.push_str(r);
        }
        pos = *e;
    }
    if pos < end {
        result.push_str(&source[pos..end]);
    }
    result
}

pub(crate) fn collect_nested_jsx_in_expr(
    node: &Node,
    source: &str,
    repls: &mut Vec<(usize, usize, String)>,
    usage: &mut JsxRuntimeUsage,
) {
    crate::ast::node_data_generated::for_each_child(node, |child| {
        match child.kind {
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                let text = generate_jsx_call(child, source, usage);
                repls.push((child.pos(), child.end(), text));
            }
            _ => {
                collect_nested_jsx_in_expr(child, source, repls, usage);
            }
        }
        false
    });
}

pub(crate) fn is_intrinsic_jsx_name(text: &str) -> bool {
    !text
        .bytes()
        .next()
        .map_or(false, |c| c.is_ascii_uppercase())
}

pub(crate) fn is_valid_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

pub(crate) fn is_whitespace_only_jsx_text(node: &Node) -> bool {
    matches!(&node.data, NodeData::JsxText(d) if d.contains_only_trivia_white_spaces)
}

pub(crate) fn is_spread_jsx_expression(node: &Node) -> bool {
    matches!(&node.data, NodeData::JsxExpression(d) if d.dot_dot_dot_token.is_some())
}
