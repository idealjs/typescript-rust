use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
