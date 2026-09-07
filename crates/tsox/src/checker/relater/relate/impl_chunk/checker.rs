#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn is_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }

        if source.flags != target.flags {
            return false;
        }
        if source.flags.contains(TYPE_FLAGS_SINGLETON) {
            return true;
        }
        self.is_simple_type_identical_to(source, target)
    }

    pub fn is_type_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Assignable)
    }

    pub fn is_type_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Subtype)
    }

    pub fn is_type_strict_subtype_of(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::StrictSubtype)
    }

    pub fn is_type_comparable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if Arc::ptr_eq(source, target) {
            return true;
        }
        self.is_type_related_to(source, target, RelationKind::Comparable)
    }

    pub fn are_types_comparable(&mut self, type1: &Arc<Type>, type2: &Arc<Type>) -> bool {
        self.is_type_comparable_to(type1, type2) || self.is_type_comparable_to(type2, type1)
    }

    pub(crate) fn is_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source = if crate::checker::is_fresh_literal_type(source) {
            self.get_regular_type_of_literal_type(source)
        } else {
            Arc::clone(source)
        };
        let target = if crate::checker::is_fresh_literal_type(target) {
            self.get_regular_type_of_literal_type(target)
        } else {
            Arc::clone(target)
        };

        if Arc::ptr_eq(&source, &target) {
            return true;
        };

        {
            let sp = source.id;
            let tp = target.id;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                && (self.degraded_type_ptrs.contains(&sp) || self.degraded_type_ptrs.contains(&tp))
            {
                return true;
            }
        }

        if !source.flags.intersects(
            TypeFlags::Object
                | TypeFlags::Union
                | TypeFlags::Intersection
                | TypeFlags::TypeParameter
                | TypeFlags::Any
                | TypeFlags::Unknown,
        ) && target.flags.contains(TypeFlags::Object)
            && target
                .as_structured()
                .is_some_and(|t| !t.index_infos.is_empty())
            && target.symbol.is_none()
        {
            if source
                .flags
                .intersects(TypeFlags::String | TypeFlags::StringLiteral | TypeFlags::StringMapping)
                && target.as_structured().is_some_and(|t| {
                    t.index_infos.iter().any(|info| {
                        info.key_type
                            .as_ref()
                            .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                    })
                })
            {
                return true;
            }
            return false;
        }

        if self.relater_overflow {
            return true;
        }
        if self.relater_depth >= RELATER_MAX_DEPTH {
            self.relater_overflow = true;
            return true;
        }

        if self.relation_count == 0 && self.relater_depth > 0 {
            self.relater_overflow = true;
            return true;
        }

        if self.relater_depth == 0 {
            self.relation_cache.clear();
            self.relation_in_progress.clear();
            self.relater_overflow = false;
            self.relater_source_stack.clear();
            self.relater_target_stack.clear();

            self.relation_count = 2_000_000;
        }
        let key = RelationCacheKey {
            source_id: source.id,
            target_id: target.id,
            relation,
        };

        if self.relation_in_progress.contains(&key) {
            return true;
        }

        if let Some(&cached) = self.relation_cache.get(&key) {
            if cached || !self.relater_chain_active {
                return cached;
            }
        }
        self.relation_in_progress.insert(key);
        self.relater_depth += 1;

        let source_deep = self.is_deeply_nested_type(&source, &self.relater_source_stack, 3);
        let target_deep = self.is_deeply_nested_type(&target, &self.relater_target_stack, 3);
        let mut result = if source_deep && target_deep {
            true
        } else {
            self.relater_source_stack.push(Arc::clone(&source));
            self.relater_target_stack.push(Arc::clone(&target));
            let r = self.is_type_related_to_inner(&source, &target, relation);
            self.relater_source_stack.pop();
            self.relater_target_stack.pop();
            r
        };
        self.relater_depth -= 1;
        self.relation_in_progress.remove(&key);

        if !result {
            self.relation_count = self.relation_count.saturating_sub(1);
        }

        if !result
            && !matches!(
                relation,
                RelationKind::Identity | RelationKind::StrictSubtype
            )
            && !self.relater_overflow
            && source.flags.contains(TypeFlags::Conditional)
        {
            let truly_deferred = match &source.data {
                TypeData::Conditional(ct) => {
                    ct.resolved_true_type.get().is_none() && ct.resolved_false_type.get().is_none()
                }
                _ => false,
            };

            if truly_deferred
                && self.deferred_constraint_depth < 100
                && let Some(constraint) = self.deferred_default_constraint_of_conditional(&source)
            {
                self.deferred_constraint_depth += 1;
                let r = self.is_type_related_to(&constraint, &target, relation);
                self.deferred_constraint_depth -= 1;
                if r {
                    result = true;
                }
            }
        }

        if !result
            && !matches!(
                relation,
                RelationKind::Identity | RelationKind::StrictSubtype
            )
            && !self.relater_overflow
            && target.flags.contains(TypeFlags::Conditional)
            && let TypeData::Conditional(tct) = &target.data
        {
            let root_ok = tct.root.as_ref().is_some_and(|r| {
                r.infer_type_parameters.is_empty() && Self::conditional_distribution_independent(r)
            });
            let source_same_root = match (
                &source.data,
                tct.root.as_ref().and_then(|r| r.node.as_ref()),
            ) {
                (TypeData::Conditional(sc), Some(node)) => sc
                    .root
                    .as_ref()
                    .and_then(|r| r.node.as_ref())
                    .map(|n| n.id() == node.id())
                    .unwrap_or(false),
                _ => false,
            };
            if root_ok
                && !source_same_root
                && let (Some(check), Some(extends)) =
                    (tct.check_type.clone(), tct.extends_type.clone())
            {
                let skip_true = {
                    let pc = self.get_permissive_instantiation(&check);
                    let pe = self.get_permissive_instantiation(&extends);
                    !self.is_type_assignable_to(&pc, &pe)
                };
                if skip_true {
                    result = true;
                } else if let Some(true_branch) =
                    self.get_forced_branch_type_of_conditional_type(&target, true)
                {
                    if self.is_type_related_to(&source, &true_branch, relation) {
                        let skip_false = {
                            let rc = self.get_restrictive_instantiation(&check);
                            let re = self.get_restrictive_instantiation(&extends);
                            self.is_type_assignable_to(&rc, &re)
                        };
                        if skip_false {
                            result = true;
                        } else if let Some(false_branch) =
                            self.get_forced_branch_type_of_conditional_type(&target, false)
                        {
                            if self.is_type_related_to(&source, &false_branch, relation) {
                                result = true;
                            }
                        }
                    }
                }
            }
        }
        self.relation_cache.insert(key, result);
        result
    }

    pub(crate) fn chain_message_key(&self, index: usize) -> Option<&'static str> {
        let len = self.relater_error_chain.len();
        if len <= index {
            return None;
        }
        Some(self.relater_error_chain[len - 1 - index].message.key)
    }

    pub(crate) fn chain_args(&self, index: usize) -> Option<&[String]> {
        let len = self.relater_error_chain.len();
        if len <= index {
            return None;
        }
        Some(&self.relater_error_chain[len - 1 - index].args)
    }
}
