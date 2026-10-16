#![allow(dead_code, unused_imports, unused_variables)]

use super::super::resolver_impl_chunk_3::Resolved;

pub fn should_continue_searching(resolved: &Option<Resolved>) -> bool { ::tsox_core::fntrace::enter("should_continue_searching"); 
    resolved.is_none()
}
