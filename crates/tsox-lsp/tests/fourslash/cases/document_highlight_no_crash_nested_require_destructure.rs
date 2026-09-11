use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_nested_require_destructure_no_crash1() {
    let content = r#"// @allowJs: true
// @Filename: /bar.js
const { a: { b } } = require('./foo');
/**/b;"#;
    let mut s = Session::new_for_test("documentHighlightNestedRequireDestructureNoCrash1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
