//! Go completions.go getJSDocParameterCompletions 移植：JSDoc 标签位附加
//! 按函数形参预填的 @param 条目（JS 内从初始化器推类型、解构形参展开子标签）。

use std::sync::Arc;

use tsox_checker::checker::types::TypeFlags;
use tsox_checker::checker::Checker;
use tsox_frontend::ast::node_data_generated::for_each_child;
use tsox_frontend::ast::{Node, NodeData, SourceFile, SyntaxKind};

/// Go getJSDocParameterCompletions：doc 注释尾部之后首个 function-like 的
/// 形参逐一生成预填条目；已标注的（tag 位在光标前的 identifier 名 @param）
/// 跳过；解构形参仅在未标注位展开为多条。tag_name_only 剥离前导 @
pub(crate) fn jsdoc_parameter_completions(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    doc_start: usize,
    doc_end: usize,
    tags: &[Arc<Node>],
    position: usize,
    tag_name_only: bool,
) -> Vec<String> {
    // Go jsDoc.Parent：JSDoc 经 HasJSDoc 挂在其宿主声明上，取挂了本
    // doc 的 function-like
    let Some(fun) = jsdoc_host_function_like(file, doc_start, doc_end) else {
        if std::env::var("TSOX_DEBUG_JSDOC").is_ok() {
            eprintln!("[jsdoc-params] no function-like hosting [{doc_start},{doc_end})");
        }
        return Vec::new();
    };
    if std::env::var("TSOX_DEBUG_JSDOC").is_ok() {
        eprintln!("[jsdoc-params] fun={:?} pos={}", fun.kind, fun.pos());
    }
    let is_js = file.file_name.ends_with(".js") || file.file_name.ends_with(".jsx");
    let param_tag_count = tags
        .iter()
        .filter(|tag| {
            tag.kind == SyntaxKind::JSDocParameterTag
                && tag.pos() < position
                && tag_param_name_identifier(tag)
        })
        .count();
    let Some(params) = function_like_parameters(&fun) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (param_index, param) in params.nodes.iter().enumerate() {
        if param_index < param_tag_count {
            continue;
        }
        let NodeData::ParameterDeclaration(d) = &param.data else {
            continue;
        };
        let label = if d.name.kind == SyntaxKind::Identifier {
            jsdoc_param_annotation(
                checker,
                file,
                d.name.text().to_string(),
                d.initializer.clone(),
                d.dot_dot_dot_token.is_some(),
                is_js,
                false,
            )
        } else if param_index == param_tag_count {
            let path = format!("param{param_index}");
            jsdoc_param_tags_for_destructuring(
                checker,
                file,
                path,
                &d.name,
                d.initializer.clone(),
                d.dot_dot_dot_token.is_some(),
                is_js,
            )
            .join("\n* ")
        } else {
            continue;
        };
        let label = if tag_name_only {
            label.strip_prefix('@').unwrap_or(&label).to_string()
        } else {
            label
        };
        out.push(label);
    }
    out
}

fn jsdoc_param_tags_for_destructuring(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    path: String,
    pattern: &Arc<Node>,
    initializer: Option<Arc<Node>>,
    dot_dot_dot: bool,
    is_js: bool,
) -> Vec<String> {
    if !is_js {
        return vec![jsdoc_param_annotation(
            checker, file, path, initializer, dot_dot_dot, is_js, false,
        )];
    }
    jsdoc_param_pattern_worker(checker, file, path, pattern, initializer, dot_dot_dot, is_js)
}

fn jsdoc_param_pattern_worker(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    path: String,
    pattern: &Arc<Node>,
    initializer: Option<Arc<Node>>,
    dot_dot_dot: bool,
    is_js: bool,
) -> Vec<String> {
    if pattern.kind == SyntaxKind::ObjectBindingPattern && !dot_dot_dot {
        if let NodeData::BindingPattern(d) = &pattern.data {
            let root = jsdoc_param_annotation(
                checker, file, path.clone(), initializer.clone(), dot_dot_dot, is_js, true,
            );
            let mut child_tags = Vec::new();
            for element in &d.elements.nodes {
                let tags = jsdoc_param_element_worker(checker, file, &path, element, is_js);
                if tags.is_empty() {
                    child_tags = Vec::new();
                    break;
                }
                child_tags.extend(tags);
            }
            if !child_tags.is_empty() {
                child_tags.insert(0, root);
                return child_tags;
            }
        }
    }
    vec![jsdoc_param_annotation(
        checker, file, path, initializer, dot_dot_dot, is_js, false,
    )]
}

fn jsdoc_param_element_worker(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    path: &str,
    element: &Arc<Node>,
    is_js: bool,
) -> Vec<String> {
    let NodeData::BindingElement(d) = &element.data else {
        return Vec::new();
    };
    let Some(name) = &d.name else {
        return Vec::new();
    };
    let dot_dot_dot = d.dot_dot_dot_token.is_some();
    if name.kind == SyntaxKind::Identifier {
        let property_name = match &d.property_name {
            Some(pn) => property_name_text(pn),
            None => name.text().to_string(),
        };
        if property_name.is_empty() {
            return Vec::new();
        }
        return vec![jsdoc_param_annotation(
            checker,
            file,
            format!("{path}.{property_name}"),
            d.initializer.clone(),
            dot_dot_dot,
            is_js,
            false,
        )];
    }
    if let Some(pn) = &d.property_name {
        let property_name = property_name_text(pn);
        if property_name.is_empty() {
            return Vec::new();
        }
        return jsdoc_param_pattern_worker(
            checker,
            file,
            format!("{path}.{property_name}"),
            name,
            d.initializer.clone(),
            dot_dot_dot,
            is_js,
        );
    }
    Vec::new()
}

