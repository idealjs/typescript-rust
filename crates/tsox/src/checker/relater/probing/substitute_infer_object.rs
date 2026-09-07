#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn substitute_infer_object(
        &mut self,
        t: &Arc<Type>,
        o: &ObjectTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        if t.object_flags.contains(ObjectFlags::Reference)
            && o.target.is_none()
            && o.type_arguments.len() == 1
        {
            let new_elem =
                self.substitute_infer_type_parameters(&o.type_arguments[0], params, substitutions);

            if Arc::ptr_eq(&new_elem, &o.type_arguments[0]) {
                return Arc::clone(t);
            }
            return self.create_array_type(new_elem);
        }

        if !o.type_arguments.is_empty() {
            let new_args: Vec<Arc<Type>> = o
                .type_arguments
                .iter()
                .map(|arg| self.substitute_infer_type_parameters(arg, params, substitutions))
                .collect();
            let changed = o
                .type_arguments
                .iter()
                .zip(new_args.iter())
                .any(|(old, new)| !Arc::ptr_eq(old, new));
            if changed {
                let new_index_infos: Vec<Arc<crate::checker::IndexInfo>> = o
                    .structured
                    .index_infos
                    .iter()
                    .map(|info| {
                        let new_value = info.value_type.as_ref().map(|v| {
                            self.substitute_infer_type_parameters(v, params, substitutions)
                        });
                        if new_value.is_some()
                            && !new_value.as_ref().is_some_and(|nv| {
                                info.value_type
                                    .as_ref()
                                    .is_some_and(|ov| Arc::ptr_eq(nv, ov))
                            })
                        {
                            Arc::new(crate::checker::IndexInfo {
                                key_type: info.key_type.clone(),
                                value_type: new_value,
                                is_readonly: info.is_readonly,
                                declaration: info.declaration.clone(),
                                index_symbol: info.index_symbol.clone(),
                                components: info.components.clone(),
                            })
                        } else {
                            Arc::clone(info)
                        }
                    })
                    .collect();
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: o.structured.signatures.clone(),
                            call_signature_count: o.structured.call_signature_count,
                            index_infos: new_index_infos,
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: new_args,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                return Arc::new(rebuilt);
            }
            return Arc::clone(t);
        }

        if t.object_flags.contains(ObjectFlags::Anonymous) && !o.structured.signatures.is_empty() {
            let signatures = o.structured.signatures.clone();
            let call_signature_count = o.structured.call_signature_count;
            let mut changed = false;
            let mut new_sigs: Vec<Arc<Signature>> = Vec::with_capacity(signatures.len());
            for sig in &signatures {
                let rest_offset = usize::from(sig.has_rest_parameter());
                let fixed = sig.parameters.len().saturating_sub(rest_offset);
                let mut new_params: Vec<Arc<Type>> = Vec::with_capacity(sig.parameters.len());
                let mut old_params: Vec<Arc<Type>> = Vec::with_capacity(sig.parameters.len());
                for i in 0..fixed {
                    let pt = self
                        .try_get_type_at_position(sig, i)
                        .unwrap_or_else(|| self.any_type());
                    old_params.push(Arc::clone(&pt));
                    new_params.push(self.substitute_infer_type_parameters(
                        &pt,
                        params,
                        substitutions,
                    ));
                }
                if rest_offset == 1 {
                    if let Some(last) = sig.parameters.last() {
                        let rt = self.get_type_of_symbol(last);
                        old_params.push(Arc::clone(&rt));
                        new_params.push(self.substitute_infer_type_parameters(
                            &rt,
                            params,
                            substitutions,
                        ));
                    }
                }
                let new_return = self
                    .get_return_type_of_signature(sig)
                    .map(|rt| self.substitute_infer_type_parameters(&rt, params, substitutions));
                let params_changed = old_params
                    .iter()
                    .zip(new_params.iter())
                    .any(|(old, new)| !Arc::ptr_eq(old, new));
                let return_changed = new_return.as_ref().is_some_and(|nr| {
                    self.get_return_type_of_signature(sig)
                        .is_some_and(|old| !Arc::ptr_eq(nr, &old))
                });
                if !params_changed && !return_changed {
                    new_sigs.push(Arc::clone(sig));
                    continue;
                }
                changed = true;
                let mut inst = Signature::new();
                inst.flags = sig.flags;
                inst.min_argument_count = sig.min_argument_count;
                inst.resolved_min_argument_count = sig.resolved_min_argument_count;
                inst.declaration = sig.declaration.clone();
                inst.target = Some(Arc::clone(sig));
                inst.parameters = sig.parameters.clone();
                inst.this_parameter = sig.this_parameter.clone();
                inst.type_parameters = sig.type_parameters.clone();
                inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
                inst.instantiated_parameter_types = Some(new_params);
                if let Some(nr) = new_return {
                    let _ = inst.resolved_return_type.set(nr);
                }
                new_sigs.push(Arc::new(inst));
            }
            if !changed {
                return Arc::clone(t);
            }
            let is_construct = call_signature_count == 0;
            return self.create_function_or_constructor_type(new_sigs, is_construct);
        }

        if self.in_return_substitution && t.symbol.is_none() && !o.structured.properties.is_empty()
        {
            let fresh = self.subst_object_in_progress.is_empty();
            let result = self.substitute_object_properties_deep(t, params, substitutions);
            if fresh {
                self.subst_object_in_progress.clear();
            }
            return result;
        }

        Arc::clone(t)
    }
}
