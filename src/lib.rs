//! TypeScript compiler ported from Go to Rust.
//!
//! This is a port of microsoft/typescript-go (codename "Corsa") to Rust.
//! Module layout mirrors the Go `internal/` package structure.

pub mod api;
pub mod ast;
pub mod astnav;
pub mod binder;
pub mod bundled;
pub mod checker;
pub mod collections;
pub mod compiler;
pub mod core;
pub mod debug;
pub mod diagnostics;
pub mod diagnosticwriter;
pub mod emitter;
pub mod evaluator;
pub mod execute;
pub mod format;
pub mod fourslash;
pub mod glob;
pub mod incremental;
pub mod jsnum;
pub mod json;
pub mod jsonrpc;
pub mod locale;
pub mod ls;
pub mod lsp;
pub mod module;
pub mod modulespecifiers;
pub mod nativepath;
pub mod packagejson;
pub mod parser;
pub mod printer;
pub mod project;
pub mod scanner;
pub mod semver;
pub mod sourcemap;
pub mod stringutil;
pub mod symlinks;
pub mod tracing;
pub mod tsoptions;
pub mod tspath;
pub mod vfs;
