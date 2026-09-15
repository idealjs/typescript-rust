use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_for_js_doc_with_http_links() {
    let content = r#"// @checkJs: true
// @filename: quickInfoForJSDocWithHttpLinks.js
/** @typedef {number} /*1*/https://wat */

/**
* @typedef {Object} Oops
* @property {number} /*2*/https://wass
*/


/** @callback /*3*/http://vad */

/** @see https://hvad */
var /*4*/see1 = true

/** @see {@link https://hva} */
var /*5*/see2 = true

/** {@link https://hvaD} */
var /*6*/see3 = true"#;
    let _s = Session::new_for_test("quickInfoForJSDocWithHttpLinks", content);
    // TODO: f.VerifyBaselineHover(t)
}
