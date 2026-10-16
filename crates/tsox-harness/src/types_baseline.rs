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
use tsox_compile::compiler::Program;
use tsox_frontend::ast::mig::m3e_4::{get_meaning_from_declaration, SemanticMeaning};
use tsox_frontend::ast::mig::m3g_3::is_part_of_type_node;
use tsox_frontend::ast::for_each_child;
use tsox_frontend::ast::node_data_generated::node_name;
use tsox_frontend::ast::{Node, NodeData, NodeFlags, SourceFile, SyntaxKind};

struct Row {
    line: usize,
    source_text: String,
    typ: String,
}

/// Go removeTestPathPrefixes（tsc/internal/testutil/tsbaseline/util.go:25-51）
/// retainTrailingDirectorySeparator=false 的替换单（testPathPrefixReplacer）。
/// strings.Replacer 语义：按出现位置单趟扫描，同位置按参数序取先命中者，
/// 输出不回扫。
fn remove_test_path_prefixes(text: &str) -> String {
    const PATTERNS: [(&str, &str); 7] = [
        ("/.ts/", ""),
        ("/.lib/", ""),
        ("/.src/", ""),
        ("bundled:///libs/", ""),
        ("file:///./ts/", "file:///"),
        ("file:///./lib/", "file:///"),
        ("file:///./src/", "file:///"),
    ];
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < text.len() {
        let mut hit: Option<(usize, &'static str)> = None;
        for (pat, rep) in PATTERNS {
            let longer = match hit {
                Some((l, _)) => pat.len() > l,
                None => true,
            };
            if bytes[i..].starts_with(pat.as_bytes()) && longer {
                hit = Some((pat.len(), rep));
            }
        }
        match hit {
            Some((len, rep)) => {
                out.push_str(rep);
                i += len;
            }
            None => {
                let ch_len = utf8_len(bytes[i]);
                out.push_str(&text[i..i + ch_len]);
                i += ch_len;
            }
        }
    }
    out
}

fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

fn as_satisfies_expression(p: &Arc<Node>) -> Option<Arc<Node>> {
    match &p.data {
        NodeData::AsExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::SatisfiesExpression(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

fn collect_nodes(root: &Arc<Node>, _sf: &SourceFile) -> Vec<Arc<Node>> {
    // Go forEachASTNode（type_symbol_baseline.go:330-356）以 elem.ForEachChild
    // 下钻——生成的各 kind visitor（ast_generated.go）不含 JSDoc 子节点，
    // JSDoc 注释/tag 不产生发射行；Reparsed 节点整体不进遍历（不发射、
    // 不下钻，其子树随宿主一起缺席），唯 As/SatisfiesExpression 本身
    // （344-345）及其 expression 子节点（345 尾子句）例外，内层发射再以
    // 「父级为 As/Satisfies」收窄（346）
    let mut order = Vec::new();
    let mut work = vec![Arc::clone(root)];
    while let Some(elem) = work.pop() {
        let reparsed = elem.flags.contains(NodeFlags::Reparsed);
        let parent = elem.parent();
        let parent_is_as_satisfies = parent.as_ref().is_some_and(|p| {
            matches!(
                p.kind,
                SyntaxKind::AsExpression | SyntaxKind::SatisfiesExpression
            )
        });
        let is_parent_expression = parent
            .as_ref()
            .and_then(|p| as_satisfies_expression(p))
            .is_some_and(|e| Arc::ptr_eq(&e, &elem));
        let visit = !reparsed
            || matches!(
                elem.kind,
                SyntaxKind::AsExpression | SyntaxKind::SatisfiesExpression
            )
            || is_parent_expression;
        if visit {
            if !reparsed || parent_is_as_satisfies {
                order.push(Arc::clone(&elem));
            }
            let mut children = Vec::new();
            for_each_child(&elem, |c| {
                children.push(Arc::clone(c));
                false
            });
            children.reverse();
            work.extend(children);
        }
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
    // Go writeTypeOrSymbol（tsc/internal/testutil/tsbaseline/type_symbol_baseline.go:372-382）：
    // identifier 的声明宿主无 value meaning 不发射（如 type A<T> 的 T，宿主
    // TypeParameter 的 meaning 为 Type），唯类型别名自己的名字例外
    if is_part_of_type_node(node) {
        return None;
    }
    if node.kind == SyntaxKind::Identifier {
        let value_meaning = node
            .parent()
            .is_some_and(|p| get_meaning_from_declaration(&p).contains(SemanticMeaning::VALUE));
        if !value_meaning && !is_alias_declaration_name(node) {
            return None;
        }
    }
    if let Some(p) = node.parent() {
        if matches!(p.kind, SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeReference) {
            return None;
        }
    }
    if node.kind == SyntaxKind::OmittedExpression {
        return None;
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

    let type_flags =
        TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE.union(TypeFormatFlags::NO_TRUNCATION);
    // Go type_symbol_baseline.go:407-409：TypeToTypeNode 以 node.Parent 为
    // enclosingDeclaration，限定名按其可及性判定
    //（nodebuilderimpl.go:1110/1127-1135）——命名空间内短名、顶层带前缀
    let mut type_string = checker.type_to_string_ex_with_enclosing(
        &t,
        node.parent().as_ref(),
        type_flags,
    );
    // Go type_symbol_baseline.go:410-414：对同一 t 建 typeNode，仅当其本身是裸
    // Identifier（IsIdentifier(typeNode)）且文本与别名名相同时才以 InTypeAlias
    // 重渲染（nodebuilderimpl.go:3405 InTypeAlias 单层跳过 alias gate）。alias
    // gate 呈现的别名名经 symbolToTypeNode 包成 TypeReference
    //（nodebuilderimpl.go:3552-3563）——非 identifier，复合型（union/intersection
    // 等）按别名名呈现时保持首渲染结果
    if is_alias_declaration_name(node) {
        let type_node = checker.type_to_type_node(&t);
        if type_node.kind == SyntaxKind::Identifier && type_node.text() == node.text() {
            type_string = checker.type_to_string_ex_with_enclosing(
                &t,
                node.parent().as_ref(),
                type_flags.union(TypeFormatFlags::IN_TYPE_ALIAS),
            );
        }
    }
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
    program: &Arc<Program>,
    header: &str,
    units: &[(String, String, Arc<SourceFile>)],
) -> String {
    let mut sections = Vec::new();
    for (name, content, file) in units {
        // Go typeWriterWalker.getTypeCheckerForCurrentFile（type_symbol_baseline.go:285-289）：
        // 按 GetTypeCheckerForFile 取该文件关联的 checker——发射查询命中检查期
        // 暖缓存（typeNodeLinks 等），不存在冷实例重推导
        let mut checker = program.get_type_checker_for_file(file);
        let nodes = collect_nodes(&file.node, file);
        let mut rows = Vec::new();
        for n in &nodes {
            if is_expression_node(n) || n.kind == SyntaxKind::Identifier || is_declaration_name(n)
            {
                if let Some(r) = write_type_line(&mut checker, n, &file.text) {
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
    // Go iterateBaseline（type_symbol_baseline.go:262）逐 section 过
    // removeTestPathPrefixes（/.lib/ 等测试路径前缀抹除，基线行
    // /.lib/react16.d.ts → react16.d.ts），header 在替换之后拼接
    let sections = sections
        .into_iter()
        .map(|s| remove_test_path_prefixes(&s))
        .collect::<Vec<_>>();
    format!("//// [{header}] ////\r\n\r\n{}", sections.join(""))
}
