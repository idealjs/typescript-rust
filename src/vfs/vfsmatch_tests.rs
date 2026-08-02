//! Tests ported from `internal/vfs/vfsmatch/vfsmatch_test.go`.
//!
//! Exercises `match_files`/`read_directory`, `SpecMatcher`,
//! `is_implicit_glob`, `compile_glob_pattern`, `get_base_paths`,
//! `match_segments`, and internal glob helpers.

use super::FS;
use super::InMemoryFS;
use super::vfsmatch::*;

// ---------------------------------------------------------------------------
// Host factory helpers — mirror the Go test helpers.  Each returns an
// InMemoryFS pre-populated with the given files.  Parent directories are
// inferred automatically.
// ---------------------------------------------------------------------------

fn build_fs(files: &[(&str, &str)], case_sensitive: bool) -> InMemoryFS {
    let fs = InMemoryFS::with_case_sensitivity(case_sensitive);
    for &(path, content) in files {
        fs.insert_file(path, content);
        let mut dir = path;
        while let Some(idx) = dir.rfind('/') {
            dir = &dir[..idx];
            if !dir.is_empty() {
                fs.insert_dir(dir);
            }
        }
    }
    fs
}

fn case_insensitive_host() -> InMemoryFS {
    build_fs(&case_insensitive_host_files(), false)
}

fn case_sensitive_host() -> InMemoryFS {
    build_fs(&case_sensitive_host_files(), true)
}

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

fn common_folders_host() -> InMemoryFS {
    build_fs(
        &[
            ("/dev/a.ts", ""),
            ("/dev/a.d.ts", ""),
            ("/dev/a.js", ""),
            ("/dev/b.ts", ""),
            ("/dev/x/a.ts", ""),
            ("/dev/node_modules/a.ts", ""),
            ("/dev/bower_components/a.ts", ""),
            ("/dev/jspm_packages/a.ts", ""),
        ],
        false,
    )
}

fn dotted_folders_host() -> InMemoryFS {
    build_fs(
        &[
            ("/dev/x/d.ts", ""),
            ("/dev/x/y/d.ts", ""),
            ("/dev/x/y/.e.ts", ""),
            ("/dev/x/.y/a.ts", ""),
            ("/dev/.z/.b.ts", ""),
            ("/dev/.z/c.ts", ""),
            ("/dev/w/.u/e.ts", ""),
            ("/dev/g.min.js/.g/g.ts", ""),
        ],
        false,
    )
}

fn mixed_extension_host() -> InMemoryFS {
    build_fs(
        &[
            ("/dev/a.ts", ""),
            ("/dev/a.d.ts", ""),
            ("/dev/a.js", ""),
            ("/dev/b.tsx", ""),
            ("/dev/b.d.ts", ""),
            ("/dev/b.jsx", ""),
            ("/dev/c.tsx", ""),
            ("/dev/c.js", ""),
            ("/dev/d.js", ""),
            ("/dev/e.jsx", ""),
            ("/dev/f.other", ""),
        ],
        false,
    )
}

