use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics5() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C implements D { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics5", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
