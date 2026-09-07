use crate::ast::SyntaxKind;
use crate::core::compiler_options::ScriptTarget;
use std::collections::HashMap;
use std::sync::OnceLock;
mod error_callback;
mod impl_chunk;
mod is_conflict_marker_trivia;
mod is_jsx_line_break;
mod iterate_comment_ranges;
mod token_to_string;
#[allow(unused_imports)]
pub use error_callback::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use is_conflict_marker_trivia::*;
#[allow(unused_imports)]
pub use is_jsx_line_break::*;
#[allow(unused_imports)]
pub use iterate_comment_ranges::*;
#[allow(unused_imports)]
pub use token_to_string::*;
mod regexp;
mod unicode_properties;
#[cfg(test)]
mod tests;
