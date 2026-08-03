//! Change tracking (1:1 port of Go's `internal/ls/change/`).
//!
//! The `Tracker` accumulates text edits (insertions, replacements, deletions)
//! and produces `TextEdit` maps. This is a skeleton port: type definitions and
//! the public API surface are ported in full, while method bodies that depend
//! on the not-yet-ported printer/format/scanner/checker infrastructure are
//! stubbed (`todo!()`) and marked `// TODO`.

#![allow(dead_code)]

pub mod delete;
pub mod tracker;
pub mod tracker_impl;

pub use tracker::{
    DeletedNode, LeadingTriviaOption, NodeOptions, Tracker, TrailingTriviaOption, new_tracker,
};
