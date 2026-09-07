#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_object_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        if relation != RelationKind::Comparable
            && self.relater_intersection_target_depth == 0
            && !source_struct.properties.is_empty()
            && self.is_weak_type(target)
            && !self.has_common_properties(source, target, false)
        {
            let has_calls = !source_struct.call_signatures().is_empty();
            let has_constructs = !source_struct.construct_signatures().is_empty();
            if self.relater_chain_active {
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(target);
                if has_calls || has_constructs {
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            VALUE_OF_TYPE_0_HAS_NO_PROPERTIES_IN_COMMON_WITH_TYPE_1_DID_YOU_MEAN_TO_CALL_IT,
                        vec![source_str, target_str],
                    );
                } else {
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            TYPE_0_HAS_NO_PROPERTIES_IN_COMMON_WITH_TYPE_1,
                        vec![source_str, target_str],
                    );
                }
            }
            return false;
        }

        if self.is_array_type(target)
            && target_struct.properties.is_empty()
            && !self.is_array_type(source)
            && !self.is_tuple_type(source)
            && !source.object_flags.contains(ObjectFlags::EvolvingArray)
        {
            let mut missing: Vec<String> = Vec::new();
            for prop in self.declared_array_member_symbols() {
                if prop.flags.contains(SymbolFlags::Optional) {
                    continue;
                }
                let found = source_struct.members.get(&prop.name).is_some()
                    || (!source_struct.call_signatures().is_empty()
                        && self
                            .global_interface_member_symbol("Function", &prop.name)
                            .is_some())
                    || self
                        .global_interface_member_symbol("Object", &prop.name)
                        .is_some();
                if !found {
                    missing.push(prop.name.clone());
                }
            }
            if !missing.is_empty() {
                if self.should_report_unmatched_property_error(source, target) {
                    let source_str = self.type_to_string(source);
                    let target_str = self.type_to_string(target);
                    if missing.len() == 1 {
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                            vec![missing[0].clone(), source_str, target_str],
                        );
                    } else if missing.len() <= 5 {
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                            vec![source_str, target_str, missing.join(", ")],
                        );
                    } else {
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                            vec![
                                source_str,
                                target_str,
                                missing[..4].join(", "),
                                (missing.len() - 4).to_string(),
                            ],
                        );
                    }
                }
                return false;
            }
        }

        let mut missing_props: Vec<String> = Vec::new();

        let source_is_bare_array = (self.is_array_type(source)
            || source.object_flags.contains(ObjectFlags::EvolvingArray))
            && source_struct.members.is_empty();
        for target_prop in &target_struct.properties {
            let source_declares_locally = source_struct.members.get(&target_prop.name).is_some();
            let source_prop = match source_struct.members.get(&target_prop.name) {
                Some(p) => Arc::clone(p),
                None => {
                    if source_is_bare_array
                        && let Some(p) = self.declared_array_member_symbol(&target_prop.name)
                    {
                        p
                    } else {
                        if target_prop.flags.contains(SymbolFlags::Optional) {
                            continue;
                        }
                        missing_props.push(target_prop.name.clone());
                        continue;
                    }
                }
            };

            if target_prop.name.starts_with('[')
                || (!source_declares_locally
                    && self
                        .global_interface_member_symbol("Object", &target_prop.name)
                        .is_some())
            {
                continue;
            }

            {
                let src_mod = crate::checker::exports::get_declaration_modifier_flags_from_symbol(
                    &source_prop,
                );
                let tgt_mod = crate::checker::exports::get_declaration_modifier_flags_from_symbol(
                    target_prop,
                );
                if src_mod.intersects(ModifierFlags::Private)
                    || tgt_mod.intersects(ModifierFlags::Private)
                {
                    let decl_of = |s: &Arc<crate::ast::Symbol>| {
                        s.value_declaration
                            .clone()
                            .or_else(|| s.declarations.first().cloned())
                    };
                    let same_decl = Arc::ptr_eq(&source_prop, target_prop)
                        || match (decl_of(&source_prop), decl_of(target_prop)) {
                            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
                            _ => false,
                        };
                    if !same_decl {
                        if src_mod.intersects(ModifierFlags::Private)
                            && tgt_mod.intersects(ModifierFlags::Private)
                        {
                            self.relater_report_error(
                                crate::diagnostics::messages_generated::
                                    TYPES_HAVE_SEPARATE_DECLARATIONS_OF_A_PRIVATE_PROPERTY_0,
                                vec![target_prop.name.clone()],
                            );
                        } else {
                            let private_side = if src_mod.intersects(ModifierFlags::Private) {
                                self.type_to_string(source)
                            } else {
                                self.type_to_string(target)
                            };
                            let public_side = if src_mod.intersects(ModifierFlags::Private) {
                                self.type_to_string(target)
                            } else {
                                self.type_to_string(source)
                            };
                            self.relater_report_error(
                                crate::diagnostics::messages_generated::
                                    PROPERTY_0_IS_PRIVATE_IN_TYPE_1_BUT_NOT_IN_TYPE_2,
                                vec![target_prop.name.clone(), private_side, public_side],
                            );
                        }
                        return false;
                    }
                } else if src_mod.intersects(ModifierFlags::Protected)
                    && !tgt_mod.intersects(ModifierFlags::Protected)
                {
                    let src_str = self.type_to_string(source);
                    let tgt_str = self.type_to_string(target);
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            PROPERTY_0_IS_PROTECTED_IN_TYPE_1_BUT_PUBLIC_IN_TYPE_2,
                        vec![target_prop.name.clone(), src_str, tgt_str],
                    );
                    return false;
                }
            }

            let source_type = if source_is_bare_array {
                self.instantiate_array_member_type(source, &source_prop)
                    .unwrap_or_else(|| self.get_type_of_symbol(&source_prop))
            } else {
                self.substituted_member_type_of(source, &source_prop)
            };

            let source_type = self.erase_bare_generic_params(source, &source_type);
            let target_type = self.substituted_member_type_of(target, target_prop);
            let target_type = self.erase_bare_generic_params(target, &target_type);
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
                    vec![
                        source_str,
                        target_str,
                        missing_props.join(", "),
                    ],
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

        if self.is_tuple_type(target)
            && !self.is_array_type(source)
            && source.object_flags.contains(ObjectFlags::EvolvingArray) == false
            && let TypeData::Tuple(tup) = &target.data
        {
            for (i, ei) in tup.element_infos.iter().enumerate() {
                let Some(elem_type) = &ei.type_ else { continue };
                let name = i.to_string();
                let Some(source_prop) = source_struct.members.get(&name) else {
                    let optional = ei.flags.contains(ElementFlags::Optional);
                    if optional {
                        continue;
                    }
                    return false;
                };
                let source_type = self.get_type_of_symbol(source_prop);
                if !self.is_type_related_to(&source_type, elem_type, relation) {
                    return false;
                }
            }
        }

        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }

        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }

        if !self.is_index_signatures_related_to(source, target, relation) {
            return false;
        }

        true
    }
}
