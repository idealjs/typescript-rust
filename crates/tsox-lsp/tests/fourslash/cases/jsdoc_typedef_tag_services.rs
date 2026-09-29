use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("jsdocTypedefTagServices", content);
    fourslash::verify_quick_info_at(&mut s, "use", "type Product = {\n    title: string;\n}", "Doc comment");
    // TODO: f.VerifyBaselineFindAllReferences(t, "use", "def")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges()[1:])...)
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use")
}
