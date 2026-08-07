//! Test utilities for integration testing — ported from typescript-go's
//! `internal/testutil/` package.
//!
//! Provides:
//! - [`test_case_parser`] — parses `// @FileName` / `// @Option` multi-file test cases
//! - [`baseline`] — golden-file comparison with accept workflow

pub mod baseline;
pub mod test_case_parser;
