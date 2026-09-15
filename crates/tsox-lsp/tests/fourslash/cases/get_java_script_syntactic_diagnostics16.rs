use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics16() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F(p?) { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics16", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
