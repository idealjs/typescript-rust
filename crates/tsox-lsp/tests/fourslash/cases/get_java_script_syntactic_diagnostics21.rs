use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics21() {
    let content = r#"// @allowJs: true
// @experimentalDecorators: true
// @Filename: a.js
@internal class C {}"#;
    let mut s = Session::new_for_test("getJavaScriptSyntacticDiagnostics21", content);
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
}
