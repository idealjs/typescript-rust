#![allow(unused_imports)]

//! w9a: build orchestrator 余量收尾批(w9)

use crate::mig::m4x_build_3::Orchestrator;

impl Orchestrator {
    pub(crate) fn order(&self) -> &[String] { ::tsox_core::fntrace::enter("order"); 
        &self.order
    }
}
