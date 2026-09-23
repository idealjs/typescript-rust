#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub fn infer_types(
        &mut self,
        inferences: &mut [InferenceInfo],
        original_source: Option<Arc<Type>>,
        original_target: Option<Arc<Type>>,
        priority: InferencePriority,
        contravariant: bool,
    ) {
        let mut state = InferenceState {
            inferences,
            original_source: original_source.clone(),
            original_target: original_target.clone(),
            priority,
            inference_priority: InferencePriority::MaxValue,
            contravariant,
            bivariant: false,
            expanding_flags: ExpandingFlags::None,
            propagation_type: None,
            visited: HashMap::new(),
            once_visited: HashMap::new(),
            depth: 0,
        };
        if let (Some(source), Some(target)) = (original_source, original_target) {
            self.infer_from_types(&mut state, &source, &target);
        }
    }

    fn min_priority(a: InferencePriority, b: InferencePriority) -> InferencePriority {
        if a.bits() < b.bits() {
            a
        } else {
            b
        }
    }

    fn invoke_once_enter(
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<InferencePriority> {
        let key = (source.id, target.id);
        if let Some(&status) = state.once_visited.get(&key) {
            state.inference_priority = Self::min_priority(state.inference_priority, status);
            return None;
        }
        state.once_visited.insert(key, InferencePriority::Circularity);
        let save = state.inference_priority;
        state.inference_priority = InferencePriority::MaxValue;
        Some(save)
    }

    fn invoke_once_exit(
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
        save: InferencePriority,
    ) {
        state
            .once_visited
            .insert((source.id, target.id), state.inference_priority);
        state.inference_priority = Self::min_priority(state.inference_priority, save);
    }

    pub(crate) fn infer_from_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if !self.could_contain_type_variables(target) || self.is_no_infer_type(target) {
            return;
        }

        let key = (source.id, target.id);
        if state.visited.contains_key(&key) {
            return;
        }
        state.visited.insert(key, state.priority);
        self.infer_from_types_inner(state, source, target);
        state.visited.remove(&key);
    }

    pub(crate) fn infer_from_types_inner(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if Arc::ptr_eq(source, target)
            && source
                .flags
                .intersects(TypeFlags::Union | TypeFlags::Intersection)
        {
            for t in source.types().unwrap_or_default() {
                self.infer_from_types(state, t, t);
            }
            return;
        }

        if target.flags.contains(TypeFlags::Union) {
            let source_types = if source.flags.contains(TypeFlags::Union) {
                source.types().unwrap_or_default().to_vec()
            } else {
                vec![Arc::clone(source)]
            };
            let target_types = target.types().unwrap_or_default().to_vec();
            let (temp_sources, temp_targets) = self.infer_from_matching_types(
                state,
                &source_types,
                &target_types,
                MatchingKind::OrBaseIdentical,
            );
            let (sources, targets) = self.infer_from_matching_types(
                state,
                &temp_sources,
                &temp_targets,
                MatchingKind::CloselyMatched,
            );
            if targets.is_empty() {
                return;
            }
            let target = self.get_union_type(targets);
            if sources.is_empty() {
                self.infer_with_priority(
                    state,
                    source,
                    &target,
                    InferencePriority::NakedTypeVariable,
                );
                return;
            }
            let source = self.get_union_type(sources);
            if target.flags.contains(TypeFlags::Union) {
                let target_list: Vec<Arc<Type>> = target.types().unwrap_or_default().to_vec();
                self.infer_to_multiple_types_union(state, &source, &target_list);
            } else {
                // 匹配消去后归约为单成分：继续主流程（Go 分支重赋值
                // source/target 后继续执行，走 TypeVariable 登记）
                self.infer_from_types_inner(state, &source, &target);
            }
            return;
        }

        if target.flags.contains(TypeFlags::Intersection) {
            self.infer_from_types_intersection(state, source, target);
            return;
        }

        // Go TypeFlagsTypeVariable 为复合标志：TypeParameter | IndexedAccess | Substitution，
        // 延迟 IndexedAccess（反向映射的 T[K]）同样登记候选
        if target
            .flags
            .intersects(TypeFlags::TypeParameter | TypeFlags::IndexedAccess)
        {
            let both_indexed = source.flags.contains(TypeFlags::IndexedAccess)
                && target.flags.contains(TypeFlags::IndexedAccess);
            let matched = state.inferences.iter().any(|info| {
                crate::checker::utilities::type_parameters_match(&info.type_parameter, target)
            });
            if !both_indexed || matched {
                self.infer_to_type_variable(state, source, target);
                return;
            }
            // Go inferFromTypes：source/target 均为索引访问且 target 非推断目标
            // （如 U[L]，被推断的是 U 与 L）时分解成分推断 —— T[K] 对 U[L]
            // 产生 T→U、K→L 两组候选（higherOrder 签名关系推断依赖）
            if let (TypeData::IndexedAccess(sd), TypeData::IndexedAccess(td)) =
                (&source.data, &target.data)
            {
                if let (Some(so), Some(to)) = (&sd.object_type, &td.object_type) {
                    self.infer_from_types(state, so, to);
                }
                if let (Some(si), Some(ti)) = (&sd.index_type, &td.index_type) {
                    self.infer_from_types(state, si, ti);
                }
            }
            return;
        }

        // Go inferFromTypes switch 的 source-union 分发：source 为联合而
        // target 非联合时按成分推断（如 ActionFunction<X> | undefined →
        // 带调用签名的结构目标）
        if source.flags.contains(TypeFlags::Union) {
            if let Some(members) = source.types() {
                for m in members {
                    self.infer_from_types(state, m, target);
                }
                return;
            }
        }

        if target.flags.contains(TypeFlags::Conditional) {
            if let Some(save) = Self::invoke_once_enter(state, source, target) {
                self.infer_to_conditional_type(state, source, target);
                Self::invoke_once_exit(state, source, target, save);
            }
            return;
        }

        if target.flags.contains(TypeFlags::Object) {
            if let Some(save) = Self::invoke_once_enter(state, source, target) {
                self.infer_from_object_types(state, source, target);
                Self::invoke_once_exit(state, source, target, save);
            }
            return;
        }
    }

    fn infer_to_conditional_type(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let tc = match &target.data {
            TypeData::Conditional(tc) => tc,
            _ => return,
        };
        if let TypeData::Conditional(sc) = &source.data {
            if let (Some(scheck), Some(tcheck)) = (&sc.check_type, &tc.check_type) {
                self.infer_from_types(state, scheck, tcheck);
            }
            if let (Some(sextends), Some(textends)) = (&sc.extends_type, &tc.extends_type) {
                self.infer_from_types(state, sextends, textends);
            }
            let s_true = self.get_forced_branch_type_of_conditional_type(source, true);
            let t_true = self.get_forced_branch_type_of_conditional_type(target, true);
            if let (Some(s), Some(t)) = (&s_true, &t_true) {
                self.infer_from_types(state, s, t);
            }
            let s_false = self.get_forced_branch_type_of_conditional_type(source, false);
            let t_false = self.get_forced_branch_type_of_conditional_type(target, false);
            if let (Some(s), Some(t)) = (&s_false, &t_false) {
                self.infer_from_types(state, s, t);
            }
            return;
        }
        let group_priority = if state.contravariant && !state.bivariant {
            InferencePriority::ContravariantConditional
        } else {
            InferencePriority::None
        };
        let mut branch_types = Vec::new();
        if let Some(t) = self.get_forced_branch_type_of_conditional_type(target, true) {
            branch_types.push(t);
        }
        if let Some(t) = self.get_forced_branch_type_of_conditional_type(target, false) {
            branch_types.push(t);
        }
        self.infer_to_multiple_types_non_union(state, source, &branch_types, group_priority);
    }

    pub(crate) fn infer_from_types_intersection(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_types = if source.flags.contains(TypeFlags::Intersection) {
            source.types().unwrap_or_default().to_vec()
        } else {
            vec![Arc::clone(source)]
        };
        let target_types = target.types().unwrap_or_default().to_vec();
        let (sources, targets) =
            self.infer_matching_types_identical(state, &source_types, &target_types);
        if sources.is_empty() || targets.is_empty() {
            return;
        }
        let source = self.get_intersection_type(sources);
        let target = self.get_intersection_type(targets);
        for t in target.types().unwrap_or(&[]) {
            self.infer_from_types(state, &source, t);
        }
    }

    pub(crate) fn infer_to_type_variable(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if self.is_from_inference_blocked_source(source) {
            return;
        }
        // NoInfer 形态不产候选：Go 检查期对 NoInfer 包裹的上下文成员解析为
        // never/零候选（终值回退约束或 unknown），此处按形态守卫等价实现
        if self.is_no_infer_type(source) {
            return;
        }

        let inference_idx = state.inferences.iter().position(|info| {
            crate::checker::utilities::type_parameters_match(&info.type_parameter, target)
        });
        let Some(idx) = inference_idx else { return };

        let priority = state.priority;
        let contravariant = state.contravariant;
        let bivariant = state.bivariant;
        let depth = state.depth;
        let propagation_type = state.propagation_type.clone();

        let mut cleared = false;
        let inference = &mut state.inferences[idx];
        if source.object_flags.contains(ObjectFlags::NonInferrableType) {
            return;
        }
        if !inference.is_fixed {
            let candidate = propagation_type.unwrap_or_else(|| Arc::clone(source));
            // CS 实参部分代入后的回声域内自引用推断（T←T）无信息量：Go 用
            // non-fixing mapper 让上下文化实参的类型参数引用替换为已推断值，
            // 此处以候选与被推断类型参数符号等价拦截；域外（如递归泛型调用
            // Generator<U> → Generator<U> 的实参推断）自引用候选合法，
            // Go getCovariantInference 取其公共超类型即类型参数自身
            if self.cs_echo_inference
                && crate::checker::utilities::type_parameters_match(
                    &candidate,
                    &inference.type_parameter,
                )
            {
                return;
            }
            if priority.bits() < inference.priority.bits() {
                inference.candidates.clear();
                inference.candidate_depths.clear();
                inference.contra_candidates.clear();
                inference.top_level = true;
                inference.priority = priority;
                cleared = true;
            }
            if priority == inference.priority {
                if contravariant && !bivariant {
                    if !inference
                        .contra_candidates
                        .iter()
                        .any(|c| Arc::ptr_eq(c, &candidate))
                    {
                        inference.contra_candidates.push(candidate);
                        cleared = true;
                    }
                } else {
                    if !inference
                        .candidates
                        .iter()
                        .any(|c| Arc::ptr_eq(c, &candidate))
                    {
                        inference.candidates.push(candidate);
                        inference.candidate_depths.push(depth);
                        cleared = true;
                    }
                }
            }
            // Go inference.go:207：仅当类型参数不在原始目标的顶层位置时清除 topLevel
            let at_top_level = match state.original_target.as_ref() {
                Some(orig) => self.is_type_parameter_at_top_level(orig, target, 0),
                None => true,
            };
            if !priority.contains(InferencePriority::ReturnType)
                && target.flags.contains(TypeFlags::TypeParameter)
                && inference.top_level
                && !at_top_level
            {
                inference.top_level = false;
                cleared = true;
            }
        }
        let _ = inference;
        if cleared {
            for info in state.inferences.iter_mut() {
                info.inferred_type = None;
            }
        }
        state.inference_priority = if state.inference_priority.bits() < state.priority.bits() {
            state.inference_priority
        } else {
            state.priority
        };
    }
}
