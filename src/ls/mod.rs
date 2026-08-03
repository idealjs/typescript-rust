//! Language Service (1:1 port of Go's `internal/ls/`).
//!
//! LSP-agnostic layer between the LSP server and the compiler.

pub mod change;
pub mod lsconv;
pub mod lsutil;
