#![allow(dead_code)]

mod hooks;
mod reference_resolver;
mod resolver_impl;

pub use hooks::ReferenceResolverHooks;
pub use reference_resolver::ReferenceResolver;
pub use resolver_impl::{ReferenceResolverImpl, new_reference_resolver};
