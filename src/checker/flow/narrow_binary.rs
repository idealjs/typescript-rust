use std::sync::Arc;

use crate::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::*;


use super::FlowRef;

use super::NarrowKind;

impl Checker {
    pub(crate) fn narrow_by_binary(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return Arc::clone(type_);
        };
        let op = bin.operator_token.kind;

        if op == SyntaxKind::InstanceOfKeyword {
            return self.narrow_by_instanceof(type_, &bin.left, &bin.right, target, kind);
        }

        if op == SyntaxKind::InKeyword {
            return self.narrow_by_in_keyword(type_, &bin.left, &bin.right, target, kind);
        }

        if matches!(
            op,
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::BarBarToken
        ) {
            let is_and = op == SyntaxKind::AmpersandAmpersandToken;
            return match kind {
                NarrowKind::TrueBranch if is_and => {
                    let t =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::TrueBranch);
                    self.narrow_by_binary(&t, &bin.right, target, NarrowKind::TrueBranch)
                }
                NarrowKind::FalseBranch if is_and => {
                    let a =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::FalseBranch);
                    let b =
                        self.narrow_by_binary(type_, &bin.right, target, NarrowKind::FalseBranch);
                    self.flow_union_of(&[a, b])
                }
                NarrowKind::TrueBranch => {
                    let a =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::TrueBranch);
                    let b =
                        self.narrow_by_binary(type_, &bin.right, target, NarrowKind::TrueBranch);
                    self.flow_union_of(&[a, b])
                }
                NarrowKind::FalseBranch => {
                    let t =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::FalseBranch);
                    self.narrow_by_binary(&t, &bin.right, target, NarrowKind::FalseBranch)
                }
            };
        }

        let is_strict = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        let is_loose =
            op == SyntaxKind::EqualsEqualsToken || op == SyntaxKind::ExclamationEqualsToken;
        if !is_strict && !is_loose {
            return Arc::clone(type_);
        }

        let is_equality =
            op == SyntaxKind::EqualsEqualsEqualsToken || op == SyntaxKind::EqualsEqualsToken;

        let narrow_to_value = if is_equality {
            kind == NarrowKind::TrueBranch
        } else {
            kind == NarrowKind::FalseBranch
        };

        if bin.left.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_target(&bin.left, target)
        {
            return self.narrow_by_typeof(type_, &bin.right, narrow_to_value, is_loose);
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_target(&bin.right, target)
        {
            return self.narrow_by_typeof(type_, &bin.left, narrow_to_value, is_loose);
        }

        if bin.left.kind == SyntaxKind::TypeOfExpression {
            if let Some(narrowed) = self.try_narrow_by_typeof_discriminant(
                type_,
                &bin.left,
                &bin.right,
                target,
                narrow_to_value,
            ) {
                return narrowed;
            }
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression {
            if let Some(narrowed) = self.try_narrow_by_typeof_discriminant(
                type_,
                &bin.right,
                &bin.left,
                target,
                narrow_to_value,
            ) {
                return narrowed;
            }
        }

        if let Some(narrowed) = self.try_narrow_by_discriminant_property(type_, expr, target, kind)
        {
            return narrowed;
        }

        if self.optional_chain_contains_target(&bin.left, target) {
            return self.narrow_by_optional_chain_containment(type_, op, &bin.right, kind);
        }
        if self.optional_chain_contains_target(&bin.right, target) {
            return self.narrow_by_optional_chain_containment(type_, op, &bin.left, kind);
        }

        let (value_node, is_symbol_on_left) = if self.expr_matches_target(&bin.left, target) {
            (&bin.right, true)
        } else if self.expr_matches_target(&bin.right, target) {
            (&bin.left, false)
        } else {
            return Arc::clone(type_);
        };
        let _ = is_symbol_on_left;

        let value_type = self.get_type_of_node(value_node);
        self.narrow_by_equality(type_, &value_type, narrow_to_value, is_loose)
    }

    fn narrow_by_equality(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }

        if value_type.flags.contains(TypeFlags::BooleanLiteral)
            && type_.flags.contains(TypeFlags::Boolean)
            && !is_loose
        {
            let is_true_value = match value_type.literal_value() {
                Some(LiteralValue::Boolean(b)) => *b,
                _ => true,
            };
            let target_is_true = if narrow_to_value {
                is_true_value
            } else {
                !is_true_value
            };
            return if target_is_true {
                self.true_type()
            } else {
                self.false_type()
            };
        }

        if value_type.flags.intersects(TYPE_FLAGS_NULLABLE) {
            if !self.strict_null_checks {
                return Arc::clone(type_);
            }
            let value_is_null = value_type.flags.contains(TypeFlags::Null);

            return if is_loose {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TYPE_FLAGS_NULLABLE)
                } else {
                    self.remove_flags_from_union(type_, TYPE_FLAGS_NULLABLE)
                }
            } else if value_is_null {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TypeFlags::Null)
                } else {
                    self.remove_flags_from_union(type_, TypeFlags::Null)
                }
            } else {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TypeFlags::Undefined)
                } else {
                    self.remove_flags_from_union(type_, TypeFlags::Undefined)
                }
            };
        }
        if narrow_to_value {

            let filtered = self.filter_comparable_or_coercible(type_, value_type, is_loose);
            self.replace_primitives_with_literals(&filtered, value_type)
        } else {

            if !value_type.flags.intersects(TYPE_FLAGS_UNIT) {
                return Arc::clone(type_);
            }
            self.remove_comparable_units(type_, value_type)
        }
    }

    fn filter_comparable_or_coercible(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        is_loose: bool,
    ) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let value_constituents = self.constituent_types(value_type);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {

                let comparable = value_constituents
                    .iter()
                    .any(|vc| self.is_type_comparable_to(t, vc));
                if comparable {
                    return true;
                }

                is_loose
                    && value_constituents
                        .iter()
                        .any(|vc| Self::is_coercible_under_double_equals(t, vc))
            })
            .collect();
        self.rebuild_union_or_never(type_, matching)
    }

    fn remove_comparable_units(&mut self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let value_constituents = self.constituent_types(value_type);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {

                if !t.flags.intersects(TYPE_FLAGS_UNIT) {
                    return true;
                }

                !value_constituents
                    .iter()
                    .any(|vc| self.is_type_comparable_to(t, vc))
            })
            .collect();
        self.rebuild_union_or_never(type_, remaining)
    }

    fn replace_primitives_with_literals(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {

        let has_primitives = type_
            .flags
            .intersects(TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt);
        let has_literals = value_type
            .flags
            .intersects(TYPE_FLAGS_LITERAL | TypeFlags::TemplateLiteral | TypeFlags::StringMapping);
        if !has_primitives || !has_literals {
            return Arc::clone(type_);
        }

        let value_constituents = self.constituent_types(value_type);
        let string_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| {
                t.flags.intersects(
                    TypeFlags::StringLiteral
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                )
            })
            .cloned()
            .collect();
        let number_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::NumberLiteral))
            .cloned()
            .collect();
        let bigint_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::BigIntLiteral))
            .cloned()
            .collect();
        let constituents = self.constituent_types(type_);
        let mut result: Vec<Arc<Type>> = Vec::new();
        for t in constituents {
            if t.flags.contains(TypeFlags::String) {

                let has_string_value =
                    value_type.flags.contains(TypeFlags::String) || string_literals.is_empty();
                if has_string_value {
                    result.push(t);
                } else {
                    result.extend(string_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::Number) {
                let has_number_value =
                    value_type.flags.contains(TypeFlags::Number) || number_literals.is_empty();
                if has_number_value {
                    result.push(t);
                } else {
                    result.extend(number_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::BigInt) {
                let has_bigint_value =
                    value_type.flags.contains(TypeFlags::BigInt) || bigint_literals.is_empty();
                if has_bigint_value {
                    result.push(t);
                } else {
                    result.extend(bigint_literals.iter().cloned());
                }
            } else {
                result.push(t);
            }
        }
        self.rebuild_union_or_never(type_, result)
    }

    fn is_coercible_under_double_equals(source: &Arc<Type>, target: &Arc<Type>) -> bool {
        source
            .flags
            .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::BooleanLiteral)
            && target
                .flags
                .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::Boolean)
    }

    fn narrow_by_instanceof(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        if !self.expr_matches_target(left, target) {
            return Arc::clone(type_);
        }
        let right_type = self.get_type_of_node(right);
        let Some(instance_type) = self.get_instance_type_of_constructor(&right_type) else {
            return Arc::clone(type_);
        };
        match kind {
            NarrowKind::TrueBranch => self.narrow_to_subtype(type_, &instance_type),
            NarrowKind::FalseBranch => self.remove_subtype_from_union(type_, &instance_type),
        }
    }

    fn narrow_by_in_keyword(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }
        if !self.expr_matches_target(right, target) {
            return Arc::clone(type_);
        }
        let Some(prop_name) = Self::get_accessed_property_name_from_node(left) else {
            return Arc::clone(type_);
        };
        let keep_present = match kind {
            NarrowKind::TrueBranch => true,
            NarrowKind::FalseBranch => false,
        };

        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let has_prop = self.type_has_property(t, &prop_name);
                if keep_present {
                    !has_prop.is_definitely_not()
                } else {
                    !has_prop.is_definitely()
                }
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }
}
