use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_unique_symbol_js_doc() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @filename: ./a.js
/** @type {unique symbol} */
const foo = Symbol();
foo/**/"#;
    let mut s = Session::new_for_test("quickInfoUniqueSymbolJsDoc", content);
    // TODO: f.VerifyBaselineHover(t)
}
