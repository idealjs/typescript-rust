use crate::ast::*;
use crate::parser::{for_each_child, Parser, ParserDiagnostic};
use std::sync::Arc;
use tsox_core::core::text::TextRange;

// Go reparseTopLevelAwait 的等价：模块文件确认后，对含 await 标识符的顶层
// 语句以 await 上下文重解析（`await(1)` 等 CallExpression 形态转 AwaitExpression）。
// 检测改为解析后 AST 遍历：await 标识符出现在引用位即算，函数体内部不算
// （Go parseFunctionDeclarationOrMember 对 statementHasAwaitIdentifier 的 save/restore）。
pub(crate) fn reparse_top_level_await(file: &mut SourceFile, diagnostics: &mut Vec<ParserDiagnostic>) {
    if file.is_declaration_file || file.external_module_indicator.is_none() {
        return;
    }

    let statements: Vec<Arc<Node>> = match &file.node.data {
        NodeData::SourceFile(d) => d.statements.nodes.clone(),
        _ => return,
    };

    let detected: Vec<usize> = statements
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            !s.flags.contains(NodeFlags::AwaitContext) && statement_has_await_identifier(s)
        })
        .map(|(i, _)| i)
        .collect();
    if detected.is_empty() {
        return;
    }

    let mut spans: Vec<(usize, usize)> = Vec::new();
    for i in detected {
        match spans.last_mut() {
            Some(last) if last.1 == i => last.1 = i + 1,
            _ => spans.push((i, i + 1)),
        }
    }

    let mut new_statements: Vec<Arc<Node>> = Vec::with_capacity(statements.len());
    let mut idx = 0;
    let mut changed = false;

    for (s, e) in spans {
        new_statements.extend(statements[idx..s].iter().cloned());
        idx = e;

        let span_start = statements[s].pos();
        let span_end = statements[e - 1].end();
        let (reparsed, new_diags) =
            reparse_span(&file.text, file.language_variant, file.script_kind, span_start, span_end);
        if reparsed.is_empty() {
            new_statements.extend(statements[s..e].iter().cloned());
            continue;
        }
        changed = true;
        diagnostics.retain(|d| !(d.range.pos() >= span_start && d.range.pos() < span_end));
        diagnostics.extend(new_diags);
        new_statements.extend(reparsed);
    }
    new_statements.extend(statements[idx..].iter().cloned());

    if !changed {
        return;
    }

    let (end_of_file_token, old_loc, old_flags) = match &file.node.data {
        NodeData::SourceFile(d) => (d.end_of_file_token.clone(), file.node.loc, file.node.flags),
        _ => return,
    };
    let new_list = Arc::new(NodeList {
        loc: TextRange::new(
            old_loc.pos(),
            new_statements
                .last()
                .map(|s| s.end())
                .unwrap_or(old_loc.pos()),
        ),
        nodes: new_statements,
    });
    let new_node = Arc::new(Node::with_loc_flags(
        SyntaxKind::SourceFile,
        NodeData::SourceFile(SourceFileData {
            statements: new_list,
            end_of_file_token,
        }),
        old_loc,
        old_flags,
    ));
    file.node = new_node;
    file.parse_error_spans = diagnostics.iter().map(|d| d.range).collect();
    file.has_parse_diagnostics = !diagnostics.is_empty();
}

fn statement_has_await_identifier(stmt: &Arc<Node>) -> bool {
    visit_for_await(stmt)
}

fn visit_for_await(node: &Arc<Node>) -> bool {
    if matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    ) {
        return false;
    }
    if node.kind == SyntaxKind::Identifier && node.text() == "await" {
        return true;
    }
    for_each_child(node, visit_for_await)
}

fn reparse_span(
    text: &str,
    language_variant: LanguageVariant,
    script_kind: ScriptKind,
    span_start: usize,
    span_end: usize,
) -> (Vec<Arc<Node>>, Vec<ParserDiagnostic>) {
    let mut parser = Parser::new_with_language_variant(text.to_string(), language_variant);
    parser.set_javascript_file(matches!(script_kind, ScriptKind::Js | ScriptKind::Jsx));
    parser.scanner.set_range(span_start, text.len());
    parser.next_token();
    parser.await_context = true;

    let mut parsed: Vec<Arc<Node>> = Vec::new();
    let mut guard = 0;
    while parser.token != SyntaxKind::EndOfFile && guard < 1000 {
        guard += 1;
        let full_start = parser.scanner.full_start_pos();
        let stmt = parser.parse_statement();
        let covered = stmt.end() >= span_end;
        parsed.push(stmt);
        if covered {
            break;
        }
        if parser.scanner.full_start_pos() == full_start {
            parser.next_token();
        }
    }
    parser.drain_scanner_errors();
    (parsed, parser.diagnostics)
}
