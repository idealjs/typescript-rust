#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn try_get_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        if let Some(overrides) = &sig.instantiated_parameter_types {
            let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
            let param_count = overrides.len().saturating_sub(rest_offset);
            if pos < param_count {
                return Some(Arc::clone(&overrides[pos]));
            }
            if sig.has_rest_parameter() {
                let rest_type = Arc::clone(&overrides[param_count]);
                if is_tuple_type(&rest_type) {
                    if let TypeData::Tuple(t) = &rest_type.data {
                        let index = pos - param_count;
                        let has_variadic = t.combined_flags.contains(ElementFlags::Variadic);
                        if index < t.fixed_length || has_variadic {
                            return t
                                .element_infos
                                .get(index)
                                .and_then(|info| info.type_.clone())
                                .or_else(|| Some(self.any_type()));
                        }
                    }
                } else if let Some(elem) = self.get_array_element_type_of(&rest_type) {
                    return Some(elem);
                }
                return Some(self.any_type());
            }
            return None;
        }
        let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
        let param_count = sig.parameters.len() - rest_offset;
        if pos < param_count {
            return Some(self.get_type_of_symbol(&sig.parameters[pos]));
        }
        if sig.has_rest_parameter() {
            let rest_param = &sig.parameters[param_count];
            let rest_type = self.get_type_of_symbol(rest_param);

            if is_tuple_type(&rest_type) {
                if let TypeData::Tuple(t) = &rest_type.data {
                    let index = pos - param_count;
                    let has_variadic = t.combined_flags.contains(ElementFlags::Variadic);
                    if index < t.fixed_length || has_variadic {
                        return t
                            .element_infos
                            .get(index)
                            .and_then(|info| info.type_.clone())
                            .or_else(|| Some(self.any_type()));
                    }
                }
            }
        }
        None
    }

    pub fn get_rest_or_any_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        let rest_type = self.get_rest_type_at_position(sig, pos);
        if let Some(rt) = &rest_type {
            if self.is_array_type(rt) {
                let elem = self.get_type_arguments(rt).into_iter().next();
                if let Some(elem) = elem {
                    if elem.flags.contains(TypeFlags::Any) {
                        return self.any_type();
                    }
                }
            }
        }
        rest_type.unwrap_or_else(|| self.any_type())
    }

    pub fn get_rest_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        let parameter_count = self.get_parameter_count(sig);
        if pos >= parameter_count.saturating_sub(1) {
            return self.get_effective_rest_type(sig);
        }
        None
    }

    pub fn get_effective_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        if let Some(overrides) = &sig.instantiated_parameter_types {
            return overrides.last().cloned();
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);

        Some(rest_type)
    }

    pub fn get_non_array_rest_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        if !sig.has_rest_parameter() {
            return None;
        }
        if let Some(overrides) = &sig.instantiated_parameter_types {
            let rest_type = overrides.last()?.clone();
            if is_tuple_type(&rest_type) {
                return Some(rest_type);
            }
            if self.is_array_type(&rest_type) {
                return self.get_type_arguments(&rest_type).into_iter().next();
            }
            return Some(rest_type);
        }
        let last = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(last);

        if is_tuple_type(&rest_type) {
            return Some(rest_type);
        }

        if self.is_array_type(&rest_type) {
            return self.get_type_arguments(&rest_type).into_iter().next();
        }
        Some(rest_type)
    }

    pub(crate) fn get_array_element_type_of(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if self.is_array_type(t) {
            return Some(self.get_array_element_type(t));
        }
        None
    }

    pub fn is_top_signature(&mut self, sig: &Arc<Signature>) -> bool {
        if !sig.type_parameters.is_empty() {
            return false;
        }

        if let Some(this_param) = &sig.this_parameter {
            let this_type = self.get_type_of_symbol(this_param);
            if !this_type.flags.contains(TypeFlags::Any) {
                return false;
            }
        }
        if sig.parameters.len() != 1 || !sig.has_rest_parameter() {
            return false;
        }
        let Some(param) = sig.parameters.first() else {
            return false;
        };
        let param_type = self.get_type_of_symbol(param);
        let rest_type = if self.is_array_type(&param_type) {
            self.get_type_arguments(&param_type).into_iter().next()
        } else {
            Some(param_type)
        };
        match rest_type {
            Some(rt) => {
                if !rt.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                    return false;
                }
                let return_type = self.get_return_type_of_signature(sig);
                match return_type {
                    Some(rt) => rt.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN),
                    None => false,
                }
            }
            None => false,
        }
    }

    pub fn get_this_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        let this_param = sig.this_parameter.as_ref()?;
        let links = self.value_symbol_links.get(this_param)?;
        links.resolved_type.clone()
    }

    pub fn get_non_circular_return_type_of_signature(&self, sig: &Arc<Signature>) -> Arc<Type> {
        self.get_return_type_of_signature(sig)
            .unwrap_or_else(|| self.any_type())
    }

    pub fn get_erased_signature(&mut self, sig: &Arc<Signature>) -> Arc<Signature> {
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let args: Vec<Arc<Type>> = sig
            .type_parameters
            .iter()
            .map(|_| self.any_type())
            .collect();
        self.get_signature_instantiation(sig, &args)
    }

    pub fn get_signature_instantiation(
        &mut self,
        sig: &Arc<Signature>,
        type_args: &[Arc<Type>],
    ) -> Arc<Signature> {
        if type_args.is_empty() || sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }

        let mut param_types: Vec<Arc<Type>> = Vec::with_capacity(sig.parameters.len());
        let rest_offset = if sig.has_rest_parameter() { 1 } else { 0 };
        let fixed = sig.parameters.len() - rest_offset;
        for i in 0..fixed {
            let t = self
                .try_get_type_at_position(sig, i)
                .unwrap_or_else(|| self.any_type());
            param_types.push(self.substitute_infer_type_parameters(
                &t,
                &sig.type_parameters,
                type_args,
            ));
        }
        if rest_offset == 1 {
            let last = sig.parameters.last().expect("rest parameter");
            let rest_type = self.get_type_of_symbol(last);
            param_types.push(self.substitute_infer_type_parameters(
                &rest_type,
                &sig.type_parameters,
                type_args,
            ));
        }
        let mut inst = Signature::new();
        inst.flags = sig.flags;
        inst.min_argument_count = sig.min_argument_count;
        inst.resolved_min_argument_count = sig.resolved_min_argument_count;
        inst.declaration = sig.declaration.clone();

        inst.target = Some(Arc::clone(sig));
        inst.parameters = sig.parameters.clone();
        inst.this_parameter = sig.this_parameter.clone();
        inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
        inst.instantiated_parameter_types = Some(param_types);
        if let Some(rt) = self.get_return_type_of_signature(sig) {
            let substituted =
                self.substitute_infer_type_parameters(&rt, &sig.type_parameters, type_args);
            let _ = inst.resolved_return_type.set(substituted);
        }
        Arc::new(inst)
    }
}
