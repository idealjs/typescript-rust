#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_new_expression(&mut self, node: &Arc<Node>) {
        if let crate::ast::NodeData::NewExpression(data) = &node.data {
            self.check_expression(&data.expression);
            if let Some(args) = &data.arguments {
                for (i, arg) in args.iter().enumerate() {
                    self.check_call_arg_with_context(&data.expression, i, arg);
                }
            }

            let mut reported_abstract = false;
            if data.expression.kind == SyntaxKind::Identifier {
                if let Some(symbol) = self.resolve_identifier(&data.expression) {
                    if self.symbol_is_abstract_class(&symbol) {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                            vec![],
                        ));
                        reported_abstract = true;
                    }
                }
            }

            if !reported_abstract {
                let callee_type = self.get_type_of_node(&data.expression);
                if self.type_includes_abstract_constructor(&callee_type) {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        node.loc,
                        CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                        vec![],
                    ));
                }
            }
        }
        self.check_call_arguments(node, true);
    }

    pub fn check_function_like_expression(&mut self, node: &Arc<Node>) {
        let mut contextual_param_count = self
            .call_arg_arrow_context
            .last_mut()
            .map(|v| std::mem::replace(v, 0))
            .unwrap_or(0);
        if contextual_param_count == 0 {
            contextual_param_count = self
                .contextual_signature_of_arrow(node)
                .map_or(0, |sig| sig.parameters.len());
        }
        match &node.data {
            crate::ast::NodeData::ArrowFunction(d) => {
                self.check_parameter_property_modifiers(&d.parameters, false);
                self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);

                for param in d.parameters.iter() {
                    self.check_parameter_default_initializer(param);
                }
            }
            crate::ast::NodeData::FunctionExpression(d) => {
                self.check_parameter_property_modifiers(&d.parameters, false);
                self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);
                for param in d.parameters.iter() {
                    self.check_parameter_default_initializer(param);
                }
            }
            _ => {}
        }

        if matches!(node.data, crate::ast::NodeData::FunctionExpression(_)) {
            self.this_container_stack
                .push(ThisContainerKind::PlainFunction);
        }
        self.check_function_like_body(node);
        if matches!(node.data, crate::ast::NodeData::FunctionExpression(_)) {
            self.this_container_stack.pop();
        }
    }
}
