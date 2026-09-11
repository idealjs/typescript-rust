use tsox_lsp::fourslash::{self, Session};


#[test]
fn navbar_nested_common_js_exports() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
exports.a = exports.b = exports.c = 0;"#;
    let mut s = Session::new_for_test("navbarNestedCommonJsExports", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
