#![allow(unused_imports)]

use super::*;

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
                                    && self.type_param_symbols_share_container(ps, ts))
                        })))
            {
                return Arc::clone(&substitutions[i.min(substitutions.len() - 1)]);
            }
        }

        match &t.data {
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
            _ => Arc::clone(t),
        }
    }
}
