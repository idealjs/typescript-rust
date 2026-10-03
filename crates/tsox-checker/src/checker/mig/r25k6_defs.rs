#![allow(unused_imports)]
use crate::checker::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::*;

pub(crate) fn map_type_with_checker(
    c: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> Option<Arc<Type>>,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("map_type_with_checker"); 
    map_type_ex_with_checker(c, t, f, false)
}

fn map_type_ex_with_checker(
    c: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> Option<Arc<Type>>,
    no_reductions: bool,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("map_type_ex_with_checker"); 
    if t.flags.contains(TypeFlags::Never) {
        return Some(Arc::clone(t));
    }
    if !t.flags.contains(TypeFlags::Union) {
        return f(c, t);
    }
    let mut types = t.types().unwrap_or(&[]).to_vec();
    if let Some(origin) = t.as_union_type().and_then(|u| u.origin.clone()) {
        if origin.flags.contains(TypeFlags::Union) {
            types = origin.types().unwrap_or(&[]).to_vec();
        }
    }
    let mut mapped_types: Vec<Arc<Type>> = Vec::new();
    let mut changed = false;
    for s in &types {
        let mapped = if s.flags.contains(TypeFlags::Union) {
            map_type_ex_with_checker(c, s, f, no_reductions)
        } else {
            f(c, s)
        };
        if !mapped.as_ref().is_some_and(|m| Arc::ptr_eq(m, s)) {
            changed = true;
        }
        if let Some(mapped) = mapped {
            mapped_types.push(mapped);
        }
    }
    if changed {
        if mapped_types.is_empty() {
            return None;
        }
        let reduction = if no_reductions {
            UnionReduction::None
        } else {
            UnionReduction::Literal
        };
        return Some(c.get_union_type_ex(mapped_types, reduction));
    }
    Some(Arc::clone(t))
}

thread_local! {
    static ALIAS_TYPE_INSTANTIATIONS: RefCell<HashMap<usize, HashMap<CacheHashKey, Arc<Type>>>> =
        RefCell::new(HashMap::new());
}

fn alias_instantiations_slot(symbol: &Arc<Symbol>) -> usize { ::tsox_core::fntrace::enter("alias_instantiations_slot"); 
    Arc::as_ptr(symbol) as *const () as usize
}

pub(crate) fn alias_instantiations_get(
    symbol: &Arc<Symbol>,
    key: &CacheHashKey,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("alias_instantiations_get"); 
    ALIAS_TYPE_INSTANTIATIONS.with(|map| {
        map.borrow_mut()
            .get(&alias_instantiations_slot(symbol))
            .and_then(|entries| entries.get(key))
            .cloned()
    })
}

pub(crate) fn alias_instantiations_insert(
    symbol: &Arc<Symbol>,
    key: CacheHashKey,
    instantiation: Arc<Type>,
) { ::tsox_core::fntrace::enter("alias_instantiations_insert"); 
    ALIAS_TYPE_INSTANTIATIONS.with(|map| {
        map.borrow_mut()
            .entry(alias_instantiations_slot(symbol))
            .or_default()
            .insert(key, instantiation);
    });
}
