use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions21() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
class Prv {
    #privatething = 1;
    notSoPrivate = 1;
}
new Prv()['[|/**/|]'];"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions21", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
