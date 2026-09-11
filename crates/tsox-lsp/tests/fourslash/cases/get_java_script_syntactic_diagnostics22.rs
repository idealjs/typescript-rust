use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics22() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function foo(...a) {}"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics22", content);
    // TODO: f.VerifyNonSuggestionDiagnostics(t, nil)
}