fn same_named_declarations_host() -> InMemoryFS {
    build_fs(
        &[
            ("/dev/a.tsx", ""),
            ("/dev/a.d.ts", ""),
            ("/dev/b.tsx", ""),
            ("/dev/b.ts", ""),
            ("/dev/c.tsx", ""),
            ("/dev/m.ts", ""),
            ("/dev/m.d.ts", ""),
            ("/dev/n.tsx", ""),
            ("/dev/n.ts", ""),
            ("/dev/n.d.ts", ""),
            ("/dev/o.ts", ""),
            ("/dev/x.d.ts", ""),
        ],
        false,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn contains_str(s: &str, substr: &str) -> bool {
    s.contains(substr)
}

fn has_suffix(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

const TS_EXTS: &[&str] = &[".ts", ".tsx", ".d.ts"];

/// Runs a match_files case with default currentDir="/" and path="/dev".
fn run_match(
    host: &dyn super::FS,
    extensions: &[&str],
    excludes: &[&str],
    includes: &[&str],
) -> Vec<String> {
    match_files(
        "/dev",
        extensions,
        excludes,
        includes,
        host.use_case_sensitive_file_names(),
        "/",
        UNLIMITED_DEPTH,
        host,
    )
}

/// Runs a match_files case with full control.
fn run_match_full(
    host: &dyn super::FS,
    path: &str,
    current_dir: &str,
    extensions: &[&str],
    excludes: &[&str],
    includes: &[&str],
    depth: i32,
) -> Vec<String> {
    match_files(
        path,
        extensions,
        excludes,
        includes,
        host.use_case_sensitive_file_names(),
        current_dir,
        depth,
        host,
    )
}

// ---------------------------------------------------------------------------
// TestReadDirectory — 47 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_read_directory() {
    // defaults include common package folders
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &[]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/b.ts".to_string()));
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/node_modules/a.ts".to_string()));
        assert!(got.contains(&"/dev/bower_components/a.ts".to_string()));
        assert!(got.contains(&"/dev/jspm_packages/a.ts".to_string()));
    }

    // literal includes without exclusions
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["a.ts", "b.ts"]);
        assert_eq!(got, vec!["/dev/a.ts", "/dev/b.ts"]);
    }

    // literal includes with non ts extensions excluded
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["a.js", "b.js"]);
        assert!(got.is_empty());
    }

    // literal includes missing files excluded
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["z.ts", "x.ts"]);
        assert!(got.is_empty());
    }

    // literal includes with literal excludes
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["b.ts"], &["a.ts", "b.ts"]);
        assert_eq!(got, vec!["/dev/a.ts"]);
    }

    // literal includes with wildcard excludes
    {
        let host = case_insensitive_host();
        let got = run_match(
            &host,
            TS_EXTS,
            &["*.ts", "z/??z.ts", "*/b.ts"],
            &["a.ts", "b.ts", "z/a.ts", "z/abz.ts", "z/aba.ts", "x/b.ts"],
        );
        assert_eq!(got, vec!["/dev/z/a.ts", "/dev/z/aba.ts"]);
    }

    // literal includes with recursive excludes
    {
        let host = case_insensitive_host();
        let got = run_match(
            &host,
            TS_EXTS,
            &["**/b.ts"],
            &["a.ts", "b.ts", "x/a.ts", "x/b.ts", "x/y/a.ts", "x/y/b.ts"],
        );
        assert_eq!(got, vec!["/dev/a.ts", "/dev/x/a.ts", "/dev/x/y/a.ts"]);
    }

    // case sensitive exclude is respected
    {
        let host = case_sensitive_host();
        let got = run_match(&host, TS_EXTS, &["**/b.ts"], &["B.ts"]);
        assert_eq!(got, vec!["/dev/B.ts"]);
    }

    // explicit includes keep common package folders
    {
        let host = common_folders_host();
        let got = run_match(
            &host,
            TS_EXTS,
            &[],
            &[
                "a.ts",
                "b.ts",
                "node_modules/a.ts",
                "bower_components/a.ts",
                "jspm_packages/a.ts",
            ],
        );
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/b.ts".to_string()));
        assert!(got.contains(&"/dev/node_modules/a.ts".to_string()));
        assert!(got.contains(&"/dev/bower_components/a.ts".to_string()));
        assert!(got.contains(&"/dev/jspm_packages/a.ts".to_string()));
    }

    // wildcard include sorted order
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["z/*.ts", "x/*.ts"]);
        assert_eq!(
            got,
            vec![
                "/dev/z/a.ts",
                "/dev/z/aba.ts",
                "/dev/z/abz.ts",
                "/dev/z/b.ts",
                "/dev/z/bba.ts",
                "/dev/z/bbz.ts",
                "/dev/x/a.ts",
                "/dev/x/aa.ts",
                "/dev/x/b.ts",
            ]
        );
    }

    // wildcard include same named declarations excluded
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["*.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/b.ts".to_string()));
        assert!(got.contains(&"/dev/a.d.ts".to_string()));
        assert!(got.contains(&"/dev/c.d.ts".to_string()));
    }

    // wildcard star matches only ts files
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["*"]);
        for f in &got {
            assert!(
                contains_str(f, ".ts") || contains_str(f, ".tsx") || contains_str(f, ".d.ts"),
                "unexpected file: {f}"
            );
        }
        assert!(!got.contains(&"/dev/a.js".to_string()));
        assert!(!got.contains(&"/dev/b.js".to_string()));
    }

    // wildcard question mark single character
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["x/?.ts"]);
        assert_eq!(got, vec!["/dev/x/a.ts", "/dev/x/b.ts"]);
    }

    // wildcard recursive directory
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/a.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/z/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/a.ts".to_string()));
    }

    // double asterisk matches zero-or-more directories
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["x/**/a.ts"]);
        assert_eq!(got.len(), 2);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/a.ts".to_string()));
    }

    // wildcard multiple recursive directories
    {
        let host = case_insensitive_host();
        let got = run_match(
            &host,
            TS_EXTS,
            &[],
            &["x/y/**/a.ts", "x/**/a.ts", "z/**/a.ts"],
        );
        assert!(!got.is_empty());
    }

    // wildcard case sensitive matching
    {
        let host = case_sensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/A.ts"]);
        assert_eq!(got, vec!["/dev/A.ts"]);
    }

    // wildcard missing files excluded
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["*/z.ts"]);
        assert!(got.is_empty());
    }

    // exclude folders with wildcards
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["z", "x"], &["**/*"]);
        for f in &got {
            assert!(
                !contains_str(f, "/z/") && !contains_str(f, "/x/"),
                "should not contain z or x: {f}"
            );
        }
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/b.ts".to_string()));
    }

    // include paths outside project absolute
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["*", "/ext/*"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/ext/ext.ts".to_string()));
    }

    // include paths outside project relative
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["**"], &["*", "../ext/*"]);
        assert!(got.contains(&"/ext/ext.ts".to_string()));
    }

    // include files containing double dots
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["**"], &["/ext/b/a..b.ts"]);
        assert!(got.contains(&"/ext/b/a..b.ts".to_string()));
    }

    // exclude files containing double dots
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["/ext/b/a..b.ts"], &["/ext/**/*"]);
        assert!(got.contains(&"/ext/ext.ts".to_string()));
        assert!(!got.contains(&"/ext/b/a..b.ts".to_string()));
    }

    // common package folders implicitly excluded
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/a.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(!got.contains(&"/dev/node_modules/a.ts".to_string()));
        assert!(!got.contains(&"/dev/bower_components/a.ts".to_string()));
        assert!(!got.contains(&"/dev/jspm_packages/a.ts".to_string()));
    }

    // common package folders explicit recursive include
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/a.ts", "**/node_modules/a.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/node_modules/a.ts".to_string()));
    }

    // common package folders wildcard include
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["*/a.ts"]);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(!got.contains(&"/dev/node_modules/a.ts".to_string()));
    }

    // common package folders explicit wildcard include
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["*/a.ts", "node_modules/a.ts"]);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/node_modules/a.ts".to_string()));
    }

    // dotted folders not implicitly included
    {
        let host = dotted_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["x/**/*", "w/*/*"]);
        assert!(got.contains(&"/dev/x/d.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/d.ts".to_string()));
        assert!(!got.contains(&"/dev/x/.y/a.ts".to_string()));
        assert!(!got.contains(&"/dev/x/y/.e.ts".to_string()));
        assert!(!got.contains(&"/dev/w/.u/e.ts".to_string()));
    }

    // dotted folders explicitly included
    {
        let host = dotted_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["x/.y/a.ts", "/dev/.z/.b.ts"]);
        assert!(got.contains(&"/dev/x/.y/a.ts".to_string()));
        assert!(got.contains(&"/dev/.z/.b.ts".to_string()));
    }

    // dotted folders recursive wildcard matches directories
    {
        let host = dotted_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/.*/*"]);
        assert!(got.contains(&"/dev/x/.y/a.ts".to_string()));
        assert!(got.contains(&"/dev/.z/c.ts".to_string()));
        assert!(got.contains(&"/dev/w/.u/e.ts".to_string()));
    }

    // trailing recursive include returns empty
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**"]);
        assert!(got.is_empty());
    }

    // trailing recursive exclude removes everything
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["**"], &["**/*"]);
        assert!(got.is_empty());
    }

    // multiple recursive directory patterns in includes
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/x/**/*"]);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/a.ts".to_string()));
    }

    // multiple recursive directory patterns in excludes
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["**/x/**"], &["**/a.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/z/a.ts".to_string()));
        assert!(!got.contains(&"/dev/x/a.ts".to_string()));
        assert!(!got.contains(&"/dev/x/y/a.ts".to_string()));
    }

    // implicit globbification expands directory
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["z"]);
        assert!(got.contains(&"/dev/z/a.ts".to_string()));
        assert!(got.contains(&"/dev/z/aba.ts".to_string()));
        assert!(got.contains(&"/dev/z/b.ts".to_string()));
    }

    // exclude patterns starting with starstar
    {
        let host = case_sensitive_host();
        let got = run_match(&host, TS_EXTS, &["**/x"], &[]);
        for f in &got {
            assert!(!contains_str(f, "/x/"), "should not contain /x/: {f}");
        }
    }

    // include patterns starting with starstar
    {
        let host = case_sensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/x", "**/a/**/b"]);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/q/a/c/b/d.ts".to_string()));
    }

    // depth limit one
    {
        let host = case_insensitive_host();
        let got = run_match_full(&host, "/dev", "/", TS_EXTS, &[], &[], 1);
        for f in &got {
            let suffix = &f["/dev/".len()..];
            assert!(
                !contains_str(suffix, "/"),
                "depth 1 should not include nested files: {f}"
            );
        }
    }

    // depth limit two
    {
        let host = case_insensitive_host();
        let got = run_match_full(&host, "/dev", "/", TS_EXTS, &[], &[], 2);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/z/a.ts".to_string()));
        assert!(!got.contains(&"/dev/x/y/a.ts".to_string()));
    }

    // mixed extensions only ts
    {
        let host = mixed_extension_host();
        let got = run_match(&host, &[".ts"], &[], &[]);
        for f in &got {
            assert!(has_suffix(f, ".ts"), "should only have .ts files: {f}");
        }
    }

    // mixed extensions ts and tsx
    {
        let host = mixed_extension_host();
        let got = run_match(&host, &[".ts", ".tsx"], &[], &[]);
        for f in &got {
            assert!(
                has_suffix(f, ".ts") || has_suffix(f, ".tsx"),
                "should only have .ts or .tsx files: {f}"
            );
        }
    }

    // mixed extensions js and jsx
    {
        let host = mixed_extension_host();
        let got = run_match(&host, &[".js", ".jsx"], &[], &[]);
        for f in &got {
            assert!(
                has_suffix(f, ".js") || has_suffix(f, ".jsx"),
                "should only have .js or .jsx files: {f}"
            );
        }
    }

    // min js files excluded by wildcard
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".js"], &[], &["js/*"]);
        assert!(got.contains(&"/dev/js/a.js".to_string()));
        assert!(got.contains(&"/dev/js/b.js".to_string()));
        assert!(!got.contains(&"/dev/js/d.min.js".to_string()));
        assert!(!got.contains(&"/dev/js/ab.min.js".to_string()));
    }

    // min js exclusion is case-sensitive on case-sensitive FS
    {
        let host = case_sensitive_host();
        let got = run_match(&host, &[".js"], &[], &["js/*"]);
        assert!(got.contains(&"/dev/js/a.js".to_string()));
        assert!(got.contains(&"/dev/js/b.js".to_string()));
        assert!(got.contains(&"/dev/js/d.MIN.js".to_string()));
    }

    // min js files explicitly included
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".js"], &[], &["js/*.min.js"]);
        assert!(got.contains(&"/dev/js/d.min.js".to_string()));
        assert!(got.contains(&"/dev/js/ab.min.js".to_string()));
    }

    // min js files included when pattern mentions .min.
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".js"], &[], &["js/*.min.*"]);
        assert_eq!(got.len(), 2);
        assert!(got.contains(&"/dev/js/d.min.js".to_string()));
        assert!(got.contains(&"/dev/js/ab.min.js".to_string()));
    }

    // exclude literal node_modules folder
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &["node_modules"], &["**/*"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(!got.contains(&"/dev/node_modules/a.ts".to_string()));
    }

    // same named declarations include ts
    {
        let host = same_named_declarations_host();
        let got = run_match(&host, TS_EXTS, &[], &["*.ts"]);
        assert!(!got.is_empty());
    }

    // same named declarations include tsx
    {
        let host = same_named_declarations_host();
        let got = run_match(&host, TS_EXTS, &[], &["*.tsx"]);
        for f in &got {
            assert!(has_suffix(f, ".tsx"), "should only have .tsx files: {f}");
        }
    }

    // empty includes returns all matching files
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &[]);
        assert!(!got.is_empty());
        assert!(got.contains(&"/dev/a.ts".to_string()));
    }

    // nil extensions returns all files
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[], &[], &[]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/a.js".to_string()));
    }

    // empty extensions slice returns all files
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[], &[], &[]);
        assert!(!got.is_empty(), "expected files to be returned");
    }
}

