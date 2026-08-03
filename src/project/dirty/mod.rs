//! Dirty-tracking containers for snapshot cloning.
//! Port of Go's `internal/project/dirty/` package.
//!
//! Provides copy-on-write containers (`Box`, `Map`, `SyncMap`, `MapBuilder`)
//! that track which entries have been modified since a snapshot was taken.

pub mod box_;
pub mod map_;
pub mod map_builder;
