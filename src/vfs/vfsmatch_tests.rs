//! Tests ported from `internal/vfs/vfsmatch/vfsmatch_test.go` (17 tests).
//!
//! All tests are marked `#[ignore]` because the core functions
//! (`match_files`/`ReadDirectory`, `SpecMatcher`, `is_implicit_glob`,
//! `compile_glob_pattern`, `get_base_paths`) from the Go `vfsmatch` package
//! have not yet been ported to Rust. The test descriptions preserve the Go
//! test scenarios so they can be filled in once the functionality is
//! implemented.

// ---------------------------------------------------------------------------
// Host factory functions — mirror the Go test helpers.
// These will be used once match_files is implemented.
// ---------------------------------------------------------------------------

/// Port of Go `caseInsensitiveHost` — simulates a Windows-like FS.
#[allow(dead_code)]
fn case_insensitive_host_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("/dev/a.ts", ""),
        ("/dev/a.d.ts", ""),
        ("/dev/a.js", ""),
        ("/dev/b.ts", ""),
        ("/dev/b.js", ""),
        ("/dev/c.d.ts", ""),
        ("/dev/z/a.ts", ""),
        ("/dev/z/abz.ts", ""),
        ("/dev/z/aba.ts", ""),
        ("/dev/z/b.ts", ""),
        ("/dev/z/bbz.ts", ""),
        ("/dev/z/bba.ts", ""),
        ("/dev/x/a.ts", ""),
        ("/dev/x/aa.ts", ""),
        ("/dev/x/b.ts", ""),
        ("/dev/x/y/a.ts", ""),
        ("/dev/x/y/b.ts", ""),
        ("/dev/js/a.js", ""),
        ("/dev/js/b.js", ""),
        ("/dev/js/d.min.js", ""),
        ("/dev/js/ab.min.js", ""),
        ("/ext/ext.ts", ""),
        ("/ext/b/a..b.ts", ""),
    ]
}

/// Port of Go `caseSensitiveHost` — simulates a Unix-like case-sensitive FS.
#[allow(dead_code)]
fn case_sensitive_host_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("/dev/a.ts", ""),
        ("/dev/a.d.ts", ""),
        ("/dev/a.js", ""),
        ("/dev/b.ts", ""),
        ("/dev/b.js", ""),
        ("/dev/A.ts", ""),
        ("/dev/B.ts", ""),
        ("/dev/c.d.ts", ""),
        ("/dev/z/a.ts", ""),
        ("/dev/z/abz.ts", ""),
        ("/dev/z/aba.ts", ""),
        ("/dev/z/b.ts", ""),
        ("/dev/z/bbz.ts", ""),
        ("/dev/z/bba.ts", ""),
        ("/dev/x/a.ts", ""),
        ("/dev/x/b.ts", ""),
        ("/dev/x/y/a.ts", ""),
        ("/dev/x/y/b.ts", ""),
        ("/dev/q/a/c/b/d.ts", ""),
        ("/dev/js/a.js", ""),
        ("/dev/js/b.js", ""),
        ("/dev/js/d.MIN.js", ""),
    ]
}

// ---------------------------------------------------------------------------
// TestReadDirectory — 47 sub-cases
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files (vfsmatch::ReadDirectory) not yet ported to Rust"]
#[test]
fn test_read_directory() {
    // Port of Go TestReadDirectory (47 sub-cases).
    // Tests match_files with various extensions, excludes, includes, depth,
    // and case sensitivity. Key scenarios:
    //  - defaults include common package folders (node_modules etc.)
    //  - literal includes with/without exclusions
    //  - wildcard excludes (*, ??, **)
    //  - recursive directory matching (**)
    //  - depth limits
    //  - mixed extensions
    //  - min.js exclusion
    //  - dotted folders
    // Go source: internal/vfs/vfsmatch/vfsmatch_test.go:TestReadDirectory
}

// ---------------------------------------------------------------------------
// TestReadDirectoryEdgeCases
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files (vfsmatch::ReadDirectory) not yet ported to Rust"]
#[test]
fn test_read_directory_edge_cases() {
    // Port of Go TestReadDirectoryEdgeCases (8 sub-cases).
    // Tests: rooted include paths, extension in path, special regex chars,
    // question mark / star prefixes, case-insensitive matching, nested
    // subdirectory base paths, differing current directory.
}

// ---------------------------------------------------------------------------
// TestReadDirectoryEmptyIncludes
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files (vfsmatch::ReadDirectory) not yet ported to Rust"]
#[test]
fn test_read_directory_empty_includes() {
    // Port of Go TestReadDirectoryEmptyIncludes.
    // Tests behavior when includes slice is empty.
}

// ---------------------------------------------------------------------------
// TestReadDirectorySymlinkCycle
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files + symlink support not yet ported to Rust"]
#[test]
fn test_read_directory_symlink_cycle() {
    // Port of Go TestReadDirectorySymlinkCycle.
    // Tests that cyclic symlinks don't cause infinite loops during traversal.
    // Requires both match_files and InMemoryFS symlink support.
}

// ---------------------------------------------------------------------------
// TestReadDirectoryMatchesTypeScriptBaselines
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files (vfsmatch::ReadDirectory) not yet ported to Rust"]
#[test]
fn test_read_directory_matches_typescript_baselines() {
    // Port of Go TestReadDirectoryMatchesTypeScriptBaselines (19 sub-cases).
    // Verifies Go implementation matches TypeScript baseline outputs from
    // tests/baselines/reference/config/matchFiles/.
}

// ---------------------------------------------------------------------------
// TestIsImplicitGlob
// ---------------------------------------------------------------------------

