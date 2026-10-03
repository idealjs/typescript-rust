use std::sync::Arc;

#[path = "r24k7_defs.rs"]
pub(crate) mod r24k7_defs;

use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::diagnostics::messages_generated::TYPE_0_HAS_NO_SIGNATURES_FOR_WHICH_THE_TYPE_ARGUMENT_LIST_IS_APPLICABLE;
use tsox_frontend::ast::mig::m3c::type_argument_list;
use tsox_frontend::ast::mig::m3f::get_node_id;
use tsox_frontend::ast::{Node, NodeFlags, SymbolFlags, SyntaxKind};
use tsox_frontend::scanner::skip_trivia;

use crate::checker::checker::Checker;
use crate::checker::types::{
    CacheHashKey, InstantiationExpressionKey, IntersectionFlags, IterationTypeKind, IterationTypes, IterationUse, KeyBuilder,
    ObjectFlags, Signature, SignatureKind, Type, TypeAlias, TypeData, TypeFacts, TypeFlags, UnionReduction,
};
use crate::checker::types::{IterationTypesResolver};

use super::m1e::r17k3_flags::*;
use super::m1e::r18k5_helpers::*;
use super::m2a::{is_not_null_type, is_not_undefined_type};
use super::m1e::r26k3_defs::{every_type_with_checker, instantiation_expression_types_get, instantiation_expression_types_insert, some_type_with_checker};
use super::m1f::r25k6_defs::map_type_with_checker;
use super::m2c_3::some_type;
use super::wc1b::{every_type, is_type_any};
use super::wc3_2::is_es2015_or_later_iterable;
use super::wc3_3::is_intersection_type;

