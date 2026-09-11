use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics11() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F(): number { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics11", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
