#![allow(unused_imports)]

use crate::checker::checker_calls::*;

impl Checker {
    pub(crate) fn report_no_overload_matches(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> bool {
        let saved = self.diagnostics.take_inner();
        let mut entries: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        
        for sig in signatures.iter() {
            
            if !self.candidate_reaches_argument_check(node, sig, arguments) {
                continue;
            }
            if let Some(d) = self.probe_first_argument_error(node, sig, arguments) {
                entries.push(d);
            }
        }
        let _probe_only = self.diagnostics.take_inner();
        self.diagnostics.set_inner(saved);
        if entries.is_empty() {
            return false;
        }
        // Go reportCallResolutionErrors：candidatesForArgumentError 非空时取
        // 最后一个候选的实参错误链；候选数 >1 时包「最后一个过载给出如下错误」
        let anchor = entries.first().map(|d| d.loc).unwrap_or(node.loc);
        let multiple = entries.len() > 1;
        let last_entry = entries
            .pop()
            .expect("non-empty entries checked above");
        if multiple {
            let file = self.current_file.clone();
            let mut head = tsox_frontend::ast::Diagnostic::new(
                file,
                anchor,
                tsox_core::diagnostics::messages_generated::NO_OVERLOAD_MATCHES_THIS_CALL,
                Vec::new(),
            );
            let mut last_overload = tsox_frontend::ast::Diagnostic::new(
                None,
                anchor,
                tsox_core::diagnostics::messages_generated::THE_LAST_OVERLOAD_GAVE_THE_FOLLOWING_ERROR,
                Vec::new(),
            );
            last_overload.message_chain = vec![last_entry];
            head.message_chain = vec![last_overload];
            self.diagnostics.add(head);
        } else {
            let file = self.current_file.clone();
            let mut single = last_entry;
            single.file = file;
            self.diagnostics.add(single);
        }
        true
    }

    pub(crate) fn candidate_reaches_argument_check(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> bool {
        let arg_count = arguments.len();
        let max_params = if sig.has_rest_parameter() {
            usize::MAX
        } else {
            sig.parameters.len()
        };
        if arg_count > max_params || arg_count < sig.min_argument_count.max(0) as usize {
            return false;
        }
        let provided = Self::explicit_type_argument_count(node);
        if provided != 0
            && (provided > sig.type_parameters.len()
                || provided < Self::declared_min_type_argument_count(sig))
        {
            return false;
        }
        !self.explicit_type_args_violate_constraints(node, sig)
    }

    pub(crate) fn explicit_type_args_violate_constraints(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
    ) -> bool {
        let type_arg_nodes: Vec<Arc<Node>> = match &node.data {
            tsox_frontend::ast::NodeData::CallExpression(d) => d.type_arguments.as_ref(),
            tsox_frontend::ast::NodeData::NewExpression(d) => d.type_arguments.as_ref(),
            _ => None,
        }
        .map(|l| l.iter().cloned().collect())
        .unwrap_or_default();
        if type_arg_nodes.is_empty() || sig.type_parameters.is_empty() {
            return false;
        }
        let tps = sig.type_parameters.clone();
        let arg_types: Vec<Arc<Type>> = type_arg_nodes
            .iter()
            .map(|t| self.get_type_from_type_node(t))
            .collect();
        let class_subst = self.receiver_class_type_argument_substitution(node);
        for i in 0..type_arg_nodes.len().min(tps.len()) {
            let Some(constraint) = self.get_constraint_of_type_parameter(&tps[i]) else {
                continue;
            };
            let constraint = self.substitute_infer_type_parameters(&constraint, &tps, &arg_types);
            let constraint = match &class_subst {
                Some((class_tps, class_args)) => {
                    self.substitute_infer_type_parameters(&constraint, class_tps, class_args)
                }
                None => constraint,
            };
            let arg_type = Arc::clone(&arg_types[i]);
            if arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                || self.is_error_type(&arg_type)
                || self.is_error_type(&constraint)
                || self.degraded_type_ptrs.contains(&arg_type.id)
                || self.degraded_type_ptrs.contains(&constraint.id)
                || keeps_unsubstituted_type_parameter(&constraint)
            {
                continue;
            }
            if !self.is_type_assignable_to(&arg_type, &constraint) {
                return true;
            }
        }
        false
    }

    pub(crate) fn probe_first_argument_error(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> Option<tsox_frontend::ast::Diagnostic> {
        let arg_count = arguments.len();
        let max_params = if sig.has_rest_parameter() {
            usize::MAX
        } else {
            sig.parameters.len()
        };
        if arg_count > max_params || arg_count < sig.min_argument_count.max(0) as usize {
            return None;
        }
        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {
            match self.signature_instantiated_param_type(sig, rest_index) {
                Some(arr) => Some(self.get_array_element_type(&arr)),
                None => match self.try_get_type_at_position(sig, rest_index) {
                    Some(t) => Some(t),
                    None => {
                        let rest_param_type = self.get_type_of_symbol(&sig.parameters[rest_index]);
                        Some(self.get_array_element_type(&rest_param_type))
                    }
                },
            }
        } else {
            None
        };
        let explicit_types: Option<Vec<Arc<Type>>> = match &node.data {
            tsox_frontend::ast::NodeData::CallExpression(d) => d
                .type_arguments
                .as_ref()
                .map(|ta| ta.iter().map(|t| self.get_type_from_type_node(t)).collect()),
            tsox_frontend::ast::NodeData::NewExpression(d) => d
                .type_arguments
                .as_ref()
                .map(|ta| ta.iter().map(|t| self.get_type_from_type_node(t)).collect()),
            _ => None,
        };
        let explicit_types = explicit_types.filter(|ex| ex.len() == sig.type_parameters.len());
        let inferred_types = match &explicit_types {
            Some(ex) => ex.clone(),
            None => self.infer_call_type_arguments(node, sig, &arguments.nodes),
        };
        for (i, arg) in arguments.iter().enumerate() {
            let base_param_type = if has_rest && i >= rest_index {
                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {
                self.signature_instantiated_param_type(sig, i)
                    .or_else(|| self.try_get_type_at_position(sig, i))
                    .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[i]))
            } else {
                continue;
            };
            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &base_param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else {
                base_param_type
            };
            let inference_empty = !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }
            let arg_type = if self.is_context_sensitive(arg) {
                self.type_of_context_sensitive_arg(arg, &param_type)
            } else {
                self.get_type_of_node(arg)
            };
            if self.is_type_related_to(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
            ) {
                continue;
            }
            let param_optional = i < sig.parameters.len()
                && (sig.parameters[i]
                    .flags
                    .contains(tsox_frontend::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            tsox_frontend::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    }));
            let display_param = if param_optional {
                Some(self.strip_optional_undefined(&param_type))
            } else {
                None
            };
            let mut out: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
            self.check_type_related_to_and_elaborate_display(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
                Some(arg),
                Some(arg),
                Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
                Some(&mut out),
                display_param.as_ref(),
            );
            return out.into_iter().next();
        }
        let (spread, rest_type, err_node) =
            self.non_array_rest_spread_parts(node, sig, arguments, &inferred_types)?;
        let mut out: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        self.check_type_related_to_and_elaborate_display(
            &spread,
            &rest_type,
            crate::checker::relater::RelationKind::Assignable,
            Some(&err_node),
            Some(&err_node),
            Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
            Some(&mut out),
            None,
        );
        out.into_iter().next()
    }

