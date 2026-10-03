use std::sync::Arc;

use crate::checker::checker_checker::*;
use crate::checker::mig::m2c_3::signature_has_rest_parameter;
use crate::checker::utilities_is_optional_symbol::is_type_any;
use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

impl Checker {
    pub fn is_node_within_class(&mut self, node: &Arc<Node>, class_declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_node_within_class"); 
        let mut containing_class = tsox_frontend::ast::get_containing_class(node);
        while let Some(class) = containing_class {
            if Arc::ptr_eq(&class, class_declaration) {
                return true;
            }
            containing_class = tsox_frontend::ast::get_containing_class(&class);
        }
        false
    }

    pub fn get_global_awaited_symbol_or_nil(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_awaited_symbol_or_nil"); 
        (self.get_global_type_alias_resolver("Awaited", 1, false))(self)
    }

    pub fn get_global_es_symbol_constructor_type_symbol_or_nil(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_es_symbol_constructor_type_symbol_or_nil"); 
        (self.get_global_type_symbol_resolver("SymbolConstructor", false))(self)
    }

    pub fn is_mixin_constructor_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_mixin_constructor_type"); 
        let signatures = self.get_signatures_of_type(t, SignatureKind::Construct);
        if signatures.len() == 1 {
            let s = &signatures[0];
            if s.type_parameters.is_empty()
                && s.parameters.len() == 1
                && signature_has_rest_parameter(s)
            {
                let param_type = self.get_type_of_parameter(&s.parameters[0]);
                return is_type_any(&param_type)
                    || self
                        .get_element_type_of_array_type(&param_type)
                        .is_some_and(|e| Arc::ptr_eq(&e, &self.any_type()));
            }
        }
        false
    }

    pub fn get_element_types(&mut self, t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_element_types"); 
        if let crate::checker::TypeData::Tuple(tuple) = &t.data {
            return tuple
                .element_infos
                .iter()
                .map(|e| e.type_.clone())
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default();
        }
        let type_arguments = self.get_type_arguments(t);
        let arity = self.get_type_reference_arity(t) as usize;
        if type_arguments.len() == arity {
            return type_arguments;
        }
        type_arguments.into_iter().take(arity).collect()
    }

    pub fn is_array_or_tuple_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_array_or_tuple_type"); 
        crate::checker::mig::wc1b::is_array_or_tuple_type(t)
    }

    pub fn get_actual_type_variable(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_actual_type_variable"); 
        get_actual_type_variable_r18k4(t)
    }
}

pub fn get_actual_type_variable_r18k4(t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_actual_type_variable_r18k4"); 
    if t.flags.contains(TypeFlags::Substitution)
        && let TypeData::Substitution(sub) = &t.data
        && let Some(base) = &sub.base_type
    {
        return get_actual_type_variable_r18k4(base);
    }
    Arc::clone(t)
}

pub fn any_yield_expression(body: &Node, visitor: &mut dyn FnMut(&Arc<Node>) -> bool) -> bool { ::tsox_core::fntrace::enter("any_yield_expression"); 
    tsox_frontend::ast::node_data_generated::for_each_child(body, |child: &Arc<Node>| {
        if child.kind == SyntaxKind::YieldExpression {
            visitor(child)
        } else if !tsox_frontend::ast::is_function_like(child)
            && any_yield_expression(child, visitor)
        {
            true
        } else {
            false
        }
    })
}
