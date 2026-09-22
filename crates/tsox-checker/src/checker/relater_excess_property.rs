#![allow(unused_imports)]

use crate::checker::relater_compare::*;
use crate::checker::types::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn has_excess_properties(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        if !crate::checker::relater_predicates::is_excess_property_check_target(target)
            || !self.no_implicit_any && target.object_flags.contains(ObjectFlags::JSLiteral)
        {
            return false;
        }
        let is_comparing_jsx_attributes = source.object_flags.contains(ObjectFlags::JsxAttributes);
        if matches!(relation, RelationKind::Assignable | RelationKind::Comparable)
            && (self.target_admits_any_properties(target)
                || !is_comparing_jsx_attributes && self.is_empty_object_type(target))
        {
            return false;
        }
        let mut reduced_target = Arc::clone(target);
        let mut check_types: Vec<Arc<Type>> = Vec::new();
        if target.flags.contains(TypeFlags::Union)
            && let Some(members) = target.types()
        {
            let mut effective: Vec<Arc<Type>> = Vec::new();
            if source
                .flags
                .intersects(TypeFlags::Intersection | TypeFlags::Object)
                && let Some(discriminated) =
                    self.matching_discriminant_constituents(source, target, relation)
                && discriminated.len() < members.len()
            {
                effective = discriminated;
            } else {
                let non_primitives: Vec<Arc<Type>> = members
                    .iter()
                    .filter(|m| m.flags.contains(TypeFlags::NonPrimitive))
                    .cloned()
                    .collect();
                effective = if non_primitives.is_empty() {
                    members.to_vec()
                } else {
                    non_primitives
                };
            }
            if effective.len() == 1 {
                reduced_target = Arc::clone(&effective[0]);
            } else if effective.len() != members.len() {
                reduced_target = self.get_union_type(effective.clone());
            }
            check_types = effective;
        }
        let source_props = self.get_properties_of_type(source);
        let source_symbol = source.symbol.clone();
        for prop in source_props {
            let container = match &source_symbol {
                Some(s) => Arc::clone(s),
                None => break,
            };
            if !crate::checker::relater_predicates::should_check_as_excess_property(&prop, &container)
            {
                continue;
            }
            if !self.is_known_property(&reduced_target, &prop.name, is_comparing_jsx_attributes) {
                if self.relater_chain_active {
                    let error_target = self.excess_check_error_target(&reduced_target);
                    let prop_name = crate::checker::property_name_for_display(&prop.name);
                    let target_str = self.type_to_string(&error_target);
                    if let Some(decl) = prop.value_declaration.as_ref() {
                        self.relater_excess_error_node = decl.name().cloned();
                    }
                    self.relater_report_error(
                        msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                        vec![prop_name, target_str],
                    );
                }
                return true;
            }
            if !check_types.is_empty() {
                let prop_type = self.get_type_of_symbol(&prop);
                let Some(target_prop_union) =
                    self.type_of_property_in_types(&check_types, &prop.name)
                else {
                    continue;
                };
                if !self.is_type_related_to(&prop_type, &target_prop_union, relation) {
                    if self.relater_chain_active {
                        let prop_name = crate::checker::property_name_for_display(&prop.name);
                        self.relater_report_error(
                            msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                            vec![prop_name],
                        );
                    }
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn excess_check_error_target(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(types) = t.types()
        {
            let kept: Vec<Arc<Type>> = types
                .iter()
                .filter(|m| {
                    crate::checker::relater_predicates::is_excess_property_check_target(m)
                })
                .cloned()
                .collect();
            if kept.len() == types.len() {
                return Arc::clone(t);
            }
            if kept.len() == 1 {
                return Arc::clone(&kept[0]);
            }
            if kept.is_empty() {
                return self.never_type();
            }
            return self.get_union_type(kept);
        }
        if t.flags.contains(TypeFlags::Never)
            || crate::checker::relater_predicates::is_excess_property_check_target(t)
        {
            return Arc::clone(t);
        }
        self.never_type()
    }

    fn type_of_property_in_types(
        &mut self,
        types: &[Arc<Type>],
        name: &str,
    ) -> Option<Arc<Type>> {
        let mut parts: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            parts.push(self.type_of_property_in_type(t, name));
        }
        if parts.is_empty() {
            return None;
        }
        Some(self.get_union_type(parts))
    }

    fn type_of_property_in_type(&mut self, t: &Arc<Type>, name: &str) -> Arc<Type> {
        if let Some(prop) = self.get_property_of_type(t, name) {
            return self.get_type_of_symbol(&prop);
        }
        if let Some(structured) = t.as_structured() {
            let numeric = name.parse::<f64>().is_ok();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    if key.flags.contains(TypeFlags::String)
                        || (numeric && key.flags.contains(TypeFlags::Number))
                    {
                        if let Some(v) = &info.value_type {
                            return Arc::clone(v);
                        }
                    }
                }
            }
        }
        self.undefined_type()
    }

    fn matching_discriminant_constituents(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Vec<Arc<Type>>> {
        let members: Vec<Arc<Type>> = union_target.types()?.to_vec();
        if !source
            .flags
            .intersects(TypeFlags::Intersection | TypeFlags::Object)
        {
            return None;
        }
        let source_props = self.get_properties_of_type(source);
        let mut discriminants: Vec<(String, Arc<Type>)> = Vec::new();
        for prop in source_props {
            let name = prop.name.clone();
            if self.is_discriminant_property_of_union_members(union_target, &name) {
                discriminants.push((name, self.get_type_of_symbol(&prop)));
            }
        }
        if discriminants.is_empty() {
            return None;
        }
        let mut include: Vec<bool> = Vec::with_capacity(members.len());
        for m in &members {
            include.push(
                m.flags
                    .intersects(TypeFlags::Object | TypeFlags::Intersection | TypeFlags::NonPrimitive),
            );
        }
        for (name, prop_type) in &discriminants {
            let mut matched = false;
            let mut maybe: Vec<usize> = Vec::new();
            for (i, m) in members.iter().enumerate() {
                if !include[i] {
                    continue;
                }
                if let Some(tt) = self.type_of_property_in_type_opt(m, name) {
                    if self.discriminant_matches(prop_type, &tt, relation) {
                        matched = true;
                    } else {
                        maybe.push(i);
                    }
                }
            }
            for i in maybe {
                include[i] = !matched;
            }
        }
        if include.iter().any(|b| !*b) {
            let filtered: Vec<Arc<Type>> = members
                .iter()
                .zip(include.iter())
                .filter(|(_, b)| **b)
                .map(|(m, _)| Arc::clone(m))
                .collect();
            let filtered_union = self.get_union_type(filtered);
            if !filtered_union.flags.contains(TypeFlags::Never) {
                let result: Vec<Arc<Type>> = filtered_union
                    .types()
                    .map(|ts| ts.to_vec())
                    .unwrap_or_else(|| vec![filtered_union]);
                return Some(result);
            }
        }
        None
    }

    fn discriminant_matches(
        &mut self,
        prop_type: &Arc<Type>,
        target_type: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let distributed: Vec<Arc<Type>> = prop_type
            .types()
            .map(|ts| ts.to_vec())
            .unwrap_or_else(|| vec![Arc::clone(prop_type)]);
        for s in distributed {
            if self.is_type_related_to(&s, target_type, relation) {
                return true;
            }
        }
        false
    }

    fn type_of_property_in_type_opt(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        if let Some(prop) = self.get_property_of_type(t, name) {
            return Some(self.get_type_of_symbol(&prop));
        }
        if let Some(structured) = t.as_structured() {
            let numeric = name.parse::<f64>().is_ok();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    if key.flags.contains(TypeFlags::String)
                        || (numeric && key.flags.contains(TypeFlags::Number))
                    {
                        if let Some(v) = &info.value_type {
                            return Some(Arc::clone(v));
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn is_discriminant_property_of_union_members(
        &mut self,
        union_target: &Arc<Type>,
        name: &str,
    ) -> bool {
        let Some(prop) = self.get_union_or_intersection_property(union_target, name) else {
            return false;
        };
        if !prop
            .check_flags
            .contains(tsox_frontend::ast::CheckFlags::SyntheticProperty)
        {
            return false;
        }
        if !prop.check_flags.contains(
            tsox_frontend::ast::CheckFlags::HasNonUniformType
                | tsox_frontend::ast::CheckFlags::HasLiteralType,
        ) {
            return false;
        }
        let t = self.get_type_of_symbol(&prop);
        !self.is_generic_type(&t)
    }

}
