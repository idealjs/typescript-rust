use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_element_access_declaration() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @Filename: a.js
const mod = {};
mod["@@thing1"] = {};
mod["/**/@@thing1"]["@@thing2"] = 0;"#;
    let mut s = Session::new_for_test("quickInfoElementAccessDeclaration", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "module mod[\"@@thing1\"]\n(property) mod[\"@@thing1\"]: typeof mod.@@thing1"
}
