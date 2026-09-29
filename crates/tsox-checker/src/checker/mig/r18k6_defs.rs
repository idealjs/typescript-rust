#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use std::sync::Arc;

use tsox_core::diagnostics::{Category, Message};

pub(crate) static PRIVATE_IDENTIFIERS_CANNOT_BE_USED_IN_DESTRUCTURING_PATTERNS: Message = Message {
    code: 18064,
    category: Category::Error,
    key: "Private_identifiers_cannot_be_used_in_destructuring_patterns_18064",
    text: "Private identifiers cannot be used in destructuring patterns.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};

#[allow(non_camel_case_types)]
pub(crate) struct InternalSymbolName;

#[allow(non_upper_case_globals)]
impl InternalSymbolName {
    pub(crate) const Missing: &'static str = tsox_frontend::ast::INTERNAL_SYMBOL_NAME_MISSING;
    pub(crate) const ModuleExports: &'static str =
        tsox_frontend::ast::INTERNAL_SYMBOL_NAME_MODULE_EXPORTS;
    pub(crate) const Default: &'static str = tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT;
    pub(crate) const AssignmentDeclaration: &'static str =
        tsox_frontend::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT;
}
