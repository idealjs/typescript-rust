use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics1() {
    let content = r#"// @allowJs: true
// @Filename: a.js
import a = b;"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics1", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
