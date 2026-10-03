#![allow(unused_imports)]

use crate::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3e::FunctionFlags;

use crate::checker::types::*;
use tsox_frontend::ast::Node;

impl Checker {
    pub fn check_type_argument_constraints(
        &mut self,
        node: &Arc<Node>,
        type_parameters: &[Arc<Type>],
    ) -> bool { ::tsox_core::fntrace::enter("check_type_argument_constraints"); 
        let mut type_arguments: Option<Vec<Arc<Type>>> = None;
        let mut mapper: Option<Arc<TypeMapper>> = None;
        let mut result = true;
        let node_type_arguments = tsox_frontend::ast::mig::m3c::type_arguments(node);
        for (i, type_parameter) in type_parameters.iter().enumerate() {
            let constraint = self.get_constraint_of_type_parameter(type_parameter);
            if let Some(constraint) = constraint {
                if type_arguments.is_none() {
                    type_arguments =
                        Some(self.get_effective_type_arguments(node, type_parameters));
                    mapper = Some(Arc::new(crate::checker::mig::w9a::new_type_mapper(
                        type_parameters.to_vec(),
                        type_arguments.as_ref().unwrap().clone(),
                    )));
                }
                let error_node = node_type_arguments.get(i).cloned();
                let instantiated = self.instantiate_type(&constraint, mapper.as_ref());
                result = result
                    && self.check_type_assignable_to(
                        &type_arguments.as_ref().unwrap()[i],
                        &instantiated,
                        error_node.as_ref(),
                        Some(&tsox_core::diagnostics::messages_generated::TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1),
                    );
            }
        }
        result
    }

    pub fn check_generator_instantiation_assignability_to_return_type_no_error(
        &mut self,
        return_type: &Arc<Type>,
        function_flags: FunctionFlags,
    ) -> bool { ::tsox_core::fntrace::enter("check_generator_instantiation_assignability_to_return_type_no_error"); 
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        let generator_yield_type = self
            .get_iteration_type_of_generator_function_return_type(
                IterationTypeKind::YIELD,
                return_type,
                is_async,
            )
            .unwrap_or_else(|| self.any_type());
        let generator_return_type = self
            .get_iteration_type_of_generator_function_return_type(
                IterationTypeKind::RETURN,
                return_type,
                is_async,
            )
            .unwrap_or_else(|| Arc::clone(&generator_yield_type));
        let generator_next_type = self
            .get_iteration_type_of_generator_function_return_type(
                IterationTypeKind::NEXT,
                return_type,
                is_async,
            )
            .unwrap_or_else(|| self.unknown_type());
        let generator_instantiation = self.create_generator_type(
            &generator_yield_type,
            &generator_return_type,
            &generator_next_type,
            is_async,
        );
        self.check_type_assignable_to(&generator_instantiation, return_type, None, None)
    }
}
