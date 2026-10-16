#![allow(unused_imports)]

use crate::checker::flow_narrow_binary::*;

impl Checker {
    pub(crate) fn narrow_by_binary(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        depth: u32,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_binary");
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return Arc::clone(type_);
        };
        let op = bin.operator_token.kind;

        if op == SyntaxKind::InstanceOfKeyword {
            return self.narrow_type_by_instanceof(type_, expr, target, kind);
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
                    let t = self.narrow_by_binary(type_, &bin.left, target, NarrowKind::TrueBranch, depth);
                    self.narrow_by_binary(&t, &bin.right, target, NarrowKind::TrueBranch, depth)
                }
                NarrowKind::FalseBranch if is_and => {
                    let a =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::FalseBranch, depth);
                    let b =
                        self.narrow_by_binary(type_, &bin.right, target, NarrowKind::FalseBranch, depth);
                    self.flow_union_of(&[a, b])
                }
                NarrowKind::TrueBranch => {
                    let a = self.narrow_by_binary(type_, &bin.left, target, NarrowKind::TrueBranch, depth);
                    let b =
                        self.narrow_by_binary(type_, &bin.right, target, NarrowKind::TrueBranch, depth);
                    self.flow_union_of(&[a, b])
                }
                NarrowKind::FalseBranch => {
                    let t =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::FalseBranch, depth);
                    self.narrow_by_binary(&t, &bin.right, target, NarrowKind::FalseBranch, depth)
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

        // Go narrowTypeByTypeof（flow.go:640-643）：typeof 链根收窄
        if bin.left.kind == SyntaxKind::TypeOfExpression
            && tsox_frontend::ast::is_string_literal_like(&bin.right)
            && let Some(narrowed) =
                self.try_narrow_by_typeof_chain_root(type_, &bin.left, &bin.right, target, narrow_to_value)
        {
            return narrowed;
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression
            && tsox_frontend::ast::is_string_literal_like(&bin.left)
            && let Some(narrowed) =
                self.try_narrow_by_typeof_chain_root(type_, &bin.right, &bin.left, target, narrow_to_value)
        {
            return narrowed;
        }

        // Go flow.go:506-512：匹配引用先于链根收窄
        let matched_target =
            self.expr_matches_target(&bin.left, target) || self.expr_matches_target(&bin.right, target);

        // Go flow.go:514-521：链根包含收窄在判别属性之前
        if !matched_target && self.strict_null_checks {
            if self.optional_chain_contains_target(&bin.left, target) {
                return self.narrow_by_optional_chain_containment(type_, op, &bin.right, kind);
            }
            if self.optional_chain_contains_target(&bin.right, target) {
                return self.narrow_by_optional_chain_containment(type_, op, &bin.left, kind);
            }
        }

        if bin.left.kind == SyntaxKind::TypeOfExpression
            && tsox_frontend::ast::is_string_literal_like(&bin.right)
        {
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
        if bin.right.kind == SyntaxKind::TypeOfExpression
            && tsox_frontend::ast::is_string_literal_like(&bin.left)
        {
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

        let (value_node, is_symbol_on_left) = if self.expr_matches_target(&bin.left, target) {
            (&bin.right, true)
        } else if self.expr_matches_target(&bin.right, target) {
            (&bin.left, false)
        } else {
            if tsox_frontend::ast::is_boolean_literal(&bin.right)
                && !tsox_frontend::ast::is_access_expression(&bin.left)
            {
                return self.narrow_by_boolean_comparison(
                    type_,
                    &bin.left,
                    &bin.right,
                    op,
                    kind,
                    target,
                    depth,
                );
            }
            if tsox_frontend::ast::is_boolean_literal(&bin.left)
                && !tsox_frontend::ast::is_access_expression(&bin.right)
            {
                return self.narrow_by_boolean_comparison(
                    type_,
                    &bin.right,
                    &bin.left,
                    op,
                    kind,
                    target,
                    depth,
                );
            }
            return Arc::clone(type_);
        };
        let _ = is_symbol_on_left;

        let value_type = self.get_type_of_node(value_node);
        self.narrow_by_equality(type_, &value_type, narrow_to_value, is_loose)
    }

    pub(crate) fn try_narrow_by_typeof_chain_root(
        &mut self,
        type_: &Arc<Type>,
        typeof_expr: &Arc<Node>,
        literal_node: &Arc<Node>,
        target: &FlowRef,
        narrow_to_value: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("try_narrow_by_typeof_chain_root");
        let NodeData::TypeOfExpression(typeof_data) = &typeof_expr.data else {
            return None;
        };
        let literal_text: &str = match &literal_node.data {
            NodeData::StringLiteral(literal) => literal.text.as_str(),
            NodeData::NoSubstitutionTemplateLiteral(literal) => literal.text.as_str(),
            _ => return None,
        };
        if self.typeof_expr_matches_target(typeof_expr, target) {
            return None;
        }
        if self.strict_null_checks
            && self.optional_chain_contains_target(&typeof_data.expression, target)
            && narrow_to_value == (literal_text != "undefined")
        {
            return Some(self.get_non_nullable_type(type_));
        }
        None
    }

    pub(crate) fn narrow_by_boolean_comparison(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        bool_value: &Arc<Node>,
        operator: SyntaxKind,
        kind: NarrowKind,
        target: &FlowRef,
        depth: u32,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_boolean_comparison");
        let assume_true = ((kind == NarrowKind::TrueBranch)
            != (bool_value.kind == SyntaxKind::TrueKeyword))
            != (operator != SyntaxKind::ExclamationEqualsEqualsToken
                && operator != SyntaxKind::ExclamationEqualsToken);
        let narrowed_kind = if assume_true {
            NarrowKind::TrueBranch
        } else {
            NarrowKind::FalseBranch
        };
        self.narrow_by_expression(type_, expr, target, narrowed_kind, depth)
    }

    pub(crate) fn narrow_by_equality(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_equality"); 
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

    pub(crate) fn filter_comparable_or_coercible(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        is_loose: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_comparable_or_coercible"); 
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

    pub(crate) fn remove_comparable_units(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_comparable_units"); 
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
}
