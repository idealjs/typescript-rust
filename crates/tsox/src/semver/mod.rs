use std::cmp::Ordering;
use std::fmt;
mod parse_comparator;
mod version;
mod version_range;
#[allow(unused_imports)]
pub use parse_comparator::*;
#[allow(unused_imports)]
pub use version::*;
#[allow(unused_imports)]
pub use version_range::*;
#[cfg(test)]
mod tests;
