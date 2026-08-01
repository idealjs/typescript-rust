//! Tests ported from `internal/vfs/cachedvfs/cachedvfs_test.go` (10 tests).
//!
//! All tests are marked `#[ignore]` because the `CachedFS` wrapper type from
//! the Go `cachedvfs` package has not yet been ported to Rust. The test
//! descriptions preserve the Go test scenarios (which verify caching behavior
//! by counting underlying FS calls via a mock) so they can be filled in
//! once a `CachedFS` and mock/counting FS are implemented.

// ---------------------------------------------------------------------------
// All 10 tests follow the same pattern:
// 1. Create a mock FS that records every call.
// 2. Wrap it in a CachedFS.
// 3. Verify that repeated calls hit the cache (call count stays the same).
// 4. Verify ClearCache / DisableAndClearCache / Enable behavior.
// ---------------------------------------------------------------------------

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_directory_exists() {
    // Port of Go TestDirectoryExists.
    // Verifies caching of directory_exists calls.
    // After first call: 1 underlying call.
    // After second identical call: still 1 (cached).
    // After ClearCache + call: 2 calls.
    // After DisableAndClearCache + 2 calls: 4 then 5 (not cached).
    // After Enable + call: 6, then second call still 6 (cached again).
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_file_exists() {
    // Port of Go TestFileExists.
    // Same caching pattern as test_cached_directory_exists for file_exists.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_get_accessible_entries() {
    // Port of Go TestGetAccessibleEntries.
    // Same caching pattern for get_accessible_entries.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_realpath() {
    // Port of Go TestRealpath.
    // Same caching pattern for realpath.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_stat() {
    // Port of Go TestStat.
    // Same caching pattern for stat.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_read_file() {
    // Port of Go TestReadFile.
    // ReadFile is NOT cached — each call increments the underlying count.
    // ClearCache / DisableAndClearCache do not affect read_file caching
    // because it is always passed through.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_use_case_sensitive_file_names() {
    // Port of Go TestUseCaseSensitiveFileNames.
    // use_case_sensitive_file_names is NOT cached — always passed through.
}

#[ignore = "TODO: CachedFS + WalkDir not yet ported to Rust"]
#[test]
fn test_cached_walk_dir() {
    // Port of Go TestWalkDir.
    // WalkDir is NOT cached — always passed through.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_remove() {
    // Port of Go TestRemove.
    // Remove is NOT cached — always passed through.
}

#[ignore = "TODO: CachedFS wrapper not yet ported to Rust"]
#[test]
fn test_cached_write_file() {
    // Port of Go TestWriteFile.
    // WriteFile is NOT cached — always passed through, and call args are
    // verifiable on the mock.
}
