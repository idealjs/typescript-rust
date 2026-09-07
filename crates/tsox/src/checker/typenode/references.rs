use super::*;
use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    CheckFlags, ModifierFlags, Node, NodeList, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::checker::checker::Checker;
use crate::jsnum;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
mod checker;
mod checker_2;
mod checker_3;
mod checker_4;
mod checker_5;
mod checker_6;
mod checker_7;
mod checker_8;
mod interface_instantiation_helpers;
mod interface_extends_check;
mod type_reference_resolution;
mod qualified_name_diagnostics;
mod interface_members_properties;
mod interface_members_accessors;
#[allow(unused_imports)]
pub use checker::*;
#[allow(unused_imports)]
pub use checker_2::*;
#[allow(unused_imports)]
pub use checker_3::*;
#[allow(unused_imports)]
pub use checker_4::*;
#[allow(unused_imports)]
pub use checker_5::*;
#[allow(unused_imports)]
pub use checker_6::*;
#[allow(unused_imports)]
pub use checker_7::*;
#[allow(unused_imports)]
pub use checker_8::*;
#[allow(unused_imports)]
pub use interface_instantiation_helpers::*;
#[allow(unused_imports)]
pub use interface_extends_check::*;
#[allow(unused_imports)]
pub use type_reference_resolution::*;
#[allow(unused_imports)]
pub use qualified_name_diagnostics::*;
#[allow(unused_imports)]
pub use interface_members_properties::*;
#[allow(unused_imports)]
pub use interface_members_accessors::*;