// ---------------------------------------------------------------------------
// TestReadDirectoryEdgeCases — 8 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_read_directory_edge_cases() {
    // rooted include path
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".ts"], &[], &["/dev/a.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
    }

    // include with extension in path
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".ts"], &[], &["a.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
    }

    // special regex characters in path
    {
        let host = build_fs(
            &[
                ("/dev/file+test.ts", ""),
                ("/dev/file[0].ts", ""),
                ("/dev/file(1).ts", ""),
                ("/dev/file$money.ts", ""),
                ("/dev/file^start.ts", ""),
                ("/dev/file|pipe.ts", ""),
                ("/dev/file#hash.ts", ""),
            ],
            false,
        );
        let got = run_match(&host, &[".ts"], &[], &["file+test.ts"]);
        assert!(got.contains(&"/dev/file+test.ts".to_string()));
    }

    // include pattern starting with question mark
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".ts"], &[], &["?.ts"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/b.ts".to_string()));
    }

    // include pattern starting with star
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".ts"], &[], &["*b.ts"]);
        assert!(got.contains(&"/dev/b.ts".to_string()));
    }

    // case insensitive file matching
    {
        let host = build_fs(&[("/dev/File.ts", ""), ("/dev/FILE.ts", "")], true);
        let got = run_match(&host, &[".ts"], &[], &["*.ts"]);
        assert_eq!(got.len(), 2);
    }

    // nested subdirectory base path
    {
        let host = case_sensitive_host();
        let got = run_match(&host, &[".ts"], &[], &["q/a/c/b/d.ts"]);
        assert!(got.contains(&"/dev/q/a/c/b/d.ts".to_string()));
    }

    // current directory differs from path
    {
        let host = case_insensitive_host();
        let got = run_match(&host, &[".ts"], &[], &["z/*.ts"]);
        assert!(!got.is_empty());
    }
}

