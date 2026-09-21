#![allow(unused_imports)]

use crate::checker::inference::{InferenceContext, InferenceInfo, InferencePriority};
use crate::checker::relater_relate_impl_chunk::*;
use crate::checker::types::next_type_id;
use crate::checker::types::{
    ConstrainedTypeData, ObjectFlags, Signature, SignatureKind, Type, TypeData, TypeFlags,
    TypeParameterData,
};
use std::sync::OnceLock;
use tsox_frontend::ast::{Symbol, SymbolFlags};

fn info_has_candidates(info: &InferenceInfo) -> bool {
    !info.candidates.is_empty() || !info.contra_candidates.is_empty()
}

fn symbol_name(t: &Arc<Type>) -> String {
    t.symbol
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_default()
}

impl Checker {
    /// Go instantiateTypeWithSingleGenericCallSignature 的推断类型参数分支
    /// （checker.go:7813-7841）：外层签名返回单非泛型调用签名的函数型且外层
    /// 推断尚有零候选类型参数时，用实参签名自身类型参数（重名时克隆改名）
    /// 恒等实例化，向上下文签名逆向（contra）推断外层候选，命中则合并回外层
    /// 语境并把实参类型参数记入 inferredTypeParameters
    pub(crate) fn instantiate_with_inferred_return_tps(
        &mut self,
        sig: &Arc<Signature>,
        ctx_sig: &Arc<Signature>,
        outer: &mut InferenceContext,
        construct: bool,
    ) -> Option<Arc<Type>> {
        let outer_sig = Arc::clone(outer.signature.as_ref()?);
        let return_type = self.get_return_type_of_signature(&outer_sig)?;
        let (return_signature, _) = self.single_call_or_construct_signature(&return_type)?;
        if !return_signature.type_parameters.is_empty() {
            return None;
        }
        if outer.inferences.iter().all(info_has_candidates) {
            return None;
        }
        let unique_tps =
            self.get_unique_type_parameters(&outer.inferred_type_parameters, &sig.type_parameters);
        let instantiated = self.get_signature_instantiation(sig, &unique_tps);
        let mut fresh: Vec<InferenceInfo> = outer
            .inferences
            .iter()
            .map(|i| InferenceInfo::new(Arc::clone(&i.type_parameter)))
            .collect();
        self.infer_signature_pair_into(&instantiated, ctx_sig, &mut fresh);
        if !fresh.iter().any(info_has_candidates) {
            return None;
        }
        let overlap = outer
            .inferences
            .iter()
            .zip(fresh.iter())
            .any(|(a, b)| info_has_candidates(a) && info_has_candidates(b));
        if overlap {
            return None;
        }
        for (target, source) in outer.inferences.iter_mut().zip(fresh.into_iter()) {
            if !info_has_candidates(target) && info_has_candidates(&source) {
                *target = source;
            }
        }
        outer
            .inferred_type_parameters
            .extend(unique_tps.iter().cloned());
        Some(self.create_function_or_constructor_type(vec![instantiated], construct))
    }

    /// Go getSignatureInstantiation 的 inferredTypeParameters 后半
    /// （checker.go:19616-19628）：实例化返回型为单调用/构造签名的函数型时，
    /// 克隆该签名并装上推断类型参数，重建为泛型函数型
    pub(crate) fn regenericize_return_with_tps(
        &mut self,
        return_type: &Arc<Type>,
        tps: &[Arc<Type>],
    ) -> Arc<Type> {
        let Some((single, construct)) = self.single_call_or_construct_signature(return_type) else {
            return Arc::clone(return_type);
        };
        let mut rebuilt = Signature::new();
        rebuilt.flags = single.flags;
        rebuilt.min_argument_count = single.min_argument_count;
        rebuilt.resolved_min_argument_count = single.resolved_min_argument_count;
        rebuilt.declaration = single.declaration.clone();
        rebuilt.target = single.target.clone();
        rebuilt.mapper = single.mapper.clone();
        rebuilt.parameters = single.parameters.clone();
        rebuilt.this_parameter = single.this_parameter.clone();
        rebuilt.type_parameters = tps.to_vec();
        rebuilt.instantiated_parameter_types = single.instantiated_parameter_types.clone();
        if let Some(rt) = self.get_return_type_of_signature(&single) {
            let _ = rebuilt.resolved_return_type.set(rt);
        }
        rebuilt.resolved_type_predicate = single.resolved_type_predicate.clone();
        self.create_function_or_constructor_type(vec![Arc::new(rebuilt)], construct)
    }

