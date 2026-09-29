use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_jsdoc_typedef_missing_type() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @typedef /**/A
 */
var x;"#;
    let mut s = Session::new_for_test("quickInfoJsdocTypedefMissingType", content);
    fourslash::verify_quick_info_at(&mut s, "", "type A = any", "");
}
