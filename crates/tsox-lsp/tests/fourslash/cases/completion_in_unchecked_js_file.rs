use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_unchecked_js_file() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @checkJs: false
// @Filename: index.js
function hello() {

}

const goodbye = 5;

console./*0*/"#;
    let mut s = Session::new_for_test("completionInUncheckedJSFile", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
}
