use std::sync::Arc;

use tsox_frontend::ast::mig::m3e::FunctionFlags;
use crate::checker::checker::*;
use crate::checker::types::IterationTypeKind;

impl Checker {
    #[allow(non_snake_case)]
    pub fn unwrapReturnType(&mut self, return_type: &Arc<Type>, function_flags: FunctionFlags) -> Arc<Type> { ::tsox_core::fntrace::enter("unwrapReturnType"); 
        let is_generator = function_flags.contains(FunctionFlags::GENERATOR);
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        if is_generator {
            let return_iteration_type =
                self.get_iteration_type_of_generator_function_return_type(IterationTypeKind::RETURN, return_type, is_async);
            match return_iteration_type {
                None => self.error_type(),
                Some(return_iteration_type) => {
                    if is_async {
                        let unwrapped = self.unwrap_awaited_type(&return_iteration_type);
                        self.get_awaited_type_no_alias(&unwrapped)
                            .unwrap_or_else(|| self.error_type())
                    } else {
                        return_iteration_type
                    }
                }
            }
        } else if is_async {
            self.get_awaited_type_no_alias(return_type)
                .unwrap_or_else(|| self.error_type())
        } else {
            Arc::clone(return_type)
        }
    }
}
