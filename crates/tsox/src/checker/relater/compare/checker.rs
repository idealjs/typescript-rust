#![allow(unused_imports)]

use super::*;

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
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
    ) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_assignable_to_ex(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
        _diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn check_type_comparable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _error_node: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
    ) -> bool {
        self.is_type_comparable_to(source, target)
    }

    pub fn check_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        _error_node: Option<&Arc<crate::ast::Node>>,
    ) -> bool {
        self.is_type_related_to(source, target, relation)
    }

    pub(crate) fn elaborate_error(
        &mut self,
        expr: &Arc<crate::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        out: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        match expr.kind {
            crate::ast::SyntaxKind::ParenthesizedExpression => {
                let inner = match &expr.data {
                    crate::ast::NodeData::ParenthesizedExpression(d) => Arc::clone(&d.expression),
                    _ => return false,
                };
                self.elaborate_error(&inner, source, target, relation, out)
            }
            crate::ast::SyntaxKind::ObjectLiteralExpression => {
                self.elaborate_object_literal(expr, source, target, relation, out)
            }
            crate::ast::SyntaxKind::ArrayLiteralExpression => {
                self.elaborate_array_literal(expr, source, target, relation, out)
            }
            _ => false,
        }
    }

    pub(crate) fn elaborate_object_literal(
        &mut self,
        node: &Arc<crate::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<crate::ast::Diagnostic>>,
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
            crate::ast::NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => return false,
        };
        let mut reported = false;
        for prop in properties.iter() {
            if prop.kind == crate::ast::SyntaxKind::SpreadAssignment {
                continue;
            }
            let (name_node, initializer): (&Arc<crate::ast::Node>, Option<Arc<crate::ast::Node>>) =
                match &prop.data {
                    crate::ast::NodeData::PropertyAssignment(d) => {
                        (&d.name, Some(Arc::clone(&d.initializer)))
                    }
                    crate::ast::NodeData::ShorthandPropertyAssignment(d) => (&d.name, None),
                    crate::ast::NodeData::MethodDeclaration(d) => (&d.name, None),
                    crate::ast::NodeData::GetAccessorDeclaration(d) => (&d.name, None),
                    crate::ast::NodeData::SetAccessorDeclaration(d) => (&d.name, None),
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
