#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub(crate) fn get_contextual_type_for_argument(
        &mut self,
        call_node: &Arc<tsox_frontend::ast::Node>,
        arg_node: &Arc<tsox_frontend::ast::Node>,
    ) -> Option<Arc<Type>> {
        self.get_contextual_type_for_argument_ex(call_node, arg_node, ContextFlags::None)
    }

    // Go runWithInferenceBlockedFromSourceNode：IgnoreNodeInferences 把当前节点
    // （到包含调用为止）从推断源剔除，completionsType 回落到未代入的参数型
    pub(crate) fn get_contextual_type_for_argument_ex(
        &mut self,
        call_node: &Arc<tsox_frontend::ast::Node>,
        arg_node: &Arc<tsox_frontend::ast::Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;

        let args = match &call_node.data {
            NodeData::CallExpression(data) => Some(&data.arguments),
            NodeData::NewExpression(data) => data.arguments.as_ref(),
            _ => None,
        }?;


        let arg_index = args.iter().position(|a| Arc::ptr_eq(a, arg_node))?;

        let is_new = matches!(&call_node.data, NodeData::NewExpression(_));
        let expression_type = match &call_node.data {
            NodeData::CallExpression(data) => Some(self.get_type_of_node(&data.expression)),
            NodeData::NewExpression(data) => Some(self.get_type_of_node(&data.expression)),
            _ => None,
        }?;
        let kind = if is_new {
            SignatureKind::Construct
        } else {
            SignatureKind::Call
        };
        let signatures = self.get_signatures_of_type(&expression_type, kind);

        // 全重载不可适用时用联合签名（参数位=各重载并集，Go
        // getCandidateForOverloadFailure → createUnionOfSignaturesForOverloadFailure）；
        // 重载判定会回查实参类型（自递归，见 resolving_contextual_calls）
        let resolved: Option<Arc<Signature>> = {
            let key = call_node.id();
            if self.resolving_contextual_calls.insert(key) {
                let found = self.find_matching_signature_opt(call_node, &signatures, args);
                let combined = match found {
                    Some(idx) => Some(Arc::clone(&signatures[idx])),
                    None => self.candidate_for_overload_failure(call_node, &signatures, args),
                };
                self.resolving_contextual_calls.remove(&key);
                combined
            } else {
                None
            }
        };
        let sig = match resolved {
            Some(s) => s,
            None => signatures
                .iter()
                .find(|s| s.parameters.len() > arg_index)
                .or_else(|| signatures.first())?
                .clone(),
        };

        if arg_index >= sig.parameters.len() && !sig.has_rest_parameter() {
            return None;
        }

        let base_param_type = self
            .try_get_type_at_position(&sig, arg_index)
            .or_else(|| {
                // rest 位（含联合签名的合成 rest 数组）给元素类型
                if sig.has_rest_parameter() && arg_index >= sig.parameters.len() - 1 {
                    let rest = self.get_type_of_symbol(sig.parameters.last()?);
                    Some(self.get_array_element_type_of(&rest).unwrap_or(rest))
                } else {
                    None
                }
            })
            .or_else(|| {
                sig.parameters
                    .get(arg_index)
                    .map(|p| self.get_type_of_symbol(p))
            })
            .unwrap_or_else(|| self.any_type());

        if !sig.type_parameters.is_empty() {
            let key = call_node.id();
            if self.resolving_contextual_calls.insert(key) {
                let ignore_node = context_flags
                    .contains(ContextFlags::IgnoreNodeInferences);
                // 调用位显式类型实参优先（Go inferSignature：显式实参直接
                // 固定映射，不从实参推断；错误实参落 error 型）
                let explicit: Option<Vec<Arc<Type>>> = match &call_node.data {
                    NodeData::CallExpression(d) => d.type_arguments.as_ref().map(|ta| {
                        ta.iter().map(|t| self.get_type_from_type_node(t)).collect()
                    }),
                    NodeData::NewExpression(d) => d.type_arguments.as_ref().map(|ta| {
                        ta.iter().map(|t| self.get_type_from_type_node(t)).collect()
                    }),
                    _ => None,
                };
                let sibling_args: Vec<Arc<tsox_frontend::ast::Node>> = args
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| !(ignore_node && Arc::ptr_eq(a, arg_node)))
                    .map(|(_, a)| Arc::clone(a))
                    .collect();
                let inferred = match &explicit {
                    Some(ex) if ex.len() == sig.type_parameters.len() => ex.clone(),
                    _ => self.infer_call_type_arguments(call_node, &sig, &sibling_args),
                };
                if std::env::var("TSOX_DEBUG_SUBST").is_ok() {
                    let rendered: Vec<String> = inferred.iter().map(|t| self.type_to_string(t)).collect();
                    let node_text = self.node_source_text(call_node).unwrap_or_default();
                    let exp_len = explicit.as_ref().map(|e| e.len());
                    eprintln!("[ctx-arg-explicit] used={} sig_tps={} exp={:?} call=`{}` -> [{}]",
                        explicit.is_some() && explicit.as_ref().is_some_and(|e| e.len() == sig.type_parameters.len()),
                        sig.type_parameters.len(), exp_len, node_text.chars().take(40).collect::<String>(), rendered.join(", "));
                }
                self.resolving_contextual_calls.remove(&key);
                if !inferred.is_empty() {
                    let substed = self.substitute_infer_type_parameters(
                        &base_param_type,
                        &sig.type_parameters,
                        &inferred,
                    );
                    return Some(substed);
                }
            }
        }
        Some(base_param_type)
    }

    pub(crate) fn get_contextual_type_for_binary_operand(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;

        let parent = node.parent()?;
        let binary = match &parent.data {
            NodeData::BinaryExpression(data) => data,
            _ => return None,
        };

        if !Arc::ptr_eq(node, &binary.right) {
            return None;
        }

        match binary.operator_token.kind {
            SyntaxKind::EqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => self
                .assignment_target_type(&binary.left)
                .or_else(|| Some(self.get_type_of_node(&binary.left))),
            SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken => {
                let binary_ctx = self.get_contextual_type(&parent, _context_flags);
                if Arc::ptr_eq(node, &binary.right) && binary_ctx.is_none() {
                    return Some(self.get_type_of_node(&binary.left));
                }
                binary_ctx
            }
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::CommaToken => {
                self.get_contextual_type(&parent, _context_flags)
            }
            _ => None,
        }
    }

    pub(crate) fn get_contextual_type_for_object_literal_element(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;

        let object_literal = node.parent()?;

        let contextual_type = self.get_contextual_type(&object_literal, _context_flags)?;

        let name = match &node.data {
            NodeData::PropertyAssignment(data) => match &data.name.data {
                NodeData::Identifier(id) => Some(id.text.clone()),
                NodeData::StringLiteral(s) => Some(s.text.clone()),
                // 计算属性名 `[Foo]`：按标识符文本查上下文成员
                //（Go getContextualTypeForObjectLiteralElement 的
                // getLiteralTypeFromPropertyName + findApplicableIndexInfo 通道，
                // 映射型成员经索引信息命中；此处以文本键等价）
                NodeData::ComputedPropertyName(cd)
                    if matches!(&cd.expression.data, NodeData::Identifier(_)) =>
                {
                    match &cd.expression.data {
                        NodeData::Identifier(id) => Some(id.text.clone()),
                        _ => None,
                    }
                }
                _ => None,
            },
            NodeData::ShorthandPropertyAssignment(data) => match &data.name.data {
                NodeData::Identifier(id) => Some(id.text.clone()),
                _ => None,
            },
            _ => None,
        }?;

        self.get_type_of_property_of_contextual_type(&contextual_type, &name)
    }

    pub(crate) fn get_contextual_type_for_array_literal_element(
        &mut self,
        _node: &tsox_frontend::ast::Node,
        parent: &Arc<tsox_frontend::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let contextual_type = self.get_contextual_type(parent, _context_flags)?;

        let type_args = self.get_type_arguments(&contextual_type);
        if !type_args.is_empty() {
            return Some(Arc::clone(&type_args[0]));
        }

        if let Some(structured) = contextual_type.as_structured() {
            for index_info in &structured.index_infos {
                if let Some(ref value_type) = index_info.value_type {
                    return Some(Arc::clone(value_type));
                }
            }
        }

        None
    }

    pub(crate) fn get_covariant_inference(
        &mut self,
        inference: &InferenceInfo,
        _signature: &Arc<Signature>,
    ) -> Option<Arc<Type>> {
        // Go non-fixing mapper：候选即类型参数自身（自引用推断）不参与
        let filtered: Vec<Arc<Type>> = inference
            .candidates
            .iter()
            .filter(|c| {
                !crate::checker::utilities::type_parameters_match(c, &inference.type_parameter)
            })
            .cloned()
            .collect();
        if filtered.is_empty() {
            return None;
        }
        let candidates = self.union_object_and_array_literal_candidates(&filtered);
        // Go convertAutoToAny/widen：widening 标记候选（auto、undefinedWidening 等
        // 内部标记型）按 any 参与联合（_.all([], ...) → T=any）
        let candidates: Vec<Arc<Type>> = candidates
            .into_iter()
            .map(|t| {
                if t.object_flags
                    .contains(crate::checker::types::ObjectFlags::ContainsWideningType)
                {
                    self.get_any_type()
                } else {
                    t
                }
            })
            .collect();
        let primitive_constraint = self.has_primitive_constraint(&inference.type_parameter)
            || self.is_const_type_variable(&inference.type_parameter, 0);
        let widen_literal_types = !primitive_constraint
            && inference.top_level
            && (inference.is_fixed
                || !self.is_type_parameter_at_top_level_in_return_type(
                    _signature,
                    &inference.type_parameter,
                ));
        let base_candidates: Vec<Arc<Type>> = if primitive_constraint {
            candidates
                .iter()
                .map(|t| self.get_regular_type_of_literal_type(t))
                .collect()
        } else if widen_literal_types {
            candidates
                .iter()
                .map(|t| self.get_widened_literal_type(t))
                .collect()
        } else {
            candidates
        };
        let unwidened_type = if inference
            .priority
            .contains(InferencePriority::PriorityImpliesCombination)
        {
            self.get_union_type(base_candidates)
        } else {
            self.get_common_supertype(&base_candidates)
        };
        // Go getWidenedType 不拓宽顶层字面量类型；推断结果保留字面量（f<2>(a: 2)）
        if unwidened_type.flags.intersects(crate::checker::types::TYPE_FLAGS_LITERAL) {
            return Some(unwidened_type);
        }
        Some(self.get_widened_type(&unwidened_type))
    }

    pub(crate) fn get_contravariant_inference(
        &mut self,
        inference: &InferenceInfo,
    ) -> Option<Arc<Type>> {
        if inference.contra_candidates.is_empty() {
            return None;
        }
        if inference
            .priority
            .contains(InferencePriority::PriorityImpliesCombination)
        {
            Some(self.get_intersection_type(inference.contra_candidates.clone()))
        } else {
            Some(self.get_common_subtype(&inference.contra_candidates))
        }
    }

    pub(crate) fn union_object_and_array_literal_candidates(
        &self,
        candidates: &[Arc<Type>],
    ) -> Vec<Arc<Type>> {
        if candidates.len() > 1 {
            let object_literals: Vec<Arc<Type>> = candidates
                .iter()
                .filter(|t| self.is_object_or_array_literal_type(t))
                .cloned()
                .collect();
            if !object_literals.is_empty() {
                let literals_type = self.create_union_type(object_literals);
                let non_literal_types: Vec<Arc<Type>> = candidates
                    .iter()
                    .filter(|t| !self.is_object_or_array_literal_type(t))
                    .cloned()
                    .collect();
                let mut result = non_literal_types;
                result.push(literals_type);
                return result;
            }
        }
        candidates.to_vec()
    }

    pub(crate) fn has_primitive_constraint(&self, t: &Arc<Type>) -> bool {
        let constraint = self.get_constraint_of_type_parameter(t);
        if let Some(constraint) = constraint {
            let c = if constraint.flags.contains(TypeFlags::Conditional) {
                self.get_default_constraint_of_conditional_type(&constraint)
            } else {
                Some(constraint)
            };
            if let Some(c) = c {
                return self.maybe_type_of_kind(
                    &c,
                    TypeFlags::String
                        | TypeFlags::Number
                        | TypeFlags::BigInt
                        | TypeFlags::Boolean
                        | TypeFlags::ESSymbol
                        | TypeFlags::Enum
                        | TypeFlags::Index
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                );
            }
        }
        false
    }

    pub(crate) fn is_type_parameter_at_top_level(&self, t: &Type, tp: &Type, depth: i32) -> bool {
        if crate::checker::utilities::type_parameters_match(t, tp) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union | TypeFlags::Intersection) {
            if let Some(types) = t.types() {
                return types
                    .iter()
                    .any(|tt| self.is_type_parameter_at_top_level(tt, tp, depth));
            }
        }
        false
    }
}
