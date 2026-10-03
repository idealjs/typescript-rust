#![allow(unused_imports)]

#![allow(unused_imports)]
use crate::checker::checker::*;
use crate::checker::utilities_has_only_expression_initialization::is_optional_declaration;

#[path = "r24k16_defs.rs"]
pub mod r24k16_defs;

use tsox_frontend::ast::mig::m3c::type_arguments;

use super::m3a_2::has_dot_dot_dot_token;
use super::m2c::r18k3_defs::get_string_literal_value;
use super::wc2_2::get_number_literal_value;
use super::wc3_3::is_zero_big_int;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    pub(crate) fn get_type_arguments_for_alias_symbol(
        &mut self,
        symbol: Option<&Arc<Symbol>>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_arguments_for_alias_symbol"); 
        if let Some(symbol) = symbol {
            return self.get_local_type_parameters_of_class_or_interface_or_type_alias(symbol);
        }
        Vec::new()
    }

    pub(crate) fn get_type_arguments_from_node(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_arguments_from_node"); 
        type_arguments(node)
            .iter()
            .map(|n| self.get_type_from_type_node(n))
            .collect()
    }

    pub(crate) fn get_type_arguments_from_nodes(
        &mut self,
        type_argument_nodes: &[Arc<Node>],
        type_parameters: &[Arc<Type>],
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_arguments_from_nodes"); 
        let nodes: &[Arc<Node>] = if type_argument_nodes.len() > type_parameters.len() {
            &type_argument_nodes[..type_parameters.len()]
        } else {
            type_argument_nodes
        };
        let mut type_arguments: Vec<Arc<Type>> = nodes
            .iter()
            .map(|n| self.get_type_from_type_node(n))
            .collect();
        while type_arguments.len() < type_parameters.len() {
            let idx = type_arguments.len();
            let t = self
                .get_default_from_type_parameter(&type_parameters[idx])
                .or_else(|| self.get_constraint_of_type_parameter(&type_parameters[idx]))
                .unwrap_or_else(|| self.unknown_type());
            type_arguments.push(t);
        }
        type_arguments
    }

    pub(crate) fn get_type_facts_worker(
        &mut self,
        t: &Arc<Type>,
        caller_only_needs: TypeFacts,
    ) -> TypeFacts { ::tsox_core::fntrace::enter("get_type_facts_worker"); 
        let mut t = Arc::clone(t);
        if t.flags.intersects(TypeFlags::Intersection | TYPE_FLAGS_INSTANTIABLE) {
            t = self
                .get_base_constraint_of_type(&t)
                .unwrap_or_else(|| self.unknown_type());
        }
        let flags = t.flags;
        if flags.intersects(TypeFlags::String | TypeFlags::StringMapping) {
            if self.strict_null_checks {
                return TypeFacts::STRING_STRICT_FACTS;
            }
            return TypeFacts::STRING_FACTS;
        }
        if flags.intersects(TypeFlags::StringLiteral | TypeFlags::TemplateLiteral) {
            let is_empty =
                flags.contains(TypeFlags::StringLiteral) && get_string_literal_value(&t) == "";
            if self.strict_null_checks {
                if is_empty {
                    return TypeFacts::EMPTY_STRING_STRICT_FACTS;
                }
                return TypeFacts::NON_EMPTY_STRING_STRICT_FACTS;
            }
            if is_empty {
                return TypeFacts::EMPTY_STRING_FACTS;
            }
            return TypeFacts::NON_EMPTY_STRING_FACTS;
        }
        if flags.intersects(TypeFlags::Number | TypeFlags::Enum) {
            if self.strict_null_checks {
                return TypeFacts::NUMBER_STRICT_FACTS;
            }
            return TypeFacts::NUMBER_FACTS;
        }
        if flags.contains(TypeFlags::NumberLiteral) {
            let is_zero = get_number_literal_value(&t) == 0.0;
            if self.strict_null_checks {
                if is_zero {
                    return TypeFacts::ZERO_NUMBER_STRICT_FACTS;
                }
                return TypeFacts::NON_ZERO_NUMBER_STRICT_FACTS;
            }
            if is_zero {
                return TypeFacts::ZERO_NUMBER_FACTS;
            }
            return TypeFacts::NON_ZERO_NUMBER_FACTS;
        }
        if flags.contains(TypeFlags::BigInt) {
            if self.strict_null_checks {
                return TypeFacts::BIGINT_STRICT_FACTS;
            }
            return TypeFacts::BIGINT_FACTS;
        }
        if flags.contains(TypeFlags::BigIntLiteral) {
            let is_zero = is_zero_big_int(&t);
            if self.strict_null_checks {
                if is_zero {
                    return TypeFacts::ZERO_BIGINT_STRICT_FACTS;
                }
                return TypeFacts::NON_ZERO_BIGINT_STRICT_FACTS;
            }
            if is_zero {
                return TypeFacts::ZERO_BIGINT_FACTS;
            }
            return TypeFacts::NON_ZERO_BIGINT_FACTS;
        }
        if flags.contains(TypeFlags::Boolean) {
            if self.strict_null_checks {
                return TypeFacts::BOOLEAN_STRICT_FACTS;
            }
            return TypeFacts::BOOLEAN_FACTS;
        }
        if flags.contains(TYPE_FLAGS_BOOLEAN_LIKE) {
            let is_false = Arc::ptr_eq(&t, &self.false_type())
                || Arc::ptr_eq(&t, &self.regular_false_type());
            if self.strict_null_checks {
                if is_false {
                    return TypeFacts::FALSE_STRICT_FACTS;
                }
                return TypeFacts::TRUE_STRICT_FACTS;
            }
            if is_false {
                return TypeFacts::FALSE_FACTS;
            }
            return TypeFacts::TRUE_FACTS;
        }
        if flags.contains(TypeFlags::Object) {
            let possible_facts = if self.strict_null_checks {
                TypeFacts::EMPTY_OBJECT_STRICT_FACTS
                    | TypeFacts::FUNCTION_STRICT_FACTS
                    | TypeFacts::OBJECT_STRICT_FACTS
            } else {
                TypeFacts::EMPTY_OBJECT_FACTS
                    | TypeFacts::FUNCTION_FACTS
                    | TypeFacts::OBJECT_FACTS
            };
            if caller_only_needs.intersection(possible_facts).is_empty() {
                return TypeFacts::empty();
            }
            if t.object_flags.contains(ObjectFlags::Anonymous)
                && self.is_empty_object_type(&t)
            {
                if self.strict_null_checks {
                    return TypeFacts::EMPTY_OBJECT_STRICT_FACTS;
                }
                return TypeFacts::EMPTY_OBJECT_FACTS;
            }
            if self.is_function_object_type_fwd(&t) {
                if self.strict_null_checks {
                    return TypeFacts::FUNCTION_STRICT_FACTS;
                }
                return TypeFacts::FUNCTION_FACTS;
            }
            if self.strict_null_checks {
                return TypeFacts::OBJECT_STRICT_FACTS;
            }
            return TypeFacts::OBJECT_FACTS;
        }
        if flags.contains(TypeFlags::Void) {
            return TypeFacts::VOID_FACTS;
        }
        if flags.contains(TypeFlags::Undefined) {
            return TypeFacts::UNDEFINED_FACTS;
        }
        if flags.contains(TypeFlags::Null) {
            return TypeFacts::NULL_FACTS;
        }
        if flags.contains(TYPE_FLAGS_ES_SYMBOL_LIKE) {
            if self.strict_null_checks {
                return TypeFacts::SYMBOL_STRICT_FACTS;
            }
            return TypeFacts::SYMBOL_FACTS;
        }
        if flags.contains(TypeFlags::NonPrimitive) {
            if self.strict_null_checks {
                return TypeFacts::OBJECT_STRICT_FACTS;
            }
            return TypeFacts::OBJECT_FACTS;
        }
        if flags.contains(TypeFlags::Never) {
            return TypeFacts::empty();
        }
        if flags.contains(TypeFlags::Union) {
            let mut facts = TypeFacts::empty();
            if let Some(constituents) = t.types() {
                for constituent in constituents {
                    facts |= self.get_type_facts_worker(constituent, caller_only_needs);
                }
            }
            return facts;
        }
        if flags.contains(TypeFlags::Intersection) {
            return self.get_intersection_type_facts(&t, caller_only_needs);
        }
        TypeFacts::UNKNOWN_FACTS
    }

    pub(crate) fn get_type_for_binding_element(
        &mut self,
        declaration: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_for_binding_element"); 
        let check_mode = if has_dot_dot_dot_token(declaration) {
            CheckMode::RestBindingElement
        } else {
            CheckMode::Normal
        };
        let grand_parent = declaration.parent().and_then(|p| p.parent());
        let parent_type = grand_parent
            .as_ref()
            .and_then(|gp| self.get_type_for_binding_element_parent(gp, check_mode));
        if let Some(parent_type) = parent_type {
            return Some(self.get_binding_element_type_from_parent_type(
                declaration,
                &parent_type,
                false,
            ));
        }
        None
    }

    pub(crate) fn get_type_for_binding_element_parent(
        &mut self,
        node: &Arc<Node>,
        check_mode: CheckMode,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_for_binding_element_parent"); 
        if check_mode == CheckMode::Normal {
            if let Some(symbol) = self.get_symbol_of_declaration(node) {
                if let Some(resolved_type) = self
                    .value_symbol_links
                    .get(&symbol)
                    .and_then(|l| l.resolved_type.clone())
                {
                    if !(self.strict_null_checks && is_optional_declaration(node)) {
                        return Some(resolved_type);
                    }
                }
            }
        }
        self.get_type_for_variable_like_declaration(node, false, check_mode)
    }
}
