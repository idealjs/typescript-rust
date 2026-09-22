#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;
use tsox_frontend::ast::CheckFlags;

const MAX_DISCRIMINANT_COMBINATIONS: usize = 25;

impl Checker {
    pub(crate) fn get_non_missing_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let t = self.get_type_of_symbol(symbol);
        let is_optional = symbol.flags.contains(SymbolFlags::Optional);
        self.remove_missing_type(t, is_optional)
    }

    fn distributed_types(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(ui) = t.as_union_or_intersection()
        {
            return ui.types.to_vec();
        }
        vec![Arc::clone(t)]
    }

    fn discriminant_prop_type_related(
        &mut self,
        combination_type: &Arc<Type>,
        target_prop: &Arc<Symbol>,
        relation: RelationKind,
    ) -> bool {
        let target_type = self.get_non_missing_type_of_symbol(target_prop);
        if target_type
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown)
        {
            return true;
        }
        self.is_type_related_to(combination_type, &target_type, relation)
    }

    fn discriminant_target_props_related(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        excluded: &std::collections::HashSet<String>,
    ) -> bool {
        for target_prop in self.get_properties_of_type(target) {
            if excluded.contains(&target_prop.name) {
                continue;
            }
            let Some(source_prop) = self.get_property_of_type(source, &target_prop.name) else {
                if target_prop.flags.contains(SymbolFlags::Optional) {
                    continue;
                }
                return false;
            };
            let source_type = self.substituted_member_type_of(source, &source_prop);
            let source_type = self.erase_bare_generic_params(source, &source_type);
            let target_type = self.substituted_member_type_of(target, &target_prop);
            let target_type = self.erase_bare_generic_params(target, &target_type);
            if !self.is_type_related_to(&source_type, &target_type, relation) {
                return false;
            }
        }
        if !self.is_call_signatures_related_to(source, target, relation) {
            return false;
        }
        if !self.is_construct_signatures_related_to(source, target, relation) {
            return false;
        }
        if !(self.is_tuple_type(source) && self.is_tuple_type(target))
            && !self.is_index_signatures_related_to(source, target, relation, false)
        {
            return false;
        }
        true
    }

    pub(crate) fn type_related_to_discriminated_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        // Go typeRelatedToDiscriminatedType：全部子探针 reportErrors=false
        //（"We do not report errors here"），错误统一由 union 成员检查给出
        let was_active = self.silence_relation_chain();
        let result = self.type_related_to_discriminated_type_impl(source, target, relation);
        self.restore_relation_chain(was_active);
        result
    }

    fn type_related_to_discriminated_type_impl(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_properties = self.get_properties_of_type(source);
        let mut discriminant_props: Vec<Arc<Symbol>> = Vec::new();
        for prop in &source_properties {
            if self.is_discriminant_property_of_union_members(target, &prop.name) {
                discriminant_props.push(Arc::clone(prop));
            }
        }
        if discriminant_props.is_empty() {
            return false;
        }

        let mut num_combinations: usize = 1;
        let mut source_discriminant_types: Vec<Vec<Arc<Type>>> =
            Vec::with_capacity(discriminant_props.len());
        let mut excluded: std::collections::HashSet<String> = std::collections::HashSet::new();
        for prop in &discriminant_props {
            let prop_type = self.get_non_missing_type_of_symbol(prop);
            let types = self.distributed_types(&prop_type);
            num_combinations = num_combinations.saturating_mul(types.len());
            if num_combinations > MAX_DISCRIMINANT_COMBINATIONS || types.is_empty() {
                return false;
            }
            source_discriminant_types.push(types);
            excluded.insert(prop.name.clone());
        }

        let target_types: Vec<Arc<Type>> = target
            .as_union_or_intersection()
            .map(|ui| {
                ui.types
                    .iter()
                    .filter(|m| m.flags.intersects(TypeFlags::Object | TypeFlags::Intersection))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        if target_types.len() < 2 {
            return false;
        }

        let mut matching_types: Vec<Arc<Type>> = Vec::new();
        for combination_index in 0..num_combinations {
            let mut n = combination_index;
            let mut combination: Vec<Arc<Type>> = Vec::with_capacity(source_discriminant_types.len());
            for types in source_discriminant_types.iter().rev() {
                combination.push(Arc::clone(&types[n % types.len()]));
                n /= types.len();
            }
            combination.reverse();

            let mut has_match = false;
            'outer: for t in &target_types {
                for (i, source_prop) in discriminant_props.iter().enumerate() {
                    let Some(target_prop) = self.get_property_of_type(t, &source_prop.name) else {
                        continue 'outer;
                    };
                    if Arc::ptr_eq(source_prop, &target_prop) {
                        continue;
                    }
                    if !self.discriminant_prop_type_related(&combination[i], &target_prop, relation)
                    {
                        continue 'outer;
                    }
                }
                if !matching_types.iter().any(|m| Arc::ptr_eq(m, t)) {
                    matching_types.push(Arc::clone(t));
                }
                has_match = true;
            }
            if !has_match {
                return false;
            }
        }

        for t in &matching_types {
            if !self.discriminant_target_props_related(source, t, relation, &excluded) {
                return false;
            }
        }
        true
    }
}
