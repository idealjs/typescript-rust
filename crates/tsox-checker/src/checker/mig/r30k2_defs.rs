use crate::checker::checker_checker_checker::Checker;
use crate::checker::inference_inference_key_2::InferenceContext;
use crate::checker::types::{Type, TypeMapper, TypeMapperKind};
use std::sync::Arc;

pub(crate) struct ActiveChecker(*mut Checker);
unsafe impl Send for ActiveChecker {}
unsafe impl Sync for ActiveChecker {}

impl ActiveChecker {
    fn checker(&self) -> &mut Checker { ::tsox_core::fntrace::enter("checker"); 
        unsafe { &mut *self.0 }
    }
}

pub(crate) struct ContextPtr(*mut InferenceContext);
unsafe impl Send for ContextPtr {}
unsafe impl Sync for ContextPtr {}

impl ContextPtr {
    fn context(&self) -> &mut InferenceContext { ::tsox_core::fntrace::enter("context"); 
        unsafe { &mut *self.0 }
    }
}

pub(crate) fn new_inference_type_mapper(
    c: &Checker,
    n: &mut InferenceContext,
    fixing: bool,
) -> Arc<TypeMapper> { ::tsox_core::fntrace::enter("new_inference_type_mapper"); 
    let checker = ActiveChecker(c as *const Checker as *mut Checker);
    let context = ContextPtr(n as *mut InferenceContext);
    Arc::new(TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            let n = context.context();
            for (i, inference) in n.inferences.iter().enumerate() {
                if Arc::ptr_eq(t, &inference.type_parameter) {
                    let c = checker.checker();
                    if fixing && !inference.is_fixed {
                        c.infer_from_intra_expression_sites(n);
                        crate::checker::mig::m2e_2::clear_cached_inferences(&mut n.inferences);
                        n.inferences[i].is_fixed = true;
                    }
                    return c.get_inferred_type(n, i);
                }
            }
            Arc::clone(t)
        }),
        TypeMapperKind::Unknown,
        false,
    ))
}
