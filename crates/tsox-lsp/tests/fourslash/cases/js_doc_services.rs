use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_services() {
    let content = r#"interface /*I*/I {}

/**
 * @param /*use*/[|foo|] I pity the foo
 */
function f([|[|/*def*/{| "contextRangeIndex": 1 |}foo|]: I|]) {
    return /*use2*/[|foo|];
}"#;
    let mut s = Session::new_for_test("jsDocServices", content);
    fourslash::go_to_marker(&mut s, "use");
    // TODO: f.VerifyQuickInfoIs(t, "(parameter) foo: I", "I pity the foo")
    // TODO: f.VerifyBaselineFindAllReferences(t, "use", "def", "use2")
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[0], f.Ranges()[2], f.Ranges()[3])
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0], f.Ranges()[2], f.Ranges()[
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "use")
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "use")
}
