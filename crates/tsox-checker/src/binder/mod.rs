pub(crate) mod binder;
#[allow(unused_imports)]
pub use binder::*;
pub(crate) mod container_flags;
pub(crate) mod flow_label;
pub(crate) mod helpers;
pub mod nameresolver;
pub mod referenceresolver;

pub(crate) use container_flags::*;
pub(crate) use flow_label::{ActiveLabel, FlowLabel};
pub(crate) use helpers::*;
pub(crate) use bind_js_assignment_declarations::{
    get_assignment_declaration_kind, expression_is_alias, is_module_exports_access_expression,
    is_exports_identifier,
};

pub(crate) use std::sync::Arc;
pub(crate) use tsox_core::diagnostics::messages_generated::A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION;
pub(crate) use tsox_core::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0;
pub(crate) use tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0;
pub(crate) use tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE;
pub(crate) use tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE;
pub(crate) use tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE;
pub(crate) use tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE;
pub(crate) use tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE;
pub(crate) use tsox_frontend::ast::*;
pub(crate) mod bind_js_assignment_declarations;
pub(crate) mod bind_walk;
pub(crate) mod bind_walk_bind_module_declaration;
pub(crate) mod bind_walk_bind_statement_kinds;
pub(crate) mod bind_walk_binder;
pub(crate) mod bind_walk_binder_2;
pub(crate) mod bind_walk_binder_3;
pub(crate) mod flow_bind;
pub(crate) mod flow_bind_binder;
pub(crate) mod flow_bind_binder_2;
pub(crate) mod flow_bind_binder_3;
pub(crate) mod flow_bind_binder_4;
pub(crate) mod flow_bind_binder_5;
pub(crate) mod flow_bind_binder_6;
pub(crate) mod nameresolver_get_local_symbol_for_export_default;
pub(crate) mod nameresolver_impl_chunk;
pub(crate) mod nameresolver_impl_chunk_name_resolver;
pub(crate) mod nameresolver_impl_chunk_name_resolver_2;
pub(crate) mod nameresolver_impl_chunk_name_resolver_3;
pub(crate) mod nameresolver_name_resolver;
pub(crate) mod referenceresolver_hooks;
pub(crate) mod referenceresolver_reference_resolver;
pub(crate) mod referenceresolver_resolver_impl;
pub(crate) mod symbols;
pub(crate) mod symbols_binder;
pub(crate) mod symbols_binder_2;
pub(crate) mod symbols_binder_3;
pub(crate) mod symbols_binder_4;
pub(crate) mod symbols_merge_symbol;
pub(crate) mod symbols_symbol_conflicts;
pub(crate) mod symbols_symbol_insertion;
#[cfg(test)]
pub(crate) mod tests;