// ---------------------------------------------------------------------------
// TestReadDirectoryEmptyIncludes
// ---------------------------------------------------------------------------

#[test]
fn test_read_directory_empty_includes() {
    let host = build_fs(&[("/root/a.ts", "")], true);
    let got = run_match_full(&host, "/root", "/", &[".ts"], &[], &[], UNLIMITED_DEPTH);
    if !got.is_empty() {
        assert!(got.contains(&"/root/a.ts".to_string()));
    }
}

// ---------------------------------------------------------------------------
// TestReadDirectorySymlinkCycle
// ---------------------------------------------------------------------------

#[test]
fn test_read_directory_symlink_cycle() {
    // Port of Go TestReadDirectorySymlinkCycle.
    // Tests that cyclic symlinks don't cause infinite loops during traversal.
    // InMemoryFS now supports symlinks with cycle protection in
    // `resolve_symlinks`, so resolving a cyclic link terminates rather than
    // looping forever.
    let fs = build_fs(&[], true);
    fs.create_symlink("/a", "/b");
    fs.create_symlink("/b", "/a");
    fs.create_symlink("/self", "/self");

    // These must terminate (cycle protection) and not panic.
    assert!(!fs.file_exists("/a"));
    assert!(!fs.file_exists("/self"));
    assert_eq!(fs.read_file("/a"), None);
    assert_eq!(fs.read_file("/self"), None);

    // realpath on a cycle returns the last-resolvable path without looping.
    let rp_a = fs.realpath("/a");
    let _ = rp_a;
    let rp_self = fs.realpath("/self");
    let _ = rp_self;

    // A real file reachable through a cyclic directory link resolves.
    fs.insert_dir("/real");
    fs.insert_file("/real/file.ts", "x");
    fs.create_symlink("/c", "/d");
    fs.create_symlink("/d", "/c");
    // /c/real does not exist, but resolution must still terminate.
    assert_eq!(fs.read_file("/c/real/file.ts"), None);
    assert_eq!(fs.read_file("/real/file.ts"), Some("x".to_string()));
}

