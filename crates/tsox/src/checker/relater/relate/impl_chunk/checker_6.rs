#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn lookup_property_on_single_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        visited: &mut Vec<usize>,
    ) -> Option<Arc<crate::ast::Symbol>> {
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
                let mut first: Option<Arc<crate::ast::Symbol>> = None;
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
                if self.relater_chain_active && self.speculation_depth == 0 {
                    self.relater_error_chain.truncate(save_len);
                    if let Some(t) = failed_nullish {
                        let member_str = self.type_to_string(&t);
                        let target_str = self.type_to_string(target);
                        self.relater_report_error(
                            crate::diagnostics::messages_generated::
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec![member_str, target_str],
                        );
                    } else if let Some(t) = first_failed {
                        self.is_type_related_to(&t, target, relation);
                        let target_str = self.type_to_string(target);

                        let head_source = if !self.type_could_have_top_level_singleton_types(target)
                            && (crate::checker::is_fresh_literal_type(&t)
                                || t.flags.intersects(TYPE_FLAGS_LITERAL))
                        {
                            let base = self.get_base_type_of_literal_type_for_display(&t);
                            self.type_to_string(&base)
                        } else {
                            self.type_to_string(&t)
                        };

                        let mut suppress = false;
                        if let Some(entry) = self.relater_error_chain.last() {
                            let m = entry.message;
                            let a = &entry.args;
                            suppress = if m
                                == crate::diagnostics::messages_generated::
                                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2
                            {
                                a.len() == 3 && a[1] == head_source && a[2] == target_str
                            } else if m
                                == crate::diagnostics::messages_generated::
                                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2
                                || m
                                    == crate::diagnostics::messages_generated::
                                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE
                            {
                                a.len() >= 2 && a[0] == head_source && a[1] == target_str
                            } else if m
                                == crate::diagnostics::messages_generated::
                                    THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1
                            {
                                a.len() == 2 && a[0] == head_source && a[1] == target_str
                            } else {
                                false
                            };
                        }
                        if !suppress {
                            let msg = if head_source == target_str {
                                crate::diagnostics::messages_generated::
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
                            } else {
                                crate::diagnostics::messages_generated::
                                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
                            };
                            self.relater_report_error(msg, vec![head_source, target_str]);
                        }
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
        if let Some(ui) = target.as_union_or_intersection() {
            let save_len = self.relater_error_chain.len();
            let mut best: Option<Vec<RelaterChainEntry>> = None;
            for t in &ui.types {
                if self.is_type_related_to(source, t, relation) {
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

            if source.flags.contains(TypeFlags::Intersection)
                && let Some(si) = source.as_union_or_intersection()
            {
                self.relater_error_chain.truncate(save_len);
                for s in &si.types {
                    if self.is_type_related_to(s, target, relation) {
                        return true;
                    }
                }
                self.relater_error_chain.truncate(save_len);
            }
            if let Some(b) = best {
                self.relater_error_chain = b;
            }

            if self.relater_chain_active
                && self.speculation_depth == 0
                && let Some(best_t) = self.get_best_matching_type_for_error(source, target)
            {
                self.relater_error_chain.truncate(save_len);
                self.is_type_related_to(source, &best_t, relation);
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(&best_t);
                let msg = if source_str == target_str {
                    crate::diagnostics::messages_generated::
                        TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
                } else {
                    crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
                };
                self.relater_report_error(msg, vec![source_str, target_str]);
            }
        }
        false
    }
}
