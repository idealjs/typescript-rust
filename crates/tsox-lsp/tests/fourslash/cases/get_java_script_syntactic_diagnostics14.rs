use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics14() {
    let content = r#"// @allowJs: true
// @Filename: a.js
Foo<number>();"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics14", content);
    fourslash::unsupported("VerifyBaselineNonSuggestionDiagnostics"); // f.VerifyBaselineNonSuggestionDiagnostics(t)
}
