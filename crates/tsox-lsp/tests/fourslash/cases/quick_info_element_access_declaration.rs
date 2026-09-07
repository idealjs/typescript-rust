use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_element_access_declaration() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @checkJs: true
// @allowJs: true
// @Filename: a.js
const mod = {};
mod["@@thing1"] = {};
mod["/**/@@thing1"]["@@thing2"] = 0;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "module mod[\"@@thing1\"]\n(property) mod[\"@@thing1\"]: typeof mod.@@thing1"
}