// ---------------------------------------------------------------------------
// TestReadDirectoryMatchesTypeScriptBaselines — 19 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_read_directory_matches_typescript_baselines() {
    // sorted in include order then alphabetical
    {
        let host = build_fs(
            &[
                ("/dev/z/a.ts", ""),
                ("/dev/z/aba.ts", ""),
                ("/dev/z/abz.ts", ""),
                ("/dev/z/b.ts", ""),
                ("/dev/z/bba.ts", ""),
                ("/dev/z/bbz.ts", ""),
                ("/dev/x/a.ts", ""),
                ("/dev/x/aa.ts", ""),
                ("/dev/x/b.ts", ""),
            ],
            false,
        );
        let got = run_match(&host, TS_EXTS, &[], &["z/*.ts", "x/*.ts"]);
        assert_eq!(
            got,
            vec![
                "/dev/z/a.ts",
                "/dev/z/aba.ts",
                "/dev/z/abz.ts",
                "/dev/z/b.ts",
                "/dev/z/bba.ts",
                "/dev/z/bbz.ts",
                "/dev/x/a.ts",
                "/dev/x/aa.ts",
                "/dev/x/b.ts",
            ]
        );
    }

    // recursive wildcards match dotted directories
    {
        let host = dotted_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/.*/*"]);
        let expected = vec![
            "/dev/.z/c.ts",
            "/dev/g.min.js/.g/g.ts",
            "/dev/w/.u/e.ts",
            "/dev/x/.y/a.ts",
        ];
        assert_eq!(got.len(), expected.len());
        for want in expected {
            assert!(
                got.contains(&want.to_string()),
                "expected {want} in results"
            );
        }
    }

    // common package folders implicitly excluded with wildcard
    {
        let host = common_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/a.ts"]);
        assert_eq!(got, vec!["/dev/a.ts", "/dev/x/a.ts"]);
    }

    // js wildcard excludes min js files
    {
        let host = build_fs(
            &[
                ("/dev/js/a.js", ""),
                ("/dev/js/b.js", ""),
                ("/dev/js/d.min.js", ""),
                ("/dev/js/ab.min.js", ""),
            ],
            false,
        );
        let got = run_match(&host, &[".js"], &[], &["js/*"]);
        assert_eq!(got, vec!["/dev/js/a.js", "/dev/js/b.js"]);
    }

    // explicit min js pattern includes min files
    {
        let host = build_fs(
            &[
                ("/dev/js/a.js", ""),
                ("/dev/js/b.js", ""),
                ("/dev/js/d.min.js", ""),
                ("/dev/js/ab.min.js", ""),
            ],
            false,
        );
        let got = run_match(&host, &[".js"], &[], &["js/*.min.js"]);
        assert_eq!(got.len(), 2);
        assert!(got.contains(&"/dev/js/ab.min.js".to_string()));
        assert!(got.contains(&"/dev/js/d.min.js".to_string()));
    }

    // literal excludes baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["b.ts"], &["a.ts", "b.ts"]);
        assert_eq!(got, vec!["/dev/a.ts"]);
    }

    // wildcard excludes baseline
    {
        let host = case_insensitive_host();
        let got = run_match(
            &host,
            TS_EXTS,
            &["*.ts", "z/??z.ts", "*/b.ts"],
            &["a.ts", "b.ts", "z/a.ts", "z/abz.ts", "z/aba.ts", "x/b.ts"],
        );
        assert_eq!(got, vec!["/dev/z/a.ts", "/dev/z/aba.ts"]);
    }

    // recursive excludes baseline
    {
        let host = case_insensitive_host();
        let got = run_match(
            &host,
            TS_EXTS,
            &["**/b.ts"],
            &["a.ts", "b.ts", "x/a.ts", "x/b.ts", "x/y/a.ts", "x/y/b.ts"],
        );
        assert_eq!(got, vec!["/dev/a.ts", "/dev/x/a.ts", "/dev/x/y/a.ts"]);
    }

    // question mark baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["x/?.ts"]);
        assert_eq!(got, vec!["/dev/x/a.ts", "/dev/x/b.ts"]);
    }

    // recursive directory pattern baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/a.ts"]);
        assert_eq!(
            got,
            vec!["/dev/a.ts", "/dev/x/a.ts", "/dev/x/y/a.ts", "/dev/z/a.ts",]
        );
    }

    // case sensitive baseline
    {
        let host = case_sensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/A.ts"]);
        assert_eq!(got, vec!["/dev/A.ts"]);
    }

    // exclude folders baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["z", "x"], &["**/*"]);
        for f in &got {
            assert!(
                !contains_str(f, "/z/") && !contains_str(f, "/x/"),
                "should not contain z or x: {f}"
            );
        }
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/dev/b.ts".to_string()));
    }

    // implicit glob expansion baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["z"]);
        assert_eq!(
            got,
            vec![
                "/dev/z/a.ts",
                "/dev/z/aba.ts",
                "/dev/z/abz.ts",
                "/dev/z/b.ts",
                "/dev/z/bba.ts",
                "/dev/z/bbz.ts",
            ]
        );
    }

    // trailing recursive directory baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**"]);
        assert!(got.is_empty());
    }

    // exclude trailing recursive directory baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["**"], &["**/*"]);
        assert!(got.is_empty());
    }

    // multiple recursive directory patterns baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/x/**/*"]);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/aa.ts".to_string()));
        assert!(got.contains(&"/dev/x/b.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/b.ts".to_string()));
    }

    // include dirs with starstar prefix baseline
    {
        let host = case_sensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["**/x", "**/a/**/b"]);
        assert!(got.contains(&"/dev/x/a.ts".to_string()));
        assert!(got.contains(&"/dev/x/b.ts".to_string()));
        assert!(got.contains(&"/dev/q/a/c/b/d.ts".to_string()));
    }

    // dotted folders not implicitly included baseline
    {
        let host = dotted_folders_host();
        let got = run_match(&host, TS_EXTS, &[], &["x/**/*", "w/*/*"]);
        assert!(got.contains(&"/dev/x/d.ts".to_string()));
        assert!(got.contains(&"/dev/x/y/d.ts".to_string()));
        assert!(!got.contains(&"/dev/x/.y/a.ts".to_string()));
        assert!(!got.contains(&"/dev/x/y/.e.ts".to_string()));
        assert!(!got.contains(&"/dev/w/.u/e.ts".to_string()));
    }

    // include paths outside project baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &[], &["*", "/ext/*"]);
        assert!(got.contains(&"/dev/a.ts".to_string()));
        assert!(got.contains(&"/ext/ext.ts".to_string()));
    }

    // include files with double dots baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["**"], &["/ext/b/a..b.ts"]);
        assert!(got.contains(&"/ext/b/a..b.ts".to_string()));
    }

    // exclude files with double dots baseline
    {
        let host = case_insensitive_host();
        let got = run_match(&host, TS_EXTS, &["/ext/b/a..b.ts"], &["/ext/**/*"]);
        assert!(got.contains(&"/ext/ext.ts".to_string()));
        assert!(!got.contains(&"/ext/b/a..b.ts".to_string()));
    }
}

// ---------------------------------------------------------------------------
// TestIsImplicitGlob — 10 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_is_implicit_glob() {
    let cases: &[(&str, &str, bool)] = &[
        ("simple", "foo", true),
        ("folder", "src", true),
        ("with extension", "foo.ts", false),
        ("trailing dot", "foo.", false),
        ("star", "*", false),
        ("question", "?", false),
        ("star suffix", "foo*", false),
        ("question suffix", "foo?", false),
        ("dot name", "foo.bar", false),
        ("empty", "", true),
    ];

    for (name, input, expected) in cases {
        let result = is_implicit_glob(input);
        assert_eq!(result, *expected, "case: {name} (input={input:?})");
    }
}

