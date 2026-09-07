#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn substitute_infer_tuple(
        &mut self,
        t: &Arc<Type>,
        tup: &TupleTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let new_elems: Vec<Arc<Type>> = tup
            .element_infos
            .iter()
            .map(|ei| match &ei.type_ {
                Some(ty) => self.substitute_infer_type_parameters(ty, params, substitutions),
                None => self.error_type(),
            })
            .collect();

        let changed = tup
            .element_infos
            .iter()
            .zip(new_elems.iter())
            .any(|(ei, new_t)| match &ei.type_ {
                Some(old_t) => !Arc::ptr_eq(old_t, new_t),
                None => true,
            });
        if !changed {
            return Arc::clone(t);
        }
        self.create_tuple_type(new_elems)
    }

    pub(crate) fn substitute_infer_indexed_access(
        &mut self,
        t: &Arc<Type>,
        ia: &IndexedAccessTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let new_object = ia
            .object_type
            .as_ref()
            .map(|o| self.substitute_infer_type_parameters(o, params, substitutions));
        let new_index = ia
            .index_type
            .as_ref()
            .map(|idx| self.substitute_infer_type_parameters(idx, params, substitutions));
        let object_changed = new_object
            .as_ref()
            .zip(ia.object_type.as_ref())
            .map(|(new, old)| !Arc::ptr_eq(new, old))
            .unwrap_or(false);
        let index_changed = new_index
            .as_ref()
            .zip(ia.index_type.as_ref())
            .map(|(new, old)| !Arc::ptr_eq(new, old))
            .unwrap_or(false);
        if !object_changed && !index_changed {
            return Arc::clone(t);
        }
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::IndexedAccess(IndexedAccessTypeData {
                constrained: ConstrainedTypeData::default(),
                object_type: new_object.or_else(|| ia.object_type.clone()),
                index_type: new_index.or_else(|| ia.index_type.clone()),
                access_flags: ia.access_flags,
            }),
        );
        rebuilt.object_flags = t.object_flags;
        rebuilt.symbol = t.symbol.clone();
        Arc::new(rebuilt)
    }

    pub(crate) fn substitute_infer_conditional(
        &mut self,
        t: &Arc<Type>,
        ct: &ConditionalTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let Some(old_check) = ct.check_type.clone() else {
            return Arc::clone(t);
        };
        let new_check = self.substitute_infer_type_parameters(&old_check, params, substitutions);
        if Arc::ptr_eq(&new_check, &old_check) || type_contains_type_parameter(&new_check) {
            return Arc::clone(t);
        }
        self.resolve_conditional_type_with_check(t, Some(new_check))
            .unwrap_or_else(|| Arc::clone(t))
    }
}
