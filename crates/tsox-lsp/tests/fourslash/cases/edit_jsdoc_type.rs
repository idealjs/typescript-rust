use tsox_lsp::fourslash::{self, Session};


#[test]
fn edit_jsdoc_type() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @noLib: true
// @Filename: /a.js
/** @type/**/ */
const x = 0;"#;
    let mut s = Session::new_for_test("editJsdocType", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "", "")
    fourslash::insert(&mut s, " ");
    // TODO: f.VerifyQuickInfoIs(t, "", "")
}
