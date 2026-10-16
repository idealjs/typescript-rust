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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_call_expression"); 
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

    fn callback_predicate_type(&mut self, arg: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("callback_predicate_type"); 
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

    // Go getTypeAtFlowCall 谓词取自调用的解析签名（relater.go:2161-2165
    // 实例化签名谓词经 sig.mapper 实例化）；Rust 消费调用检查期驻留的
    // call_inference 推断实参对谓词型代入，无驻留时仅显式类型实参取
    // 解析签名谓词（显式实参为固定映射），否则回退实参重推断
    fn instantiated_assertion_predicate(
        &mut self,
        call_expr: &Arc<Node>,
        sig: &Arc<Signature>,
        args: &[Arc<Node>],
    ) -> Option<TypePredicate> { ::tsox_core::fntrace::enter("instantiated_assertion_predicate");
        let mut predicate = self.compute_type_predicate_of_signature(sig)?;
        let Some(pred_type) = &predicate.t else {
            return Some(predicate);
        };
        if sig.type_parameters.is_empty() {
            return Some(predicate);
        }
        let inference = self
            .signature_links
            .get(call_expr)
            .and_then(|l| l.call_inference.clone());
        let instantiated = match inference {
            Some(inference)
                if inference.inferred_types.len() == sig.type_parameters.len() =>
            {
                self.substitute_infer_type_parameters(
                    pred_type,
                    &sig.type_parameters,
                    &inference.inferred_types,
                )
            }
            _ => {
                let explicit = matches!(
                    &call_expr.data,
                    tsox_frontend::ast::NodeData::CallExpression(d)
                        if d.type_arguments.is_some()
                );
                if explicit
                    && let Some(resolved) = self.get_resolved_signature(call_expr)
                    && !self.signature_is_resolving_signature(&resolved)
                    && let Some(resolved_predicate) =
                        self.get_type_predicate_of_signature(&resolved).cloned()
                {
                    return Some(resolved_predicate);
                }
                let inferred = self.infer_call_type_arguments(call_expr, sig, args);
                self.substitute_infer_type_parameters(pred_type, &sig.type_parameters, &inferred)
            }
        };
        predicate.t = Some(instantiated);
        Some(predicate)
    }

    pub(crate) fn narrow_by_assertion_call(
        &mut self,
        type_: &Arc<Type>,
        declared: &Arc<Type>,
        call_expr: &Arc<Node>,
        target: &FlowRef,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_assertion_call");
        let NodeData::CallExpression(call) = &call_expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        for sig in &signatures {
            let Some(predicate) =
                self.instantiated_assertion_predicate(call_expr, sig, &call.arguments.nodes)
            else {
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
                if let Some(pred_type) = &predicate.t {
                    // Go narrowTypeByTypePredicate（flow.go:340-348）：可选链实参
                    // 蕴含引用非空，先摘 undefined/null；实参为判别式属性访问时
                    // 按谓词型走判别式收窄，摘除后无访问则直接返回摘除结果
                    let mut t = Arc::clone(type_);
                    let mut adjusted = false;
                    if self.strict_null_checks
                        && self.optional_chain_contains_target(arg, target)
                        && !self.has_type_facts(pred_type, TypeFacts::EQ_UNDEFINED)
                    {
                        t = self.get_adjusted_type_with_facts(&t, TypeFacts::NE_UNDEFINED_OR_NULL);
                        adjusted = true;
                    }
                    // Go getDiscriminantPropertyAccess 准入等价兜底：属性为联合
                    // 全成分字面量键属性（getKeyPropertyName 语义，本地
                    // get_key_property_name 为恒空壳）时，窄化 filter 与
                    // narrowTypeByDiscriminant 在 Go 下的行为一致
                    if let Some(access) =
                        self.assertion_discriminant_access(&t, declared, arg, target)
                            .or_else(|| self.key_property_discriminant_access(&t, arg, target))
                    {
                        return self.narrow_type_by_discriminant(&t, &access, &|checker, t| {
                            checker.get_narrowed_type(t, pred_type, true, false)
                        });
                    }
                    if adjusted {
                        return t;
                    }
                }
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

    // Go getDiscriminantPropertyAccess（flow.go:1487）+ IsAccessExpression 候选
    //（flow.go:1517-1521）：标识符/解构候选已被前置引用匹配分支覆盖，断言
    // 实参只考察接收者与引用配对的属性访问
    fn assertion_discriminant_access(
        &mut self,
        computed: &Arc<Type>,
        declared: &Arc<Type>,
        arg: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("assertion_discriminant_access");
        if !declared.flags.intersects(TypeFlags::Union) && !computed.flags.intersects(TypeFlags::Union)
        {
            return None;
        }
        let receiver = match &arg.data {
            NodeData::PropertyAccessExpression(pa) => Some(&pa.expression),
            NodeData::ElementAccessExpression(ea) => Some(&ea.expression),
            _ => None,
        }?;
        if !self.expr_matches_target(&receiver, target) {
            return None;
        }
        let name = self.get_accessed_property_name(arg)?;
        let t = if declared.flags.intersects(TypeFlags::Union)
            && self.is_type_subset_of(computed, declared)
        {
            declared
        } else {
            computed
        };
        if self.is_discriminant_property(t, &name) {
            Some(Arc::clone(arg))
        } else {
            None
        }
    }

    // Go getKeyPropertyName（flow.go:749 语义）：联合每一成分在同名属性上均为
    // unit（字面量）型，该属性即键属性；本地 get_key_property_name 为恒空壳，
    // 这里按语义内联
    fn key_property_discriminant_access(
        &mut self,
        base: &Arc<Type>,
        arg: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("key_property_discriminant_access");
        if !base.is_union() {
            return None;
        }
        let receiver = match &arg.data {
            NodeData::PropertyAccessExpression(pa) => Some(&pa.expression),
            NodeData::ElementAccessExpression(ea) => Some(&ea.expression),
            _ => None,
        }?;
        if !self.expr_matches_target(&receiver, target) {
            return None;
        }
        let name = self.get_accessed_property_name(arg)?;
        for constituent in self.constituent_types(base) {
            let prop_type = self.get_property_type_of_type(&constituent, &name)?;
            if !crate::checker::is_unit_type(&prop_type) {
                return None;
            }
        }
        Some(Arc::clone(arg))
    }

    fn narrow_by_asserted_comparison(
        &mut self,
        type_: &Arc<Type>,
        arg: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("narrow_by_asserted_comparison"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_type_predicate");
        self.get_narrowed_type(type_, pred_type, assume_true, false)
    }

    pub(crate) fn typeof_expr_matches_target(&mut self, expr: &Arc<Node>, target: &FlowRef) -> bool { ::tsox_core::fntrace::enter("typeof_expr_matches_target"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_typeof"); 
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

    pub(crate) fn narrow_by_truthiness(&mut self, type_: &Arc<Type>, kind: NarrowKind) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_truthiness");
        let constituents = self.constituent_types(&self.split_intrinsic_boolean(type_));
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

    pub(crate) fn split_intrinsic_boolean(&self, type_: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("split_intrinsic_boolean"); 
        if type_.flags.contains(TypeFlags::Boolean) && !type_.is_union() {
            return self.flow_union_of(&[self.false_type(), self.true_type()]);
        }
        Arc::clone(type_)
    }

    // Go getTypeFactsWorker 的 Truthy 位：字面量按值判定，非字面
    // number/string/bigint/boolean/enum 与 symbol/object/any 均持有
    pub(crate) fn has_truthy_fact(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("has_truthy_fact"); 
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

    // Go hasTypeFacts（checker.go:32784）：Falsy 位经对齐的 get_type_facts_worker
    // 表判定（非 strict 模式 NonZero/NonEmpty/True*Facts 表项全持有 Falsy）
    pub(crate) fn has_falsy_fact(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("has_falsy_fact"); 
        self.get_type_facts(t, TypeFacts::FALSY) != TypeFacts::empty()
    }

    pub(crate) fn narrow_by_optionality(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        _depth: u32,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_optionality"); 
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
) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_this_type"); 
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
