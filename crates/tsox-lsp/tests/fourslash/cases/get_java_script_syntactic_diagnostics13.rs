use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics13() {
    let content = r#"// @allowJs: true
// @Filename: a.js
var v: () => number;"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics13", content);
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
