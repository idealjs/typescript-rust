//! .types 基线发射器：对照 Go tsc/internal/testutil/tsbaseline/
//! type_symbol_baseline.go 的 types 通道（节点遍历/标注过滤/类型渲染/
//! 源码交织/CRLF）。env TSOX_TYPES_EMIT_DIR 存在时由 corpus worker
//! 逐例生成 <stem>.types，供 tools/types_csv.py 汇总为
//! corpus_rust_types.csv 与 Go 侧对照。
#![allow(dead_code)]

use std::sync::Arc;
use tsox_checker::checker::checker_get_excluded_symbol_flags::is_declaration_name;
use tsox_checker::checker::mig::m1f::r24k9_defs::is_expression_node;
use tsox_checker::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use tsox_checker::checker::types_type_id::TypeFlags;
use tsox_checker::checker::Checker;
use tsox_frontend::ast::mig::m3g_3::is_part_of_type_node;
use tsox_frontend::ast::mig::w3::for_each_child_and_js_doc;
use tsox_frontend::ast::node_data_generated::node_name;
use tsox_frontend::ast::{Node, SourceFile, SyntaxKind};

struct Row {
    line: usize,
    source_text: String,
    typ: String,
}

fn collect_nodes(root: &Arc<Node>, sf: &SourceFile) -> Vec<Arc<Node>> {
    let mut order = Vec::new();
    let mut work = vec![Arc::clone(root)];
    while let Some(elem) = work.pop() {
        order.push(Arc::clone(&elem));
        let mut children = Vec::new();
        for_each_child_and_js_doc(&elem, sf, &mut |c| children.push(Arc::clone(c)));
        children.reverse();
        work.extend(children);
    }
    order
}

fn is_alias_declaration_name(node: &Arc<Node>) -> bool {
    match node.parent() {
        Some(p) if p.kind == SyntaxKind::TypeAliasDeclaration => node_name(&p)
            .map(|n| Arc::ptr_eq(&n, node))
            .unwrap_or(false),
        _ => false,
    }
}

fn write_type_line(checker: &mut Checker, node: &Arc<Node>, file_text: &str) -> Option<Row> {
    if is_part_of_type_node(node) && !is_alias_declaration_name(node) {
        return None;
    }
    if let Some(p) = node.parent() {
        if matches!(p.kind, SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeReference) {
            return None;
        }
    }
    if node.kind == SyntaxKind::OmittedExpression {
        return None;
    }

    if is_alias_declaration_name(node) {
        let name = node.text().to_string();
        let pos = (node.loc.pos as usize).min(file_text.len());
        let line = file_text[..pos].matches('\n').count();
        return Some(Row { line, source_text: name.clone(), typ: name });
    }

    let own = checker.get_type_at_location(node);
    let t = match node.parent() {
        Some(p)
            if p.kind == SyntaxKind::ExpressionWithTypeArguments
                && p.parent().is_some_and(|g| g.kind == SyntaxKind::HeritageClause) =>
        {
            let parent_ty = checker.get_type_at_location(&p);
            if parent_ty.flags.intersects(TypeFlags::Any) { own } else { parent_ty }
        }
        _ => own,
    };
    let mut t = t;
    if t.flags.intersects(TypeFlags::Any) && node.kind == SyntaxKind::Identifier {
        if let Some(sym) = checker.get_symbol_at_location(node) {
            let sym_ty = checker.get_type_of_symbol(&sym);
            if !sym_ty.flags.intersects(TypeFlags::Any) {
                t = sym_ty;
            }
        }
    }

    let type_string = checker.type_to_string_ex(
        &t,
        TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE.union(TypeFormatFlags::NO_TRUNCATION),
    );
    let pos = (node.loc.pos as usize).min(file_text.len());
    let end = (node.loc.end as usize).min(file_text.len());
    let raw: String = if end > pos { file_text[pos..end].to_string() } else { String::new() };
    let source_text = raw.replace(['\n', '\r'], "");
    let line = file_text[..pos].matches('\n').count();
    Some(Row { line, source_text, typ: type_string })
}

fn weave_unit(unit_name: &str, content: &str, rows: &[Row]) -> String {
    let mut out = String::new();
    out.push_str("=== ");
    out.push_str(unit_name);
    out.push_str(" ===\r\n");
    let code_lines: Vec<&str> = content.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let bracket_or_blank = |l: &str| { let t = l.trim(); t.is_empty() || t == "{" || t == "}" };
    let mut last: isize = -1;
    for r in rows {
        let cur = r.line as isize;
        if last == -1 {
            if (cur as usize) < code_lines.len() {
                out.push_str(&code_lines[..cur as usize + 1].join("\r\n"));
                out.push_str("\r\n");
            }
        } else if last != cur && (cur as usize) < code_lines.len() {
            if (last + 1) < code_lines.len() as isize
                && !bracket_or_blank(code_lines[(last + 1) as usize])
            {
                out.push_str("\r\n");
            }
            let from = (last + 1) as usize;
            if from <= cur as usize {
                out.push_str(&code_lines[from..cur as usize + 1].join("\r\n"));
                out.push_str("\r\n");
            }
        }
        last = cur;
        out.push('>');
        out.push_str(&r.source_text);
        out.push_str(" : ");
        out.push_str(&r.typ);
        out.push_str("\r\n");
    }
    if last + 1 < code_lines.len() as isize {
        if !bracket_or_blank(code_lines[(last + 1) as usize]) {
            out.push_str("\r\n");
        }
        out.push_str(&code_lines[(last + 1) as usize..].join("\r\n"));
    }
    out.push_str("\r\n");
    out
}

/// 生成整例 .types 基线文本；units 为 (unit 名, unit 源内容, SourceFile)。
pub fn generate(
    checker: &mut Checker,
    header: &str,
    units: &[(String, String, Arc<SourceFile>)],
) -> String {
    let mut sections = Vec::new();
    for (name, content, file) in units {
        let nodes = collect_nodes(&file.node, file);
        let mut rows = Vec::new();
        for n in &nodes {
            if is_expression_node(n) || n.kind == SyntaxKind::Identifier || is_declaration_name(n)
            {
                if let Some(r) = write_type_line(checker, n, &file.text) {
                    rows.push(r);
                }
            }
        }
        rows.sort_by_key(|r| r.line);
        sections.push(weave_unit(name, content, &rows));
    }
    if sections.is_empty() {
        return String::new();
    }
    format!("//// [{header}] ////\r\n\r\n{}", sections.join(""))
}
