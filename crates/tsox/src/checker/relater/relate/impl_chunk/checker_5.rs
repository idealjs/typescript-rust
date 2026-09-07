#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_tuple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_tuple = match &source.data {
            TypeData::Tuple(t) => t,
            _ => return false,
        };
        let target_tuple = match &target.data {
            TypeData::Tuple(t) => t,
            _ => return false,
        };

        let min_len = source_tuple
            .element_infos
            .len()
            .min(target_tuple.element_infos.len());
        for i in 0..min_len {
            let source_elem = &source_tuple.element_infos[i];
            let target_elem = &target_tuple.element_infos[i];

            let source_type = self.get_tuple_element_type(source, i);
            let target_type = self.get_tuple_element_type(target, i);

            if let (Some(st), Some(tt)) = (source_type, target_type) {
                if !self.is_type_related_to(&st, &tt, relation) {
                    return false;
                }
            }

            if !self.is_element_flags_compatible(source_elem.flags, target_elem.flags, relation) {
                return false;
            }
        }

        if source_tuple.element_infos.len() < target_tuple.element_infos.len() {
            for i in source_tuple.element_infos.len()..target_tuple.element_infos.len() {
                let flags = target_tuple.element_infos[i].flags;
                if !flags.contains(ElementFlags::Optional)
                    && !flags.contains(ElementFlags::Rest)
                    && !flags.contains(ElementFlags::Variadic)
                {
                    return false;
                }
            }
        }

        true
    }

    pub(crate) fn get_tuple_element_type(&self, t: &Arc<Type>, index: usize) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::Tuple(tuple) => tuple
                .element_infos
                .get(index)
                .and_then(|info| info.type_.clone()),
            _ => None,
        }
    }

    pub(crate) fn is_element_flags_compatible(
        &self,
        source: ElementFlags,
        target: ElementFlags,
        _relation: RelationKind,
    ) -> bool {
        if source.contains(ElementFlags::Required) {
            target.contains(ElementFlags::Required) || target.contains(ElementFlags::Optional)
        } else if source.contains(ElementFlags::Optional) {
            target.contains(ElementFlags::Optional)
        } else if source.contains(ElementFlags::Rest) {
            target.contains(ElementFlags::Rest)
        } else if source.contains(ElementFlags::Variadic) {
            target.contains(ElementFlags::Variadic) || target.contains(ElementFlags::Rest)
        } else {
            true
        }
    }

    pub(crate) fn is_union_or_intersection_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        if s.contains(TypeFlags::Union) {
            if relation == RelationKind::Comparable {
                return self.some_type_related_to_type(source, target, relation);
            }
            return self.each_type_related_to_type(source, target, relation);
        }

        if t.contains(TypeFlags::Union) {
            return self.type_related_to_some_type(source, target, relation);
        }

        if t.contains(TypeFlags::Intersection) {
            return self.type_related_to_each_type(source, target, relation);
        }

        if s.contains(TypeFlags::Intersection) {
            let save_len = self.relater_error_chain.len();
            let mut immediately_related = false;
            if let Some(ui) = source.as_union_or_intersection() {
                for c in &ui.types {
                    if self.is_type_related_to(c, target, relation) {
                        immediately_related = true;
                        break;
                    }
                }
            }
            self.relater_error_chain.truncate(save_len);
            if immediately_related {
                return true;
            }

            if t.contains(TypeFlags::Object) {
                return self.intersection_source_structurally_related(source, target, relation);
            }
            if t.contains(TypeFlags::TypeParameter) {
                if let Some(constraint) = self.get_constraint_of_type_parameter(target) {
                    return self.is_type_related_to(source, &constraint, relation);
                }
            }
            return false;
        }

        false
    }

    pub(crate) fn intersection_source_structurally_related(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let Some(ui) = source.as_union_or_intersection() else {
            return false;
        };
        let Some(target_struct) = target.as_structured() else {
            return false;
        };
        let mut missing_props: Vec<String> = Vec::new();
        for target_prop in &target_struct.properties {
            let found =
                self.intersection_lookup_property(&ui.types, &target_prop.name, &mut Vec::new());
            let Some(source_prop) = found else {
                if target_prop.flags.contains(SymbolFlags::Optional) {
                    continue;
                }
                missing_props.push(target_prop.name.clone());
                continue;
            };
            let source_type = self.get_type_of_symbol(&source_prop);
            let target_type = self.substituted_member_type_of(target, target_prop);
            if !self.is_type_related_to(&source_type, &target_type, relation) {
                let prop_source_str = self.type_to_string(&source_type);
                let prop_target_str = self.type_to_string(&target_type);
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![prop_source_str, prop_target_str],
                );
                self.relater_report_error(
                    crate::diagnostics::messages_generated::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                    vec![self.chain_property_arg_name(target_prop)],
                );
                return false;
            }
        }
        if !missing_props.is_empty() {
            if !self.should_report_unmatched_property_error(source, target) {
                return false;
            }
            let source_str = self.type_to_string(source);
            let target_str = self.type_to_string(target);
            if missing_props.len() == 1 {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing_props[0].clone(), source_str, target_str],
                );
            } else if missing_props.len() <= 5 {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![source_str, target_str, missing_props.join(", ")],
                );
            } else {
                self.relater_report_error(
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                    vec![
                        source_str,
                        target_str,
                        missing_props[..4].join(", "),
                        (missing_props.len() - 4).to_string(),
                    ],
                );
            }
            return false;
        }

        let target_call = target_struct.call_signatures().to_vec();
        let target_construct = target_struct.construct_signatures().to_vec();
        for (kind, target_sigs) in [
            (SignatureKind::Call, target_call),
            (SignatureKind::Construct, target_construct),
        ] {
            if target_sigs.is_empty() {
                continue;
            }
            let mut source_sigs: Vec<Arc<crate::checker::types::Signature>> = Vec::new();
            for c in &ui.types {
                if let Some(cs) = c.as_structured() {
                    let sigs = match kind {
                        SignatureKind::Call => cs.call_signatures(),
                        SignatureKind::Construct => cs.construct_signatures(),
                    };
                    source_sigs.extend(sigs.iter().cloned());
                }
            }
            if source_sigs.is_empty() {
                continue;
            }

            let mut all_matched = true;
            for t in &target_sigs {
                let mut matched = false;
                for s in &source_sigs {
                    if !self
                        .compare_signatures_related(s, t, SignatureCheckMode::empty(), relation)
                        .is_false()
                    {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    all_matched = false;
                    break;
                }
            }
            if !all_matched {
                return false;
            }
        }
        true
    }

    pub(crate) fn intersection_lookup_property(
        &mut self,
        constituents: &[Arc<Type>],
        name: &str,
        visited: &mut Vec<usize>,
    ) -> Option<Arc<crate::ast::Symbol>> {
        for c in constituents {
            if let Some(sym) = self.lookup_property_on_single_type(c, name, visited) {
                return Some(sym);
            }
        }
        None
    }
}
