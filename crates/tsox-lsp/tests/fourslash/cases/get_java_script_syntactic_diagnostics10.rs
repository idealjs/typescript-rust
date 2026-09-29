use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics10() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function F<T>() { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics10", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
