use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics19() {
    let content = r#"// @allowJs: true
// @Filename: a.js
enum E { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics19", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