    pub(crate) fn non_array_rest_spread_parts(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
        inferred_types: &[Arc<Type>],
    ) -> Option<(Arc<Type>, Arc<Type>, Arc<Node>)> {
        if !sig.has_rest_parameter() {
            return None;
        }
        let rest_index = sig.parameters.len().saturating_sub(1);
        let rest_type = self.get_type_of_symbol(&sig.parameters[rest_index]);
        if self.is_array_type(&rest_type)
            || self.is_tuple_type(&rest_type)
            || rest_type.flags.contains(TypeFlags::Any)
        {
            return None;
        }
        let rest_type = if !inferred_types.is_empty() {
            self.substitute_infer_type_parameters(
                &rest_type,
                &sig.type_parameters,
                inferred_types,
            )
        } else if !sig.type_parameters.is_empty() {
            return None;
        } else {
            rest_type
        };
        if self.is_array_type(&rest_type)
            || self.is_tuple_type(&rest_type)
            || rest_type.flags.contains(TypeFlags::Any | TypeFlags::Unknown)
            || rest_type.is_type_parameter()
        {
            return None;
        }
        let remaining: Vec<Arc<Node>> = arguments.iter().skip(rest_index).cloned().collect();
        let (spread, err_node) = match (remaining.first(), remaining.last()) {
            (Some(first), Some(last)) => {
                if matches!(&last.data, tsox_frontend::ast::NodeData::SpreadElement(_)) {
                    let expr = match &last.data {
                        tsox_frontend::ast::NodeData::SpreadElement(d) => {
                            Arc::clone(&d.expression)
                        }
                        _ => unreachable!(),
                    };
                    (self.get_type_of_node(&expr), Arc::clone(last))
                } else {
                    let types = remaining.iter().map(|a| self.get_type_of_node(a)).collect();
                    (self.create_tuple_type(types), Arc::clone(first))
                }
            }
            _ => (self.create_tuple_type(Vec::new()), Arc::clone(node)),
        };
        if self.is_type_related_to(
            &spread,
            &rest_type,
            crate::checker::relater::RelationKind::Assignable,
        ) {
            return None;
        }
        Some((spread, rest_type, err_node))
    }
    fn call_receiver_type(&mut self, callee: &Arc<Node>) -> Option<Arc<Type>> {
        let mut cur = Arc::clone(callee);
        while cur.kind == tsox_frontend::ast::SyntaxKind::ParenthesizedExpression {
            cur = match &cur.data {
                tsox_frontend::ast::NodeData::ParenthesizedExpression(d) => {
                    Arc::clone(&d.expression)
                }
                _ => return None,
            };
        }
        match &cur.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(d) => {
                Some(self.get_type_of_node(&d.expression))
            }
            tsox_frontend::ast::NodeData::ElementAccessExpression(d) => {
                Some(self.get_type_of_node(&d.expression))
            }
            _ => None,
        }
    }

    pub(crate) fn get_return_type_of_call_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        // Go getResolvedSignature：调用位签名/返回型解析同步强制（inferSignature
        // 会查询实参函数的返回型），此窗口内不开函数体推断环抑制界
        self.call_return_query_depth += 1;
        let t = self.get_return_type_of_call_expression_inner(node);
        self.call_return_query_depth -= 1;
        t
    }

    fn get_return_type_of_call_expression_inner(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let callee = match &node.data {
            tsox_frontend::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            _ => return self.get_any_type(),
        };
        let explicit_type_args = match &node.data {
            tsox_frontend::ast::NodeData::CallExpression(d) => d.type_arguments.clone(),
            _ => None,
        };
        // Go checkCallExpression：callee 为 super 时调用结果恒为 void
        if callee.0.kind == SyntaxKind::SuperKeyword {
            return self.get_void_type();
        }
        let callee_type = self.get_type_of_node(&callee.0);
        // 多态 this 返回位：lib 形如 sort(): this —— 返回型以接收者类型实例化
        let receiver_type: Option<Arc<Type>> = self.call_receiver_type(&callee.0);
        if let Some(structured) = callee_type.as_structured() {
            let signatures = structured.call_signatures();
            if signatures.is_empty() {
                return self.get_any_type();
            }

            // 显式类型实参优先选元数匹配的泛型重载
            let matching_idx = if let Some(ta) = &explicit_type_args
                && signatures.len() > 1
                && let Some(idx) = signatures
                    .iter()
                    .position(|s| s.type_parameters.len() == ta.len())
            {
                Some(idx)
            } else if signatures.len() == 1 {
                Some(0)
            } else {
                self.find_matching_signature_opt(node, signatures, &callee.1)
            };
            // 全重载不可适用：Go getCandidateForOverloadFailure 给联合签名
            //（参数位并集/返回交集）
            let combined: Option<Arc<Signature>> = match matching_idx {
                Some(_idx) => None,
                None => self.candidate_for_overload_failure(node, signatures, &callee.1),
            };
            let sig: &Arc<Signature> = match &combined {
                Some(c) => c,
                None => &signatures[matching_idx.unwrap_or(0)],
            };
            if let Some(mut rt) = self.get_return_type_of_signature(sig) {
                if matches!(
                    &rt.data,
                    crate::checker::types::TypeData::TypeParameter(tp) if tp.is_this_type
                ) && let Some(receiver) = &receiver_type
                {
                    rt = crate::checker::flow_narrow_calls::substitute_this_type(self, &rt, receiver);
                }
                if rt
                    .flags
                    .intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol)
                    && self.is_symbol_or_symbol_for_call(node)
                {
                    let target = node
                        .parent()
                        .map(|p| {
                            crate::checker::checker_es_symbol::walk_up_parenthesized_expressions(&p)
                        })
                        .unwrap_or_else(|| Arc::clone(node));
                    return self.get_es_symbol_like_type_for_node(&target);
                }
                if !sig.type_parameters.is_empty() {
                    let args: Vec<Arc<Node>> = callee.1.iter().cloned().collect();
                    let inferred = match &explicit_type_args {
                        Some(ta) if ta.len() == sig.type_parameters.len() => ta
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect(),
                        _ => self.infer_call_type_arguments(node, sig, &args),
                    };
                    self.in_return_substitution = true;
                    let r =
                        self.substitute_infer_type_parameters(&rt, &sig.type_parameters, &inferred);
                    self.in_return_substitution = false;
                    // 泛型函数体内声明的类经调用位实例化：携带 (外层函数, 实参) 显示元
                    // （变量侧渲染 f<string>.C 形态；仅限体内声明，不污染顶层类）
                    if !inferred.is_empty()
                        && r.alias.is_none()
                        && let Some(fn_decl) = sig.declaration.clone()
                        && let Some(fn_sym) = fn_decl.name().and_then(|n| self.resolve_identifier(&n))
                        && r.symbol.as_ref().is_some_and(|class_sym| {
                            class_sym.flags.contains(tsox_frontend::ast::SymbolFlags::Class)
                                && class_sym.declarations.iter().any(|d| {
                                    let mut cur = d.parent();
                                    while let Some(n) = cur {
                                        if Arc::ptr_eq(&n, &fn_decl) {
                                            return true;
                                        }
                                        cur = n.parent();
                                    }
                                    false
                                })
                        })
                    {
                        let ptr = Arc::as_ptr(&r) as *mut crate::checker::types::Type;
                        unsafe {
                            (*ptr).alias = Some(Box::new(crate::checker::types::TypeAlias::new(
                                Some(Arc::clone(&fn_sym)),
                                inferred.clone(),
                            )));
                        }
                    }
                    return r;
                }
                return rt;
            }

            return self.get_any_type();
        }
        self.get_any_type()
    }

    pub(crate) fn get_return_type_of_new_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.call_return_query_depth += 1;
        let t = self.get_return_type_of_new_expression_inner(node);
        self.call_return_query_depth -= 1;
        t
    }

    fn get_return_type_of_new_expression_inner(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (callee, args) = match &node.data {
            tsox_frontend::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(callee);
        if let Some(structured) = callee_type.as_structured() {
            let construct_sigs = structured.construct_signatures();
            if construct_sigs.is_empty() {
                return self.get_any_type();
            }
            // 多构造重载按实参选择（对齐 call 路径 find_matching_signature）
            let matching_idx = if construct_sigs.len() == 1 {
                0
            } else {
                self.find_matching_signature(node, construct_sigs, &args)
            };
            let sigs_vec: Vec<Arc<crate::checker::types::Signature>> =
                construct_sigs.iter().cloned().collect();
            let sig = &sigs_vec[matching_idx];
            {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    let rt = if !sig.type_parameters.is_empty() {
                        let arg_vec: Vec<Arc<Node>> = args.iter().cloned().collect();
                        let inferred = self.infer_call_type_arguments(node, sig, &arg_vec);
                        self.substitute_infer_type_parameters(&rt, &sig.type_parameters, &inferred)
                    } else {
                        rt
                    };

                    let explicit_type_args = match &node.data {
                        tsox_frontend::ast::NodeData::NewExpression(d) => d.type_arguments.clone(),
                        _ => None,
                    };
                    if let Some(type_args) = &explicit_type_args
                        && let Some(class_sym) = rt.symbol.clone()
                    {
                        let arg_types: Vec<Arc<Type>> = type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect();
                        // 实参对位按首个声明的类型参数表：增强声明的同名 T
                        // 是独立符号，并集计数会让 Set<T> 的显式实参挂不上
                        let expected =
                            self.first_declared_type_parameter_count(&class_sym);
                        if expected != 0 && expected == arg_types.len() {
                            return self.attach_explicit_type_arguments_cached(&rt, arg_types);
                        }
                    }
                    // new SS() / new SS：未绑定实参的类类型参数按 unknown 实例化
                    if explicit_type_args.is_none()
                        && let Some(class_sym) = rt.symbol.clone()
                    {
                        let has_attached_args = match &rt.data {
                            crate::checker::types::TypeData::Object(o) => !o.type_arguments.is_empty(),
                            crate::checker::types::TypeData::Interface(i) => {
                                !i.object.type_arguments.is_empty()
                            }
                            _ => false,
                        };
                        if !has_attached_args {
                            let tps = self.declared_type_parameter_types(&class_sym);
                            if !tps.is_empty() {
                                let unknowns: Vec<Arc<Type>> = tps
                                    .iter()
                                    .map(|_| self.get_unknown_type())
                                    .collect();
                                return self
                                    .attach_explicit_type_arguments_cached(&rt, unknowns);
                            }
                        }
                    }
                    return rt;
                }
                return self.get_any_type();
            }
        }
        self.get_any_type()
    }
}
