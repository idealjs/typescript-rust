#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::inference_inference_key_2::{
    InferenceContext, InferenceFlags, InferenceInfo, InferencePriority, InferenceState,
};
use crate::checker::mig::m2c::r18k3_defs::get_string_literal_value;
use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::mig::wc1b::{is_object_literal_type, is_tuple_type};
use crate::checker::mapper::merge_type_mappers;
use super::m2e::r19k3_defs::has_type_parameter_default;
#[path = "r24k19_defs.rs"]
pub mod r24k19_defs;
use self::r24k19_defs::{
    add_intra_expression_inference_site_to, take_intra_expression_inference_sites,
};
#[path = "r30k2_defs.rs"]
pub mod r30k2_defs;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_core::jsnum::Number;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    pub(crate) fn add_intra_expression_inference_site(
        &mut self,
        n: &mut InferenceContext,
        node: &Arc<Node>,
        t: &Arc<Type>,
    ) {
        add_intra_expression_inference_site_to(n, Arc::clone(node), Arc::clone(t));
    }

    pub(crate) fn apply_to_parameter_types(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        callback: &mut dyn FnMut(&Arc<Type>, &Arc<Type>),
    ) {
        let source_count = self.get_parameter_count(source);
        let target_count = self.get_parameter_count(target);
        let source_rest_type = self.get_effective_rest_type(source);
        let target_rest_type = self.get_effective_rest_type(target);
        let mut target_non_rest_count = target_count;
        if target_rest_type.is_some() {
            target_non_rest_count -= 1;
        }
        let param_count = if source_rest_type.is_none() {
            source_count.min(target_non_rest_count)
        } else {
            target_non_rest_count
        };
        let source_this_type = self.get_this_type_of_signature(source);
        if let Some(source_this_type) = source_this_type {
            if let Some(target_this_type) = self.get_this_type_of_signature(target) {
                callback(&source_this_type, &target_this_type);
            }
        }
        for i in 0..param_count {
            let s = self.get_type_at_position(source, i);
            let t = self.get_type_at_position(target, i);
            callback(&s, &t);
        }
        if let Some(target_rest_type) = target_rest_type {
            let readonly = self.is_const_type_variable(&target_rest_type, 0)
                && !self.any_constituent_is_mutable_array_like(&target_rest_type);
            if let Some(s) = self.get_rest_type_at_position(source, param_count) {
                callback(&s, &target_rest_type);
            }
        }
    }

    pub(crate) fn apply_to_return_types(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        callback: &mut dyn FnMut(&Arc<Type>, &Arc<Type>),
    ) {
        let target_type_predicate = self.get_type_predicate_of_signature(target);
        if let Some(target_type_predicate) = target_type_predicate {
            if let Some(source_type_predicate) = self.get_type_predicate_of_signature(source) {
                if self.type_predicate_kinds_match(&source_type_predicate, &target_type_predicate)
                    && source_type_predicate.t.is_some()
                    && target_type_predicate.t.is_some()
                {
                    callback(
                        source_type_predicate.t.as_ref().unwrap(),
                        target_type_predicate.t.as_ref().unwrap(),
                    );
                    return;
                }
            }
        }
        let target_return_type = self.get_return_type_of_signature(target).unwrap();
        if self.could_contain_type_variables(&target_return_type) {
            let source_return_type = self.get_return_type_of_signature(source).unwrap();
            callback(&source_return_type, &target_return_type);
        }
    }

    pub(crate) fn clone_inference_context(
        &mut self,
        n: Option<&InferenceContext>,
        extra_flags: InferenceFlags,
    ) -> Option<Box<InferenceContext>> {
        let n = n?;
        let inferences = n.inferences.iter().map(clone_inference_info).collect();
        Some(self.new_inference_context_worker(inferences, n.signature.clone(), n.flags | extra_flags))
    }

    pub(crate) fn clone_inferred_part_of_context(&mut self, n: &InferenceContext) -> Option<Box<InferenceContext>> {
        let inferences: Vec<InferenceInfo> =
            n.inferences.iter().filter(|info| has_inference_candidates(info)).cloned().collect();
        if inferences.is_empty() {
            return None;
        }
        Some(self.new_inference_context_worker(inferences, n.signature.clone(), n.flags))
    }

    pub(crate) fn create_empty_object_type_from_string_literal(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let mut members = SymbolTable::new();
        for dt in t.distributed() {
            if !dt.flags.contains(TypeFlags::STRING_LITERAL) {
                continue;
            }
            let name = get_string_literal_value(&dt);
            let mut literal_prop = self.new_symbol(SymbolFlags::Property, &name);
            if let Some(symbol) = &dt.symbol {
                let prop_mut = Arc::get_mut(&mut literal_prop).unwrap();
                prop_mut.declarations = symbol.declarations.clone();
                prop_mut.value_declaration = symbol.value_declaration.clone();
            }
            let any_type = self.any_type();
            self.value_symbol_links
                .get_or_default(&literal_prop)
                .resolved_type = Some(any_type);
            members.insert(&name, literal_prop);
        }
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        if t.flags.contains(TypeFlags::STRING) {
            let string_type = self.string_type();
            let empty_object_type = self.empty_object_type();
            index_infos.push(self.new_index_info(
                &string_type,
                &empty_object_type,
                false,
                None,
                &[],
            ));
        }
        let result = self.new_object_type(ObjectFlags::Anonymous, t.symbol.clone());
        self.set_structured_type_members(&result, Some(members), Vec::new(), Vec::new(), index_infos);
        result
    }

    pub(crate) fn create_outer_return_mapper(&mut self, context: &mut InferenceContext) -> Option<Arc<TypeMapper>> {
        if context.outer_return_mapper.is_none() {
            let cloned = self.clone_inference_context(Some(context), InferenceFlags::None)?;
            let mut mapper = cloned.mapper.clone();
            if let Some(return_mapper) = &context.return_mapper {
                mapper = merge_type_mappers(Some(return_mapper.as_ref()), mapper.as_deref()).map(Arc::new);
            }
            context.outer_return_mapper = mapper;
        }
        context.outer_return_mapper.clone()
    }

    pub(crate) fn get_limited_constraint(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let reverse_mapped = t.as_reverse_mapped_type()?;
        let constraint = self.get_constraint_type_from_mapped_type(reverse_mapped.mapped_type.as_ref()?)?;
        if !(constraint.flags.contains(TypeFlags::Union) || constraint.flags.contains(TypeFlags::Intersection)) {
            return None;
        }
        let mut origin = Arc::clone(&constraint);
        if constraint.flags.contains(TypeFlags::Union) {
            origin = Arc::clone(constraint.as_union_type()?.origin.as_ref()?);
        }
        if !origin.flags.contains(TypeFlags::Intersection) {
            return None;
        }
        let constraint_type = reverse_mapped.constraint_type.clone()?;
        let limited_constraint = self.get_intersection_type(
            origin
                .types()
                .unwrap_or_default()
                .iter()
                .filter(|t| !Arc::ptr_eq(*t, &constraint_type))
                .cloned()
                .collect(),
        );
        if !Arc::ptr_eq(&limited_constraint, &self.never_type()) {
            return Some(limited_constraint);
        }
        None
    }

    pub(crate) fn get_mapper_from_context(&self, n: Option<&InferenceContext>) -> Option<Arc<TypeMapper>> {
        n?.mapper.clone()
    }

    pub(crate) fn get_type_of_reverse_mapped_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let resolved = self.value_symbol_links.get(symbol).and_then(|l| l.resolved_type.clone());
        if resolved.is_none() {
            let Some(reverse_links) = self.reverse_mapped_symbol_links.get(symbol) else {
                return Arc::clone(&self.unknown_type());
            };
            let (Some(property_type), Some(mapped_type), Some(constraint_type)) = (
                reverse_links.property_type.clone(),
                reverse_links.mapped_type.clone(),
                reverse_links.constraint_type.clone(),
            ) else {
                return Arc::clone(&self.unknown_type());
            };
            let t = self
                .infer_reverse_mapped_type(&property_type, &mapped_type, &constraint_type)
                .unwrap_or_else(|| Arc::clone(&self.unknown_type()));
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return t;
        }
        resolved.unwrap()
    }

    pub(crate) fn infer_from_contravariant_types(&mut self, n: &mut InferenceState, source: &Arc<Type>, target: &Arc<Type>) {
        n.contravariant = !n.contravariant;
        self.infer_from_types(n, source, target);
        n.contravariant = !n.contravariant;
    }

    pub(crate) fn infer_from_contravariant_types_if_strict_function_types(
        &mut self,
        n: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        if self.strict_function_types || n.priority.contains(InferencePriority::AlwaysStrict) {
            self.infer_from_contravariant_types(n, source, target);
        } else {
            self.infer_from_types(n, source, target);
        }
    }

    pub(crate) fn infer_from_contravariant_types_with_priority(
        &mut self,
        n: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
        new_priority: InferencePriority,
    ) {
        let save_priority = n.priority;
        n.priority |= new_priority;
        self.infer_from_contravariant_types(n, source, target);
        n.priority = save_priority;
    }

    pub(crate) fn infer_from_intra_expression_sites(&mut self, n: &mut InferenceContext) {
        let sites = take_intra_expression_inference_sites(n);
        for site in &sites {
            let contextual_type = if site.node.kind == SyntaxKind::MethodDeclaration {
                self.get_contextual_type_for_object_literal_method(&site.node, ContextFlags::NoConstraints)
            } else {
                self.get_contextual_type(&site.node, ContextFlags::NoConstraints)
            };
            if let Some(contextual_type) = contextual_type {
                self.infer_types(
                    &mut n.inferences,
                    Some(Arc::clone(&site.t)),
                    Some(Arc::clone(&contextual_type)),
                    InferencePriority::None,
                    false,
                );
            }
        }
    }

    pub(crate) fn infer_to_multiple_types_with_priority(
        &mut self,
        n: &mut InferenceState,
        source: &Arc<Type>,
        targets: &[Arc<Type>],
        target_flags: TypeFlags,
        new_priority: InferencePriority,
    ) {
        let save_priority = n.priority;
        n.priority |= new_priority;
        if target_flags.contains(TypeFlags::Union) || target_flags.contains(TypeFlags::Enum) {
            self.infer_to_multiple_types_union(n, source, targets);
        } else {
            self.infer_to_multiple_types_non_union(n, source, targets, InferencePriority::None);
        }
        n.priority = save_priority;
    }

    pub(crate) fn is_partially_inferable_type(&mut self, t: &Arc<Type>) -> bool {
        !t.object_flags.contains(ObjectFlags::NonInferrableType)
            || is_object_literal_type(t) && self.get_properties_of_type(t).iter().any(|prop| {
                let prop_type = self.get_type_of_symbol(prop);
                self.is_partially_inferable_type(&prop_type)
            })
            || is_tuple_type(t) && self.get_element_types(t).iter().any(|e| self.is_partially_inferable_type(e))
    }

    pub(crate) fn merge_inferences(&mut self, target: &mut [InferenceInfo], source: &[InferenceInfo]) {
        for i in 0..target.len() {
            if !has_inference_candidates(&target[i]) && has_inference_candidates(&source[i]) {
                target[i] = source[i].clone();
            }
        }
    }

    pub(crate) fn new_inference_context(
        &mut self,
        type_parameters: &[Arc<Type>],
        signature: Option<Arc<Signature>>,
        flags: InferenceFlags,
        _compare_types: Option<TypeComparer>,
    ) -> Box<InferenceContext> {
        let inferences = type_parameters.iter().map(|tp| new_inference_info(tp)).collect();
        self.new_inference_context_worker(inferences, signature, flags)
    }

    pub(crate) fn new_inference_context_worker(
        &mut self,
        inferences: Vec<InferenceInfo>,
        signature: Option<Arc<Signature>>,
        flags: InferenceFlags,
    ) -> Box<InferenceContext> {
        let mut n = Box::new(InferenceContext {
            inferences,
            signature,
            flags,
            inferred_type_parameters: Vec::new(),
            mapper: None,
            non_fixing_mapper: None,
            return_mapper: None,
            outer_return_mapper: None,
        });
        n.mapper = Some(r30k2_defs::new_inference_type_mapper(self, &mut n, true));
        n.non_fixing_mapper = Some(r30k2_defs::new_inference_type_mapper(self, &mut n, false));
        n
    }

    pub(crate) fn replace_indexed_access(
        &mut self,
        instantiable: &Arc<Type>,
        t: &Arc<Type>,
        replacement: &Arc<Type>,
    ) -> Arc<Type> {
        let Some(indexed_access) = t.as_indexed_access_type() else {
            return Arc::clone(instantiable);
        };
        let (Some(index_type), Some(object_type)) =
            (indexed_access.index_type.clone(), indexed_access.object_type.clone())
        else {
            return Arc::clone(instantiable);
        };
        let mapper = Arc::new(new_type_mapper(
            vec![index_type, object_type],
            vec![
                self.get_number_literal_type(Number::from(0i64)),
                self.create_tuple_type(vec![Arc::clone(replacement)]),
            ],
        ));
        self.instantiate_type(instantiable, Some(&mapper))
    }

    pub(crate) fn types_definitely_unrelated(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if is_tuple_type(source) && is_tuple_type(target) {
            return tuple_types_definitely_unrelated(source, target);
        }
        self.get_unmatched_property(source, target, false, true).is_some()
            && self.get_unmatched_property(target, source, false, false).is_some()
    }

    pub(crate) fn any_constituent_is_mutable_array_like(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(TypeFlags::Union) {
            if let Some(types) = t.types() {
                return types.iter().any(|t| self.is_mutable_array_like_type(t));
            }
            return false;
        }
        self.is_mutable_array_like_type(t)
    }
}

