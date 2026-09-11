use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_require_property_destructure1() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
const { a } = require("m").f;
a/*m*/;"#;
    let mut s = Session::new_for_test("documentHighlightRequirePropertyDestructure1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}
