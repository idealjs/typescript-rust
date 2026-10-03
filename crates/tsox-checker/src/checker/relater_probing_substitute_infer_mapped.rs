#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    /// Go instantiateIndexType：keyof 替换。目标仍泛型保持惰性 Index，
    /// 已具体则立即归约为 keyof 结果
    pub(crate) fn substitute_infer_index(
        &mut self,
        t: &Arc<Type>,
        idx: &crate::checker::types::IndexTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_index"); 
        let Some(old_target) = idx.target.clone() else {
            return Arc::clone(t);
        };
        let new_target = self.substitute_infer_type_parameters(&old_target, params, substitutions);
        if Arc::ptr_eq(&new_target, &old_target) {
            return Arc::clone(t);
        }
        let still_generic = new_target.flags.intersects(
            TypeFlags::TypeParameter
                | TypeFlags::Index
                | TypeFlags::IndexedAccess
                | TypeFlags::Conditional,
        ) || matches!(&new_target.data, TypeData::IndexedAccess(_));
        if still_generic {
            let key = new_target.id;
            if let Some(cached) = self.index_type_cache.get(&key) {
                return Arc::clone(cached);
            }
            let rebuilt = Arc::new(Type::new(
                TypeFlags::Index,
                TypeData::Index(crate::checker::types::IndexTypeData {
                    constrained: Default::default(),
                    target: Some(Arc::clone(&new_target)),
                    index_flags: idx.index_flags,
                }),
            ));
            self.index_type_cache.insert(key, Arc::clone(&rebuilt));
            return rebuilt;
        }
        self.get_index_type(&new_target)
    }

    pub(crate) fn substitute_infer_mapped(
        &mut self,
        t: &Arc<Type>,
        m: &crate::checker::types::MappedTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_mapped"); 
        self.substitute_infer_mapped_inner(t, m, params, substitutions)
    }

    fn substitute_infer_mapped_inner(
        &mut self,
        t: &Arc<Type>,
        m: &crate::checker::types::MappedTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_mapped_inner"); 
        let chain = self.chained_subst(m, params, substitutions);
        let old_constraint = m.constraint_type.clone();
        let new_constraint = old_constraint
            .as_ref()
            .map(|c| self.substitute_infer_type_parameters(c, params, substitutions));
        let constraint_changed = new_constraint
            .as_ref()
            .zip(old_constraint.as_ref())
            .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));

        // Go instantiateMappedType：同态应用只建惰性壳（instantiateAnonymousType
        // + mapper），keyof 实参的成员域在壳上按需解析。类构建窗口内实参
        // 的成员尚未对齐（pending_base_merges 未合并），急切展开会把不完整
        // keyof 钉进成员，窗口外重算得到两个不同实例（keyofUnresolvedBaseMembers
        // 假 TS2415）。此时保持未替换约束 + template_subst 链，消费方经声明
        // 节点 + 链在取用时重解析
        let subst_target_class_building = m
            .constraint_type
            .as_ref()
            .and_then(|c| match &c.data {
                TypeData::Index(idx) => idx.target.clone(),
                _ => None,
            })
            .and_then(|target| {
                let pos = params.iter().position(|p| self.infer_param_matches(p, &target))?;
                let s = substitutions.get(pos.min(substitutions.len() - 1))?.clone();
                Some(
                    s.symbol.as_ref().is_some_and(|sym| {
                        sym.flags
                            .contains(tsox_frontend::ast::SymbolFlags::Class)
                            && sym.declarations.iter().any(|d| {
                                self.class_build_in_progress.contains(&d.id())
                            })
                    }),
                )
            })
            .unwrap_or(false);
        let new_constraint = if subst_target_class_building {
            None
        } else {
            new_constraint.clone()
        };
        let constraint_changed = !subst_target_class_building && constraint_changed;

        // Go instantiateMappedType 同态分支：{[P in keyof T]: X} 以 T 的实参分发应用
        if !subst_target_class_building
            && let Some(applied) = self.apply_homomorphic_if_concrete(t, m, params, substitutions)
        {
            return applied;
        }
        // 约束已具体（非同态）：按约束域展开成员（key 字面量并集 / 索引签名）
        if !subst_target_class_building
            && let Some(constraint) = new_constraint
                .as_ref()
                .filter(|c| !self.type_is_generic(c))
        {
            if constraint.flags.intersects(TypeFlags::StringLiteral | TypeFlags::Union)
                || constraint.flags.intersects(TypeFlags::String | TypeFlags::Number)
            {
                let constraint = Arc::clone(constraint);
                let chain = chain.clone().unwrap_or_default();
                if let Some(expanded) =
                    self.expand_mapped_by_constraint(t, m, &constraint, &chain)
                {
                    self.attach_substituted_alias(t, &expanded, params, substitutions);
                    return expanded;
                }
            }
        }

        let new_name = m
            .name_type
            .as_ref()
            .map(|n| self.substitute_infer_type_parameters(n, params, substitutions));
        let new_template = m
            .template_type
            .as_ref()
            .map(|tp| self.substitute_infer_type_parameters(tp, params, substitutions));
        let template_changed = new_template
            .as_ref()
            .zip(m.template_type.as_ref())
            .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));
        if !constraint_changed && !template_changed && chain.is_none() {
            return Arc::clone(t);
        }
        let mut rebuilt = Type::new(
            TypeFlags::Object,
            TypeData::Mapped(crate::checker::types::MappedTypeData {
                object: crate::checker::types::ObjectTypeData {
                    target: Some(Arc::clone(t)),
                    mapper: Some(Arc::new(
                        crate::checker::mig::w9a::new_type_mapper(
                            params.to_vec(),
                            substitutions.to_vec(),
                        ),
                    )),
                    ..Default::default()
                },
                declaration: m.declaration.clone(),
                type_parameter: m.type_parameter.clone(),
                constraint_type: new_constraint.or_else(|| m.constraint_type.clone()),
                name_type: new_name.or_else(|| m.name_type.clone()),
                template_type: new_template.or_else(|| m.template_type.clone()),
                template_node: m.template_node.clone(),
                template_subst: chain.map(Box::new),
                modifiers_type: m.modifiers_type.clone(),
                resolved_apparent_type: OnceLock::new(),
                contains_error: std::sync::atomic::AtomicBool::new(
                    m.contains_error.load(std::sync::atomic::Ordering::Relaxed),
                ),
            }),
        );
        rebuilt.object_flags = t.object_flags
            | crate::checker::types::ObjectFlags::Instantiated;
        rebuilt.symbol = t.symbol.clone();
        if let Some(alias) = t.alias.as_ref() {
            let new_args: Vec<Arc<Type>> = alias
                .type_arguments
                .iter()
                .map(|a| self.substitute_infer_type_parameters(a, params, substitutions))
                .collect();
            rebuilt.alias = Some(Box::new(crate::checker::types::TypeAlias::new(
                alias.symbol.clone(),
                new_args,
            )));
        }
        Arc::new(rebuilt)
    }

    /// 替换链 = 原 shell 上的链尾接本次 (params, substitutions)
    fn chained_subst(
        &self,
        m: &crate::checker::types::MappedTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Option<Vec<(Vec<Arc<Type>>, Vec<Arc<Type>>)>> { ::tsox_core::fntrace::enter("chained_subst"); 
        let mut chain: Vec<(Vec<Arc<Type>>, Vec<Arc<Type>>)> = m
            .template_subst
            .as_ref()
            .map(|c| c.as_ref().clone())
            .unwrap_or_default();
        let touched = params.iter().any(|p| {
            m.constraint_type.as_ref().is_some_and(|c| {
                self.type_mentions_param(c, p)
            }) || m.template_type.as_ref().is_some_and(|tp| self.type_mentions_param(tp, p))
        });
        if touched && !params.is_empty() {
            chain.push((params.to_vec(), substitutions.to_vec()));
        }
        if chain.is_empty() {
            None
        } else {
            Some(chain)
        }
    }

    fn type_mentions_param(&self, t: &Arc<Type>, p: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_mentions_param"); 
        if self.infer_param_matches(p, t) {
            return true;
        }
        match &t.data {
            TypeData::Index(idx) => idx
                .target
                .as_ref()
                .is_some_and(|tt| self.type_mentions_param(tt, p)),
            TypeData::IndexedAccess(ia) => {
                ia.object_type
                    .as_ref()
                    .is_some_and(|o| self.type_mentions_param(o, p))
                    || ia
                        .index_type
                        .as_ref()
                        .is_some_and(|i| self.type_mentions_param(i, p))
            }
            TypeData::Mapped(m) => {
                m.constraint_type
                    .as_ref()
                    .is_some_and(|c| self.type_mentions_param(c, p))
                    || m.template_type
                        .as_ref()
                        .is_some_and(|tp| self.type_mentions_param(tp, p))
            }
            _ => false,
        }
    }

    /// Go instantiateMappedType：同态 {[P in keyof T]: X} 且 T 的替换结果已具体时分发应用。
    /// 返回 None 表示保持惰性 shell（T 仍泛型或非同态）
    fn apply_homomorphic_if_concrete(
        &mut self,
        t: &Arc<Type>,
        m: &crate::checker::types::MappedTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("apply_homomorphic_if_concrete"); 
        if m.name_type.is_some() {
            return None;
        }
        let TypeData::Index(idx) = m.constraint_type.as_ref().map(|c| &c.data)? else {
            return None;
        };
        let target = idx.target.as_ref()?;
        if !target.flags.contains(TypeFlags::TypeParameter) {
            return None;
        }
        let pos = params.iter().position(|p| self.infer_param_matches(p, target))?;
        let s = &substitutions[pos.min(substitutions.len() - 1)];
        if self.type_is_generic(s) {
            return None;
        }
        let chain = self.chained_subst(m, params, substitutions).unwrap_or_default();
        let applied = self.apply_mapped_over_type(t, m, s, &chain);
        // Go mapTypeWithAlias：同态应用结果携带别名元数据（显示层按 Deep<实参> 呈现），
        // 实参经本次替换更新。透传（应用结果即替换型本身）以浅拷贝承载别名，
        // 避免污染反向映射候选自身（变量侧按结构显示）
        let applied = if Arc::ptr_eq(&applied, s) && t.alias.is_some() {
            let mut copy = Type::new(
                TypeFlags::Object,
                TypeData::Object(crate::checker::types::ObjectTypeData { node: None,
                    structured: crate::checker::types::StructuredTypeData {
                        members: applied
                            .as_structured()
                            .map(|st| st.members.clone())
                            .unwrap_or_default(),
                        properties: applied
                            .as_structured()
                            .map(|st| st.properties.clone())
                            .unwrap_or_default(),
                        signatures: applied
                            .as_structured()
                            .map(|st| st.signatures.clone())
                            .unwrap_or_default(),
                        call_signature_count: applied
                            .as_structured()
                            .map(|st| st.call_signature_count)
                            .unwrap_or(0),
                        index_infos: applied
                            .as_structured()
                            .map(|st| st.index_infos.clone())
                            .unwrap_or_default(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
            copy.symbol = applied.symbol.clone();
            copy.object_flags = applied.object_flags;
            Arc::new(copy)
        } else {
            applied
        };
        self.attach_substituted_alias(t, &applied, params, substitutions);
        Some(applied)
    }

    pub(crate) fn type_is_generic(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_is_generic"); 
        if t.flags.intersects(
            TypeFlags::TypeParameter | TypeFlags::Index | TypeFlags::IndexedAccess,
        ) {
            return true;
        }
        match &t.data {
            TypeData::IndexedAccess(_) | TypeData::Conditional(_) => true,
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.type_is_generic(c)),
            TypeData::Intersection(i) => i
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.type_is_generic(c)),
            TypeData::Mapped(m) => m
                .constraint_type
                .as_ref()
                .is_some_and(|c| self.type_is_generic(c))
                || m.template_type.as_ref().is_some_and(|tp| self.type_is_generic(tp))
                || m.template_type.is_none(),
            _ => false,
        }
    }
}
