#![allow(unused_imports)]

use crate::checker::inference::InferencePriority;
use crate::checker::relater_conditional::*;

impl Checker {
    pub fn get_false_type_from_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_false_type_from_conditional_type");
        let branch_node = match &t.data {
            TypeData::Conditional(ct) => {
                if let Some(rt) = ct.resolved_false_type.get() {
                    return Some(Arc::clone(rt));
                }
                let cond_node = ct.root.as_ref()?.node.as_ref()?;
                match &cond_node.data {
                    NodeData::ConditionalTypeNode(d) => {
                        Some((Arc::clone(&d.false_type), ct.mapper.clone()))
                    }
                    _ => None,
                }
            }
            _ => None,
        }?;
        let (node, mapper) = branch_node;
        let branch = self.resolve_conditional_branch_node(t, &node);
        let branch = match mapper.as_ref() {
            Some(m) => self.instantiate_type(&branch, Some(m)),
            None => branch,
        };
        if let TypeData::Conditional(ct) = &t.data {
            let _ = ct.resolved_false_type.set(Arc::clone(&branch));
        }
        Some(branch)
    }

    pub fn conditional_is_distribution_dependent(&self, _t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("conditional_is_distribution_dependent"); 
        true
    }

    pub(crate) fn get_simplified_type_for_relation(
        &mut self,
        t: &Arc<Type>,
        writing: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_simplified_type_for_relation");
        if t.flags.contains(TypeFlags::Conditional) {
            return self.get_simplified_conditional_type(t, writing);
        }
        // Go getNormalizedType（checker.go:29536-29538）TypeFlagsSimplifiable 臂
        //（IndexedAccess|Conditional|Index，types.go:497）：索引访问源经
        // getSimplifiedType 归一（getSimplifiedIndexedAccessType checker.go:29570
        // ——交集对象 distributeIndexOverObjectType checker.go:29646 逐成分分发 +
        // 泛型映射对象 substituteIndexedMappedType checker.go:29624 模板代入），
        // 缺失使延迟索引访问源在关系判定中无法化简（如
        // ObjectHasKey<T,'1'> = ({[K in Extract<keyof T,string>]:'true'} &
        // {[k:string]:'false'})['1'] 应化简为 'true' & 'false' = never），
        // checkIndexedAccessIndexType（checker.go:8709）的 everyType 可指派判定
        // 失败误报 TS2536（deferredLookupTypeResolution2 波及回归）
        if t.flags.contains(TypeFlags::IndexedAccess) {
            return self.get_simplified_type(t, writing);
        }
        Arc::clone(t)
    }

    pub(crate) fn get_simplified_conditional_type(
        &mut self,
        t: &Arc<Type>,
        writing: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_simplified_conditional_type"); 
        let (Some(check_type), Some(extends_type)) = (match &t.data {
            TypeData::Conditional(ct) => (ct.check_type.clone(), ct.extends_type.clone()),
            _ => (None, None),
        })
        else {
            return Arc::clone(t);
        };
        let Some(true_type) = self.get_forced_branch_type_of_conditional_type(t, true) else {
            return Arc::clone(t);
        };
        let Some(false_type) = self.get_forced_branch_type_of_conditional_type(t, false) else {
            return Arc::clone(t);
        };
        let check_actual = get_actual_type_variable(&check_type);
        let restrictive_check = self.get_restrictive_instantiation(&check_type);
        let restrictive_extends = self.get_restrictive_instantiation(&extends_type);
        let was_silent = self.silence_relation_chain();
        let check_assignable_extends =
            self.is_type_assignable_to(&restrictive_check, &restrictive_extends);
        self.restore_relation_chain(was_silent);
        let parts_empty = self.conditional_parts_intersection_empty(&check_type, &extends_type);
        let mut result: Option<Arc<Type>> = None;
        if false_type.flags.contains(TypeFlags::Never)
            && Arc::ptr_eq(&get_actual_type_variable(&true_type), &check_actual)
        {
            if check_type.flags.contains(TypeFlags::Any) || check_assignable_extends {
                result = Some(self.get_simplified_type_for_relation(&true_type, writing));
            } else if parts_empty {
                result = Some(self.never_type());
            }
        } else if true_type.flags.contains(TypeFlags::Never)
            && Arc::ptr_eq(&get_actual_type_variable(&false_type), &check_actual)
        {
            if !check_type.flags.contains(TypeFlags::Any) && check_assignable_extends {
                result = Some(self.never_type());
            } else if check_type.flags.contains(TypeFlags::Any) || parts_empty {
                result = Some(self.get_simplified_type_for_relation(&false_type, writing));
            }
        }
        result.unwrap_or_else(|| Arc::clone(t))
    }

    pub(crate) fn mapped_source_related_to_type_param_target(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<bool> { ::tsox_core::fntrace::enter("mapped_source_related_to_type_param_target"); 
        let m = match &source.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };
        if !source.object_flags.contains(ObjectFlags::Mapped) || m.name_type.is_some() {
            return None;
        }
        let key_type = self.get_index_type(target);
        let constraint_type = m.constraint_type.clone()?;
        if !self.is_type_related_to(&key_type, &constraint_type, relation) {
            return None;
        }
        let has_optional = m.declaration.as_ref().and_then(|d| match &d.data {
            NodeData::MappedTypeNode(md) => md
                .question_token
                .as_ref()
                .map(|t| t.kind == SyntaxKind::QuestionToken),
            _ => None,
        });
        if has_optional == Some(true) {
            return None;
        }
        let type_param = m.type_parameter.clone()?;
        let template = self.get_template_type_from_mapped_type(source)?;
        let indexed = self.get_indexed_access_type(target, &type_param);
        Some(self.is_type_related_to(&template, &indexed, relation))
    }

    fn conditional_parts_intersection_empty(
        &mut self,
        a: &Arc<Type>,
        b: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("conditional_parts_intersection_empty"); 
        if a.flags.contains(TypeFlags::Never) || b.flags.contains(TypeFlags::Never) {
            return true;
        }
        let i = self.get_intersection_type(vec![Arc::clone(a), Arc::clone(b)]);
        i.flags.contains(TypeFlags::Never)
    }

    pub(crate) fn constraint_of_distributive_conditional(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("constraint_of_distributive_conditional"); 
        // Go getConstraintOfDistributiveConditionalType（checker.go:18291-18316），
        // m1d_2 get_constraint_of_distributive_conditional_type 为同构实现：
        // 约束 = check 的简化型或约束（getSimplifiedType 恒等回落
        // getConstraintOfType），约束实例化整个条件型
        //（getConditionalTypeInstantiation forConstraint，check 型映射到约束，
        // 联合约束分布），实例化归约 never 即放弃。原自造结构以约束直判
        // permissive/restrictive 分支（thenable 约束命中 true 分支递归产自引用
        // deferred），r1 声明回源使类型参数拿到真约束后
        // anticipatedTypeCrash/asyncFunctionReturnType 的 Awaited<T>（T 约束
        // Promise<never>，实例化 = Awaited<Promise<never>> = never 应放弃）
        // 与 Awaited<TObj[K]> 停 deferred 未归约——unknown 兜底时代判 false
        // 分支返 T 的巧合掩盖了结构偏离，委派 Go 同构版收敛
        self.get_constraint_of_distributive_conditional_type(t)
    }

    pub(crate) fn conditional_fallback_related(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool { ::tsox_core::fntrace::enter("conditional_fallback_related"); 
        self.conditional_fallback_related_probing(source, target, relation)
    }

    fn conditional_root_node_id(t: &Arc<Type>) -> Option<u64> { ::tsox_core::fntrace::enter("conditional_root_node_id"); 
        match &t.data {
            TypeData::Conditional(tct) => tct
                .root
                .as_ref()
                .and_then(|r| r.node.as_ref())
                .map(|n| n.id()),
            _ => None,
        }
    }

    fn conditional_fallback_related_probing(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool { ::tsox_core::fntrace::enter("conditional_fallback_related_probing"); 
        if source.flags.contains(TypeFlags::Conditional)
            && self.deferred_constraint_depth < 100
        {
            if let Some(root_id) = Self::conditional_root_node_id(source) {
                if self.deferred_conditional_root_stack.contains(&root_id) {
                    return relation == RelationKind::Comparable;
                }
                self.deferred_conditional_root_stack.push(root_id);
                let recursed = self
                    .deferred_default_constraint_of_conditional(source)
                    .is_some_and(|constraint| {
                        self.deferred_constraint_depth += 1;
                        let r = self.is_type_related_to(&constraint, target, relation);
                        self.deferred_constraint_depth -= 1;
                        r
                    });
                self.deferred_conditional_root_stack.pop();
                if recursed {
                    return true;
                }
            }
        }

        if source.flags.contains(TypeFlags::Conditional)
            && !target.flags.contains(TypeFlags::Conditional)
            && self.deferred_constraint_depth < 100
            && let Some(distributive) = self.constraint_of_distributive_conditional(source)
        {
            self.deferred_constraint_depth += 1;
            let r = self.is_type_related_to(&distributive, target, relation);
            self.deferred_constraint_depth -= 1;
            if r {
                return true;
            }
        }

        if let TypeData::Conditional(tct) = &target.data {
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
                    return true;
                } else if let Some(true_branch) =
                    self.get_forced_branch_type_of_conditional_type(target, true)
                {
                    if self.is_type_related_to(source, &true_branch, relation) {
                        let skip_false = {
                            let rc = self.get_restrictive_instantiation(&check);
                            let re = self.get_restrictive_instantiation(&extends);
                            self.is_type_assignable_to(&rc, &re)
                        };
                        if skip_false {
                            return true;
                        } else if let Some(false_branch) =
                            self.get_forced_branch_type_of_conditional_type(target, false)
                        {
                            if self.is_type_related_to(source, &false_branch, relation) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

fn get_actual_type_variable(t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_actual_type_variable"); 
    if t.flags.contains(TypeFlags::Substitution)
        && let TypeData::Substitution(sub) = &t.data
        && let Some(base) = &sub.base_type
    {
        return get_actual_type_variable(base);
    }
    Arc::clone(t)
}

impl Checker {
    pub(crate) fn conditional_four_way_related(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<bool> { ::tsox_core::fntrace::enter("conditional_four_way_related"); 
        let (sct, tct) = match (&source.data, &target.data) {
            (TypeData::Conditional(s), TypeData::Conditional(t)) => (s, t),
            _ => return None,
        };
        let source_params = sct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        let (s_check, t_check) = (sct.check_type.clone()?, tct.check_type.clone()?);
        let s_extends_raw = sct.extends_type.clone()?;
        let t_extends = tct.extends_type.clone()?;
        let (s_extends, mapper_types) = if source_params.is_empty() {
            (s_extends_raw, None)
        } else {
            let inferences: Vec<crate::checker::inference::InferenceInfo> = source_params
                .iter()
                .map(|p| crate::checker::inference::InferenceInfo::new(Arc::clone(p)))
                .collect();
            let mut context = crate::checker::inference::InferenceContext::new(inferences);
            self.infer_types(
                &mut context.inferences,
                Some(Arc::clone(&t_extends)),
                Some(Arc::clone(&s_extends_raw)),
                InferencePriority::NoConstraints | InferencePriority::AlwaysStrict,
                false,
            );
            let inferred = self.get_inferred_types(&context);
            let inst =
                self.substitute_infer_type_parameters(&s_extends_raw, &source_params, &inferred);
            (inst, Some(inferred))
        };
        if !self.is_type_related_to(&s_extends, &t_extends, RelationKind::Identity) {
            return None;
        }
        if !self.is_type_related_to(&s_check, &t_check, relation)
            && !self.is_type_related_to(&t_check, &s_check, relation)
        {
            return None;
        }
        let apply_mapper = |c: &mut Self, t: &Arc<Type>| -> Arc<Type> {
            match &mapper_types {
                Some(inferred) => c.substitute_infer_type_parameters(t, &source_params, inferred),
                None => Arc::clone(t),
            }
        };
        let s_true = self.get_forced_branch_type_of_conditional_type(source, true)?;
        let t_true = self.get_forced_branch_type_of_conditional_type(target, true)?;
        let s_false = self.get_forced_branch_type_of_conditional_type(source, false)?;
        let t_false = self.get_forced_branch_type_of_conditional_type(target, false)?;
        let s_true = apply_mapper(self, &s_true);
        let s_false = apply_mapper(self, &s_false);
        if !self.is_type_related_to(&s_true, &t_true, relation) {
            return Some(false);
        }
        Some(self.is_type_related_to(&s_false, &t_false, relation))
    }
}
