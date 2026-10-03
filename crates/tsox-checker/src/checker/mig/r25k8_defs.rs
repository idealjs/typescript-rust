#![allow(unused_imports)]

use crate::checker::nodecopy_builder::PseudoCheckerStub;

pub fn new_pseudo_checker(
    _strict_null_checks: bool,
    _exact_optional_property_types: bool,
) -> PseudoCheckerStub { ::tsox_core::fntrace::enter("new_pseudo_checker"); 
    PseudoCheckerStub
}
