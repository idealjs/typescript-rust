use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::SyntaxKind;

use crate::checker::checker::Checker;
use crate::checker::types::*;

use crate::checker::flow::FlowRef;

use crate::checker::flow::NarrowKind;

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
                    // 有回调实参（Array.filter 谓词回灌类型参数 U）时经回调
                    // 谓词定型；无参直谓词（isSundries(): this is X）按原样。
                    // 谓词中的多态 this 按接收者当前型实例化（Go 经调用签名
                    // 实例化后 this 已定型；`this is (this & {...})` 不替换
                    // 会让交集中的 this 悬空，后续联合归并无法收纳）
                    let instantiated = match call.arguments.nodes.first() {
                        Some(callback_arg) => {
                            let Some(u) = self.callback_predicate_type(callback_arg) else {
                                continue;
                            };
                            if sig.type_parameters.is_empty() {
                                substitute_this_type(self, pred_type, type_)
                            } else {
                                let args: Vec<Arc<Type>> = sig
                                    .type_parameters
                                    .iter()
                                    .map(|_| Arc::clone(&u))
                                    .collect();
                                let substed = self.substitute_infer_type_parameters(
                                    pred_type,
                                    &sig.type_parameters,
                                    &args,
                                );
                                substitute_this_type(self, &substed, type_)
                            }
                        }
                        None => substitute_this_type(self, pred_type, type_),
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
            let instantiated_pred = if sig.type_parameters.is_empty() {
                Arc::clone(pred_type)
            } else {
                let inferred = self.infer_call_type_arguments(expr, sig, &call.arguments.nodes);
                self.substitute_infer_type_parameters(pred_type, &sig.type_parameters, &inferred)
            };
            return self.narrow_by_type_predicate(type_, &instantiated_pred, assume_true);
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
        use tsox_frontend::ast::SyntaxKind::*;
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
        if narrow_to_value {
            return self.narrow_type_by_type_name(type_, type_name);
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
                return self.remove_object_from_union(type_);
            }
            _ => return Arc::clone(type_),
        };
        let _ = is_loose;
        self.remove_flags_from_union(type_, matching_flags)
    }

    pub(crate) fn narrow_by_truthiness(&self, type_: &Arc<Type>, kind: NarrowKind) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let kept: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| match kind {
                NarrowKind::TrueBranch => self.has_truthy_fact(t),
                NarrowKind::FalseBranch => self.has_falsy_fact(t),
            })
            .collect();
        if kept.is_empty() {
            return self.never_type();
        }
        if kept.len() == 1 {
            return kept.into_iter().next().expect("exactly one");
        }
        self.flow_union_of(&kept)
    }

    // Go getTypeFactsWorker 的 Truthy 位：字面量按值判定，非字面
    // number/string/bigint/boolean/enum 与 symbol/object/any 均持有
    pub(crate) fn has_truthy_fact(&self, t: &Arc<Type>) -> bool {
        let flags = t.flags;
        if flags.contains(TypeFlags::Never)
            || flags.intersects(TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void)
        {
            return false;
        }
        if flags.contains(TypeFlags::BooleanLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(lit.value, LiteralValue::Boolean(true)));
        }
        if flags.contains(TypeFlags::StringLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(&lit.value, LiteralValue::String(s) if !s.is_empty()));
        }
        if flags.contains(TypeFlags::NumberLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(&lit.value, LiteralValue::Number(n) if n.0 != 0.0));
        }
        if flags.contains(TypeFlags::BigIntLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(&lit.value, LiteralValue::BigInt(b) if !b.is_zero()));
        }
        true
    }

    // Go getTypeFactsWorker 的 Falsy 位：非字面 number/string/bigint/boolean/enum
    // 双持有（两分支都保留）；symbol/object/nonPrimitive 仅非 strict 持有；
    // any/unknown/类型参数等 instantiable 走 UnknownFacts（全持有）
    pub(crate) fn has_falsy_fact(&self, t: &Arc<Type>) -> bool {
        let flags = t.flags;
        if flags.contains(TypeFlags::Never) {
            return false;
        }
        if flags.intersects(TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void) {
            return true;
        }
        if flags.contains(TypeFlags::BooleanLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(lit.value, LiteralValue::Boolean(false)));
        }
        if flags.contains(TypeFlags::StringLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(&lit.value, LiteralValue::String(s) if s.is_empty()));
        }
        if flags.contains(TypeFlags::NumberLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(&lit.value, LiteralValue::Number(n) if n.0 == 0.0));
        }
        if flags.contains(TypeFlags::BigIntLiteral) {
            return matches!(&t.data, TypeData::Literal(lit)
                if matches!(&lit.value, LiteralValue::BigInt(b) if b.is_zero()));
        }
        if flags.intersects(
            TypeFlags::Number
                | TypeFlags::String
                | TypeFlags::StringMapping
                | TypeFlags::TemplateLiteral
                | TypeFlags::BigInt
                | TypeFlags::Boolean
                | TypeFlags::Enum
                | TypeFlags::EnumLiteral,
        ) {
            return true;
        }
        if flags.intersects(
            TypeFlags::ESSymbol
                | TypeFlags::UniqueESSymbol
                | TypeFlags::Object
                | TypeFlags::NonPrimitive
                | TypeFlags::Intersection,
        ) {
            return !self.strict_null_checks;
        }
        true
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

/// 谓词型中的多态 this 按接收者当前型替换（Go 调用签名实例化后的形态）
pub(crate) fn substitute_this_type(
    checker: &mut Checker,
    t: &Arc<Type>,
    replacement: &Arc<Type>,
) -> Arc<Type> {
    if let Some(tp) = match &t.data {
        TypeData::TypeParameter(tp) if tp.is_this_type => Some(()),
        _ => None,
    } {
        let _ = tp;
        return Arc::clone(replacement);
    }
    match &t.data {
        TypeData::Union(u) => {
            let parts: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .map(|inner| substitute_this_type(checker, inner, replacement))
                .collect();
            checker.get_union_type(parts)
        }
        TypeData::Intersection(i) => {
            let parts: Vec<Arc<Type>> = i
                .union_or_intersection
                .types
                .iter()
                .map(|inner| substitute_this_type(checker, inner, replacement))
                .collect();
            checker.get_intersection_type(parts)
        }
        _ => Arc::clone(t),
    }
}