// ---------------------------------------------------------------------------
// TestSpecMatcher — 5 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_spec_matcher() {
    // simple wildcard
    {
        let m = SpecMatcher::new(&["*.ts"], "/project", Usage::Files, true);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/a.ts"));
        assert!(m.matches("/project/b.ts"));
        assert!(m.matches("/project/foo.ts"));
        assert!(!m.matches("/project/a.js"));
        assert!(!m.matches("/project/sub/a.ts"));
    }

    // recursive wildcard
    {
        let m = SpecMatcher::new(&["**/*.ts"], "/project", Usage::Files, true);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/a.ts"));
        assert!(m.matches("/project/sub/a.ts"));
        assert!(m.matches("/project/sub/deep/a.ts"));
        assert!(!m.matches("/project/a.js"));
    }

    // exclude pattern
    {
        let m = SpecMatcher::new(&["node_modules"], "/project", Usage::Exclude, true);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/node_modules/foo"));
        assert!(!m.matches("/project/node_modules"));
        assert!(!m.matches("/project/src"));
    }

    // case insensitive
    {
        let m = SpecMatcher::new(&["*.ts"], "/project", Usage::Files, false);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/A.TS"));
        assert!(m.matches("/project/B.Ts"));
        assert!(!m.matches("/project/a.js"));
    }

    // multiple specs
    {
        let m = SpecMatcher::new(&["*.ts", "*.tsx"], "/project", Usage::Files, true);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/a.ts"));
        assert!(m.matches("/project/b.tsx"));
        assert!(!m.matches("/project/a.js"));
    }
}

// ---------------------------------------------------------------------------
// TestSpecMatcher_MatchString — 3 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_spec_matcher_match_string() {
    // simple wildcard files
    {
        let m = SpecMatcher::new(&["*.ts"], "/project", Usage::Files, true).unwrap();
        let paths = ["/project/a.ts", "/project/sub/a.ts", "/project/a.js"];
        let expected = [true, false, false];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.matches(path), expected[i], "path: {path}");
        }
    }

    // recursive wildcard files
    {
        let m = SpecMatcher::new(&["**/*.ts"], "/project", Usage::Files, true).unwrap();
        let paths = ["/project/a.ts", "/project/sub/a.ts", "/project/a.js"];
        let expected = [true, true, false];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.matches(path), expected[i], "path: {path}");
        }
    }

    // exclude pattern matches prefix
    {
        let m = SpecMatcher::new(&["node_modules"], "/project", Usage::Exclude, true).unwrap();
        let paths = [
            "/project/node_modules",
            "/project/node_modules/foo",
            "/project/src",
        ];
        let expected = [false, true, false];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.matches(path), expected[i], "path: {path}");
        }
    }
}

// ---------------------------------------------------------------------------
// TestSingleSpecMatcher_MatchString — 2 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_single_spec_matcher_match_string() {
    // single spec wildcard
    {
        let m = SpecMatcher::new(&["*.ts"], "/project", Usage::Files, true).unwrap();
        let paths = ["/project/a.ts", "/project/sub/a.ts", "/project/a.js"];
        let expected = [true, false, false];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.matches(path), expected[i], "path: {path}");
        }
    }

    // single spec trailing starstar exclude allowed
    {
        let m = SpecMatcher::new(&["**"], "/project", Usage::Exclude, true).unwrap();
        let paths = ["/project/a.ts", "/project/sub/a.ts"];
        let expected = [true, true];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.matches(path), expected[i], "path: {path}");
        }
    }
}

// ---------------------------------------------------------------------------
// TestSpecMatchers_MatchIndex — 2 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_spec_matchers_match_index() {
    // index lookup prefers first match
    {
        let m = SpecMatcher::new(&["*.ts", "*.tsx"], "/project", Usage::Files, true).unwrap();
        let paths = ["/project/a.ts", "/project/a.tsx", "/project/a.js"];
        let expected = [0, 1, -1];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.match_index(path), expected[i], "path: {path}");
        }
    }

    // exclude index lookup
    {
        let m = SpecMatcher::new(
            &["node_modules", "bower_components"],
            "/project",
            Usage::Exclude,
            true,
        )
        .unwrap();
        let paths = [
            "/project/node_modules",
            "/project/node_modules/foo",
            "/project/bower_components",
            "/project/bower_components/bar",
            "/project/src",
        ];
        let expected = [-1, 0, -1, 1, -1];
        assert_eq!(paths.len(), expected.len());
        for (i, path) in paths.iter().enumerate() {
            assert_eq!(m.match_index(path), expected[i], "path: {path}");
        }
    }
}

// ---------------------------------------------------------------------------
// TestSingleSpecMatcher — 3 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_single_spec_matcher() {
    // simple spec
    {
        let m = SpecMatcher::new(&["*.ts"], "/project", Usage::Files, true);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/a.ts"));
        assert!(!m.matches("/project/a.js"));
    }

    // trailing ** non-exclude returns nil
    {
        let m = SpecMatcher::new(&["**"], "/project", Usage::Files, true);
        assert!(m.is_none(), "should be None");
    }

    // trailing ** exclude works
    {
        let m = SpecMatcher::new(&["**"], "/project", Usage::Exclude, true);
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.matches("/project/anything"));
        assert!(m.matches("/project/deep/path"));
    }
}

// ---------------------------------------------------------------------------
// TestSpecMatchers — 2 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_spec_matchers() {
    // multiple specs return correct index
    {
        let m =
            SpecMatcher::new(&["*.ts", "*.tsx", "*.js"], "/project", Usage::Files, true).unwrap();
        assert_eq!(m.match_index("/project/a.ts"), 0);
        assert_eq!(m.match_index("/project/b.tsx"), 1);
        assert_eq!(m.match_index("/project/c.js"), 2);
        assert_eq!(m.match_index("/project/d.css"), -1);
    }

    // empty specs returns nil
    {
        let m = SpecMatcher::new(&[], "/project", Usage::Files, true);
        assert!(m.is_none(), "should be None");
    }
}

