#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn is_enum_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        let Some(source_symbol) = source.symbol.as_ref() else {
            return false;
        };
        let Some(target_symbol) = target.symbol.as_ref() else {
            return false;
        };

        let source_parent = if source_symbol.flags.contains(SymbolFlags::EnumMember) {
            source_symbol.parent().clone().unwrap_or_else(|| Arc::clone(source_symbol))
        } else {
            Arc::clone(source_symbol)
        };
        let target_parent = if target_symbol.flags.contains(SymbolFlags::EnumMember) {
            target_symbol.parent().clone().unwrap_or_else(|| Arc::clone(target_symbol))
        } else {
            Arc::clone(target_symbol)
        };

        if Arc::ptr_eq(&source_parent, &target_parent) {
            return true;
        }

        if source_parent.name != target_parent.name
            || !source_parent.flags.contains(SymbolFlags::RegularEnum)
            || !target_parent.flags.contains(SymbolFlags::RegularEnum)
        {
            return false;
        }

        let key = EnumRelationKey {
            source_id: source_parent.id(),
            target_id: target_parent.id(),
        };

        if let Some(entry) = self.enum_relation.get(&key).copied() {
            if entry != RelationComparisonResult::None {
                return entry.contains(RelationComparisonResult::Succeeded);
            }
        }

        let source_type = self.get_type_of_symbol(&source_parent);
        let target_type = self.get_type_of_symbol(&target_parent);
        let source_properties = self.get_properties_of_type(&source_type);

        for source_prop in source_properties {
            if !source_prop.flags.contains(SymbolFlags::EnumMember) {
                continue;
            }
            let Some(target_prop) = self.get_property_of_type(&target_type, &source_prop.name)
            else {
                self.enum_relation
                    .insert(key, RelationComparisonResult::Failed);
                return false;
            };
            if !target_prop.flags.contains(SymbolFlags::EnumMember) {
                self.enum_relation
                    .insert(key, RelationComparisonResult::Failed);
                return false;
            }

            let source_decl = self.get_declaration_of_kind(&source_prop, SyntaxKind::EnumMember);
            let target_decl = self.get_declaration_of_kind(&target_prop, SyntaxKind::EnumMember);
            if let (Some(sd), Some(td)) = (source_decl, target_decl) {
                let source_value = self.get_enum_member_value(&sd);
                let target_value = self.get_enum_member_value(&td);
                let sv = source_value.value.as_ref();
                let tv = target_value.value.as_ref();
                if sv != tv {
                    if sv.is_some() && tv.is_some() {
                        self.enum_relation
                            .insert(key, RelationComparisonResult::Failed);
                        return false;
                    }

                    let source_is_string = matches!(sv, Some(EvalValue::String(_)));
                    let target_is_string = matches!(tv, Some(EvalValue::String(_)));
                    if source_is_string || target_is_string {
                        self.enum_relation
                            .insert(key, RelationComparisonResult::Failed);
                        return false;
                    }
                }
            }
        }

        self.enum_relation
            .insert(key, RelationComparisonResult::Succeeded);
        true
    }

    pub(crate) fn is_unknown_like_union_type(&self, t: &Arc<Type>) -> bool {
        if !self.strict_null_checks || !t.flags.contains(TypeFlags::Union) {
            return false;
        }
        let Some(types) = t.types() else {
            return false;
        };
        if types.len() < 3 {
            return false;
        }
        let has_undefined = types
            .iter()
            .any(|ty| ty.flags.contains(TypeFlags::Undefined));
        let has_null = types.iter().any(|ty| ty.flags.contains(TypeFlags::Null));
        let has_empty_object = types
            .iter()
            .any(|ty| self.is_empty_anonymous_object_type(ty));
        has_undefined && has_null && has_empty_object
    }

    pub(crate) fn is_empty_anonymous_object_type(&self, t: &Arc<Type>) -> bool {
        if !t.object_flags.contains(ObjectFlags::Anonymous) {
            return false;
        }
        if t.object_flags.contains(ObjectFlags::MembersResolved) {
            return self.structured_type_is_empty(t);
        }

        if let Some(sym) = t.symbol.as_ref() {
            if sym.flags.contains(SymbolFlags::TypeLiteral) {
                return self.get_properties_of_type(t).is_empty();
            }
        }
        false
    }

    pub(crate) fn structured_type_is_empty(&self, t: &Arc<Type>) -> bool {
        self.get_properties_of_type(t).is_empty()
    }

    pub(crate) fn is_index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        source_is_primitive: bool,
    ) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        if source.flags.contains(TypeFlags::Any) {
            return true;
        }
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return true,
        };

        let target_indexes = target_struct.index_infos.clone();
        if target_indexes.is_empty() {
            return true;
        }
        let target_has_string_index = target_indexes.iter().any(|info| {
            info.key_type
                .as_ref()
                .is_some_and(|k| k.flags.contains(TypeFlags::String))
        });

        for target_index in target_indexes {
            let target_key_is_string = target_index
                .key_type
                .as_ref()
                .is_some_and(|k| k.flags.contains(TypeFlags::String));
            let target_value_any = target_index
                .value_type
                .as_ref()
                .is_some_and(|v| v.flags.contains(TypeFlags::Any));

            if relation != RelationKind::StrictSubtype
                && !source_is_primitive
                && target_has_string_index
                && target_value_any
            {
                continue;
            }
            if self.is_generic_mapped_type(source) && target_key_is_string {
                let template = self.get_template_type_from_mapped_type(source);
                match template {
                    Some(template) => {
                        let target_value = target_index
                            .value_type
                            .clone()
                            .unwrap_or_else(|| self.any_type());
                        if !self.is_type_related_to(&template, &target_value, relation) {
                            return false;
                        }
                    }
                    None => return false,
                }
                continue;
            }

            let target_key = target_index.key_type.clone().unwrap_or_else(|| self.string_type());
            match self.get_applicable_index_info(source, &target_key) {
                Some(source_info) => {
                    let source_value = source_info
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    let target_value = target_index
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    if !self.is_type_related_to(&source_value, &target_value, relation) {
                        if self.relater_chain_active {
                            let sv_str = self.type_to_string(&source_value);
                            let tv_str = self.type_to_string(&target_value);
                            self.push_relation_head_with_tp_note(
                                &source_value,
                                &target_value,
                                msg::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![sv_str, tv_str],
                            );
                            let same_key = source_info
                                .key_type
                                .as_ref()
                                .and_then(|k| target_index.key_type.as_ref().map(|t| (k, t)))
                                .is_some_and(|(k, t)| {
                                    Arc::ptr_eq(k, t) || k.flags == t.flags
                                });
                            if same_key {
                                let key_str = self.type_to_string(&target_key);
                                self.relater_report_error(
                                    msg::X_0_INDEX_SIGNATURES_ARE_INCOMPATIBLE,
                                    vec![key_str],
                                );
                            } else {
                                let sk = source_info
                                    .key_type
                                    .clone()
                                    .unwrap_or_else(|| self.string_type());
                                let sk_str = self.type_to_string(&sk);
                                let tk_str = self.type_to_string(&target_key);
                                self.relater_report_error(
                                    msg::X_0_AND_1_INDEX_SIGNATURES_ARE_INCOMPATIBLE,
                                    vec![sk_str, tk_str],
                                );
                            }
                        }
                        return false;
                    }
                }
                None => {
                    let inferable = relation != RelationKind::StrictSubtype
                        || source.object_flags.contains(ObjectFlags::FreshLiteral);
                    if inferable && self.is_object_type_with_inferable_index(source) {
                        if self.members_related_to_index_info(source, &target_index, relation)
                            .is_false()
                        {
                            return false;
                        }
                    } else {
                        if self.relater_chain_active {
                            let key_str = self.type_to_string(&target_key);
                            let source_str = self.type_to_string(source);
                            self.relater_report_error(
                                msg::INDEX_SIGNATURE_FOR_TYPE_0_IS_MISSING_IN_TYPE_1,
                                vec![key_str, source_str],
                            );
                        }
                        return false;
                    }
                }
            }
        }

        true
    }
}