pub(crate) fn clear_cached_inferences(inferences: &mut [InferenceInfo]) {
    for inference in inferences {
        if !inference.is_fixed {
            inference.inferred_type = None;
        }
    }
}

pub(crate) fn clone_inference_info(info: &InferenceInfo) -> InferenceInfo {
    InferenceInfo {
        type_parameter: Arc::clone(&info.type_parameter),
        candidates: info.candidates.clone(),
        candidate_depths: info.candidate_depths.clone(),
        contra_candidates: info.contra_candidates.clone(),
        inferred_type: info.inferred_type.clone(),
        priority: info.priority,
        top_level: info.top_level,
        is_fixed: info.is_fixed,
        implied_arity: info.implied_arity,
    }
}

pub(crate) fn get_inference_info_for_type<'a>(n: &'a InferenceState, t: &Arc<Type>) -> Option<&'a InferenceInfo> {
    if t.flags.contains(TypeFlags::TYPE_VARIABLE) {
        for inference in n.inferences.iter() {
            if Arc::ptr_eq(t, &inference.type_parameter) {
                return Some(inference);
            }
        }
    }
    None
}

pub(crate) fn get_single_type_variable_from_intersection_types(
    n: &InferenceState,
    types: &[Arc<Type>],
) -> Option<Arc<Type>> {
    let mut type_variable: Option<Arc<Type>> = None;
    for t in types {
        if !t.flags.contains(TypeFlags::INTERSECTION) {
            return None;
        }
        let v = t
            .types()?
            .iter()
            .find(|t| get_inference_info_for_type(n, t).is_some())?;
        if type_variable.is_some() && !Arc::ptr_eq(type_variable.as_ref().unwrap(), v) {
            return None;
        }
        type_variable = Some(Arc::clone(v));
    }
    type_variable
}

