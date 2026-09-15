pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::sync::Arc;
pub(crate) use std::sync::atomic::{AtomicU32, Ordering};
pub(crate) use tsox_frontend::ast::Node;
pub(crate) use tsox_frontend::ast::SyntaxKind;
pub(crate) use tsox_frontend::scanner::CommentRange;
pub(crate) use tsox_frontend::scanner::CommentRangeKind;
pub(crate) mod generated_identifier_flags;
pub(crate) mod impl_chunk;
pub(crate) mod impl_chunk_3;
pub(crate) mod name_generator;
pub(crate) mod skip_white_space_single_line;
#[allow(unused_imports)]
pub use generated_identifier_flags::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use impl_chunk_3::*;
#[allow(unused_imports)]
pub use name_generator::*;
#[allow(unused_imports)]
pub use skip_white_space_single_line::*;
pub(crate) mod impl_chunk_3_name_generator;
pub(crate) mod impl_chunk_3_name_generator_2;
#[cfg(test)]
pub(crate) mod tests;
