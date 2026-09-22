#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn find_matching_discriminant_constituent(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if !source.flags.contains(TypeFlags::Object) {
            return None;
        }
        let ui = target.as_union_or_intersection()?;
        if ui.types.len() < 2
            || !ui
                .types
                .iter()
                .all(|t| t.flags.contains(TypeFlags::Object))
        {
            return None;
        }
        for prop in self.get_properties_of_type(source) {
            let prop_type = self.get_type_of_symbol(&prop);
            if !crate::checker::utilities::is_unit_type(&prop_type) {
                continue;
            }
            let regular_prop = self.get_regular_type_of_literal_type(&prop_type);
            let mut matching = Vec::new();
            let mut distinct_seen = false;
            for t in &ui.types {
                let Some(cprop) = self.get_property_of_type(t, &prop.name) else {
                    matching.clear();
                    break;
                };
                let cprop_type = self.get_type_of_symbol(&cprop);
                if !crate::checker::utilities::is_unit_type(&cprop_type) {
                    matching.clear();
                    break;
                }
                let regular_cprop = self.get_regular_type_of_literal_type(&cprop_type);
                if regular_cprop.id == regular_prop.id {
                    matching.push(Arc::clone(t));
                } else {
                    distinct_seen = true;
                }
            }
            if distinct_seen && matching.len() == 1 {
                return matching.into_iter().next();
            }
        }
        None
    }

    pub(crate) fn get_best_matching_type_for_error(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let ui = target.as_union_or_intersection()?;

        if let Some(t) = self.find_matching_discriminant_constituent(source, target) {
            return Some(t);
        }

        if source
            .object_flags
            .intersects(ObjectFlags::Reference | ObjectFlags::Anonymous)
        {
            for t in &ui.types {
                if !t.flags.contains(TypeFlags::Object) {
                    continue;
                }
                let overlap = source.object_flags & t.object_flags;
                if overlap.contains(ObjectFlags::Reference)
                    && source
                        .target()
                        .zip(t.target())
                        .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                {
                    return Some(Arc::clone(t));
                }
                if overlap.contains(ObjectFlags::Anonymous)
                    && source
                        .alias
                        .as_ref()
                        .and_then(|a| a.symbol.as_ref())
                        .zip(t.alias.as_ref().and_then(|a| a.symbol.as_ref()))
                        .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                {
                    return Some(Arc::clone(t));
                }
            }
        }

        if source.object_flags.contains(ObjectFlags::ObjectLiteral)
            && ui.types.iter().any(|t| self.is_array_like_type(t))
        {
            if let Some(t) = ui.types.iter().find(|t| !self.is_array_like_type(t)) {
                return Some(Arc::clone(t));
            }
        }

        if let Some(s) = source.as_structured() {
            for kind in [false, true] {
                let has = if kind {
                    !s.construct_signatures().is_empty()
                } else {
                    !s.call_signatures().is_empty()
                };
                if has
                    && let Some(t) = ui.types.iter().find(|t| {
                        t.as_structured().is_some_and(|ts| {
                            if kind {
                                !ts.construct_signatures().is_empty()
                            } else {
                                !ts.call_signatures().is_empty()
                            }
                        })
                    })
                {
                    return Some(Arc::clone(t));
                }
            }
        }

        self.find_most_overlappy_type(source, target)
    }

    pub(crate) fn type_related_to_each_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = target.as_union_or_intersection() {
            self.relater_intersection_target_depth += 1;
            let result = (|| {
                for t in &ui.types {
                    if !self.is_type_related_to(source, t, relation) {
                        return false;
                    }
                }
                true
            })();
            self.relater_intersection_target_depth -= 1;
            return result;
        }
        false
    }
}
