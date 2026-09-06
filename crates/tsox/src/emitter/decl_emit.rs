use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::node_flags::ModifierFlags;
use crate::ast::{Node, NodeList, SourceFile, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::tspath::{self};
use super::*;
use super::text_ranges::*;
use super::commonjs::*;
use super::text_transform::*;

pub(crate) fn emit_declaration_text(source_file: &SourceFile, _options: &CompilerOptions) -> String {
    let source = &source_file.text;
    let statements = match &source_file.node.data {
        NodeData::SourceFile(d) => &d.statements,
        _ => return source.clone(),
    };

    let mut output = String::new();
    let mut prev_end = 0usize;

    for stmt in statements.iter() {

        if !is_declaration_statement(stmt) {
            prev_end = stmt.end();
            continue;
        }

        if is_value_only_import(stmt) {
            prev_end = stmt.end();
            continue;
        }

        let export_cuts = collect_export_modifier_cuts(stmt, source);
        let has_export = export_cuts.iter().any(|(s, e)| *e > *s);
        let has_default = stmt
            .modifiers()
            .map(|m| m.modifier_flags.contains(ModifierFlags::Default))
            .unwrap_or(false);

        let mod_start = export_cuts.first().map(|&(s, _)| s).unwrap_or(stmt.pos());

        let content_start = if !export_cuts.is_empty() {
            export_cuts.last().map(|&(_, e)| e).unwrap_or(stmt.pos())
        } else {
            stmt.pos()
        };

        if mod_start > prev_end {
            output.push_str(&source[prev_end..mod_start]);
        }

        if has_export {
            output.push_str("export ");
        }
        if has_default {
            output.push_str("default ");
        }

        if needs_declare_keyword(stmt) && !has_default {
            output.push_str("declare ");
        }

        emit_declaration_statement(stmt, source, content_start, &mut output);
        prev_end = stmt.end();
    }

    if prev_end < source.len() {
        output.push_str(&source[prev_end..]);
    }

    let output = rewrite_import_extensions(&output);
    let output = reindent_and_dedup(&output);
    add_implicit_semicolons(&output)
}

pub(crate) fn is_declaration_statement(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ModuleDeclaration
    )
}

pub(crate) fn is_value_only_import(node: &Node) -> bool {
    if let NodeData::ImportDeclaration(d) = &node.data {
        match &d.import_clause {

            None => false,
            Some(clause) => match &clause.data {
                NodeData::ImportClause(ic) => ic.phase_modifier != Some(SyntaxKind::TypeKeyword),
                _ => false,
            },
        }
    } else {
        false
    }
}

pub(crate) fn needs_declare_keyword(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
    )
}

pub(crate) fn emit_declaration_statement(node: &Node, source: &str, start: usize, output: &mut String) {
    match &node.data {

        NodeData::FunctionDeclaration(d) => {
            if let Some(body) = &d.body {

                let sig = &source[start..body.pos()];
                let mut sig_trimmed = sig.trim_end().to_string();

                let is_generator = d.asterisk_token.is_some();
                if is_generator {
                    sig_trimmed = sig_trimmed.replace("function*", "function");
                }
                if sig_trimmed.starts_with("async ")
                    || sig_trimmed.contains(" async function")
                {
                    if let Some(pos) = sig_trimmed.find("async ") {
                        sig_trimmed.replace_range(pos..pos + 6, "");
                    }
                }

                let has_return_type = sig_trimmed.rfind(')').map_or(false, |close_paren| {
                    sig_trimmed[close_paren..].contains(':')
                });

                if !has_return_type && function_returns_jsx(body) {

                    output.push_str(&sig_trimmed);
                    output.push_str(": import(\"react\").JSX.Element;");
                } else if !has_return_type {

                    output.push_str(&sig_trimmed);
                    output.push_str(if is_generator { ": {};" } else { ": unknown;" });
                } else {
                    output.push_str(&sig_trimmed);
                    output.push(';');
                }

                let bytes = source.as_bytes();
                let mut brace_end = body.end();
                while brace_end > body.pos() && bytes[brace_end - 1].is_ascii_whitespace() {
                    brace_end -= 1;
                }
                if brace_end < body.end() {
                    output.push_str(&source[brace_end..body.end()]);
                }
            } else {

                output.push_str(&source[start..node.end()]);
            }
        }

        NodeData::VariableStatement(d) => {
            let mut cuts: Vec<(usize, usize)> = Vec::new();
            collect_variable_initializer_cuts(&d.declaration_list, &mut cuts, true);
            if cuts.is_empty() {
                output.push_str(&source[start..node.end()]);
            } else {
                emit_with_cuts(source, start, node.end(), &cuts, output);
            }
        }

        NodeData::ClassDeclaration(d) => {
            emit_class_members(&d.members, source, start, node.end(), output);
        }

        _ => {
            output.push_str(&source[start..node.end()]);
        }
    }
}

