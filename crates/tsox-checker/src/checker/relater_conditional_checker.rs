#![allow(unused_imports)]

use crate::checker::relater_conditional::*;

impl Checker {
    pub fn conditional_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> { ::tsox_core::fntrace::enter("conditional_type_related_to"); 
        let ct = match &target.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        if let Some(root) = &ct.root {
            if !root.infer_type_parameters.is_empty() {
                return None;
            }

            if root.is_distributive && self.conditional_is_distribution_dependent(target) {
                return None;
            }
        }

        if let TypeData::Conditional(sct) = &source.data {
            if let (Some(s_root), Some(t_root)) = (&sct.root, &ct.root) {
                if std::ptr::eq(s_root.as_ref() as *const _, t_root.as_ref() as *const _) {
                    return None;
                }
            }
        }

        let skip_true = match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
            (Some(check), Some(extends)) => {
                let pc = self.get_permissive_instantiation(check);
                let pe = self.get_permissive_instantiation(extends);
                let was_silent = self.silence_relation_chain();
                let r = !self.is_type_assignable_to(&pc, &pe);
                self.restore_relation_chain(was_silent);
                r
            }
            _ => false,
        };
        let skip_false = if skip_true {
            false
        } else {
            match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                (Some(check), Some(extends)) => {
                    let rc = self.get_restrictive_instantiation(check);
                    let re = self.get_restrictive_instantiation(extends);
                    let was_silent = self.silence_relation_chain();
                    let r = self.is_type_assignable_to(&rc, &re);
                    self.restore_relation_chain(was_silent);
                    r
                }
                _ => false,
            }
        };

        let mut result = Ternary::True;
        if !skip_true {
            let true_branch = self.get_true_type_from_conditional_type(target)?;
            let r = self.compare_types(Arc::clone(source), true_branch, relation, false);
            if r.is_false() {
                return Some(Ternary::False);
            }
            result = result.and(r);
        }
        if !skip_false {
            let false_branch = self.get_false_type_from_conditional_type(target)?;
            let r = self.compare_types(Arc::clone(source), false_branch, relation, false);
            if r.is_false() {
                return Some(Ternary::False);
            }
            result = result.and(r);
        }
        Some(result)
    }

    pub fn mapped_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> { ::tsox_core::fntrace::enter("mapped_type_related_to"); 
        match (&source.data, &target.data) {
            (TypeData::Mapped(_), TypeData::Mapped(_)) => {}
            _ => return None,
        }

        let source_optionality = self.get_combined_mapped_type_optionality(source);
        let modifiers_related = relation == RelationKind::Comparable
            || (relation == RelationKind::Identity
                && crate::checker::mig::wc2_2::get_mapped_type_modifiers(source)
                    == crate::checker::mig::wc2_2::get_mapped_type_modifiers(target))
            || (relation != RelationKind::Identity
                && source_optionality <= self.get_combined_mapped_type_optionality(target));
        if !modifiers_related {
            return Some(Ternary::False);
        }

        let source_constraint = self.get_constraint_type_from_mapped_type(source)?;
        let target_constraint = self.get_constraint_type_from_mapped_type(target)?;
        if source_optionality < 0 {
            self.report_unmeasurable_markers(&source_constraint);
        } else {
            self.report_unreliable_markers(&source_constraint);
        }
        let constraint_related = self.compare_types(
            Arc::clone(&target_constraint),
            Arc::clone(&source_constraint),
            relation,
            false,
        );
        if constraint_related.is_false() {
            return Some(Ternary::False);
        }

        let source_tp = self.get_type_parameter_from_mapped_type(source)?;
        let target_tp = self.get_type_parameter_from_mapped_type(target)?;
        let mapper = Arc::new(crate::checker::mapper::new_simple_type_mapper(
            Arc::clone(&source_tp),
            Arc::clone(&target_tp),
        ));
        let names_equal = match (
            self.get_name_type_from_mapped_type(source),
            self.get_name_type_from_mapped_type(target),
        ) {
            (None, None) => true,
            (Some(s), Some(t)) => {
                let s_mapped = self.instantiate_type(&s, Some(&mapper));
                let t_mapped = self.instantiate_type(&t, Some(&mapper));
                Arc::ptr_eq(&s_mapped, &t_mapped) || self.is_type_identical_to(&s_mapped, &t_mapped)
            }
            _ => false,
        };
        if !names_equal {
            return Some(Ternary::False);
        }

        let source_template = self.get_template_type_from_mapped_type(source)?;
        let target_template = self.get_template_type_from_mapped_type(target)?;
        let source_template = self.instantiate_type(&source_template, Some(&mapper));
        let template_related = self.compare_types(source_template, target_template, relation, false);
        Some(constraint_related.and(template_related))
    }

    pub fn get_constraint_type_from_mapped_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_constraint_type_from_mapped_type"); 
        if let TypeData::Mapped(m) = &t.data {
            if let Some(constraint) = &m.constraint_type {
                return Some(Arc::clone(constraint));
            }
            let target = m.object.target.clone()?;
            let mapper = m.object.mapper.clone();
            // 壳链环守卫：同一壳的约束解析在途时按 Go instantiateType 深度上限
            // 的等价出口收敛为 error 型，阻断 target 链自环造成的无限实例化
            if !self.mapped_shell_resolving.insert(t.id) {
                return Some(self.error_type());
            }
            let resolved = self.get_constraint_type_from_mapped_type(&target);
            self.mapped_shell_resolving.remove(&t.id);
            let source_constraint = resolved?;
            return Some(self.instantiate_type(&source_constraint, mapper.as_ref()));
        }
        None
    }

    pub fn get_type_parameter_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_parameter_from_mapped_type"); 
        if let TypeData::Mapped(m) = &t.data {
            return m.type_parameter.clone();
        }
        None
    }

    pub fn get_name_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_name_type_from_mapped_type"); 
        if let TypeData::Mapped(m) = &t.data {
            return m.name_type.clone();
        }
        None
    }

    pub fn get_forced_branch_type_of_conditional_type(
        &mut self,
        t: &Arc<Type>,
        take_true: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_forced_branch_type_of_conditional_type"); 
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };
        if let Some(cached) = if take_true {
            ct.resolved_true_type.get()
        } else {
            ct.resolved_false_type.get()
        } {
            return Some(Arc::clone(cached));
        }
        let cond_node = ct.root.as_ref()?.node.as_ref()?;
        let branch_node = match &cond_node.data {
            NodeData::ConditionalTypeNode(d) => {
                if take_true {
                    Arc::clone(&d.true_type)
                } else {
                    Arc::clone(&d.false_type)
                }
            }
            _ => return None,
        };

        let creation_scopes: Vec<u64> = ct
            .root
            .as_ref()
            .map(|r| r.creation_scopes.clone())
            .unwrap_or_default();
        let mut common = 0usize;
        while common < creation_scopes.len()
            && common < self.scope_stack.len()
            && creation_scopes[common] == self.scope_stack[common]
        {
            common += 1;
        }
        let scopes_pushed = creation_scopes.len() - common;
        self.scope_stack
            .extend_from_slice(&creation_scopes[common..]);

        let mut merged_creation: HashMap<usize, Arc<Type>> = HashMap::new();
        for frame in ct.creation_type_argument_stack.iter() {
            for (k, v) in frame {
                merged_creation.insert(*k, Arc::clone(v));
            }
        }
        for map in self.type_argument_stack.iter() {
            for k in map.keys() {
                merged_creation.remove(&(*k as usize));
            }
        }
        let pushes_creation = !merged_creation.is_empty();
        if pushes_creation {
            self.type_argument_stack.push(
                merged_creation
                    .into_iter()
                    .map(|(k, v)| ((k as *const Symbol), v))
                    .collect(),
            );
        }

        if take_true {
            self.push_scope(&cond_node);
        }
        let branch = self.get_type_from_type_node(&branch_node);
        if take_true {
            self.pop_scope();
        }
        let branch = match ct.mapper.as_ref() {
            Some(m) => m.map(&branch),
            None => branch,
        };
        if pushes_creation {
            self.type_argument_stack.pop();
        }
        if scopes_pushed > 0 {
            self.scope_stack
                .truncate(self.scope_stack.len() - scopes_pushed);
        }
        if let TypeData::Conditional(ct_cell) = &t.data {
            let cell = if take_true {
                &ct_cell.resolved_true_type
            } else {
                &ct_cell.resolved_false_type
            };
            let _ = cell.set(Arc::clone(&branch));
        }
        Some(branch)
    }

    pub(crate) fn deferred_default_constraint_of_conditional(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("deferred_default_constraint_of_conditional"); 
        let true_branch = self.get_inferred_true_type_of_conditional(t);
        let false_branch = self.get_forced_branch_type_of_conditional_type(t, false);
        match (true_branch, false_branch) {
            (Some(tb), Some(fb)) => {
                if tb.flags.contains(TypeFlags::Any) {
                    Some(fb)
                } else if fb.flags.contains(TypeFlags::Any) {
                    Some(tb)
                } else {
                    Some(self.get_union_type(vec![tb, fb]))
                }
            }
            (only, None) | (None, only) => only,
        }
    }

    pub(crate) fn conditional_distribution_independent(root: &ConditionalRoot) -> bool { ::tsox_core::fntrace::enter("conditional_distribution_independent"); 
        if !root.is_distributive {
            return true;
        }
        // Go isDistributionDependent：check 型类型参数在任一结果分支中被
        // 引用（isTypeParameterPossiblyReferenced 递归整棵子树，含嵌套位）
        // 即分布依赖；符号缺失或声明不唯一时保守视为依赖
        let Some(param_sym) = root.check_type_parameter_symbol.as_ref() else {
            return false;
        };
        let cond_node = match root.node.as_ref().map(|n| &n.data) {
            Some(NodeData::ConditionalTypeNode(d)) => d,
            _ => return false,
        };
        if param_sym.declarations.len() != 1 {
            return false;
        }
        let name = param_sym.name.clone();
        !crate::checker::typenode_type_operators::type_node_references_names(&cond_node.true_type, &[name.clone()])
            && !crate::checker::typenode_type_operators::type_node_references_names(&cond_node.false_type, &[name])
    }

    pub fn get_true_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_true_type_from_conditional_type"); 
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_true_type.get() {
                return Some(rt.clone());
            }

            if let Some(rt) = ct.resolved_inferred_true_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    fn get_inferred_true_type_of_conditional(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_inferred_true_type_of_conditional"); 
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_inferred_true_type.get() {
                return Some(rt.clone());
            }
        }
        self.get_forced_branch_type_of_conditional_type(t, true)
    }
}
