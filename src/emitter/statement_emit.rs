use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, NodeList, SourceFile, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::core::compiler_options::JsxEmit;
use crate::tspath::{self};
use super::*;
use super::commonjs::*;

pub(crate) fn emit_statement<S: EmitSink>(
    node: &Node,
    source: &str,
    comment_cuts: &[(usize, usize)],
    replacements: &[(usize, usize, &str, Option<usize>)],
    sink: &mut S,
) {
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    collect_type_cuts(node, source, &mut cuts);

    if !comment_cuts.is_empty() {
        for &(cs, ce) in comment_cuts {
            if ce > node.pos() && cs < node.end() {
                cuts.push((cs, ce));
            }
        }
    }

    let mut stmt_replacements: Vec<(usize, usize, &str, Option<usize>)> = Vec::new();
    for &(rs, re, repl, src_pos) in replacements {
        if re > node.pos() && rs < node.end() {
            stmt_replacements.push((rs, re, repl, src_pos));
        }
    }

    if cuts.is_empty() && stmt_replacements.is_empty() {

        sink.emit_source(source, node.pos(), node.end());
        return;
    }

    let mut ops: Vec<(usize, usize, Option<(&str, Option<usize>)>)> = Vec::new();
    for (cs, ce) in &cuts {
        if *ce > node.pos() && *cs < node.end() {
            ops.push(((*cs).max(node.pos()), (*ce).min(node.end()), None));
        }
    }
    for (rs, re, repl, src_pos) in &stmt_replacements {
        ops.push((*rs, *re, Some((*repl, *src_pos))));
    }
    ops.sort_by_key(|&(s, _, _)| s);

    let mut pos = node.pos();
    for (s, e, repl) in &ops {
        if *s > pos {
            sink.emit_source(source, pos, *s);
        }
        if let Some((r, src_pos)) = repl {
            if let Some(sp) = src_pos {
                sink.emit_source_mapped(r, *sp);
            } else {
                sink.emit_generated(r);
            }
        }
        pos = *e;
    }
    if pos < node.end() {
        sink.emit_source(source, pos, node.end());
    }
}