pub(crate) fn function_returns_jsx(body: &Arc<Node>) -> bool {
    fn returns_jsx_recursive(node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ReturnStatement(d) => {
                if let Some(expr) = &d.expression {
                    if is_jsx_expression(expr) {
                        return true;
                    }
                }
                false
            }
            NodeData::Block(d) => d.statements.iter().any(returns_jsx_recursive),
            NodeData::IfStatement(d) => {
                let then_jsx = returns_jsx_recursive(&d.then_statement);
                let else_jsx = d
                    .else_statement
                    .as_ref()
                    .map_or(false, |s| returns_jsx_recursive(s));
                then_jsx || else_jsx
            }
            _ => false,
        }
    }
    returns_jsx_recursive(body)
}

pub(crate) fn is_jsx_expression(node: &Arc<Node>) -> bool {
    if matches!(
        node.kind,
        SyntaxKind::JsxElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxExpression
    ) {
        return true;
    }

    if let NodeData::ParenthesizedExpression(d) = &node.data {
        return is_jsx_expression(&d.expression);
    }
    false
}

pub(crate) fn emit_class_members(
    members: &NodeList,
    source: &str,
    start: usize,
    end: usize,
    output: &mut String,
) {

    let mut ops: Vec<(usize, usize)> = Vec::new();
    let bytes = source.as_bytes();
    for member in members.iter() {
        if let Some(body) = class_member_body(member) {

            let mut sig_end = body.pos();
            while sig_end > start && bytes[sig_end - 1].is_ascii_whitespace() {
                sig_end -= 1;
            }
            ops.push((sig_end, body.end()));
        }
    }
    ops.sort_by_key(|&(s, _)| s);
    let mut pos = start;
    for (cs, ce) in &ops {
        if *cs > pos {
            output.push_str(&source[pos..*cs]);
        }
        output.push(';');
        pos = *ce;
    }
    if pos < end {
        output.push_str(&source[pos..end]);
    }
}

pub(crate) fn class_member_body(member: &Node) -> Option<&Arc<Node>> {
    match &member.data {
        NodeData::MethodDeclaration(d) => d.body.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
        _ => None,
    }
}

pub(crate) fn collect_variable_initializer_cuts(
    list: &Arc<Node>,
    cuts: &mut Vec<(usize, usize)>,
    declaration_mode: bool,
) {
    if let NodeData::VariableDeclarationList(d) = &list.data {
        for decl in d.declarations.iter() {
            if let NodeData::VariableDeclaration(vd) = &decl.data {
                if let (Some(type_node), Some(init)) = (&vd.type_node, &vd.initializer) {

                    cuts.push((type_node.end(), init.end()));
                } else if declaration_mode {
                    if let Some(init) = &vd.initializer {

                        cuts.push((vd.name.end(), init.end()));
                    }
                }

                collect_variable_initializer_cuts(&vd.name, cuts, declaration_mode);
            }
        }
    }
}

pub(crate) fn emit_with_cuts(
    source: &str,
    start: usize,
    end: usize,
    cuts: &[(usize, usize)],
    output: &mut String,
) {
    if cuts.is_empty() {
        output.push_str(&source[start..end]);
        return;
    }
    let mut sorted: Vec<(usize, usize)> = cuts
        .iter()
        .filter(|&&(cs, ce)| ce > start && cs < end)
        .map(|&(cs, ce)| (cs.max(start), ce.min(end)))
        .collect();
    sorted.sort_by_key(|&(s, _)| s);
    let mut pos = start;
    for (cs, ce) in &sorted {
        if *cs > pos {
            output.push_str(&source[pos..*cs]);
        }
        pos = *ce;
    }
    if pos < end {
        output.push_str(&source[pos..end]);
    }
}

pub(crate) fn get_dts_output_path(
    source_file: &SourceFile,
    options: &CompilerOptions,
    common_source_directory: &str,
) -> String {
    let file_name = &source_file.file_name;
    let dts_ext = get_declaration_extension(file_name);

    if !options.declaration_dir.is_empty() {
        let common_dir = if common_source_directory.is_empty() {
            compute_common_source_directory(options)
        } else {
            common_source_directory.to_string()
        };
        let path_in_new_dir =
            get_source_file_path_in_new_dir(file_name, &options.declaration_dir, &common_dir);
        let without_ext = tspath::remove_file_extension(&path_in_new_dir);
        format!("{without_ext}{dts_ext}")
    } else if !options.out_dir.is_empty() {

        let js_path = get_js_output_path(source_file, options, common_source_directory);
        let without_ext = tspath::remove_file_extension(&js_path);
        format!("{without_ext}{dts_ext}")
    } else {
        let without_ext = tspath::remove_file_extension(file_name);
        format!("{without_ext}{dts_ext}")
    }
}

pub(crate) fn get_declaration_extension(file_name: &str) -> &'static str {
    if tspath::file_extension_is_one_of(file_name, &[".mts", ".mjs"]) {
        return ".d.mts";
    }
    if tspath::file_extension_is_one_of(file_name, &[".cts", ".cjs"]) {
        return ".d.cts";
    }
    ".d.ts"
}
