use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics3() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C<T> { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics3", content);
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
