//! LSP protocol types (1:1 port of Go's `internal/lsp/lsproto/`).
//!
//! This module provides strongly-typed LSP protocol definitions matching
//! the generated types in Go's `lsp_generated.go`.

pub mod baseproto;
pub mod jsonrpc;
pub mod lsp;
pub mod util;

// Re-export all public types from lsp.rs so callers can use `lsproto::TypeName`.
pub use lsp::*;
