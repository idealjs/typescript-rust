use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics15() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F(public p) { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics15", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
