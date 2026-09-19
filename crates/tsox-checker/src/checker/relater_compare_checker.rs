#![allow(unused_imports)]

use crate::checker::relater_compare::*;

impl Checker {
    pub fn compare_types_identical(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_identical_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_assignable_simple(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        if self.is_type_assignable_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_assignable_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _report_errors: bool,
    ) -> Ternary {
        if self.is_type_assignable_to(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn compare_types_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_subtype_of(source, target) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn check_type_assignable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        _head_message: Option<&tsox_core::diagnostics::Message>,
    ) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_assignable_to_ex(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        _head_message: Option<&tsox_core::diagnostics::Message>,
        _diagnostic_output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_comparable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        _head_message: Option<&tsox_core::diagnostics::Message>,
    ) -> bool {
        self.is_type_comparable_to(source, target)
    }

    pub fn check_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        _error_node: Option<&Arc<tsox_frontend::ast::Node>>,
    ) -> bool {
        self.is_type_related_to(source, target, relation)
    }

    pub(crate) fn elaborate_error(
        &mut self,
        expr: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        self.elaborate_error_with_head(expr, source, target, relation, None, out)
    }

    pub(crate) fn elaborate_error_with_head(
        &mut self,
        expr: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        head_message: Option<&tsox_core::diagnostics::Message>,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        // Go elaborateError：泛型条件目标不细化
        if self.is_or_has_generic_conditional(target) {
            return false;
        }
        if self.elaborate_did_you_mean_to_call_or_construct(
            expr,
            source,
            target,
            relation,
            crate::checker::types::SignatureKind::Construct,
            head_message,
            out.as_deref_mut(),
        ) || self.elaborate_did_you_mean_to_call_or_construct(
            expr,
            source,
            target,
            relation,
            crate::checker::types::SignatureKind::Call,
            head_message,
            out.as_deref_mut(),
        ) {
            return true;
        }
        match expr.kind {
            tsox_frontend::ast::SyntaxKind::ParenthesizedExpression => {
                let inner = match &expr.data {
                    tsox_frontend::ast::NodeData::ParenthesizedExpression(d) => {
                        Arc::clone(&d.expression)
                    }
                    _ => return false,
                };
                self.elaborate_error(&inner, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::AsExpression => {
                let is_const_assertion = match &expr.data {
                    tsox_frontend::ast::NodeData::AsExpression(d) => {
                        d.type_node.kind == tsox_frontend::ast::SyntaxKind::TypeReference
                            && matches!(&d.type_node.data,
                                tsox_frontend::ast::NodeData::TypeReferenceNode(tr)
                                    if tr.type_name.text() == "const")
                    }
                    _ => false,
                };
                if is_const_assertion {
                    let inner = match &expr.data {
                        tsox_frontend::ast::NodeData::AsExpression(d) => {
                            Arc::clone(&d.expression)
                        }
                        _ => return false,
                    };
                    return self.elaborate_error(&inner, source, target, relation, out);
                }
                false
            }
            tsox_frontend::ast::SyntaxKind::JsxExpression => {
                let inner = match &expr.data {
                    tsox_frontend::ast::NodeData::JsxExpression(d) => d.expression.clone(),
                    _ => None,
                };
                match inner {
                    Some(inner) => self.elaborate_error(&inner, source, target, relation, out),
                    None => false,
                }
            }
            tsox_frontend::ast::SyntaxKind::BinaryExpression => {
                let inner = match &expr.data {
                    tsox_frontend::ast::NodeData::BinaryExpression(d) => {
                        if matches!(
                            d.operator_token.kind,
                            tsox_frontend::ast::SyntaxKind::EqualsToken
                                | tsox_frontend::ast::SyntaxKind::CommaToken
                        ) {
                            Some(Arc::clone(&d.right))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                match inner {
                    Some(inner) => self.elaborate_error(&inner, source, target, relation, out),
                    None => false,
                }
            }
            tsox_frontend::ast::SyntaxKind::ObjectLiteralExpression => {
                self.elaborate_object_literal(expr, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::ArrayLiteralExpression => {
                self.elaborate_array_literal(expr, source, target, relation, out)
            }
            tsox_frontend::ast::SyntaxKind::ArrowFunction => {
                self.elaborate_arrow_function(expr, source, target, relation, out)
            }
            _ => false,
        }
    }

    pub(crate) fn elaborate_object_literal(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        if target.flags.intersects(
            TypeFlags::String
                | TypeFlags::Number
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::Void
                | TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Never
                | TypeFlags::Enum
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BooleanLiteral,
        ) {
            return false;
        }
        let properties = match &node.data {
            tsox_frontend::ast::NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => return false,
        };
        let mut reported = false;
        for prop in properties.iter() {
            if prop.kind == tsox_frontend::ast::SyntaxKind::SpreadAssignment {
                continue;
            }
            let (name_node, initializer): (
                &Arc<tsox_frontend::ast::Node>,
                Option<Arc<tsox_frontend::ast::Node>>,
            ) = match &prop.data {
                tsox_frontend::ast::NodeData::PropertyAssignment(d) => {
                    (&d.name, Some(Arc::clone(&d.initializer)))
                }
                tsox_frontend::ast::NodeData::ShorthandPropertyAssignment(d) => (&d.name, None),
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => (&d.name, None),
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => (&d.name, None),
                tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => (&d.name, None),
                _ => continue,
            };
            let name = self.get_property_name_from_node(name_node);
            if name.is_empty() {
                continue;
            }
            let Some(target_prop_type) = self.get_type_of_property_of_type(target, &name) else {
                continue;
            };
            let Some(source_prop_type) = self.get_type_of_property_of_type(source, &name) else {
                continue;
            };
            if self.is_type_related_to(&source_prop_type, &target_prop_type, relation) {
                continue;
            }
            if let Some(init) = initializer
                && self.elaborate_error(
                    &init,
                    &source_prop_type,
                    &target_prop_type,
                    relation,
                    out.as_deref_mut(),
                )
            {
                reported = true;
                continue;
            }

            match out.as_deref_mut() {
                Some(o) => {
                    self.check_type_related_to_and_optionally_elaborate(
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        Some(name_node),
                        None,
                        None,
                        Some(o),
                    );
                }
                None => {
                    self.check_type_related_to_and_optionally_elaborate(
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        Some(name_node),
                        None,
                        None,
                        None,
                    );
                }
            }
            reported = true;
        }
        reported
    }
}
