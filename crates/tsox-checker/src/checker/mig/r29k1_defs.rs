#![allow(unused_imports)]
use crate::checker::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    static OBJECT_TYPE_INSTANTIATIONS: RefCell<HashMap<usize, HashMap<CacheHashKey, Arc<Type>>>> =
        RefCell::new(HashMap::new());
}

fn object_type_instantiations_slot(t: &Arc<Type>) -> usize { ::tsox_core::fntrace::enter("object_type_instantiations_slot"); 
    Arc::as_ptr(t) as *const () as usize
}

pub(crate) fn object_type_instantiations_is_empty(target: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("object_type_instantiations_is_empty"); 
    OBJECT_TYPE_INSTANTIATIONS.with(|map| {
        map.borrow()
            .get(&object_type_instantiations_slot(target))
            .map_or(true, HashMap::is_empty)
    })
}

pub(crate) fn object_type_instantiations_get(
    target: &Arc<Type>,
    key: &CacheHashKey,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("object_type_instantiations_get"); 
    OBJECT_TYPE_INSTANTIATIONS.with(|map| {
        map.borrow_mut()
            .get(&object_type_instantiations_slot(target))
            .and_then(|entries| entries.get(key))
            .cloned()
    })
}

pub(crate) fn object_type_instantiations_insert(
    target: &Arc<Type>,
    key: CacheHashKey,
    t: Arc<Type>,
) { ::tsox_core::fntrace::enter("object_type_instantiations_insert"); 
    OBJECT_TYPE_INSTANTIATIONS.with(|map| {
        map.borrow_mut()
            .entry(object_type_instantiations_slot(target))
            .or_default()
            .insert(key, t);
    });
}

thread_local! {
    static FLOW_NODE_POST_SUPER: RefCell<HashMap<usize, bool>> = RefCell::new(HashMap::new());
}

fn flow_node_post_super_slot(flow: &Arc<tsox_frontend::ast::FlowNode>) -> usize { ::tsox_core::fntrace::enter("flow_node_post_super_slot"); 
    Arc::as_ptr(flow) as *const () as usize
}

pub(crate) fn flow_node_post_super_get(
    flow: &Arc<tsox_frontend::ast::FlowNode>,
) -> Option<bool> { ::tsox_core::fntrace::enter("flow_node_post_super_get"); 
    FLOW_NODE_POST_SUPER.with(|map| map.borrow().get(&flow_node_post_super_slot(flow)).copied())
}

pub(crate) fn flow_node_post_super_insert(
    flow: Arc<tsox_frontend::ast::FlowNode>,
    post_super: bool,
) { ::tsox_core::fntrace::enter("flow_node_post_super_insert"); 
    FLOW_NODE_POST_SUPER.with(|map| {
        map.borrow_mut()
            .insert(flow_node_post_super_slot(&flow), post_super);
    });
}
