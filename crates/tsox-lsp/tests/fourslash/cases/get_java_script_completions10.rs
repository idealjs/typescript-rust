use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_completions10() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @type {function(this:number)}
 */
function f() { this./**/ }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
