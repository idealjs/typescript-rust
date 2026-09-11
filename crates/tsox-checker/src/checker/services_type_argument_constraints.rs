use std::sync::Arc;

use super::*;
use tsox_frontend::ast::NodeData;


pub(super) fn type_arguments_of(node: &Arc<Node>) -> Option<Vec<Arc<Node>>> {
    let list = match &node.data {
        NodeData::CallExpression(d) => d.type_arguments.clone(),
        NodeData::NewExpression(d) => d.type_arguments.clone(),
        NodeData::TaggedTemplateExpression(d) => d.type_arguments.clone(),
        NodeData::TypeReferenceNode(d) => d.type_arguments.clone(),
        NodeData::JsxOpeningElement(d) => d.type_arguments.clone(),
        NodeData::JsxSelfClosingElement(d) => d.type_arguments.clone(),
        NodeData::ExpressionWithTypeArguments(d) => d.type_arguments.clone(),
        _ => None,
    }?;
    Some(list.iter().cloned().collect())
}

pub(super) fn callee_expression_of(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::CallExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::NewExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::TaggedTemplateExpression(d) => Some(Arc::clone(&d.tag)),
        NodeData::JsxOpeningElement(d) => Some(Arc::clone(&d.tag_name)),
        NodeData::JsxSelfClosingElement(d) => Some(Arc::clone(&d.tag_name)),
        NodeData::ExpressionWithTypeArguments(d) => Some(Arc::clone(&d.expression)),
        NodeData::Decorator(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub(super) fn single_constraint(checker: &mut Checker, constraints: Vec<Arc<Type>>) -> Option<Arc<Type>> {
    if constraints.is_empty() {
        return None;
    }
    Some(checker.get_union_type(constraints))
}

pub(super) fn declaration_type_parameters(
    node: &Arc<Node>,
) -> Option<Arc<tsox_frontend::ast::NodeList>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.type_parameters.clone(),
        NodeData::FunctionExpression(d) => d.type_parameters.clone(),
        NodeData::ArrowFunction(d) => d.type_parameters.clone(),
        NodeData::MethodDeclaration(d) => d.type_parameters.clone(),
        NodeData::InterfaceDeclaration(d) => d.type_parameters.clone(),
        NodeData::ClassDeclaration(d) => d.type_parameters.clone(),
        NodeData::ClassExpression(d) => d.type_parameters.clone(),
        NodeData::TypeAliasDeclaration(d) => d.type_parameters.clone(),
        _ => None,
    }
}

pub(super) fn type_parameter_nodes_of_type_reference(
    checker: &mut Checker,
    type_ref: &Arc<Node>,
) -> Vec<Arc<Node>> {
    let NodeData::TypeReferenceNode(d) = &type_ref.data else {
        return Vec::new();
    };
    let Some(sym) = checker.resolve_identifier(&d.type_name) else {
        return Vec::new();
    };
    let base = checker.resolve_alias_base(sym);
    for decl in &base.declarations {
        if let Some(params) = declaration_type_parameters(decl) {
            return params.iter().cloned().collect();
        }
    }
    Vec::new()
}

pub(super) fn constraint_type_of_type_parameter_node(
    checker: &mut Checker,
    tp: &Arc<Node>,
) -> Option<Arc<Type>> {
    let NodeData::TypeParameterDeclaration(d) = &tp.data else {
        return None;
    };
    let constraint = d.constraint.clone()?;
    Some(checker.get_type_from_type_node(&constraint))
}

pub(super) fn type_argument_constraint(
    checker: &mut Checker,
    node: &Arc<Node>,
) -> Option<Arc<Type>> {
        let parent = node.parent.clone()?;
        let args = type_arguments_of(&parent)?;
        let position = args.iter().position(|a| Arc::ptr_eq(a, node))?;
        if let Some(callee) = callee_expression_of(&parent) {
            let mut callee = callee;
            while callee.kind == SyntaxKind::ParenthesizedExpression
                && let NodeData::ParenthesizedExpression(d) = &callee.data
            {
                callee = Arc::clone(&d.expression);
            }
            let t = if callee.kind == SyntaxKind::ClassExpression {
                checker.get_type_of_class_declaration(&callee)
            } else {
                checker.get_type_of_node(&callee)
            };
            let mut constraints: Vec<Arc<Type>> = Vec::new();
            for kind in [SignatureKind::Call, SignatureKind::Construct] {
                for sig in checker.get_signatures_of_type(&t, kind) {
                    if let Some(tp) = sig.type_parameters.get(position)
                        && let Some(c) = checker.get_constraint_of_type_parameter(tp)
                    {
                        constraints.push(c);
                    }
                }
            }
            if let Some(c) = single_constraint(checker, constraints) {
                return Some(c);
            }
            return None;
        }
        if parent.kind == SyntaxKind::TypeReference {
            let tps = type_parameter_nodes_of_type_reference(checker, &parent);
            let tp = tps.get(position)?;
            return constraint_type_of_type_parameter_node(checker, tp);
        }
        None
    }
