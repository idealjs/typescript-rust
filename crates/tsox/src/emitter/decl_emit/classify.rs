use std::sync::Arc;

use super::*;
use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::tspath::{self};

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
