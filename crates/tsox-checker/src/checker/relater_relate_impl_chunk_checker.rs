#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub fn is_type_identical_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
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
        }
        let simplifiable = TYPE_FLAGS_UNION_OR_INTERSECTION
            | TypeFlags::from_bits_truncate(
                TypeFlags::IndexedAccess.bits()
                    | TypeFlags::Conditional.bits()
                    | TypeFlags::Substitution.bits(),
            );
        if !(source.flags | target.flags).intersects(simplifiable) {
            if source.flags != target.flags {
                return false;
            }
            if source.flags.contains(TYPE_FLAGS_SINGLETON) {
                return true;
            }
            if !source
                .flags
                .intersects(TYPE_FLAGS_STRUCTURED_OR_INSTANTIABLE)
                && !target
                    .flags
                    .intersects(TYPE_FLAGS_STRUCTURED_OR_INSTANTIABLE)
            {
                return match (&source.data, &target.data) {
                    (TypeData::Intrinsic(s), TypeData::Intrinsic(t)) => {
                        s.intrinsic_name == t.intrinsic_name
                    }
                    (TypeData::Literal(s), TypeData::Literal(t)) => {
                        s.value == t.value
                            && match (&source.symbol, &target.symbol) {
                                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                                (None, None) => true,
                                _ => false,
                            }
                    }
                    _ => false,
                };
            }
        }
        if source
            .flags
            .intersects(TYPE_FLAGS_STRUCTURED_OR_INSTANTIABLE)
            || target
                .flags
                .intersects(TYPE_FLAGS_STRUCTURED_OR_INSTANTIABLE)
        {
            self.is_type_related_to(&source, &target, RelationKind::Identity)
        } else {
            false
        }
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

    // 空成员且带接口/类符号：解析重入期返回的未完成实例（非 `{}` 字面量）
    pub(crate) fn side_is_incomplete_shell(&self, t: &Arc<Type>, _id: u32) -> bool {
        t.as_structured().is_some_and(|s| {
            s.members.entries.is_empty()
                && t.symbol
                    .as_ref()
                    .is_some_and(|sym| {
                        sym.flags.intersects(
                            tsox_frontend::ast::SymbolFlags::Interface
                                | tsox_frontend::ast::SymbolFlags::Class,
                        ) && !sym.declarations.is_empty()
                    })
        })
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

        // Substitution（NoInfer 包装）关系期按 base 展开（Go substitution
        // 类型的 constraint 仅推断期参与，关系判定由 base 承担）
        let source = substitution_base_or_self(&source);
        let target = substitution_base_or_self(&target);

        let source = self.get_simplified_type_for_relation(&source, false);
        let target = self.get_simplified_type_for_relation(&target, true);

        if Arc::ptr_eq(&source, &target) {
            return true;
        }

        // Go isRelatedToEx：definitely non-nullable 源对「可空成分 + 单个非
        // nullable 成员」的 union 目标，先剔除可空成分再进入比较；identity
        // 关系在 Go 中于重绑定前提前返回，不参与
        let target = if relation == RelationKind::Identity {
            target
        } else {
            match self.rebind_non_nullable_union_target(&source, &target) {
                Some(candidate) => candidate,
                None => target,
            }
        };

        {
            let sp = source.id;
            let tp = target.id;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                // 任一方是「空成员且带符号」的壳型（lib 解析重入期的未完成实例）
                // 即放行：带符号才免于误放 `{}` 字面量；有成员的完整实例照常
                // 结构比较（递归由 relation_in_progress 兜底）。仅解析重入
                // 真在进行时（pending 壳未清）生效，常态检查期的惰性空壳
                // 引用（如未水化的 IPromise<unknown>）走正常结构比较
                && !self.pending_interface_shells.is_empty()
                && (self.side_is_incomplete_shell(&source, sp)
                    || self.side_is_incomplete_shell(&target, tp))
            {
                return true;
            }
        }

        // 循环引用中的接口壳（声明正在解析、成员未就绪）：比较无意义，放行
        if source.flags.contains(TypeFlags::Object)
            && source
                .symbol
                .as_ref()
                .is_some_and(|s| {
                    self.pending_interface_shells
                        .contains_key(&(Arc::as_ptr(s) as *const tsox_frontend::ast::Symbol as usize))
                })
            && source.as_structured().is_some_and(|s| s.members.entries.is_empty())
        {
            return true;
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
            intersection_target: self.relater_intersection_target_depth > 0,
        };

        if crate::checker::is_object_literal_type(&source)
            && source.object_flags.contains(ObjectFlags::FreshLiteral)
            && self.relater_intersection_target_depth == 0
            && self.has_excess_properties(&source, &target, relation)
        {
            return false;
        }

        if self.relation_in_progress.contains(&key) {
            return true;
        }

        if let Some(&cached) = self.relation_cache.get(&key) {
            if cached || !self.relater_chain_active {
                return cached;
            }
        }
        let is_top_level = self.relater_depth == 0;
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
        {
            result = self.conditional_fallback_related(&source, &target, relation);
        }
        if !result && self.relater_chain_active && !is_top_level {
            self.report_nested_relation_failure(&source, &target, relation);
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

fn substitution_base_or_self(t: &Arc<Type>) -> Arc<Type> {
    if t.flags.contains(TypeFlags::Substitution)
        && let TypeData::Substitution(sub) = &t.data
        && let Some(base) = &sub.base_type
    {
        return Arc::clone(base);
    }
    Arc::clone(t)
}
