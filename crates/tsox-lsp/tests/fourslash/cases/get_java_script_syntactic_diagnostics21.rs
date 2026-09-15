use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics21() {
    let content = r#"// @allowJs: true
// @experimentalDecorators: true
// @Filename: a.js
@internal class C {}"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics21", content);
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
}
