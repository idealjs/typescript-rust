use tsox_lsp::fourslash::Session;


#[test]
fn refactor_convert_to_es_module_not_at_top_level() {
    let content = r#"// @allowJs: true
// @target: esnext
// @Filename: /a.js
(function() {
    module.exports = 0;
})();"#;
    let _s = Session::new_for_test("refactorConvertToEsModule_notAtTopLevel", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
