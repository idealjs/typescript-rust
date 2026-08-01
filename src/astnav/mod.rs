//! AST navigation, ported from Go `internal/astnav` (`tokens.go`).
//!
//! Provides token-position lookup utilities used by the language service:
//! - `get_token_at_position` — token at a given position
//! - `get_touching_property_name` — property-name token touching a position
//! - `find_preceding_token` — leftmost token with `position < token.end()`
//! - `find_next_token` — token starting immediately after a given token
//!
//! NOTE: These functions are **not yet implemented** in Rust. (The LSP module
//! has a simpler `find_deepest_node` helper that overlaps in purpose but does
//! not cover scanner-synthesized tokens, JSDoc handling, or the full
//! binary-search navigation logic.)
//!
//! The test stubs below document the Go test data from
//! `internal/astnav/tokens_test.go` and are marked `#[ignore]` until the
//! implementation lands. They mirror `tokens.go` so the behaviour contract is
//! captured up front.
//!
//! `TestMain` (Go `testmain_test.go`) only applies `core.ApplyDebugStackLimit`
//! and baseline tracking — it has no Rust equivalent and is intentionally not
//! ported as a test.

#[cfg(test)]
mod tests {
    use crate::ast::SyntaxKind;

    // -------------------------------------------------------------------------
    // TestGetTokenAtPosition
    //
    // Go: 3 concrete sub-tests + 2 baseline sub-tests (require Node.js + the
    // TypeScript submodule). Only the concrete sub-tests are documented here.
    // -------------------------------------------------------------------------

    /// Go sub-test "JSDoc type assertion".
    ///
    /// Source file (`ScriptKind::Js`):
    /// ```text
    /// function foo(x) {
    ///     const s = /**@type {string}*/(x)
    /// }
    /// ```
    /// `get_touching_property_name(file, 52)` must not panic. Previously
    /// panicked with "did not expect KindParenthesizedExpression to have
    /// KindIdentifier in its trivia". The returned token kind must be
    /// `Identifier` or `ParenthesizedExpression`.
    #[test]
    #[ignore = "astnav::get_touching_property_name not yet ported to Rust"]
    fn get_token_at_position_jsdoc_type_assertion() {
        // file_text at position 52 ('x' inside the parenthesised expression)
        let _position: usize = 52;
        // expected: token != None, kind in {Identifier, ParenthesizedExpression}
    }

    /// Go sub-test "JSDoc type assertion with comment".
    ///
    /// Source file (`ScriptKind::Js`):
    /// ```text
    /// function foo(x) {
    ///     const s = /**@type {string}*/(x)  // Go-to-definition on x causes panic
    /// }
    /// ```
    /// `get_touching_property_name(file, 52)` must not panic and must return a
    /// token.
    #[test]
    #[ignore = "astnav::get_touching_property_name not yet ported to Rust"]
    fn get_token_at_position_jsdoc_type_assertion_with_comment() {
        let _x_pos: usize = 52; // position of 'x' in (x)
        // expected: token != None
    }

    /// Go sub-test "pointer equality".
    ///
    /// Source file (`ScriptKind::Ts`):
    /// ```text
    ///
    /// \t\t\tfunction foo() {
    /// \t\t\t\treturn 0;
    /// \t\t\t}
    /// ```
    /// Two calls to `get_token_at_position(file, 0)` must return the *same*
    /// node (pointer-equal in Go; `Arc::ptr_eq` / identical index in Rust).
    #[test]
    #[ignore = "astnav::get_token_at_position not yet ported to Rust"]
    fn get_token_at_position_pointer_equality() {
        // assert same node returned for repeated calls at position 0
    }

    /// Go baseline sub-tests ("baseline" + "go baseline json") iterate every
    /// position in `testFiles` (the TypeScript submodule's
    /// `src/services/mapCode.ts`) and compare Go output against the Node.js
    /// TypeScript oracle. These require Node.js + the TS submodule and are not
    /// portable as unit tests.
    #[test]
    #[ignore = "requires Node.js oracle + TypeScript submodule baseline"]
    fn get_token_at_position_baseline() {}

    // -------------------------------------------------------------------------
    // TestGetTouchingPropertyName
    //
    // Go: baseline-only test (requires Node.js + TS submodule).
    // -------------------------------------------------------------------------

    /// Go `TestGetTouchingPropertyName`. Baseline parity test against the
    /// Node.js TypeScript oracle over `src/services/mapCode.ts`.
    #[test]
    #[ignore = "astnav::get_touching_property_name not yet ported; baseline needs Node.js oracle"]
    fn get_touching_property_name_baseline() {}

    // -------------------------------------------------------------------------
    // TestFindPrecedingToken
    //
    // Go: baseline-only test (requires Node.js + TS submodule).
    // -------------------------------------------------------------------------

    /// Go `TestFindPrecedingToken`. Baseline parity test (includes EOF
    /// position) against the Node.js TypeScript oracle over
    /// `src/services/mapCode.ts`.
    #[test]
    #[ignore = "astnav::find_preceding_token not yet ported; baseline needs Node.js oracle"]
    fn find_preceding_token_baseline() {}

    // -------------------------------------------------------------------------
    // TestFindNextToken
    //
    // Go: "go baseline json" only. Uses `get_token_at_position` then
    // `find_next_token`; the Go test recovers from panics where the scanner
    // finds trivia between `previousToken.end()` and the next syntactic token.
    // -------------------------------------------------------------------------

    /// Go `TestFindNextToken`. Baseline test that walks every position, gets the
    /// token at that position, then finds the next token. Positions that would
    /// cause the scanner to panic are recorded as `None`.
    #[test]
    #[ignore = "astnav::find_next_token not yet ported; baseline needs Node.js oracle"]
    fn find_next_token_baseline() {}

    // -------------------------------------------------------------------------
    // TestUnitFindPrecedingToken
    //
    // Go: table-driven unit test with concrete inputs. This is the most
    // directly portable test; the two cases are captured below.
    // -------------------------------------------------------------------------

    /// Go `TestUnitFindPrecedingToken`, case "after dot in jsdoc".
    ///
    /// `find_preceding_token(file, 839)` must return a `DotToken`.
    /// The file content is a large TS snippet ending in
    /// `backslashRegExp.` followed by a blank line and a JSDoc-commented
    /// `isAnyDirectorySeparator` function (see Go source for full text).
    #[test]
    #[ignore = "astnav::find_preceding_token not yet ported to Rust"]
    fn find_preceding_token_after_dot_in_jsdoc() {
        let _position: usize = 839;
        let _expected = SyntaxKind::DotToken;
        // Full file content is in Go tokens_test.go TestUnitFindPrecedingToken.
    }

    /// Go `TestUnitFindPrecedingToken`, case "after comma in parameter list".
    ///
    /// Source file (`ScriptKind::Ts`): `takesCb((n, s, ))`
    /// `find_preceding_token(file, 15)` must return a `CommaToken`.
    #[test]
    #[ignore = "astnav::find_preceding_token not yet ported to Rust"]
    fn find_preceding_token_after_comma_in_parameter_list() {
        // file_content: "takesCb((n, s, ))"
        let _position: usize = 15;
        let _expected = SyntaxKind::CommaToken;
    }
}
