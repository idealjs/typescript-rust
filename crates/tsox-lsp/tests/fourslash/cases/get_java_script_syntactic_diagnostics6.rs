use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics6() {
    let content = r#"// @allowJs: true
// @Filename: a.js
interface I { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics6", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
