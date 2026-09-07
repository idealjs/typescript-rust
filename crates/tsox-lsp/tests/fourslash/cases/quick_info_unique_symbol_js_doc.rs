use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_unique_symbol_js_doc() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @filename: ./a.js
/** @type {unique symbol} */
const foo = Symbol();
foo/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
