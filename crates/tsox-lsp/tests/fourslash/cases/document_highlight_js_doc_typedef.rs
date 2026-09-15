use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_js_doc_typedef() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: index.js
/**
 * @typedef {{
 *   [|foo|]: string;
 *   [|bar|]: number;
 * }} Foo
 */

/** @type {Foo} */
const x = {
  [|foo|]: "",
  [|bar|]: 42,
};"#;
    let _s = Session::new_for_test("documentHighlightJSDocTypedef", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
