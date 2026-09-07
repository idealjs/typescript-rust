#![allow(dead_code)]
use super::*;
use crate::ast::*;
use crate::core::compiler_options::ScriptTarget;
use crate::core::tristate::Tristate;
use crate::diagnostics::Message;
use std::sync::Arc;
mod name_resolver;
mod name_resolver_2;
mod name_resolver_3;
#[allow(unused_imports)]
pub use name_resolver::*;
#[allow(unused_imports)]
pub use name_resolver_2::*;
#[allow(unused_imports)]
pub use name_resolver_3::*;
