use crate::ast::{Node, SyntaxKind};
use crate::scanner::{CommentRange, CommentRangeKind};
use crate::stringutil;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
mod generated_identifier_flags;
mod impl_chunk;
mod impl_chunk_3;
mod name_generator;
mod skip_white_space_single_line;
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
#[cfg(test)]
mod tests;
