use std::sync::Arc;

use crate::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::*;

use super::FlowRef;

use super::NarrowKind;

impl Checker {
    pub(crate) fn narrow_by_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        depth: u32,
    ) -> Arc<Type> {
        if expr.kind == SyntaxKind::ParenthesizedExpression {
            if let NodeData::ParenthesizedExpression(p) = &expr.data {
                return self.narrow_by_expression(type_, &p.expression, target, kind, depth);
            }
        }

        if expr.kind == SyntaxKind::BinaryExpression {
            if let NodeData::BinaryExpression(bin) = &expr.data {
                if bin.operator_token.kind == SyntaxKind::AmpersandAmpersandToken {
                    if kind == NarrowKind::TrueBranch {
                        let narrowed =
                            self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                        return self
                            .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                    }

                    if kind == NarrowKind::FalseBranch {
                        let a_false = self.narrow_by_expression(
                            type_,
                            &bin.left,
                            target,
                            NarrowKind::FalseBranch,
                            depth,
                        );
                        let a_true = self.narrow_by_expression(
                            type_,
                            &bin.left,
                            target,
                            NarrowKind::TrueBranch,
                            depth,
                        );
                        let b_false = self.narrow_by_expression(
                            &a_true,
                            &bin.right,
                            target,
                            NarrowKind::FalseBranch,
                            depth,
                        );
                        return self.flow_union_of(&[a_false, b_false]);
                    }

                    let narrowed = self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                    return self.narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                }
                if bin.operator_token.kind == SyntaxKind::BarBarToken {
                    if kind == NarrowKind::FalseBranch {
                        let narrowed =
                            self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                        return self
                            .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                    }

                    let a_true = self.narrow_by_expression(
                        type_,
                        &bin.left,
                        target,
                        NarrowKind::TrueBranch,
                        depth,
                    );
                    let a_false = self.narrow_by_expression(
                        type_,
                        &bin.left,
                        target,
                        NarrowKind::FalseBranch,
                        depth,
                    );
                    let b_true = self.narrow_by_expression(
                        &a_false,
                        &bin.right,
                        target,
                        NarrowKind::TrueBranch,
                        depth,
                    );
                    return self.flow_union_of(&[a_true, b_true]);
                }
                if bin.operator_token.kind == SyntaxKind::QuestionQuestionToken {
                    if kind == NarrowKind::TrueBranch {
                        return Arc::clone(type_);
                    }

                    let narrowed =
                        self.narrow_by_optionality(type_, &bin.left, target, kind, depth);
                    return self.narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                }
            }
        }

        if expr.kind == SyntaxKind::PrefixUnaryExpression {
            if let NodeData::PrefixUnaryExpression(unary) = &expr.data {
                if unary.operator == SyntaxKind::ExclamationToken {
                    let inverted = if kind == NarrowKind::TrueBranch {
                        NarrowKind::FalseBranch
                    } else {
                        NarrowKind::TrueBranch
                    };
                    return self.narrow_by_expression(
                        type_,
                        &unary.operand,
                        target,
                        inverted,
                        depth,
                    );
                }
            }
        }

        if expr.kind == SyntaxKind::BinaryExpression {
            return self.narrow_by_binary(type_, expr, target, kind);
        }

        if expr.kind == SyntaxKind::CallExpression {
            return self.narrow_by_call_expression(type_, expr, target, kind);
        }

        if expr.kind == SyntaxKind::Identifier
            && !self.expr_matches_target(expr, target)
            && self.flow_inline_level < 5
        {
            if let Some(init_expr) = self.const_alias_initializer(expr) {
                self.flow_inline_level += 1;
                let result = self.narrow_by_expression(type_, &init_expr, target, kind, depth);
                self.flow_inline_level -= 1;
                return result;
            }
        }

        if self.expr_matches_target(expr, target) {
            return self.narrow_by_truthiness(type_, kind);
        }

        if kind == NarrowKind::TrueBranch {
            let contains = self.optional_chain_contains_target(expr, target);
            if contains {
                return self.remove_nullable_from_union(type_);
            }
        }

        if let Some(name) = self.discriminant_property_name_on_target(expr, target) {
            return self.narrow_by_property_truthiness(type_, &name, kind);
        }

        Arc::clone(type_)
    }

    fn discriminant_property_name_on_target(
        &self,
        expr: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<String> {
        match &expr.data {
            NodeData::PropertyAccessExpression(pa) => {
                if self.expr_matches_target(&pa.expression, target) {
                    Some(pa.name.text().to_string())
                } else {
                    None
                }
            }
            NodeData::ElementAccessExpression(ea) => {
                if self.expr_matches_target(&ea.expression, target) {
                    match &ea.argument_expression.data {
                        NodeData::StringLiteral(s) => Some(s.text.clone()),
                        NodeData::NumericLiteral(n) => Some(n.text.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn narrow_by_property_truthiness(
        &mut self,
        type_: &Arc<Type>,
        name: &str,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let constituents = match type_.flags.contains(TypeFlags::Union) {
            true => match &type_.data {
                TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => return Arc::clone(type_),
            },
            false => return Arc::clone(type_),
        };
        let mut kept: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
        for c in &constituents {
            let prop_type = match self.get_constituent_property(c, name) {
                Some(sym) => self.get_type_of_symbol(&sym),
                None => {
                    kept.push(Arc::clone(c));
                    continue;
                }
            };
            let undecidable = prop_type.flags.intersects(
                TypeFlags::Any
                    | TypeFlags::Unknown
                    | TypeFlags::TypeParameter
                    | TypeFlags::Conditional
                    | TypeFlags::IndexedAccess,
            );
            if undecidable {
                kept.push(Arc::clone(c));
                continue;
            }
            let parts: Vec<Arc<Type>> = if prop_type.flags.contains(TypeFlags::Union) {
                prop_type.types().unwrap_or(&[]).to_vec()
            } else {
                vec![Arc::clone(&prop_type)]
            };
            let any_falsy = parts
                .iter()
                .any(|p| self.constituent_is_definitely_falsy(p));
            let all_falsy = parts
                .iter()
                .all(|p| self.constituent_is_definitely_falsy(p));
            match kind {
                NarrowKind::FalseBranch if any_falsy => kept.push(Arc::clone(c)),

                NarrowKind::TrueBranch if !all_falsy => kept.push(Arc::clone(c)),
                _ => {}
            }
        }
        if kept.is_empty() || kept.len() == constituents.len() {
            return Arc::clone(type_);
        }
        self.flow_union_of(&kept)
    }
}
