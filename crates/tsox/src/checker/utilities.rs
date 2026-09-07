#![allow(dead_code)]
use super::types::*;
use crate::ast::{
    ModifierFlags, Node, NodeFlags, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
    get_combined_modifier_flags, is_element_access_expression, is_property_access_expression,
    is_qualified_name, is_variable_declaration_list, is_variable_statement,
};
use std::sync::Arc;
mod get_assignment_target;
mod has_only_expression_initialization;
mod is_optional_symbol;
mod is_private_within_ambient;
mod token_is_identifier_or_keyword;
#[allow(unused_imports)]
pub use get_assignment_target::*;
#[allow(unused_imports)]
pub use has_only_expression_initialization::*;
#[allow(unused_imports)]
pub use is_optional_symbol::*;
#[allow(unused_imports)]
pub use is_private_within_ambient::*;
#[allow(unused_imports)]
pub use token_is_identifier_or_keyword::*;
#[cfg(test)]
mod tests;
