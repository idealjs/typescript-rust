use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics14() {
    let content = r#"// @allowJs: true
// @Filename: a.js
Foo<number>();"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics14", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
