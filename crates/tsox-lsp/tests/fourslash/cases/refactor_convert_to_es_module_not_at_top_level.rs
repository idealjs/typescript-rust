use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySuggestionDiagnostics"]
#[test]
fn refactor_convert_to_es_module_not_at_top_level() {
    let content = r#"// @allowJs: true
// @target: esnext
// @Filename: /a.js
(function() {
    module.exports = 0;
})();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySuggestionDiagnostics"); // f.VerifySuggestionDiagnostics(t, nil)
}
