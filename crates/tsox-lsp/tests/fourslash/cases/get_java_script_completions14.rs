use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions14() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file1.js
interface Number {
    toExponential(fractionDigits?: number): string;
}
var x = 1;
x./*1*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
