use super::*;
use crate::ast::{ModifierFlags, Node, NodeData, SymbolFlags, SyntaxKind};
use crate::checker::inference::{InferenceContext, InferenceInfo};
use crate::checker::utilities::{AssignmentKind, get_assignment_target_kind};
use crate::diagnostics::messages_generated::*;
use std::sync::Arc;
mod checker;
mod checker_2;
mod checker_3;
#[allow(unused_imports)]
pub use checker::*;
#[allow(unused_imports)]
pub use checker_2::*;
#[allow(unused_imports)]
pub use checker_3::*;
