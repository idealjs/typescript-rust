#![allow(unused_imports)]

use crate::checker::relater_conditional::*;

impl Checker {
    pub fn get_false_type_from_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    pub fn conditional_is_distribution_dependent(&self, _t: &Arc<Type>) -> bool {
        true
    }

    pub(crate) fn get_simplified_type_for_relation(
        &mut self,
        t: &Arc<Type>,
        writing: bool,
    ) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Conditional) {
            return self.get_simplified_conditional_type(t, writing);
        }
        Arc::clone(t)
    }

    pub(crate) fn get_simplified_conditional_type(
        &mut self,
        t: &Arc<Type>,
        writing: bool,
    ) -> Arc<Type> {
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
        let check_assignable_extends =
            self.is_type_assignable_to(&restrictive_check, &restrictive_extends);
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
    ) -> Option<bool> {
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
    ) -> bool {
        if a.flags.contains(TypeFlags::Never) || b.flags.contains(TypeFlags::Never) {
            return true;
        }
        let i = self.get_intersection_type(vec![Arc::clone(a), Arc::clone(b)]);
        i.flags.contains(TypeFlags::Never)
    }

    pub(crate) fn conditional_fallback_related(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if source.flags.contains(TypeFlags::Conditional)
            && self.deferred_constraint_depth < 100
            && let Some(constraint) = self.deferred_default_constraint_of_conditional(source)
        {
            self.deferred_constraint_depth += 1;
            let r = self.is_type_related_to(&constraint, target, relation);
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

fn get_actual_type_variable(t: &Arc<Type>) -> Arc<Type> {
    if t.flags.contains(TypeFlags::Substitution)
        && let TypeData::Substitution(sub) = &t.data
        && let Some(base) = &sub.base_type
    {
        return get_actual_type_variable(base);
    }
    Arc::clone(t)
}
