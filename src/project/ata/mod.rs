//! Automatic Type Acquisition (ATA) (1:1 port of Go's `internal/project/ata/`).
//!
//! This module handles discovering and installing TypeScript type definitions
//! for JavaScript projects.

#![allow(dead_code)]

pub mod ata;
pub mod discover_typings;
pub mod types_map;
pub mod validate_package_name;
