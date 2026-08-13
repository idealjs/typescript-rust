//! Shared test helpers for the baseline test runners.
//!
//! Ports the subset of tsgo's `internal/testrunner` / `internal/testutil`
//! machinery needed to execute TypeScript's official test cases (from the
//! `_submodules/TypeScript` git submodule) and compare their output against
//! baseline snapshots.

pub mod baseline;
pub mod case_parser;
