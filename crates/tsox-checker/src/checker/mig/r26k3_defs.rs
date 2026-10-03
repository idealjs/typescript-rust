use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::Symbol;

use crate::checker::checker::Checker;
use crate::checker::types::{InstantiationExpressionKey, Type, TypeData, TypeFlags, TypeMapper, TypeMapperKind};
use crate::checker::utilities_has_only_expression_initialization::is_this_type_parameter;

use super::EnumLiteralValue;

thread_local! {
    static ENUM_NAN_LITERAL_TYPES: RefCell<HashMap<usize, Arc<Type>>> = RefCell::new(HashMap::new());
    static ENUM_LITERAL_TYPES: RefCell<HashMap<(usize, EnumLiteralValue), Arc<Type>>> = RefCell::new(HashMap::new());
    static DEFERRED_GLOBAL_IMPORT_META_EXPRESSION_TYPES: RefCell<HashMap<usize, Arc<Type>>> = RefCell::new(HashMap::new());
    static INSTANTIATION_EXPRESSION_TYPES: RefCell<HashMap<InstantiationExpressionKey, Arc<Type>>> = RefCell::new(HashMap::new());
}

pub(crate) fn enum_nan_literal_types_get(key: &usize) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("enum_nan_literal_types_get"); 
    ENUM_NAN_LITERAL_TYPES.with(|m| m.borrow().get(key).cloned())
}

pub(crate) fn enum_nan_literal_types_insert(key: usize, t: Arc<Type>) { ::tsox_core::fntrace::enter("enum_nan_literal_types_insert"); 
    ENUM_NAN_LITERAL_TYPES.with(|m| {
        m.borrow_mut().insert(key, t);
    });
}

pub(crate) fn enum_literal_types_get(key: &(usize, EnumLiteralValue)) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("enum_literal_types_get"); 
    ENUM_LITERAL_TYPES.with(|m| m.borrow().get(key).cloned())
}

pub(crate) fn enum_literal_types_insert(key: (usize, EnumLiteralValue), t: Arc<Type>) { ::tsox_core::fntrace::enter("enum_literal_types_insert"); 
    ENUM_LITERAL_TYPES.with(|m| {
        m.borrow_mut().insert(key, t);
    });
}

fn checker_slot(c: &Checker) -> usize { ::tsox_core::fntrace::enter("checker_slot"); 
    c as *const Checker as *const () as usize
}

pub(crate) fn deferred_global_import_meta_expression_type_get(c: &Checker) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("deferred_global_import_meta_expression_type_get"); 
    DEFERRED_GLOBAL_IMPORT_META_EXPRESSION_TYPES.with(|m| m.borrow().get(&checker_slot(c)).cloned())
}

pub(crate) fn deferred_global_import_meta_expression_type_set(c: &Checker, t: Arc<Type>) { ::tsox_core::fntrace::enter("deferred_global_import_meta_expression_type_set"); 
    DEFERRED_GLOBAL_IMPORT_META_EXPRESSION_TYPES.with(|m| {
        m.borrow_mut().insert(checker_slot(c), t);
    });
}

pub(crate) fn instantiation_expression_types_get(key: &InstantiationExpressionKey) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiation_expression_types_get"); 
    INSTANTIATION_EXPRESSION_TYPES.with(|m| m.borrow().get(key).cloned())
}

pub(crate) fn instantiation_expression_types_insert(key: InstantiationExpressionKey, t: Arc<Type>) { ::tsox_core::fntrace::enter("instantiation_expression_types_insert"); 
    INSTANTIATION_EXPRESSION_TYPES.with(|m| {
        m.borrow_mut().insert(key, t);
    });
}

pub(crate) fn every_type_with_checker(
    c: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> bool,
) -> bool { ::tsox_core::fntrace::enter("every_type_with_checker"); 
    if t.flags.intersects(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            let types = u.union_or_intersection.types.clone();
            return types.iter().all(|t| f(c, t));
        }
    }
    f(c, t)
}

pub(crate) fn some_type_with_checker(
    c: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> bool,
) -> bool { ::tsox_core::fntrace::enter("some_type_with_checker"); 
    if t.flags.intersects(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            let types = u.union_or_intersection.types.clone();
            return types.iter().any(|t| f(c, t));
        }
    }
    f(c, t)
}

struct ActiveChecker(*mut Checker);
unsafe impl Send for ActiveChecker {}
unsafe impl Sync for ActiveChecker {}

impl ActiveChecker {
    fn checker(&self) -> &mut Checker { ::tsox_core::fntrace::enter("checker"); 
        unsafe { &mut *self.0 }
    }
}

pub(crate) type DeferredTargetFn = Box<dyn Fn(&mut Checker) -> Arc<Type> + Send + Sync>;

pub(crate) fn new_deferred_type_mapper(
    c: &Checker,
    sources: &[Arc<Type>],
    targets: Vec<DeferredTargetFn>,
) -> Arc<TypeMapper> { ::tsox_core::fntrace::enter("new_deferred_type_mapper"); 
    let maps_this_only = sources.len() == 1 && is_this_type_parameter(&sources[0]);
    let checker = ActiveChecker(c as *const Checker as *mut Checker);
    let sources = sources.to_vec();
    Arc::new(TypeMapper::new(
        Arc::new(move |t: &Arc<Type>| {
            for (i, s) in sources.iter().enumerate() {
                if Arc::ptr_eq(t, s) {
                    let c = checker.checker();
                    return (targets[i])(c);
                }
            }
            Arc::clone(t)
        }),
        TypeMapperKind::Array,
        maps_this_only,
    ))
}
