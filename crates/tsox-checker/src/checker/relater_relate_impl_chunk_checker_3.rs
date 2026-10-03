#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn constraint_of_indexed_access(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("constraint_of_indexed_access"); 
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
                                if let tsox_frontend::ast::NodeData::TypeParameterDeclaration(
                                    data,
                                ) = &decl.data
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
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("reduce_type_for_constraint"); 
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

    pub(crate) fn indexed_access_constraint_for_chain(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("indexed_access_constraint_for_chain"); 
        let ia = match &t.data {
            TypeData::IndexedAccess(ia) => ia,
            _ => return None,
        };
        let object = ia.object_type.as_ref()?;
        let index = ia.index_type.as_ref()?;
        if let Some(substituted) = self.substitute_generic_mapped_indexed_access(object, index) {
            return Some(substituted);
        }
        if let Some(ic) = self.chain_simplified_or_constraint(index) {
            if !Arc::ptr_eq(&ic, index) {
                if let Some(access) = self.chain_indexed_access(object, &ic) {
                    return Some(access);
                }
            }
        }
        if let Some(oc) = self.chain_simplified_or_constraint(object) {
            if !Arc::ptr_eq(&oc, object) {
                if let Some(access) = self.chain_indexed_access(&oc, index) {
                    return Some(access);
                }
            }
        }
        None
    }

    fn chain_simplified_or_constraint(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("chain_simplified_or_constraint"); 
        if t.flags.contains(TypeFlags::Index) || matches!(&t.data, TypeData::Index(_)) {
            let parts = vec![
                self.string_type(),
                self.number_type(),
                self.es_symbol_type(),
            ];
            return Some(self.get_union_type(parts));
        }
        if t.flags.contains(TypeFlags::TypeParameter) {
            return self.get_constraint_of_type_parameter(t);
        }
        if t.flags.contains(TypeFlags::IndexedAccess)
            || matches!(&t.data, TypeData::IndexedAccess(_))
        {
            return self.indexed_access_constraint_for_chain(t);
        }
        None
    }

    fn chain_indexed_access(
        &mut self,
        object: &Arc<Type>,
        index: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("chain_indexed_access"); 
        if object.flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
            return Some(Arc::clone(object));
        }
        if index.flags.contains(TypeFlags::Union) {
            let members = index.types()?.to_vec();
            let mut parts = Vec::with_capacity(members.len());
            for m in &members {
                parts.push(self.chain_indexed_access(object, m)?);
            }
            return Some(self.get_union_type(parts));
        }
        if object.is_union() {
            let members = object.types()?.to_vec();
            let mut parts = Vec::with_capacity(members.len());
            for m in &members {
                parts.push(self.chain_indexed_access(m, index)?);
            }
            return Some(self.get_union_type(parts));
        }
        if self.chain_is_generic_type(object) {
            return Some(self.deferred_indexed_access(object, index));
        }
        self.try_get_indexed_access_type(object, index, AccessFlags::None)
    }

    fn chain_is_generic_type(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("chain_is_generic_type"); 
        if t.flags
            .intersects(TypeFlags::TypeParameter | TypeFlags::Index | TypeFlags::IndexedAccess)
        {
            return true;
        }
        if let TypeData::IndexedAccess(ia) = &t.data {
            return ia
                .object_type
                .as_ref()
                .is_some_and(|o| self.chain_is_generic_type(o))
                || ia
                    .index_type
                    .as_ref()
                    .is_some_and(|i| self.chain_is_generic_type(i));
        }
        false
    }
}
