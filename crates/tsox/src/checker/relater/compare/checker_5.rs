#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn instantiate_type_predicate(
        &mut self,
        _predicate: &TypePredicate,
        _mapper: &Arc<TypeMapper>,
    ) -> Option<Box<TypePredicate>> {
        None
    }

    pub fn new_type_predicate(
        &mut self,
        kind: TypePredicateKind,
        parameter_name: String,
        parameter_index: i32,
        t: Arc<Type>,
    ) -> Box<TypePredicate> {
        Box::new(TypePredicate {
            kind,
            parameter_name,
            parameter_index,
            t: Some(t),
        })
    }

    pub fn is_resolving_return_type_of_signature(&mut self, _signature: &Arc<Signature>) -> bool {
        false
    }

    pub fn find_matching_signatures(
        &mut self,
        _signature_lists: &[Vec<Arc<Signature>>],
        _signature: &Arc<Signature>,
        _list_index: usize,
    ) -> Vec<Arc<Signature>> {
        Vec::new()
    }

    pub fn is_matching_signature(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        partial_match: bool,
    ) -> bool {
        self.compare_signatures_identical(source, target, partial_match, false, false)
            != Ternary::False
    }

    pub fn compare_type_predicates_identical(
        &mut self,
        source: &TypePredicate,
        target: &TypePredicate,
        _compare_types: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Ternary {
        if source.kind != target.kind {
            return Ternary::False;
        }
        if source.parameter_name != target.parameter_name {
            return Ternary::False;
        }
        Ternary::True
    }

    pub fn get_effective_constraint_of_intersection(
        &mut self,
        _types: &[Arc<Type>],
        _target_is_union: bool,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn template_literal_types_definitely_unrelated(
        &mut self,
        _source: &TemplateLiteralTypeData,
        _target: &TemplateLiteralTypeData,
    ) -> bool {
        false
    }

    pub fn is_type_matched_by_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
        _compare_types: TypeComparer,
    ) -> bool {
        false
    }

    pub fn infer_types_from_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
    ) -> Vec<Arc<Type>> {
        Vec::new()
    }

    pub fn get_string_like_type_for_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.intersects(TYPE_FLAGS_STRING_LIKE) {
            Some(Arc::clone(t))
        } else {
            None
        }
    }

    pub fn is_valid_type_for_template_literal_placeholder(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
        _compare_types: TypeComparer,
    ) -> bool {
        false
    }

    pub fn is_member_of_string_mapping(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> bool {
        false
    }

    pub fn apply_target_string_mapping_to_source(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> (Arc<Type>, Arc<Type>) {
        (Arc::clone(source), Arc::clone(target))
    }

    pub fn get_type_of_property_in_types(
        &mut self,
        _types: &[Arc<Type>],
        _name: &str,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn get_type_of_property_in_type(
        &mut self,
        _t: &Arc<Type>,
        _name: &str,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn is_type_subset_of_union(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        self.is_type_subset_of(source, target)
    }

    pub fn is_type_derived_from(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        self.is_type_assignable_to(source, target)
    }

    pub fn is_distribution_dependent(&mut self, _root: &ConditionalRoot) -> bool {
        false
    }
}
