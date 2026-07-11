//! TypeScript compiler ported from Go to Rust.
//!
//! This is a port of microsoft/typescript-go (codename "Corsa") to Rust.
//! Module layout mirrors the Go `internal/` package structure.

pub mod ast;
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
pub mod glob;
pub mod jsnum;
pub mod json;
pub mod locale;
pub mod module;
pub mod packagejson;
pub mod parser;
pub mod printer;
pub mod scanner;
pub mod semver;
pub mod sourcemap;
pub mod stringutil;
pub mod tsoptions;
pub mod tspath;
pub mod vfs;
