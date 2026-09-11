use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_jsdoc_type_tag_cast() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
const x = /** @type {{ s: string }} */ ({ /**/ });"#;
    let mut s = Session::new_for_test("completionsJsdocTypeTagCast", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
