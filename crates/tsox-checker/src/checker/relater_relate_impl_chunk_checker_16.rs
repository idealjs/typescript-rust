#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn properties_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        if !source.flags.contains(TypeFlags::Object)
            || !target.flags.contains(TypeFlags::Object)
        {
            return false;
        }
        let source_properties = self.get_properties_of_type(source);
        let target_properties = self.get_properties_of_type(target);
        if source_properties.len() != target_properties.len() {
            return false;
        }
        for source_prop in &source_properties {
            let Some(target_prop) = self.get_property_of_type(target, &source_prop.name) else {
                return false;
            };
            if !self.compare_properties_identical(source, target, source_prop, &target_prop) {
                return false;
            }
        }
        true
    }

    pub(crate) fn compare_properties_identical(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        source_prop: &Arc<Symbol>,
        target_prop: &Arc<Symbol>,
    ) -> bool {
        if Arc::ptr_eq(source_prop, target_prop) {
            return true;
        }
        let non_public = tsox_frontend::ast::ModifierFlags::NonPublicAccessibilityModifier;
        let source_accessibility =
            crate::checker::exports::get_declaration_modifier_flags_from_symbol(source_prop)
                .intersection(non_public);
        let target_accessibility =
            crate::checker::exports::get_declaration_modifier_flags_from_symbol(target_prop)
                .intersection(non_public);
        if source_accessibility != target_accessibility {
            return false;
        }
        if !source_accessibility.is_empty() {
            let source_target = self.get_target_symbol(source_prop);
            let target_target = self.get_target_symbol(target_prop);
            if !Arc::ptr_eq(&source_target, &target_target)
                && !symbol_declarations_overlap(&source_target, &target_target)
            {
                return false;
            }
        } else if source_prop.flags.contains(SymbolFlags::Optional)
            != target_prop.flags.contains(SymbolFlags::Optional)
        {
            return false;
        }
        if self.is_readonly_symbol_for_identity(source_prop)
            != self.is_readonly_symbol_for_identity(target_prop)
        {
            return false;
        }
        let source_type = self.substituted_member_type_of(source, source_prop);
        let target_type = self.substituted_member_type_of(target, target_prop);
        self.is_type_related_to(&source_type, &target_type, RelationKind::Identity)
    }

    fn get_target_symbol(&self, s: &Arc<Symbol>) -> Arc<Symbol> {
        if s.check_flags
            .contains(tsox_frontend::ast::CheckFlags::Instantiated)
            && let Some(links) = self.value_symbol_links.get(s)
            && let Some(target) = &links.target
        {
            return Arc::clone(target);
        }
        Arc::clone(s)
    }

    pub(crate) fn is_readonly_symbol_for_identity(&self, symbol: &Arc<Symbol>) -> bool {
        symbol
            .check_flags
            .contains(tsox_frontend::ast::CheckFlags::Readonly)
            || (symbol.flags.contains(SymbolFlags::Property)
                && crate::checker::exports::get_declaration_modifier_flags_from_symbol(symbol)
                    .contains(tsox_frontend::ast::ModifierFlags::Readonly))
            || symbol.flags.contains(SymbolFlags::EnumMember)
            || (symbol.flags.contains(SymbolFlags::GetAccessor)
                && !symbol.flags.contains(SymbolFlags::SetAccessor))
    }
}

fn symbol_declarations_overlap(a: &Arc<Symbol>, b: &Arc<Symbol>) -> bool {
    a.declarations
        .iter()
        .any(|d| b.declarations.iter().any(|e| Arc::ptr_eq(d, e)))
}
