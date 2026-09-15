use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_unique_symbol_js_doc() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @filename: ./a.js
/** @type {unique symbol} */
const foo = Symbol();
foo/**/"#;
    let _s = Session::new_for_test("quickInfoUniqueSymbolJsDoc", content);
    // TODO: f.VerifyBaselineHover(t)
}
