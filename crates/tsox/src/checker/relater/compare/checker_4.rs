#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn find_most_overlappy_type(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let _ = (source, union_target);
        None
    }

    pub fn find_best_type_for_object_literal(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let _ = (source, union_target);
        None
    }

    pub fn should_report_unmatched_property_error(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        let Some(s) = source.as_structured() else {
            return true;
        };
        let type_call_signatures = s.call_signatures().len();
        let type_construct_signatures = s.construct_signatures().len();
        let type_properties = s.properties.len();
        if (type_call_signatures != 0 || type_construct_signatures != 0) && type_properties == 0 {
            let target_calls = target
                .as_structured()
                .map(|t| t.call_signatures().len())
                .unwrap_or(0);
            let target_constructs = target
                .as_structured()
                .map(|t| t.construct_signatures().len())
                .unwrap_or(0);
            if (target_calls != 0 && type_call_signatures != 0)
                || (target_constructs != 0 && type_construct_signatures != 0)
            {
                return true;
            }
            return false;
        }
        true
    }

    pub fn get_unmatched_property(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _require_optional_properties: bool,
        _match_discriminant_properties: bool,
    ) -> Option<Arc<Symbol>> {
        let _ = (source, target);
        None
    }

    pub fn get_unmatched_properties(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        require_optional_properties: bool,
        match_discriminant_properties: bool,
    ) -> Vec<Arc<Symbol>> {
        let _ = (
            source,
            target,
            require_optional_properties,
            match_discriminant_properties,
        );
        Vec::new()
    }

    pub fn find_matching_discriminant_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_related_to: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Option<Arc<Type>> {
        let _ = (source, target);
        None
    }

    pub fn find_discriminant_properties(
        &mut self,
        _source_properties: &[Arc<Symbol>],
        _target: &Arc<Type>,
    ) -> Vec<Arc<Symbol>> {
        Vec::new()
    }

    pub fn is_discriminant_property(&mut self, _t: &Arc<Type>, _name: &str) -> bool {
        false
    }

    pub fn get_matching_union_constituent_for_type(
        &mut self,
        _union_type: &Arc<Type>,
        _t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn get_key_property_name(&mut self, t: &Arc<Type>) -> Option<String> {
        let _ = t;
        None
    }

    pub fn get_constituent_type_for_key_type(
        &mut self,
        _t: &Arc<Type>,
        _key_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn filter_primitives_if_contains_non_primitive(
        &mut self,
        union_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let _ = union_type;
        None
    }

    pub fn get_type_names_for_error_display(
        &mut self,
        left: &Arc<Type>,
        right: &Arc<Type>,
    ) -> (String, String) {
        (
            self.get_type_name_for_error_display(left),
            self.get_type_name_for_error_display(right),
        )
    }

    pub fn get_type_name_for_error_display(&mut self, t: &Arc<Type>) -> String {
        crate::checker::utilities::type_to_string(t)
    }

    pub fn symbol_value_declaration_is_context_sensitive(&mut self, _symbol: &Arc<Symbol>) -> bool {
        false
    }

    pub fn type_could_have_top_level_singleton_types(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral
                | TypeFlags::UniqueESSymbol
                | TypeFlags::EnumLiteral
                | TypeFlags::TypeParameter
                | TypeFlags::IndexedAccess
                | TypeFlags::Conditional,
        ) || crate::checker::is_fresh_literal_type(t)
        {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let Some(members) = t.types() {
                return members
                    .iter()
                    .any(|m| self.type_could_have_top_level_singleton_types(m));
            }
        }
        false
    }

    pub fn get_alias_variances(&mut self, _symbol: &Arc<Symbol>) -> Vec<VarianceFlags> {
        Vec::new()
    }

    pub fn create_marker_type(
        &mut self,
        _symbol: &Arc<Symbol>,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn get_type_parameter_modifiers(&mut self, _tp: &Arc<Type>) -> crate::ast::ModifierFlags {
        crate::ast::ModifierFlags::empty()
    }

    pub fn has_covariant_void_argument(
        &mut self,
        _type_arguments: &[Arc<Type>],
        _variances: &[VarianceFlags],
    ) -> bool {
        false
    }

    pub fn is_signature_assignable_to(
        &mut self,
        _source: &Arc<Signature>,
        _target: &Arc<Signature>,
        _ignore_return_types: bool,
    ) -> bool {
        false
    }

    pub fn get_min_argument_count_ex(
        &mut self,
        sig: &Arc<Signature>,
        _flags: MinArgumentCountFlags,
    ) -> usize {
        sig.min_argument_count.max(0) as usize
    }

    pub fn get_parameter_name_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> String {
        String::new()
    }

    pub fn get_tuple_element_label(
        &mut self,
        _element_info: &TupleElementInfo,
        _rest_symbol: Option<&Arc<Symbol>>,
        _index: usize,
    ) -> String {
        String::new()
    }

    pub fn get_tuple_element_label_from_binding_element(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _index: usize,
        _element_flags: ElementFlags,
    ) -> String {
        String::new()
    }

    pub fn get_nameable_declaration_at_position(
        &mut self,
        _signature: &Arc<Signature>,
        _pos: usize,
    ) -> Option<Arc<crate::ast::Node>> {
        None
    }

    pub fn is_valid_declaration_for_tuple_label(&mut self, _d: &Arc<crate::ast::Node>) -> bool {
        false
    }

    pub fn slice_tuple_type(
        &mut self,
        _t: &Arc<Type>,
        _index: usize,
        _end_skip_count: usize,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn get_known_keys_of_tuple_type(&mut self, _t: &Arc<Type>) -> Option<Arc<Type>> {
        None
    }

    pub fn get_rest_array_type_of_tuple_type(&mut self, _t: &Arc<Type>) -> Option<Arc<Type>> {
        None
    }

    pub fn get_union_or_intersection_type_predicate(
        &mut self,
        _signatures: &[Arc<Signature>],
        _is_union: bool,
    ) -> Option<Box<TypePredicate>> {
        None
    }

    pub fn type_predicate_kinds_match(&mut self, a: &TypePredicate, b: &TypePredicate) -> bool {
        a.kind == b.kind
    }

    pub fn create_type_predicate_from_type_predicate_node(
        &mut self,
        _node: &Arc<crate::ast::Node>,
        _signature: &Arc<Signature>,
    ) -> Option<Box<TypePredicate>> {
        None
    }
}
