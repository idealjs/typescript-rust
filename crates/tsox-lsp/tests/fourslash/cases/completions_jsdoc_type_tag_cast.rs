use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_jsdoc_type_tag_cast() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: /a.js
const x = /** @type {{ s: string }} */ ({ /**/ });"#;
    let mut s = Session::new_for_test("completionsJsdocTypeTagCast", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
