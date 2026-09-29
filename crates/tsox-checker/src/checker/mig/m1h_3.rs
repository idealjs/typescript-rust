#![allow(unused_imports)]
use super::m1c_3::contains_type;
use super::wc2_2::get_mapped_type_modifiers;
use super::wc2_2::get_modified_readonly_state;
use super::m2e_2::{has_inference_candidates, has_inference_candidates_or_default, has_overlapping_inferences};
use super::wc1b::is_tuple_type;
use super::wc3::MappedTypeModifiers;
use crate::checker::mapper::{append_type_mapping, prepend_type_mapping};
use crate::checker::types_type_flags_instantiable_non_primitive::TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE;
use crate::checker::inference_inference_key_2::{InferenceFlags, InferenceInfo, InferencePriority};

use crate::checker::checker::*;
use std::sync::Arc;

impl Checker {
    pub fn instantiate_mapped_type(
        &mut self,
        t: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
        alias: Option<&TypeAlias>,
    ) -> Arc<Type> {
        let declaration = self.get_mapped_declaration(t);
        let type_variable = self.get_homomorphic_type_variable(t);
        fn instantiate_constituent(
            c: &mut Checker,
            declaration: &Option<Arc<Node>>,
            t: &Arc<Type>,
            type_variable: &Arc<Type>,
            m: Option<&Arc<TypeMapper>>,
            s: &Arc<Type>,
        ) -> Arc<Type> {
            if !s.flags.intersects(
                TYPE_FLAGS_ANY_OR_UNKNOWN
                    | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE
                    | TypeFlags::Object
                    | TypeFlags::Intersection,
            ) || Arc::ptr_eq(s, &c.wildcard_type)
                || c.is_error_type(s)
            {
                return Arc::clone(s);
            }
            if c.mapped_name_type_is_none(declaration) {
                if c.is_array_type(s)
                    || (s.flags.intersects(TypeFlags::Any)
                        && c.find_resolution_cycle_start_index(
                            TypeSystemEntity::Type(Arc::clone(type_variable)),
                            TypeSystemPropertyName::ResolvedBaseConstraint,
                        ) < 0
                            && c.has_array_or_type_type_constraint(type_variable))
                {
                    return c.instantiate_mapped_array_type(
                        s,
                        t,
                        Some(&Arc::new(prepend_type_mapping(
                            Arc::clone(type_variable),
                            Arc::clone(s),
                            m.map(|v| &**v),
                        ))),
                    );
                }
                if is_tuple_type(s) {
                    return c.instantiate_mapped_tuple_type(s, t, type_variable, m);
                }
                if c.is_array_or_tuple_or_intersection(s) {
                    let constituents: Vec<Arc<Type>> = s
                        .types()
                        .unwrap_or(&[])
                        .iter()
                        .map(|u| {
                            instantiate_constituent(c, declaration, t, type_variable, m, u)
                        })
                        .collect();
                    return c.get_intersection_type(constituents);
                }
            }
            c.instantiate_anonymous_type(
                t,
                Some(&Arc::new(prepend_type_mapping(
                    Arc::clone(type_variable),
                    Arc::clone(s),
                    m.map(|v| &**v),
                ))),
                None,
            )
        }
        if let Some(type_variable) = &type_variable {
            let mapped_type_variable = self.instantiate_type(type_variable, m);
            if !Arc::ptr_eq(type_variable, &mapped_type_variable) {
                let reduced = self.get_reduced_type(&mapped_type_variable);
                let checker_ptr: *mut Checker = self;
                let tv = Arc::clone(type_variable);
                let reduced_fallback = Arc::clone(&reduced);
                return unsafe {
                    (*checker_ptr).map_type_with_alias(
                        &reduced,
                        &mut |s: &Arc<Type>| {
                            Some(instantiate_constituent(
                                &mut *checker_ptr,
                                &declaration,
                                t,
                                &tv,
                                m,
                                s,
                            ))
                        },
                        alias,
                    )
                }
                .unwrap_or_else(|| reduced_fallback);
            }
        }
        if let Some(constraint) = self.get_constraint_type_from_mapped_type(t) {
            let instantiated = self.instantiate_type(&constraint, m);
            if Arc::ptr_eq(&instantiated, &self.wildcard_type) {
                return Arc::clone(&self.wildcard_type);
            }
        }
        self.instantiate_anonymous_type(t, m, alias)
    }

