use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics11() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F(): number { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics11", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
