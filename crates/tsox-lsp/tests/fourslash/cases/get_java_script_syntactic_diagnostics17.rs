use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics17() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F(a: number) { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics17", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
