use std::sync::Arc;

use crate::ast::{
    Node, NodeList, SymbolFlags, SyntaxKind,
};
use crate::core::text::TextRange;
use crate::diagnostics::messages_generated::*;







use super::*;


impl Checker {
    fn report_get_accessor_call(&mut self, callee_expr: &Arc<Node>) -> bool {
        let crate::ast::NodeData::PropertyAccessExpression(pa) = &callee_expr.data else {
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
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,

            pa.name.loc,
            crate::diagnostics::messages_generated::
                THIS_EXPRESSION_IS_NOT_CALLABLE_BECAUSE_IT_IS_A_GET_ACCESSOR_DID_YOU_MEAN_TO_USE_IT_WITHOUT,
            vec![],
        ));
        true
    }
    fn check_call_arity(
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
            .position(|a| matches!(a.data, crate::ast::NodeData::SpreadElement(_)))
        {

            let min_count = self.get_min_argument_count(sig);
            let max_count = self.get_parameter_count(sig);
            let has_rest = self.has_effective_rest_parameter(sig);
            let spread_ok = spread_idx >= min_count && (has_rest || spread_idx < max_count);
            if !spread_ok {
                let file = self.current_file.clone();
                let spread_node = Arc::clone(&arguments.nodes[spread_idx]);
                self.diagnostics.add(crate::ast::Diagnostic::new(
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

        if !has_rest && arg_count > max_count {
            let file = self.current_file.clone();
            let loc = self.extra_arguments_range(arguments, max_count);
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                loc,
                EXPECTED_0_ARGUMENTS_BUT_GOT_1,
                vec![min_count.to_string(), arg_count.to_string()],
            ));
            return false;
        }

        if arg_count < min_count {
            let file = self.current_file.clone();

            let error_loc = if is_new {
                node.loc
            } else if let crate::ast::NodeData::PropertyAccessExpression(d) = &callee_expr.data
            {
                d.name.loc
            } else {
                callee_expr.loc
            };
            let message = if has_rest {
                EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1
            } else {
                EXPECTED_0_ARGUMENTS_BUT_GOT_1
            };
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                error_loc,
                message,
                vec![min_count.to_string(), arg_count.to_string()],
            ));
            return false;
        }

        true
    }

    fn extra_arguments_range(&self, arguments: &Arc<NodeList>, max_count: usize) -> TextRange {
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

    fn signature_accepts_arguments(
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
            let arg_type = self.get_type_of_node(arg);
            if !self.is_type_assignable_to(&arg_type, &param_type) {
                return false;
            }
        }
        true
    }

    fn find_matching_signature(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> usize {

        self.speculation_depth += 1;
        let result = (|| {
            for (idx, sig) in signatures.iter().enumerate() {
                if self.signature_accepts_arguments(node, sig, arguments) {
                    return idx;
                }
            }

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
                    return idx;
                }
            }
            0
        })();
        self.speculation_depth -= 1;
        result
    }
    pub(crate) fn check_call_arguments(&mut self, node: &Arc<Node>, is_new: bool) {
        let (callee_expr, arguments) = match &node.data {
            crate::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            crate::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return,
        };

        if !is_new && callee_expr.kind == SyntaxKind::SuperKeyword {
            let Some(base_ctor_type) = self.resolve_base_class_constructor_type() else {
                return;
            };
            self.check_call_arguments_against(
                node,
                &base_ctor_type,
                &arguments,
                callee_expr,
                 true,
            );
            return;
        }
        let callee_type = self.get_type_of_node(callee_expr);

        if !is_new {
            let optional_call = matches!(
                &node.data,
                crate::ast::NodeData::CallExpression(d) if d.question_dot_token.is_some()
            );
            if !optional_call {
                self.report_possibly_null_or_undefined(callee_expr, &callee_type, true);
            }
        }
        self.check_call_arguments_against(node, &callee_type, &arguments, callee_expr, is_new);
    }

    fn report_invocation_error(
        &mut self,
        callee_expr: &Arc<Node>,
        callee_type: &Arc<Type>,
        is_new: bool,
    ) {
        let head = if is_new {
            THIS_EXPRESSION_IS_NOT_CONSTRUCTABLE
        } else {
            THIS_EXPRESSION_IS_NOT_CALLABLE
        };
        let no_sigs = if is_new {
            TYPE_0_HAS_NO_CONSTRUCT_SIGNATURES
        } else {
            TYPE_0_HAS_NO_CALL_SIGNATURES
        };
        let chain = if callee_type.flags.contains(TypeFlags::Union)
            && let Some(u) = callee_type.as_union_or_intersection()
        {

            let union_str = self.type_to_string(callee_type);
            let mut has_signatures = false;
            let mut first_without: Option<String> = None;
            for c in u.types.iter() {
                let n = if is_new {
                    c.as_structured()
                        .map(|s| s.construct_signatures().len())
                        .unwrap_or(0)
                } else {
                    c.as_structured()
                        .map(|s| s.call_signatures().len())
                        .unwrap_or(0)
                };
                if n != 0 {
                    has_signatures = true;
                    if first_without.is_some() {
                        break;
                    }
                } else if first_without.is_none() {
                    first_without = Some(self.type_to_string(c));
                }
            }
            let msg = if !has_signatures {
                if is_new {
                    NO_CONSTITUENT_OF_TYPE_0_IS_CONSTRUCTABLE
                } else {
                    NO_CONSTITUENT_OF_TYPE_0_IS_CALLABLE
                }
            } else if first_without.is_some() {
                if is_new {
                    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CONSTRUCTABLE
                } else {
                    NOT_ALL_CONSTITUENTS_OF_TYPE_0_ARE_CALLABLE
                }
            } else if is_new {
                EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_CONSTRUCT_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
            } else {
                EACH_MEMBER_OF_THE_UNION_TYPE_0_HAS_SIGNATURES_BUT_NONE_OF_THOSE_SIGNATURES_ARE_COMPATIBLE_WITH_EACH_OTHER
            };
            let mut outer = crate::ast::Diagnostic::new(
                self.current_file.clone(),
                callee_expr.loc,
                msg,
                vec![union_str],
            );
            if let Some(first) = first_without.filter(|_| has_signatures) {
                outer.message_chain = vec![crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    callee_expr.loc,
                    no_sigs,
                    vec![first],
                )];
            }
            vec![outer]
        } else {

            let apparent_str = if callee_type.flags.contains(TypeFlags::Intersection)
                && self.is_never_intersection(callee_type)
            {
                "never".to_string()
            } else {
                match self.primitive_apparent_name(callee_type) {
                    Some(name) => name.to_string(),
                    None => self.type_to_string(callee_type),
                }
            };
            vec![crate::ast::Diagnostic::new(
                self.current_file.clone(),
                callee_expr.loc,
                no_sigs,
                vec![apparent_str],
            )]
        };
        let mut diag = crate::ast::Diagnostic::new(
            self.current_file.clone(),
            callee_expr.loc,
            head,
            vec![],
        );
        diag.message_chain = chain;
        self.diagnostics.add(diag);
    }

    fn primitive_apparent_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        let name = if t.flags.intersects(
            TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) {
            "String"
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            "Number"
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            "Boolean"
        } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
            "Symbol"
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            "BigInt"
        } else {
            return None;
        };
        self.globals.get(name).map(|_| name)
    }

    fn is_never_intersection(&mut self, t: &Arc<Type>) -> bool {
        let Some(ui) = t.as_union_or_intersection() else {
            return false;
        };
        let domain = |t: &Arc<Type>| -> u8 {
            if t.flags.intersects(
                TypeFlags::String
                    | TypeFlags::StringLiteral
                    | TypeFlags::TemplateLiteral
                    | TypeFlags::StringMapping,
            ) {
                1
            } else if t.flags.intersects(TypeFlags::Number | TypeFlags::NumberLiteral) {
                2
            } else if t
                .flags
                .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
            {
                3
            } else if t
                .flags
                .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
            {
                4
            } else if t.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
                5
            } else if t.flags.contains(TypeFlags::Undefined) {
                6
            } else if t.flags.contains(TypeFlags::Null) {
                7
            } else {
                0
            }
        };
        let disjoint = |a: &Arc<Type>, b: &Arc<Type>| -> bool {
            let (da, db) = (domain(a), domain(b));
            if da == 0 || db == 0 {
                return false;
            }
            if da != db {
                return true;
            }
            match (a.literal_value(), b.literal_value()) {
                (Some(x), Some(y)) => x != y,
                _ => false,
            }
        };
        for (i, c) in ui.types.iter().enumerate() {
            let Some(cs) = c.as_structured() else {
                continue;
            };
            for prop in &cs.properties {
                for (j, other) in ui.types.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let Some(os) = other.as_structured() else {
                        continue;
                    };
                    if let Some(other_prop) = os
                        .properties
                        .iter()
                        .find(|p| p.name == prop.name)
                        .cloned()
                    {
                        let pt = self.get_type_of_symbol(prop);
                        let ot = self.get_type_of_symbol(&other_prop);
                        if disjoint(&pt, &ot) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn check_call_arguments_against(
        &mut self,
        node: &Arc<Node>,
        callee_type: &Arc<Type>,
        arguments: &Arc<NodeList>,
        callee_expr: &Arc<Node>,
        is_new: bool,
    ) {

        if callee_type.flags.contains(TypeFlags::Any) {
            return;
        }

        let cond_constraint;
        let callee_type: &Arc<Type> = if callee_type.flags.contains(TypeFlags::Conditional) {
            match self.deferred_default_constraint_of_conditional(callee_type) {
                Some(constraint) => {
                    cond_constraint = constraint;
                    &cond_constraint
                }
                None => callee_type,
            }
        } else {
            callee_type
        };

        let mut union_signatures: Vec<Arc<Signature>> = Vec::new();
        let signatures: &[Arc<Signature>] =
            if callee_type.as_union_or_intersection().is_some() {

                let mut leaves: Vec<&Arc<Type>> = Vec::new();
                flatten_union_leaves(callee_type, &mut leaves);
                if is_new {

                    let all_constructable = !leaves.is_empty()
                        && leaves.iter().all(|m| {
                            m.as_structured()
                                .is_some_and(|s| !s.construct_signatures().is_empty())
                        });
                    if all_constructable {
                        for m in &leaves {
                            if let Some(s) = m.as_structured() {
                                union_signatures
                                    .extend(s.construct_signatures().iter().cloned());
                            }
                        }
                        &union_signatures
                    } else {

                        self.report_invocation_error(callee_expr, callee_type, is_new);
                        return;
                    }
                } else {

                    let mut expanded_leaves: Vec<Arc<Type>> = Vec::new();
                    for m in leaves.iter().copied() {
                        if m.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                            continue;
                        }
                        if m.flags.contains(TypeFlags::Conditional) {
                            if let Some(constraint) = self
                                .deferred_default_constraint_of_conditional(m)
                            {
                                if let Some(u) = constraint.as_union_or_intersection() {
                                    for c in u.types.iter() {
                                        if !c.flags.intersects(
                                            TypeFlags::Undefined | TypeFlags::Null,
                                        ) && !c.flags.contains(TypeFlags::Never)
                                        {
                                            expanded_leaves.push(Arc::clone(c));
                                        }
                                    }
                                } else if !constraint.flags.intersects(
                                    TypeFlags::Undefined | TypeFlags::Null,
                                ) && !constraint.flags.contains(TypeFlags::Never)
                                {
                                    expanded_leaves.push(constraint);
                                }
                                continue;
                            }
                        }
                        expanded_leaves.push(Arc::clone(m));
                    }
                    let all_callable = !expanded_leaves.is_empty()
                        && expanded_leaves.iter().all(|m| {
                            m.as_structured()
                                .is_some_and(|s| !s.call_signatures().is_empty())
                        });
                    if all_callable {
                        for m in &expanded_leaves {
                            if let Some(s) = m.as_structured() {
                                union_signatures
                                    .extend(s.call_signatures().iter().cloned());
                            }
                        }
                        &union_signatures
                    } else {

                        self.report_invocation_error(callee_expr, callee_type, is_new);
                        return;
                    }
                }
            } else if let Some(structured) = callee_type.as_structured() {
                if is_new {
                    structured.construct_signatures()
                } else {
                    structured.call_signatures()
                }
            } else {

                if !is_new && self.report_get_accessor_call(callee_expr) {
                    return;
                }
                self.report_invocation_error(callee_expr, callee_type, is_new);
                return;
            };

        let type_arg_filtered: Vec<Arc<Signature>>;
        let signatures: &[Arc<Signature>] = {
            let provided = Self::explicit_type_argument_count(node);
            if provided != 0 && signatures.len() > 1 {
                type_arg_filtered = signatures
                    .iter()
                    .filter(|s| s.type_parameters.len() == provided)
                    .cloned()
                    .collect();
                if !type_arg_filtered.is_empty() {
                    &type_arg_filtered
                } else {
                    signatures
                }
            } else {
                signatures
            }
        };
        if signatures.is_empty() {
            if !is_new {

                if callee_expr.kind == SyntaxKind::Identifier
                    && let Some(structured) = callee_type.as_structured()
                    && !structured.construct_signatures().is_empty()
                {
                    let type_str = self.type_to_string(callee_type);
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        callee_expr.loc,
                        crate::diagnostics::messages_generated::
                            VALUE_OF_TYPE_0_IS_NOT_CALLABLE_DID_YOU_MEAN_TO_INCLUDE_NEW,
                        vec![type_str],
                    ));
                    return;
                }
            }
            if is_new {

                if let Some(structured) = callee_type.as_structured() {
                    let call_sigs: &[Arc<Signature>] = structured.call_signatures();
                    if !call_sigs.is_empty() {
                        if !self.no_implicit_any {
                            let matching = self.find_matching_signature(node, call_sigs, &arguments);
                            let ret_is_void = self
                                .get_return_type_of_signature(&call_sigs[matching])
                                .is_some_and(|t| t.flags.contains(TypeFlags::Void));
                            if !ret_is_void {
                                let file = self.current_file.clone();
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    node.loc,
                                    crate::diagnostics::messages_generated::
                                        ONLY_A_VOID_FUNCTION_CAN_BE_CALLED_WITH_THE_NEW_KEYWORD,
                                    Vec::new(),
                                ));
                            }
                        }
                        self.check_call_arguments_against(
                            node,
                            callee_type,
                            &arguments,
                            callee_expr,
                             false,
                        );
                        return;
                    }
                }
            }

            if !is_new && self.report_get_accessor_call(callee_expr) {
                return;
            }
            self.report_invocation_error(callee_expr, callee_type, is_new);
            return;
        }

        let matching_idx = if signatures.len() == 1 {
            0
        } else {

            let no_match = {
                self.speculation_depth += 1;
                let r = !signatures
                    .iter()
                    .any(|s| self.signature_accepts_arguments(node, s, &arguments));
                self.speculation_depth -= 1;
                r
            };
            if no_match && self.report_no_overload_matches(node, signatures, &arguments) {
                return;
            }
            self.find_matching_signature(node, signatures, &arguments)
        };
        let sig = Arc::clone(&signatures[matching_idx]);

        if !self.check_call_arity(node, &sig, &arguments, callee_expr, is_new) {
            return;
        }
        let _file = self.current_file.clone();

        let has_rest = sig.has_rest_parameter();
        let rest_index = if has_rest {
            sig.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        let rest_element_type = if has_rest {

            let ret = match self.try_get_type_at_position(&sig, rest_index) {
                Some(t) => Some(t),
                None => {
                    let rest_param_type =
                        self.get_type_of_symbol(&sig.parameters[rest_index]);
                    Some(self.get_array_element_type(&rest_param_type))
                }
            };
            ret
        } else {
            None
        };

        if !sig.type_parameters.is_empty() || Self::has_explicit_type_arguments(node) {
            let provided = Self::explicit_type_argument_count(node);

            let expected = if is_new {
                self.get_return_type_of_signature(&sig)
                    .and_then(|rt| rt.symbol.clone())
                    .map(|class_sym| {
                        let tps = self.declared_type_parameter_types(&class_sym);
                        if tps.is_empty() {
                            sig.type_parameters.len()
                        } else {
                            tps.len()
                        }
                    })
                    .unwrap_or_else(|| sig.type_parameters.len())
            } else {
                sig.type_parameters.len()
            };
            if provided != 0
                && provided != expected

                && !callee_type.flags.contains(TypeFlags::Any)
            {
                let loc = match &node.data {
                    crate::ast::NodeData::CallExpression(d) => d
                        .type_arguments
                        .as_ref()
                        .and_then(|t| t.iter().next())
                        .map(|t| t.loc)
                        .unwrap_or(node.loc),
                    crate::ast::NodeData::NewExpression(d) => d
                        .type_arguments
                        .as_ref()
                        .and_then(|t| t.iter().next())
                        .map(|t| t.loc)
                        .unwrap_or(node.loc),
                    _ => node.loc,
                };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    loc,
                    crate::diagnostics::messages_generated::EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                    vec![expected.to_string(), provided.to_string()],
                ));
            }
        }
        let inferred_types = self.infer_call_type_arguments(node, &sig, &arguments.nodes);

        let new_explicit_subst: Option<(Vec<Arc<Type>>, Vec<Arc<Type>>)> = if is_new {
            self.get_return_type_of_signature(&sig)
                .and_then(|rt| rt.symbol.clone())
                .and_then(|class_sym| {
                    let tps = self.declared_type_parameter_types(&class_sym);
                    if tps.is_empty() {
                        return None;
                    }

                    let args: Option<Vec<Arc<Type>>> = match &node.data {
                        crate::ast::NodeData::NewExpression(d) => d
                            .type_arguments
                            .as_ref()
                            .map(|ta| ta.iter().map(|t| self.get_type_from_type_node(t)).collect()),
                        _ => None,
                    };
                    let args = match args {
                        Some(a) if a.len() == tps.len() => Some(a),

                        _ if callee_expr.kind == SyntaxKind::SuperKeyword => self
                            .heritage_type_arguments_for_base(&class_sym)
                            .filter(|a| a.len() == tps.len()),
                        _ => None,
                    };
                    args.map(|args| (tps, args))
                })
        } else {
            None
        };
        if std::env::var_os("TSOX_DEBUG_INFER").is_some() {
            eprintln!(
                "[infer] sig params={} tp={}",
                sig.parameters.len(),
                sig.type_parameters.len()
            );
            for (i, t) in inferred_types.iter().enumerate() {
                eprintln!("[infer]   {} -> {}", i, self.type_to_string(t));
            }
        }
        for (i, arg) in arguments.iter().enumerate() {

            let base_param_type = if has_rest && i >= rest_index {

                Arc::clone(rest_element_type.as_ref().unwrap())
            } else if i < sig.parameters.len() {

                self.try_get_type_at_position(&sig, i)
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
            } else if let Some((tps, args)) = new_explicit_subst.as_ref() {
                self.substitute_infer_type_parameters(&base_param_type, tps, args)
            } else {
                Arc::clone(&base_param_type)
            };

            let inference_empty =
                !sig.type_parameters.is_empty() && inferred_types.is_empty();
            if param_type.flags.contains(TypeFlags::Any)
                || (inference_empty && param_type.is_type_parameter())
            {
                continue;
            }

            if matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) {
                let pt = Arc::clone(&param_type);
                self.check_contextual_elements(arg, &pt, arg.loc);
            }
            let arg_type = self.get_type_of_node(arg);

            let display_param = if i < sig.parameters.len() {
                let param_optional = sig.parameters[i]
                    .flags
                    .contains(crate::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    });
                if param_optional {
                    Some(self.strip_optional_undefined(&param_type))
                } else {
                    None
                }
            } else {
                None
            };

            let elements_reported = matches!(
                arg.kind,
                SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression
            ) && self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc.pos() >= arg.loc.pos() && d.loc.end() <= arg.loc.end());
            if elements_reported {
                continue;
            }
            let ok = self.check_type_related_to_and_elaborate_display(
                &arg_type,
                &param_type,
                crate::checker::relater::RelationKind::Assignable,
                Some(arg),
                None,
                Some(&ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1),
                None,
                display_param.as_ref(),
            );

            if !ok {
                break;
            }
        }
    }

    fn report_no_overload_matches(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        arguments: &Arc<NodeList>,
    ) -> bool {
        let saved = self.diagnostics.take_inner();
        let mut entries: Vec<crate::ast::Diagnostic> = Vec::new();
        let mut all_failed = true;
        for sig in signatures.iter() {
            match self.probe_first_argument_error(node, sig, arguments) {
                Some(d) => entries.push(d),
                None => {
                    all_failed = false;
                    break;
                }
            }
        }
        let _probe_only = self.diagnostics.take_inner();
        self.diagnostics.set_inner(saved);
        if !all_failed {
            return false;
        }
        let file = self.current_file.clone();
        let anchor = entries
            .first()
            .map(|d| d.loc)
            .unwrap_or(node.loc);
        let mut chain: Vec<crate::ast::Diagnostic> = Vec::new();
        for (i, (entry, sig)) in entries.into_iter().zip(signatures.iter()).enumerate() {
            let sig_str = self.signature_display_colon(sig, "");
            let mut d = crate::ast::Diagnostic::new(
                file.clone(),
                anchor,
                crate::diagnostics::messages_generated::
                    OVERLOAD_0_OF_1_2_GAVE_THE_FOLLOWING_ERROR,
                vec![(i + 1).to_string(), signatures.len().to_string(), sig_str],
            );
            d.message_chain = vec![entry];
            chain.push(d);
        }
        let mut head = crate::ast::Diagnostic::new(
            file,
            anchor,
            crate::diagnostics::messages_generated::NO_OVERLOAD_MATCHES_THIS_CALL,
            Vec::new(),
        );
        head.message_chain = chain;
        self.diagnostics.add(head);
        true
    }

    fn probe_first_argument_error(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
        arguments: &Arc<NodeList>,
    ) -> Option<crate::ast::Diagnostic> {

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
                        let rest_param_type =
                            self.get_type_of_symbol(&sig.parameters[rest_index]);
                        Some(self.get_array_element_type(&rest_param_type))
                    }
                },
            }
        } else {
            None
        };
        let inferred_types = self.infer_call_type_arguments(node, sig, &arguments.nodes);
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
            let arg_type = self.get_type_of_node(arg);
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
                    .contains(crate::ast::SymbolFlags::Optional)
                    || sig.parameters[i].declarations.iter().any(|d| {
                        matches!(
                            &d.data,
                            crate::ast::NodeData::ParameterDeclaration(pd)
                                if pd.question_token.is_some() || pd.initializer.is_some()
                        )
                    }));
            let display_param = if param_optional {
                Some(self.strip_optional_undefined(&param_type))
            } else {
                None
            };
            let mut out: Vec<crate::ast::Diagnostic> = Vec::new();
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
        None
    }
    pub(crate) fn get_return_type_of_call_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let callee = match &node.data {
            crate::ast::NodeData::CallExpression(data) => {
                (&data.expression, data.arguments.clone())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(&callee.0);
        if let Some(structured) = callee_type.as_structured() {
            let signatures = structured.call_signatures();
            if signatures.is_empty() {
                return self.get_any_type();
            }

            let matching_idx = if signatures.len() == 1 {
                0
            } else {
                self.find_matching_signature(node, signatures, &callee.1)
            };
            let sig = &signatures[matching_idx];
            if let Some(rt) = self.get_return_type_of_signature(sig) {

                if !sig.type_parameters.is_empty() {
                    let args: Vec<Arc<Node>> = callee.1.iter().cloned().collect();
                    let inferred = self.infer_call_type_arguments(node, sig, &args);
                    self.in_return_substitution = true;
                    let r = self.substitute_infer_type_parameters(
                        &rt,
                        &sig.type_parameters,
                        &inferred,
                    );
                    self.in_return_substitution = false;
                    return r;
                }
                return rt;
            }

            return self.get_any_type();
        }
        self.get_any_type()
    }

    pub(crate) fn get_return_type_of_new_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (callee, args) = match &node.data {
            crate::ast::NodeData::NewExpression(data) => {
                (&data.expression, data.arguments.clone().unwrap_or_default())
            }
            _ => return self.get_any_type(),
        };
        let callee_type = self.get_type_of_node(callee);
        if let Some(structured) = callee_type.as_structured() {
            for sig in structured.construct_signatures() {
                if let Some(rt) = self.get_return_type_of_signature(sig) {

                    let rt = if !sig.type_parameters.is_empty() {
                        let arg_vec: Vec<Arc<Node>> = args.iter().cloned().collect();
                        let inferred = self.infer_call_type_arguments(node, sig, &arg_vec);
                        self.substitute_infer_type_parameters(
                            &rt,
                            &sig.type_parameters,
                            &inferred,
                        )
                    } else {
                        rt
                    };

                    if let crate::ast::NodeData::NewExpression(d) = &node.data
                        && let Some(type_args) = &d.type_arguments
                        && let Some(class_sym) = rt.symbol.clone()
                    {
                        let tps = self.declared_type_parameter_types(&class_sym);
                        let arg_types: Vec<Arc<Type>> = type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect();
                        if !tps.is_empty() && tps.len() == arg_types.len() {
                            return self.attach_explicit_type_arguments_cached(&rt, arg_types);
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
