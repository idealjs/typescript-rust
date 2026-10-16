use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics12() {
    let content = r#"// @allowJs: true
// @Filename: a.js
declare var v;"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics12", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
