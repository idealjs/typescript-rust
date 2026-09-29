#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SyntaxKind};

use super::m5y_pc::{
    new_pseudo_type_no_result, new_pseudo_type_union, pseudo_type_undefined, PseudoType,
    PseudoTypeKind,
};
use super::m5y_pc_2::{is_undefined_pseudo_type, type_node_could_refer_to_undefined, PseudoChecker};

pub fn new_pseudo_checker(strict_null_checks: bool, exact_optional_property_types: bool) -> PseudoChecker {
    PseudoChecker {
        strict_null_checks,
        exact_optional_property_types,
    }
}

impl PseudoChecker {
    pub fn get_return_type_of_signature(&self, signature_node: &Arc<Node>) -> Option<PseudoType> {
        match signature_node.kind {
            SyntaxKind::GetAccessor => Some(self.type_from_accessor(signature_node)),
            SyntaxKind::MethodDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::JSDocSignature => Some(self.create_return_from_signature(signature_node)),
            _ => None,
        }
    }

    pub fn get_type_of_accessor(&self, accessor: &Arc<Node>) -> PseudoType {
        self.type_from_accessor(accessor)
    }

    pub fn get_type_of_expression(&self, node: &Arc<Node>) -> PseudoType {
        self.type_from_expression(node)
    }

    pub fn get_type_of_declaration(&self, node: &Arc<Node>) -> Option<PseudoType> {
        match node.kind {
            SyntaxKind::Parameter => Some(self.type_from_parameter(node)),
            SyntaxKind::VariableDeclaration => Some(self.type_from_variable(node)),
            SyntaxKind::PropertySignature
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::JSDocPropertyTag => Some(self.type_from_property(node)),
            SyntaxKind::BindingElement => Some(new_pseudo_type_no_result(Arc::clone(node))),
            SyntaxKind::ExportAssignment => {
                let Some(expr) = node.expression() else {
                    return None;
                };
                Some(self.type_from_expression(expr))
            }
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::BinaryExpression => Some(self.type_from_expando_property(node)),
            SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
                Some(self.type_from_property_assignment(node))
            }
            SyntaxKind::CallExpression => {
                match ast::mig::m3e_4::get_assignment_declaration_kind(node) {
                    ast::mig::m3e_4::JsDeclarationKind::ObjectDefinePropertyValue
                    | ast::mig::m3e_4::JsDeclarationKind::ObjectDefinePropertyExports => {
                        // Go: TODO，两分支为空后统一返回 NoResult
                    }
                    _ => {}
                }
                Some(new_pseudo_type_no_result(Arc::clone(node)))
            }
            _ => None,
        }
    }
}

pub fn is_in_const_context(node: &Arc<Node>) -> bool {
    let mut current = node.parent();
    let mut maybe_assertion: Option<Arc<Node>> = None;
    while let Some(n) = current {
        let stop = ast::is_assertion_expression(&n) || !is_const_context_propagating_kind(n.kind);
        if stop {
            maybe_assertion = Some(n);
            break;
        }
        current = n.parent();
    }
    maybe_assertion
        .map(|n| ast::mig::m3f_4::is_const_assertion(&n))
        .unwrap_or(false)
}

pub fn is_const_context_propagating_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SpreadElement
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::TemplateSpan
            | SyntaxKind::PrefixUnaryExpression
    )
}

pub fn could_already_refer_to_undefined_type(t: &PseudoType) -> bool {
    if t.kind == PseudoTypeKind::NoResult
        || t.kind == PseudoTypeKind::Inferred
        || is_undefined_pseudo_type(t)
    {
        return true;
    }
    if t.kind == PseudoTypeKind::MaybeConstLocation {
        return t
            .as_pseudo_type_maybe_const_location()
            .and_then(|mc| mc.regular_type.as_deref())
            .is_some_and(could_already_refer_to_undefined_type);
    }
    if t.kind == PseudoTypeKind::Direct {
        return t
            .as_pseudo_type_direct()
            .is_some_and(|d| type_node_could_refer_to_undefined(&d.type_node));
    }
    if t.kind == PseudoTypeKind::Union {
        return t
            .as_pseudo_type_union()
            .is_some_and(|u| u.types.iter().any(could_already_refer_to_undefined_type));
    }
    false
}

pub fn add_undefined_if_definitely_required(expr: PseudoType) -> PseudoType {
    if could_already_refer_to_undefined_type(&expr) {
        return expr;
    }
    new_pseudo_type_union(vec![expr, pseudo_type_undefined()])
}
