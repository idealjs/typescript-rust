use tsox_lsp::fourslash::Session;


#[test]
fn get_java_script_syntactic_diagnostics18() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class C {
    x; // Regular property declaration allowed
    static y; // static allowed
    public z; // public not allowed
}
// @Filename: b.js
class C {
    x: number; // Types not allowed
}"#;
    let _s = Session::new_for_test("getJavaScriptSyntacticDiagnostics18", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