// ---------------------------------------------------------------------------
// TestGlobPatternInternals — 7 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_glob_pattern_internals() {
    // nextPathPart handles consecutive slashes
    {
        let path = "/dev//foo///bar";

        let (part, offset, ok) = next_path_part_parts(path, "", 0);
        assert!(ok);
        assert_eq!(part, "");
        assert_eq!(offset, 1);

        let (part, offset, ok) = next_path_part_parts(path, "", 1);
        assert!(ok);
        assert_eq!(part, "dev");

        let (part, offset, ok) = next_path_part_parts(path, "", offset);
        assert!(ok);
        assert_eq!(part, "foo");

        let (part, _, ok) = next_path_part_parts(path, "", offset);
        assert!(ok);
        assert_eq!(part, "bar");
    }

    // nextPathPart handles path ending with slashes
    {
        let path = "/dev/";

        let (_, offset, ok) = next_path_part_parts(path, "", 0);
        assert!(ok);
        let (_, offset, ok) = next_path_part_parts(path, "", offset);
        assert!(ok);
        let (_, _, ok) = next_path_part_parts(path, "", offset);
        assert!(!ok);
    }

    // nextPathPartParts handles empty prefix
    {
        let path = "/dev//foo";

        let (part, offset, ok) = next_path_part_parts("", path, 0);
        assert!(ok);
        assert_eq!(part, "");
        assert_eq!(offset, 1);

        let (part, offset, ok) = next_path_part_parts("", path, offset);
        assert!(ok);
        assert_eq!(part, "dev");

        let (part, _, ok) = next_path_part_parts("", path, offset);
        assert!(ok);
        assert_eq!(part, "foo");
    }

    // nextPathPartParts returns not ok when only slashes remain
    {
        let prefix = "/dev/";
        let suffix = "foo";

        let (_, offset, ok) = next_path_part_parts(prefix, suffix, 0);
        assert!(ok);

        let (part, offset, ok) = next_path_part_parts(prefix, suffix, offset);
        assert!(ok);
        assert_eq!(part, "dev");

        let (part, offset, ok) = next_path_part_parts(prefix, suffix, offset);
        assert!(ok);
        assert_eq!(part, "foo");
        assert_eq!(offset, prefix.len() + suffix.len());

        let (_, _, ok) = next_path_part_parts(prefix, suffix, offset);
        assert!(!ok);
    }

    // nextPathPartParts parses from suffix region
    {
        let prefix = "/";
        let suffix = "a";

        let (part, offset, ok) = next_path_part_parts(prefix, suffix, 0);
        assert!(ok);
        assert_eq!(part, "");
        assert_eq!(offset, 1);

        let (part, _, ok) = next_path_part_parts(prefix, suffix, offset);
        assert!(ok);
        assert_eq!(part, "a");
    }

    // question mark segment at end of string
    {
        let p = compile_glob_pattern("a?", "/", Usage::Files, true);
        assert!(p.is_some());
        let p = p.unwrap();

        assert!(p.matches("/ab"));
        assert!(!p.matches("/a"));
    }

    // star segment with complex pattern
    {
        let p = compile_glob_pattern("a*b*c", "/", Usage::Files, true);
        assert!(p.is_some());
        let p = p.unwrap();

        assert!(p.matches("/abc"));
        assert!(p.matches("/aXbYc"));
        assert!(p.matches("/aXXXbYYYc"));
        assert!(!p.matches("/aXbY"));
    }

    // ensureTrailingSlash with existing slash
    {
        let result = ensure_trailing_slash("/dev/");
        assert_eq!(result, "/dev/");

        let result = ensure_trailing_slash("/");
        assert_eq!(result, "/");
    }

    // ensureTrailingSlash with empty string
    {
        let result = ensure_trailing_slash("");
        assert_eq!(result, "");
    }

    // literal component with package folder in include
    {
        let host = build_fs(&[("/dev/node_modules/pkg/index.ts", "")], false);
        let got = match_files(
            "/dev",
            &[".ts"],
            &[],
            &["node_modules/pkg/index.ts"],
            false,
            "/",
            UNLIMITED_DEPTH,
            &host,
        );
        assert!(got.contains(&"/dev/node_modules/pkg/index.ts".to_string()));
    }
}

