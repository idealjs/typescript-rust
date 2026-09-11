use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics9() {
    let content = r#"// @allowJs: true
// @Filename: a.js
public function F() { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics9", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
