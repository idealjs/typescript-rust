use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, NodeList, SyntaxKind};
use crate::emitter::commonjs::*;

use super::*;

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
