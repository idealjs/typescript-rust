//! Watch options, ported from `internal/core/watchoptions.go`.
//!
//! Watch options are modeled as an independent axis from `CompilerOptions`
//! and `BuildOptions`, mirroring the Go port: a separate struct, a separate
//! option declarations list (`OPTIONS_FOR_WATCH` in `tsoptions`), a separate
//! name map (`find_watch_option`), and a separate parser pass. On the CLI,
//! watch flags (`--watchFile`, `--watchDirectory`, `--fallbackPolling`,
//! `--synchronousWatchDirectory`, `--watchInterval`, `--excludeDirectories`,
//! `--excludeFiles`) are accepted alongside compiler flags but routed into a
//! `WatchOptions` value held on `ParsedCommandLine` /
//! `ParsedBuildCommandLine`.
//!
//! Mirroring the current Go state, a `watchOptions` key inside `tsconfig.json`
//! is **not** parsed (the corresponding Go code paths in `tsconfigparsing.go`
//! are commented out). Only the CLI path is wired.

use crate::core::tristate::Tristate;

/// Mirrors `core.WatchFileKind` in Go (`internal/core/watchoptions.go:15-25`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum WatchFileKind {
    #[default]
    None = 0,
    FixedPollingInterval = 1,
    PriorityPollingInterval = 2,
    DynamicPriorityPolling = 3,
    FixedChunkSizePolling = 4,
    UseFsEvents = 5,
    UseFsEventsOnParentDirectory = 6,
}

/// Mirrors `core.WatchDirectoryKind` in Go (`internal/core/watchoptions.go:27-35`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum WatchDirectoryKind {
    #[default]
    None = 0,
    UseFsEvents = 1,
    FixedPollingInterval = 2,
    DynamicPriorityPolling = 3,
    FixedChunkSizePolling = 4,
}

/// Mirrors `core.PollingKind` in Go (`internal/core/watchoptions.go:37-45`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum PollingKind {
    #[default]
    None = 0,
    FixedInterval = 1,
    PriorityInterval = 2,
    DynamicPriority = 3,
    FixedChunkSize = 4,
}

/// Parses a `WatchFileKind` from its lowercase canonical name, mirroring the
/// keys of Go's `watchFileEnumMap` (`internal/tsoptions/enummaps.go`).
pub fn parse_watch_file_kind(s: &str) -> Option<WatchFileKind> {
    match s.to_lowercase().as_str() {
        "fixedpollinginterval" => Some(WatchFileKind::FixedPollingInterval),
        "prioritypollinginterval" => Some(WatchFileKind::PriorityPollingInterval),
        "dynamicprioritypolling" => Some(WatchFileKind::DynamicPriorityPolling),
        "fixedchunksizepolling" => Some(WatchFileKind::FixedChunkSizePolling),
        "usefsevents" => Some(WatchFileKind::UseFsEvents),
        "usefseventsonparentdirectory" => Some(WatchFileKind::UseFsEventsOnParentDirectory),
        _ => None,
    }
}

/// Parses a `WatchDirectoryKind` from its lowercase canonical name, mirroring
/// the keys of Go's `watchDirectoryEnumMap`.
pub fn parse_watch_directory_kind(s: &str) -> Option<WatchDirectoryKind> {
    match s.to_lowercase().as_str() {
        "usefsevents" => Some(WatchDirectoryKind::UseFsEvents),
        "fixedpollinginterval" => Some(WatchDirectoryKind::FixedPollingInterval),
        "dynamicprioritypolling" => Some(WatchDirectoryKind::DynamicPriorityPolling),
        "fixedchunksizepolling" => Some(WatchDirectoryKind::FixedChunkSizePolling),
        _ => None,
    }
}

/// Parses a `PollingKind` from its lowercase canonical name, mirroring the
/// keys of Go's `fallbackEnumMap`.
pub fn parse_polling_kind(s: &str) -> Option<PollingKind> {
    match s.to_lowercase().as_str() {
        "fixedinterval" => Some(PollingKind::FixedInterval),
        "priorityinterval" => Some(PollingKind::PriorityInterval),
        "dynamicpriority" => Some(PollingKind::DynamicPriority),
        "fixedchunksize" => Some(PollingKind::FixedChunkSize),
        _ => None,
    }
}

