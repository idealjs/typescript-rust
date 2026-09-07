use crate::ast::node_data_generated::{ImportDeclarationData, NodeData};
use crate::ast::node_flags::ModifierFlags;
use crate::ast::{Node, SyntaxKind};
mod rewrite_import_extensions;
mod transform_commonjs_import;
#[allow(unused_imports)]
pub use rewrite_import_extensions::*;
#[allow(unused_imports)]
pub use transform_commonjs_import::*;
