pub(crate) use crate::sourcemap::{Generator, SourceIndex};
pub(crate) use std::sync::Arc;
pub(crate) use tsox_core::core::compiler_options::CompilerOptions;
#[cfg(test)]
pub(crate) use tsox_core::core::compiler_options::JsxEmit;
pub(crate) use tsox_core::core::compiler_options::ModuleKind;
pub(crate) use tsox_core::tspath::ComparePathsOptions;
pub(crate) use tsox_frontend::ast::Node;
pub(crate) use tsox_frontend::ast::NodeList;
pub(crate) use tsox_frontend::ast::SourceFile;
pub(crate) use tsox_frontend::ast::SyntaxKind;
pub(crate) use tsox_frontend::ast::node_data_generated::NodeData;
pub(crate) use tsox_tsoptions::vfs::FS;
pub(crate) mod compute_common_source_directory;
pub(crate) mod emit_result;
pub(crate) mod fixup_jsx_text;
pub(crate) mod generate_fragment_call;
#[allow(unused_imports)]
pub use compute_common_source_directory::*;
#[allow(unused_imports)]
pub use emit_result::*;
#[allow(unused_imports)]
pub use fixup_jsx_text::*;
#[allow(unused_imports)]
pub use generate_fragment_call::*;
pub(crate) mod commonjs;
pub(crate) mod decl_emit;
pub(crate) mod sourcemap;
pub(crate) mod statement_emit;
pub(crate) mod text_ranges;
pub(crate) mod text_transform;
pub(crate) use commonjs::*;
pub(crate) use decl_emit::*;
pub(crate) use sourcemap::*;
pub(crate) use statement_emit::JsxRuntimeUsage;
pub(crate) use statement_emit::*;
pub(crate) mod commonjs_rewrite_import_extensions;
pub(crate) mod commonjs_transform_commonjs_import;
pub(crate) mod decl_emit_classify;
pub(crate) mod statement_emit_collect_import_clause_type_cuts;
pub(crate) mod statement_emit_collect_type_cuts;
pub(crate) mod statement_emit_emit_statement;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod text_ranges_comments;
pub(crate) mod text_ranges_es5;
pub(crate) mod text_transform_fold_tracked;
pub(crate) mod text_transform_fold_untracked;
pub(crate) mod text_transform_imports;
pub(crate) mod text_transform_reindent;
pub(crate) mod text_transform_semicolons;
