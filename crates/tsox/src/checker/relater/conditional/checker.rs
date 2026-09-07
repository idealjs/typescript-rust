#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn conditional_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> Option<Ternary> {
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
            (Some(check), Some(extends)) => !self.is_type_assignable_to(check, extends),
            _ => false,
        };
        let skip_false = if skip_true {
            false
        } else {
            match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                (Some(check), Some(extends)) => self.is_type_assignable_to(check, extends),
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
    ) -> Option<Ternary> {
        let sm = match &source.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };
        let tm = match &target.data {
            TypeData::Mapped(m) => m,
            _ => return None,
        };

        if sm.name_type.is_some() || tm.name_type.is_some() {
            return None;
        }

        if relation == RelationKind::Identity {
            return None;
        }

        let source_constraint = self.get_constraint_type_from_mapped_type(source)?;
        let target_constraint = self.get_constraint_type_from_mapped_type(target)?;
        let constraint_related = self.compare_types(
            Arc::clone(&target_constraint),
            Arc::clone(&source_constraint),
            relation,
            false,
        );
        if constraint_related.is_false() {
            return Some(Ternary::False);
        }

        let source_template = self.get_template_type_from_mapped_type(source)?;
        let target_template = self.get_template_type_from_mapped_type(target)?;
        let template_related = self.compare_types(
            Arc::clone(&source_template),
            Arc::clone(&target_template),
            relation,
            false,
        );
        Some(constraint_related.and(template_related))
    }

    pub fn get_constraint_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.constraint_type.clone();
        }
        None
    }

    pub fn get_type_parameter_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.type_parameter.clone();
        }
        None
    }

    pub fn get_name_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.name_type.clone();
        }
        None
    }

    pub fn get_forced_branch_type_of_conditional_type(
        &mut self,
        t: &Arc<Type>,
        take_true: bool,
    ) -> Option<Arc<Type>> {
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
        if pushes_creation {
            self.type_argument_stack.pop();
        }
        if scopes_pushed > 0 {
            self.scope_stack
                .truncate(self.scope_stack.len() - scopes_pushed);
        }
        Some(branch)
    }

    pub(crate) fn deferred_default_constraint_of_conditional(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let root = match &t.data {
            TypeData::Conditional(ct) => ct.root.as_ref()?,
            _ => return None,
        };
        if !Self::conditional_distribution_independent(root) {
            return None;
        }
        let (true_branch, false_branch) = (
            self.get_forced_branch_type_of_conditional_type(t, true),
            self.get_forced_branch_type_of_conditional_type(t, false),
        );
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

    pub(crate) fn conditional_distribution_independent(root: &ConditionalRoot) -> bool {
        if !root.is_distributive {
            return true;
        }
        let Some(param_sym) = root.check_type_parameter_symbol.as_ref() else {
            return false;
        };
        let cond_node = match root.node.as_ref().map(|n| &n.data) {
            Some(NodeData::ConditionalTypeNode(d)) => d,
            _ => return false,
        };
        let is_top_level_reference = |node: &Arc<Node>| -> bool {
            let mut queue: Vec<&Arc<Node>> = vec![node];
            while let Some(current) = queue.pop() {
                match &current.data {
                    NodeData::UnionTypeNode(u) => {
                        for member in u.types.iter() {
                            queue.push(member);
                        }
                    }
                    NodeData::ParenthesizedTypeNode(p) => queue.push(&p.type_node),
                    NodeData::TypeReferenceNode(r) => {
                        if r.type_name.kind == SyntaxKind::Identifier
                            && r.type_name.text() == param_sym.name
                        {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        };
        !is_top_level_reference(&cond_node.true_type)
            && !is_top_level_reference(&cond_node.false_type)
    }

    pub fn get_true_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
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
}
