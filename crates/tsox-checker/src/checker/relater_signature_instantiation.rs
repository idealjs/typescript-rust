#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;
use std::sync::Arc;

impl Checker {
    pub fn get_signature_instantiation(
        &mut self,
        sig: &Arc<Signature>,
        type_args: &[Arc<Type>],
        inferred_type_parameters: &[Arc<Type>],
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("get_signature_instantiation");
        let instantiated = self.signature_instantiation_worker(sig, type_args);
        if inferred_type_parameters.is_empty() {
            return instantiated;
        }
        let Some(return_type) = self.get_return_type_of_signature(&instantiated) else {
            return instantiated;
        };
        let Some(return_signature) = self.get_single_call_or_construct_signature(&return_type)
        else {
            return instantiated;
        };
        let mut new_return_signature = Signature::new();
        new_return_signature.flags = return_signature.flags;
        new_return_signature.min_argument_count = return_signature.min_argument_count;
        new_return_signature.resolved_min_argument_count =
            return_signature.resolved_min_argument_count;
        new_return_signature.declaration = return_signature.declaration.clone();
        new_return_signature.type_parameters = inferred_type_parameters.to_vec();
        new_return_signature.parameters = return_signature.parameters.clone();
        new_return_signature.this_parameter = return_signature.this_parameter.clone();
        new_return_signature.target = return_signature.target.clone();
        new_return_signature.mapper = return_signature.mapper.clone();
        new_return_signature.instantiated_parameter_types =
            return_signature.instantiated_parameter_types.clone();
        if let Some(rt) = self.get_return_type_of_signature(&return_signature) {
            let _ = new_return_signature.resolved_return_type.set(rt);
        }
        let new_return_type =
            self.get_or_create_type_from_signature(&Arc::new(new_return_signature));
        let mut new_instantiated = Signature::new();
        new_instantiated.flags = instantiated.flags;
        new_instantiated.min_argument_count = instantiated.min_argument_count;
        new_instantiated.resolved_min_argument_count = instantiated.resolved_min_argument_count;
        new_instantiated.declaration = instantiated.declaration.clone();
        new_instantiated.type_parameters = instantiated.type_parameters.clone();
        new_instantiated.parameters = instantiated.parameters.clone();
        new_instantiated.this_parameter = instantiated.this_parameter.clone();
        new_instantiated.target = instantiated.target.clone();
        new_instantiated.mapper = instantiated.mapper.clone();
        new_instantiated.instantiated_parameter_types =
            instantiated.instantiated_parameter_types.clone();
        let _ = new_instantiated.resolved_return_type.set(new_return_type);
        Arc::new(new_instantiated)
    }

    fn signature_instantiation_worker(
        &mut self,
        sig: &Arc<Signature>,
        type_args: &[Arc<Type>],
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("signature_instantiation_worker");
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
            let rest_type = self
                .get_effective_rest_type(sig)
                .expect("rest parameter");
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

        let mapper = self.create_signature_type_mapper(sig, type_args);
        inst.mapper = Some(Arc::clone(&mapper));
        inst.target = Some(Arc::clone(sig));
        inst.parameters = sig.parameters.clone();
        inst.this_parameter = sig.this_parameter.clone();
        inst.instantiated_parameter_types = Some(param_types);
        if let Some(rt) = self.get_return_type_of_signature(sig) {
            let substituted =
                self.substitute_infer_type_parameters(&rt, &sig.type_parameters, type_args);
            let _ = inst.resolved_return_type.set(substituted);
        }
        Arc::new(inst)
    }
}
