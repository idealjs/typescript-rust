use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_js_doc() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
/** @typedef {(number|string)} NumberLike */
/** @typedef {(string|number)} */
const x = 0;"#;
    let mut s = Session::new_for_test("navigationBarJsDoc", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
