#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use std::sync::Arc;

pub trait R19K9NodeExt {
    fn parent_rc(&self) -> Arc<Node>;
    fn property_name_or_name(&self) -> Arc<Node>;
}

impl R19K9NodeExt for Node {
    fn parent_rc(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("parent_rc"); 
        self.parent()
            .unwrap_or_else(|| panic!("nil parent for {:?}", self.kind))
    }

    fn property_name_or_name(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("property_name_or_name"); 
        tsox_frontend::ast::mig::m3b::property_name_or_name(self)
            .cloned()
            .unwrap_or_else(|| panic!("nil name for {:?}", self.kind))
    }
}