pub(crate) fn collect_type_cuts(node: &Node, source: &str, cuts: &mut Vec<(usize, usize)>) {

    match node.kind {
        SyntaxKind::JsxElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxAttributes
        | SyntaxKind::JsxAttribute
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::JsxClosingElement
        | SyntaxKind::JsxExpression
        | SyntaxKind::JsxText
        | SyntaxKind::JsxTextAllWhiteSpaces
        | SyntaxKind::JsxOpeningFragment
        | SyntaxKind::JsxClosingFragment
        | SyntaxKind::JsxNamespacedName => return,
        _ => {}
    }

    collect_modifier_cuts(node, source, cuts);
    match &node.data {
        NodeData::VariableDeclaration(d) => {

            if let Some(type_node) = &d.type_node {
                cuts.push((d.name.end(), type_node.end()));
            }

            collect_type_cuts(&d.name, source, cuts);
            if let Some(init) = &d.initializer {
                collect_type_cuts(init, source, cuts);
            }
        }
        NodeData::ParameterDeclaration(d) => {

            if let Some(type_node) = &d.type_node {
                cuts.push((d.name.end(), type_node.end()));
            }

            if let Some(q) = &d.question_token {
                cuts.push((d.name.end(), q.end()));
            }

            if let Some(init) = &d.initializer {
                collect_type_cuts(init, source, cuts);
            }
        }
        NodeData::VariableDeclarationList(d) => {
            for decl in d.declarations.iter() {
                collect_type_cuts(decl, source, cuts);
            }
        }
        NodeData::VariableStatement(d) => {
            collect_type_cuts(&d.declaration_list, source, cuts);
        }
        NodeData::FunctionDeclaration(d) => {

            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }

            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }

            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }

            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::FunctionExpression(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            collect_type_cuts(&d.body, source, cuts);
        }
        NodeData::ArrowFunction(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            collect_type_cuts(&d.body, source, cuts);
        }
        NodeData::ClassDeclaration(d) => {

            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }

            cut_implements_clauses(d.heritage_clauses.as_deref(), source, cuts);

            for member in d.members.iter() {
                collect_type_cuts(member, source, cuts);
            }
        }
        NodeData::ClassExpression(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            cut_implements_clauses(d.heritage_clauses.as_deref(), source, cuts);
            for member in d.members.iter() {
                collect_type_cuts(member, source, cuts);
            }
        }
        NodeData::MethodDeclaration(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::ConstructorDeclaration(d) => {
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::GetAccessorDeclaration(d) => {
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::SetAccessorDeclaration(d) => {
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::PropertyDeclaration(d) => {

            if let Some(type_node) = &d.type_node {
                cuts.push((d.name.end(), type_node.end()));
            }
        }
        NodeData::ImportDeclaration(d) => {

            if let Some(clause) = &d.import_clause {
                collect_import_clause_type_cuts(clause, source, cuts);
            }
        }
        NodeData::AsExpression(d) => {

            cuts.push((d.expression.end(), d.type_node.end()));
        }
        NodeData::TypeAssertion(d) => {

            cuts.push((node.pos(), d.expression.pos()));
            collect_type_cuts(&d.expression, source, cuts);
        }
        NodeData::SatisfiesExpression(d) => {

            cuts.push((d.expression.end(), d.type_node.end()));
        }
        NodeData::NonNullExpression(d) => {

            cuts.push((d.expression.end(), node.end()));
            collect_type_cuts(&d.expression, source, cuts);
        }
        NodeData::ExpressionStatement(d) => {
            collect_type_cuts(&d.expression, source, cuts);
        }
        NodeData::ReturnStatement(d) => {
            if let Some(expr) = &d.expression {
                collect_type_cuts(expr, source, cuts);
            }
        }
        NodeData::Block(d) => {
            for stmt in d.statements.iter() {
                if !is_type_only_statement(stmt) {
                    collect_type_cuts(stmt, source, cuts);
                }
            }
        }
        NodeData::IfStatement(d) => {
            collect_type_cuts(&d.expression, source, cuts);
            collect_type_cuts(&d.then_statement, source, cuts);
            if let Some(else_stmt) = &d.else_statement {
                collect_type_cuts(else_stmt, source, cuts);
            }
        }
        NodeData::ForStatement(d) => {
            if let Some(init) = &d.initializer {
                collect_type_cuts(init, source, cuts);
            }
            if let Some(cond) = &d.condition {
                collect_type_cuts(cond, source, cuts);
            }
            if let Some(incr) = &d.incrementor {
                collect_type_cuts(incr, source, cuts);
            }
            collect_type_cuts(&d.statement, source, cuts);
        }
        NodeData::ForInOrOfStatement(d) => {
            collect_type_cuts(&d.initializer, source, cuts);
            collect_type_cuts(&d.expression, source, cuts);
            collect_type_cuts(&d.statement, source, cuts);
        }
        NodeData::WhileStatement(d) => {
            collect_type_cuts(&d.expression, source, cuts);
            collect_type_cuts(&d.statement, source, cuts);
        }
        NodeData::DoStatement(d) => {
            collect_type_cuts(&d.statement, source, cuts);
            collect_type_cuts(&d.expression, source, cuts);
        }

        _ => {
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collect_type_cuts(child, source, cuts);
                false
            });
        }
    }
}

pub(crate) fn collect_modifier_cuts(node: &Node, source: &str, cuts: &mut Vec<(usize, usize)>) {
    let modifiers = node.modifier_nodes();
    if modifiers.is_empty() {
        return;
    }
    let bytes = source.as_bytes();
    for mod_node in modifiers {
        if matches!(
            mod_node.kind,
            SyntaxKind::AbstractKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::ReadonlyKeyword
        ) {
            let start = mod_node.pos();
            let mut end = mod_node.end();

            while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
                end += 1;
            }
            cuts.push((start, end));
        }
    }
}

pub(crate) fn cut_implements_clauses(
    heritage_clauses: Option<&NodeList>,
    source: &str,
    cuts: &mut Vec<(usize, usize)>,
) {
    let clauses = match heritage_clauses {
        Some(c) => c,
        None => return,
    };
    let bytes = source.as_bytes();
    for hc in clauses.iter() {
        if let NodeData::HeritageClause(hcd) = &hc.data {
            if hcd.token == SyntaxKind::ImplementsKeyword {
                let mut start = hc.pos();
                while start > 0 && (bytes[start - 1] == b' ' || bytes[start - 1] == b'\t') {
                    start -= 1;
                }
                cuts.push((start, hc.end()));
            }
        }
    }
}

pub(crate) fn collect_import_clause_type_cuts(clause: &Node, source: &str, cuts: &mut Vec<(usize, usize)>) {
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