fn property_name_text(pn: &Arc<Node>) -> String {
    match &pn.data {
        NodeData::Identifier(d) => d.text.clone(),
        NodeData::StringLiteral(d) => d.text.clone(),
        NodeData::NumericLiteral(d) => d.text.clone(),
        _ => String::new(),
    }
}

/// Go getJSDocParamAnnotation（非 snippet 路径）：JS 文件带 {type} 段，
/// 初始化器存在时取形参声明位推断类型；TS 文件只有 @param name
fn jsdoc_param_annotation(
    checker: &mut Checker,
    file: &Arc<SourceFile>,
    name: String,
    initializer: Option<Arc<Node>>,
    dot_dot_dot: bool,
    is_js: bool,
    is_object: bool,
) -> String {
    let name = match &initializer {
        Some(init) => jsdoc_param_name_with_initializer(file, name, init),
        None => name,
    };
    if !is_js {
        return format!("@param {name} ");
    }
    let mut t = "*".to_string();
    if is_object {
        t = "object".to_string();
    } else if let Some(init) = &initializer
        && let Some(param) = init.parent()
    {
        let mut inferred = checker.get_type_of_node(&param);
        if inferred.flags.contains(TypeFlags::Any) {
            let init_t = checker.get_type_of_node(init);
            inferred = checker.get_widened_literal_type_for_initializer(&param, &init_t);
        }
        if !inferred.flags.contains(TypeFlags::Any | TypeFlags::Void) {
            t = checker.type_to_string(&inferred);
        }
    }
    let dot_dot_dot_text = if !is_object && dot_dot_dot { "..." } else { "" };
    format!("@param {{{dot_dot_dot_text}{t}}} {name} ")
}

fn jsdoc_param_name_with_initializer(
    file: &Arc<SourceFile>,
    name: String,
    initializer: &Arc<Node>,
) -> String {
    let (s, e) = (initializer.pos().min(file.text.len()), initializer.end().min(file.text.len()));
    let text = if s < e { file.text[s..e].trim() } else { "" };
    if text.contains('\n') || text.len() > 80 {
        format!("[{name}]")
    } else {
        format!("[{name}={text}]")
    }
}

fn tag_param_name_identifier(tag: &Arc<Node>) -> bool {
    matches!(&tag.data, NodeData::JSDocParameterOrPropertyTag(d) if d.name.kind == SyntaxKind::Identifier)
}

fn jsdoc_host_function_like(sf: &Arc<SourceFile>, doc_start: usize, doc_end: usize) -> Option<Arc<Node>> {
    let mut best: Option<Arc<Node>> = None;
    walk_hosts(sf, &sf.node, doc_start, doc_end, &mut best);
    best
}

fn walk_hosts(sf: &Arc<SourceFile>, n: &Arc<Node>, doc_start: usize, doc_end: usize, best: &mut Option<Arc<Node>>) {
    let dbg = std::env::var("TSOX_DEBUG_JSDOC").is_ok();
    if is_function_like_kind(n.kind) {
        let docs = tsox_frontend::parser::parse_jsdoc_for_node(sf, n);
        if dbg {
            eprintln!(
                "[jsdoc-params] cand kind={:?} pos={} end={} ndocs={} ranges={:?}",
                n.kind,
                n.pos(),
                n.end(),
                docs.len(),
                docs.iter().map(|d| (d.pos(), d.end())).collect::<Vec<_>>()
            );
        }
        if docs.iter().any(|jd| jd.pos() <= doc_start && doc_end <= jd.end()) {
            let better = best.as_ref().is_none_or(|b| b.pos() < n.pos());
            if better {
                *best = Some(Arc::clone(n));
            }
        }
    }
    for_each_child(n, |c| {
        walk_hosts(sf, c, doc_start, doc_end, best);
        false
    });
}

fn is_function_like_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
    )
}

fn function_like_parameters(fun: &Arc<Node>) -> Option<&Arc<tsox_frontend::ast::NodeList>> {
    match &fun.data {
        NodeData::FunctionDeclaration(d) => Some(&d.parameters),
        NodeData::MethodDeclaration(d) => Some(&d.parameters),
        NodeData::MethodSignatureDeclaration(d) => Some(&d.parameters),
        NodeData::ConstructorDeclaration(d) => Some(&d.parameters),
        NodeData::GetAccessorDeclaration(d) => Some(&d.parameters),
        NodeData::SetAccessorDeclaration(d) => Some(&d.parameters),
        NodeData::FunctionExpression(d) => Some(&d.parameters),
        NodeData::ArrowFunction(d) => Some(&d.parameters),
        _ => None,
    }
}
