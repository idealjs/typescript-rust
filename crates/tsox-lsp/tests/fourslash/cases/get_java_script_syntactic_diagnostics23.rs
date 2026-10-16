use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics23() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function Person(age) {
    if (age >= 18) {
        this.canVote = true;
    } else {
        this.canVote = false;
    }
}"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics23", content);
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
}
