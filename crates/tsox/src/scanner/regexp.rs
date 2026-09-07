mod reg_exp_flag_modifiers;
#[allow(unused_imports)]
pub use reg_exp_flag_modifiers::*;
mod class_ranges;
mod class_set;
mod class_set_operand;
mod cursor;
mod escapes;
mod pattern;
mod property_escape;

use crate::core::compiler_options::ScriptTarget;
use crate::diagnostics;
use crate::scanner::ScannerError;
use std::collections::HashSet;
