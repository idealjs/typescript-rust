#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    pub fn substitute_infer_type_parameters(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        if params.is_empty() || substitutions.is_empty() {
            return Arc::clone(t);
        }

        for (i, p) in params.iter().enumerate() {
            if Arc::ptr_eq(p, t)
                || (p.is_type_parameter()
                    && t.is_type_parameter()
                    && (p
                        .symbol
                        .as_ref()
                        .zip(t.symbol.as_ref())
                        .is_some_and(|(ps, ts)| {
                            Arc::ptr_eq(ps, ts)
                                || (ps.name == ts.name
                                    && self.type_param_symbols_equivalent(ps, ts))
                        })))
            {
                return Arc::clone(&substitutions[i.min(substitutions.len() - 1)]);
            }
        }

        match &t.data {
            TypeData::Substitution(sub) => {
                let new_base = sub
                    .base_type
                    .as_ref()
                    .map(|b| self.substitute_infer_type_parameters(b, params, substitutions));
                let changed = new_base
                    .as_ref()
                    .zip(sub.base_type.as_ref())
                    .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));
                if !changed {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Substitution(SubstitutionTypeData {
                        constrained: ConstrainedTypeData::default(),
                        base_type: new_base.or_else(|| sub.base_type.clone()),
                        constraint: sub.constraint.clone(),
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Union(u) => {
                let new_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let new_types: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => self.substitute_infer_object(t, o, params, substitutions),
            TypeData::Tuple(tup) => self.substitute_infer_tuple(t, tup, params, substitutions),
            TypeData::IndexedAccess(ia) => {
                self.substitute_infer_indexed_access(t, ia, params, substitutions)
            }
            TypeData::Conditional(ct) => {
                self.substitute_infer_conditional(t, ct, params, substitutions)
            }
            TypeData::Index(idx) => self.substitute_infer_index(t, idx, params, substitutions),
            TypeData::Mapped(m) => self.substitute_infer_mapped(t, m, params, substitutions),
            _ => Arc::clone(t),
        }
    }
}
