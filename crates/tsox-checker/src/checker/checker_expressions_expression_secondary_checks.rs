#![allow(unused_imports)]

use crate::checker::checker_expressions::*;

impl Checker {
    pub fn check_new_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_new_expression"); 
        if let tsox_frontend::ast::NodeData::NewExpression(data) = &node.data {
            self.check_expression(&data.expression);
            if let Some(args) = &data.arguments {
                for (i, arg) in args.iter().enumerate() {
                    self.check_call_arg_with_context(&data.expression, i, arg);
                }
            }
        }
        if !self.check_new_expression_ctor_accessibility(node) {
            self.check_call_arguments(node, true);
        }
    }

    pub fn check_function_like_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_function_like_expression"); 
        let has_grammar_error = self.check_grammar_function_like_declaration(node);
        if !has_grammar_error
            && matches!(
                node.data,
                tsox_frontend::ast::NodeData::FunctionExpression(_)
            )
        {
            self.check_grammar_for_generator(node);
        }
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
            tsox_frontend::ast::NodeData::ArrowFunction(d) => {
                self.check_parameter_property_modifiers(&d.parameters, false);
                self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);

                for param in d.parameters.iter() {
                    self.check_parameter_default_initializer(param);
                }
            }
            tsox_frontend::ast::NodeData::FunctionExpression(d) => {
                self.check_parameter_property_modifiers(&d.parameters, false);
                self.check_parameter_implicit_any(node, &d.parameters, contextual_param_count);
                for param in d.parameters.iter() {
                    self.check_parameter_default_initializer(param);
                }
            }
            _ => {}
        }

        if matches!(
            node.data,
            tsox_frontend::ast::NodeData::FunctionExpression(_)
        ) {
            self.this_container_stack
                .push(ThisContainerKind::PlainFunction);
        }
        self.check_function_like_body(node);
        if matches!(
            node.data,
            tsox_frontend::ast::NodeData::FunctionExpression(_)
        ) {
            self.this_container_stack.pop();
        }
    }
}
