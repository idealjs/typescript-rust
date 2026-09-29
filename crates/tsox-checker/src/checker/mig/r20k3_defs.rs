#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::checker_checker_checker::Checker;
use crate::checker::mig::m1f_2::{IterationTypes, IterationTypesResolver};
use crate::checker::types::*;
use crate::checker::utilities_is_optional_symbol::is_type_any;
use std::sync::Arc;
use tsox_frontend::ast::{Diagnostic, Node, Symbol, SymbolFlags, SymbolTable, SyntaxKind};
use tsox_core::jsnum::{Number, PseudoBigInt};

use crate::checker::mig::m1d_2::get_base_type_node_of_class;
use crate::checker::mig::w9a::new_type_mapper;

impl Checker {
    pub fn empty_string_type(&mut self) -> Arc<Type> {
        self.get_string_literal_type("")
    }

    pub fn zero_type(&mut self) -> Arc<Type> {
        self.get_number_literal_type(Number::from(0))
    }

    pub fn zero_big_int_type(&mut self) -> Arc<Type> {
        self.get_big_int_literal_type(PseudoBigInt::new("0", false))
    }

    pub fn empty_fresh_jsx_object_type(&mut self) -> Arc<Type> {
        self.new_object_type(ObjectFlags::Anonymous, None)
    }

    pub fn get_global_template_strings_array_type(&mut self) -> Arc<Type> {
        self.get_global_type("TemplateStringsArray", 0, true)
    }

    pub fn get_builtin_iterator_return_type(&mut self) -> Arc<Type> {
        if self.strict_builtin_iterator_return {
            self.undefined_type()
        } else {
            self.any_type()
        }
    }

    pub fn is_nullable_type(&mut self, t: &Arc<Type>) -> bool {
        self.has_type_facts(t, TypeFacts::IS_UNDEFINED_OR_NULL)
    }

