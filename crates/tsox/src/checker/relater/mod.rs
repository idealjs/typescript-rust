#![allow(dead_code)]

use super::types::*;

mod compare;
mod conditional;
mod index_signatures;
mod predicates;
mod probing;
mod relate;
mod relation;
mod type_arguments;
mod type_params;

pub use predicates::*;
pub use relation::RelationComparisonResult;
pub use relation::*;
pub(crate) use type_params::*;

#[cfg(test)]
mod tests;
