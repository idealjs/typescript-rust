pub(crate) mod binary_precedence;
pub(crate) mod impl_chunk;
pub(crate) mod parsing_context;
#[allow(unused_imports)]
pub use binary_precedence::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use parsing_context::*;
pub(crate) mod jsdoc;
pub(crate) mod parser_json;
pub(crate) mod references;
pub(crate) mod reparse_await;
pub(crate) mod reparser;

pub use jsdoc::{parse_jsdoc_comment_range, parse_jsdoc_for_node};
pub use references::{collect_external_module_references, set_external_module_indicator};
pub use reparser::reparse_tags;

pub(crate) use crate::ast::*;
pub(crate) use crate::scanner::{Scanner, token_to_string};
pub(crate) use std::sync::Arc;
pub(crate) use tsox_core::core::text::TextRange;
pub(crate) use tsox_core::diagnostics;
pub(crate) use tsox_core::diagnostics::Message;
#[cfg(test)]
pub(crate) mod batch1100_tests;
pub(crate) mod declarations;
pub(crate) mod declarations_parser;
pub(crate) mod declarations_parser_2;
pub(crate) mod declarations_parser_3;
pub(crate) mod declarations_parser_4;
pub(crate) mod declarations_parser_5;
pub(crate) mod expressions;
pub(crate) mod expressions_parser;
pub(crate) mod expressions_parser_2;
pub(crate) mod expressions_parser_3;
pub(crate) mod expressions_parser_4;
pub(crate) mod expressions_parser_5;
pub(crate) mod expressions_parser_6;
pub(crate) mod expressions_parser_7;
pub(crate) mod impl_chunk_parser;
pub(crate) mod impl_chunk_parser_2;
pub(crate) mod impl_chunk_parser_3;
pub(crate) mod impl_chunk_parser_4;
pub(crate) mod impl_chunk_parser_5;
pub(crate) mod impl_chunk_parser_6;
pub(crate) mod jsdoc_j_s_doc_state;
pub(crate) mod jsdoc_parser;
pub(crate) mod jsdoc_parser_3;
pub(crate) mod jsdoc_parser_4;
pub(crate) mod jsdoc_parser_5;
pub(crate) mod jsdoc_parser_6;
pub(crate) mod jsdoc_parser_7;
pub(crate) mod jsdoc_parser_8;
pub(crate) mod jsdoc_parser_9;
#[cfg(test)]
pub(crate) mod jsdoc_tests;
pub(crate) mod jsdoc_tokens;
pub(crate) mod jsx;
pub(crate) mod members;
pub(crate) mod members_parser;
pub(crate) mod members_parser_2;
pub(crate) mod members_parser_3;
pub(crate) mod members_parser_4;
pub(crate) mod members_parser_5;
#[cfg(test)]
pub(crate) mod references_tests;
pub(crate) mod reparser_namespace;
pub(crate) mod reparser_signature;
pub(crate) mod reparser_tags;
#[cfg(test)]
pub(crate) mod reparser_tests;
pub(crate) mod reparser_type_literal;
pub(crate) mod reparser_type_parameters;
pub(crate) mod statements;
pub(crate) mod statements_parser;
pub(crate) mod statements_parser_2;
pub(crate) mod statements_parser_3;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod types;
pub(crate) mod types_parser;
pub(crate) mod types_parser_2;
pub(crate) mod types_parser_3;
pub(crate) mod types_parser_4;
