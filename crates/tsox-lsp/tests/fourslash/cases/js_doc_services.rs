use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn js_doc_services() {
    let content = r#"interface /*I*/I {}

/**
 * @param /*use*/[|foo|] I pity the foo
 */
function f([|[|/*def*/{| "contextRangeIndex": 1 |}foo|]: I|]) {
    return /*use2*/[|foo|];
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "use");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(parameter) foo: I", "I pity the foo")
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "use", "def", "use2")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[0], f.Ranges()[2], f.Ranges()[3])
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0], f.Ranges()[2], f.Ranges()[
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "use")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "use")
}