/// Watch options, mirroring `core.WatchOptions` in Go
/// (`internal/core/watchoptions.go:5-13`).
///
/// All fields are optional (`Option`/`Tristate`/`Vec`) to mirror Go's
/// pointer-based zero-value semantics. CLI parsing only populates the fields
/// that are explicitly provided; the JSON defaults declared in
/// `declswatch.go` (e.g. `UseFsEvents` for `watchFile`) are applied during
/// tsconfig.json ingestion, which is not yet implemented (matching Go).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WatchOptions {
    /// `watchInterval` — polling interval in milliseconds.
    pub interval: Option<i32>,
    /// `watchFile` — strategy for watching files.
    pub file_kind: WatchFileKind,
    /// `watchDirectory` — strategy for watching directories.
    pub directory_kind: WatchDirectoryKind,
    /// `fallbackPolling` — fallback when native file watchers run out.
    pub fallback_polling: PollingKind,
    /// `synchronousWatchDirectory` — synchronously call callbacks on platforms
    /// without recursive watching.
    pub sync_watch_dir: Tristate,
    /// `excludeDirectories` — directories removed from the watch process.
    pub exclude_dir: Vec<String>,
    /// `excludeFiles` — files removed from the watch mode's processing.
    pub exclude_files: Vec<String>,
}

impl WatchOptions {
    /// Returns true if no watch option was explicitly set. Mirrors a nil
    /// `*WatchOptions` check in Go.
    pub fn is_empty(&self) -> bool {
        self.interval.is_none()
            && self.file_kind == WatchFileKind::None
            && self.directory_kind == WatchDirectoryKind::None
            && self.fallback_polling == PollingKind::None
            && self.sync_watch_dir.is_unknown()
            && self.exclude_dir.is_empty()
            && self.exclude_files.is_empty()
    }

    /// Returns the watch interval as milliseconds, defaulting to 2000ms when
    /// unset. Mirrors `(*WatchOptions).WatchInterval()` in Go
    /// (`internal/core/watchoptions.go:47-52`).
    pub fn watch_interval_ms(&self) -> i32 {
        self.interval.unwrap_or(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_watch_file_kind_roundtrip() {
        assert_eq!(
            parse_watch_file_kind("UseFsEvents"),
            Some(WatchFileKind::UseFsEvents)
        );
        assert_eq!(
            parse_watch_file_kind("fixedpollinginterval"),
            Some(WatchFileKind::FixedPollingInterval)
        );
        assert_eq!(parse_watch_file_kind("bogus"), None);
    }

    #[test]
    fn parse_watch_directory_kind_roundtrip() {
        assert_eq!(
            parse_watch_directory_kind("UseFsEvents"),
            Some(WatchDirectoryKind::UseFsEvents)
        );
        assert_eq!(
            parse_watch_directory_kind("dynamicprioritypolling"),
            Some(WatchDirectoryKind::DynamicPriorityPolling)
        );
        assert_eq!(parse_watch_directory_kind("bogus"), None);
    }

    #[test]
    fn parse_polling_kind_roundtrip() {
        assert_eq!(
            parse_polling_kind("FixedInterval"),
            Some(PollingKind::FixedInterval)
        );
        assert_eq!(
            parse_polling_kind("priorityinterval"),
            Some(PollingKind::PriorityInterval)
        );
        assert_eq!(parse_polling_kind("bogus"), None);
    }

    #[test]
    fn default_is_empty_and_interval() {
        let w = WatchOptions::default();
        assert!(w.is_empty());
        assert_eq!(w.watch_interval_ms(), 2000);
    }

    #[test]
    fn non_default_is_not_empty() {
        let w = WatchOptions {
            interval: Some(100),
            ..WatchOptions::default()
        };
        assert!(!w.is_empty());
        assert_eq!(w.watch_interval_ms(), 100);
    }
}