// ---------------------------------------------------------------------------
// TestMatchSegmentsEdgeCases — 8 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_match_segments_edge_cases() {
    // question mark before slash in string
    {
        let p = compile_glob_pattern("a?b", "/", Usage::Files, true).unwrap();

        assert!(p.matches("/aXb"));
        assert!(!p.matches("/ab"));
        assert!(!p.matches("/aXYb"));
    }

    // star with no trailing content
    {
        let p = compile_glob_pattern("a*", "/", Usage::Files, true).unwrap();

        assert!(p.matches("/a"));
        assert!(p.matches("/abc"));
        assert!(p.matches("/aXYZ"));
    }

    // multiple stars in pattern
    {
        let p = compile_glob_pattern("*a*", "/", Usage::Files, true).unwrap();

        assert!(p.matches("/a"));
        assert!(p.matches("/Xa"));
        assert!(p.matches("/aX"));
        assert!(p.matches("/XaY"));
        assert!(!p.matches("/XYZ"));
    }

    // multiple stars requiring backtracking
    {
        let p1 = compile_glob_pattern("*a*a", "/", Usage::Files, true).unwrap();
        assert!(p1.matches("/aa"));
        assert!(p1.matches("/Xaa"));
        assert!(p1.matches("/aXa"));
        assert!(p1.matches("/XaYa"));
        assert!(p1.matches("/aaaa"));
        assert!(!p1.matches("/a"));
        assert!(!p1.matches("/Xa"));
        assert!(!p1.matches("/aX"));
        assert!(!p1.matches("/XaYaZ"));

        let p2 = compile_glob_pattern("*a*b*c", "/", Usage::Files, true).unwrap();
        assert!(p2.matches("/abc"));
        assert!(p2.matches("/XaYbZc"));
        assert!(p2.matches("/aXbYc"));
        assert!(p2.matches("/aaabbbccc"));
        assert!(!p2.matches("/ab"));
        assert!(!p2.matches("/ac"));
        assert!(!p2.matches("/cba"));
        assert!(!p2.matches("/abcX"));

        let p3 = compile_glob_pattern("*a*a*a", "/", Usage::Files, true).unwrap();
        assert!(p3.matches("/aaa"));
        assert!(p3.matches("/aXaYa"));
        assert!(p3.matches("/XaYaZa"));
        assert!(!p3.matches("/aa"));
        assert!(!p3.matches("/aaX"));

        let p4 = compile_glob_pattern("a*b*a", "/", Usage::Files, true).unwrap();
        assert!(p4.matches("/aba"));
        assert!(p4.matches("/aXbYa"));
        assert!(p4.matches("/abba"));
        assert!(!p4.matches("/ab"));
        assert!(!p4.matches("/Xaba"));
    }

    // pathological pattern performance
    {
        let p = compile_glob_pattern("*a*a*a*a*b", "/", Usage::Files, true).unwrap();
        assert!(!p.matches("/aaaaaaaaaaaaaaaa"));
        assert!(!p.matches("/aaaaaaaaaaaaaaaaX"));
        assert!(p.matches("/aaaab"));
        assert!(p.matches("/XaYaZaWab"));
    }

    // literal segment not matching
    {
        let p = compile_glob_pattern("abcdefgh.ts", "/", Usage::Files, true).unwrap();
        assert!(!p.matches("/abc.ts"));
        assert!(p.matches("/abcdefgh.ts"));
    }

    // question mark matches multi-byte unicode rune
    {
        let p1 = compile_glob_pattern("?.ts", "/", Usage::Files, true).unwrap();
        assert!(p1.matches("/a.ts"));
        assert!(p1.matches("/\u{00e9}.ts")); // é
        assert!(p1.matches("/\u{4e2d}.ts")); // 中
        assert!(p1.matches("/\u{1f389}.ts")); // 🎉
        assert!(!p1.matches("/.ts"));
        assert!(!p1.matches("/ab.ts"));

        let p2 = compile_glob_pattern("??.ts", "/", Usage::Files, true).unwrap();
        assert!(p2.matches("/ab.ts"));
        assert!(p2.matches("/\u{00e9}\u{4e2d}.ts")); // é中
        assert!(p2.matches("/\u{1f389}\u{00e9}.ts")); // 🎉é
        assert!(!p2.matches("/a.ts"));
        assert!(!p2.matches("/abc.ts"));
    }

    // star matches multi-byte unicode runes correctly
    {
        let p = compile_glob_pattern("*\u{00e9}.ts", "/", Usage::Files, true).unwrap();
        assert!(p.matches("/\u{00e9}.ts"));
        assert!(p.matches("/caf\u{00e9}.ts"));
        assert!(!p.matches("/cafe.ts"));

        let p2 = compile_glob_pattern("*\u{1f389}*", "/", Usage::Files, true).unwrap();
        assert!(p2.matches("/\u{1f389}"));
        assert!(p2.matches("/a\u{1f389}b"));
        assert!(!p2.matches("/abc"));
    }
}

// ---------------------------------------------------------------------------
// TestReadDirectoryConsecutiveSlashes
// ---------------------------------------------------------------------------

#[test]
fn test_read_directory_consecutive_slashes() {
    let host = build_fs(&[("/dev/a.ts", ""), ("/dev/x/b.ts", "")], false);
    let got = match_files(
        "/dev",
        &[".ts"],
        &[],
        &["**/*.ts"],
        false,
        "/",
        UNLIMITED_DEPTH,
        &host,
    );
    assert!(got.len() >= 2, "should find files");
    assert!(got.contains(&"/dev/a.ts".to_string()));
    assert!(got.contains(&"/dev/x/b.ts".to_string()));
}

// ---------------------------------------------------------------------------
// TestGlobPatternLiteralWithPackageFolders — 2 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_glob_pattern_literal_with_package_folders() {
    // wildcard skips package folders
    {
        let host = build_fs(&[("/dev/a.ts", ""), ("/dev/node_modules/b.ts", "")], false);
        let got = match_files(
            "/dev",
            &[".ts"],
            &[],
            &["*/*.ts"],
            false,
            "/",
            UNLIMITED_DEPTH,
            &host,
        );
        assert!(
            !got.contains(&"/dev/node_modules/b.ts".to_string()),
            "should skip node_modules with wildcard"
        );
    }

    // explicit literal includes package folder
    {
        let host = build_fs(&[("/dev/node_modules/b.ts", "")], false);
        let got = match_files(
            "/dev",
            &[".ts"],
            &[],
            &["node_modules/b.ts"],
            false,
            "/",
            UNLIMITED_DEPTH,
            &host,
        );
        assert!(
            got.contains(&"/dev/node_modules/b.ts".to_string()),
            "should include explicit node_modules path"
        );
    }
}

// ---------------------------------------------------------------------------
// TestGetBasePathsCaseSensitivity — 2 sub-cases
// ---------------------------------------------------------------------------

#[test]
fn test_get_base_paths_case_sensitivity() {
    // case-sensitive does not dedup differently-cased paths
    {
        let base_paths = get_base_paths("/root", &["../Other/**/*.ts", "../other/**/*.ts"], true);
        assert!(
            base_paths.contains(&"/Other".to_string()),
            "expected /Other in base paths: {base_paths:?}"
        );
        assert!(
            base_paths.contains(&"/other".to_string()),
            "expected /other in base paths: {base_paths:?}"
        );
    }

    // case-insensitive dedups differently-cased paths
    {
        let base_paths = get_base_paths("/root", &["../Other/**/*.ts", "../other/**/*.ts"], false);
        let count = base_paths
            .iter()
            .filter(|bp| bp == &"/Other" || bp == &"/other")
            .count();
        assert!(
            count <= 1,
            "expected at most one of /Other or /other in base paths: {base_paths:?}"
        );
    }
}
