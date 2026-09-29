#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn lookup_property_on_single_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        visited: &mut Vec<usize>,
    ) -> Option<Arc<tsox_frontend::ast::Symbol>> {
        let ptr = Arc::as_ptr(t) as usize;
        if visited.contains(&ptr) {
            return None;
        }
        visited.push(ptr);
        if t.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.lookup_property_on_single_type(&constraint, name, visited);
        }
        if let Some(ui) = t.as_union_or_intersection() {
            if t.flags.contains(TypeFlags::Union) {
                let mut first: Option<Arc<tsox_frontend::ast::Symbol>> = None;
                for c in &ui.types {
                    match self.lookup_property_on_single_type(c, name, visited) {
                        Some(sym) => {
                            if first.is_none() {
                                first = Some(sym);
                            }
                        }
                        None => return None,
                    }
                }
                return first;
            }
            for c in &ui.types {
                if let Some(sym) = self.lookup_property_on_single_type(c, name, visited) {
                    return Some(sym);
                }
            }
            return None;
        }
        if let Some(st) = t.as_structured() {
            if let Some(p) = st.members.get(name) {
                return Some(Arc::clone(p));
            }
            return None;
        }
        if self.is_array_type(t) {
            return self.declared_array_member_symbol(name);
        }
        None
    }

    pub(crate) fn each_type_related_to_some_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if source.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            && let Some(si) = source.as_union_or_intersection()
        {
            for s in &si.types {
                if !self.type_related_to_some_type(s, target, relation) {
                    return false;
                }
            }
            return true;
        }
        self.type_related_to_some_type(source, target, relation)
    }

    pub(crate) fn some_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            let save_len = self.relater_error_chain.len();
            let mut best: Option<Vec<RelaterChainEntry>> = None;
            for t in &ui.types {
                if self.is_type_related_to(t, target, relation) {
                    return true;
                }
                if best
                    .as_ref()
                    .is_none_or(|b| b.len() < self.relater_error_chain.len())
                {
                    best = Some(self.relater_error_chain.clone());
                }
                self.relater_error_chain.truncate(save_len);
            }
            if let Some(b) = best {
                self.relater_error_chain = b;
            }
        }
        false
    }

    pub(crate) fn each_type_related_to_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if let Some(ui) = source.as_union_or_intersection() {
            let save_len = self.relater_error_chain.len();
            let mut any_failed = false;
            let mut failed_nullish: Option<Arc<Type>> = None;
            let mut first_failed: Option<Arc<Type>> = None;
            for t in &ui.types {
                if !self.is_type_related_to(t, target, relation) {
                    any_failed = true;
                    if first_failed.is_none() {
                        first_failed = Some(Arc::clone(t));
                    }
                    if t.flags.contains(TypeFlags::Undefined) {
                        if failed_nullish
                            .as_ref()
                            .is_none_or(|f| f.flags.contains(TypeFlags::Null))
                        {
                            failed_nullish = Some(Arc::clone(t));
                        }
                    } else if t.flags.contains(TypeFlags::Null) && failed_nullish.is_none() {
                        failed_nullish = Some(Arc::clone(t));
                    }
                }
            }
            if any_failed {
                if self.relater_chain_active {
                    self.relater_error_chain.truncate(save_len);
                    if let Some(t) = failed_nullish {
                        let member_str = self.type_to_string(&t);
                        let target_str = self.type_to_string(target);
                        self.relater_report_error(
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec![member_str, target_str],
                        );
                    } else if let Some(t) = first_failed {
                        let _ = self.is_type_related_to(&t, target, relation);
                    }
                }
                return false;
            }
            return true;
        }
        false
    }

    pub(crate) fn type_related_to_some_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source = self.get_regular_type_of_object_literal(source);
        if let Some(ui) = target.as_union_or_intersection() {
            let save_len = self.relater_error_chain.len();
            let matched = {
                let was_active = self.silence_relation_chain();
                let mut m = false;
                for t in &ui.types {
                    if self.is_type_related_to(&source, t, relation) {
                        m = true;
                        break;
                    }
                }
                self.restore_relation_chain(was_active);
                self.relater_error_chain.truncate(save_len);
                m
            };
            if matched {
                return true;
            }

            if source.flags.contains(TypeFlags::Intersection)
                && let Some(si) = source.as_union_or_intersection()
            {
                let was_active = self.silence_relation_chain();
                let mut any = false;
                for s in &si.types {
                    if self.is_type_related_to(s, target, relation) {
                        any = true;
                        break;
                    }
                }
                self.restore_relation_chain(was_active);
                if any {
                    return true;
                }
                self.relater_error_chain.truncate(save_len);
            }

            if self.relater_chain_active
                && self.speculation_depth == 0
                && !source
                    .flags
                    .intersects(crate::checker::types_type_id::TYPE_FLAGS_PRIMITIVE)
                && !target
                    .flags
                    .intersects(crate::checker::types_type_id::TYPE_FLAGS_PRIMITIVE)
                && let Some(best_t) = self.get_best_matching_type_for_error(&source, target)
            {
                self.relater_error_chain.truncate(save_len);
                self.is_type_related_to(&source, &best_t, relation);
            }
        }
        false
    }
}
