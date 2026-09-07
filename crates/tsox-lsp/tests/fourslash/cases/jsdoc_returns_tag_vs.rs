use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn jsdoc_returns_tag_vs() {
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
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
