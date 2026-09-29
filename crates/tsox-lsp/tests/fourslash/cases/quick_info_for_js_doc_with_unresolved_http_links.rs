use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_for_js_doc_with_unresolved_http_links() {
    let content = r#"// @checkJs: true
// @filename: quickInfoForJSDocWithHttpLinks.js
/** @see {@link https://hva} */
var /*5*/see2 = true

/** {@link https://hvaD} */
var /*6*/see3 = true"#;
    let _s = Session::new_for_test("quickInfoForJSDocWithUnresolvedHttpLinks", content);
    // TODO: f.VerifyBaselineHover(t)
}
