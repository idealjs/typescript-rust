use crate::checker::mig::m1c_3::contains_type;
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2b::r22k6_defs;
use crate::checker::mig::m1g_5::get_type_list_key;
use crate::checker::mig::m1h_2::insert_type;
use crate::checker::mig::wc1b::every_type;
use crate::checker::utilities_is_optional_symbol::is_fresh_literal_type;
use crate::checker::utilities_has_only_expression_initialization::compare_types;
use crate::checker::utilities_token_is_identifier_or_keyword::is_unit_type;
use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol};

pub fn resolution_extension_is_ts_or_json(ext: &str) -> bool { ::tsox_core::fntrace::enter("resolution_extension_is_ts_or_json"); 
    tsox_core::tspath::extension_is_ts(ext) || ext == tsox_core::tspath::EXTENSION_JSON
}

impl Checker {
    pub fn remove_constrained_type_variables(&mut self, types: &[Arc<Type>]) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("remove_constrained_type_variables"); 
        let mut types = types.to_vec();
        let mut type_variables: Vec<Arc<Type>> = Vec::new();
        for t in &types {
            if t.flags.intersects(TypeFlags::Intersection)
                && t.object_flags
                    .intersects(ObjectFlags::IsConstrainedTypeVariable)
            {
                if let TypeData::Intersection(intersection) = &t.data {
                    let mut index = 0;
                    if !intersection.union_or_intersection.types[0]
                        .flags
                        .intersects(TypeFlags::TYPE_VARIABLE)
                    {
                        index = 1;
                    }
                    let candidate = Arc::clone(&intersection.union_or_intersection.types[index]);
                    if !type_variables
                        .iter()
                        .any(|existing| Arc::ptr_eq(existing, &candidate))
                    {
                        type_variables.push(candidate);
                    }
                }
            }
        }
        for type_variable in &type_variables {
            let mut primitives: Vec<Arc<Type>> = Vec::new();
            for t in &types {
                if t.flags.intersects(TypeFlags::Intersection)
                    && t.object_flags
                        .intersects(ObjectFlags::IsConstrainedTypeVariable)
                {
                    if let TypeData::Intersection(intersection) = &t.data {
                        let mut index = 0;
                        if !intersection.union_or_intersection.types[0]
                            .flags
                            .intersects(TypeFlags::TYPE_VARIABLE)
                        {
                            index = 1;
                        }
                        if Arc::ptr_eq(
                            &intersection.union_or_intersection.types[index],
                            type_variable,
                        ) {
                            let primitive =
                                Arc::clone(&intersection.union_or_intersection.types[1 - index]);
                            let (updated, inserted) = insert_type(&primitives, &primitive);
                            primitives = updated;
                            let _ = inserted;
                        }
                    }
                }
            }
            let constraint = self.get_base_constraint_of_type(type_variable);
            let every_covered = constraint
                .as_ref()
                .is_some_and(|constraint| {
                    every_type(constraint, &|t| contains_type(&primitives, t))
                });
            if every_covered {
                let mut i = types.len();
                while i > 0 {
                    i -= 1;
                    let t = Arc::clone(&types[i]);
                    if t.flags.intersects(TypeFlags::Intersection)
                        && t.object_flags
                            .intersects(ObjectFlags::IsConstrainedTypeVariable)
                    {
                        if let TypeData::Intersection(intersection) = &t.data {
                            let mut index = 0;
                            if !intersection.union_or_intersection.types[0]
                                .flags
                                .intersects(TypeFlags::TYPE_VARIABLE)
                            {
                                index = 1;
                            }
                            if Arc::ptr_eq(
                                &intersection.union_or_intersection.types[index],
                                type_variable,
                            ) && contains_type(
                                &primitives,
                                &intersection.union_or_intersection.types[1 - index],
                            )
                            {
                                types.remove(i);
                            }
                        }
                    }
                }
                let (updated, inserted) = insert_type(&types, type_variable);
                types = updated;
                let _ = inserted;
            }
        }
        types
    }

    pub fn remove_definitely_falsy_types(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_definitely_falsy_types"); 
        r22k6_defs::filter_type_ext(self, t, &mut |c, t| {
            c.has_type_facts(t, TypeFacts::TRUTHY)
        })
    }

    pub fn remove_nullable_by_intersection(
        &mut self,
        t: &Arc<Type>,
        target_facts: TypeFacts,
        other_facts: TypeFacts,
        other_includes_facts: TypeFacts,
        other_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_nullable_by_intersection"); 
        let facts = self.get_type_facts(
            t,
            TypeFacts::EQ_UNDEFINED | TypeFacts::EQ_NULL | TypeFacts::IS_UNDEFINED | TypeFacts::IS_NULL,
        );
        if !facts.intersects(target_facts) {
            return Arc::clone(t);
        }
        let empty_object_type = self.empty_object_type();
        let empty_and_other_union =
            self.get_union_type(vec![empty_object_type, Arc::clone(other_type)]);
        r22k6_defs::map_type_ext(self, t, &mut |c, t| {
            if c.has_type_facts(t, target_facts) {
                if !facts.intersects(other_includes_facts)
                    && c.has_type_facts(t, other_facts)
                {
                    return Some(c.get_intersection_type(vec![
                        Arc::clone(t),
                        Arc::clone(&empty_and_other_union),
                    ]));
                }
                let empty_object_type = c.empty_object_type();
                return Some(c.get_intersection_type(vec![
                    Arc::clone(t),
                    empty_object_type,
                ]));
            }
            Some(Arc::clone(t))
        })
        .unwrap_or_else(|| Arc::clone(t))
    }

    pub fn remove_optionality_from_declared_type(
        &mut self,
        declared_type: &Arc<Type>,
        declaration: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_optionality_from_declared_type"); 
        let remove_undefined = self.strict_null_checks
            && tsox_frontend::ast::is_parameter_declaration(declaration)
            && declaration.initializer().is_some()
            && self.has_type_facts(declared_type, TypeFacts::IS_UNDEFINED)
            && !self.parameter_initializer_contains_undefined(declaration);
        if remove_undefined {
            return self.get_type_with_facts(declared_type, TypeFacts::NE_UNDEFINED);
        }
        Arc::clone(declared_type)
    }

    pub fn remove_redundant_literal_types(
        &mut self,
        types: &[Arc<Type>],
        includes: TypeFlags,
        reduce_void_undefined: bool,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("remove_redundant_literal_types"); 
        let mut types = types.to_vec();
        let mut i = types.len();
        while i > 0 {
            i -= 1;
            let t = Arc::clone(&types[i]);
            let flags = t.flags;
            let remove = (flags
                .intersects(
                    TypeFlags::StringLiteral
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                )
                && includes.intersects(TypeFlags::String))
                || (flags.intersects(TypeFlags::NumberLiteral)
                    && includes.intersects(TypeFlags::Number))
                || (flags.intersects(TypeFlags::BigIntLiteral)
                    && includes.intersects(TypeFlags::BigInt))
                || (flags.intersects(TypeFlags::UniqueESSymbol)
                    && includes.intersects(TypeFlags::ESSymbol))
                || (reduce_void_undefined
                    && flags.intersects(TypeFlags::Undefined)
                    && includes.intersects(TypeFlags::Void))
                || (is_fresh_literal_type(&t)
                    && types.iter().any(|existing| {
                        if let TypeData::Literal(literal) = &t.data {
                            literal
                                .regular_type
                                .get()
                                .is_some_and(|r| Arc::ptr_eq(r, existing))
                        } else {
                            false
                        }
                    }));
            if remove {
                types.remove(i);
            }
        }
        types
    }

    pub fn remove_redundant_supertypes(
        &mut self,
        types: &[Arc<Type>],
        includes: TypeFlags,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("remove_redundant_supertypes"); 
        let mut types = types.to_vec();
        let mut i = types.len();
        while i > 0 {
            i -= 1;
            let t = Arc::clone(&types[i]);
            let remove = (t.flags.intersects(TypeFlags::String)
                && includes.intersects(
                    TypeFlags::StringLiteral | TypeFlags::TemplateLiteral | TypeFlags::StringMapping,
                ))
                || (t.flags.intersects(TypeFlags::Number)
                    && includes.intersects(TypeFlags::NumberLiteral))
                || (t.flags.intersects(TypeFlags::BigInt)
                    && includes.intersects(TypeFlags::BigIntLiteral))
                || (t.flags.intersects(TypeFlags::ESSymbol)
                    && includes.intersects(TypeFlags::UniqueESSymbol))
                || (t.flags.intersects(TypeFlags::Void)
                    && includes.intersects(TypeFlags::Undefined))
                || (self.is_empty_anonymous_object_type(&t)
                    && includes.intersects(TypeFlags::DEFINITELY_NON_NULLABLE));
            if remove {
                types.remove(i);
            }
        }
        types
    }

    pub fn remove_string_literals_matched_by_template_literals(
        &mut self,
        types: &[Arc<Type>],
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("remove_string_literals_matched_by_template_literals"); 
        let mut types = types.to_vec();
        let templates: Vec<Arc<Type>> = types
            .iter()
            .filter(|t| self.is_pattern_literal_type(t))
            .cloned()
            .collect();
        if !templates.is_empty() {
            let mut i = types.len();
            while i > 0 {
                i -= 1;
                let t = Arc::clone(&types[i]);
                if t.flags.intersects(TypeFlags::StringLiteral)
                    && templates.iter().any(|template| {
                        self.is_type_matched_by_template_literal_or_string_mapping(&t, template)
                    })
                {
                    types.remove(i);
                }
            }
        }
        types
    }

    pub fn remove_subtypes(
        &mut self,
        types: &[Arc<Type>],
        has_object_types: bool,
    ) -> Option<Vec<Arc<Type>>> { ::tsox_core::fntrace::enter("remove_subtypes"); 
        if types.len() < 2 {
            return Some(types.to_vec());
        }
        let key = get_type_list_key(types);
        if let Some(cached) = r22k6_defs::subtype_reduction_cache_get(&key) {
            return Some(cached);
        }
        let mut types = types.to_vec();
        let has_empty_object = has_object_types
            && types.iter().any(|t| {
                t.flags.intersects(TypeFlags::Object)
                    && !self.is_generic_mapped_type(t)
                    && self.is_empty_resolved_type(&self.resolve_structured_type_members(t))
            });
        let length = types.len();
        let mut i = length;
        let mut count = 0usize;
        while i > 0 {
            i -= 1;
            let source = Arc::clone(&types[i]);
            if has_empty_object || source.flags.intersects(TypeFlags::STRUCTURED_OR_INSTANTIABLE) {
                let base_constraint = self.get_base_constraint_or_type(&source);
                if source.flags.intersects(TypeFlags::TypeParameter)
                    && base_constraint.flags.intersects(TypeFlags::Union)
                {
                    let mut others: Vec<Arc<Type>> = Vec::with_capacity(types.len());
                    let never_type = self.never_type();
                    for t in &types {
                        if Arc::ptr_eq(t, &source) {
                            others.push(Arc::clone(&never_type));
                        } else {
                            others.push(Arc::clone(t));
                        }
                    }
                    let union = self.get_union_type(others);
                    let strict_subtyping = self.strict_subtyping_relation();
                    if self.is_type_related_to(&source, &union, strict_subtyping) {
                        types.remove(i);
                    }
                    continue;
                }
                let mut key_property: Option<Arc<Symbol>> = None;
                let mut key_property_type: Option<Arc<Type>> = None;
                if source.flags.intersects(
                    TypeFlags::Object | TypeFlags::Intersection | TypeFlags::INSTANTIABLE_NON_PRIMITIVE,
                ) {
                    let properties = self.get_properties_of_type(&source);
                    key_property = properties
                        .into_iter()
                        .find(|p| {
                            let property_type = self.get_type_of_symbol(p);
                            is_unit_type(&property_type)
                        });
                }
                if let Some(key_property_ref) = &key_property {
                    let property_type = self.get_type_of_symbol(key_property_ref);
                    key_property_type =
                        Some(self.get_regular_type_of_literal_type(&property_type));
                }
                for target_index in 0..types.len() {
                    let target = Arc::clone(&types[target_index]);
                    if !Arc::ptr_eq(&source, &target) {
                        if count == 100000 {
                            let estimated_count = (count / (length - i)) * length;
                            if estimated_count > 1000000 {
                                self.error_message(
                                    &self.current_node.clone().unwrap(),tsox_core::diagnostics::messages_generated::EXPRESSION_PRODUCES_A_UNION_TYPE_THAT_IS_TOO_COMPLEX_TO_REPRESENT,
                                    &[],
                                );
                                return None;
                            }
                        }
                        count += 1;
                        if key_property.is_some()
                            && target.flags.intersects(
                                TypeFlags::Object
                                    | TypeFlags::Intersection
                                    | TypeFlags::INSTANTIABLE_NON_PRIMITIVE,
                            )
                        {
                            let key_name = key_property.as_ref().unwrap().name.clone();
                            let t = self.get_type_of_property_of_type(&target, &key_name);
                            if let Some(t) = t {
                                if is_unit_type(&t)
                                    && !option_arc_ptr_eq(
                                        &Some(self.get_regular_type_of_literal_type(&t)),
                                        &key_property_type,
                                    )
                                {
                                    continue;
                                }
                            }
                        }
                        let empty_object_type = self.empty_object_type();
                        let unknown_empty_object_type = self.unknown_empty_object_type();
                        if (Arc::ptr_eq(&source, &empty_object_type)
                            || Arc::ptr_eq(&source, &unknown_empty_object_type))
                            && target.symbol.is_some()
                            && self.is_empty_anonymous_object_type(&target)
                        {
                            continue;
                        }
                        let strict_subtyping = self.strict_subtyping_relation();
                        let source_target_type = self.get_target_type(&source);
                        let target_target_type = self.get_target_type(&target);
                        if self.is_type_related_to(&source, &target, strict_subtyping)
                            && (!source_target_type
                                .object_flags
                                .intersects(ObjectFlags::Class)
                                || !target_target_type
                                    .object_flags
                                    .intersects(ObjectFlags::Class)
                                || self.is_type_derived_from(&source, &target))
                        {
                            types.remove(i);
                            break;
                        }
                    }
                }
            }
        }
        r22k6_defs::subtype_reduction_cache_insert(key, types.clone());
        Some(types)
    }

    pub fn remove_type(&mut self, t: &Arc<Type>, target_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_type"); 
        if !t.flags.intersects(TypeFlags::Union) {
            if Arc::ptr_eq(t, target_type) {
                return self.never_type();
            }
            return Arc::clone(t);
        }
        if let TypeData::Union(union) = &t.data {
            if let Some(origin) = &union.origin {
                if origin.flags.intersects(TypeFlags::Union)
                    && origin
                        .types()
                        .is_some_and(|ots| contains_type(ots, target_type))
                {
                    return self.filter_type(t, &mut |t| !Arc::ptr_eq(t, target_type));
                }
            }
        }
        let types = t.types().unwrap_or(&[]).to_vec();
        if let Ok(i) = types
            .binary_search_by(|probe| compare_types(probe, target_type))
        {
            if types.len() == 2 {
                return Arc::clone(&types[1 - i]);
            }
            let mut filtered = types.clone();
            filtered.remove(i);
            let object_flags = t.object_flags
                & (ObjectFlags::PrimitiveUnion | ObjectFlags::ContainsIntersections);
            return self.get_union_type_from_sorted_list(filtered, object_flags, None, None);
        }
        Arc::clone(t)
    }
}

fn option_arc_ptr_eq(a: &Option<Arc<Type>>, b: &Option<Arc<Type>>) -> bool { ::tsox_core::fntrace::enter("option_arc_ptr_eq"); 
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}
