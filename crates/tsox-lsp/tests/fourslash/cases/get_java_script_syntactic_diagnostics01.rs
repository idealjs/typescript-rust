use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics01() {
    let content = r#"// @lib: es5
// @allowJs: true
// @Filename: a.js
var ===;"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
