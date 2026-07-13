//! The type checker.
//!
//! Ported from `internal/checker/` in the Go implementation. This is the
//! largest and most complex module in the compiler (~60K lines in Go).
//!
//! ## Structure
//!
//! - `types.rs` — Core type system: `TypeFlags`, `ObjectFlags`, `Type`,
//!   `Signature`, `IndexInfo`, and all supporting types.
//! - `checker.rs` — The `Checker` struct and its initialization.
//! - `mapper.rs` — Type mapper factory functions.
//! - `tracer.rs` — Tracing infrastructure for `--generateTrace`.
//!
//! ## Status
//!
//! The foundational type system and checker skeleton are ported. Full
//! type-checking logic (type inference, flow analysis, relation checking,
//! etc.) is added incrementally in future sessions.

pub mod checker;
pub mod mapper;
pub mod relater;
pub mod tracer;
pub mod typenode;
pub mod types;
pub mod utilities;

pub use checker::*;
pub use mapper::*;
pub use relater::*;
pub use tracer::*;
pub use types::*;
pub use utilities::*;
pub mod inference;

