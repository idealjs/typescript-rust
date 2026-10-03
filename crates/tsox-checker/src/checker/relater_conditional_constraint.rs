#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn check_type_base_constraint(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("check_type_base_constraint"); 
        if t.flags.intersects(TypeFlags::Union | TypeFlags::Intersection)
            && let Some(constituents) = t.types()
        {
            let mut constraints: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
            let mut different = false;
            for s in constituents {
                let base = match &s.data {
                    TypeData::TypeParameter(_) => self
                        .get_base_constraint_of_type(s)
                        .map(|b| self.resolve_indexed_access_base(&b)),
                    TypeData::Conditional(_)
                    | TypeData::IndexedAccess(_)
                    | TypeData::Index(_) => self.get_base_constraint_of_type(s),
                    _ => Some(Arc::clone(s)),
                };
                match base {
                    Some(c) => {
                        if !Arc::ptr_eq(&c, s) {
                            different = true;
                        }
                        constraints.push(c);
                    }
                    None => different = true,
                }
            }
            if !different {
                return None;
            }
            if t.flags.contains(TypeFlags::Union) {
                if constraints.len() == constituents.len() {
                    return Some(self.get_union_type(constraints));
                }
                return None;
            }
            if constraints.is_empty() {
                return None;
            }
            return Some(self.get_intersection_type(constraints));
        }
        let base = self.get_base_constraint_of_type(t)?;
        if Arc::ptr_eq(&base, t) {
            return None;
        }
        Some(base)
    }

    fn resolve_indexed_access_base(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_indexed_access_base"); 
        let ia = match &t.data {
            TypeData::IndexedAccess(ia) => ia,
            _ => return Arc::clone(t),
        };
        let (Some(object), Some(index)) = (&ia.object_type, &ia.index_type) else {
            return Arc::clone(t);
        };
        let resolved = self.get_indexed_access_type(&Arc::clone(object), &Arc::clone(index));
        if resolved.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
            return Arc::clone(t);
        }
        resolved
    }

    pub(crate) fn constraint_of_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("constraint_of_conditional_type"); 
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
        } else if check_type.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(&check_type)?;
            if constraint.flags.contains(TypeFlags::Union) {
                constraint.types()?.to_vec()
            } else {
                vec![constraint]
            }
        } else if let Some(constraint) = self.check_type_base_constraint(&check_type) {
            if constraint.flags.contains(TypeFlags::Union) {
                constraint.types()?.to_vec()
            } else {
                vec![constraint]
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
            let r = self.resolve_conditional_type_with_check(t, Some(constituent), None);
            self.type_argument_stack.pop();
            results.push(r?);
        }
        Some(self.get_union_type(results))
    }
}
