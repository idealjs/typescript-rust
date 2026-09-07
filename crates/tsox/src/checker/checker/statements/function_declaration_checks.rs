#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_function_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);

        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                self.check_cjs_reserved_top_level_name(node, name);
            }
        }

        self.check_duplicate_function_implementations(node);

        self.check_overload_implementation_follows(node);
        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(tps) = &data.type_parameters {
                let _ = tps;
            }
            self.check_grammar_parameter_list(&data.parameters);

            self.check_parameter_property_modifiers(&data.parameters, false);

            self.check_parameter_implicit_any(node, &data.parameters, 0);
            for p in data.parameters.iter() {
                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                    && let Some(pt) = &pd.type_node
                {
                    self.check_type_annotation(pt);
                }
            }
            if let Some(tn) = &data.type_node {
                self.check_type_annotation(tn);
            }

            if self.no_implicit_any
                && data.type_node.is_none()
                && data.body.is_none()
                && let Some(name) = &data.name
                && name.kind == SyntaxKind::Identifier
            {
                let file = self.current_file.clone();
                let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        name.loc,
                        crate::diagnostics::messages_generated::
                            X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                        vec![name.text().to_string(), "any".to_string()],
                    );
                self.diagnostics.add(diagnostic);
            }
        }

        self.check_unmatched_jsdoc_parameters(node);

        let fn_type = self.get_type_of_function_like(node);

        let fn_symbol = match &node.data {
            crate::ast::NodeData::FunctionDeclaration(data) => {
                data.name.as_ref().and_then(|n| self.resolve_identifier(n))
            }
            _ => None,
        };
        let fn_type = match &fn_symbol {
            Some(sym) => self.attach_function_expando_type(sym, fn_type),
            None => fn_type,
        };
        self.type_node_links.get_or_default(node).resolved_type = Some(fn_type.clone());
        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                if let Some(symbol) = self.resolve_identifier(name) {
                    let symbol_type = match self.build_overload_function_type(&symbol) {
                        Some(overload_type) => overload_type,
                        None => fn_type.clone(),
                    };
                    self.value_symbol_links
                        .get_or_default(&symbol)
                        .resolved_type = Some(symbol_type.clone());
                    self.type_node_links.get_or_default(name).resolved_type = Some(symbol_type);
                }
            }
        }

        self.push_function_scope(node);
        self.break_continue_context_stack
            .push(BreakContinueContext {
                kind: BreakContinueContextKind::Function,
                label: None,
                is_iteration: false,
            });

        let declared_return = match &node.data {
            crate::ast::NodeData::FunctionDeclaration(data) => {
                let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
                data.type_node
                    .as_ref()
                    .map(|tn| self.get_type_from_type_node(tn))
                    .map(|t| self.unwrap_async_return_type(t, is_async))
            }
            _ => None,
        };
        self.return_type_stack.push(declared_return.clone());
        self.in_ctor_body_stack.push(false);

        self.this_container_stack
            .push(ThisContainerKind::PlainFunction);
        if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
            if let Some(body) = &data.body {
                self.check_statement(body);
            }
        }
        self.this_container_stack.pop();

        if let Some(ret_type) = &declared_return {
            if !ret_type.flags.contains(TypeFlags::Void)
                && !ret_type.flags.contains(TypeFlags::Undefined)
                && !ret_type.flags.contains(TypeFlags::Any)
            {
                if let crate::ast::NodeData::FunctionDeclaration(data) = &node.data {
                    if let Some(body) = &data.body {
                        if !self.function_body_definitely_returns(body) {
                            if !Self::function_body_has_explicit_return(body) {
                                let loc = data.type_node.as_ref().map_or(node.loc, |tn| tn.loc);
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        loc,
                                        A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                                        vec![],
                                    ));
                            } else {
                                let loc = data.type_node.as_ref().map_or(node.loc, |tn| tn.loc);
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        loc,
                                        FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                                        vec![],
                                    ));
                            }
                        }
                    }
                }
            }
        }
        self.return_type_stack.pop();
        self.in_ctor_body_stack.pop();
        self.break_continue_context_stack.pop();
        self.pop_function_scope();
    }
}
