#![allow(dead_code)]
#![allow(unused_variables)]
use std::sync::Arc;
use crate::ast::{
    CheckFlags, Node, NodeData, SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::evaluator::EvalValue;
use super::checker::Checker;
use super::types::*;
use super::utilities::{
    get_property_name_from_type, is_literal_type, is_tuple_type, is_type_any,
    is_type_usable_as_property_name,
};
mod is_reserved_member_name_2;
mod checker_4;
mod checker_5;
mod checker_6;
mod checker_7;
#[allow(unused_imports)]
pub use is_reserved_member_name_2::*;
#[allow(unused_imports)]
pub use checker_4::*;
#[allow(unused_imports)]
pub use checker_5::*;
#[allow(unused_imports)]
pub use checker_6::*;
#[allow(unused_imports)]
pub use checker_7::*;
