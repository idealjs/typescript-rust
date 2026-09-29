#![allow(unused_imports)]
use tsox_core::diagnostics::{Message, messages_generated::*};

use crate::checker::inference_inference_key_2::{InferenceFlags, InferencePriority};
use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::mig::m2c_3::some_type;
use crate::checker::mig::m1e::r20k2_defs::R20k2NodeExt;
use tsox_frontend::ast::node_data_generated::{
    is_named_tuple_member, is_optional_type_node, is_rest_type_node, is_tuple_type_node,
};
use tsox_frontend::ast::mig::m3c::question_token;
use crate::checker::mig::m2b_2::TupleNormalizer;
use tsox_frontend::ast::mig::m3g_3::skip_type_parentheses;
use super::wc1b::{every_type, walk_up_parenthesized_expressions, class_or_constructor_parameter_is_decorated, is_tuple_type, is_object_literal_type, is_array_or_tuple_type, is_literal_type, is_initialized_property};

use crate::checker::checker_checker::*;
use std::sync::Arc;

impl Checker {
    pub fn get_conditional_type(
        &mut self,
        mut root: Arc<ConditionalRoot>,
        mut mapper: Option<Arc<TypeMapper>>,
        for_constraint: bool,
        mut alias: Option<Arc<TypeAlias>>,
    ) -> Arc<Type> {
        let mut result: Option<Arc<Type>> = None;
        let mut extra_types: Vec<Arc<Type>> = Vec::new();
        let mut tail_count = 0;
        loop {
            if tail_count == 1000 {
                self.error_message(
                    &self.current_node.clone().unwrap(),TYPE_INSTANTIATION_IS_EXCESSIVELY_DEEP_AND_POSSIBLY_INFINITE,
                    &[],
                );
                return self.error_type();
            }
            let root_check_type = root.check_type.clone().unwrap_or_else(|| self.unknown_type());
            let actual_check_type = self.get_actual_type_variable(&root_check_type);
            let check_type = self.instantiate_type(&actual_check_type, mapper.as_ref());
            let extends_type =
                self.instantiate_type(root.extends_type.as_ref().unwrap(), mapper.as_ref());
            if Arc::ptr_eq(&check_type, &self.error_type())
                || Arc::ptr_eq(&extends_type, &self.error_type())
            {
                return self.error_type();
            }
            if Arc::ptr_eq(&check_type, &self.wildcard_type())
                || Arc::ptr_eq(&extends_type, &self.wildcard_type())
            {
                return self.wildcard_type();
            }
            let root_node = root.node.clone().unwrap();
            let check_type_node = skip_type_parentheses(&root_node.check_type());
            let extends_type_node = skip_type_parentheses(&root_node.extends_type());
            let check_tuples = self.is_simple_tuple_type(&check_type_node)
                && self.is_simple_tuple_type(&extends_type_node)
                && check_type_node.elements().map(|l| l.nodes.len()).unwrap_or(0)
                    == extends_type_node.elements().map(|l| l.nodes.len()).unwrap_or(0);
            let check_type_deferred = self.is_deferred_type(&check_type, check_tuples);
            let mut combined_mapper: Option<Arc<TypeMapper>> = None;
            if !root.infer_type_parameters.is_empty() {
                let mut context = self.new_inference_context(
                    &root.infer_type_parameters,
                    None,
                    InferenceFlags::None,
                    None,
                );
                if let Some(mapper) = &mapper {
                    let combined =
                        self.combine_type_mappers(context.non_fixing_mapper.as_ref(), Some(mapper));
                    context.non_fixing_mapper = combined;
                }
                if !check_type_deferred {
                    self.infer_types(
                        &mut context.inferences,
                        Some(Arc::clone(&check_type)),
                        Some(Arc::clone(&extends_type)),
                        InferencePriority::NoConstraints | InferencePriority::AlwaysStrict,
                        false,
                    );
                }
                combined_mapper = match &mapper {
                    Some(mapper) => {
                        self.combine_type_mappers(context.mapper.as_ref(), Some(mapper))
                    }
                    None => context.mapper.clone(),
                };
            }
            let inferred_extends_type = match &combined_mapper {
                Some(combined_mapper) => {
                    self.instantiate_type(root.extends_type.as_ref().unwrap(), Some(combined_mapper))
                }
                None => extends_type.clone(),
            };
            if !check_type_deferred && !self.is_deferred_type(&inferred_extends_type, check_tuples) {
                if !inferred_extends_type
                    .flags
                    .intersects(TYPE_FLAGS_ANY_OR_UNKNOWN)
                    && (check_type.flags.intersects(TypeFlags::Any)
                        || {
                            let permissive_check = self.get_permissive_instantiation(&check_type);
                            let permissive_extends =
                                self.get_permissive_instantiation(&inferred_extends_type);
                            !self.is_type_assignable_to(&permissive_check, &permissive_extends)
                        })
                {
                    if check_type.flags.intersects(TypeFlags::Any)
                        || for_constraint
                            && !inferred_extends_type.flags.intersects(TypeFlags::Never)
                            && {
                                let permissive_check =
                                    self.get_permissive_instantiation(&check_type);
                                let permissive_extends =
                                    self.get_permissive_instantiation(&inferred_extends_type);
                                let checker_ptr: *mut Checker = self;
                                some_type(&permissive_extends, &|t: &Arc<Type>| unsafe {
                                    (*checker_ptr).is_type_assignable_to(t, &permissive_check)
                                })
                            }
                    {
                        let true_type =
                            self.get_type_from_type_node(&root_node.true_type());
                        let mapper_ref = combined_mapper.as_ref().or(mapper.as_ref());
                        extra_types.push(self.instantiate_type(&true_type, mapper_ref));
                    }
                    let false_type = self.get_type_from_type_node(&root_node.false_type());
                    if false_type.flags.intersects(TypeFlags::Conditional) {
                        let new_root = false_type
                            .as_conditional_type()
                            .unwrap()
                            .root
                            .clone()
                            .unwrap();
                        if Arc::ptr_eq(
                            new_root.node.as_ref().unwrap().parent().as_ref().unwrap(),
                            &root_node,
                        )
                            && (!new_root.is_distributive
                                || Arc::ptr_eq(
                                    new_root.check_type.as_ref().unwrap(),
                                    root.check_type.as_ref().unwrap(),
                                ))
                        {
                            root = Arc::new(*new_root);
                            continue;
                        }
                        if let Some((new_root, new_root_mapper)) =
                            self.get_tail_recursion_root(&false_type, mapper.as_ref())
                        {
                            let has_alias = new_root.alias.is_some();
                            root = new_root;
                            mapper = new_root_mapper;
                            alias = None;
                            if has_alias {
                                tail_count += 1;
                            }
                            continue;
                        }
                    }
                    result = Some(self.instantiate_type(&false_type, mapper.as_ref()));
                    break;
                }
                if inferred_extends_type.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN)
                    || {
                        let restrictive_check = self.get_restrictive_instantiation(&check_type);
                        let restrictive_extends =
                            self.get_restrictive_instantiation(&inferred_extends_type);
                        self.is_type_assignable_to(&restrictive_check, &restrictive_extends)
                    }
                {
                    let true_type = self.get_type_from_type_node(&root_node.true_type());
                    let true_mapper = combined_mapper.clone().or_else(|| mapper.clone());
                    if let Some((new_root, new_root_mapper)) =
                        self.get_tail_recursion_root(&true_type, true_mapper.as_ref())
                    {
                        let has_alias = new_root.alias.is_some();
                        root = new_root;
                        mapper = new_root_mapper;
                        alias = None;
                        if has_alias {
                            tail_count += 1;
                        }
                        continue;
                    }
                    result = Some(self.instantiate_type(&true_type, true_mapper.as_ref()));
                    break;
                }
            }
            let mut deferred = self.new_conditional_type(
                (*root).clone(),
                mapper.as_ref().unwrap(),
                combined_mapper.as_ref(),
            );
            match &alias {
                Some(alias) => Arc::get_mut(&mut deferred)
                    .unwrap()
                    .set_alias(Some(alias.as_ref().clone())),
                None => {
                    let instantiated =
                        self.instantiate_type_alias(root.alias.as_deref(), mapper.as_ref());
                    if let Some(inst) = instantiated {
                        Arc::get_mut(&mut deferred).unwrap().set_alias(Some(inst));
                    }
                }
            }
            result = Some(deferred);
            break;
        }
        if !extra_types.is_empty() {
            extra_types.push(result.unwrap());
            return self.get_union_type(extra_types);
        }
        result.unwrap()
    }

    pub fn get_tail_recursion_root(
        &mut self,
        new_type: &Arc<Type>,
        new_mapper: Option<&Arc<TypeMapper>>,
    ) -> Option<(Arc<ConditionalRoot>, Option<Arc<TypeMapper>>)> {
        if new_type.flags.intersects(TypeFlags::Conditional) {
            if let Some(new_mapper) = new_mapper {
                let root = Arc::new(*new_type.as_conditional_type().unwrap().root.clone().unwrap());
                if !root.outer_type_parameters.is_empty() {
                    let type_param_mapper = self.combine_type_mappers(
                        Some(
                            new_type
                                .as_conditional_type()
                                .unwrap()
                                .mapper
                                .as_ref()
                                .unwrap(),
                        ),
                        Some(new_mapper),
                    );
                    let type_arguments: Vec<Arc<Type>> = root
                        .outer_type_parameters
                        .iter()
                        .map(|t| type_param_mapper.as_ref().unwrap().map(t))
                        .collect();
                    let new_root_mapper = Arc::new(new_type_mapper(root.outer_type_parameters.clone(), type_arguments));
                    let mut new_check_type: Option<Arc<Type>> = None;
                    if root.is_distributive {
                        new_check_type =
                            Some(new_root_mapper.map(root.check_type.as_ref().unwrap()));
                    }
                    match new_check_type {
                        None => return Some((root, Some(new_root_mapper))),
                        Some(new_check_type) => {
                            if Arc::ptr_eq(&new_check_type, root.check_type.as_ref().unwrap())
                                || !new_check_type
                                    .flags
                                    .intersects(TypeFlags::Union | TypeFlags::Never)
                            {
                                return Some((root, Some(new_root_mapper)));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn is_simple_tuple_type(&self, node: &Arc<Node>) -> bool {
        let elems = node.elements().map(|l| l.nodes.as_slice()).unwrap_or(&[]);
        is_tuple_type_node(node)
            && !elems.is_empty()
            && !elems.iter().any(|e| {
                is_optional_type_node(e)
                    || is_rest_type_node(e)
                    || (is_named_tuple_member(e)
                        && (question_token(e).is_some()
                            || e.as_named_tuple_member().dot_dot_dot_token.is_some()))
            })
    }


    pub fn create_normalized_tuple_type_ex(
        &mut self,
        target: &Arc<Type>,
        element_types: Vec<Arc<Type>>,
        object_flags: ObjectFlags,
    ) -> Arc<Type> {
        let d = target.as_tuple_type().unwrap();
        if !d.combined_flags.intersects(ELEMENT_FLAGS_NON_REQUIRED) {
            return self.create_type_reference_ex(target, &element_types, object_flags);
        }
        if d.combined_flags.intersects(ElementFlags::Variadic) {
            for (i, e) in element_types.iter().enumerate() {
                if i < d.element_infos.len()
                    && d.element_infos[i].flags.intersects(ElementFlags::Variadic)
                    && e.flags
                        .intersects(TypeFlags::Never | TypeFlags::Union)
                {
                    let check_types: Vec<Arc<Type>> = element_types
                        .iter()
                        .enumerate()
                        .map(|(j, t)| {
                            if j < d.element_infos.len()
                                && d.element_infos[j].flags.intersects(ElementFlags::Variadic)
                            {
                                Arc::clone(t)
                            } else {
                                self.unknown_type()
                            }
                        })
                        .collect();
                    if self.check_cross_product_union(&self.current_node.clone().unwrap(), &check_types) {
                        let checker_ptr: *mut Checker = self;
                        return unsafe {
                            (*checker_ptr).map_type(e, &mut |t: &Arc<Type>| {
                                let mut replaced = element_types.clone();
                                replaced[i] = Arc::clone(t);
                                Some((*checker_ptr).create_normalized_tuple_type_ex(target, replaced, object_flags))
                            })
                        }
                        .unwrap_or_else(|| Arc::clone(e));
                    }
                }
            }
        }
        let mut n = TupleNormalizer {
            types: Vec::new(),
            infos: Vec::new(),
            last_required_index: -1,
            first_rest_index: -1,
            last_optional_or_rest_index: -1,
        };
        let infos = d.element_infos.clone();
        if !n.normalize(self, &element_types[..infos.len()], &infos) {
            return self.error_type();
        }
        if element_types.len() > infos.len() {
            n.types.push(element_types[infos.len()].clone());
        }
        let tuple_target = self.get_tuple_target_type(&n.infos, d.readonly);
        if Arc::ptr_eq(&tuple_target, &self.empty_generic_type()) {
            return self.empty_object_type();
        }
        if !n.types.is_empty() {
            return self.create_type_reference_ex(&tuple_target, &n.types, object_flags);
        }
        tuple_target
    }
}
