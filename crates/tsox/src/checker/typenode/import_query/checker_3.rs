#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn instantiate_value_type_for_type_query(
        &mut self,
        base: &Arc<Type>,
        arg_types: &[Arc<Type>],
    ) -> Arc<Type> {
        if arg_types.is_empty() {
            return Arc::clone(base);
        }
        match &base.data {
            TypeData::Intersection(i) => {
                let parts: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_intersection_type(parts)
            }
            TypeData::Union(u) => {
                let parts: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_union_type(parts)
            }
            TypeData::Object(o) => {
                let call_count = o.structured.call_signature_count;
                let sigs = &o.structured.signatures;
                let mut changed = false;
                let mut new_sigs: Vec<Arc<crate::checker::types::Signature>> =
                    Vec::with_capacity(sigs.len());
                for (idx, sig) in sigs.iter().enumerate() {
                    let is_construct = idx >= call_count;

                    let mut params: Vec<Arc<Type>> = sig.type_parameters.clone();
                    if params.is_empty() && is_construct {
                        if let Some(rt) = self.get_return_type_of_signature(sig)
                            && let Some(class_sym) = &rt.symbol
                        {
                            let class_tps = self.declared_type_parameter_types(class_sym);
                            if !class_tps.is_empty() {
                                params = class_tps;
                            }
                        }
                    }
                    if params.is_empty() || arg_types.len() > params.len() {
                        new_sigs.push(Arc::clone(sig));
                        continue;
                    }
                    let inst = self.get_signature_instantiation(sig, arg_types);

                    let rt0 = self.get_return_type_of_signature(&inst);
                    let inst = match rt0 {
                        Some(rt0) => {
                            let deep =
                                self.substitute_object_properties_deep(&rt0, &params, arg_types);
                            let mut rebuilt = crate::checker::types::Signature::new();
                            rebuilt.flags = inst.flags;
                            rebuilt.min_argument_count = inst.min_argument_count;
                            rebuilt.resolved_min_argument_count = inst.resolved_min_argument_count;
                            rebuilt.declaration = inst.declaration.clone();
                            rebuilt.parameters = inst.parameters.clone();
                            rebuilt.this_parameter = inst.this_parameter.clone();
                            rebuilt.resolved_type_predicate = inst.resolved_type_predicate.clone();
                            rebuilt.target = inst.target.clone();
                            rebuilt.mapper = inst.mapper.clone();
                            rebuilt.instantiated_parameter_types =
                                inst.instantiated_parameter_types.clone();
                            if let Some(it) = inst.isolated_signature_type.get() {
                                let _ = rebuilt.isolated_signature_type.set(it.clone());
                            }
                            let _ = rebuilt.resolved_return_type.set(deep);
                            Arc::new(rebuilt)
                        }
                        None => inst,
                    };
                    changed = true;
                    new_sigs.push(inst);
                }
                if !changed {
                    return Arc::clone(base);
                }
                let shell = Arc::new(Type::new(
                    base.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: new_sigs,
                            call_signature_count: call_count,
                            index_infos: o.structured.index_infos.clone(),
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: o.type_arguments.clone(),
                    }),
                ));
                {
                    let t_mut = Arc::as_ptr(&shell) as *mut Type;
                    unsafe {
                        (*t_mut).object_flags =
                            base.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                        (*t_mut).symbol = base.symbol.clone();
                    }
                }
                shell
            }
            _ => Arc::clone(base),
        }
    }
}
