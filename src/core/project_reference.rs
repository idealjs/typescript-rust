//! Project reference type, ported from `internal/core/projectreference.go`.

/// A typed project reference, mirroring Go's `core.ProjectReference`.
///
/// Created during `tsconfig.json` parsing from the `references` array. `path`
/// is the normalized absolute path to the referenced config; `original_path`
/// is the raw string as written in the config; `circular` is set later by the
/// build orchestrator when a reference cycle is detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectReference {
    /// Normalized absolute path to the referenced tsconfig file or directory.
    pub path: String,
    /// The raw `path` string as written in `references[].path`.
    pub original_path: String,
    /// Whether this reference is part of a circular reference chain.
    pub circular: bool,
}