    pub fn instantiate_mapped_array_type(
        &mut self,
        array_type: &Arc<Type>,
        mapped_type: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> {
        let number_type = self.number_type();
        let element_type =
            self.instantiate_mapped_type_template(mapped_type, &number_type, true, m);
        if self.is_error_type(&element_type) {
            return self.error_type();
        }
        let readonly = get_modified_readonly_state(
            self.is_readonly_array_type(array_type),
            get_mapped_type_modifiers(mapped_type),
        );
        self.create_array_type_ex(element_type, readonly)
    }

    pub fn instantiate_mapped_tuple_type(
        &mut self,
        tuple_type: &Arc<Type>,
        mapped_type: &Arc<Type>,
        type_variable: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> {
        let Some(target) = tuple_type.target_tuple_type() else {
            return Arc::clone(tuple_type);
        };
        let element_infos = &target.element_infos;
        let fixed_length = target.fixed_length;
        let fixed_mapper = if fixed_length != 0 {
            Some(Arc::new(prepend_type_mapping(
                Arc::clone(type_variable),
                Arc::clone(tuple_type),
                m.map(|v| &**v),
            )))
        } else {
            m.map(Arc::clone)
        };
        let modifiers = get_mapped_type_modifiers(mapped_type);
        let element_types = self.get_element_types(tuple_type);
        let mut new_element_types: Vec<Arc<Type>> = Vec::with_capacity(element_types.len());
        let mut new_element_infos = element_infos.clone();
        for (i, e) in element_types.iter().enumerate() {
            let flags = element_infos[i].flags;
            let mapped: Arc<Type>;
            if i < fixed_length {
                let key = self.get_string_literal_type(&i.to_string());
                let is_optional = flags.intersects(ElementFlags::Optional);
                mapped = self.instantiate_mapped_type_template(
                    mapped_type,
                    &key,
                    is_optional,
                    fixed_mapper.as_ref(),
                );
            } else if flags.intersects(ElementFlags::Variadic) {
                let inner = Arc::new(prepend_type_mapping(
                    Arc::clone(type_variable),
                    Arc::clone(e),
                    m.map(|v| &**v),
                ));
                mapped = self.instantiate_type(mapped_type, Some(&inner));
            } else {
                let array = self.create_array_type(Arc::clone(e));
                let inner = Arc::new(prepend_type_mapping(
                    Arc::clone(type_variable),
                    array,
                    m.map(|v| &**v),
                ));
                let instantiated = self.instantiate_type(mapped_type, Some(&inner));
                mapped = self
                    .get_element_type_of_array_type(&instantiated)
                    .unwrap_or_else(|| self.unknown_type());
            }
            if modifiers.intersects(MappedTypeModifiers::IncludeOptional) {
                if flags.intersects(ElementFlags::Required) {
                    new_element_infos[i].flags = ElementFlags::Optional;
                }
            } else if modifiers.intersects(MappedTypeModifiers::ExcludeOptional)
                && flags.intersects(ElementFlags::Optional)
            {
                new_element_infos[i].flags = ElementFlags::Required;
            }
            new_element_types.push(mapped);
        }
        let new_readonly = get_modified_readonly_state(
            target.readonly,
            get_mapped_type_modifiers(mapped_type),
        );
        if new_element_types
            .iter()
            .any(|e| Arc::ptr_eq(e, &self.error_type()))
        {
            return self.error_type();
        }
        self.create_tuple_type_ex(new_element_types, new_element_infos, new_readonly)
    }

    pub fn instantiate_mapped_type_template(
        &mut self,
        t: &Arc<Type>,
        key: &Arc<Type>,
        is_optional: bool,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> {
        let template_mapper = match self.get_type_parameter_from_mapped_type(t) {
            Some(tp) => Some(Arc::new(append_type_mapping(
                m.map(|v| &**v),
                tp,
                Arc::clone(key),
            ))),
            None => m.map(Arc::clone),
        };
        let target = self.get_mapped_target_or_self(t);
        let prop_type = match self.get_template_type_from_mapped_type(&target) {
            Some(tpl) => self.instantiate_type(&tpl, template_mapper.as_ref()),
            None => {
                let wildcard = Arc::clone(&self.wildcard_type);
                self.instantiate_type(&wildcard, template_mapper.as_ref())
            }
        };
        let modifiers = get_mapped_type_modifiers(t);
        if self.strict_null_checks
            && modifiers.intersects(MappedTypeModifiers::IncludeOptional)
            && !self
                .maybe_type_of_kind(&prop_type, TypeFlags::Undefined | TypeFlags::Void)
        {
            return self.get_optional_type(prop_type);
        }
        if self.strict_null_checks
            && modifiers.intersects(MappedTypeModifiers::ExcludeOptional)
            && is_optional
        {
            return self.remove_missing_or_undefined_type(&prop_type);
        }
        prop_type
    }

    pub fn instantiate_reverse_mapped_type(
        &mut self,
        t: &Arc<Type>,
        m: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> {
        let Some(r) = t.as_reverse_mapped_type() else {
            return Arc::clone(t);
        };
        let (Some(inner_source), Some(inner_target), Some(inner_index)) =
            (&r.mapped_type, &r.constraint_type, &r.source)
        else {
            return Arc::clone(t);
        };
        let inner_mapped_type = self.instantiate_type(inner_source, m);
        if !inner_mapped_type.object_flags.intersects(ObjectFlags::Mapped) {
            return Arc::clone(t);
        }
        let inner_index_type = self.instantiate_type(inner_target, m);
        if !inner_index_type.flags.intersects(TypeFlags::Index) {
            return Arc::clone(t);
        }
        let instantiated_source = self.instantiate_type(inner_index, m);
        if let Some(instantiated) = self.infer_type_for_homomorphic_mapped_type(
            &instantiated_source,
            &inner_mapped_type,
            &inner_index_type,
        ) {
            return instantiated;
        }
        Arc::clone(t)
    }

    pub fn instantiate_contextual_type(
        &mut self,
        contextual_type: &Arc<Type>,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Arc<Type> {
        if self.maybe_type_of_kind(contextual_type, TYPE_FLAGS_INSTANTIABLE) {
            let context = match self.get_inference_context_arc(node).cloned() {
                Some(ic) => self.clone_inference_context(Some(ic.as_ref()), InferenceFlags::None),
                None => None,
            };
            let Some(inference_context) = context else {
                return Arc::clone(contextual_type);
            };
            if context_flags.intersects(ContextFlags::Signature)
                && inference_context
                    .inferences
                    .iter()
                    .any(|i| has_inference_candidates_or_default(i))
            {
                let t = self.instantiate_instantiable_types(
                    contextual_type,
                    inference_context.mapper.as_ref(),
                );
                if !t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
                    return t;
                }
            }
            if let Some(return_mapper) = &inference_context.return_mapper {
                let t =
                    self.instantiate_instantiable_types(contextual_type, Some(return_mapper));
                if !t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
                    if t.flags.intersects(TypeFlags::Union)
                        && contains_type(t.types().unwrap_or(&[]), &self.regular_false_type())
                        && contains_type(t.types().unwrap_or(&[]), &self.regular_true_type)
                    {
                        let false_type = self.regular_false_type();
                        let true_type = self.regular_true_type.clone();
                        return self.filter_type(&t, &mut |u: &Arc<Type>| {
                            !Arc::ptr_eq(u, &false_type) && !Arc::ptr_eq(u, &true_type)
                        });
                    }
                    return t;
                }
            }
        }
        Arc::clone(contextual_type)
    }

    pub fn instantiate_instantiable_types(
        &mut self,
        t: &Arc<Type>,
        mapper: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> {
        if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE) {
            return self.instantiate_type(t, mapper);
        }
        if t.flags.intersects(TypeFlags::Union) {
            let mapped: Vec<Arc<Type>> = t
                .types()
                .unwrap_or(&[])
                .iter()
                .map(|u| self.instantiate_instantiable_types(u, mapper))
                .collect();
            return self.get_union_type_ex(mapped, UnionReduction::None);
        }
        if t.flags.intersects(TypeFlags::Intersection) {
            let mapped: Vec<Arc<Type>> = t
                .types()
                .unwrap_or(&[])
                .iter()
                .map(|u| self.instantiate_instantiable_types(u, mapper))
                .collect();
            return self.get_intersection_type(mapped);
        }
        Arc::clone(t)
    }

    pub fn instantiate_type_with_single_generic_call_signature(
        &mut self,
        node: &Arc<Node>,
        t: &Arc<Type>,
        check_mode: CheckMode,
    ) -> Arc<Type> {
        if !check_mode
            .intersects(CheckMode::Inferential | CheckMode::SkipGenericFunctions)
        {
            return Arc::clone(t);
        }
        let call_signature = self.get_single_signature(t, SignatureKind::Call, true);
        let construct_signature = self.get_single_signature(t, SignatureKind::Construct, true);
        let signature = call_signature.clone().or(construct_signature.clone());
        let signature = match signature {
            Some(sig) if !sig.type_parameters.is_empty() => sig,
            _ => return Arc::clone(t),
        };
        let contextual_type =
            self.get_apparent_type_of_contextual_type(node, ContextFlags::NoConstraints);
        let contextual_type = match contextual_type {
            Some(contextual_type) => contextual_type,
            None => return Arc::clone(t),
        };
        let kind = if call_signature.is_some() {
            SignatureKind::Call
        } else {
            SignatureKind::Construct
        };
        let non_nullable_type = self.get_non_nullable_type(&contextual_type);
        let contextual_signature = self.get_single_signature(
            &non_nullable_type,
            kind,
            false,
        );
        let contextual_signature = match contextual_signature {
            Some(sig) if sig.type_parameters.is_empty() => sig,
            _ => return Arc::clone(t),
        };
        if check_mode.intersects(CheckMode::SkipGenericFunctions) {
            self.skipped_generic_function(node, check_mode);
            return self.any_function_type();
        }
        let context = match self.get_inference_context_arc(node).cloned() {
            Some(ic) => self.clone_inference_context(Some(ic.as_ref()), InferenceFlags::None),
            None => None,
        };
        let Some(mut context) = context else {
            return Arc::clone(t);
        };
        let mut return_signature: Option<Arc<Signature>> = None;
        if let Some(context_signature) = &context.signature {
            if let Some(return_type) = self.get_return_type_of_signature(context_signature) {
                return_signature = self.get_single_call_or_construct_signature(&return_type);
            }
        }
        if let Some(return_signature) = &return_signature {
            if return_signature.type_parameters.is_empty()
                && !context
                    .inferences
                    .iter()
                    .all(|info| has_inference_candidates(info))
            {
                let unique_type_parameters =
                    self.get_unique_type_parameters(&context, &signature.type_parameters);
                let instantiated_signature = self
                    .get_signature_instantiation_without_filling_in_type_arguments(
                        &signature,
                        &unique_type_parameters,
                    );
                let inferences: Vec<InferenceInfo> = context
                    .inferences
                    .iter()
                    .map(|info| InferenceInfo::new(Arc::clone(&info.type_parameter)))
                    .collect();
                let source_sig = Arc::clone(&instantiated_signature);
                let target_sig = Arc::clone(&contextual_signature);
                let mut inferences_mut = inferences.clone();
                let mut param_pairs: Vec<(Arc<Type>, Arc<Type>)> = Vec::new();
                self.apply_to_parameter_types(
                    &source_sig,
                    &target_sig,
                    &mut |source: &Arc<Type>, target: &Arc<Type>| {
                        param_pairs.push((Arc::clone(source), Arc::clone(target)));
                    },
                );
                for (source, target) in &param_pairs {
                    self.infer_types(
                        &mut inferences_mut,
                        Some(Arc::clone(source)),
                        Some(Arc::clone(target)),
                        InferencePriority::None,
                        true,
                    );
                }
                if inferences_mut.iter().any(|info| has_inference_candidates(info)) {
                    let mut return_pairs: Vec<(Arc<Type>, Arc<Type>)> = Vec::new();
                    self.apply_to_return_types(
                        &source_sig,
                        &target_sig,
                        &mut |source: &Arc<Type>, target: &Arc<Type>| {
                            return_pairs.push((Arc::clone(source), Arc::clone(target)));
                        },
                    );
                    for (source, target) in &return_pairs {
                        self.infer_types(
                            &mut inferences_mut,
                            Some(Arc::clone(source)),
                            Some(Arc::clone(target)),
                            InferencePriority::None,
                            false,
                        );
                    }
                    if !has_overlapping_inferences(&context.inferences, &inferences_mut) {
                        self.merge_inferences(&mut context.inferences, &inferences_mut);
                        return self
                            .get_or_create_type_from_signature(&instantiated_signature);
                    }
                }
            }
        }
        Arc::clone(t)
    }
}
