//! Language-service host interfaces (1:1 port of Go's `internal/ls/host.go`).
//!
//! The `Host` trait provides file-system access, converters, preferences,
//! and source-map info to the language service. The `Project` trait
//! (defined in `cross_project.rs`) represents an LS-side project.

#![allow(dead_code)]

use crate::ls::lsconv::converters::Converters;
use crate::ls::lsutil::UserPreferences;

/// Source-map line info for `.d.ts` files.
///
/// Mirrors Go's `*sourcemap.ECMALineInfo`. Not yet ported; stub.
pub struct EcmaLineInfo;

/// Auto-import registry (stub — Go's `*autoimport.Registry`).
pub struct AutoImportRegistry;

/// The host environment for the language service.
///
/// Mirrors Go's `ls.Host` interface.
pub trait Host: Send + Sync {
    fn use_case_sensitive_file_names(&self) -> bool;
    fn read_file(&self, path: &str) -> Option<String>;
    fn converters(&self) -> Converters;
    fn get_preferences(&self, active_file: &str) -> UserPreferences;
    fn get_ecma_line_info(&self, file_name: &str) -> Option<EcmaLineInfo>;
    fn auto_import_registry(&self) -> AutoImportRegistry;

    /// Used for module specifier completions.
    ///
    /// ! Do not use for anything else, as this violates the principle that
    /// the host is a snapshot-in-time.
    fn read_directory(
        &self,
        current_dir: &str,
        path: &str,
        extensions: &[String],
        excludes: &[String],
        includes: &[String],
        depth: i32,
    ) -> Vec<String>;
    fn get_directories(&self, path: &str) -> Vec<String>;
    fn directory_exists(&self, path: &str) -> bool;
    fn file_exists(&self, path: &str) -> bool;
}
