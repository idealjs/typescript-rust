use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, NodeList, SourceFile, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::core::compiler_options::ModuleKind;
#[cfg(test)]
use crate::core::compiler_options::{JsxEmit, ScriptTarget};
use crate::sourcemap::{Generator, SourceIndex};
use crate::tspath::{self, ComparePathsOptions};
use crate::vfs::FS;
use std::sync::Arc;
mod compute_common_source_directory;
mod emit_result;
mod fixup_jsx_text;
mod generate_fragment_call;
#[allow(unused_imports)]
pub use compute_common_source_directory::*;
#[allow(unused_imports)]
pub use emit_result::*;
#[allow(unused_imports)]
pub use fixup_jsx_text::*;
#[allow(unused_imports)]
pub use generate_fragment_call::*;
mod commonjs;
mod decl_emit;
mod sourcemap;
mod statement_emit;
mod text_ranges;
mod text_transform;
use commonjs::*;
use decl_emit::*;
use sourcemap::*;
use statement_emit::JsxRuntimeUsage;
use statement_emit::*;
use text_ranges::*;
use text_transform::*;
#[cfg(test)]
mod tests;
