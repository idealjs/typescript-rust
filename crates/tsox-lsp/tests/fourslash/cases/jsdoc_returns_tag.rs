use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_returns_tag() {
    let content = r#"// @allowJs: true
// @Filename: dummy.js
/**
 * Find an item
 * @template T
 * @param {T[]} l
 * @param {T} x
 * @returns {?T}  The names of the found item(s).
 */
function find(l, x) {
}
find(''/**/);"#;
    let mut s = Session::new_for_test("jsdocReturnsTag", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
