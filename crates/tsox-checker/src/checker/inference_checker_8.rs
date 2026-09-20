#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub(crate) fn infer_from_object_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        // Go inferFromObjectTypes：目标是 { [P in keyof T]: X } / { [P in K]: X }
        // 时走映射型推理（命中则短路普通成员推断）
        if let TypeData::Mapped(m) = &target.data
            && m.name_type.is_none()
        {
            let constraint = m.constraint_type.clone();
            if self.infer_to_mapped_type(state, source, target, constraint.as_ref()) {
                return;
            }
        }

        // Go inferFromObjectTypes 的元组分支：元组/数组对元组按元素位推断
        //（元素存于 element_infos，get_type_arguments 不覆盖 Tuple 形态）
        let source_is_tuple = crate::checker::utilities::is_tuple_type(source);
        let target_is_tuple = crate::checker::utilities::is_tuple_type(target);
        if (source_is_tuple || self.is_array_type(source)) && target_is_tuple {
            self.infer_from_tuple_elements(state, source, target);
            return;
        }

        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if Arc::ptr_eq(source, target) && !target_args.is_empty() {
            self.infer_from_type_arguments(state, &target_args, &target_args, &[]);
            return;
        }

        let same_target = match (source.target(), target.target()) {
            (Some(st), Some(tt)) => Arc::ptr_eq(st, tt),
            _ => false,
        };
        // 实例化引用可能丢 Reference 标志：同 symbol 且双侧带类型实参时视作
        // 同目标泛型引用（Go inferTo 的 reference-vs-reference 分支，只推
        // 类型实参并提前返回，不做成员推断——成员的声明型类型参数未替换，
        // 会以外层类型参数污染候选集）
        let same_symbol = match (source.symbol.as_ref(), target.symbol.as_ref()) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if (source.object_flags.contains(ObjectFlags::Reference)
            && target.object_flags.contains(ObjectFlags::Reference)
            && (same_target || self.is_array_type(source) && self.is_array_type(target)))
            || (same_symbol && !source_args.is_empty() && !target_args.is_empty())
        {
            self.infer_from_type_arguments(state, &source_args, &target_args, &[]);
            return;
        }
        if !source_args.is_empty() && !target_args.is_empty() {
            self.infer_from_type_arguments(state, &source_args, &target_args, &[]);
        }
        self.infer_from_properties(state, source, target);
        self.infer_from_signatures(state, source, target);
        self.infer_from_index_types(state, source, target);
    }

    pub(crate) fn infer_from_type_arguments(
        &mut self,
        state: &mut InferenceState,
        source_types: &[Arc<Type>],
        target_types: &[Arc<Type>],
        _variances: &[VarianceFlags],
    ) {
        let count = source_types.len().min(target_types.len());
        for i in 0..count {
            self.infer_from_types(state, &source_types[i], &target_types[i]);
        }
    }

    fn tuple_structure_matching(s: &Arc<Type>, t: &Arc<Type>) -> bool {
        let (TypeData::Tuple(sd), TypeData::Tuple(td)) = (&s.data, &t.data) else {
            return false;
        };
        if sd.element_infos.len() != td.element_infos.len() {
            return false;
        }
        sd.element_infos
            .iter()
            .zip(td.element_infos.iter())
            .all(|(se, te)| {
                se.flags.intersects(crate::checker::types::ELEMENT_FLAGS_VARIABLE)
                    == te.flags.intersects(crate::checker::types::ELEMENT_FLAGS_VARIABLE)
            })
    }

    pub(crate) fn end_fixed_element_count(t: &Arc<Type>) -> usize {
        match &t.data {
            TypeData::Tuple(tuple) => {
                let infos = &tuple.element_infos;
                for i in (0..infos.len()).rev() {
                    if !infos[i].flags.intersects(crate::checker::types::ELEMENT_FLAGS_FIXED) {
                        return infos.len() - i - 1;
                    }
                }
                infos.len()
            }
            _ => 0,
        }
    }

    fn infer_from_tuple_elements(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_is_tuple = crate::checker::utilities::is_tuple_type(source);
        let source_args = if source_is_tuple {
            Self::tuple_type_arguments(source)
        } else {
            self.get_type_arguments(source)
        };
        let target_args = Self::tuple_type_arguments(target);
        let source_arity = source_args.len();
        let target_arity = target_args.len();
        if source_arity == 0 || target_arity == 0 {
            self.infer_from_index_types(state, source, target);
            return;
        }
        let target_flags: Vec<crate::checker::types::ElementFlags> = match &target.data {
            TypeData::Tuple(tuple) => tuple.element_infos.iter().map(|e| e.flags).collect(),
            _ => Vec::new(),
        };
        let source_flags: Vec<crate::checker::types::ElementFlags> = match &source.data {
            TypeData::Tuple(tuple) => tuple.element_infos.iter().map(|e| e.flags).collect(),
            _ => Vec::new(),
        };

        if source_is_tuple && Self::tuple_structure_matching(source, target) {
            for i in 0..target_arity.min(source_arity) {
                self.infer_from_types(state, &source_args[i], &target_args[i]);
            }
            return;
        }

        let start_length = if source_is_tuple {
            let source_fixed = match &source.data {
                TypeData::Tuple(tuple) => tuple.fixed_length,
                _ => 0,
            };
            let target_fixed = match &target.data {
                TypeData::Tuple(tuple) => tuple.fixed_length,
                _ => 0,
            };
            source_fixed.min(target_fixed)
        } else {
            0
        };
        let target_variable = matches!(&target.data, TypeData::Tuple(t)
            if t.combined_flags.intersects(crate::checker::types::ELEMENT_FLAGS_VARIABLE));
        let end_length = if source_is_tuple && target_variable {
            Self::end_fixed_element_count(source).min(Self::end_fixed_element_count(target))
        } else {
            0
        };

        for i in 0..start_length {
            self.infer_from_types(state, &source_args[i], &target_args[i]);
        }

        if !source_is_tuple {
            let rest_type = self.get_array_element_type(source);
            self.infer_rest_element_to_target(
                state,
                &rest_type,
                &target_args,
                &target_flags,
                start_length,
                end_length,
            );
        } else {
            let source_single_rest = source_arity - start_length - end_length == 1
                && source_flags
                    .get(start_length)
                    .is_some_and(|f| f.contains(crate::checker::types::ElementFlags::Rest));
            if source_single_rest {
                let rest_type = Arc::clone(&source_args[start_length]);
                self.infer_rest_element_to_target(
                    state,
                    &rest_type,
                    &target_args,
                    &target_flags,
                    start_length,
                    end_length,
                );
            } else {
                let middle_length = target_arity.saturating_sub(start_length + end_length);
                if middle_length == 1
                    && target_flags
                        .get(start_length)
                        .is_some_and(|f| f.contains(crate::checker::types::ElementFlags::Variadic))
                {
                    let middle_start = start_length.min(source_arity);
                    let middle_end = (source_arity - end_length).max(middle_start);
                    let middle_infos: Vec<crate::checker::types::TupleElementInfo> =
                        source_flags[middle_start..middle_end]
                            .iter()
                            .map(|f| crate::checker::types::TupleElementInfo {
                                label: None,
                                flags: *f,
                                labeled_declaration: None,
                                type_: None,
                            })
                            .collect();
                    let slice = self.create_tuple_type_ex(
                        source_args[middle_start..middle_end].to_vec(),
                        middle_infos,
                        false,
                    );
                    self.infer_from_types(state, &slice, &target_args[start_length]);
                } else if middle_length == 1
                    && target_flags
                        .get(start_length)
                        .is_some_and(|f| f.contains(crate::checker::types::ElementFlags::Rest))
                {
                    let middle_start = start_length.min(source_arity);
                    let middle_end = (source_arity - end_length).max(middle_start);
                    if middle_start < middle_end {
                        let middle: Vec<Arc<Type>> =
                            source_args[middle_start..middle_end].to_vec();
                        let rest_type = self.get_union_type(middle);
                        self.infer_from_types(state, &rest_type, &target_args[start_length]);
                    }
                }
            }
        }

        for i in 0..end_length {
            let s = &source_args[source_arity - i - 1];
            let t = &target_args[target_arity - i - 1];
            self.infer_from_types(state, s, t);
        }
    }

    fn infer_rest_element_to_target(
        &mut self,
        state: &mut InferenceState,
        rest_type: &Arc<Type>,
        target_args: &[Arc<Type>],
        target_flags: &[crate::checker::types::ElementFlags],
        start_length: usize,
        end_length: usize,
    ) {
        for i in start_length..target_args.len().saturating_sub(end_length) {
            let t = if target_flags
                .get(i)
                .is_some_and(|f| f.contains(crate::checker::types::ElementFlags::Variadic))
            {
                self.create_array_type(Arc::clone(rest_type))
            } else {
                Arc::clone(rest_type)
            };
            self.infer_from_types(state, &t, &target_args[i]);
        }
    }

    pub(crate) fn infer_from_properties(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_struct = source.as_structured();
        let target_struct = target.as_structured();
        if let (Some(source_s), Some(target_s)) = (source_struct, target_struct) {
            for target_prop in &target_s.properties {
                for source_prop in &source_s.properties {
                    if source_prop.name == target_prop.name {
                        let source_type = self.get_type_of_symbol(source_prop);
                        let target_type = self.get_type_of_symbol(target_prop);
                        self.infer_from_types(state, &source_type, &target_type);
                        break;
                    }
                }
            }
        }
    }

    pub(crate) fn infer_from_signatures(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let source_sigs = self.get_signatures_of_type(source, SignatureKind::Call);
        let target_sigs = self.get_signatures_of_type(target, SignatureKind::Call);
        if source_sigs.len() == 1 && target_sigs.len() == 1 {
            self.infer_from_signature(state, &source_sigs[0], &target_sigs[0]);
        }
        let source_ctors = self.get_signatures_of_type(source, SignatureKind::Construct);
        let target_ctors = self.get_signatures_of_type(target, SignatureKind::Construct);
        if source_ctors.len() == 1 && target_ctors.len() == 1 {
            self.infer_from_signature(state, &source_ctors[0], &target_ctors[0]);
        }
    }

    pub(crate) fn infer_from_signature(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
    ) {
        // Go inferFromSignature：方法/构造签名的参数位置是双变的，进入后保持
        let save_biv = state.bivariant;
        let target_is_method = target.declaration.as_ref().is_some_and(|d| {
            matches!(
                d.kind,
                SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature | SyntaxKind::Constructor
            )
        });
        state.bivariant = state.bivariant || target_is_method;
        // Go applyToParameterTypes：非 rest 参数成对（源无 rest 时按源参数数截断），
        // 目标带 rest 时用源在 rest 起点的展开类型对位（...t: T vs ...rest: infer R
        // 推出 R = T[number][]）
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
        for i in 0..param_count {
            // 位置取型（实例化签名携带 instantiated_parameter_types），
            // get_type_of_symbol 只给声明形态（符号类型不含替换）
            let st = self
                .try_get_type_at_position(source, i)
                .unwrap_or_else(|| self.get_type_of_symbol(&source.parameters[i]));
            let tt = self
                .try_get_type_at_position(target, i)
                .unwrap_or_else(|| self.get_type_of_symbol(&target.parameters[i]));
                        // Go applyToParameterTypes→inferFromContravariantTypesIfStrictFunctionTypes：
            // 类型方向不变（target 仍是推断目标），仅 strictFunctionTypes 下翻转 contra 标志
            if self.strict_function_types {
                let save_contra = state.contravariant;
                state.contravariant = !state.contravariant;
                self.infer_from_types(state, &st, &tt);
                state.contravariant = save_contra;
            } else {
                self.infer_from_types(state, &st, &tt);
            }
        }
        if let Some(target_rest_type) = target_rest {
            let source_rest_at = self.source_rest_type_at(source, param_count);
            if self.strict_function_types {
                let save_contra = state.contravariant;
                state.contravariant = !state.contravariant;
                self.infer_from_types(state, &source_rest_at, &target_rest_type);
                state.contravariant = save_contra;
            } else {
                self.infer_from_types(state, &source_rest_at, &target_rest_type);
            }
        }
        state.bivariant = save_biv;

        // Go applyToReturnTypes：目标签名带类型谓词且源谓词 kind/参数位匹配时，
        // 由源谓词类型推断目标谓词类型（find<S extends T>(... => value is S) 的
        // S 推断来源），命中后不再走常规返回位推断
        let target_pred = self.compute_type_predicate_of_signature(target);
        let source_pred = self.compute_type_predicate_of_signature(source);
        if let (Some(tp), Some(sp)) = (target_pred, source_pred)
            && tp.kind == sp.kind
            && tp.parameter_index == sp.parameter_index
            && tp.t.is_some()
            && sp.t.is_some()
        {
            let sp_t = sp.t.unwrap();
            let tp_t = tp.t.unwrap();
            self.infer_from_types(state, &sp_t, &tp_t);
            return;
        }
        let st = self.get_return_type_of_signature(source);
        let tt = self.get_return_type_of_signature(target);
        if let (Some(st), Some(tt)) = (st, tt) {
            if self.could_contain_type_variables(&tt) {
                self.infer_from_types(state, &st, &tt);
            }
        }
    }

    pub(crate) fn infer_from_index_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let mut priority = InferencePriority::None;
        if source.object_flags.intersects(ObjectFlags::Mapped)
            && target.object_flags.intersects(ObjectFlags::Mapped)
        {
            priority = InferencePriority::HomomorphicMappedType;
        }
        let target_infos = self.get_index_infos_of_type(target);
        if self.is_object_type_with_inferable_index(source) {
            for target_info in &target_infos {
                let Some(target_key) = target_info.key_type.clone() else {
                    continue;
                };
                let mut prop_types: Vec<Arc<Type>> = Vec::new();
                for prop in self.get_properties_of_type(source) {
                    let literal_key = self.get_literal_type_from_property(&prop);
                    if self.is_applicable_index_type(&literal_key, &target_key) {
                        let prop_type = self.get_type_of_symbol(&prop);
                        let prop_type = if prop.flags.contains(SymbolFlags::Optional) {
                            self.remove_missing_or_undefined_type(&prop_type)
                        } else {
                            prop_type
                        };
                        prop_types.push(prop_type);
                    }
                }
                for info in self.get_index_infos_of_type(source) {
                    if let Some(src_key) = &info.key_type
                        && self.is_applicable_index_type(src_key, &target_key)
                        && let Some(v) = &info.value_type
                    {
                        prop_types.push(Arc::clone(v));
                    }
                }
                if !prop_types.is_empty()
                    && let Some(tv) = &target_info.value_type
                {
                    let union = self.get_union_type(prop_types);
                    self.infer_with_priority(state, &union, tv, priority);
                }
            }
        }
        for target_info in &target_infos {
            let Some(target_key) = target_info.key_type.clone() else {
                continue;
            };
            if let Some(source_info) = self.get_applicable_index_info(source, &target_key)
                && let (Some(sv), Some(tv)) = (&source_info.value_type, &target_info.value_type)
            {
                self.infer_with_priority(state, sv, tv, priority);
            }
        }
    }

    pub(crate) fn infer_from_matching_types(
        &mut self,
        state: &mut InferenceState,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
        use_identical: bool,
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        let mut remaining_sources: Vec<Arc<Type>> = sources.to_vec();
        let mut remaining_targets: Vec<Arc<Type>> = targets.to_vec();
        let mut i = 0;
        while i < remaining_sources.len() {
            let mut matched = false;
            let mut j = 0;
            while j < remaining_targets.len() {
                let is_match = if use_identical {
                    self.is_type_identical_to(&remaining_sources[i], &remaining_targets[j])
                } else {
                    self.is_type_identical_to(&remaining_sources[i], &remaining_targets[j])
                };
                if is_match {
                    self.infer_from_types(state, &remaining_sources[i], &remaining_targets[j]);
                    remaining_sources.remove(i);
                    remaining_targets.remove(j);
                    matched = true;
                    break;
                }
                j += 1;
            }
            if !matched {
                i += 1;
            }
        }
        (remaining_sources, remaining_targets)
    }

    pub(crate) fn infer_matching_types_identical(
        &mut self,
        state: &mut InferenceState,
        sources: &[Arc<Type>],
        targets: &[Arc<Type>],
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        self.infer_from_matching_types(state, sources, targets, true)
    }

    pub(crate) fn infer_with_priority(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
        new_priority: InferencePriority,
    ) {
        if new_priority == InferencePriority::None {
            self.infer_from_types(state, source, target);
            return;
        }
        let save = state.priority;
        state.priority = new_priority;
        self.infer_from_types(state, source, target);
        state.priority = save;
    }

    pub(crate) fn could_contain_type_variables(&self, t: &Type) -> bool {
        t.flags.intersects(TypeFlags::Union)
            || t.flags.intersects(TypeFlags::Intersection)
            || t.flags.intersects(TypeFlags::Object)
            || t.flags.intersects(TypeFlags::TypeParameter)
            || t.flags.intersects(TypeFlags::IndexedAccess)
            || t.flags.intersects(TypeFlags::Conditional)
            || t.flags.intersects(TypeFlags::Substitution)
    }
    pub(crate) fn is_no_infer_type(&self, t: &Type) -> bool {
        // Go isNoInferType：内置 NoInfer 表示为 unknown 约束的 Substitution
        if t.flags.contains(TypeFlags::Substitution)
            && let TypeData::Substitution(sub) = &t.data
            && sub
                .constraint
                .as_ref()
                .is_some_and(|c| c.flags.contains(TypeFlags::Unknown))
        {
            return true;
        }
        // 用户自定义形态 [T][T extends any ? 0 : never]：延迟 IndexedAccess，
        // 索引为 extends any 的条件型，对象元组恰好含该条件型的 checkType
        if t.flags.contains(TypeFlags::IndexedAccess)
            && let TypeData::IndexedAccess(ia) = &t.data
            && let Some(index) = &ia.index_type
            && index.flags.contains(TypeFlags::Conditional)
            && let TypeData::Conditional(cond) = &index.data
            && cond
                .extends_type
                .as_ref()
                .is_some_and(|e| e.flags.contains(TypeFlags::Any))
            && let Some(check) = &cond.check_type
            && check.flags.contains(TypeFlags::TypeParameter)
            && let Some(object) = &ia.object_type
            && let TypeData::Tuple(tuple) = &object.data
            && tuple.element_infos.len() == 1
            && tuple.element_infos[0]
                .type_
                .as_ref()
                .is_some_and(|e| e.id == check.id)
        {
            return true;
        }
        false
    }

    pub(crate) fn is_from_inference_blocked_source(&self, _source: &Type) -> bool {
        false
    }

    // Go contextuallyCheckFunctionExpressionOrObjectLiteralMethod 的固定阶段：
    // 用固定实例化后的上下文签名重定型上下文敏感实参（参数写入符号缓存、返回类型重推）
    pub(crate) fn type_of_context_sensitive_arg(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        ctx_type: &Arc<Type>,
    ) -> Arc<Type> {
        // Go getContextualSignature：联合上下文逐成分取调用签名（跳过
        // undefined 等无签名成分），单一命中即用
        let ctx_sig = if let Some(members) = ctx_type.types().filter(|_| ctx_type.is_union()) {
            members
                .iter()
                .filter_map(|m| {
                    self.get_signatures_of_type(m, SignatureKind::Call)
                        .into_iter()
                        .next()
                })
                .next()
        } else {
            self.get_signatures_of_type(ctx_type, SignatureKind::Call)
                .into_iter()
                .next()
        };
        let Some(ctx_sig) = ctx_sig else {
            // 无上下文签名（callee 尚未定型等）：结果劣化，不冻结子树缓存，
            // 后续推断轮次用更好的上下文重算
            let t = self.get_type_of_node(node);
            self.clear_node_type_cache_under(node);
            return t;
        };
        let (parameters, body, type_node) = match &node.data {
            tsox_frontend::ast::NodeData::ArrowFunction(d) => {
                (&d.parameters, Some(&d.body), d.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::FunctionExpression(d) => {
                (&d.parameters, Some(&d.body), d.type_node.as_ref())
            }
            _ => return self.get_type_of_node(node),
        };
        let is_arrow = matches!(node.data, tsox_frontend::ast::NodeData::ArrowFunction(_));
        // 前一轮（部分推断）可能已把 body 定型为劣化类型并缓存，重定型前失效子树节点缓存
        self.clear_node_type_cache_under(node);
        if is_arrow {
            self.push_arrow_function_scope(node);
        } else {
            self.push_function_scope(node);
        }
        for (i, param) in parameters.iter().enumerate() {
            let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.type_node.is_some() {
                continue;
            }
            let is_rest = pd.dot_dot_dot_token.is_some();
            let is_this_param = i == 0
                && matches!(&pd.name.data, tsox_frontend::ast::NodeData::Identifier(id) if id.text == "this");
            let Some(sym) = self
                .program
                .symbol_map()
                .symbol_of(param)
                .cloned()
            else {
                continue;
            };
            if let Some(t) = self.contextual_param_type_at(
                &ctx_sig, parameters, i, param, is_rest, is_this_param,
            ) {
                self.value_symbol_links.insert(
                    &sym,
                    crate::checker::types_impl_chunk_3::ValueSymbolLinks {
                        resolved_type: Some(t),
                        ..Default::default()
                    },
                );
            }
        }
        let return_type = self.infer_function_return_type(body, type_node);
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }
        let sig = self.build_signature_from_function_like_type_node(
            parameters,
            return_type,
            false,
            None,
            Some(Arc::clone(node)),
        );
        self.create_function_or_constructor_type(vec![sig], false)
    }

    pub(crate) fn clear_node_type_cache_under(&mut self, node: &Arc<tsox_frontend::ast::Node>) {
        self.type_node_links.data.remove(&node.id());
        tsox_frontend::ast::node_data_generated::for_each_child(node, |c| {
            self.clear_node_type_cache_under(c);
            false
        });
    }

    pub fn infer_type_arguments(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        signature: &Arc<Signature>,
        args: &[Arc<tsox_frontend::ast::Node>],
        context: &mut InferenceContext,
    ) -> Vec<Arc<Type>> {
        if matches!(
            node.kind,
            SyntaxKind::CallExpression | SyntaxKind::NewExpression
        ) {
            if let Some(contextual_type) = self.get_contextual_type_for_call_or_new(node) {
                if let Some(return_type) = self.get_return_type_of_signature(signature) {
                    if self.could_contain_type_variables(&return_type) {
                        // Go createOuterReturnMapper：返回位推断前用外层推断语境的
                        // NoDefault 映射擦掉外层类型参数（→silentNever，不可推断），
                        // 使 NoInfer<T> 等外层包裹形态不向本签名泄漏候选
                        let own: Vec<Arc<Type>> = context
                            .inferences
                            .iter()
                            .map(|i| Arc::clone(&i.type_parameter))
                            .collect();
                        let mut outer: Vec<Arc<Type>> = Vec::new();
                        self.collect_outer_type_params(&contextual_type, &own, &mut outer, 0);
                        let inference_source = if outer.is_empty() {
                            contextual_type
                        } else {
                            let eraser = self.silent_never_type();
                            let subs = vec![eraser; outer.len()];
                            self.substitute_infer_type_parameters(&contextual_type, &outer, &subs)
                        };
                        self.infer_types(
                            &mut context.inferences,
                            Some(inference_source),
                            Some(return_type),
                            InferencePriority::ReturnType,
                            false,
                        );
                    }
                }
            }
        }

        let has_rest = signature.has_rest_parameter();
        let rest_index = if has_rest {
            signature.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        // Go chooseOverload 两阶段：上下文敏感实参（箭头/函数表达式含无注解参数）后置，
        // 先由非敏感实参固定类型参数，再用固定后的实例化上下文签名重定型敏感实参
        let has_cs_args = !signature.type_parameters.is_empty()
            && args.iter().any(|a| self.is_context_sensitive(a));
        let order: Vec<usize> = if has_cs_args {
            let mut ordered: Vec<usize> = (0..args.len())
                .filter(|&i| !self.is_context_sensitive(&args[i]))
                .collect();
            ordered.extend((0..args.len()).filter(|&i| self.is_context_sensitive(&args[i])));
            ordered
        } else {
            (0..args.len()).collect()
        };
        // Go inferTypeArguments + chooseOverload 两阶段：
        // 候选选择阶段 CS 实参以 any 检查（不产生推断），选中后二阶段以
        // 非CS实参推断固定类型参数（isFixed，字面量拓宽），再用固定后的
        // 上下文定型 CS 实参；固定结果缓存为最终推断值
        let mut cs_snapshot: Option<Vec<(Vec<Arc<Type>>, Vec<Arc<Type>>)>> = None;
        let mut cs_fixed: Option<Vec<Arc<Type>>> = None;
        for i in order {
            let param_type = if has_rest && i >= rest_index {
                let rest_type = self
                    .try_get_type_at_position(signature, rest_index)
                    .unwrap_or_else(|| self.get_type_of_symbol(&signature.parameters[rest_index]));
                self.get_array_element_type(&rest_type)
            } else if i < signature.parameters.len() {
                self.signature_instantiated_param_type(signature, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(&signature.parameters[i]))
            } else {
                continue;
            };
            if self.could_contain_type_variables(&param_type) {
                let is_cs = has_cs_args && self.is_context_sensitive(&args[i]);
                if is_cs {
                    if cs_snapshot.is_none() {
                        cs_snapshot = Some(
                            context
                                .inferences
                                .iter()
                                .map(|info| (info.candidates.clone(), info.contra_candidates.clone()))
                                .collect(),
                        );
                        // fixTypeParameters：已有候选的类型参数固定（isFixed 后部分推断
                        // 拓宽字面量并缓存），无候选的保持可推断
                        for info in context.inferences.iter_mut() {
                            if !info.candidates.is_empty() || !info.contra_candidates.is_empty() {
                                info.is_fixed = true;
                            }
                        }
                        let fixed = self.get_inferred_types(context);
                        for (info, t) in context.inferences.iter_mut().zip(fixed.iter()) {
                            info.inferred_type = Some(Arc::clone(t));
                        }
                        cs_fixed = Some(fixed);
                    }
                    let saved = cs_snapshot.as_ref().expect("snapshot exists for CS args");
                    for (info, (cands, contra)) in context.inferences.iter_mut().zip(saved) {
                        info.candidates = cands.clone();
                        info.contra_candidates = contra.clone();
                    }
                    let partial = cs_fixed.clone().expect("fixed types exist");
                    let inst_param = self.substitute_infer_type_parameters(
                        &param_type,
                        &signature.type_parameters,
                        &partial,
                    );
                    let arg_type = self.type_of_context_sensitive_arg(&args[i], &inst_param);
                    if std::env::var_os("TSOX_DEBUG_HOVER").is_some() {
                        eprintln!(
                            "[infer-cs] inst_param={} arg_type={}",
                            self.type_to_string(&inst_param),
                            self.type_to_string(&arg_type)
                        );
                    }
                    self.infer_types(
                        &mut context.inferences,
                        Some(arg_type),
                        Some(param_type),
                        InferencePriority::None,
                        false,
                    );
                    // 固定结果为最终值：仅锁定快照时已有候选（is_fixed）的类型参数；
                    // 快照时无候选的（约束回退是占位）由本阶段新候选参与最终推断
                    let fixed = cs_fixed.as_ref().expect("fixed types exist");
                    for (info, t) in context.inferences.iter_mut().zip(fixed.iter()) {
                        if info.is_fixed && !self.is_uninferred_type(t) {
                            info.candidates = Vec::new();
                            info.contra_candidates = Vec::new();
                            info.inferred_type = Some(Arc::clone(t));
                        }
                    }
                } else {
                    let mut arg_type = self.get_type_of_node(&args[i]);
                    // Go checkExpressionWithContextualType 尾部：字面量是其
                    // 上下文型（含类型参数约束）的成员时剥 fresh 标记，
                    // 候选保留字面量（createColor('rgb', …) → T='rgb'）
                    if arg_type
                        .flags
                        .intersects(crate::checker::types::TYPE_FLAGS_LITERAL)
                        && self.is_literal_of_contextual_type(&arg_type, &param_type)
                    {
                        arg_type = self.get_regular_type_of_literal_type(&arg_type);
                    }
                    self.infer_types(
                        &mut context.inferences,
                        Some(arg_type),
                        Some(param_type),
                        InferencePriority::None,
                        false,
                    );
                }
            }
        }
        let result = self.get_inferred_types(context);
        result
    }
}

impl Checker {
    /// 推断缺省回退（unknown）视为未推断，二阶段候选可补充
    fn is_uninferred_type(&self, t: &Arc<Type>) -> bool {
        t.flags.contains(TypeFlags::Unknown)
    }
}

impl Checker {
    /// Go getRestTypeAtPosition 的目标 rest 对位：pos 超出固定参数时，
    /// rest 数组按元素展开（泛型 T 保持延迟 T[number]，具体类型则解析）
    pub(crate) fn source_rest_type_at(&mut self, sig: &Arc<Signature>, pos: usize) -> Arc<Type> {
        let parameter_count = self.get_parameter_count(sig);
        if let Some(rest) = self.get_effective_rest_type(sig) {
            if pos >= parameter_count.saturating_sub(1) {
                if pos == parameter_count.saturating_sub(1) {
                    return rest;
                }
                let number = self.number_type();
                let element = if rest.flags.contains(TypeFlags::TypeParameter) {
                    let mut ia = Type::new(
                        TypeFlags::IndexedAccess,
                        TypeData::IndexedAccess(crate::checker::types::IndexedAccessTypeData {
                            constrained: Default::default(),
                            object_type: Some(Arc::clone(&rest)),
                            index_type: Some(number),
                            access_flags: Default::default(),
                        }),
                    );
                    ia.symbol = rest.symbol.clone();
                    Arc::new(ia)
                } else {
                    self.get_indexed_access_type(&rest, &number)
                };
                return self.create_array_type(element);
            }
        }
        let length = parameter_count.saturating_sub(pos);
        if length == 0 {
            return self.create_tuple_type_ex(Vec::new(), Vec::new(), false);
        }
        // Go getRestTypeAtPosition：元素带标签（源参数名）与 Required/
        // Optional/Variadic 标志，签名的 rest 元组展示为具名参数序列
        let min_argument_count = self.get_min_argument_count(sig);
        let rest = self.get_effective_rest_type(sig);
        let mut types: Vec<Arc<Type>> = Vec::with_capacity(length);
        let mut infos: Vec<crate::checker::types::TupleElementInfo> = Vec::with_capacity(length);
        for i in 0..length {
            let p = i + pos;
            if rest.is_some() && i == length - 1 {
                types.push(rest.clone().expect("checked above"));
                let label = sig.parameters.get(p).map(|s| s.name.clone());
                infos.push(crate::checker::types::TupleElementInfo {
                    label,
                    flags: ElementFlags::Variadic,
                    labeled_declaration: None,
                    type_: None,
                });
            } else {
                types.push(self.get_type_at_position(sig, p));
                let flags = if p < min_argument_count {
                    ElementFlags::Required
                } else {
                    ElementFlags::Optional
                };
                let label = sig.parameters.get(p).map(|s| s.name.clone());
                infos.push(crate::checker::types::TupleElementInfo {
                    label,
                    flags,
                    labeled_declaration: None,
                    type_: None,
                });
            }
        }
        self.create_tuple_type_ex(types, infos, false)
    }
}

impl Checker {
    /// 收集类型中不属于本签名推断列表的类型参数（外层类型参数）
    pub(crate) fn collect_outer_type_params(
        &mut self,
        t: &Arc<Type>,
        own: &[Arc<Type>],
        out: &mut Vec<Arc<Type>>,
        depth: usize,
    ) {
        if depth > 6 {
            return;
        }
        let is_own = |c: &Arc<Type>| {
            own.iter()
                .any(|p| crate::checker::utilities::type_parameters_match(p, c))
        };
        match &t.data {
            TypeData::TypeParameter(_) => {
                if !is_own(t) && !out.iter().any(|p| p.id == t.id) {
                    out.push(Arc::clone(t));
                }
            }
            TypeData::Union(_) | TypeData::Intersection(_) => {
                let members: Vec<Arc<Type>> = t
                    .types()
                    .map(|ms| ms.to_vec())
                    .unwrap_or_default();
                for m in members {
                    self.collect_outer_type_params(&m, own, out, depth + 1);
                }
            }
            TypeData::Tuple(tup) => {
                for elem in &tup.element_infos {
                    if let Some(e) = &elem.type_ {
                        self.collect_outer_type_params(e, own, out, depth + 1);
                    }
                }
            }
            TypeData::IndexedAccess(ia) => {
                if let Some(o) = &ia.object_type {
                    self.collect_outer_type_params(o, own, out, depth + 1);
                }
                if let Some(idx) = &ia.index_type {
                    self.collect_outer_type_params(idx, own, out, depth + 1);
                }
            }
            TypeData::Conditional(c) => {
                if let Some(chk) = &c.check_type {
                    self.collect_outer_type_params(chk, own, out, depth + 1);
                }
                if let Some(ext) = &c.extends_type {
                    self.collect_outer_type_params(ext, own, out, depth + 1);
                }
                if let Some(root) = &c.root {
                    if let Some(chk) = &root.check_type {
                        self.collect_outer_type_params(chk, own, out, depth + 1);
                    }
                    if let Some(ext) = &root.extends_type {
                        self.collect_outer_type_params(ext, own, out, depth + 1);
                    }
                }
            }
            TypeData::Object(o) => {
                for arg in &o.type_arguments {
                    self.collect_outer_type_params(arg, own, out, depth + 1);
                }
                for info in &o.structured.index_infos {
                    if let Some(v) = &info.value_type {
                        self.collect_outer_type_params(v, own, out, depth + 1);
                    }
                }
                let props = self.get_properties_of_type(t);
                for p in props.into_iter().take(32) {
                    let pt = self.get_type_of_symbol(&p);
                    self.collect_outer_type_params(&pt, own, out, depth + 1);
                }
            }
            _ => {}
        }
    }
}
