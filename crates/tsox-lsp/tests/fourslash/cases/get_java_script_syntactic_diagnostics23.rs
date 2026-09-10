use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics23() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function Person(age) {
    if (age >= 18) {
        this.canVote = true;
    } else {
        this.canVote = false;
    }
}"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics23", content);
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
}
