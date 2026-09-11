use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_object_literal_module_exports() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: index.js
const almanac = 0;
module.exports = {
  a/**/
};"#;
    let mut s = Session::new_for_test("completionsObjectLiteralModuleExports", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
