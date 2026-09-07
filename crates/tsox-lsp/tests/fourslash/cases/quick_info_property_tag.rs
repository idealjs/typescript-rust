use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_property_tag() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @typedef I
 * @property {number} x Doc
 *                      More doc
 */

/** @type {I} */
const obj = { /**/x: 10 };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(property) x: number", "Doc\nMore doc")
}
