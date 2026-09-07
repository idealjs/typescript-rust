mod binary_precedence;
mod impl_chunk;
mod parsing_context;
#[allow(unused_imports)]
pub use binary_precedence::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use parsing_context::*;
mod jsdoc;
mod references;
mod reparser;

pub use jsdoc::parse_jsdoc_for_node;
pub use references::{collect_external_module_references, set_external_module_indicator};
pub use reparser::reparse_tags;

use crate::ast::*;
use crate::core::text::TextRange;
use crate::diagnostics::{self, Message};
use crate::scanner::{Scanner, token_to_string};
use std::sync::Arc;
mod declarations;
mod expressions;
mod jsx;
mod members;
mod statements;
mod types;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod batch1100_tests;
