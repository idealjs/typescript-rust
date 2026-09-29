use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics14() {
    let content = r#"// @allowJs: true
// @Filename: a.js
Foo<number>();"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics14", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
