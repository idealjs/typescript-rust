use tsox_lsp::fourslash::{self, Session};


#[test]
fn double_underscore_completions() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: a.js
function MyObject(){
    this.__property = 1;
}
var instance = new MyObject();
instance./*1*/"#;
    let mut s = Session::new_for_test("doubleUnderscoreCompletions", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
