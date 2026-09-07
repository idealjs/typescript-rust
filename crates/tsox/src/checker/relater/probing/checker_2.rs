#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_permissive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let key = t.id;
        if let Some(cached) = self.probe_cache_permissive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Permissive);
        self.probe_cache_permissive.insert(key, Arc::clone(&result));
        result
    }

    pub fn get_restrictive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let key = t.id;
        if let Some(cached) = self.probe_cache_restrictive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Restrictive);
        self.probe_cache_restrictive
            .insert(key, Arc::clone(&result));
        result
    }

    pub(crate) fn instantiate_probing(&mut self, t: &Arc<Type>, mode: ProbeMode) -> Arc<Type> {
        match &t.data {
            TypeData::TypeParameter(_) => match mode {
                ProbeMode::Permissive => self.any_function_type(),
                ProbeMode::Restrictive => {
                    let tp = match &t.data {
                        TypeData::TypeParameter(tp) => tp,
                        _ => unreachable!(),
                    };
                    if tp.constraint.is_none() {
                        return Arc::clone(t);
                    }
                    let mut rebuilt = Type::new(
                        t.flags,
                        TypeData::TypeParameter(TypeParameterData {
                            constrained: ConstrainedTypeData::default(),
                            constraint: None,
                            target: tp.target.clone(),
                            mapper: tp.mapper.clone(),
                            is_this_type: tp.is_this_type,
                            resolved_default_type: OnceLock::new(),
                        }),
                    );
                    rebuilt.symbol = t.symbol.clone();
                    rebuilt.object_flags = t.object_flags;
                    Arc::new(rebuilt)
                }
            },
            TypeData::Union(u) => {
                let types = u.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types
                    .iter()
                    .zip(types.iter())
                    .all(|(n, o)| Arc::ptr_eq(n, o))
                {
                    return Arc::clone(t);
                }
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let types = i.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types
                    .iter()
                    .zip(types.iter())
                    .all(|(n, o)| Arc::ptr_eq(n, o))
                {
                    return Arc::clone(t);
                }
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {
                if o.type_arguments.is_empty() {
                    return Arc::clone(t);
                }
                let new_args: Vec<Arc<Type>> = o
                    .type_arguments
                    .iter()
                    .map(|a| self.instantiate_probing(a, mode))
                    .collect();
                if new_args
                    .iter()
                    .zip(o.type_arguments.iter())
                    .all(|(n, old)| Arc::ptr_eq(n, old))
                {
                    return Arc::clone(t);
                }
                if o.target.is_none() && o.type_arguments.len() == 1 && self.is_array_type(t) {
                    return self.create_array_type(Arc::clone(&new_args[0]));
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData::default(),
                        target: o.target.clone(),
                        mapper: None,
                        type_arguments: new_args,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Tuple(tup) => {
                let args: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .filter_map(|ei| ei.type_.clone())
                    .collect();
                if args.is_empty() {
                    return Arc::clone(t);
                }
                let new_elems: Vec<Arc<Type>> = args
                    .iter()
                    .map(|e| self.instantiate_probing(e, mode))
                    .collect();
                if new_elems
                    .iter()
                    .zip(args.iter())
                    .all(|(n, o)| Arc::ptr_eq(n, o))
                {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }
            TypeData::Conditional(ct) => {
                let (old_check, old_extends) =
                    match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                        (Some(c), Some(e)) => (Arc::clone(c), Arc::clone(e)),
                        _ => return Arc::clone(t),
                    };
                let new_check = self.instantiate_probing(&old_check, mode);
                let new_extends = self.instantiate_probing(&old_extends, mode);
                if Arc::ptr_eq(&new_check, &old_check) && Arc::ptr_eq(&new_extends, &old_extends) {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Conditional(ConditionalTypeData {
                        constrained: ConstrainedTypeData::default(),
                        root: ct.root.as_ref().map(|r| {
                            Box::new(ConditionalRoot {
                                node: r.node.clone(),
                                check_type: r.check_type.clone(),
                                extends_type: r.extends_type.clone(),
                                is_distributive: r.is_distributive,
                                check_type_parameter_symbol: r.check_type_parameter_symbol.clone(),
                                infer_type_parameters: r.infer_type_parameters.clone(),
                                outer_type_parameters: r.outer_type_parameters.clone(),
                                alias: None,
                                creation_scopes: r.creation_scopes.clone(),
                            })
                        }),
                        check_type: Some(new_check),
                        extends_type: Some(new_extends),
                        resolved_true_type: OnceLock::new(),
                        resolved_false_type: OnceLock::new(),
                        resolved_inferred_true_type: OnceLock::new(),
                        resolved_default_constraint: OnceLock::new(),
                        resolved_constraint_of_distributive: OnceLock::new(),
                        mapper: None,
                        combined_mapper: None,
                        creation_type_argument_stack: Vec::new(),
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            TypeData::IndexedAccess(ia) => {
                let (Some(old_obj), Some(old_idx)) =
                    (ia.object_type.as_ref(), ia.index_type.as_ref())
                else {
                    return Arc::clone(t);
                };
                let new_obj = self.instantiate_probing(old_obj, mode);
                let new_idx = self.instantiate_probing(old_idx, mode);
                if Arc::ptr_eq(&new_obj, old_obj) && Arc::ptr_eq(&new_idx, old_idx) {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: Some(new_obj),
                        index_type: Some(new_idx),
                        access_flags: ia.access_flags,
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            _ => Arc::clone(t),
        }
    }

    pub(crate) fn type_param_symbols_share_container(
        &self,
        a: &Arc<Symbol>,
        b: &Arc<Symbol>,
    ) -> bool {
        let symbol_map = self.program.symbol_map();
        let container_of = |s: &Arc<Symbol>| -> Option<usize> {
            let mut node = s.declarations.first()?.parent.as_ref()?;
            for _ in 0..4 {
                if let Some(sym) = symbol_map.symbols.get(&node.id()) {
                    return Some(Arc::as_ptr(sym) as *const Symbol as usize);
                }
                node = node.parent.as_ref()?;
            }
            None
        };
        match (container_of(a), container_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }
}
