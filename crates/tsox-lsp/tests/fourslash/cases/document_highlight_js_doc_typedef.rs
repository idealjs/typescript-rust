use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
