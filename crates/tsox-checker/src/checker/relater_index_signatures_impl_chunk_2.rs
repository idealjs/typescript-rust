#![allow(unused_imports)]

use crate::checker::relater_index_signatures::*;

impl Checker {
    pub fn is_object_type_with_inferable_index(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Intersection) {
            if let Some(ui) = t.as_union_or_intersection() {
                return ui
                    .types
                    .iter()
                    .all(|c| self.is_object_type_with_inferable_index(c));
            }
            return false;
        }

        if let Some(sym) = &t.symbol {
            let sf = sym.flags;
            let inferable_symbol_kinds = sf.intersects(
                SymbolFlags::ObjectLiteral
                    | SymbolFlags::TypeLiteral
                    | SymbolFlags::ENUM
                    | SymbolFlags::ValueModule,
            );
            if inferable_symbol_kinds
                && !sf.contains(SymbolFlags::Class)
                && !self.type_has_call_or_construct_signatures(t)
            {
                return true;
            }
        }

        if t.object_flags
            .intersects(ObjectFlags::JSLiteral | ObjectFlags::ObjectRestType)
        {
            return true;
        }

        if t.object_flags.contains(ObjectFlags::ReverseMapped) {
            if let TypeData::ReverseMapped(rm) = &t.data {
                if let Some(src) = &rm.source {
                    return self.is_object_type_with_inferable_index(src);
                }
            }
        }
        false
    }

    pub fn members_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        use tsox_core::diagnostics::messages_generated as msg;
        let Some(target_key) = target_info.key_type.as_ref() else {
            return Ternary::True;
        };
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());

        let props = self.get_properties_of_type(source);
        let mut result = Ternary::True;
        for prop in props {
            let literal_key = self.get_literal_type_from_property(&prop);
            if !self.is_applicable_index_type(&literal_key, target_key) {
                continue;
            }
            let prop_type = self.get_type_of_symbol(&prop);
            let compared = if self.exact_optional_property_types
                || prop_type.flags.contains(TypeFlags::Undefined)
                || target_key.flags.contains(TypeFlags::Number)
                || !prop.flags.contains(SymbolFlags::Optional)
            {
                prop_type
            } else {
                self.remove_undefined_from_union(&prop_type)
            };
            let related = self.compare_types(compared, Arc::clone(&target_value), relation, false);
            if related.is_false() {
                if self.relater_chain_active {
                    let name = crate::checker::property_name_for_display(&prop.name);
                    self.relater_report_error(
                        msg::PROPERTY_0_IS_INCOMPATIBLE_WITH_INDEX_SIGNATURE,
                        vec![name],
                    );
                }
                return Ternary::False;
            }
            result = result.and(related);
        }

        for info in self.get_index_infos_of_type(source) {
            if let Some(src_key) = &info.key_type {
                if self.is_applicable_index_type(src_key, target_key) {
                    let related = self.index_info_related_to(&info, target_info, relation);
                    if related.is_false() {
                        return Ternary::False;
                    }
                    result = result.and(related);
                }
            }
        }
        result
    }

    pub fn is_applicable_index_type(&self, key: &Arc<Type>, target_key: &Arc<Type>) -> bool {
        if Arc::ptr_eq(key, target_key) {
            return true;
        }

        if key.flags.contains(TypeFlags::StringLiteral)
            && target_key.flags.contains(TypeFlags::String)
        {
            return true;
        }

        if key.flags.contains(TypeFlags::NumberLiteral)
            && target_key.flags.contains(TypeFlags::Number)
        {
            return true;
        }

        if key.flags.contains(TypeFlags::Number) && target_key.flags.contains(TypeFlags::String) {
            return true;
        }
        false
    }

    pub fn get_literal_type_from_property(&mut self, prop: &Arc<Symbol>) -> Arc<Type> {
        let non_public =
            crate::checker::exports::get_declaration_modifier_flags_from_symbol(prop)
                .intersects(tsox_frontend::ast::ModifierFlags::NonPublicAccessibilityModifier);
        if non_public {
            return self.never_type();
        }
        let t = if let Some(decl) = prop.value_declaration.as_ref() {
            tsox_frontend::ast::utilities::get_name_of_declaration(decl)
                .and_then(|name| self.get_literal_type_from_property_name(&name))
                .unwrap_or_else(|| self.get_string_literal_type(&prop.name))
        } else {
            self.get_string_literal_type(&prop.name)
        };
        if t.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::UniqueESSymbol,
        ) {
            t
        } else {
            self.never_type()
        }
    }
}