pub(crate) fn has_inference_candidates(info: &InferenceInfo) -> bool {
    !info.candidates.is_empty() || !info.contra_candidates.is_empty()
}

pub(crate) fn has_inference_candidates_or_default(info: &InferenceInfo) -> bool {
    has_inference_candidates(info) || has_type_parameter_default(&info.type_parameter)
}

pub(crate) fn has_overlapping_inferences(a: &[InferenceInfo], b: &[InferenceInfo]) -> bool {
    for i in 0..a.len() {
        if has_inference_candidates(&a[i]) && has_inference_candidates(&b[i]) {
            return true;
        }
    }
    false
}

pub(crate) fn new_inference_info(type_parameter: &Arc<Type>) -> InferenceInfo {
    InferenceInfo::new(Arc::clone(type_parameter))
}

pub(crate) fn tuple_types_definitely_unrelated(source: &Arc<Type>, target: &Arc<Type>) -> bool {
    let (Some(s), Some(t)) = (source.target_tuple_type(), target.target_tuple_type()) else {
        return false;
    };
    (t.combined_flags & ElementFlags::Variadic).is_empty() && t.min_length > s.min_length
        || (t.combined_flags & ELEMENT_FLAGS_VARIABLE).is_empty()
            && (!(s.combined_flags & ELEMENT_FLAGS_VARIABLE).is_empty()
                || t.fixed_length < s.fixed_length)
}
