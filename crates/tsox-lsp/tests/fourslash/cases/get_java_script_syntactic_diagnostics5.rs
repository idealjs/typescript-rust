use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics5() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C implements D { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics5", content);
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
