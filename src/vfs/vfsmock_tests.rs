//! Test ported from `internal/vfs/vfsmock/wrapper_test.go` (1 test).
//!
//! The Go `vfsmock` package provides a counting/call-recording wrapper around
//! `vfs.FS` used by cachedvfs tests. Rust has no equivalent mock wrapper, so
//! this test is marked `#[ignore]`.

#[ignore = "TODO: mock/counting FS wrapper not yet ported to Rust"]
#[test]
fn test_wrap() {
    // Port of Go TestWrap.
    // Verifies that vfsmock.Wrap initializes all exported fields of the
    // wrapper struct (so that call-tracking fields are non-zero).
}
