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
        // Go getUnionSignatures 的上下文取用近似：联合 callee 的调用签名取
        // 首个含签名的成分（成分序即声明序，RegExpMatchArray | [] 的 map 取前者）
        let signatures = if expression_type.is_union() {
            let mut sigs: Vec<Arc<Signature>> = Vec::new();
            if let Some(constituents) = expression_type.types().map(|ts| ts.to_vec()) {
                for c in constituents {
                    if c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                        continue;
                    }
                    sigs = self.get_signatures_of_type(&c, kind);
                    if !sigs.is_empty() {
                        break;
                    }
                }
            }
            sigs
        } else {
            self.get_signatures_of_type(&expression_type, kind)
        };

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
                };                self.resolving_contextual_calls.remove(&key);
                if !inferred.is_empty() {
                    let substed = self.substitute_infer_type_parameters(
                        &base_param_type,
                        &sig.type_parameters,
                        &inferred,
                    );                    return Some(substed);
                }
            }
        }        Some(base_param_type)
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

        // Go getContextualTypeForObjectLiteralElement 经
        // getApparentTypeOfContextualType：对象字面量的联合上下文型先经
        // 判别式成员筛选（discriminateContextualTypeByObjectMembers）
        let contextual_type = if contextual_type.is_union() {
            self.discriminate_contextual_type_by_object_members(&object_literal, &contextual_type)
        } else {
            contextual_type
        };

        let name = match &node.data {
            // Go getContextualTypeForObjectLiteralElement：统一走
            // getLiteralTypeFromPropertyName（well-known `Symbol.x` 计算名映射
            // 为内部名 `__@x`，与 binder 成员键一致）
            NodeData::PropertyAssignment(data) => {
                let name = self.get_property_name_from_node(&data.name);
                (!name.is_empty()).then_some(name)
            }
            NodeData::ShorthandPropertyAssignment(data) => {
                let name = self.get_property_name_from_node(&data.name);
                (!name.is_empty()).then_some(name)
            }
            // 方法成员 `m(n) { }` 的上下文型 = 字面量上下文的同名属性
            NodeData::MethodDeclaration(data) => {
                let name = self.get_property_name_from_node(&data.name);
                (!name.is_empty()).then_some(name)
            }
            _ => None,
        }?;

        self.get_type_of_property_of_contextual_type(&contextual_type, &name)
    }

    pub(crate) fn get_contextual_type_for_array_literal_element(
        &mut self,
        node: &tsox_frontend::ast::Node,
        parent: &Arc<tsox_frontend::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let contextual_type = self.get_contextual_type(parent, _context_flags)?;

        let elements = match &parent.data {
            tsox_frontend::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return None,
        };
        let index = elements.iter().position(|e| e.id() == node.id())?;
        let length = elements.len() as i64;
        let first_spread = elements
            .iter()
            .position(|e| e.kind == tsox_frontend::ast::SyntaxKind::SpreadElement)
            .map(|i| i as i64)
            .unwrap_or(-1);
        let last_spread = elements
            .iter()
            .rposition(|e| e.kind == tsox_frontend::ast::SyntaxKind::SpreadElement)
            .map(|i| i as i64)
            .unwrap_or(-1);

        let constituents: Vec<Arc<Type>> = match &contextual_type.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .filter(|c| {
                    !c.flags.intersects(crate::checker::types::TypeFlags::Null | crate::checker::types::TypeFlags::Undefined)
                })
                .cloned()
                .collect(),
            _ => vec![Arc::clone(&contextual_type)],
        };

        let mut results: Vec<Arc<Type>> = Vec::new();
        for c in constituents {
            if let Some(rt) =
                self.contextual_element_of_constituent(&c, index, length, first_spread, last_spread)
            {
                results.push(rt);
            }
        }
        match results.len() {
            0 => None,
            1 => Some(results.into_iter().next().unwrap()),
            _ => Some(self.get_union_type(results)),
        }
    }

    fn contextual_element_of_constituent(
        &mut self,
        t: &Arc<Type>,
        index: usize,
        length: i64,
        first_spread: i64,
        last_spread: i64,
    ) -> Option<Arc<Type>> {
        if crate::checker::utilities::is_tuple_type(t) {
            let element_types = Self::tuple_type_arguments(t);
            let (fixed_length, combined_flags) = match &t.data {
                TypeData::Tuple(tuple) => (tuple.fixed_length, tuple.combined_flags),
                _ => return None,
            };
            let idx = index as i64;
            if (first_spread < 0 || idx < first_spread) && idx < fixed_length as i64 {
                return element_types.get(index).cloned();
            }
            let mut offset = 0;
            if length >= 0 && (last_spread < 0 || idx > last_spread) {
                offset = length - idx;
            }
            let mut fixed_end_length = 0;
            if offset > 0 && combined_flags.intersects(crate::checker::types::ELEMENT_FLAGS_VARIABLE)
            {
                fixed_end_length = Self::end_fixed_element_count(t) as i64;
            }
            if offset > 0 && offset <= fixed_end_length {
                return element_types
                    .get(element_types.len() - offset as usize)
                    .cloned();
            }
            let mut tuple_index = fixed_length as i64;
            if first_spread >= 0 {
                tuple_index = tuple_index.min(first_spread);
            }
            let mut end_skip_count = fixed_end_length;
            if length >= 0 && last_spread >= 0 {
                end_skip_count = end_skip_count.min(length - last_spread);
            }
            let start = (tuple_index.max(0) as usize).min(element_types.len());
            let end = ((element_types.len() as i64 - end_skip_count.max(0)).max(0) as usize)
                .min(element_types.len());
            if start < end {
                let middle: Vec<Arc<Type>> = element_types[start..end].to_vec();
                return Some(self.get_union_type(middle));
            }
            return None;
        }
        let idx = index as i64;
        if first_spread < 0 || idx < first_spread {
            if let Some(prop) =
                self.get_type_of_property_of_contextual_type(t, &index.to_string())
            {
                return Some(prop);
            }
        }
        if let Some(elem) = self.get_type_arguments(t).into_iter().next() {
            return Some(elem);
        }
        if self.is_array_type(t) {
            return Some(self.get_array_element_type(t));
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
