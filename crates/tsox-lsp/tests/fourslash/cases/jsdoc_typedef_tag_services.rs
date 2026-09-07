use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn jsdoc_typedef_tag_services() {
    let content = r#"// @allowJs: true
// @Filename: a.js
/**
 * Doc comment
 * [|@typedef /*def*/[|{| "contextRangeIndex": 0 |}Product|]
 * @property {string} title
 |]*/
/**
 * @type {[|/*use*/Product|]}
 */
const product = null;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "use", "type Product = {\n    title: string;\n}", "Doc comment")
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "use", "def")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "use")
}
