use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlights_export_equals_in_merged_namespace() {
    let content = r#"
class C {}
namespace C {
    /*marker*/export = C;
}
"#;
    let mut s = Session::new_for_test("documentHighlightsExportEqualsInMergedNamespace", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "marker")
}
