//! Test ported from `internal/vfs/iovfs/iofs_test.go` (1 test).
//!
//! The Go `iovfs` package adapts Go's `io/fs.FS` interface (e.g.
//! `testing/fstest.MapFS`) to the `vfs.FS` interface. Rust has no equivalent
//! adapter, so this test is marked `#[ignore]`.

#[ignore = "TODO: IOFS adapter (wrapping std::fs / MapFS) not yet ported to Rust"]
#[test]
fn test_iofs() {
    // Port of Go TestIOFS.
    // Tests ReadFile, FileExists, DirectoryExists, GetAccessibleEntries,
    // WalkDir (with SkipDir), Realpath, and UseCaseSensitiveFileNames
    // on an iovfs.From(fstest.MapFS{...}, true) wrapper.
}
