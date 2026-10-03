#![allow(unused_imports)]

use std::sync::Arc;

use crate::checker::mapper::new_array_type_mapper;
use crate::checker::relater_compare::*;

impl Checker {
    /// Go findMatchingSignatures（relater.go:2267）：泛型签名要求其余成分
    /// 逐列表精确匹配且只从首成分发起；非泛型签名按忽略返回型精确匹
    /// 配，失配退到部分匹配（子类型比较 + 放宽元数）
    pub(crate) fn find_matching_signatures(
        &mut self,
        signature_lists: &[Vec<Arc<Signature>>],
        signature: &Arc<Signature>,
        list_index: usize,
    ) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("find_matching_signatures"); 
        if !signature.type_parameters.is_empty() {
            if list_index > 0 {
                return Vec::new();
            }
            for list in signature_lists.iter().skip(1) {
                if self
                    .find_matching_signature_in_list(list, signature, false, false, false)
                    .is_none()
                {
                    return Vec::new();
                }
            }
            return vec![Arc::clone(signature)];
        }
        let mut result: Vec<Arc<Signature>> = Vec::new();
        for (i, list) in signature_lists.iter().enumerate() {
            let matched = if i == list_index {
                Some(Arc::clone(signature))
            } else {
                self.find_matching_signature_in_list(list, signature, false, false, true)
                    .or_else(|| {
                        self.find_matching_signature_in_list(list, signature, true, false, true)
                    })
            };
            let Some(matched) = matched else {
                return Vec::new();
            };
            if !result.iter().any(|sig| Arc::ptr_eq(sig, &matched)) {
                result.push(matched);
            }
        }
        result
    }

    /// Go findMatchingSignature（relater.go:2303）
    fn find_matching_signature_in_list(
        &mut self,
        signature_list: &[Arc<Signature>],
        signature: &Arc<Signature>,
        partial_match: bool,
        ignore_this_types: bool,
        ignore_return_types: bool,
    ) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("find_matching_signature_in_list"); 
        for candidate in signature_list {
            let related = self.compare_signatures_identical_ex(
                candidate,
                signature,
                partial_match,
                ignore_this_types,
                ignore_return_types,
            );
            if !related.is_false() {
                return Some(Arc::clone(candidate));
            }
        }
        None
    }

    /// Go compareSignaturesIdentical（relater.go:2316）：compareTypes 随
    /// partialMatch 在同一/子类型间切换；类型参数先经约束/默认值同一性
    /// 检查再以目标类型参数实例化擦除；参数位按目标→源方向比较
    fn compare_signatures_identical_ex(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        partial_match: bool,
        ignore_this_types: bool,
        ignore_return_types: bool,
    ) -> Ternary { ::tsox_core::fntrace::enter("compare_signatures_identical_ex"); 
        if Arc::ptr_eq(source, target) {
            return Ternary::True;
        }
        if !self.signatures_match_arity(source, target, partial_match) {
            return Ternary::False;
        }
        if source.type_parameters.len() != target.type_parameters.len() {
            return Ternary::False;
        }
        let mut source = Arc::clone(source);
        if !target.type_parameters.is_empty() {
            let mapper = new_array_type_mapper(
                source.type_parameters.iter().map(Arc::clone).collect(),
                target.type_parameters.iter().map(Arc::clone).collect(),
            );
            for i in 0..target.type_parameters.len() {
                let s = Arc::clone(&source.type_parameters[i]);
                let t = Arc::clone(&target.type_parameters[i]);
                if Arc::ptr_eq(&s, &t) || s.id == t.id {
                    continue;
                }
                let s_constraint = self
                    .get_constraint_of_type_parameter(&s)
                    .unwrap_or_else(|| self.unknown_type());
                let t_constraint = self
                    .get_constraint_of_type_parameter(&t)
                    .unwrap_or_else(|| self.unknown_type());
                let s_default = self
                    .get_default_from_type_parameter(&s)
                    .unwrap_or_else(|| self.unknown_type());
                let t_default = self
                    .get_default_from_type_parameter(&t)
                    .unwrap_or_else(|| self.unknown_type());
                if self
                    .compare_types_for_match(partial_match, &mapper.map(&s_constraint), &t_constraint)
                    .is_false()
                    || self
                        .compare_types_for_match(partial_match, &mapper.map(&s_default), &t_default)
                        .is_false()
                {
                    return Ternary::False;
                }
            }
            let target_params: Vec<Arc<Type>> =
                target.type_parameters.iter().map(Arc::clone).collect();
            let saved_strict = self.erase_signature_strict;
            self.erase_signature_strict = true;
            let instantiated = self.get_signature_instantiation(&source, &target_params);
            self.erase_signature_strict = saved_strict;
            source = instantiated;
        }
        let mut result = Ternary::True;
        if !ignore_this_types {
            if let Some(source_this) = self.get_this_type_of_signature(&source) {
                if let Some(target_this) = self.get_this_type_of_signature(target) {
                    let related =
                        self.compare_types_for_match(partial_match, &source_this, &target_this);
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }
        let target_count = self.get_parameter_count(target);
        for i in 0..target_count {
            let s = self.get_type_at_position(&source, i);
            let t = self.get_type_at_position(target, i);
            let related = self.compare_types_for_match(partial_match, &t, &s);
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        if !ignore_return_types {
            let source_predicate = self.get_type_predicate_of_signature(&source);
            let target_predicate = self.get_type_predicate_of_signature(target);
            match (source_predicate, target_predicate) {
                (Some(sp), Some(tp)) => {
                    let related = self.compare_type_predicates_identical(
                        sp,
                        tp,
                        &|_s: &Arc<Type>, _t: &Arc<Type>| Ternary::True,
                    );
                    result = result.and(related);
                }
                (None, None) => {
                    let s_return = self
                        .get_return_type_of_signature(&source)
                        .unwrap_or_else(|| self.any_type());
                    let t_return = self
                        .get_return_type_of_signature(target)
                        .unwrap_or_else(|| self.any_type());
                    let related =
                        self.compare_types_for_match(partial_match, &s_return, &t_return);
                    result = result.and(related);
                }
                _ => return Ternary::False,
            }
        }
        result
    }

    /// Go isMatchingSignature（relater.go:2383）：严格档要求参数数/最少
    /// 实参/rest 三同；部分匹配档放宽为源最少实参不多于目标
    fn signatures_match_arity(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        partial_match: bool,
    ) -> bool { ::tsox_core::fntrace::enter("signatures_match_arity"); 
        let source_parameter_count = self.get_parameter_count(source);
        let target_parameter_count = self.get_parameter_count(target);
        let source_min = self.get_min_argument_count(source);
        let target_min = self.get_min_argument_count(target);
        let source_has_rest = self.has_effective_rest_parameter(source);
        let target_has_rest = self.has_effective_rest_parameter(target);
        if source_parameter_count == target_parameter_count
            && source_min == target_min
            && source_has_rest == target_has_rest
        {
            return true;
        }
        partial_match && source_min <= target_min
    }

    fn compare_types_for_match(
        &mut self,
        partial_match: bool,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary { ::tsox_core::fntrace::enter("compare_types_for_match"); 
        if partial_match {
            self.compare_types_subtype_of(source, target)
        } else {
            self.compare_types_identical(source, target)
        }
    }
}
