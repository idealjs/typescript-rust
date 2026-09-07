#![allow(dead_code)]
use std::sync::Arc;
use crate::ast::{
    ModifierFlags, Node, NodeData, NodeFlags, SyntaxKind, is_class_declaration,
    is_class_expression, is_jsx_namespaced_name, is_module_block, is_source_file,
};
use crate::diagnostics::Message;
use crate::diagnostics::messages_generated::*;
use crate::scanner::token_to_string;
use super::checker::Checker;
mod checker_5;
mod checker_6;
mod checker_7;
mod is_this_parameter_2;
mod checker_8;
mod checker_9;
mod modifier_kind_checks_a;
mod modifier_kind_checks_b;
mod modifier_tail_checks;
#[allow(unused_imports)]
pub use checker_5::*;
#[allow(unused_imports)]
pub use checker_6::*;
#[allow(unused_imports)]
pub use checker_7::*;
#[allow(unused_imports)]
pub use is_this_parameter_2::*;
#[allow(unused_imports)]
pub use checker_8::*;
#[allow(unused_imports)]
pub use checker_9::*;
#[allow(unused_imports)]
pub use modifier_kind_checks_a::*;
#[allow(unused_imports)]
pub use modifier_kind_checks_b::*;
#[allow(unused_imports)]
pub use modifier_tail_checks::*;
