use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn edit_jsdoc_type() {
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