    /// Go getSingleCallOrConstructSignature（allowMembers=false）：对象型
    /// 无属性无索引信息且恰一个调用（或构造）签名
    pub(crate) fn single_call_or_construct_signature(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<(Arc<Signature>, bool)> {
        if !t.flags.contains(TypeFlags::Object) {
            return None;
        }
        if !self.get_properties_of_type(t).is_empty() || !self.get_index_infos_of_type(t).is_empty()
        {
            return None;
        }
        let call = self.get_signatures_of_type(t, SignatureKind::Call);
        let construct = self.get_signatures_of_type(t, SignatureKind::Construct);
        if call.len() == 1 && construct.is_empty() {
            return Some((Arc::clone(&call[0]), false));
        }
        if construct.len() == 1 && call.is_empty() {
            return Some((Arc::clone(&construct[0]), true));
        }
        None
    }

    /// Go getUniqueTypeParameters（checker.go:7862）：与已收集的推断类型参数
    /// 重名时克隆出新符号并改名，约束与默认值经 old→new 映射代入
    pub(crate) fn get_unique_type_parameters(
        &mut self,
        taken: &[Arc<Type>],
        tps: &[Arc<Type>],
    ) -> Vec<Arc<Type>> {
        let mut renames: Vec<(Arc<Type>, String)> = Vec::new();
        let mut used_names: Vec<String> = taken.iter().map(symbol_name).collect();
        for tp in tps {
            let name = symbol_name(tp);
            if used_names.iter().any(|n| *n == name) {
                let mut base = name.clone();
                while base.len() > 1 && base.ends_with(|c: char| c.is_ascii_digit()) {
                    base.pop();
                }
                let mut index = 1;
                let new_name = loop {
                    let candidate = format!("{}{}", base, index);
                    if !used_names.iter().any(|n| *n == candidate) {
                        break candidate;
                    }
                    index += 1;
                };
                used_names.push(new_name.clone());
                renames.push((Arc::clone(tp), new_name));
            } else {
                used_names.push(name);
            }
        }
        if renames.is_empty() {
            return tps.to_vec();
        }
        let old: Vec<Arc<Type>> = renames.iter().map(|(tp, _)| Arc::clone(tp)).collect();
        let mut shells: Vec<Arc<Type>> = renames
            .iter()
            .map(|(tp, name)| {
                Arc::new(Type {
                    flags: TypeFlags::TypeParameter,
                    object_flags: ObjectFlags::None,
                    id: next_type_id(),
                    symbol: Some(Arc::new(Symbol::new(SymbolFlags::TypeParameter, name.clone()))),
                    alias: None,
                    data: TypeData::TypeParameter(TypeParameterData {
                        constrained: ConstrainedTypeData::default(),
                        constraint: None,
                        target: Some(Arc::clone(tp)),
                        mapper: None,
                        is_this_type: false,
                        resolved_default_type: OnceLock::new(),
                    }),
                })
            })
            .collect();
        let constraints: Vec<Option<Arc<Type>>> = old
            .iter()
            .map(|tp| {
                self.get_constraint_of_type_parameter(tp)
                    .map(|c| self.substitute_infer_type_parameters(&c, &old, &shells))
            })
            .collect();
        let defaults: Vec<Option<Arc<Type>>> = old
            .iter()
            .map(|tp| {
                self.get_default_from_type_parameter(tp)
                    .map(|d| self.substitute_infer_type_parameters(&d, &old, &shells))
            })
            .collect();
        for (i, shell) in shells.iter_mut().enumerate() {
            let Some(t) = Arc::get_mut(shell) else {
                continue;
            };
            if let TypeData::TypeParameter(data) = &mut t.data {
                if let Some(c) = &constraints[i] {
                    data.constraint = Some(Arc::clone(c));
                }
                if let Some(d) = &defaults[i] {
                    let _ = data.resolved_default_type.set(Arc::clone(d));
                }
            }
        }
        let mut rename_iter = shells.into_iter();
        tps.iter()
            .map(|tp| {
                if renames.iter().any(|(old_tp, _)| Arc::ptr_eq(old_tp, tp)) {
                    rename_iter.next().unwrap_or_else(|| Arc::clone(tp))
                } else {
                    Arc::clone(tp)
                }
            })
            .collect()
    }

    /// Go applyToParameterTypes + applyToReturnTypes（inference.go:868/896）
    /// 指定推断信息集上的对应实现：参数位 contra，返回位协变
    fn infer_signature_pair_into(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        fresh: &mut [InferenceInfo],
    ) {
        let source_count = self.get_parameter_count(source);
        let target_count = self.get_parameter_count(target);
        let source_rest = self.get_effective_rest_type(source);
        let target_rest = self.get_effective_rest_type(target);
        let target_non_rest = target_count - usize::from(target_rest.is_some());
        let param_count = if source_rest.is_some() {
            target_non_rest
        } else {
            source_count.min(target_non_rest)
        };
        if let (Some(s_this), Some(t_this)) = (
            self.get_this_type_of_signature(source),
            self.get_this_type_of_signature(target),
        ) {
            self.infer_types(
                fresh,
                Some(s_this),
                Some(t_this),
                InferencePriority::None,
                true,
            );
        }
        for i in 0..param_count {
            let s = self.get_type_at_position(source, i);
            let t = self.get_type_at_position(target, i);
            self.infer_types(fresh, Some(s), Some(t), InferencePriority::None, true);
        }
        if let Some(t_rest) = target_rest {
            let s_rest = self.source_rest_type_at(source, param_count);
            self.infer_types(
                fresh,
                Some(s_rest),
                Some(t_rest),
                InferencePriority::None,
                true,
            );
        }
        let target_pred = self.compute_type_predicate_of_signature(target);
        let source_pred = self.compute_type_predicate_of_signature(source);
        if let (Some(tp), Some(sp)) = (target_pred, source_pred) {
            if tp.kind == sp.kind
                && tp.parameter_index == sp.parameter_index
                && tp.t.is_some()
                && sp.t.is_some()
            {
                let s = sp.t.unwrap();
                let t = tp.t.unwrap();
                self.infer_types(fresh, Some(s), Some(t), InferencePriority::None, false);
                return;
            }
        }
        if let Some(t_ret) = self.get_return_type_of_signature(target) {
            if self.could_contain_type_variables(&t_ret) {
                if let Some(s_ret) = self.get_return_type_of_signature(source) {
                    self.infer_types(
                        fresh,
                        Some(s_ret),
                        Some(t_ret),
                        InferencePriority::None,
                        false,
                    );
                }
            }
        }
    }
}