#[ignore = "TODO: is_implicit_glob not yet ported to Rust glob/vfs module"]
#[test]
fn test_is_implicit_glob() {
    // Port of Go TestIsImplicitGlob (10 sub-cases).
    // is_implicit_glob returns true when the last path component has no
    // extension and contains no glob characters (.*?).
    // Expected results:
    //   "foo"     -> true
    //   "src"     -> true
    //   "foo.ts"  -> false
    //   "foo."    -> false
    //   "*"       -> false
    //   "?"       -> false
    //   "foo*"    -> false
    //   "foo?"    -> false
    //   "foo.bar" -> false
    //   ""        -> true
}

// ---------------------------------------------------------------------------
// TestSpecMatcher (5 sub-cases)
// ---------------------------------------------------------------------------

#[ignore = "TODO: SpecMatcher not yet ported to Rust"]
#[test]
fn test_spec_matcher() {
    // Port of Go TestSpecMatcher (5 sub-cases).
    // Tests NewSpecMatcher with simple wildcard, recursive wildcard,
    // exclude pattern, case insensitive, and multiple specs.
}

// ---------------------------------------------------------------------------
// TestSpecMatcher_MatchString (3 sub-cases)
// ---------------------------------------------------------------------------

#[ignore = "TODO: SpecMatcher not yet ported to Rust"]
#[test]
fn test_spec_matcher_match_string() {
    // Port of Go TestSpecMatcher_MatchString.
    // Tests MatchString for simple wildcard, recursive wildcard, and
    // exclude pattern matching with expected bool results.
}

// ---------------------------------------------------------------------------
// TestSingleSpecMatcher_MatchString (2 sub-cases)
// ---------------------------------------------------------------------------

#[ignore = "TODO: SpecMatcher not yet ported to Rust"]
#[test]
fn test_single_spec_matcher_match_string() {
    // Port of Go TestSingleSpecMatcher_MatchString.
    // Tests single-spec wildcard and trailing ** exclude.
}

// ---------------------------------------------------------------------------
// TestSpecMatchers_MatchIndex (2 sub-cases)
// ---------------------------------------------------------------------------

#[ignore = "TODO: SpecMatcher not yet ported to Rust"]
#[test]
fn test_spec_matchers_match_index() {
    // Port of Go TestSpecMatchers_MatchIndex.
    // Tests MatchIndex returns the first matching spec index or -1.
}

// ---------------------------------------------------------------------------
// TestSingleSpecMatcher (3 sub-cases)
// ---------------------------------------------------------------------------

#[ignore = "TODO: SpecMatcher not yet ported to Rust"]
#[test]
fn test_single_spec_matcher() {
    // Port of Go TestSingleSpecMatcher.
    // Tests simple spec, trailing ** non-exclude returns nil,
    // and trailing ** exclude works.
}

// ---------------------------------------------------------------------------
// TestSpecMatchers (2 sub-cases)
// ---------------------------------------------------------------------------

#[ignore = "TODO: SpecMatcher not yet ported to Rust"]
#[test]
fn test_spec_matchers() {
    // Port of Go TestSpecMatchers.
    // Tests multiple specs return correct index and empty specs return nil.
}

// ---------------------------------------------------------------------------
// TestGlobPatternInternals
// ---------------------------------------------------------------------------

#[ignore = "TODO: compile_glob_pattern and internal glob helpers not yet ported to Rust"]
#[test]
fn test_glob_pattern_internals() {
    // Port of Go TestGlobPatternInternals (7 sub-cases).
    // Tests: nextPathPartParts (consecutive slashes, trailing slashes, empty
    // prefix, suffix region parsing), question mark at end of string, star
    // with complex pattern, ensureTrailingSlash, literal component with
    // package folder.
}

// ---------------------------------------------------------------------------
// TestMatchSegmentsEdgeCases
// ---------------------------------------------------------------------------

#[ignore = "TODO: compile_glob_pattern not yet ported to Rust"]
#[test]
fn test_match_segments_edge_cases() {
    // Port of Go TestMatchSegmentsEdgeCases (8 sub-cases).
    // Tests: question mark before slash, star with no trailing content,
    // multiple stars requiring backtracking (*a*a, *a*b*c, *a*a*a, a*b*a),
    // pathological pattern performance, literal segment not matching,
    // question mark / star matching multi-byte Unicode runes.
}

// ---------------------------------------------------------------------------
// TestReadDirectoryConsecutiveSlashes
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files (vfsmatch::ReadDirectory) not yet ported to Rust"]
#[test]
fn test_read_directory_consecutive_slashes() {
    // Port of Go TestReadDirectoryConsecutiveSlashes.
    // Tests handling of paths with consecutive slashes during match_files.
}

// ---------------------------------------------------------------------------
// TestGlobPatternLiteralWithPackageFolders
// ---------------------------------------------------------------------------

#[ignore = "TODO: match_files with package folder skip logic not yet ported to Rust"]
#[test]
fn test_glob_pattern_literal_with_package_folders() {
    // Port of Go TestGlobPatternLiteralWithPackageFolders (2 sub-cases).
    // Tests that wildcard patterns skip node_modules but explicit literal
    // includes do not.
}

// ---------------------------------------------------------------------------
// TestGetBasePathsCaseSensitivity
// ---------------------------------------------------------------------------

#[ignore = "TODO: get_base_paths not yet ported to Rust"]
#[test]
fn test_get_base_paths_case_sensitivity() {
    // Port of Go TestGetBasePathsCaseSensitivity (2 sub-cases).
    // Tests that case-sensitive FS does not dedup differently-cased paths,
    // while case-insensitive FS deduplicates them.
}
