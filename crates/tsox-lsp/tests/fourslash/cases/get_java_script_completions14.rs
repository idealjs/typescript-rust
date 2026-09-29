use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions14() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file1.js
interface Number {
    toExponential(fractionDigits?: number): string;
}
var x = 1;
x./*1*/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions14", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
