use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNonSuggestionDiagnostics"]
#[test]
fn get_java_script_syntactic_diagnostics22() {
    let content = r#"// @allowJs: true
// @Filename: a.js
function foo(...a) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNonSuggestionDiagnostics"); // f.VerifyNonSuggestionDiagnostics(t, nil)
}
