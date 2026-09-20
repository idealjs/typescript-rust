#![allow(unused_imports)]

use crate::checker::checker_calls::*;

impl Checker {
    pub(crate) fn report_get_accessor_call(&mut self, callee_expr: &Arc<Node>) -> bool {
        let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &callee_expr.data else {
            return false;
        };
        if pa.name.kind != SyntaxKind::Identifier {
            return false;
        }
        let target_type = self.get_type_of_node(&pa.expression);
        let name = pa.name.text().to_string();
        let is_getter = target_type
            .as_structured()
            .and_then(|s| s.properties.iter().find(|p| p.name == name))
            .is_some_and(|sym| sym.flags.contains(SymbolFlags::GetAccessor));
        if !is_getter {
            return false;
        }
        let file = self.current_file.clone();
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,

            pa.name.loc,
            tsox_core::diagnostics::messages_generated::
                THIS_EXPRESSION_IS_NOT_CALLABLE_BECAUSE_IT_IS_A_GET_ACCESSOR_DID_YOU_MEAN_TO_USE_IT_WITHOUT,
            vec![],
        ));
        true
    }
    pub(crate) fn check_call_arity(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) -> bool {
        let arg_count = arguments.len();

        if let Some(spread_idx) = arguments
            .nodes
            .iter()
            .position(|a| matches!(a.data, tsox_frontend::ast::NodeData::SpreadElement(_)))
        {
            let min_count = self.get_min_argument_count(sig);
            let max_count = self.get_parameter_count(sig);
            let has_rest = self.has_effective_rest_parameter(sig);
            let spread_ok = spread_idx >= min_count && (has_rest || spread_idx < max_count);
            if !spread_ok {
                let file = self.current_file.clone();
                let spread_node = Arc::clone(&arguments.nodes[spread_idx]);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    spread_node.loc,
                    A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER,
                    vec![],
                ));
                return false;
            }

            return true;
        }

        let min_count = self.get_min_argument_count(sig);
        let max_count = self.get_parameter_count(sig);
        let has_rest = self.has_effective_rest_parameter(sig);
        // Go getArgumentArityError：min<max 且无 rest 时参数计数展示为区间
        let parameter_range = if has_rest || min_count >= max_count {
            min_count.to_string()
        } else {
            format!("{min_count}-{max_count}")
        };

        if !has_rest && arg_count > max_count {
            let file = self.current_file.clone();
            let loc = self.extra_arguments_range(arguments, max_count);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                loc,
                EXPECTED_0_ARGUMENTS_BUT_GOT_1,
                vec![parameter_range, arg_count.to_string()],
            ));
            return false;
        }

        if arg_count < min_count {
            let file = self.current_file.clone();

            let error_loc = if is_new {
                node.loc
            } else if let tsox_frontend::ast::NodeData::PropertyAccessExpression(d) =
                &callee_expr.data
            {
                d.name.loc
            } else {
                callee_expr.loc
            };
            let (message, count_arg) = if has_rest {
                (EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1, min_count.to_string())
            } else {
                (EXPECTED_0_ARGUMENTS_BUT_GOT_1, parameter_range)
            };
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                error_loc,
                message,
                vec![count_arg, arg_count.to_string()],
            ));
            return false;
        }

        true
    }

    pub(crate) fn extra_arguments_range(
        &self,
        arguments: &Arc<NodeList>,
        max_count: usize,
    ) -> TextRange {
        if max_count >= arguments.nodes.len() {
            return arguments.loc;
        }
        let start = arguments.nodes[max_count].loc.pos;
        let mut end = arguments
            .nodes
            .last()
            .map(|a| a.loc.end)
            .unwrap_or(arguments.loc.end);
        if end < start {
            end = start;
        }
        TextRange { pos: start, end }
    }

    pub(crate) fn signature_accepts_arguments(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> bool {
        if arguments.len() < sig.min_argument_count.max(0) as usize {
            return false;
        }

        let inferred_types = if sig.type_parameters.is_empty() {
            Vec::new()
        } else {
            self.infer_call_type_arguments(node, sig, &arguments.nodes)
        };

        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for (i, arg) in arguments.iter().enumerate() {
            let param_type = if has_rest && i >= rest_index {
                match self.try_get_type_at_position(sig, i) {
                    Some(t) => t,
                    None => {
                        let rt = self.get_type_of_symbol(&sig.parameters[rest_index]);
                        match self.get_array_element_type_of(&rt) {
                            Some(e) => e,
                            None => rt,
                        }
                    }
                }
            } else if i < sig.parameters.len() {
                match self.try_get_type_at_position(sig, i) {
                    Some(t) => t,

                    None => continue,
                }
            } else {
                return false;
            };
            let param_type = if !inferred_types.is_empty() {
                self.substitute_infer_type_parameters(
                    &param_type,
                    &sig.type_parameters,
                    &inferred_types,
                )
            } else {
                param_type
            };

            if param_type.flags.contains(TypeFlags::Any) {
                continue;
            }
            // Go chooseOverload 候选检查按上下文定型实参：上下文敏感函数
            // 表达式以参数型为上下文重定型（返回位参与判定，`return "hello"`
            // 否决 (value:T)=>IPromise<U> 过载），裸 get_type_of_node 会以
            // any 放行全部过载
            let arg_type = if self.is_context_sensitive(arg) {
                self.type_of_context_sensitive_arg(arg, &param_type)
            } else {
                self.get_type_of_node(arg)
            };
            let verdict = self.is_type_assignable_to(&arg_type, &param_type);
            if !verdict {
                return false;
            }
        }
        true
    }

    pub(crate) fn find_matching_signature(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> usize {
        self.find_matching_signature_opt(node, signatures, arguments)
            .unwrap_or(0)
    }

    /// Some=命中重载；None=全部不可适用（Go resolveCall 失败态，供联合
    /// 签名与最长候选兜底）
    pub(crate) fn find_matching_signature_opt(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> Option<usize> {
        self.speculation_depth += 1;
        let result = (|| {
            for (idx, sig) in signatures.iter().enumerate() {
                // Go chooseOverload 逐候选独立推测：relater_overflow 是深层
                // 递归护栏的粘滞放行标志，外层关系一旦触发会让后续全部
                // 候选被放行（首候选胜出）。逐候选隔离，检查自身溢出仍
                // 保守放行（护栏语义不变），只不外泄、不内渗
                let saved_overflow = self.relater_overflow;
                self.relater_overflow = false;
                let accepts = self.signature_accepts_arguments(node, sig, arguments);
                self.relater_overflow |= saved_overflow;
                if accepts {
                    return Some(idx);
                }
            }

            // Go pickLongestCandidateSignature 语义：元数兜底仅在单签名或含
            // 泛型重载时启用；无泛型的多重载全败走联合签名（None）
            let has_generic = signatures.iter().any(|s| !s.type_parameters.is_empty());
            if signatures.len() == 1 || has_generic {
                let arg_count = arguments.len();
                for (idx, sig) in signatures.iter().enumerate() {
                    let max_params = if sig.has_rest_parameter() {
                        usize::MAX
                    } else {
                        sig.parameters.len()
                    };
                    if arg_count <= max_params
                        && arg_count >= sig.min_argument_count.max(0) as usize
                    {
                        return Some(idx);
                    }
                }
            }
            None
        })();
        self.speculation_depth -= 1;
        result
    }
    pub(crate) fn check_call_arguments(&mut self, node: &Arc<Node>, is_new: bool) {
        let (callee_expr, arguments) = match &node.data {
            tsox_frontend::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            tsox_frontend::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return,
        };

        if !is_new && callee_expr.kind == SyntaxKind::SuperKeyword {
            let Some(base_ctor_type) = self.resolve_base_class_constructor_type() else {
                return;
            };
            self.check_call_arguments_against(node, &base_ctor_type, &arguments, callee_expr, true);
            return;
        }
        let callee_type = self.get_type_of_node(callee_expr);

        let mut callee_type = callee_type;
        if !is_new {
            let optional_call = matches!(
                &node.data,
                tsox_frontend::ast::NodeData::CallExpression(d) if d.question_dot_token.is_some()
            );
            if !optional_call
                && self.report_possibly_null_or_undefined(callee_expr, &callee_type, true)
            {
                let non_nullable = self.remove_nullable_from_union(&callee_type);
                if non_nullable
                    .flags
                    .intersects(TypeFlags::Null | TypeFlags::Undefined | TypeFlags::Never)
                {
                    return;
                }
                callee_type = non_nullable;
            }
        }
        self.check_call_arguments_against(node, &callee_type, &arguments, callee_expr, is_new);
    }
}
