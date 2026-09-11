use tsox_lsp::fourslash::{self, Session};


#[test]
fn refactor_convert_to_es_module_not_in_commonjs_project() {
    let content = r#"// @allowJs: true
// @target: es5
// @Filename: /a.js
exports.x = 0;"#;
    let mut s = Session::new_for_test("refactorConvertToEsModule_notInCommonjsProject", content);
    // TODO: f.VerifySuggestionDiagnostics(t, nil)
}
