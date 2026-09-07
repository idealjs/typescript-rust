#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn constraint_of_indexed_access(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ia = match &t.data {
            TypeData::IndexedAccess(ia) => ia,
            _ => return None,
        };
        let object = ia.object_type.as_ref()?;
        let index = ia.index_type.as_ref()?;

        let obj_constraint = if object.flags.contains(TypeFlags::TypeParameter) {
            match self.get_constraint_of_type_parameter(object) {
                Some(c) => c,
                None => {
                    let sym = object.symbol.as_ref()?;

                    let canonical = self
                        .type_alias_links
                        .get(sym)
                        .and_then(|l| l.declared_type.clone())
                        .and_then(|c| self.get_constraint_of_type_parameter(&c));
                    match canonical {
                        Some(c) => c,
                        None => {
                            let mut from_decl = None;
                            for decl in &sym.declarations {
                                if let crate::ast::NodeData::TypeParameterDeclaration(data) =
                                    &decl.data
                                {
                                    if let Some(constraint_node) = &data.constraint {
                                        from_decl =
                                            Some(self.get_type_from_type_node(constraint_node));
                                    }
                                    break;
                                }
                            }
                            from_decl?
                        }
                    }
                }
            }
        } else if matches!(
            &object.data,
            TypeData::IndexedAccess(_) | TypeData::Conditional(_)
        ) {
            self.constraint_of_indexed_access(object)?
        } else if index.flags.contains(TypeFlags::TypeParameter) {
            let idx_constraint = self.get_constraint_of_type_parameter(index)?;
            let kind_ok = idx_constraint.flags.intersects(
                TypeFlags::String
                    | TypeFlags::Number
                    | TypeFlags::StringLiteral
                    | TypeFlags::NumberLiteral
                    | TypeFlags::ESSymbol,
            ) || (idx_constraint.is_union()
                && idx_constraint.types().is_some_and(|ts| {
                    ts.iter().all(|c| {
                        c.flags
                            .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
                    })
                }));
            if !kind_ok {
                return None;
            }
            let resolved = self.get_indexed_access_type(object, &idx_constraint);
            if resolved.flags.contains(TypeFlags::Never) {
                return None;
            }
            return Some(resolved);
        } else {
            return None;
        };

        if matches!(
            obj_constraint.intrinsic_name(),
            Some("any") | Some("unknown") | Some("error")
        ) {
            return None;
        }

        let effective_index = if index
            .flags
            .intersects(TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index)
            || matches!(&index.data, TypeData::IndexedAccess(_))
        {
            match self.reduce_type_for_constraint(index, 8) {
                Some(reduced) => reduced,
                None => return None,
            }
        } else {
            Arc::clone(index)
        };
        let resolved = self.get_indexed_access_type(&obj_constraint, &effective_index);
        if matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
            return None;
        }
        Some(resolved)
    }

    pub(crate) fn reduce_type_for_constraint(
        &mut self,
        t: &Arc<Type>,
        depth: usize,
    ) -> Option<Arc<Type>> {
        if depth == 0 {
            return None;
        }
        if t.flags.contains(TypeFlags::TypeParameter) {
            if t.flags.contains(TypeFlags::Union) {
                return Some(Arc::clone(t));
            }
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.reduce_type_for_constraint(&constraint, depth - 1);
        }
        if t.flags.contains(TypeFlags::IndexedAccess)
            || matches!(&t.data, TypeData::IndexedAccess(_))
        {
            return self.constraint_of_indexed_access(t);
        }
        if t.flags.contains(TypeFlags::Index) {
            if let TypeData::Index(it) = &t.data
                && let Some(target) = &it.target
            {
                let reduced = self.reduce_type_for_constraint(target, depth - 1)?;
                return Some(self.get_index_type(&reduced));
            }
            return None;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                let mut reduced_all = Vec::with_capacity(u.union_or_intersection.types.len());
                for c in &u.union_or_intersection.types {
                    reduced_all.push(self.reduce_type_for_constraint(c, depth - 1)?);
                }
                return Some(self.get_union_type(reduced_all));
            }
        }
        Some(Arc::clone(t))
    }

    pub(crate) fn constraint_of_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        if let Some(rt) = ct.resolved_true_type.get() {
            return Some(Arc::clone(rt));
        }
        if let Some(rt) = ct.resolved_false_type.get() {
            return Some(Arc::clone(rt));
        }
        let check_type = ct.check_type.clone()?;
        let tp_symbol = ct
            .root
            .as_ref()
            .filter(|r| r.is_distributive)
            .and_then(|r| r.check_type_parameter_symbol.clone())?;

        let constituents: Vec<Arc<Type>> = if check_type.flags.contains(TypeFlags::Union) {
            check_type.types()?.to_vec()
        } else if check_type.flags.contains(TypeFlags::IndexedAccess)
            || matches!(&check_type.data, TypeData::IndexedAccess(_))
        {
            let reduced = self.constraint_of_indexed_access(&check_type)?;
            if reduced.flags.contains(TypeFlags::Union) {
                reduced.types()?.to_vec()
            } else {
                vec![reduced]
            }
        } else {
            return None;
        };
        let key = Arc::as_ptr(&tp_symbol);
        let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
        for constituent in constituents {
            let mut mapping = std::collections::HashMap::new();
            mapping.insert(key, Arc::clone(&constituent));
            self.type_argument_stack.push(mapping);
            let r = self.resolve_conditional_type_with_check(t, Some(constituent));
            self.type_argument_stack.pop();
            results.push(r?);
        }
        Some(self.get_union_type(results))
    }
}
