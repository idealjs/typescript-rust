use std::sync::Arc;

use crate::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::*;

use super::FlowRef;

use super::NarrowKind;

impl Checker {
    pub(crate) fn narrow_by_call_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::CallExpression(call) = &expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        let assume_true = kind == NarrowKind::TrueBranch;
        for sig in &signatures {
            let Some(predicate) = self.compute_type_predicate_of_signature(sig) else {
                continue;
            };

            if predicate.kind != TypePredicateKind::Identifier
                && predicate.kind != TypePredicateKind::AssertsIdentifier
            {
                if predicate.kind == TypePredicateKind::This
                    && let Some(pred_type) = &predicate.t
                {
                    let receiver = match &call.expression.data {
                        NodeData::PropertyAccessExpression(pa) => Some(&pa.expression),
                        _ => None,
                    };
                    let Some(receiver) = receiver else {
                        continue;
                    };
                    if !self.expr_matches_target(receiver, target) {
                        continue;
                    }
                    let Some(callback_arg) = call.arguments.nodes.first() else {
                        continue;
                    };
                    let Some(u) = self.callback_predicate_type(callback_arg) else {
                        continue;
                    };

                    let instantiated = if sig.type_parameters.is_empty() {
                        Arc::clone(pred_type)
                    } else {
                        let args: Vec<Arc<Type>> =
                            sig.type_parameters.iter().map(|_| Arc::clone(&u)).collect();
                        self.substitute_infer_type_parameters(
                            pred_type,
                            &sig.type_parameters,
                            &args,
                        )
                    };
                    return self.narrow_by_type_predicate(type_, &instantiated, assume_true);
                }
                continue;
            }
            let Some(pred_type) = &predicate.t else {
                continue;
            };
            let param_idx = predicate.parameter_index as usize;
            let Some(arg) = call.arguments.nodes.get(param_idx) else {
                continue;
            };

            if !self.expr_matches_target(arg, target) {
                continue;
            }
            return self.narrow_by_type_predicate(type_, pred_type, assume_true);
        }
        Arc::clone(type_)
    }

    fn callback_predicate_type(&mut self, arg: &Arc<Node>) -> Option<Arc<Type>> {
        let arg_type = self.get_type_of_node(arg);
        let sigs = self.get_signatures_of_type(&arg_type, SignatureKind::Call);
        for sig in &sigs {
            if let Some(pred) = self.compute_type_predicate_of_signature(sig)
                && pred.kind == TypePredicateKind::Identifier
                && let Some(t) = pred.t
            {
                return Some(t);
            }
        }
        None
    }

    pub(crate) fn narrow_by_assertion_call(
        &mut self,
        type_: &Arc<Type>,
        call_expr: &Arc<Node>,
        target: &FlowRef,
    ) -> Arc<Type> {
        let NodeData::CallExpression(call) = &call_expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        for sig in &signatures {
            let Some(predicate) = self.compute_type_predicate_of_signature(sig) else {
                continue;
            };

            if predicate.kind != TypePredicateKind::AssertsIdentifier
                && predicate.kind != TypePredicateKind::AssertsThis
            {
                continue;
            }

            if predicate.kind == TypePredicateKind::AssertsThis {
                continue;
            }
            let param_idx = predicate.parameter_index as usize;
            let Some(arg) = call.arguments.nodes.get(param_idx) else {
                continue;
            };

            if !self.expr_matches_target(arg, target) {
                if let Some(narrowed) = self.narrow_by_asserted_comparison(type_, arg, target) {
                    return narrowed;
                }
                continue;
            }
            if let Some(pred_type) = &predicate.t {
                return self.intersect_or_narrow(type_, pred_type);
            }

            return self.remove_flags_from_union(type_, TYPE_FLAGS_NULLABLE);
        }
        Arc::clone(type_)
    }

    fn narrow_by_asserted_comparison(
        &mut self,
        type_: &Arc<Type>,
        arg: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> {
        let NodeData::BinaryExpression(bin) = &arg.data else {
            return None;
        };
        use crate::ast::SyntaxKind::*;
        let (cmp, target_side, literal_side) = match bin.operator_token.kind {
            ExclamationEqualsEqualsToken
            | ExclamationEqualsToken
            | EqualsEqualsEqualsToken
            | EqualsEqualsToken => {
                let l_matches = self.expr_matches_target(&bin.left, target);
                let r_matches = self.expr_matches_target(&bin.right, target);
                if l_matches {
                    (bin.operator_token.kind, &bin.left, &bin.right)
                } else if r_matches {
                    (bin.operator_token.kind, &bin.right, &bin.left)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let _ = target_side;
        let lt = self.get_type_of_node(literal_side);
        let is_eq = matches!(cmp, EqualsEqualsEqualsToken | EqualsEqualsToken);
        if is_eq {
            Some(self.intersect_or_narrow(type_, &lt))
        } else {
            Some(self.remove_type_from_union(type_, &lt))
        }
    }

    fn narrow_by_type_predicate(
        &mut self,
        type_: &Arc<Type>,
        pred_type: &Arc<Type>,
        assume_true: bool,
    ) -> Arc<Type> {
        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }
        if assume_true {
            self.intersect_or_narrow(type_, pred_type)
        } else {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !self.is_type_assignable_to(t, pred_type))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
    }

    pub(crate) fn typeof_expr_matches_target(&self, expr: &Arc<Node>, target: &FlowRef) -> bool {
        let NodeData::TypeOfExpression(typeof_data) = &expr.data else {
            return false;
        };
        self.expr_matches_target(&typeof_data.expression, target)
    }

    pub(crate) fn narrow_by_typeof(
        &mut self,
        type_: &Arc<Type>,
        type_name_node: &Arc<Node>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> {
        let type_name = match &type_name_node.data {
            NodeData::StringLiteral(data) => data.text.as_str(),
            _ => return Arc::clone(type_),
        };

        if let TypeData::Intersection(i) = &type_.data {
            let all_primitive = i
                .union_or_intersection
                .types
                .iter()
                .all(|t| t.flags.intersects(TYPE_FLAGS_PRIMITIVE));
            if !all_primitive {
                return Arc::clone(type_);
            }
        }
        let matching_flags = match type_name {
            "string" => TYPE_FLAGS_STRING_LIKE,
            "number" => TYPE_FLAGS_NUMBER_LIKE,
            "boolean" => TYPE_FLAGS_BOOLEAN_LIKE,
            "bigint" => TYPE_FLAGS_BIG_INT_LIKE,
            "symbol" => TYPE_FLAGS_ES_SYMBOL_LIKE,
            "undefined" => TypeFlags::Undefined,
            "function" => {
                return self.filter_type_by_callable(type_, narrow_to_value);
            }
            "object" => {
                if narrow_to_value {
                    return self.filter_type_by_object(type_, is_loose);
                }
                return self.remove_object_from_union(type_);
            }
            _ => return Arc::clone(type_),
        };
        if narrow_to_value {
            self.filter_type_by_flags(type_, matching_flags)
        } else {
            self.remove_flags_from_union(type_, matching_flags)
        }
    }

    pub(crate) fn narrow_by_truthiness(&self, type_: &Arc<Type>, kind: NarrowKind) -> Arc<Type> {
        match kind {
            NarrowKind::TrueBranch => {
                let falsy_flags = TypeFlags::Undefined
                    | TypeFlags::Null
                    | TypeFlags::Void
                    | TypeFlags::BooleanLiteral
                    | TypeFlags::StringLiteral
                    | TypeFlags::NumberLiteral;
                self.remove_falsy_from_union(type_, falsy_flags)
            }
            NarrowKind::FalseBranch => self.filter_to_falsy(type_),
        }
    }

    pub(crate) fn narrow_by_optionality(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        _depth: u32,
    ) -> Arc<Type> {
        if self.expr_matches_target(expr, target) {
            return match kind {
                NarrowKind::TrueBranch => self.remove_nullable_from_union(type_),
                NarrowKind::FalseBranch => {
                    self.filter_type_by_flags(type_, TypeFlags::Undefined | TypeFlags::Null)
                }
            };
        }

        if expr.kind == SyntaxKind::Identifier && self.flow_inline_level < 5 {
            if let Some(init_expr) = self.const_alias_initializer(expr) {
                self.flow_inline_level += 1;
                let result = self.narrow_by_optionality(type_, &init_expr, target, kind, _depth);
                self.flow_inline_level -= 1;
                return result;
            }
        }

        Arc::clone(type_)
    }
}
