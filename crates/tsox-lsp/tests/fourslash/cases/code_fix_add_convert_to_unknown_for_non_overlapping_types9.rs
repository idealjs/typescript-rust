use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_convert_to_unknown_for_non_overlapping_types9() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @filename: a.js
let x = /** @type {string} */ (100);"#;
    let mut s = Session::new_for_test("codeFixAddConvertToUnknownForNonOverlappingTypes9", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "Add 'unknown' conversion for non-overlapping types")
}
