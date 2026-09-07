use tsox_lsp::fourslash::{self, Session};

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
    fourslash::verify_quick_info_at(&mut s, "", "(property) x: number", "Doc\nMore doc");
}
