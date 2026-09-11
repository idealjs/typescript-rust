use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics7() {
    let content = r#"// @allowJs: true
// @Filename: a.js
namespace M { }"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics7", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
