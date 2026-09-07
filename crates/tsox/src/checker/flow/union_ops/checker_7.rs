#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn narrow_to_subtype(
        &mut self,
        type_: &Arc<Type>,
        candidate: &Arc<Type>,
    ) -> Arc<Type> {
        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(candidate);
        }
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let mapped: Vec<Arc<Type>> = constituents
                .into_iter()
                .map(|t| {
                    if self.is_type_assignable_to(&t, candidate) {
                        t
                    } else if self.is_type_assignable_to(candidate, &t) {
                        Arc::clone(candidate)
                    } else {
                        self.never_type()
                    }
                })
                .collect();
            return self.rebuild_union_or_never(type_, mapped);
        }

        if self.is_type_assignable_to(candidate, type_) {
            Arc::clone(candidate)
        } else {
            Arc::clone(type_)
        }
    }

    pub(crate) fn remove_subtype_from_union(
        &mut self,
        type_: &Arc<Type>,
        candidate: &Arc<Type>,
    ) -> Arc<Type> {
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !self.is_type_assignable_to(t, candidate))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        if self.is_type_assignable_to(type_, candidate) {
            self.never_type()
        } else {
            Arc::clone(type_)
        }
    }

    pub(crate) fn rebuild_union_or_never(
        &mut self,
        original: &Arc<Type>,
        constituents: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        if constituents.is_empty() {
            return self.never_type();
        }
        if constituents.len() == 1 {
            return constituents.into_iter().next().expect("exactly one");
        }

        if let TypeData::Union(u) = &original.data {
            if u.union_or_intersection.types.len() == constituents.len()
                && u.union_or_intersection
                    .types
                    .iter()
                    .zip(constituents.iter())
                    .all(|(a, b)| Arc::ptr_eq(a, b))
            {
                return Arc::clone(original);
            }
        }
        self.get_union_type(constituents)
    }
}
