#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub(crate) fn infer_from_object_types(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
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
        if source.object_flags.contains(ObjectFlags::Reference)
            && target.object_flags.contains(ObjectFlags::Reference)
            && (same_target || self.is_array_type(source) && self.is_array_type(target))
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
        let param_count = source.parameters.len().min(target.parameters.len());
        for i in 0..param_count {
            let source_param = &source.parameters[i];
            let target_param = &target.parameters[i];
            let st = self.get_type_of_symbol(source_param);
            let tt = self.get_type_of_symbol(target_param);
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
        state.bivariant = save_biv;

        // Go applyToReturnTypes：仅当 target 返回类型含类型变量时才推断
        // （此前无条件推断会向无类型变量的 target 灌入垃圾候选）
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
        let source_struct = source.as_structured();
        let target_struct = target.as_structured();
        if let (Some(source_s), Some(target_s)) = (source_struct, target_struct) {
            for target_index in &target_s.index_infos {
                for source_index in &source_s.index_infos {
                    let key_match = match (&target_index.key_type, &source_index.key_type) {
                        (Some(tk), Some(sk)) => self.is_type_identical_to(sk, tk),
                        _ => true,
                    };
                    if key_match {
                        if let (Some(tv), Some(sv)) =
                            (&target_index.value_type, &source_index.value_type)
                        {
                            self.infer_from_types(state, sv, tv);
                        }
                    }
                }
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
    pub(crate) fn is_no_infer_type(&self, _t: &Type) -> bool {
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
        let Some(ctx_sig) = self
            .get_signatures_of_type(ctx_type, SignatureKind::Call)
            .into_iter()
            .next()
        else {
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
        node: &tsox_frontend::ast::Node,
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
                        self.infer_types(
                            &mut context.inferences,
                            Some(contextual_type),
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
                    // 阶段二（上下文敏感实参）：快照候选，其推断结果只补充尚无候选的类型参数
                    // （tsc checkArguments 中 CS 实参经上下文定型后以 ReturnType 优先级补推断，
                    // 不覆盖已由普通实参固定的类型参数）
                    let saved: Vec<(Vec<Arc<Type>>, Vec<Arc<Type>>)> = context
                        .inferences
                        .iter()
                        .map(|info| (info.candidates.clone(), info.contra_candidates.clone()))
                        .collect();
                    let partial = self.get_inferred_types(context);
                    let inst_param = self.substitute_infer_type_parameters(
                        &param_type,
                        &signature.type_parameters,
                        &partial,
                    );
                    let arg_type = self.type_of_context_sensitive_arg(&args[i], &inst_param);
                    self.infer_types(
                        &mut context.inferences,
                        Some(arg_type),
                        Some(param_type),
                        InferencePriority::None,
                        false,
                    );
                    for (info, (cands, contra)) in context.inferences.iter_mut().zip(saved) {
                        if !cands.is_empty() || !contra.is_empty() {
                            info.candidates = cands;
                            info.contra_candidates = contra;
                            info.inferred_type = None;
                        }
                    }
                } else {
                    let arg_type = self.get_type_of_node(&args[i]);
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

        self.get_inferred_types(context)
    }
}
