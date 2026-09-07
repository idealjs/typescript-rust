use super::relater::RelaterChainEntry;
use super::tracer::Tracer;
use super::types::*;
use super::utilities::is_in_compound_like_assignment;
use super::utilities::{AssignmentKind, get_assignment_target_kind};
use crate::ast::{
    CheckFlags, DiagnosticsCollection, ModifierFlags, Node, NodeData, NodeFlags, NodeSymbolMap,
    SourceFile, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::evaluator::EvalResult;
use crate::jsnum;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
mod ast_get_combined_modifier_flags;
mod attach_explicit_type_arguments;
mod checker;
mod get_excluded_symbol_flags;
mod heritage_retry_limit;
mod impl_chunk;
mod impl_chunk_5;
mod impl_chunk_6;
mod module_alias_target_state;
mod object_literal_is_destructuring_target;
#[allow(unused_imports)]
pub use ast_get_combined_modifier_flags::*;
#[allow(unused_imports)]
pub use attach_explicit_type_arguments::*;
#[allow(unused_imports)]
pub use checker::*;
#[allow(unused_imports)]
pub use get_excluded_symbol_flags::*;
#[allow(unused_imports)]
pub use heritage_retry_limit::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use impl_chunk_5::*;
#[allow(unused_imports)]
pub use impl_chunk_6::*;
#[allow(unused_imports)]
pub use module_alias_target_state::*;
#[allow(unused_imports)]
pub use object_literal_is_destructuring_target::*;
mod assertions_interfaces;
mod assignment2;
mod calls;
mod classes;
mod contextual;
mod element_access;
mod enums;
mod expr_access;
mod expressions;
mod imports_namespace;
mod literals;
mod modules;
mod operators;
mod prop_access;
mod resolve;
mod statements;
mod suggestions_resolve;
mod symbol_types;
mod unused_diagnostics;
#[cfg(test)]
mod array_member_tests;
#[cfg(test)]
mod convergence_tests;
#[cfg(test)]
mod node_format_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod regression_fix_tests;
