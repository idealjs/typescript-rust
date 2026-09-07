mod binder;
#[allow(unused_imports)]
pub use binder::*;
mod container_flags;
mod flow_label;
mod helpers;
pub mod nameresolver;
pub mod referenceresolver;

pub(crate) use container_flags::*;
pub(crate) use flow_label::{ActiveLabel, FlowLabel};
pub(crate) use helpers::*;

use crate::ast::*;
use crate::diagnostics::messages_generated::{
    A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
    CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0, DUPLICATE_IDENTIFIER_0,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
};
use std::sync::Arc;
mod bind_walk;
mod flow_bind;
mod symbols;
#[cfg(test)]
mod tests;