impl Checker {
    pub fn get_instantiated_constructors_for_type_arguments(
        &mut self,
        t: &Arc<Type>,
        type_argument_nodes: &[Arc<Node>],
        location: Option<&Arc<Node>>,
    ) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("get_instantiated_constructors_for_type_arguments"); 
        let signatures = match location {
            Some(location) => self.get_constructors_for_type_arguments(t, type_argument_nodes, location),
            None => {
                let type_arg_count = type_argument_nodes.len();
                self.get_signatures_of_type(t, SignatureKind::Construct)
                    .into_iter()
                    .filter(|sig| {
                        let min = self.get_min_type_argument_count(&sig.type_parameters);
                        type_arg_count >= min && type_arg_count <= sig.type_parameters.len()
                    })
                    .collect()
            }
        };
        let type_arguments: Vec<Arc<Type>> = type_argument_nodes
            .iter()
            .map(|n| self.get_type_from_type_node(n))
            .collect();
        signatures
            .into_iter()
            .map(|sig| {
                if !sig.type_parameters.is_empty() {
                    self.get_signature_instantiation(&sig, &type_arguments)
                } else {
                    sig
                }
            })
            .collect()
    }

    pub fn get_instantiation_expression_type(&mut self, expr_type: &Arc<Type>, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_instantiation_expression_type"); 
        let type_arguments = type_argument_list(node);
        if Arc::ptr_eq(expr_type, &self.silent_never_type())
            || self.is_error_type(expr_type)
            || type_arguments.is_none()
        {
            return Arc::clone(expr_type);
        }
        let type_arguments = type_arguments.unwrap();
        let key = InstantiationExpressionKey {
            node_id: get_node_id(node),
            type_id: expr_type.id,
        };
        if let Some(cached) = instantiation_expression_types_get(&key) {
            return cached;
        }
        let has_some_applicable_signature_cell = std::cell::RefCell::new(false);
        let non_applicable_type_cell: std::cell::RefCell<Option<Arc<Type>>> = std::cell::RefCell::new(None);
        let mut get_instantiated_signatures = |c: &mut Checker, signatures: &[Arc<Signature>]| -> Vec<Arc<Signature>> {
            let mut instantiated: Vec<Arc<Signature>> = Vec::new();
            for sig in signatures {
                if sig.type_parameters.is_empty() || !c.has_correct_type_argument_arity(sig, type_arguments.nodes.as_slice()) {
                    continue;
                }
                let type_argument_types = c.check_type_arguments(sig, type_arguments.nodes.as_slice(), true, None);
                if let Some(type_argument_types) = type_argument_types {
                    instantiated.push(c.get_signature_instantiation(sig, &type_argument_types));
                } else {
                    instantiated.push(Arc::clone(sig));
                }
            }
            instantiated
        };
        fn get_instantiated_type(
            c: &mut Checker,
            t: &Arc<Type>,
            node: &Arc<Node>,
            get_instantiated_signatures: &mut dyn FnMut(&mut Checker, &[Arc<Signature>]) -> Vec<Arc<Signature>>,
            has_some_applicable_signature_cell: &std::cell::RefCell<bool>,
            non_applicable_type_cell: &std::cell::RefCell<Option<Arc<Type>>>,
        ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_instantiated_type"); 
            let mut has_signatures = false;
            let mut has_applicable_signature = false;
            fn get_instantiated_type_part(
                c: &mut Checker,
                t: &Arc<Type>,
                node: &Arc<Node>,
                get_instantiated_signatures: &mut dyn FnMut(&mut Checker, &[Arc<Signature>]) -> Vec<Arc<Signature>>,
                has_signatures: &mut bool,
                has_applicable_signature: &mut bool,
            ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_instantiated_type_part"); 
                if t.flags.intersects(TypeFlags::OBJECT) {
                    let resolved = c.resolve_structured_type_members(t);
                    let Some(structured) = resolved.as_structured_type() else {
                        return Arc::clone(t);
                    };
                    let call_signatures = get_instantiated_signatures(c, structured.call_signatures());
                    let construct_signatures = get_instantiated_signatures(c, structured.construct_signatures());
                    *has_signatures = *has_signatures
                        || !structured.call_signatures().is_empty()
                        || !structured.construct_signatures().is_empty();
                    *has_applicable_signature =
                        *has_applicable_signature || !call_signatures.is_empty() || !construct_signatures.is_empty();
                    if !same_signatures(&call_signatures, structured.call_signatures())
                        || !same_signatures(&construct_signatures, structured.construct_signatures())
                    {
                        let mut symbol = c.new_symbol(SymbolFlags::empty(), internal_symbol_name_instantiation_expression);
                        if let Some(symbol_mut) = Arc::get_mut(&mut symbol)
                            && let Some(t_symbol) = &t.symbol
                        {
                            symbol_mut.declarations = t_symbol.declarations.clone();
                        }
                        let mut result = c.new_object_type(ObjectFlags::ANONYMOUS | ObjectFlags::INSTANTIATION_EXPRESSION_TYPE, Some(symbol));
                        c.set_structured_type_members(
                            &result,
                            Some(structured.members.clone()),
                            call_signatures,
                            construct_signatures,
                            structured.index_infos.clone(),
                        );
                        if let Some(result_mut) = Arc::get_mut(&mut result)
                            && let TypeData::InstantiationExpression(ie) = &mut result_mut.data
                        {
                            ie.node = Some(Arc::clone(node));
                        }
                        return result;
                    }
                } else if t.flags.intersects(TypeFlags::INSTANTIABLE_NON_PRIMITIVE) {
                    if let Some(constraint) = c.get_base_constraint_of_type(t) {
                        let instantiated = get_instantiated_type_part(
                            c,
                            &constraint,
                            node,
                            get_instantiated_signatures,
                            has_signatures,
                            has_applicable_signature,
                        );
                        if !Arc::ptr_eq(&instantiated, &constraint) {
                            return instantiated;
                        }
                    }
                } else if t.flags.intersects(TypeFlags::UNION) {
                    return map_type_with_checker(c, t, &mut |c: &mut Checker, ty: &Arc<Type>| {
                        Some(get_instantiated_type_part(
                            c,
                            ty,
                            node,
                            get_instantiated_signatures,
                            has_signatures,
                            has_applicable_signature,
                        ))
                    })
                    .unwrap_or_else(|| Arc::clone(t));
                } else if t.flags.intersects(TypeFlags::INTERSECTION) {
                    if let TypeData::Intersection(i) = &t.data {
                        let parts: Vec<Arc<Type>> = i
                            .union_or_intersection
                            .types
                            .iter()
                            .map(|ty| {
                                get_instantiated_type_part(c, ty, node, get_instantiated_signatures, has_signatures, has_applicable_signature)
                            })
                            .collect();
                        return c.get_intersection_type(parts);
                    }
                }
                Arc::clone(t)
            }
            let result = get_instantiated_type_part(c, t, node, get_instantiated_signatures, &mut has_signatures, &mut has_applicable_signature);
            *has_some_applicable_signature_cell.borrow_mut() |= has_applicable_signature;
            if has_signatures && !has_applicable_signature && non_applicable_type_cell.borrow().is_none() {
                *non_applicable_type_cell.borrow_mut() = Some(Arc::clone(t));
            }
            result
        }
        let result = get_instantiated_type(
            self,
            expr_type,
            node,
            &mut get_instantiated_signatures,
            &has_some_applicable_signature_cell,
            &non_applicable_type_cell,
        );
        instantiation_expression_types_insert(key, Arc::clone(&result));
        let error_type = if *has_some_applicable_signature_cell.borrow() {
            non_applicable_type_cell.borrow().clone()
        } else {
            Some(Arc::clone(expr_type))
        };
        if let Some(error_type) = error_type {
            if let Some(source_file) = instantiation_expression_source_file(self, node) {
                let loc = new_text_range(skip_trivia(&source_file.text, type_arguments.pos()), type_arguments.end());
                let error_type_text = self.type_to_string(&error_type);
                self.add_diagnostic(new_diagnostic(
                    &source_file,
                    loc,
                    TYPE_0_HAS_NO_SIGNATURES_FOR_WHICH_THE_TYPE_ARGUMENT_LIST_IS_APPLICABLE,
                    &[error_type_text],
                ));
            }
        }
        result
    }

    pub fn get_intersection_type_ex(&mut self, types: &[Arc<Type>], flags: IntersectionFlags, alias: Option<&TypeAlias>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_intersection_type_ex"); 
        let mut ordered_types: OrderedSet<Arc<Type>> = OrderedSet::with_capacity(types.len());
        let includes = self.add_types_to_intersection(&mut ordered_types, TypeFlags::empty(), types);
        let mut type_set: Vec<Arc<Type>> = ordered_types.iter().cloned().collect();
        let mut object_flags = ObjectFlags::empty();
        if includes.intersects(TypeFlags::NEVER) {
            if type_set.iter().any(|t| Arc::ptr_eq(t, &self.silent_never_type())) {
                return Arc::clone(&self.silent_never_type());
            }
            return Arc::clone(&self.never_type());
        }
        if self.strict_null_checks
            && includes.intersects(TypeFlags::NULLABLE)
            && includes.intersects(TypeFlags::OBJECT | TypeFlags::NON_PRIMITIVE | TypeFlags::INCLUDES_EMPTY_OBJECT)
            || includes.intersects(TypeFlags::NON_PRIMITIVE)
                && includes.intersects(TypeFlags::DISJOINT_DOMAINS - TypeFlags::NON_PRIMITIVE)
            || includes.intersects(TypeFlags::STRING_LIKE)
                && includes.intersects(TypeFlags::DISJOINT_DOMAINS - TypeFlags::STRING_LIKE)
            || includes.intersects(TypeFlags::NUMBER_LIKE)
                && includes.intersects(TypeFlags::DISJOINT_DOMAINS - TypeFlags::NUMBER_LIKE)
            || includes.intersects(TypeFlags::BIGINT_LIKE)
                && includes.intersects(TypeFlags::DISJOINT_DOMAINS - TypeFlags::BIGINT_LIKE)
            || includes.intersects(TypeFlags::ES_SYMBOL_LIKE)
                && includes.intersects(TypeFlags::DISJOINT_DOMAINS - TypeFlags::ES_SYMBOL_LIKE)
            || includes.intersects(TypeFlags::VOID_LIKE)
                && includes.intersects(TypeFlags::DISJOINT_DOMAINS - TypeFlags::VOID_LIKE)
        {
            return Arc::clone(&self.never_type());
        }
        if includes.intersects(TypeFlags::TEMPLATE_LITERAL | TypeFlags::STRING_MAPPING)
            && includes.intersects(TypeFlags::STRING_LITERAL)
        {
            let (new_set, is_empty_set) = self.extract_redundant_template_literals(type_set);
            type_set = new_set;
            if is_empty_set {
                return Arc::clone(&self.never_type());
            }
        }
        if includes.intersects(TypeFlags::ANY) {
            if includes.intersects(TypeFlags::INCLUDES_WILDCARD) {
                return Arc::clone(&self.wildcard_type);
            }
            if includes.intersects(TypeFlags::INCLUDES_ERROR) {
                return Arc::clone(&self.error_type());
            }
            return Arc::clone(&self.any_type());
        }
        if !self.strict_null_checks && includes.intersects(TypeFlags::NULLABLE) {
            if includes.intersects(TypeFlags::INCLUDES_EMPTY_OBJECT) {
                return Arc::clone(&self.never_type());
            }
            if includes.intersects(TypeFlags::UNDEFINED) {
                return Arc::clone(&self.undefined_type());
            }
            return Arc::clone(&self.null_type());
        }
        if includes.intersects(TypeFlags::STRING)
            && includes.intersects(TypeFlags::STRING_LITERAL | TypeFlags::TEMPLATE_LITERAL | TypeFlags::STRING_MAPPING)
            || includes.intersects(TypeFlags::NUMBER) && includes.intersects(TypeFlags::NUMBER_LITERAL)
            || includes.intersects(TypeFlags::BIGINT) && includes.intersects(TypeFlags::BIGINT_LITERAL)
            || includes.intersects(TypeFlags::ES_SYMBOL) && includes.intersects(TypeFlags::UNIQUE_ES_SYMBOL)
            || includes.intersects(TypeFlags::VOID) && includes.intersects(TypeFlags::UNDEFINED)
            || includes.intersects(TypeFlags::INCLUDES_EMPTY_OBJECT) && includes.intersects(TypeFlags::DEFINITELY_NON_NULLABLE)
        {
            if !flags.intersects(IntersectionFlags::NO_SUPERTYPE_REDUCTION) {
                type_set = self.remove_redundant_supertypes(&type_set, includes);
            }
        }
        if includes.intersects(TypeFlags::INCLUDES_MISSING_TYPE) {
            if let Some(pos) = type_set.iter().position(|t| Arc::ptr_eq(t, &self.undefined_type())) {
                type_set[pos] = Arc::clone(&self.missing_type());
            }
        }
        if type_set.is_empty() {
            return Arc::clone(&self.unknown_type());
        }
        if type_set.len() == 1 {
            return Arc::clone(&type_set[0]);
        }
        if type_set.len() == 2 && !flags.intersects(IntersectionFlags::NO_CONSTRAINT_REDUCTION) {
            let mut type_var_index = 0;
            if !type_set[0].flags.intersects(TypeFlags::TYPE_VARIABLE) {
                type_var_index = 1;
            }
            let type_variable = Arc::clone(&type_set[type_var_index]);
            let primitive_type = Arc::clone(&type_set[1 - type_var_index]);
            if type_variable.flags.intersects(TypeFlags::TYPE_VARIABLE)
                && (primitive_type.flags.intersects(TypeFlags::PRIMITIVE | TypeFlags::NON_PRIMITIVE)
                    && !self.is_generic_string_like_type(&primitive_type)
                    || includes.intersects(TypeFlags::INCLUDES_EMPTY_OBJECT))
            {
                if let Some(constraint) = self.get_base_constraint_of_type(&type_variable)
                    && every_type_with_checker(self, &constraint, &mut |c: &mut Checker, t: &Arc<Type>| c.is_primitive_or_object_or_empty_type(t))
                {
                    if self.is_type_strict_subtype_of(&constraint, &primitive_type) {
                        return type_variable;
                    }
                    if !(constraint.flags.intersects(TypeFlags::UNION)
                        && some_type_with_checker(self, &constraint, &mut |c: &mut Checker, n: &Arc<Type>| c.is_type_strict_subtype_of(n, &primitive_type)))
                        && !self.is_type_strict_subtype_of(&primitive_type, &constraint)
                    {
                        return Arc::clone(&self.never_type());
                    }
                    object_flags |= ObjectFlags::IS_CONSTRAINED_TYPE_VARIABLE;
                }
            }
        }
        let key = get_intersection_key(&type_set, flags, alias);
        if let Some(result) = self.intersection_types.get(&key) {
            return Arc::clone(result);
        }
        let result;
        if includes.intersects(TypeFlags::UNION) {
            let (new_set, reduced) = self.intersect_unions_of_primitive_types(type_set);
            type_set = new_set;
            if reduced {
                result = self.get_intersection_type_ex(&type_set, flags, alias);
            } else if type_set.iter().all(|t| is_union_with_undefined(t)) {
                let contained_undefined_type = if type_set.iter().any(|t| self.contains_missing_type(t)) {
                    Arc::clone(&self.missing_type())
                } else {
                    Arc::clone(&self.undefined_type())
                };
                type_set.retain(|t| is_not_undefined_type(t));
                let intersection = self.get_intersection_type_ex(&type_set, flags, None);
                result = self.get_union_type_ex(vec![intersection, contained_undefined_type], UnionReduction::Literal);
            } else if type_set.iter().all(|t| is_union_with_null(t)) {
                type_set.retain(|t| is_not_null_type(t));
                let intersection = self.get_intersection_type_ex(&type_set, flags, None);
                result = self.get_union_type_ex(vec![intersection, self.null_type()], UnionReduction::Literal);
            } else if type_set.len() >= 3 && types.len() > 2 {
                let middle = type_set.len() / 2;
                let left = self.get_intersection_type_ex(&type_set[..middle], flags, None);
                let right = self.get_intersection_type_ex(&type_set[middle..], flags, None);
                result = self.get_intersection_type_ex(&[left, right], flags, alias);
            } else {
                let cross_product_ok = match self.current_node.clone() {
                    Some(current_node) => self.check_cross_product_union(&current_node, &type_set),
                    None => Checker::cross_product_union_size(&type_set) < 100_000,
                };
                if !cross_product_ok {
                    return self.error_type();
                }
                let constituents = self.get_cross_product_intersections(&type_set, flags);
                let mut origin: Option<Arc<Type>> = None;
                if constituents.iter().any(|t| is_intersection_type(t))
                    && get_constituent_count_of_types(&constituents) > get_constituent_count_of_types(&type_set)
                {
                    origin = Some(self.new_intersection_type(ObjectFlags::empty(), &type_set));
                }
                result = self.get_union_type_ex(constituents.clone(), UnionReduction::Literal);
            }
        } else {
            let mut result_inner = self.new_intersection_type(
                object_flags | self.get_propagating_flags_of_types(types, TypeFlags::NULLABLE),
                &type_set,
            );
            if let Some(alias) = alias
                && let Some(result_inner) = Arc::get_mut(&mut result_inner)
            {
                result_inner.set_alias(Some(alias.clone()));
            }
            result = result_inner;
        }
        self.intersection_types.insert(key, Arc::clone(&result));
        result
    }

    pub fn get_intersection_type_facts(&mut self, t: &Arc<Type>, caller_only_needs: TypeFacts) -> TypeFacts { ::tsox_core::fntrace::enter("get_intersection_type_facts"); 
        let ignore_objects = self.maybe_type_of_kind(t, TypeFlags::PRIMITIVE);
        let mut ored_facts = TypeFacts::empty();
        let mut anded_facts = TypeFacts::ALL;
        if let TypeData::Union(u) = &t.data {
            for ty in &u.union_or_intersection.types {
                if !(ignore_objects && ty.flags.intersects(TypeFlags::OBJECT)) {
                    let f = self.get_type_facts_worker(ty, caller_only_needs);
                    ored_facts |= f;
                    anded_facts &= f;
                }
            }
        }
        (ored_facts & TypeFacts::OR_FACTS_MASK) | (anded_facts & TypeFacts::AND_FACTS_MASK)
    }

    pub fn get_isolated_modules_like_flag_name(&self) -> &'static str { ::tsox_core::fntrace::enter("get_isolated_modules_like_flag_name"); 
        if self.compiler_options.verbatim_module_syntax.is_true() {
            "verbatimModuleSyntax"
        } else {
            "isolatedModules"
        }
    }

    pub fn get_iteration_diagnostic_details(
        &mut self,
        use_: IterationUse,
        input_type: &Arc<Type>,
        allows_strings: bool,
    ) -> (Option<&'static str>, bool) { ::tsox_core::fntrace::enter("get_iteration_diagnostic_details"); 
        let yield_type = self.get_iteration_type_of_iterable(use_, IterationTypeKind::YIELD, input_type, None);
        if yield_type.is_none() {
            return (
                Some("Type_0_can_only_be_iterated_through_when_using_the_downlevelIteration_flag_or_with_a_target_of_es2015_or_higher"),
                false,
            );
        }
        if let Some(symbol) = &input_type.symbol
            && is_es2015_or_later_iterable(&symbol.name)
        {
            return (
                Some("Type_0_can_only_be_iterated_through_when_using_the_downlevelIteration_flag_or_with_a_target_of_es2015_or_higher"),
                true,
            );
        }
        if allows_strings {
            return (Some("Type_0_is_not_an_array_type_or_a_string_type"), true);
        }
        (Some("Type_0_is_not_an_array_type"), true)
    }

    pub fn get_iteration_type_of_generator_function_return_type(
        &mut self,
        type_kind: IterationTypeKind,
        return_type: &Arc<Type>,
        is_async_generator: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_iteration_type_of_generator_function_return_type"); 
        if is_type_any(return_type) {
            return None;
        }
        let iteration_types = self.get_iteration_types_of_generator_function_return_type(return_type, is_async_generator);
        iteration_types.get_type(type_kind)
    }

    pub fn get_iteration_type_of_iterable(
        &mut self,
        use_: IterationUse,
        type_kind: IterationTypeKind,
        input_type: &Arc<Type>,
        error_node: Option<&Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_iteration_type_of_iterable"); 
        if is_type_any(input_type) {
            return None;
        }
        let iteration_types = self.get_iteration_types_of_iterable(input_type, use_, error_node);
        iteration_types.get_type(type_kind)
    }

    pub fn get_iteration_type_union(
        &mut self,
        iteration_types: &[IterationTypes],
        f: impl Fn(&IterationTypes) -> Option<Arc<Type>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_iteration_type_union"); 
        let types: Vec<Arc<Type>> = iteration_types.iter().filter_map(f).collect();
        if types.is_empty() {
            return None;
        }
        Some(self.get_union_type(types))
    }
}

fn same_signatures(a: &[Arc<Signature>], b: &[Arc<Signature>]) -> bool { ::tsox_core::fntrace::enter("same_signatures"); 
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

fn instantiation_expression_source_file(c: &crate::checker::checker::Checker, node: &Node) -> Option<Arc<tsox_frontend::ast::SourceFile>> { ::tsox_core::fntrace::enter("instantiation_expression_source_file"); 
    let mut current = node.parent()?;
    while let Some(parent) = current.parent() {
        current = parent;
    }
    if current.kind != SyntaxKind::SourceFile {
        return None;
    }
    let node_id = current.id();
    c.files.iter().find(|file| file.node.id() == node_id).cloned()
}

pub fn get_intersection_key(types: &[Arc<Type>], flags: IntersectionFlags, alias: Option<&TypeAlias>) -> CacheHashKey { ::tsox_core::fntrace::enter("get_intersection_key"); 
    let mut b = KeyBuilder::new();
    b.write_types(types);
    if !flags.intersects(IntersectionFlags::NO_CONSTRAINT_REDUCTION) {
        b.write_alias(alias);
    } else {
        b.write_byte(b'*');
    }
    b.hash()
}
