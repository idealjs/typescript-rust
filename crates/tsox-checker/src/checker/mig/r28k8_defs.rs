#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::Node;

use crate::checker::nodecopy_builder::EmitContextStub;

pub trait EmitContextStubExt28 {
    fn set_emit_flags(&self, _node: &Arc<Node>, _flags: u32) { ::tsox_core::fntrace::enter("set_emit_flags"); }
}

impl EmitContextStubExt28 for EmitContextStub {}

thread_local! {
    static PACKAGES_MAPS: RefCell<HashMap<usize, HashMap<String, bool>>> =
        RefCell::new(HashMap::new());
}

fn checker_slot(c: &crate::checker::checker_checker_checker::Checker) -> usize { ::tsox_core::fntrace::enter("checker_slot"); 
    c as *const _ as usize
}

pub(crate) fn packages_map_get(
    c: &crate::checker::checker_checker_checker::Checker,
) -> Option<HashMap<String, bool>> { ::tsox_core::fntrace::enter("packages_map_get"); 
    PACKAGES_MAPS.with(|m| m.borrow().get(&checker_slot(c)).cloned())
}

pub(crate) fn packages_map_set(
    c: &crate::checker::checker_checker_checker::Checker,
    map: HashMap<String, bool>,
) { ::tsox_core::fntrace::enter("packages_map_set"); 
    PACKAGES_MAPS.with(|m| {
        m.borrow_mut().insert(checker_slot(c), map);
    });
}
