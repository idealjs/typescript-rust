use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics3() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C<T> { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics3", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
