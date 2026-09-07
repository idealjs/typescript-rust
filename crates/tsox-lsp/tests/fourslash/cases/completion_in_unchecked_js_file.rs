use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
}
