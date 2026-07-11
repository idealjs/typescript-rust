//! Abstract Syntax Tree types.
//!
//! Ported from `internal/ast/` in the Go implementation. The AST type
//! system is partly generated from `_scripts/ast.json` and partly
//! hand-written.

pub mod diagnostic;
pub mod node;
pub mod node_data_generated;
pub mod node_flags;
pub mod positionmap;
pub mod symbol;
pub mod syntax_kind_generated;

pub use diagnostic::*;
pub use node::*;
pub use node_data_generated::*;
pub use node_flags::*;
pub use symbol::*;
pub use syntax_kind_generated::SyntaxKind;
