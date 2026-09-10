use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics16() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F(p?) { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics16", content);
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