    pub fn is_possibly_discriminant_value(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateExpression
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::Identifier
            | SyntaxKind::UndefinedKeyword => true,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ParenthesizedExpression => {
                match node.expression() {
                    Some(expr) => self.is_possibly_discriminant_value(expr),
                    None => false,
                }
            }
            SyntaxKind::JsxExpression => match node.expression() {
                Some(expr) => self.is_possibly_discriminant_value(expr),
                None => true,
            },
            _ => false,
        }
    }

    pub fn new_call_signature(
        &mut self,
        type_parameters: Option<Vec<Arc<Type>>>,
        this_parameter: Option<&Arc<Symbol>>,
        parameters: Vec<Arc<Symbol>>,
        return_type: Option<Arc<Type>>,
    ) -> Arc<Signature> {
        let return_type = return_type.unwrap_or_else(|| self.unknown_type());
        self.new_signature(
            SignatureFlags::None,
            None,
            type_parameters.as_deref().unwrap_or(&[]),
            this_parameter,
            &parameters,
            &return_type,
            None,
            parameters.len(),
        )
    }

    pub fn empty_type_literal_type(&mut self) -> Arc<Type> {
        let symbol = self.new_symbol(SymbolFlags::TypeLiteral, "__type");
        self.new_anonymous_type(&symbol, SymbolTable::default(), vec![], vec![], vec![])
    }

    pub fn get_single_base_for_non_augmenting_subtype(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let target = t.target().cloned()?;
        if !t.object_flags.intersects(ObjectFlags::Reference)
            || !target.object_flags.intersects(ObjectFlags::CLASS_OR_INTERFACE)
        {
            return None;
        }
        if target.object_flags.intersects(ObjectFlags::Class) {
            if let Some(base_type_node) = get_base_type_node_of_class(&target) {
                let simple = base_type_node
                    .expression()
                    .map(|e| tsox_frontend::ast::is_identifier(e) || tsox_frontend::ast::is_property_access_expression(e))
                    .unwrap_or(false);
                if !simple {
                    return None;
                }
            }
        }
        let bases = self.get_base_types(&target);
        if bases.len() != 1 {
            return None;
        }
        let symbol = t.symbol()?;
        if !self.get_members_of_symbol(symbol).is_empty() {
            return None;
        }
        let type_parameters: Vec<Arc<Type>> = match &target.data {
            TypeData::Interface(i) => i.all_type_parameters.clone(),
            TypeData::Tuple(td) => td.interface_data.all_type_parameters.clone(),
            _ => Vec::new(),
        };
        let type_arguments = self.get_type_arguments(t);
        let mut instantiated_base = if type_parameters.is_empty() {
            Arc::clone(&bases[0])
        } else {
            let mapper = new_type_mapper(
                type_parameters.clone(),
                type_arguments[..type_parameters.len().min(type_arguments.len())].to_vec(),
            );
            self.instantiate_type(&bases[0], Some(&Arc::new(mapper)))
        };
        let _ = type_arguments;
        Some(instantiated_base)
    }

    pub fn get_iteration_types_of_method(
        &mut self,
        t: &Arc<Type>,
        r: &IterationTypesResolver,
        method_name: &str,
        error_node: Option<&Arc<Node>>,
        diagnostic_output: &mut Vec<Diagnostic>,
    ) -> IterationTypes {
        let method = self.get_property_of_type(t, method_name);
        if method.is_none() && method_name != "next" {
            return IterationTypes::default();
        }
        let method_type = match &method {
            Some(method) if !(method_name == "next" && method.flags.intersects(SymbolFlags::Optional)) => {
                let method_type = self.get_type_of_symbol(method);
                if method_name == "next" {
                    method_type
                } else {
                    self.get_type_with_facts(&method_type, TypeFacts::NE_UNDEFINED_OR_NULL)
                }
            }
            _ => self.unknown_type(),
        };
        if is_type_any(&method_type) {
            return IterationTypes {
                yield_type: Some(self.any_type()),
                return_type: Some(self.any_type()),
                next_type: Some(self.any_type()),
            };
        }
        let method_signatures = self.get_signatures_of_type(&method_type, SignatureKind::Call);
        if method_signatures.is_empty() {
            return IterationTypes::default();
        }
        let mut method_parameter_types: Vec<Arc<Type>> = vec![];
        let mut method_return_types: Vec<Arc<Type>> = vec![];
        for signature in &method_signatures {
            if method_name != "throw" && !signature.parameters.is_empty() {
                method_parameter_types.push(self.get_type_at_position(signature, 0));
            }
            method_return_types.push(
                self.get_return_type_of_signature(signature)
                    .unwrap_or_else(|| self.unknown_type()),
            );
        }
        let mut return_types: Vec<Arc<Type>> = vec![];
        let mut next_type: Option<Arc<Type>> = None;
        if method_name != "throw" {
            let method_parameter_type = if !method_parameter_types.is_empty() {
                self.get_union_type(method_parameter_types)
            } else {
                self.unknown_type()
            };
            if method_name == "next" {
                next_type = Some(method_parameter_type);
            } else if method_name == "return" {
                let resolved = (r.resolve_iteration_type)(
                    &method_parameter_type,
                    error_node.map(|n| n.as_ref()),
                )
                .unwrap_or_else(|| self.any_type());
                return_types.push(resolved);
            }
        }
        let method_return_type = if !method_return_types.is_empty() {
            self.get_intersection_type(method_return_types)
        } else {
            self.never_type()
        };
        let resolved_method_return_type = (r.resolve_iteration_type)(
            &method_return_type,
            error_node.map(|n| n.as_ref()),
        )
        .unwrap_or_else(|| self.any_type());
        let iteration_types =
            self.get_iteration_types_of_iterator_result_r20k3(&resolved_method_return_type);
        let has_types = iteration_types.yield_type.is_some()
            || iteration_types.return_type.is_some()
            || iteration_types.next_type.is_some();
        let yield_type = if !has_types {
            return_types.push(self.any_type());
            Some(self.any_type())
        } else {
            iteration_types.yield_type.clone().map(|t| {
                return_types.push(
                    iteration_types
                        .return_type
                        .clone()
                        .unwrap_or_else(|| self.any_type()),
                );
                t
            })
        };
        let yield_type = match yield_type {
            Some(yield_type) => Some(yield_type),
            None => {
                return_types.push(self.any_type());
                Some(self.any_type())
            }
        };
        IterationTypes {
            yield_type,
            return_type: Some(self.get_union_type(return_types)),
            next_type,
        }
    }

    pub fn get_iteration_types_of_iterator_result_r20k3(
        &mut self,
        t: &Arc<Type>,
    ) -> IterationTypes {
        if is_type_any(t) {
            return IterationTypes {
                yield_type: Some(self.any_type()),
                return_type: Some(self.any_type()),
                next_type: Some(self.any_type()),
            };
        }
        IterationTypes::default()
    }

    pub fn combine_iteration_types_r20k3(&mut self, parts: Vec<IterationTypes>) -> IterationTypes {
        let union_of = |parts: &[IterationTypes],
                        pick: &dyn Fn(&IterationTypes) -> Option<&Arc<Type>>|
         -> Option<Vec<Arc<Type>>> {
            let types: Vec<Arc<Type>> = parts.iter().filter_map(|p| pick(p).cloned()).collect();
            if types.is_empty() {
                None
            } else {
                Some(types)
            }
        };
        let yields = union_of(&parts, &|p| p.yield_type.as_ref());
        let returns = union_of(&parts, &|p| p.return_type.as_ref());
        let nexts = union_of(&parts, &|p| p.next_type.as_ref());
        IterationTypes {
            yield_type: yields.map(|ts| self.get_union_type(ts)),
            return_type: returns.map(|ts| self.get_union_type(ts)),
            next_type: nexts.map(|ts| self.get_union_type(ts)),
        }
    }
}
