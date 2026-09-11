use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics1() {
    let content = r#"// @allowJs: true
// @Filename: a.js
import a = b;"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics1", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
