#![allow(unused_imports)]

use super::*;

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
            source_symbol.parent.as_ref().unwrap_or(source_symbol)
        } else {
            source_symbol
        };
        let target_parent = if target_symbol.flags.contains(SymbolFlags::EnumMember) {
            target_symbol.parent.as_ref().unwrap_or(target_symbol)
        } else {
            target_symbol
        };

        if Arc::ptr_eq(source_parent, target_parent) {
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

        let source_type = self.get_type_of_symbol(source_parent);
        let target_type = self.get_type_of_symbol(target_parent);
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
    ) -> bool {
        if source.flags.contains(TypeFlags::Any) {
            return true;
        }
        let source_struct = match source.as_structured() {
            Some(s) => s,
            None => return false,
        };
        let target_struct = match target.as_structured() {
            Some(t) => t,
            None => return false,
        };

        let source_indexes = &source_struct.index_infos;
        let target_indexes = &target_struct.index_infos;

        if target_indexes.is_empty() {
            return true;
        }

        for target_index in target_indexes {
            let target_key = &target_index.key_type;
            let target_value = &target_index.value_type;

            let mut found_match = false;
            for source_index in source_indexes {
                let source_key = &source_index.key_type;
                let source_value = &source_index.value_type;

                let key_match = match (target_key, source_key) {
                    (Some(tk), Some(sk)) => self.is_type_related_to(sk, tk, relation),
                    (None, _) => true,
                    (_, None) => false,
                };

                if !key_match {
                    continue;
                }

                let value_match = match (target_value, source_value) {
                    (Some(tv), Some(sv)) => self.is_type_related_to(sv, tv, relation),

                    (None, _) => true,
                    (_, None) => false,
                };

                if value_match {
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                let result = self.members_related_to_index_info(source, target_index, relation);
                if result.is_false() {
                    let key_str = target_key
                        .as_ref()
                        .map(|k| self.type_to_string(k))
                        .unwrap_or_else(|| "string".to_string());
                    let source_str = self.type_to_string(source);
                    self.relater_report_error(
                        crate::diagnostics::messages_generated::
                            INDEX_SIGNATURE_FOR_TYPE_0_IS_MISSING_IN_TYPE_1,
                        vec![key_str, source_str],
                    );
                    return false;
                }
            }
        }

        true
    }
}
