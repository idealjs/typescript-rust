use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_js_doc() {
    let content = r#"// @allowJs: true
// @Filename: foo.js
/** @typedef {(number|string)} NumberLike */
/** @typedef {(string|number)} */
const x = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
