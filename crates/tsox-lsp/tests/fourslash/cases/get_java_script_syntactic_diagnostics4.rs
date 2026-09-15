use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics4() {
    let content = r#"// @allowJs: true
// @Filename: a.js
public class C { }"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics4", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
