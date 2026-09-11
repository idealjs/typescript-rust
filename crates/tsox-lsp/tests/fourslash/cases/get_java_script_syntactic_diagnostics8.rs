use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics8() {
    let content = r#"// @allowJs: true
// @Filename: a.js
type a = b;"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics8", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
