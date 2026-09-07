use super::*;
use crate::ast::{ModifierFlags, Node, SymbolFlags, SyntaxKind};
use crate::checker::utilities::is_in_compound_like_assignment;
use crate::checker::utilities::{AssignmentKind, get_assignment_target_kind};
use std::sync::Arc;
mod checker;
mod checker_2;
#[allow(unused_imports)]
pub use checker::*;
#[allow(unused_imports)]
pub use checker_2::*;
