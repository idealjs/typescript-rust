use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_syntactic_diagnostics2() {
    let content = r#"// @allowJs: true
// @Filename: a.js
export = b;"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics2", content);
    // TODO: f.VerifyBaselineNonSuggestionDiagnostics(t)
}
